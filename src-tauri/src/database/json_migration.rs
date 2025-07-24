use anyhow::Result;
use std::fs;
use std::path::Path;
use serde_json::Value;
use crate::database::{get_database_manager, chain_service::ChainService, models::*};

/// JSON配置迁移到数据库
#[allow(dead_code)]
pub async fn migrate_from_json() -> Result<()> {
    let chain_config_path = "conf/chain_setting.json";
    
    if Path::new(chain_config_path).exists() {
        migrate_chain_config_from_json(chain_config_path).await?;
    }
    
    // 迁移所有链的代币配置
    migrate_coin_configs_from_json().await?;
    
    Ok(())
}

/// 迁移链配置
async fn migrate_chain_config_from_json(file_path: &str) -> Result<()> {
    let content = fs::read_to_string(file_path)?;
    let json: Value = serde_json::from_str(&content)?;
    
    if let Some(chain_list) = json["chain_list"].as_array() {
        let db_manager = get_database_manager();
        let chain_service = ChainService::new(db_manager.get_pool());
        
        for chain_item in chain_list {
            if let (Some(key), Some(name)) = (
                chain_item["key"].as_str(),
                chain_item["name"].as_str()
            ) {
                // 检查是否已经存在
                if let Ok(Some(_)) = chain_service.get_chain_by_key(key).await {
                    continue; // 跳过已存在的链
                }
                
                let chain_id = chain_item["chain_id"].as_i64().unwrap_or(0);
                let symbol = chain_item["symbol"].as_str().unwrap_or("ETH");
                let currency_name = chain_item["currency_name"].as_str().unwrap_or("Ethereum");
                let decimals = chain_item["decimals"].as_i64().unwrap_or(18) as i32;
                
                let request = CreateChainRequest {
                    chain_key: key.to_string(),
                    chain_name: name.to_string(),
                    chain_id,
                    native_currency_symbol: symbol.to_string(),
                    native_currency_name: currency_name.to_string(),
                    native_currency_decimals: decimals,
                    pic_url: chain_item["pic_url"].as_str().map(|s| s.to_string()),
                    scan_url: chain_item["scan_url"].as_str().map(|s| s.to_string()),
                    scan_api: chain_item["scan_api"].as_str().map(|s| s.to_string()),
                    verify_api: chain_item["verify_api"].as_str().map(|s| s.to_string()),
                    check_verify_api: chain_item["check_verify_api"].as_str().map(|s| s.to_string()),
                    rpc_urls: chain_item["rpc_urls"].as_array().map(|urls| urls.iter().filter_map(|url| url.as_str().map(|s| s.to_string())).collect()),
                };
                
                match chain_service.add_chain(request).await {
                    Ok(_) => println!("成功迁移链配置: {}", key),
                    Err(e) => println!("迁移链配置失败 {}: {}", key, e),
                }
            }
        }
    }
    
    Ok(())
}

/// 迁移代币配置
async fn migrate_coin_configs_from_json() -> Result<()> {
    let chains = ["eth", "bsc", "avax", "zgs", "bevm", "sahara", "story", "manta", "linea", "base", "sol", "opbnb", "geth", "sepolia"];
    
    let db_manager = get_database_manager();
    let chain_service = ChainService::new(db_manager.get_pool());
    
    for chain_key in chains {
        let coin_config_path = format!("conf/chains/coin_{}_setting.json", chain_key);
        
        if Path::new(&coin_config_path).exists() {
            match migrate_chain_coins_from_json(&chain_service, chain_key, &coin_config_path).await {
                Ok(count) => println!("成功迁移链 {} 的 {} 个代币配置", chain_key, count),
                Err(e) => println!("迁移链 {} 的代币配置失败: {}", chain_key, e),
            }
        }
    }
    
    Ok(())
}

