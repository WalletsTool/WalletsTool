use serde_json::{json, Value};
use tauri::command;
use crate::database::{get_database_manager, chain_service::ChainService, models::*};
use anyhow::Result;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[command]
pub async fn get_chain_list() -> Vec<Value> {
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    match chain_service.get_all_chains().await {
        Ok(chains) => {
            chains.into_iter().map(|chain| {
                json!({
                    "key": chain.key,
                    "name": chain.chain,
                    "chain_id": chain.chain_id,
                    "symbol": chain.symbol,
                    "currency_name": chain.currency_name,
                    "decimals": chain.decimals,
                    "pic_url": chain.pic_url,
                    "scan_url": chain.scan_url,
                    "scan_api": chain.scan_api,
                    "verify_api": chain.verify_api,
                    "check_verify_api": chain.check_verify_api,
                    "rpc_urls": chain.rpc_urls
                })
            }).collect()
        },
        Err(e) => {
            eprintln!("获取链列表失败: {:?}", e);
            Vec::new()
        }
    }
}

#[command]
pub async fn get_coin_list(chain: &str) -> Result<Vec<Value>, String> {
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    match chain_service.get_chain_tokens(chain).await {
        Ok(tokens) => {
            Ok(tokens.into_iter().map(|token| {
                json!({
                    "key": token.key,
                    "label": token.coin,
                    "symbol": token.key, // TokenInfo doesn't have symbol field, use key instead
                    "contract_address": token.contract_address,
                    "decimals": token.decimals,
                    "coin_type": token.coin_type,
                    "abi": token.abi
                })
            }).collect())
        },
        Err(e) => {
            eprintln!("获取代币列表失败: {:?}", e);
            Ok(Vec::new())
        }
    }
}

#[command]
pub async fn add_coin(chain: &str, obj_json: &str) -> Result<(), String> {
    let coin_data: Value = serde_json::from_str(obj_json)
        .map_err(|e| format!("解析JSON失败: {}", e))?;
    
    let request = CreateTokenRequest {
        chain_key: chain.to_string(),
        token_key: coin_data["key"].as_str().unwrap_or("").to_string(),
        token_name: coin_data["name"].as_str().unwrap_or("").to_string(),
        symbol: coin_data["symbol"].as_str().unwrap_or("").to_string(),
        contract_address: coin_data["contract_address"].as_str().map(|s| s.to_string()),
        decimals: coin_data["decimals"].as_i64().unwrap_or(18) as i32,
        token_type: coin_data["coin_type"].as_str().unwrap_or("token").to_string(),
        abi: coin_data["abi"].as_str().map(|s| s.to_string()),
    };
    
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    chain_service.add_token(request).await
        .map_err(|e| format!("添加代币失败: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn remove_coin(chain: &str, key: &str) -> Result<(), String> {
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    chain_service.remove_token(chain, key).await
        .map_err(|e| format!("删除代币失败: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn update_chain_pic_urls() -> Result<(), String> {
    // 暂时移除该功能，因为此前JSON配置已迁移到数据库
    // update_chain_pic_urls_from_json().await
    //     .map_err(|e| format!("更新链图标路径失败: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn update_token_abi(chain: &str, token_key: &str, abi: Option<String>) -> Result<(), String> {
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    chain_service.update_token_abi(chain, token_key, abi).await
        .map_err(|e| format!("更新代币ABI失败: {}", e))?;
    
    Ok(())
}

/// 添加新链
#[command]
pub async fn add_chain(request_json: &str) -> Result<i64, String> {
    let request: CreateChainRequest = serde_json::from_str(request_json)
        .map_err(|e| format!("解析请求JSON失败: {}", e))?;
    
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    chain_service.add_chain(request).await
        .map_err(|e| format!("添加链失败: {}", e))
}

/// 删除链
#[command]
pub async fn remove_chain(chain_key: &str) -> Result<(), String> {
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    chain_service.remove_chain(chain_key).await
        .map_err(|e| format!("删除链失败: {}", e))
}

/// 获取链的详细信息（用于编辑）
#[command]
pub async fn get_chain_detail(chain_key: &str) -> Result<Option<Value>, String> {
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    match chain_service.get_chain_by_key(chain_key).await {
        Ok(Some(chain)) => {
            let rpc_urls = chain_service.get_chain_rpc_urls(chain.id).await
                .map_err(|e| format!("获取RPC URLs失败: {}", e))?;
            
            Ok(Some(json!({
                "id": chain.id,
                "chain_key": chain.chain_key,
                "chain_name": chain.chain_name,
                "chain_id": chain.chain_id,
                "native_currency_symbol": chain.native_currency_symbol,
                "native_currency_name": chain.native_currency_name,
                "native_currency_decimals": chain.native_currency_decimals,
                "pic_url": chain.pic_url,
                "scan_url": chain.scan_url,
                "scan_api": chain.scan_api,
                "verify_api": chain.verify_api,
                "check_verify_api": chain.check_verify_api,
                "rpc_urls": rpc_urls
            })))
        },
        Ok(None) => Ok(None),
        Err(e) => Err(format!("获取链详情失败: {}", e))
    }
}

/// 更新链信息
#[command]
pub async fn update_chain(chain_key: &str, request_json: &str) -> Result<(), String> {
    let request: UpdateChainRequest = serde_json::from_str(request_json)
        .map_err(|e| format!("解析请求JSON失败: {}", e))?;
    
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    chain_service.update_chain(chain_key, request).await
        .map_err(|e| format!("更新链失败: {}", e))
}
