use serde::{Deserialize, Serialize};
use tauri::Emitter;
use ethers::{
    prelude::*,
    providers::{Http, Provider, Middleware},
    types::{Address, U256, U64, TransactionRequest},
    utils::{format_ether, format_units, parse_ether, parse_units},
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware,
};
use std::sync::Arc;
use rand::Rng;
use crate::database::get_database_manager;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferConfig {
    pub chain: String,
    pub delay: [u64; 2],
    pub transfer_type: String, // "1", "2", "3", "4"
    pub transfer_amount: f64,
    pub transfer_amount_list: [f64; 2],
    pub left_amount_list: [f64; 2],
    pub amount_precision: u8,
    pub limit_type: String, // "1", "2", "3"
    pub limit_count: u64,
    pub limit_count_list: [u64; 2],
    pub gas_price_type: String, // "1", "2", "3"
    pub gas_price: f64,
    pub gas_price_rate: f64,
    pub max_gas_price: f64,
    pub error_retry: String,
    pub error_count_limit: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferItem {
    pub private_key: String,
    pub to_addr: String,
    pub error_msg: String,
    pub error_count: u32,
    pub retry_flag: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferResult {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
}

// RPC提供商信息
#[derive(Debug, Clone)]
pub struct RpcProvider {
    pub rpc_url: String,
    pub priority: i32,
    pub failure_count: i32,
    pub avg_response_time_ms: i32,
}

// RPC节点配置
pub struct RpcConfig {
    pub providers: Vec<RpcProvider>,
    pub chain_id: u64,
}

impl RpcConfig {
    // 基于优先级权重的负载均衡选择RPC
    pub fn get_random_rpc(&self) -> &str {
        if self.providers.is_empty() {
            panic!("No RPC providers available");
        }
        
        if self.providers.len() == 1 {
            return &self.providers[0].rpc_url;
        }
        
        // 计算每个提供商的权重
        let mut weights: Vec<f64> = Vec::new();
        
        for provider in &self.providers {
            // 基础权重：优先级越高（数值越小）权重越大
            let priority_weight = 1.0 / (provider.priority as f64 + 1.0);
            
            // 失败次数惩罚：失败次数越多权重越小
            let failure_penalty = 1.0 / (provider.failure_count as f64 + 1.0);
            
            // 响应时间惩罚：响应时间越长权重越小
            let response_time_penalty = 1.0 / (provider.avg_response_time_ms as f64 + 100.0);
            
            // 综合权重计算
            let weight = priority_weight * failure_penalty * response_time_penalty;
            weights.push(weight);
        }
        
        // 计算权重总和
        let total_weight: f64 = weights.iter().sum();
        
        // 生成随机数进行加权选择
         let mut rng = rand::thread_rng();
         let random_value = rng.gen_range(0.0..total_weight);
        
        let mut cumulative_weight = 0.0;
        for (i, weight) in weights.iter().enumerate() {
            cumulative_weight += weight;
            if random_value <= cumulative_weight {
                return &self.providers[i].rpc_url;
            }
        }
        
        // 如果由于浮点精度问题没有选中，返回最后一个
        &self.providers.last().unwrap().rpc_url
    }
}

// 从数据库获取RPC配置
pub async fn get_rpc_config(chain: &str) -> Option<RpcConfig> {
    let pool = get_database_manager().get_pool();
    
    // 首先获取链信息
    let chain_info = match sqlx::query(
        r#"
        SELECT id, chain_id FROM chains WHERE chain_key = ?
        "#
    )
    .bind(chain)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            eprintln!("Chain not found: {}", chain);
            return None;
        }
        Err(e) => {
            eprintln!("Database error when fetching chain: {}", e);
            return None;
        }
    };
    
    let db_chain_id: i64 = chain_info.get("id");
    let network_chain_id: i64 = chain_info.get("chain_id");
    
    // 获取该链的所有活跃RPC提供商，按优先级排序
    let rpc_providers = match sqlx::query(
        r#"
        SELECT rpc_url, priority, failure_count, avg_response_time_ms 
        FROM rpc_providers 
        WHERE chain_id = ? AND is_active = 1 
        ORDER BY priority ASC, failure_count ASC, avg_response_time_ms ASC
        "#
    )
    .bind(db_chain_id)
    .fetch_all(pool)
    .await
    {
        Ok(providers) => providers,
        Err(e) => {
            eprintln!("Database error when fetching RPC providers: {}", e);
            return None;
        }
    };
    
    if rpc_providers.is_empty() {
        eprintln!("No active RPC providers found for chain: {}", chain);
        return None;
    }
    
    // 构建RPC提供商列表
    let providers: Vec<RpcProvider> = rpc_providers
        .iter()
        .map(|row| RpcProvider {
            rpc_url: row.get::<String, _>("rpc_url"),
            priority: row.get::<i32, _>("priority"),
            failure_count: row.get::<i32, _>("failure_count"),
            avg_response_time_ms: row.get::<i32, _>("avg_response_time_ms"),
        })
        .collect();
    
    Some(RpcConfig {
        providers,
        chain_id: network_chain_id as u64,
    })
}

