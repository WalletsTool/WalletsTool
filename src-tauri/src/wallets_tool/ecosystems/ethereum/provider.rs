use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::Rng;
use ethers::{
    providers::{Http, Provider, Middleware},
    utils::format_units,
};
use std::sync::Arc;
use crate::database::{get_database_manager, chain_service::ChainService};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChainRpcConfig {
    pub name: String,
    pub chain_id: u64,
    pub rpc_urls: Vec<String>,
    pub currency_symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasPriceInfo {
    pub gas_price_gwei: f64,
    pub chain: String,
}

// 获取所有支持的链配置（从数据库查询）
pub async fn get_all_chain_configs() -> Result<HashMap<String, ChainRpcConfig>, Box<dyn std::error::Error>> {
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    let chain_infos = chain_service.get_all_chains().await?;
    let mut configs = HashMap::new();
    
    for chain_info in chain_infos {
        configs.insert(chain_info.key.clone(), ChainRpcConfig {
            name: chain_info.chain,
            chain_id: chain_info.chain_id as u64,
            rpc_urls: chain_info.rpc_urls,
            currency_symbol: chain_info.symbol,
        });
    }
    
    Ok(configs)
}

// Provider工具类
pub struct ProviderUtils;

impl ProviderUtils {
    // 获取单个链配置
    pub async fn get_chain_config(chain: &str) -> Result<ChainRpcConfig, String> {
        let configs = get_all_chain_configs()
            .await
            .map_err(|e| e.to_string())?;
        configs
            .get(chain)
            .cloned()
            .ok_or_else(|| format!("不支持的链: {}", chain))
    }

    // 获取链ID
    pub async fn get_chain_id(chain: &str) -> Result<u64, String> {
        let cfg = Self::get_chain_config(chain).await?;
        Ok(cfg.chain_id)
    }
    // 获取指定链的Provider
    pub async fn get_provider(chain: &str) -> Result<Arc<Provider<Http>>, Box<dyn std::error::Error>> {
        let configs = get_all_chain_configs().await?;
        let config = configs.get(chain)
            .ok_or(format!("不支持的链: {}", chain))?;
        
        // 随机选择一个RPC节点
        let mut rng = rand::thread_rng();
        let rpc_url = &config.rpc_urls[rng.gen_range(0..config.rpc_urls.len())];
        
        let provider = Provider::<Http>::try_from(rpc_url)?;
        Ok(Arc::new(provider))
    }
    
    // 获取基础Gas Price
    pub async fn get_base_gas_price(chain: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let provider = Self::get_provider(chain).await?;
        let gas_price = provider.get_gas_price().await?;
        let gas_price_gwei = format_units(gas_price, "gwei")?.parse::<f64>()?;
        Ok(gas_price_gwei)
    }
    
    // 测试RPC连接
    pub async fn test_rpc_connection(rpc_url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let _block_number = provider.get_block_number().await?;
        Ok(true)
    }
}

// Tauri命令：获取指定链的Gas Price
#[tauri::command]
pub async fn get_chain_gas_price(chain: String) -> Result<GasPriceInfo, String> {
    match ProviderUtils::get_base_gas_price(&chain).await {
        Ok(gas_price) => Ok(GasPriceInfo {
            gas_price_gwei: gas_price,
            chain,
        }),
        Err(e) => Err(e.to_string()),
    }
}

// Tauri命令：测试RPC连接
#[tauri::command]
pub async fn test_rpc_url(rpc_url: String) -> Result<bool, String> {
    match ProviderUtils::test_rpc_connection(&rpc_url).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

// Tauri命令：获取多个链的Gas Price
#[tauri::command]
pub async fn get_multiple_gas_prices(chains: Vec<String>) -> Result<Vec<GasPriceInfo>, String> {
    let mut results = Vec::new();
    
    for chain in chains {
        match ProviderUtils::get_base_gas_price(&chain).await {
            Ok(gas_price) => {
                results.push(GasPriceInfo {
                    gas_price_gwei: gas_price,
                    chain: chain.clone(),
                });
            }
            Err(e) => {
                eprintln!("获取{}链Gas Price失败: {}", chain, e);
                // 继续处理其他链，不中断整个流程
                results.push(GasPriceInfo {
                    gas_price_gwei: 0.0,
                    chain: chain.clone(),
                });
            }
        }
    }
    
    Ok(results)
}