/// 迁移单个链的代币配置
async fn migrate_chain_coins_from_json(
    chain_service: &ChainService<'_>,
    chain_key: &str,
    file_path: &str,
) -> Result<usize> {
    let content = fs::read_to_string(file_path)?;
    let json: Value = serde_json::from_str(&content)?;
    
    let mut count = 0;
    
    if let Some(coin_list) = json["coin_list"].as_array() {
        for coin_item in coin_list {
            if let (Some(token_key), Some(name), Some(symbol)) = (
                coin_item["key"].as_str(),
                coin_item["name"].as_str(),
                coin_item["symbol"].as_str(),
            ) {
                // 检查是否已经存在
                if let Ok(Some(_)) = chain_service.get_token_info(chain_key, token_key).await {
                    continue; // 跳过已存在的代币
                }
                
                let contract_address = coin_item["contract_address"].as_str().map(|s| s.to_string());
                let decimals = coin_item["decimals"].as_i64().unwrap_or(18) as i32;
                let coin_type = coin_item["coin_type"].as_str().unwrap_or("token").to_string();
                
                let request = CreateTokenRequest {
                    chain_key: chain_key.to_string(),
                    token_key: token_key.to_string(),
                    token_name: name.to_string(),
                    symbol: symbol.to_string(),
                    contract_address,
                    decimals,
                    token_type: coin_type,
                    contract_type: coin_item["contract_type"].as_str().map(|s| s.to_string()),
                    abi: coin_item["abi"].as_str().map(|s| s.to_string()),
                };
                
                match chain_service.add_token(request).await {
                    Ok(_) => {
                        count += 1;
                        println!("  成功迁移代币: {} - {}", token_key, name);
                    },
                    Err(e) => println!("  迁移代币失败 {} - {}: {}", token_key, name, e),
                }
            }
        }
    }
    
    Ok(count)
}

/// 检查是否需要迁移
#[allow(dead_code)]
pub fn needs_migration() -> bool {
    let chain_config_path = "conf/chain_setting.json";
    Path::new(chain_config_path).exists()
}

/// 更新数据库中的链图标路径
#[allow(dead_code)]
pub async fn update_chain_pic_urls_from_json() -> Result<()> {
    let chain_config_path = "conf/chain_setting.json";
    
    if !Path::new(chain_config_path).exists() {
        return Err(anyhow::anyhow!("链配置文件不存在: {}", chain_config_path));
    }
    
    let content = fs::read_to_string(chain_config_path)?;
    let json: Value = serde_json::from_str(&content)?;
    
    if let Some(chain_list) = json["chain_list"].as_array() {
        let db_manager = get_database_manager();
        let pool = db_manager.get_pool();
        
        for chain_item in chain_list {
            if let (Some(key), Some(pic_url)) = (
                chain_item["key"].as_str(),
                chain_item["pic_url"].as_str()
            ) {
                // 更新数据库中的pic_url
                let result = sqlx::query(
                    "UPDATE chains SET pic_url = ?, updated_at = datetime('now') WHERE chain_key = ?"
                )
                .bind(pic_url)
                .bind(key)
                .execute(pool)
                .await?;
                
                if result.rows_affected() > 0 {
                    println!("成功更新链 {} 的图标路径: {}", key, pic_url);
                } else {
                    println!("警告: 链 {} 不存在于数据库中", key);
                }
            }
        }
    }
    
    Ok(())
}

/// 备份JSON配置文件
#[allow(dead_code)]
pub fn backup_json_configs() -> Result<()> {
    let backup_dir = "conf/backup";
    fs::create_dir_all(backup_dir)?;
    
    let chain_config_path = "conf/chain_setting.json";
    if Path::new(chain_config_path).exists() {
        let backup_path = format!("{}/chain_setting.json.bak", backup_dir);
        fs::copy(chain_config_path, backup_path)?;
        println!("已备份链配置文件");
    }
    
    let chains_dir = "conf/chains";
    if Path::new(chains_dir).exists() {
        let backup_chains_dir = format!("{}/chains", backup_dir);
        fs::create_dir_all(&backup_chains_dir)?;
        
        for entry in fs::read_dir(chains_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                if let Some(filename) = path.file_name() {
                    let backup_path = Path::new(&backup_chains_dir).join(format!("{}.bak", filename.to_string_lossy()));
                    fs::copy(&path, backup_path)?;
                }
            }
        }
        println!("已备份代币配置文件");
    }
    
    Ok(())
}