// 创建Provider
pub async fn create_provider(chain: &str) -> Result<Arc<Provider<Http>>, Box<dyn std::error::Error>> {
    let rpc_config = get_rpc_config(chain).await
        .ok_or(format!("不支持的链: {}", chain))?;
    
    let rpc_url = rpc_config.get_random_rpc();
    let provider = Provider::<Http>::try_from(rpc_url)?;
    
    Ok(Arc::new(provider))
}

// 转账工具函数
pub struct TransferUtils;

impl TransferUtils {


    // 获取Gas Price
    pub async fn get_gas_price(
        config: &TransferConfig,
        provider: Arc<Provider<Http>>,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        match config.gas_price_type.as_str() {
            "1" => {
                // 使用网络Gas Price
                let gas_price = provider.get_gas_price().await?;
                
                // 检查最大Gas Price限制
                if config.max_gas_price > 0.0 {
                    let gas_price_gwei = format_units(gas_price, "gwei")?.parse::<f64>()?;
                    if gas_price_gwei > config.max_gas_price {
                        return Err("base gas price 超出最大值限制".into());
                    }
                }
                
                Ok(gas_price)
            }
            "2" => {
                // 使用固定Gas Price
                Ok(parse_units(config.gas_price, "gwei")?.into())
            }
            "3" => {
                // 使用溢价Gas Price
                let base_gas_price = provider.get_gas_price().await?;
                
                
                // 安全地计算gas price rate，避免溢出
                let rate_percentage = config.gas_price_rate * 100.0;
                if rate_percentage < 0.0 || rate_percentage > f64::MAX / 2.0 {
                    println!("[ERROR] Gas price rate 值异常: {}", rate_percentage);
                    return Err(format!("Gas price rate 值异常: {}", rate_percentage).into());
                }
                
                // 使用U256进行安全计算，避免u64溢出
                let rate_u256 = U256::from((rate_percentage as u64).min(u64::MAX));
                let multiplier = U256::from(100) + rate_u256;
                let gas_price_with_rate = base_gas_price * multiplier / U256::from(100);
                
                
                // 检查最大Gas Price限制
                if config.max_gas_price > 0.0 {
                    let base_gas_price_gwei = format_units(base_gas_price, "gwei")?.parse::<f64>()?;
                    if base_gas_price_gwei > config.max_gas_price {
                        return Err("base gas price 超出最大值限制".into());
                    }
                    
                    let final_gas_price_gwei = format_units(gas_price_with_rate, "gwei")?.parse::<f64>()?;
                    if final_gas_price_gwei >= config.max_gas_price {
                        return Ok(parse_units(config.max_gas_price, "gwei")?.into());
                    }
                }
                
                Ok(gas_price_with_rate)
            }
            _ => Err("gas price type error".into()),
        }
    }

