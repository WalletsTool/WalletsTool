use serde::{Deserialize, Serialize};
use alloy_provider::Provider;
use alloy_primitives::{Address, U256, TxKind};
use alloy_rpc_types_eth::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;
use alloy_signer::Signer;
use std::sync::Arc;
use rand::Rng;
use tauri::Emitter;
use super::transfer::{TransferConfig, TransferItem, TransferResult, TransferUtils, create_provider, create_signer_provider, get_rpc_config, FastTransferResult, get_stop_flag};
use crate::wallets_tool::ecosystems::ethereum::provider::{ProviderUtils, AlloyProvider};
use hex;
use super::alloy_utils::{parse_ether_to_wei_f64, parse_gwei_to_wei, format_wei_to_ether, format_wei_to_gwei, u256_to_f64};




#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenTransferConfig {
    pub chain: String,
    pub contract_address: String,
    pub delay: [u64; 2],
    pub transfer_type: String,
    pub transfer_amount: f64,
    pub transfer_amount_list: [f64; 2],
    pub left_amount_list: [f64; 2],
    pub amount_precision: u8,
    pub limit_type: String,
    pub limit_count: u64,
    pub limit_count_list: [u64; 2],
    pub gas_price_type: String,
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

pub struct TokenTransferUtils;

impl TokenTransferUtils {
    pub async fn get_contract_gas_limit(
        config: &TokenTransferConfig,
        provider: Arc<AlloyProvider>,
        _contract_address: Address,
        wallet_address: Address,
        to_address: Address,
        transfer_amount: U256,
    ) -> Result<U256, Box<dyn std::error::Error>> {
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
        
        TransferUtils::get_gas_limit(&transfer_config, provider, wallet_address, to_address, transfer_amount).await
            .map_err(|e| format!("获取代币合约Gas Limit失败: {}", e).into())
    }

    pub async fn get_token_balance(
        provider: &AlloyProvider,
        contract_address: Address,
        wallet_address: Address,
    ) -> Result<U256, Box<dyn std::error::Error>> {
        let method_id = "70a08231";
        let address_param = format!("{:0>64}", &hex::encode(wallet_address)[2..]);
        let data = format!("0x{}{}", method_id, address_param);
        
        let tx = TransactionRequest {
            to: Some(TxKind::Call(contract_address)),
            input: data.parse::<alloy_primitives::Bytes>().unwrap().into(),
            ..Default::default()
        };
        
        let result = provider.call(tx).await?;
        
        if result.is_empty() {
            return Ok(U256::from(0));
        }
        
        let result_hex = hex::encode(result);
        let balance_hex = result_hex.trim_start_matches("0x");
        let balance = U256::from_str_radix(balance_hex, 16)
            .map_err(|e| format!("解析代币余额失败: {}", e))?;
        
        Ok(balance)
    }

    pub async fn get_token_info(
        provider: &AlloyProvider,
        contract_address: Address,
    ) -> Result<(String, u8), Box<dyn std::error::Error>> {
        let decimals_method = "313ce567";
        let decimals_data = format!("0x{}", decimals_method);
        
        let tx = TransactionRequest {
            to: Some(TxKind::Call(contract_address)),
            input: decimals_data.parse::<alloy_primitives::Bytes>().unwrap().into(),
            ..Default::default()
        };
        
        let decimals_result = provider.call(tx).await?;
        let decimals_str = hex::encode(decimals_result);
        let decimals_hex = decimals_str.trim_start_matches("0x");
        let decimals = u8::from_str_radix(decimals_hex, 16)
            .map_err(|e| format!("解析代币精度失败: {}", e))?;
        
        let symbol_method = "95d89b41";
        let symbol_data = format!("0x{}", symbol_method);
        
        let tx = TransactionRequest {
            to: Some(TxKind::Call(contract_address)),
            input: symbol_data.parse::<alloy_primitives::Bytes>().unwrap().into(),
            ..Default::default()
        };
        
        let symbol_result = provider.call(tx).await?;
        let symbol_hex = hex::encode(symbol_result);
        let symbol = hex::decode(symbol_hex.trim_start_matches("0x")).map_err(|e| format!("解析代币符号失败: {}", e))?;
        let symbol = String::from_utf8(symbol).map_err(|e| format!("代币符号转换失败: {}", e))?;
        
        Ok((symbol, decimals))
    }

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
}

