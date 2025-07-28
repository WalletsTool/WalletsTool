use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::Rng;
use ethers::{
    providers::{Http, Provider, Middleware},
    utils::format_units,
};
use std::sync::Arc;

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

// 获取所有支持的链配置
pub fn get_all_chain_configs() -> HashMap<String, ChainRpcConfig> {
    let mut configs = HashMap::new();
    
    // Ethereum Mainnet
    configs.insert("eth".to_string(), ChainRpcConfig {
        name: "Ethereum".to_string(),
        chain_id: 1,
        rpc_urls: vec![
            "https://rpc.ankr.com/eth/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f".to_string(),
            "https://eth-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5".to_string(),
            "https://lb.drpc.org/ogrpc?network=ethereum&dkey=Aj6S6lY4rEHYqHuH8SYuK888OpJEh1oR7qTOrkUU-y5L".to_string(),
            "https://1rpc.io/eth".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    // Binance Smart Chain
    configs.insert("bsc".to_string(), ChainRpcConfig {
        name: "Binance Smart Chain".to_string(),
        chain_id: 56,
        rpc_urls: vec![
            "https://bsc-dataseed1.bnbchain.org".to_string(),
            "https://bsc.publicnode.com".to_string(),
            "https://bsc.drpc.org".to_string(),
            "https://rpc.ankr.com/bsc".to_string(),
            "https://binance.llamarpc.com".to_string(),
        ],
        currency_symbol: "BNB".to_string(),
    });
    
    // Polygon
    configs.insert("polygon".to_string(), ChainRpcConfig {
        name: "Polygon".to_string(),
        chain_id: 137,
        rpc_urls: vec![
            "https://polygon-bor.publicnode.com".to_string(),
            "https://polygon-rpc.com".to_string(),
            "https://rpc.ankr.com/polygon".to_string(),
            "https://polygon.drpc.org".to_string(),
            "https://polygon.lava.build".to_string(),
        ],
        currency_symbol: "MATIC".to_string(),
    });
    
    // Arbitrum One
    configs.insert("arb".to_string(), ChainRpcConfig {
        name: "Arbitrum One".to_string(),
        chain_id: 42161,
        rpc_urls: vec![
            "https://arbitrum-one.public.blastapi.io".to_string(),
            "https://rpc.ankr.com/arbitrum".to_string(),
            "https://1rpc.io/arb".to_string(),
            "https://arbitrum.blockpi.network/v1/rpc/public".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    // Optimism
    configs.insert("op".to_string(), ChainRpcConfig {
        name: "Optimism".to_string(),
        chain_id: 10,
        rpc_urls: vec![
            "https://rpc.ankr.com/optimism".to_string(),
            "https://opt-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5".to_string(),
            "https://optimism.drpc.org".to_string(),
            "https://optimism.llamarpc.com".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    // Base
    configs.insert("base".to_string(), ChainRpcConfig {
        name: "Base".to_string(),
        chain_id: 8453,
        rpc_urls: vec![
            "https://base.publicnode.com".to_string(),
            "https://1rpc.io/base".to_string(),
            "https://base.llamarpc.com".to_string(),
            "https://base.drpc.org".to_string(),
            "https://base.meowrpc.com".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    // Linea
    configs.insert("linea".to_string(), ChainRpcConfig {
        name: "Linea".to_string(),
        chain_id: 59144,
        rpc_urls: vec![
            "https://linea.decubate.com".to_string(),
            "https://1rpc.io/linea".to_string(),
            "https://rpc.linea.build".to_string(),
            "https://linea.drpc.org".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    // Scroll
    configs.insert("scroll".to_string(), ChainRpcConfig {
        name: "Scroll".to_string(),
        chain_id: 534352,
        rpc_urls: vec![
            "https://scroll.drpc.org".to_string(),
            "https://1rpc.io/scroll".to_string(),
            "https://rpc.ankr.com/scroll".to_string(),
            "https://rpc.scroll.io".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    // Avalanche
    configs.insert("avax".to_string(), ChainRpcConfig {
        name: "Avalanche".to_string(),
        chain_id: 43114,
        rpc_urls: vec![
            "https://api.avax.network/ext/bc/C/rpc".to_string(),
            "https://avalanche.drpc.org".to_string(),
            "https://avax.meowrpc.com".to_string(),
            "https://1rpc.io/avax/c".to_string(),
        ],
        currency_symbol: "AVAX".to_string(),
    });
    
    // OpBNB
    configs.insert("opbnb".to_string(), ChainRpcConfig {
        name: "OpBNB".to_string(),
        chain_id: 204,
        rpc_urls: vec![
            "https://opbnb-mainnet-rpc.bnbchain.org".to_string(),
            "https://opbnb-mainnet.nodereal.io/v1/ea08c11bd0874ce19cee7fc6f63b6cf8".to_string(),
        ],
        currency_symbol: "BNB".to_string(),
    });
    
    // Manta Pacific
    configs.insert("manta".to_string(), ChainRpcConfig {
        name: "Manta Pacific".to_string(),
        chain_id: 169,
        rpc_urls: vec![
            "https://1rpc.io/manta".to_string(),
            "https://pacific-rpc.manta.network/http".to_string(),
            "https://manta-pacific.drpc.org".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    // zkSync Era
    configs.insert("zksync".to_string(), ChainRpcConfig {
        name: "zkSync Era".to_string(),
        chain_id: 324,
        rpc_urls: vec![
            "https://rpc.ankr.com/zksync_era".to_string(),
            "https://mainnet.era.zksync.io".to_string(),
            "https://zksync.drpc.org".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    // Sepolia Testnet
    configs.insert("sepolia".to_string(), ChainRpcConfig {
        name: "Sepolia Testnet".to_string(),
        chain_id: 11155111,
        rpc_urls: vec![
            "https://1rpc.io/sepolia".to_string(),
            "https://sepolia.gateway.tenderly.co".to_string(),
            "https://ethereum-sepolia.publicnode.com".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    // Holesky Testnet
    configs.insert("holesky".to_string(), ChainRpcConfig {
        name: "Holesky Testnet".to_string(),
        chain_id: 17000,
        rpc_urls: vec![
            "https://ethereum-holesky.blockpi.network/v1/rpc/public".to_string(),
            "https://rpc.holesky.ethpandaops.io".to_string(),
            "https://holesky.drpc.org".to_string(),
        ],
        currency_symbol: "ETH".to_string(),
    });
    
    configs
}

// Provider工具类
pub struct ProviderUtils;

impl ProviderUtils {
    // 获取指定链的Provider
    pub async fn get_provider(chain: &str) -> Result<Arc<Provider<Http>>, Box<dyn std::error::Error>> {
        let configs = get_all_chain_configs();
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

// Tauri命令：获取支持的链列表
#[tauri::command]
pub async fn get_supported_chains() -> Result<HashMap<String, ChainRpcConfig>, String> {
    Ok(get_all_chain_configs())
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

// Tauri命令：获取链信息
#[tauri::command]
pub async fn get_chain_info(chain: String) -> Result<ChainRpcConfig, String> {
    let configs = get_all_chain_configs();
    configs.get(&chain)
        .cloned()
        .ok_or(format!("不支持的链: {}", chain))
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