    // 获取Gas Limit
    pub async fn get_gas_limit(
        config: &TransferConfig,
        provider: Arc<Provider<Http>>,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        match config.limit_type.as_str() {
            "1" => {
                // 自动估算Gas Limit
                let tx = TransactionRequest::new()
                    .from(from)
                    .to(to)
                    .value(value);
                
                let gas_limit = provider.estimate_gas(&tx.into(), None).await?;
                Ok(gas_limit)
            }
            "2" => {
                // 使用固定Gas Limit
                Ok(U256::from(config.limit_count))
            }
            "3" => {
                // 使用随机Gas Limit
                let mut rng = rand::thread_rng();
                let gas_limit = rng.gen_range(config.limit_count_list[0]..=config.limit_count_list[1]);
                Ok(U256::from(gas_limit))
            }
            _ => Err("gas limit type error".into()),
        }
    }
}

// Tauri命令：基础币转账
#[tauri::command]
pub async fn base_coin_transfer<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    item: TransferItem,
    config: TransferConfig,
) -> Result<TransferResult, String> {
    match base_coin_transfer_internal(app_handle, index, item, config).await {
        Ok(tx_hash) => Ok(TransferResult {
            success: true,
            tx_hash: Some(tx_hash),
            error: None,
        }),
        Err(e) => Ok(TransferResult {
            success: false,
            tx_hash: None,
            error: Some(e.to_string()),
        }),
    }
}

// 内部基础币转账实现
async fn base_coin_transfer_internal<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    mut item: TransferItem,
    config: TransferConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    item.retry_flag = false;
    
    // 创建Provider
    let provider = create_provider(&config.chain).await?;
    
    // 创建钱包
    if item.private_key.trim().is_empty() {
        return Err("私钥不能为空！".into());
    }
    
    // 处理私钥格式，兼容带0x和不带0x的格式
    let private_key = if item.private_key.starts_with("0x") || item.private_key.starts_with("0X") {
        item.private_key[2..].to_string()
    } else {
        item.private_key.clone()
    };
    
    let wallet = private_key.parse::<LocalWallet>().map_err(|e| {
        format!("私钥格式错误: {}，请检查私钥格式是否正确（应为64位十六进制字符串，可带或不带0x前缀）", e)
    })?;
    let wallet = wallet.with_chain_id(get_rpc_config(&config.chain).await.unwrap().chain_id);
    let wallet_address = wallet.address();
    
    // 获取余额
    let balance = provider.get_balance(wallet_address, None).await?;
    let balance_ether = format_ether(balance);
    
    println!("序号：{}, 当前余额为: {} ETH", index, balance_ether);
    
    if balance.is_zero() {
        return Err("当前余额不足，不做转账操作！".into());
    }
    
    // 解析目标地址
    if item.to_addr.trim().is_empty() {
        return Err("目标地址不能为空，请先导入接收地址！".into());
    }
    let to_address: Address = item.to_addr.parse().map_err(|e| {
        format!("目标地址格式错误: {}，请检查地址格式是否正确", e)
    })?;
    
    // 获取Gas Price
    let gas_price = TransferUtils::get_gas_price(&config, provider.clone()).await?;
    
    // 计算转账金额
    let transfer_amount = match config.transfer_type.as_str() {
        "1" => {
            // 全部转账 - 需要预留Gas费用
            let gas_limit = TransferUtils::get_gas_limit(
                &config,
                provider.clone(),
                wallet_address,
                to_address,
                balance,
            ).await?;
            
            let gas_fee = gas_price * gas_limit;
            if gas_fee >= balance {
                return Err("当前余额不足支付Gas费用，不做转账操作！".into());
            }
            balance - gas_fee
        }
        "2" => {
            // 转账固定数量
            let amount = parse_ether(config.transfer_amount)?;
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "3" => {
            // 转账随机数量
            let mut rng = rand::thread_rng();
            let random_amount = rng.gen_range(config.transfer_amount_list[0]..=config.transfer_amount_list[1]);
            // 根据精度设置格式化随机金额
            let formatted_amount = format!("{:.precision$}", random_amount, precision = config.amount_precision as usize);
            let precise_amount: f64 = formatted_amount.parse()?;
            let amount = parse_ether(precise_amount)?;
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "4" => {
            // 剩余随机数量
            let balance_f64: f64 = balance_ether.to_string().parse()?;
            if balance_f64 >= config.left_amount_list[0] && balance_f64 <= config.left_amount_list[1] {
                return Err(format!(
                    "当前余额为：{} 在设置的剩余范围内，不做转账操作！",
                    balance_ether
                ).into());
            }
            
            let mut rng = rand::thread_rng();
            let left_amount = rng.gen_range(config.left_amount_list[0]..=config.left_amount_list[1]);
            let transfer_amount_f64 = balance_f64 - left_amount;
            
            if transfer_amount_f64 <= 0.0 {
                return Err("当前余额不足，不做转账操作！".into());
            }
            
            // 根据精度设置格式化转账金额
            let formatted_amount = format!("{:.precision$}", transfer_amount_f64, precision = config.amount_precision as usize);
            let precise_amount: f64 = formatted_amount.parse()?;
            parse_ether(precise_amount)?
        }
        _ => return Err("无效的转账类型".into()),
    };
    
    println!("序号：{}, 转账数量为: {} ETH", index, format_ether(transfer_amount));
    
    // 获取Gas Limit
    let gas_limit = TransferUtils::get_gas_limit(
        &config,
        provider.clone(),
        wallet_address,
        to_address,
        transfer_amount,
    ).await?;
    
    // 构建交易
    let tx = TransactionRequest::new()
        .from(wallet_address)
        .to(to_address)
        .value(transfer_amount)
        .gas_price(gas_price)
        .gas(gas_limit);
    
    // 发送交易
    item.error_msg = "发送交易...".to_string();
    // 发送状态更新事件到前端
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": item.error_msg.clone(),
        "exec_status": "1"
    }));
    
    let client = SignerMiddleware::new(provider.clone(), wallet);
    let pending_tx = client.send_transaction(tx, None).await?;
    
    println!("序号：{}, 交易 hash 为：{:?}", index, pending_tx.tx_hash());
    
    // 等待交易确认
    item.error_msg = "等待交易结果...".to_string();
    // 发送状态更新事件到前端
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": item.error_msg.clone(),
        "exec_status": "1"
    }));
    let receipt = pending_tx.await?;
    
    match receipt {
        Some(receipt) => {
            if receipt.status == Some(U64::from(1)) {
                Ok(format!("{:?}", receipt.transaction_hash))
            } else {
                Err("交易失败".into())
            }
        }
        None => Err("交易未确认".into()),
    }
}

