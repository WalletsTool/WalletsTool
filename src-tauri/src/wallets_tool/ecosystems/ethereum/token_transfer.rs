use serde::{Deserialize, Serialize};
use ethers::{
    prelude::*,
    providers::{Http, Provider},
    types::{Address, U256},
    utils::{format_units, parse_units},
    contract::Contract,
};
use std::sync::Arc;
use rand::Rng;
use tauri::Emitter;
use super::transfer::{TransferConfig, TransferItem, TransferResult, FastTransferResult, create_provider, get_rpc_config, TransferUtils};
use crate::database::{get_database_manager, chain_service::ChainService};

// ERC20 ABI (简化版，只包含必要的函数)
const ERC20_ABI: &str = r#"[
    {
        "constant": true,
        "inputs": [{"name": "_owner", "type": "address"}],
        "name": "balanceOf",
        "outputs": [{"name": "balance", "type": "uint256"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "decimals",
        "outputs": [{"name": "", "type": "uint8"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "symbol",
        "outputs": [{"name": "", "type": "string"}],
        "type": "function"
    },
    {
        "constant": false,
        "inputs": [
            {"name": "_to", "type": "address"},
            {"name": "_value", "type": "uint256"}
        ],
        "name": "transfer",
        "outputs": [{"name": "", "type": "bool"}],
        "type": "function"
    }
]"#;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenTransferConfig {
    pub chain: String,
    pub contract_address: String,
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
    #[serde(default)]
    pub window_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub decimals: u8,
    pub balance: String,
}

// 代币转账工具
pub struct TokenTransferUtils;

impl TokenTransferUtils {
    // 获取代币合约Gas Limit
    pub async fn get_contract_gas_limit(
        config: &TokenTransferConfig,
        provider: Arc<Provider<Http>>,
        _contract_address: Address,
        wallet_address: Address,
        to_address: Address,
        transfer_amount: U256,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        // 将TokenTransferConfig转换为TransferConfig
        let transfer_config = TransferConfig {
            chain: config.chain.clone(),
            delay: config.delay,
            transfer_type: config.transfer_type.clone(),
            transfer_amount: config.transfer_amount,
            transfer_amount_list: config.transfer_amount_list.clone(),
            left_amount_list: config.left_amount_list.clone(),
            amount_precision: config.amount_precision,
            limit_type: config.limit_type.clone(),
            limit_count: config.limit_count,
            limit_count_list: config.limit_count_list.clone(),
            gas_price_type: config.gas_price_type.clone(),
            gas_price: config.gas_price,
            gas_price_rate: config.gas_price_rate,
            max_gas_price: config.max_gas_price,
            error_retry: config.error_retry.clone(),
            error_count_limit: config.error_count_limit,
            window_id: config.window_id.clone(),
        };
        
        match config.limit_type.as_str() {
            "1" => {
                // 自动估算Gas Limit
                // 使用新的gas limit函数，传入is_eth=false表示这是代币转账
                let gas_limit = TransferUtils::get_gas_limit_with_token_type(
                    &transfer_config,
                    provider.clone(),
                    wallet_address, // from地址
                    to_address,      // to地址
                    transfer_amount, // 转账金额
                    false // is_eth = false，表示这是代币转账
                ).await?;
                
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

// Tauri命令：代币转账
#[tauri::command]
pub async fn token_transfer<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    item: TransferItem,
    config: TokenTransferConfig,
) -> Result<TransferResult, String> {
    match token_transfer_internal(app_handle, index, item, config).await {
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

// 内部代币转账实现
async fn token_transfer_internal<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    mut item: TransferItem,
    config: TokenTransferConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    item.retry_flag = false;
    
    // 从数据库获取代币的decimals配置
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    let db_decimals = chain_service.get_token_decimals_by_contract(&config.chain, &config.contract_address).await
        .map_err(|e| {
            println!("[ERROR] 从数据库获取decimals失败: {}", e);
            e
        })?;
    
    // 创建Provider
    let provider = create_provider(&config.chain, config.window_id.as_deref()).await.map_err(|e| {
        format!("获取RPC提供商失败: {}", e)
    })?;
    
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
    
    // 解析合约地址和目标地址
    let contract_address: Address = config.contract_address.parse()?;
    let to_address: Address = item.to_addr.parse()?;
    
    // 创建合约实例
    let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
    let contract: Contract<Arc<Provider<Http>>> = Contract::new(contract_address, abi, provider.clone().into());
    
    // 获取当前使用的RPC URL
    let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
        match rpc_config.get_random_rpc() {
            Ok(url) => url.to_string(),
            Err(e) => format!("获取RPC地址失败: {}", e)
        }
    } else {
        "未知RPC".to_string()
    };
    
    // 获取代币信息
    let balance: U256 = contract.method("balanceOf", wallet_address)?.call().await.map_err(|e| {
        format!("获取代币余额失败 (RPC: {}): {}", rpc_url, e)
    })?;
    
    // 使用数据库配置的decimals值，如果没有则从合约查询
    let decimals = if let Some(db_decimals) = db_decimals {
        println!("[DEBUG] 使用数据库配置的decimals: {}", db_decimals);
        db_decimals as u8
    } else {
        println!("[DEBUG] 数据库中未找到decimals配置，从合约查询...");
        let contract_decimals: u8 = contract.method("decimals", ())?.call().await.map_err(|e| {
            format!("获取代币decimals失败 (RPC: {}): {}", rpc_url, e)
        })?;
        println!("[DEBUG] 合约返回的decimals: {}", contract_decimals);
        contract_decimals
    };
    
    let symbol: String = contract.method("symbol", ())?.call().await.map_err(|e| {
        format!("获取代币symbol失败 (RPC: {}): {}", rpc_url, e)
    })?;
    
    let balance_formatted = format_units(balance, decimals as u32)?;
    
    println!("序号：{}, 当前{}余额为: {}", index, symbol, balance_formatted);
    
    if balance.is_zero() {
        return Err("当前余额不足，不做转账操作！".into());
    }
    
    // 获取Gas Price
    let transfer_config = TransferConfig {
        chain: config.chain.clone(),
        delay: config.delay,
        transfer_type: config.transfer_type.clone(),
        transfer_amount: config.transfer_amount,
        transfer_amount_list: config.transfer_amount_list,
        left_amount_list: config.left_amount_list,
        amount_precision: config.amount_precision,
        limit_type: config.limit_type.clone(),
        limit_count: config.limit_count,
        limit_count_list: config.limit_count_list,
        gas_price_type: config.gas_price_type.clone(),
        gas_price: config.gas_price,
        gas_price_rate: config.gas_price_rate,
        max_gas_price: config.max_gas_price,
        error_retry: config.error_retry.clone(),
        error_count_limit: config.error_count_limit,
        window_id: config.window_id.clone(),
    };
    
    // 获取当前使用的RPC URL用于错误信息
    let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
        match rpc_config.get_random_rpc() {
            Ok(url) => url.to_string(),
            Err(e) => format!("获取RPC地址失败: {}", e)
        }
    } else {
        "未知RPC".to_string()
    };
    
    let gas_price = TransferUtils::get_gas_price(&transfer_config, provider.clone()).await.map_err(|e| {
        format!("获取Gas Price失败 (RPC: {}): {}", rpc_url, e)
    })?;
    
    // 计算转账金额
    let transfer_amount = match config.transfer_type.as_str() {
        "1" => {
            // 全部转账
            balance
        }
        "2" => {
            // 转账固定数量
            let amount = parse_units(config.transfer_amount, decimals as u32)?.into();
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
            let amount = parse_units(precise_amount, decimals as u32)?.into();
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "4" => {
            // 剩余随机数量
            let balance_f64 = format_units(balance, decimals as u32)?.parse::<f64>()?;
            
            println!("序号：{}, 代币余额: {}", index, balance_f64);
            
            // 检查余额是否足够满足最小剩余数量要求
            if balance_f64 <= config.left_amount_list[1] {
                return Err(format!(
                    "当前代币余额为：{}，无法满足最大剩余数量 {} 要求，不做转账操作！",
                    balance_formatted, config.left_amount_list[1]
                ).into());
            }
            
            let mut rng = rand::thread_rng();
            let left_amount = rng.gen_range(config.left_amount_list[0]..=config.left_amount_list[1]);
            let transfer_amount_f64 = balance_f64 - left_amount;
            
            if transfer_amount_f64 <= 0.0 {
                return Err(format!(
                    "计算转账金额为负数或零：代币余额 {} - 剩余数量 {} = {}，不做转账操作！",
                    balance_f64, left_amount, transfer_amount_f64
                ).into());
            }
            
            // 根据精度设置格式化转账金额
            let formatted_amount = format!("{:.precision$}", transfer_amount_f64, precision = config.amount_precision as usize);
            let precise_amount: f64 = formatted_amount.parse()?;
            
            println!("序号：{}, 剩余数量: {}, 转账金额: {} (格式化后: {})", index, left_amount, transfer_amount_f64, precise_amount);
            
            parse_units(precise_amount, decimals as u32)?.into()
        }
        _ => return Err("无效的转账类型".into()),
    };
    
    let transfer_amount_formatted = format_units(transfer_amount, decimals as u32)?;
    println!("序号：{}, 转账数量为: {} {}", index, transfer_amount_formatted, symbol);
    
    // 获取Gas Limit
    let gas_limit = TokenTransferUtils::get_contract_gas_limit(
        &config,
        provider.clone(),
        contract_address,
        wallet_address,
        to_address,
        transfer_amount,
    ).await?;
    
    println!("序号：{}, gasLimit: {}", index, gas_limit);
    
    // 构建转账交易前的详细验证和日志
    println!("[DEBUG] ===== 转账交易构建阶段 =====");
    println!("[DEBUG] 序号: {}", index);
    println!("[DEBUG] 合约地址: {:?}", contract_address);
    println!("[DEBUG] 发送方地址: {:?}", wallet_address);
    println!("[DEBUG] 接收方地址: {:?}", to_address);
    println!("[DEBUG] 转账金额: {} (原始值: {})", transfer_amount_formatted, transfer_amount);
    println!("[DEBUG] Gas Price: {} wei", gas_price);
    println!("[DEBUG] Gas Limit: {}", gas_limit);
    
    // 验证地址有效性
    if wallet_address == Address::zero() {
        return Err("发送方地址无效（零地址）".into());
    }
    if to_address == Address::zero() {
        return Err("接收方地址无效（零地址）".into());
    }
    if contract_address == Address::zero() {
        return Err("合约地址无效（零地址）".into());
    }
    
    // 验证转账金额
    if transfer_amount.is_zero() {
        return Err("转账金额不能为零".into());
    }
    
    // 获取发送方平台币余额用于gas费检查
    let wallet_balance = provider.get_balance(wallet_address, None).await.map_err(|e| {
        format!("获取平台币余额失败 (RPC: {}): {}", rpc_url, e)
    })?;
    let estimated_gas_fee = gas_price * gas_limit;
    
    // 格式化为可读的单位
    let balance_formatted = format_units(wallet_balance, 18).unwrap_or_else(|_| "N/A".to_string());
    let gas_fee_formatted = format_units(estimated_gas_fee, 18).unwrap_or_else(|_| "N/A".to_string());
    let gas_price_gwei = format_units(gas_price, "gwei").unwrap_or_else(|_| "N/A".to_string());
    
    println!("[DEBUG] 平台币余额: {} wei ({} BNB/ETH)", wallet_balance, balance_formatted);
    println!("[DEBUG] Gas Price: {} wei ({} gwei)", gas_price, gas_price_gwei);
    println!("[DEBUG] Gas Limit: {}", gas_limit);
    println!("[DEBUG] 预估Gas费用: {} wei ({} BNB/ETH)", estimated_gas_fee, gas_fee_formatted);
    
    // 获取链ID以进行特殊处理
    let chain_id = provider.get_chainid().await.unwrap_or_default().as_u64();
    
    // 对BSC链进行特殊处理，使用更宽松的余额检查
    if chain_id == 56 || chain_id == 97 {
        // BSC链的余额检查应该更加宽松，考虑到可能的计算误差
        let buffer_percentage = U256::from(110); // 10%的缓冲
        let buffered_gas_fee = estimated_gas_fee * buffer_percentage / U256::from(100);
        
        println!("[DEBUG] BSC链特殊处理 - 原始预估Gas费用: {}, 带缓冲的Gas费用: {}", estimated_gas_fee, buffered_gas_fee);
        
        if wallet_balance < buffered_gas_fee {
            return Err(format!(
                "平台币余额不足支付Gas费用！\n当前余额: {} ({} wei)\n预估Gas费用: {} ({} wei)\nGas Price: {} gwei, Gas Limit: {}\n(已考虑10%缓冲)",
                balance_formatted, wallet_balance,
                gas_fee_formatted, estimated_gas_fee,
                gas_price_gwei, gas_limit
            ).into());
        }
    } else {
        // 其他链使用标准检查
        if wallet_balance < estimated_gas_fee {
            return Err(format!(
                "平台币余额不足支付Gas费用！\n当前余额: {} ({} wei)\n预估Gas费用: {} ({} wei)\nGas Price: {} gwei, Gas Limit: {}",
                balance_formatted, wallet_balance,
                gas_fee_formatted, estimated_gas_fee,
                gas_price_gwei, gas_limit
            ).into());
        }
    }
    
    // 构建转账交易
    let client = SignerMiddleware::new(provider.clone(), wallet);
    let contract_with_signer: Contract<Arc<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>> = Contract::new(contract_address, contract.abi().clone(), Arc::new(client).into());
    
    item.error_msg = "发送交易...".to_string();
    // 发送状态更新事件到前端
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": item.error_msg.clone(),
        "exec_status": "1"
    }));
    
    println!("[DEBUG] ===== 发送交易阶段 =====");
    
    // 调用transfer方法
    let call = contract_with_signer
        .method::<_, bool>("transfer", (to_address, transfer_amount))?
        .gas_price(gas_price)
        .gas(gas_limit);
    
    println!("[DEBUG] 交易调用已构建，准备发送...");
    
    let pending_tx = match call.send().await {
        Ok(tx) => {
            println!("[DEBUG] 交易发送成功，等待确认...");
            tx
        }
        Err(e) => {
            // 获取当前使用的RPC URL
            let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                match rpc_config.get_random_rpc() {
                    Ok(url) => url.to_string(),
                    Err(e) => format!("获取RPC地址失败: {}", e)
                }
            } else {
                "未知RPC".to_string()
            };
            
            let error_msg = format!("发送交易失败 (RPC: {}): {}", rpc_url, e);
            println!("[ERROR] {}", error_msg);
            
            // 分析具体的错误类型
            let detailed_error = if e.to_string().contains("insufficient funds") {
                format!("余额不足 (RPC: {}): {}", rpc_url, e)
            } else if e.to_string().contains("gas") {
                format!("Gas相关错误 (RPC: {}): {}", rpc_url, e)
            } else if e.to_string().contains("revert") {
                format!("合约执行被回滚 (RPC: {}): {}", rpc_url, e)
            } else if e.to_string().contains("nonce") {
                format!("Nonce错误 (RPC: {}): {}", rpc_url, e)
            } else {
                format!("网络或其他错误 (RPC: {}): {}", rpc_url, e)
            };
            
            return Err(detailed_error.into());
        }
    };
    
    let tx_hash = pending_tx.tx_hash();
    println!("序号：{}, 交易 hash 为：{:?}", index, tx_hash);
    
    // 等待交易确认（设置30秒超时）
    item.error_msg = "等待交易结果...".to_string();
    // 发送状态更新事件到前端
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": item.error_msg.clone(),
        "exec_status": "1"
    }));
    
    println!("[DEBUG] ===== 等待交易确认阶段 =====");
    println!("[DEBUG] 交易哈希: {:?}", tx_hash);
    println!("[DEBUG] 开始等待交易确认，设置30秒超时...");
    
    // 获取RPC URL用于错误消息
    let rpc_url_for_error = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
        match rpc_config.get_random_rpc() {
            Ok(url) => url.to_string(),
            Err(e) => format!("获取RPC地址失败: {}", e)
        }
    } else {
        "未知RPC".to_string()
    };
    
    let receipt = match tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        pending_tx
    ).await {
        Ok(result) => {
            result.map_err(|e| {
                let error_msg = format!("等待交易确认失败 (RPC: {}) (交易哈希: {:?}): {}", rpc_url_for_error, tx_hash, e);
                println!("[ERROR] {}", error_msg);
                error_msg
            })?
        }
        Err(_) => {
            // 超时处理
            let timeout_msg = format!("等待交易确认超时 (RPC: {}) - 超过30秒未收到确认，交易哈希: {:?}", rpc_url_for_error, tx_hash);
            println!("[ERROR] {}", timeout_msg);
            return Err(timeout_msg.into());
        }
    };
    
    match receipt {
        Some(receipt) => {
            println!("[DEBUG] ===== 交易收据分析 =====");
            println!("[DEBUG] 交易哈希: {:?}", receipt.transaction_hash);
            println!("[DEBUG] 区块号: {:?}", receipt.block_number);
            println!("[DEBUG] Gas使用量: {:?}", receipt.gas_used);
            println!("[DEBUG] 交易状态: {:?}", receipt.status);
            println!("[DEBUG] 累积Gas使用量: {:?}", receipt.cumulative_gas_used);
            
            if receipt.status == Some(U64::from(1)) {
                println!("[INFO] 交易执行成功！");
                Ok(format!("{:?}", receipt.transaction_hash))
            } else {
                // 获取当前使用的RPC URL
                let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                    match rpc_config.get_random_rpc() {
                        Ok(url) => url.to_string(),
                        Err(e) => format!("获取RPC地址失败: {}", e)
                    }
                } else {
                    "未知RPC".to_string()
                };
                
                let error_msg = format!(
                    "交易执行失败 (RPC: {}) - 交易哈希: {:?}, 区块号: {:?}, Gas使用: {:?}/{}, 状态: {:?}",
                    rpc_url,
                    receipt.transaction_hash,
                    receipt.block_number.unwrap_or_default(),
                    receipt.gas_used.unwrap_or_default(),
                    gas_limit,
                    receipt.status.unwrap_or_default()
                );
                println!("[ERROR] {}", error_msg);
                
                // 尝试获取更详细的失败原因
                let detailed_error = if let Some(gas_used) = receipt.gas_used {
                    if gas_used >= gas_limit {
                        format!("{} (可能原因: Gas不足，已用完所有Gas)", error_msg)
                    } else {
                        format!("{} (可能原因: 合约执行被回滚)", error_msg)
                    }
                } else {
                    error_msg
                };
                
                Err(detailed_error.into())
            }
        }
        None => {
            // 获取当前使用的RPC URL
            let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
                match rpc_config.get_random_rpc() {
                    Ok(url) => url.to_string(),
                    Err(e) => format!("获取RPC地址失败: {}", e)
                }
            } else {
                "未知RPC".to_string()
            };
            let error_msg = format!("交易未确认 (RPC: {}) (交易哈希: {:?}) - 可能网络拥堵或交易被丢弃", rpc_url, tx_hash);
            println!("[ERROR] {}", error_msg);
            Err(error_msg.into())
        }
    }
}

