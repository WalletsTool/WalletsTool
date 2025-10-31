use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::Rng;
use ethers::{
    providers::{Http, Provider, Middleware},
    utils::format_units,
};
use url::Url;
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
    
    // 随机选择RPC URL
    fn get_random_rpc_url(rpc_urls: &[String]) -> Result<&str, String> {
        if rpc_urls.is_empty() {
            return Err("没有可用的RPC URL".to_string());
        }
        let mut rng = rand::thread_rng();
        Ok(&rpc_urls[rng.gen_range(0..rpc_urls.len())])
    }
    // 获取指定链的Provider
    pub async fn get_provider(chain: &str) -> Result<Provider<Http>, Box<dyn std::error::Error>> {
        use crate::wallets_tool::ecosystems::ethereum::proxy_manager::PROXY_MANAGER;
        
        println!("[DEBUG] get_provider - 开始获取链 '{}' 的Provider", chain);
        
        let chain_config = Self::get_chain_config(chain).await?;
        println!("[DEBUG] get_provider - 获取到链配置，chain_id: {}, rpc_urls数量: {}", 
                 chain_config.chain_id, chain_config.rpc_urls.len());
        
        let rpc_url = Self::get_random_rpc_url(&chain_config.rpc_urls)
            .map_err(|e| format!("获取RPC URL失败: {}. 请检查链 '{}' 是否配置了RPC节点。", e, chain))?;
        println!("[DEBUG] get_provider - 选择的RPC URL: {}", rpc_url);
        
        // 尝试使用代理客户端，如果没有代理则使用默认方式
        let provider = if let Some(proxy_client) = PROXY_MANAGER.get_random_proxy_client() {
            println!("[DEBUG] get_provider - 使用代理客户端创建Provider");
            let url: Url = rpc_url.parse()
                .map_err(|e| format!("Failed to parse RPC URL: {}", e))?;
            let http_provider = Http::new_with_client(url, proxy_client);
            Provider::new(http_provider)
        } else {
            println!("[DEBUG] get_provider - 使用默认方式创建Provider");
            Provider::<Http>::try_from(rpc_url)
                .map_err(|e| {
                    println!("[ERROR] get_provider - Provider创建失败: {}", e);
                    e
                })?
        };
        
        println!("[DEBUG] get_provider - Provider创建成功");
        Ok(provider)
    }
    
    // 获取基础Gas Price
    pub async fn get_base_gas_price(chain: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let provider = Arc::new(Self::get_provider(chain).await?);
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