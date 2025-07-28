use serde::{Deserialize, Serialize};
use ethers::{
    prelude::*,
    providers::{Http, Provider},
    types::{Address, U256},
    utils::{format_units, parse_units},
    contract::Contract,
};
use std::str::FromStr;
use std::sync::Arc;
use rand::Rng;
use super::transfer::{TransferConfig, TransferItem, TransferResult, create_provider, get_rpc_config, TransferUtils};

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
        contract_address: Address,
        _wallet_address: Address,
        to_address: Address,
        transfer_amount: U256,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        match config.limit_type.as_str() {
            "1" => {
                // 自动估算Gas Limit
                let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
                let contract = Contract::new(contract_address, abi, provider.clone());
                
                // 构建transfer调用数据
                let call = contract.method::<_, bool>("transfer", (to_address, transfer_amount))?;
                let tx = call.tx;
                
                let gas_limit = provider.estimate_gas(&tx, None).await?;
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
pub async fn token_transfer(
    index: usize,
    item: TransferItem,
    config: TokenTransferConfig,
) -> Result<TransferResult, String> {
    match token_transfer_internal(index, item, config).await {
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
async fn token_transfer_internal(
    index: usize,
    mut item: TransferItem,
    config: TokenTransferConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    item.retry_flag = false;
    
    // 创建Provider
    let provider = create_provider(&config.chain).await?;
    
    // 创建钱包
    let wallet = item.private_key.parse::<LocalWallet>()?;
    let wallet = wallet.with_chain_id(get_rpc_config(&config.chain).unwrap().chain_id);
    let wallet_address = wallet.address();
    
    // 解析合约地址和目标地址
    let contract_address: Address = config.contract_address.parse()?;
    let to_address: Address = item.to_addr.parse()?;
    
    // 创建合约实例
    let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
    let contract = Contract::new(contract_address, abi, provider.clone());
    
    // 获取代币信息
    let balance: U256 = contract.method("balanceOf", wallet_address)?.call().await?;
    let decimals: u8 = contract.method("decimals", ())?.call().await?;
    let symbol: String = contract.method("symbol", ())?.call().await?;
    
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
    };
    
    let gas_price = TransferUtils::get_gas_price(&transfer_config, provider.clone()).await?;
    
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
            let amount = parse_units(random_amount, decimals as u32)?.into();
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "4" => {
            // 剩余随机数量
            let balance_f64 = format_units(balance, decimals as u32)?.parse::<f64>()?;
            if balance_f64 >= config.left_amount_list[0] && balance_f64 <= config.left_amount_list[1] {
                return Err(format!(
                    "当前余额为：{} 在设置的剩余范围内，不做转账操作！",
                    balance_formatted
                ).into());
            }
            
            let mut rng = rand::thread_rng();
            let left_amount = rng.gen_range(config.left_amount_list[0]..=config.left_amount_list[1]);
            let transfer_amount_f64 = balance_f64 - left_amount;
            
            if transfer_amount_f64 <= 0.0 {
                return Err("当前余额不足，不做转账操作！".into());
            }
            
            parse_units(transfer_amount_f64, decimals as u32)?.into()
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
    
    // 构建转账交易
    let client = SignerMiddleware::new(provider.clone(), wallet);
    let contract_with_signer = Contract::new(contract_address, contract.abi().clone(), Arc::new(client));
    
    item.error_msg = "发送交易...".to_string();
    
    // 调用transfer方法
    let call = contract_with_signer
        .method::<_, bool>("transfer", (to_address, transfer_amount))?
        .gas_price(gas_price)
        .gas(gas_limit);
    
    let pending_tx = call.send().await?;
    
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
    let provider = create_provider(&chain).await?;
    
    let contract_addr: Address = contract_address.parse()?;
    let wallet_addr: Address = wallet_address.parse()?;
    
    // 创建合约实例
    let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
    let contract = Contract::new(contract_addr, abi, provider);
    
    // 获取代币信息
    let balance: U256 = contract.method("balanceOf", wallet_addr)?.call().await?;
    let decimals: u8 = contract.method("decimals", ())?.call().await?;
    let symbol: String = contract.method("symbol", ())?.call().await?;
    
    let balance_formatted = format_units(balance, decimals as u32)?;
    
    Ok(TokenInfo {
        symbol,
        decimals,
        balance: balance_formatted,
    })
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
    let provider = create_provider(&chain).await?;
    
    let contract_addr: Address = contract_address.parse()?;
    
    // 创建合约实例
    let abi: ethers::abi::Abi = serde_json::from_str(ERC20_ABI)?;
    let contract = Contract::new(contract_addr, abi, provider);
    
    // 获取代币信息
    let decimals: u8 = contract.method("decimals", ())?.call().await?;
    let symbol: String = contract.method("symbol", ())?.call().await?;
    
    Ok(TokenInfo {
        symbol,
        decimals,
        balance: "0".to_string(), // 不查询余额，只获取基本信息
    })
}