async fn token_transfer_internal<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    mut item: TransferItem,
    config: TokenTransferConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    item.retry_flag = false;
    let window_id = config.window_id.as_deref().unwrap_or("");
    
    if item.private_key.trim().is_empty() {
        return Err("私钥不能为空！".into());
    }
    
    let private_key = if item.private_key.starts_with("0x") || item.private_key.starts_with("0X") {
        item.private_key[2..].to_string()
    } else {
        item.private_key.clone()
    };
    
    let signer: PrivateKeySigner = private_key.parse()
        .map_err(|e| format!("私钥格式错误: {}", e))?;
    
    let chain_id = match ProviderUtils::get_chain_id(&config.chain).await {
        Ok(id) => id,
        Err(_) => {
            match get_rpc_config(&config.chain).await {
                Some(c) => c.chain_id,
                None => {
                    return Err(format!("无法获取链 '{}' 的配置信息", config.chain).into());
                }
            }
        }
    };
    let signer = signer.with_chain_id(Some(chain_id));
    let wallet_address = signer.address();
    
    if item.to_addr.trim().is_empty() {
        return Err("目标地址不能为空".into());
    }
    let to_address: Address = item.to_addr.parse()
        .map_err(|e| format!("目标地址格式错误: {}", e))?;
    
    let contract_address: Address = config.contract_address.parse()
        .map_err(|e| format!("合约地址格式错误: {}", e))?;
    
    let rpc_url = if let Some(rpc_config) = get_rpc_config(&config.chain).await {
        match rpc_config.get_random_rpc() {
            Ok(url) => url.to_string(),
            Err(e) => return Err(format!("获取RPC地址失败: {}", e).into()),
        }
    } else {
        return Err(format!("无法获取链 '{}' 的RPC配置", config.chain).into());
    };
    
    let provider = create_provider(&config.chain, config.window_id.as_deref()).await
        .map_err(|e| format!("获取RPC提供商失败: {}", e))?;
    
    let balance = TokenTransferUtils::get_token_balance(&provider, contract_address, wallet_address).await
        .map_err(|e| format!("获取代币余额失败 (RPC: {}): {}", rpc_url, e))?;
    
    let decimals = if let Ok((_, d)) = TokenTransferUtils::get_token_info(&provider, contract_address).await {
        d
    } else {
        18
    };
    
    let balance_wei = u256_to_f64(balance);
    let balance_tokens = balance_wei / 10f64.powi(decimals as i32);
    println!("序号：{}, 当前代币余额为: {} ({} wei)", index, balance_tokens, balance);
    
    let gas_price = TransferUtils::get_gas_price(
        &TransferConfig {
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
        },
        provider.clone()
    ).await
        .map_err(|e| format!("获取Gas Price失败 (RPC: {}): {}", rpc_url, e))?;
    
    if gas_price.is_zero() {
        return Err("获取到的 gas price 为0".into());
    }
    
    let transfer_amount = match config.transfer_type.as_str() {
        "1" => balance,
        "2" => {
            let amount = parse_ether_to_wei_f64(config.transfer_amount)?
                * U256::from(10u64.pow(decimals as u32 - 18));
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "3" => {
            let mut rng = rand::thread_rng();
            let random_amount = rng.gen_range(config.transfer_amount_list[0]..=config.transfer_amount_list[1]);
            let formatted_amount = format!("{:.precision$}", random_amount, precision = config.amount_precision as usize);
            let precise_amount: f64 = formatted_amount.parse()?;
            let amount = parse_ether_to_wei_f64(precise_amount)?
                * U256::from(10u64.pow(decimals as u32 - 18));
            if amount >= balance {
                return Err("当前余额不足，不做转账操作！".into());
            }
            amount
        }
        "4" => {
            let balance_ether = balance_wei / 10f64.powi(decimals as i32);
            let gas_fee_wei = u256_to_f64(gas_price * U256::from(65000));
            let gas_fee_ether = gas_fee_wei / 1e18;
            let available_balance = balance_ether - gas_fee_ether;
            
            if available_balance <= config.left_amount_list[1] {
                return Err(format!("当前可用余额为：{}，无法满足最大剩余数量 {} 要求", available_balance, config.left_amount_list[1]).into());
            }
            
            let mut rng = rand::thread_rng();
            let left_amount = rng.gen_range(config.left_amount_list[0]..=config.left_amount_list[1]);
            let transfer_amount_f64 = available_balance - left_amount;
            
            if transfer_amount_f64 <= 0.0 {
                return Err("计算转账金额为负数或零，不做转账操作！".into());
            }
            
            let formatted_amount = format!("{:.precision$}", transfer_amount_f64, precision = config.amount_precision as usize);
            let precise_amount: f64 = formatted_amount.parse()?;
            parse_ether_to_wei_f64(precise_amount)? * U256::from(10u64.pow(decimals as u32 - 18))
        }
        _ => return Err("无效的转账类型".into()),
    };
    
    let gas_limit = TokenTransferUtils::get_contract_gas_limit(&config, provider.clone(), contract_address, wallet_address, to_address, transfer_amount).await?;
    let actual_gas_fee = gas_price * gas_limit;
    
    println!("序号：{}, 转账数量为: {}, gas_limit: {}, gas_price: {} gwei, gas_fee: {} ETH", 
        index, transfer_amount, gas_limit, format_wei_to_gwei(gas_price), format_wei_to_ether(actual_gas_fee));
    
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": "发送交易中...",
        "exec_status": "1"
    }));
    
    let method_id = "a9059cbb";
    let to_param = format!("{:0>64}", &hex::encode(to_address)[2..]);
    let amount_param = format!("{:0>64}", format!("{:x}", transfer_amount));
    let data = format!("0x{}{}{}", method_id, to_param, amount_param);
    
    let tx = TransactionRequest {
        to: Some(TxKind::Call(contract_address)),
        input: data.parse::<alloy_primitives::Bytes>().unwrap().into(),
        value: Some(U256::from(0)),
        gas: Some(gas_limit.to::<u64>()),
        gas_price: Some(gas_price.to::<u128>()),
        ..Default::default()
    };
    
    // 再次检查停止状态 - 在发送交易之前 (最关键的拦截点)
    if !window_id.is_empty() && get_stop_flag(window_id) {
        return Err("用户已停止转账任务".into());
    }
    
    let signer_provider = create_signer_provider(&config.chain, config.window_id.as_deref(), &signer).await?;
    let pending_tx = signer_provider.send_transaction(tx).await
        .map_err(|e| format!("发送交易失败: {}", e))?;
    
    let tx_hash = *pending_tx.tx_hash();
    let tx_hash_str = format!("{:?}", tx_hash);
    
    println!("序号：{}, 交易 hash 为：{:?}", index, tx_hash);
    
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": format!("已提交，等待确认: {}", &tx_hash_str[..20]),
        "exec_status": "1"
    }));
    
    Ok(tx_hash_str)
}