// Tauri命令：查询余额
#[tauri::command]
pub async fn query_balance(
    chain: String,
    address: String,
) -> Result<String, String> {
    match query_balance_internal(chain, address).await {
        Ok(balance) => Ok(balance),
        Err(e) => Err(e.to_string()),
    }
}

// 内部查询余额实现
async fn query_balance_internal(
    chain: String,
    address: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let provider = create_provider(&chain).await?;
    if address.trim().is_empty() {
        return Err("查询地址不能为空！".into());
    }
    let address: Address = address.parse().map_err(|e| {
        format!("地址格式错误: {}，请检查地址格式是否正确", e)
    })?;
    let balance = provider.get_balance(address, None).await?;
    Ok(format_ether(balance).to_string())
}

// 检查钱包最近转账记录的结果结构
#[derive(Debug, Serialize, Deserialize)]
pub struct RecentTransferResult {
    pub has_recent_transfer: bool,
    pub transaction_count: u32,
    pub latest_transaction_hash: Option<String>,
}

// Tauri命令：检查钱包最近转账记录
#[tauri::command]
pub async fn check_wallet_recent_transfers(
    chain: String,
    private_key: String,
    target_address: String,
    start_timestamp: u64,
    coin_type: String,
    contract_address: Option<String>,
) -> Result<RecentTransferResult, String> {
    match check_wallet_recent_transfers_internal(
        chain,
        private_key,
        target_address,
        start_timestamp,
        coin_type,
        contract_address,
    ).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

// 内部检查钱包最近转账记录实现
async fn check_wallet_recent_transfers_internal(
    chain: String,
    private_key: String,
    target_address: String,
    start_timestamp: u64,
    coin_type: String,
    contract_address: Option<String>,
) -> Result<RecentTransferResult, Box<dyn std::error::Error>> {
    // 创建Provider
    let provider = create_provider(&chain).await?;
    
    // 处理私钥格式
    let private_key = if private_key.starts_with("0x") || private_key.starts_with("0X") {
        private_key[2..].to_string()
    } else {
        private_key.clone()
    };
    
    // 创建钱包
    let wallet = private_key.parse::<LocalWallet>().map_err(|e| {
        format!("私钥格式错误: {}", e)
    })?;
    let wallet_address = wallet.address();
    
    // 解析目标地址
    let target_addr: Address = target_address.parse().map_err(|e| {
        format!("目标地址格式错误: {}", e)
    })?;
    
    // 获取当前区块号
    let current_block = provider.get_block_number().await?;
    
    // 计算开始查询的区块号（简化实现：从最近1000个区块开始查询）
    let start_block = if current_block.as_u64() > 1000 {
        current_block - 1000
    } else {
        U64::from(0)
    };
    
    let mut transaction_count = 0u32;
    let mut latest_transaction_hash: Option<String> = None;
    let mut has_recent_transfer = false;
    
    // 查询指定区块范围内的交易
    for block_num in start_block.as_u64()..=current_block.as_u64() {
        if let Ok(Some(block)) = provider.get_block_with_txs(U64::from(block_num)).await {
            // 检查区块时间戳是否在指定时间之后
            if block.timestamp.as_u64() < start_timestamp {
                continue;
            }
            
            // 遍历区块中的所有交易
            for tx in block.transactions {
                // 检查交易是否来自指定钱包地址
                if tx.from == wallet_address {
                    transaction_count += 1;
                    
                    // 根据币种类型检查交易
                    match coin_type.as_str() {
                        "base" => {
                            // 基础币转账：检查to地址
                            if let Some(to_addr) = tx.to {
                                if to_addr == target_addr {
                                    has_recent_transfer = true;
                                    latest_transaction_hash = Some(format!("{:?}", tx.hash));
                                }
                            }
                        }
                        "token" => {
                            // 代币转账：检查合约调用和事件日志
                            if let Some(contract_addr) = &contract_address {
                                let contract_address: Address = contract_addr.parse().map_err(|e| {
                                    format!("合约地址格式错误: {}", e)
                                })?;
                                
                                // 检查是否是对指定合约的调用
                                if let Some(to_addr) = tx.to {
                                    if to_addr == contract_address {
                                        // 获取交易回执以检查事件日志
                                        if let Ok(Some(receipt)) = provider.get_transaction_receipt(tx.hash).await {
                                            // 检查Transfer事件日志
                                            for log in receipt.logs {
                                                // Transfer事件的topic0是keccak256("Transfer(address,address,uint256)")
                                                let transfer_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
                                                
                                                if !log.topics.is_empty() && 
                                                   format!("{:?}", log.topics[0]) == transfer_topic &&
                                                   log.topics.len() >= 3 {
                                                    // topics[1]是from地址，topics[2]是to地址
                                                    let to_topic = log.topics[2];
                                                    // 将topic转换为地址（去掉前12个字节的0）
                                                    let to_bytes = &to_topic.as_bytes()[12..];
                                                    let to_address = Address::from_slice(to_bytes);
                                                    
                                                    if to_address == target_addr {
                                                        has_recent_transfer = true;
                                                        latest_transaction_hash = Some(format!("{:?}", tx.hash));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // 未知币种类型，跳过
                            continue;
                        }
                    }
                }
            }
        }
    }
    
    Ok(RecentTransferResult {
        has_recent_transfer,
        transaction_count,
        latest_transaction_hash,
    })
}