// Tauri命令：快速代币转账（狂暴模式 - 只提交不等待确认）
#[tauri::command]
pub async fn token_transfer_fast<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    item: TransferItem,
    config: TokenTransferConfig,
) -> Result<FastTransferResult, String> {
    match token_transfer_fast_internal(app_handle, index, item, config).await {
        Ok(tx_hash) => Ok(FastTransferResult {
            success: true,
            tx_hash: Some(tx_hash),
            error: None,
        }),
        Err(e) => Ok(FastTransferResult {
            success: false,
            tx_hash: None,
            error: Some(e.to_string()),
        }),
    }
}

// 内部快速代币转账实现（只提交交易，不等待确认）
async fn token_transfer_fast_internal<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    mut item: TransferItem,
    config: TokenTransferConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    item.retry_flag = false;
    
    // 从数据库获取代币的decimals配置
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    let db_decimals = chain_service.get_token_decimals_by_contract(&config.chain, &config.contract_address).await?;
    
    // 创建Provider
    let provider = create_provider(&config.chain, config.window_id.as_deref()).await?;
    
    // 创建钱包
    if item.private_key.trim().is_empty() {
        return Err("私钥不能为空！".into());
    }
    
    let private_key = if item.private_key.starts_with("0x") || item.private_key.starts_with("0X") {
        item.private_key[2..].to_string()
    } else {
        item.private_key.clone()
    };
    
    let wallet = private_key.parse::<LocalWallet>()?;
    let wallet = wallet.with_chain_id(get_rpc_config(&config.chain).await.unwrap().chain_id);
    let wallet_address = wallet.address();
    
    // 解析地址
    let contract_address: Address = config.contract_address.parse()?;
    let to_address: Address = item.to_addr.parse()?;
    
    // 创建合约实例
    let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
    let contract: Contract<Arc<Provider<Http>>> = Contract::new(contract_address, abi, provider.clone().into());
    
    // 获取代币余额
    let balance: U256 = contract.method("balanceOf", wallet_address)?.call().await?;
    
    // 获取decimals
    let decimals = if let Some(db_decimals) = db_decimals {
        db_decimals as u8
    } else {
        contract.method::<_, u8>("decimals", ())?.call().await?
    };
    
    if balance.is_zero() {
        return Err("代币余额不足！".into());
    }
    
    // 获取Gas Price
    let transfer_config = TransferConfig {
        chain: config.chain.clone(),
        delay: config.delay,
        transfer_type: config.transfer_type.clone(),
        transfer_amount: config.transfer_amount,
        transfer_amount_list: config.transfer_amount_list,
        left_amount_list: config.left_amount_list,
        amount_precision: config.amount_precision,
        limit_type: config.limit_type.clone(),
        limit_count: config.limit_count,
        limit_count_list: config.limit_count_list,
        gas_price_type: config.gas_price_type.clone(),
        gas_price: config.gas_price,
        gas_price_rate: config.gas_price_rate,
        max_gas_price: config.max_gas_price,
        error_retry: config.error_retry.clone(),
        error_count_limit: config.error_count_limit,
        window_id: config.window_id.clone(),
    };
    
    let gas_price = TransferUtils::get_gas_price(&transfer_config, provider.clone()).await?;
    
    // 计算转账金额（简化版本）
    let transfer_amount = match config.transfer_type.as_str() {
        "1" => balance, // 全部转账
        "2" => {
            let amount: U256 = parse_units(config.transfer_amount, decimals as u32)?.into();
            if amount >= balance {
                return Err("余额不足".into());
            }
            amount
        }
        "3" => {
            let mut rng = rand::thread_rng();
            let random_amount = rng.gen_range(config.transfer_amount_list[0]..=config.transfer_amount_list[1]);
            let formatted = format!("{:.precision$}", random_amount, precision = config.amount_precision as usize);
            let precise: f64 = formatted.parse()?;
            let amount: U256 = parse_units(precise, decimals as u32)?.into();
            if amount >= balance {
                return Err("余额不足".into());
            }
            amount
        }
        "4" => {
            let balance_f64: f64 = format_units(balance, decimals as u32)?.parse()?;
            if balance_f64 <= config.left_amount_list[1] {
                return Err("余额不足".into());
            }
            let mut rng = rand::thread_rng();
            let left = rng.gen_range(config.left_amount_list[0]..=config.left_amount_list[1]);
            let transfer_f64 = balance_f64 - left;
            if transfer_f64 <= 0.0 {
                return Err("计算转账金额为负".into());
            }
            let formatted = format!("{:.precision$}", transfer_f64, precision = config.amount_precision as usize);
            let precise: f64 = formatted.parse()?;
            parse_units(precise, decimals as u32)?.into()
        }
        _ => return Err("无效的转账类型".into()),
    };
    
    // 获取Gas Limit
    let gas_limit = TokenTransferUtils::get_contract_gas_limit(
        &config,
        provider.clone(),
        contract_address,
        wallet_address,
        to_address,
        transfer_amount,
    ).await?;
    
    // 发送状态更新
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": "发送交易中...",
        "exec_status": "1"
    }));
    
    // 构建并发送交易
    let client = SignerMiddleware::new(provider.clone(), wallet);
    let contract_with_signer: Contract<Arc<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>> =
        Contract::new(contract_address, contract.abi().clone(), Arc::new(client).into());
    
    let call = contract_with_signer
        .method::<_, bool>("transfer", (to_address, transfer_amount))?
        .gas_price(gas_price)
        .gas(gas_limit);
    
    let pending_tx = call.send().await.map_err(|e| {
        format!("发送交易失败: {}", e)
    })?;
    
    let tx_hash = pending_tx.tx_hash();
    let tx_hash_str = format!("{:?}", tx_hash);
    
    println!("[狂暴模式] 序号：{}, 代币交易已提交，hash: {}", index, tx_hash_str);
    
    // 发送状态更新
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": format!("已提交，等待确认: {}", &tx_hash_str[..20]),
        "exec_status": "1"
    }));
    
    // 不等待确认，直接返回交易哈希
    Ok(tx_hash_str)
}