#[tauri::command]
pub async fn query_token_balance(
    chain: String,
    contract_address: String,
    address: String,
) -> Result<String, String> {
    let provider = create_provider(&chain, None).await
        .map_err(|e| format!("获取RPC提供商失败: {}", e))?;
    
    let contract_addr: Address = contract_address.parse()
        .map_err(|e| format!("合约地址格式错误: {}", e))?;
    let wallet_addr: Address = address.parse()
        .map_err(|e| format!("钱包地址格式错误: {}", e))?;
    
    let balance = TokenTransferUtils::get_token_balance(&provider, contract_addr, wallet_addr).await
        .map_err(|e| e.to_string())?;
    
    let decimals = if let Ok((_, d)) = TokenTransferUtils::get_token_info(&provider, contract_addr).await {
        d
    } else {
        18
    };
    
    let balance_wei = u256_to_f64(balance);
    let balance_tokens = balance_wei / 10f64.powi(decimals as i32);
    Ok(format!("{:.6}", balance_tokens))
}

#[tauri::command]
pub async fn get_token_info(
    chain: String,
    contract_address: String,
) -> Result<TokenInfo, String> {
    let provider = create_provider(&chain, None).await
        .map_err(|e| format!("获取RPC提供商失败: {}", e))?;
    
    let contract_addr: Address = contract_address.parse()
        .map_err(|e| format!("合约地址格式错误: {}", e))?;
    
    let (symbol, decimals) = TokenTransferUtils::get_token_info(&provider, contract_addr).await
        .map_err(|e| e.to_string())?;
    
    let wallet_address = "0x0000000000000000000000000000000000000000".parse::<Address>()
        .map_err(|e| format!("地址解析错误: {}", e))?;
    
    let balance = TokenTransferUtils::get_token_balance(&provider, contract_addr, wallet_address).await
        .map_err(|e| e.to_string())?;
    
    let balance_wei = u256_to_f64(balance);
    let balance_tokens = balance_wei / 10f64.powi(decimals as i32);
    
    Ok(TokenInfo {
        symbol,
        decimals,
        balance: format!("{:.6}", balance_tokens),
    })
}

#[tauri::command]
pub async fn token_transfer<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    item: TransferItem,
    config: TokenTransferConfig,
) -> Result<TransferResult, String> {
    TokenTransferUtils::token_transfer(app_handle, index, item, config).await
}

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

