use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest;
use ethers::{
    prelude::*,
    providers::{Http, Provider},
    types::{Address, U256},
    utils::{format_ether, format_units, parse_ether, parse_units},
};
use std::str::FromStr;
use std::sync::Arc;
use rand::Rng;

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

// RPC节点配置
pub struct RpcConfig {
    pub rpc_urls: Vec<String>,
    pub chain_id: u64,
}

impl RpcConfig {
    pub fn get_random_rpc(&self) -> &str {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.rpc_urls.len());
        &self.rpc_urls[index]
    }
}

// 获取不同链的RPC配置
pub fn get_rpc_config(chain: &str) -> Option<RpcConfig> {
    match chain {
        "eth" => Some(RpcConfig {
            rpc_urls: vec![
                "https://rpc.ankr.com/eth/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f".to_string(),
                "https://eth-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5".to_string(),
                "https://1rpc.io/eth".to_string(),
            ],
            chain_id: 1,
        }),
        "bsc" => Some(RpcConfig {
            rpc_urls: vec![
                "https://bsc-dataseed1.bnbchain.org".to_string(),
                "https://bsc.publicnode.com".to_string(),
                "https://rpc.ankr.com/bsc".to_string(),
            ],
            chain_id: 56,
        }),
        "polygon" => Some(RpcConfig {
            rpc_urls: vec![
                "https://polygon-bor.publicnode.com".to_string(),
                "https://polygon-rpc.com".to_string(),
                "https://rpc.ankr.com/polygon".to_string(),
            ],
            chain_id: 137,
        }),
        "arb" => Some(RpcConfig {
            rpc_urls: vec![
                "https://arbitrum-one.public.blastapi.io".to_string(),
                "https://rpc.ankr.com/arbitrum".to_string(),
                "https://1rpc.io/arb".to_string(),
            ],
            chain_id: 42161,
        }),
        "op" => Some(RpcConfig {
            rpc_urls: vec![
                "https://rpc.ankr.com/optimism".to_string(),
                "https://opt-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5".to_string(),
            ],
            chain_id: 10,
        }),
        "base" => Some(RpcConfig {
            rpc_urls: vec![
                "https://base.publicnode.com".to_string(),
                "https://1rpc.io/base".to_string(),
                "https://base.llamarpc.com".to_string(),
            ],
            chain_id: 8453,
        }),
        _ => None,
    }
}

// 创建Provider
pub async fn create_provider(chain: &str) -> Result<Arc<Provider<Http>>, Box<dyn std::error::Error>> {
    let rpc_config = get_rpc_config(chain)
        .ok_or(format!("不支持的链: {}", chain))?;
    
    let rpc_url = rpc_config.get_random_rpc();
    let provider = Provider::<Http>::try_from(rpc_url)?;
    
    Ok(Arc::new(provider))
}

// 转账工具函数
pub struct TransferUtils;

impl TransferUtils {
    // 检查字符串是否为数字
    pub fn check_num(num: &str) -> bool {
        if num.is_empty() {
            return false;
        }
        let re = regex::Regex::new(r"^[0-9]+\.?[0-9]*$").unwrap();
        re.is_match(num)
    }

    // 检查字符串是否为正整数
    pub fn check_positive_integer(num: &str) -> bool {
        if num.is_empty() {
            return false;
        }
        let re = regex::Regex::new(r"^[1-9]+[0-9]*$").unwrap();
        re.is_match(num)
    }

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
                let gas_price_with_rate = base_gas_price * (100 + (config.gas_price_rate * 100.0) as u64) / 100;
                
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
pub async fn base_coin_transfer(
    index: usize,
    item: TransferItem,
    config: TransferConfig,
) -> Result<TransferResult, String> {
    match base_coin_transfer_internal(index, item, config).await {
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
async fn base_coin_transfer_internal(
    index: usize,
    mut item: TransferItem,
    config: TransferConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    item.retry_flag = false;
    
    // 创建Provider
    let provider = create_provider(&config.chain).await?;
    
    // 创建钱包
    let wallet = item.private_key.parse::<LocalWallet>()?;
    let wallet = wallet.with_chain_id(get_rpc_config(&config.chain).unwrap().chain_id);
    let wallet_address = wallet.address();
    
    // 获取余额
    let balance = provider.get_balance(wallet_address, None).await?;
    let balance_ether = format_ether(balance);
    
    println!("序号：{}, 当前余额为: {} ETH", index, balance_ether);
    
    if balance.is_zero() {
        return Err("当前余额不足，不做转账操作！".into());
    }
    
    // 解析目标地址
    let to_address: Address = item.to_addr.parse()?;
    
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
            let amount = parse_ether(random_amount)?;
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "4" => {
            // 剩余随机数量
            let balance_f64 = balance_ether.parse::<f64>()?;
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
            
            parse_ether(transfer_amount_f64)?
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
    let client = SignerMiddleware::new(provider.clone(), wallet);
    let pending_tx = client.send_transaction(tx, None).await?;
    
    println!("序号：{}, 交易 hash 为：{:?}", index, pending_tx.tx_hash());
    
    // 等待交易确认
    item.error_msg = "等待交易结果...".to_string();
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
    let address: Address = address.parse()?;
    let balance = provider.get_balance(address, None).await?;
    Ok(format_ether(balance))
}