// Tauri命令：查询代币余额
#[tauri::command]
pub async fn query_token_balance(
    chain: String,
    contract_address: String,
    wallet_address: String,
) -> Result<TokenInfo, String> {
    match query_token_balance_internal(chain, contract_address, wallet_address).await {
        Ok(token_info) => Ok(token_info),
        Err(e) => Err(e.to_string()),
    }
}

// 内部查询代币余额实现
async fn query_token_balance_internal(
    chain: String,
    contract_address: String,
    wallet_address: String,
) -> Result<TokenInfo, Box<dyn std::error::Error>> {
    println!("[DEBUG] 开始查询代币余额");
    println!("[DEBUG] 链: {}", chain);
    println!("[DEBUG] 合约地址: {}", contract_address);
    println!("[DEBUG] 钱包地址: {}", wallet_address);
    
    // 从数据库获取代币的decimals配置
    println!("[DEBUG] 正在从数据库获取代币decimals配置...");
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    let db_decimals = chain_service.get_token_decimals_by_contract(&chain, &contract_address).await
        .map_err(|e| {
            println!("[ERROR] 从数据库获取decimals失败: {}", e);
            e
        })?;
    
    println!("[DEBUG] 数据库中的decimals配置: {:?}", db_decimals);
    
    // 创建Provider
    println!("[DEBUG] 正在创建Provider...");
    let provider = create_provider(&chain, None).await?;
    println!("[DEBUG] Provider创建成功");
    
    // 解析地址
    println!("[DEBUG] 正在解析合约地址...");
    let contract_addr: Address = contract_address.parse()
        .map_err(|e| {
            println!("[ERROR] 合约地址解析失败: {}", e);
            e
        })?;
    println!("[DEBUG] 合约地址解析成功: {:?}", contract_addr);
    
    println!("[DEBUG] 正在解析钱包地址...");
    let wallet_addr: Address = wallet_address.parse()
        .map_err(|e| {
            println!("[ERROR] 钱包地址解析失败: {}", e);
            e
        })?;
    println!("[DEBUG] 钱包地址解析成功: {:?}", wallet_addr);
    
    // 创建合约实例
    println!("[DEBUG] 正在创建合约实例...");
    let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
    let contract: Contract<Arc<Provider<Http>>> = Contract::new(contract_addr, abi, provider.into());
    println!("[DEBUG] 合约实例创建成功");
    
    // 获取代币信息
    println!("[DEBUG] 正在调用balanceOf方法...");
    let balance: U256 = contract.method("balanceOf", wallet_addr)?.call().await
        .map_err(|e| {
            println!("[ERROR] balanceOf调用失败: {}", e);
            e
        })?;
    println!("[DEBUG] balanceOf原始返回值: {}", balance);
    
    // 详细的原始余额分析
    println!("[DEBUG] ===== 原始余额详细分析 =====");
    println!("[DEBUG] 原始余额十进制: {}", balance);
    println!("[DEBUG] 原始余额十六进制: 0x{:x}", balance);
    println!("[DEBUG] 原始余额是否为零: {}", balance.is_zero());
    if !balance.is_zero() {
        println!("[DEBUG] 原始余额位数: {} bits", balance.bits());
    }
    
    // 使用数据库配置的decimals值，如果没有则从合约查询
    let decimals = if let Some(db_decimals) = db_decimals {
        println!("[DEBUG] 使用数据库配置的decimals: {}", db_decimals);
        db_decimals as u8
    } else {
        println!("[DEBUG] 数据库中未找到decimals配置，从合约查询...");
        let contract_decimals: u8 = contract.method("decimals", ())?.call().await
            .map_err(|e| {
                println!("[ERROR] decimals调用失败: {}", e);
                e
            })?;
        println!("[DEBUG] 合约返回的decimals: {}", contract_decimals);
        contract_decimals
    };
    
    println!("[DEBUG] 正在获取symbol...");
    let symbol: String = contract.method("symbol", ())?.call().await
        .map_err(|e| {
            println!("[ERROR] symbol调用失败: {}", e);
            e
        })?;
    println!("[DEBUG] symbol: {}", symbol);
    
    println!("[DEBUG] 正在格式化余额...");
    println!("[DEBUG] 用于格式化的decimals值: {}", decimals);
    println!("[DEBUG] 格式化前原始余额: {} (十进制)", balance);
    println!("[DEBUG] 格式化计算: {} ÷ 10^{} = {} ÷ {}", balance, decimals, balance, 10_u64.pow(decimals as u32));
    
    let balance_formatted = format_units(balance, decimals as u32)?;
    
    println!("[DEBUG] 格式化后的余额: {}", balance_formatted);
    println!("[DEBUG] ===== 格式化对比 =====");
    println!("[DEBUG] 原始值: {}", balance);
    println!("[DEBUG] 格式化值: {}", balance_formatted);
    println!("[DEBUG] Decimals: {}", decimals);
    
    let result = TokenInfo {
        symbol: symbol.clone(),
        decimals,
        balance: balance_formatted.clone(),
    };
    
    println!("[DEBUG] 查询完成，返回结果: symbol={}, decimals={}, balance={}", 
             result.symbol, result.decimals, result.balance);
    
    Ok(result)
}

// Tauri命令：获取代币信息
#[tauri::command]
pub async fn get_token_info(
    chain: String,
    contract_address: String,
) -> Result<TokenInfo, String> {
    match get_token_info_internal(chain, contract_address).await {
        Ok(token_info) => Ok(token_info),
        Err(e) => Err(e.to_string()),
    }
}

// 内部获取代币信息实现
async fn get_token_info_internal(
    chain: String,
    contract_address: String,
) -> Result<TokenInfo, Box<dyn std::error::Error>> {
    let provider = create_provider(&chain, None).await?;
    
    let contract_addr: Address = contract_address.parse()?;
    
    // 创建合约实例
    let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
    let contract: Contract<Arc<Provider<Http>>> = Contract::new(contract_addr, abi, provider.into());
    
    // 获取代币信息
    let decimals: u8 = contract.method("decimals", ())?.call().await?;
    let symbol: String = contract.method("symbol", ())?.call().await?;
    
    Ok(TokenInfo {
        symbol,
        decimals,
        balance: "0".to_string(), // 不查询余额，只获取基本信息
    })
}