async fn token_transfer_fast_internal<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
    index: usize,
    mut item: TransferItem,
    config: TokenTransferConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    item.retry_flag = false;
    let window_id = config.window_id.as_deref().unwrap_or("");
    
    if item.private_key.trim().is_empty() {
        return Err("私钥不能为空！".into());
    }
    
    let private_key = if item.private_key.starts_with("0x") || item.private_key.starts_with("0X") {
        item.private_key[2..].to_string()
    } else {
        item.private_key.clone()
    };
    
    let signer: PrivateKeySigner = private_key.parse()
        .map_err(|e| format!("私钥格式错误: {}", e))?;
    
    let chain_id = match ProviderUtils::get_chain_id(&config.chain).await {
        Ok(id) => id,
        Err(_) => {
            match get_rpc_config(&config.chain).await {
                Some(c) => c.chain_id,
                None => {
                    return Err(format!("无法获取链 '{}' 的配置信息", config.chain).into());
                }
            }
        }
    };
    let signer = signer.with_chain_id(Some(chain_id));
    let wallet_address = signer.address();
    
    if item.to_addr.trim().is_empty() {
        return Err("目标地址不能为空".into());
    }
    let to_address: Address = item.to_addr.parse()
        .map_err(|e| format!("目标地址格式错误: {}", e))?;
    
    let contract_address: Address = config.contract_address.parse()
        .map_err(|e| format!("代币合约地址格式错误: {}", e))?;
    
    let provider = create_provider(&config.chain, config.window_id.as_deref()).await
        .map_err(|e| format!("获取RPC提供商失败: {}", e))?;
    
    let (symbol, decimals) = TokenTransferUtils::get_token_info(&provider, contract_address).await
        .map_err(|e| format!("获取代币信息失败: {}", e))?;
    println!("[DEBUG] 代币符号: {}, 精度: {}", symbol, decimals);
    
    let wallet_balance = TokenTransferUtils::get_token_balance(&provider, contract_address, wallet_address).await
        .map_err(|e| format!("获取钱包代币余额失败: {}", e))?;
    let wallet_balance_eth = u256_to_f64(wallet_balance) / 10f64.powi(decimals as i32);
    println!("[DEBUG] 钱包代币余额: {} {}", wallet_balance_eth, symbol);
    
    let transfer_amount = parse_ether_to_wei_f64(config.transfer_amount)
        .map_err(|e| format!("代币金额解析失败: {}", e))?;
    let transfer_amount_tokens = config.transfer_amount;
    println!("[DEBUG] 计划转账数量: {} {}", transfer_amount_tokens, symbol);
    
    if wallet_balance < transfer_amount {
        return Err(format!("余额不足，需要 {} {}，实际 {} {}", transfer_amount_tokens, symbol, wallet_balance_eth, symbol).into());
    }
    
    let gas_limit = TokenTransferUtils::get_contract_gas_limit(&config, provider.clone(), contract_address, wallet_address, to_address, transfer_amount).await?;
    println!("[DEBUG] 预估Gas Limit: {}", gas_limit);
    
    let gas_price = TransferUtils::get_gas_price(&TransferConfig {
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
    }, provider.clone()).await
        .map_err(|e| format!("获取Gas Price失败: {}", e))?;
    let gas_price_wei = parse_gwei_to_wei(u256_to_f64(gas_price));
    println!("[DEBUG] Gas Price: {} Gwei", gas_price);
    
    let _max_fee = gas_price_wei * U256::from(2);
    
    let method_id = "a9059cbb";
    let to_param = format!("{:0>64}", &hex::encode(to_address)[2..]);
    let amount_param = format!("{:0>64}", format!("{:x}", transfer_amount));
    let data = format!("0x{}{}{}", method_id, to_param, amount_param);
    
    let tx = TransactionRequest {
        to: Some(TxKind::Call(contract_address)),
        input: data.parse::<alloy_primitives::Bytes>().unwrap().into(),
        value: Some(U256::from(0)),
        gas: Some(gas_limit.to::<u64>()),
        gas_price: Some(gas_price.to::<u128>()),
        ..Default::default()
    };
    
    // 再次检查停止状态 - 在发送交易之前 (最关键的拦截点)
    if !window_id.is_empty() && get_stop_flag(window_id) {
        return Err("用户已停止转账任务".into());
    }

    let signer_provider = create_signer_provider(&config.chain, config.window_id.as_deref(), &signer).await?;
    let pending_tx = signer_provider.send_transaction(tx).await
        .map_err(|e| format!("发送交易失败: {}", e))?;
    
    let tx_hash = *pending_tx.tx_hash();
    let tx_hash_str = format!("{:?}", tx_hash);
    
    println!("序号：{}, 交易 hash 为：{:?}", index, tx_hash);
    
    let _ = app_handle.emit("transfer_status_update", serde_json::json!({
        "index": index - 1,
        "error_msg": format!("已提交，等待确认: {}", &tx_hash_str[..20]),
        "exec_status": "1"
    }));
    
    Ok(tx_hash_str)
}
