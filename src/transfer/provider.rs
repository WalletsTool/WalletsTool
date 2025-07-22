use ethers::{
    providers::{Http, JsonRpcClient, Provider},
    types::{Address, U256},
};
use rand::Rng;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::time::Duration;
use anyhow::Result;

use crate::transfer::config::*;

pub type EthProvider = Provider<Http>;

#[derive(Debug, Clone)]
pub struct ProviderManager {
    db_pool: Arc<SqlitePool>,
}

impl ProviderManager {
    pub fn new(db_pool: Arc<SqlitePool>) -> Self {
        Self { db_pool }
    }

    /// 从数据库获取随机 RPC URL 并创建 Provider
    pub async fn get_provider(&self, chain: &str) -> Result<Arc<EthProvider>, Box<dyn std::error::Error + Send + Sync>> {
        let rpc_url = self.get_random_rpc_url(chain).await?;
        let provider = Provider::<Http>::try_from(rpc_url.as_str())?;
        Ok(Arc::new(provider))
    }

    /// 从数据库获取指定链的随机 RPC URL
    pub async fn get_random_rpc_url(&self, chain_key: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            r#"
            SELECT rp.rpc_url 
            FROM rpc_providers rp
            JOIN chains c ON rp.chain_id = c.id
            WHERE c.chain_key = ? AND rp.is_active = TRUE
            ORDER BY rp.priority ASC
            "#
        )
        .bind(chain_key)
        .fetch_all(&*self.db_pool)
        .await?;

        if rows.is_empty() {
            return Err(format!("没有可用的RPC提供商: {}", chain_key).into());
        }

        let rpc_urls: Vec<String> = rows.into_iter().map(|row| {
            row.get::<String, _>("rpc_url")
        }).collect();

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..rpc_urls.len());
        Ok(rpc_urls[index].clone())
    }

    /// 获取指定链的最佳 RPC URL（按优先级和成功率）
    pub async fn get_best_rpc_url(&self, chain_key: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            r#"
            SELECT rp.rpc_url 
            FROM rpc_providers rp
            JOIN chains c ON rp.chain_id = c.id
            WHERE c.chain_key = ? AND rp.is_active = TRUE
            ORDER BY rp.priority ASC, rp.failure_count ASC, rp.avg_response_time_ms ASC NULLS LAST
            LIMIT 1
            "#
        )
        .bind(chain_key)
        .fetch_optional(&*self.db_pool)
        .await?
        .ok_or_else(|| format!("没有可用的RPC提供商: {}", chain_key))?;

        Ok(row.get::<String, _>("rpc_url"))
    }

    /// 获取链的所有 RPC URLs
    pub async fn get_all_rpc_urls(&self, chain_key: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            r#"
            SELECT rp.rpc_url 
            FROM rpc_providers rp
            JOIN chains c ON rp.chain_id = c.id
            WHERE c.chain_key = ? AND rp.is_active = TRUE
            ORDER BY rp.priority ASC
            "#
        )
        .bind(chain_key)
        .fetch_all(&*self.db_pool)
        .await?;

        let rpc_urls: Vec<String> = rows.into_iter().map(|row| {
            row.get::<String, _>("rpc_url")
        }).collect();

        Ok(rpc_urls)
    }

    /// 获取 Gas 价格
    pub async fn get_gas_price(&self, chain: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let provider = self.get_provider(chain).await?;
        let gas_price = provider.get_gas_price().await?;
        
        // 转换为 Gwei 单位
        let gas_price_gwei = gas_price.as_u128() as f64 / 1_000_000_000.0;
        Ok(gas_price_gwei)
    }

    /// 记录 RPC 请求成功
    pub async fn record_rpc_success(&self, rpc_url: &str, response_time_ms: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now();
        
        sqlx::query(
            r#"
            UPDATE rpc_providers 
            SET last_success_at = ?, 
                failure_count = 0,
                avg_response_time_ms = CASE 
                    WHEN avg_response_time_ms IS NULL THEN ?
                    ELSE (avg_response_time_ms + ?) / 2
                END,
                updated_at = ?
            WHERE rpc_url = ?
            "#
        )
        .bind(now)
        .bind(response_time_ms)
        .bind(response_time_ms)
        .bind(now)
        .bind(rpc_url)
        .execute(&*self.db_pool)
        .await?;

        Ok(())
    }

    /// 记录 RPC 请求失败
    pub async fn record_rpc_failure(&self, rpc_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now();
        
        // 增加失败计数
        sqlx::query(
            r#"
            UPDATE rpc_providers 
            SET failure_count = failure_count + 1,
                updated_at = ?
            WHERE rpc_url = ?
            "#
        )
        .bind(now)
        .bind(rpc_url)
        .execute(&*self.db_pool)
        .await?;

        // 如果失败次数过多（比如超过10次），暂时禁用该RPC
        sqlx::query(
            r#"
            UPDATE rpc_providers 
            SET is_active = FALSE,
                updated_at = ?
            WHERE rpc_url = ? AND failure_count >= 10
            "#
        )
        .bind(now)
        .bind(rpc_url)
        .execute(&*self.db_pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct GasConfig {
    pub gas_price: U256,
    pub gas_limit: U256,
}

pub struct GasCalculator;

impl GasCalculator {
    pub async fn calculate_gas_price(
        provider: Arc<EthProvider>,
        config: &TransferConfig,
    ) -> Result<U256, Box<dyn std::error::Error + Send + Sync>> {
        match config.gas_price_type {
            GasPriceType::Auto => {
                let gas_price = provider.get_gas_price().await?;
                
                // 检查最大 gas price 限制
                if let Some(max_gas_price) = &config.max_gas_price {
                    let max_gas_price_wei = Self::gwei_to_wei(max_gas_price.parse()?);
                    if gas_price > max_gas_price_wei {
                        return Err("Base gas price exceeds maximum limit".into());
                    }
                }
                
                Ok(gas_price)
            },
            GasPriceType::Fixed => {
                if let Some(gas_price_str) = &config.gas_price {
                    let gas_price_gwei: f64 = gas_price_str.parse()?;
                    Ok(Self::gwei_to_wei(gas_price_gwei))
                } else {
                    Err("Gas price not configured for fixed type".into())
                }
            },
            GasPriceType::Rate => {
                let base_gas_price = provider.get_gas_price().await?;
                let rate = config.gas_price_rate.unwrap_or(0.05);
                
                let gas_price_with_rate = (base_gas_price.as_u128() as f64 * (1.0 + rate)) as u128;
                let mut final_gas_price = U256::from(gas_price_with_rate);
                
                // 检查最大 gas price 限制
                if let Some(max_gas_price) = &config.max_gas_price {
                    let max_gas_price_wei = Self::gwei_to_wei(max_gas_price.parse()?);
                    
                    // 检查基础价格
                    if base_gas_price > max_gas_price_wei {
                        return Err("Base gas price exceeds maximum limit".into());
                    }
                    
                    // 如果计算后的价格超过上限，使用上限
                    if final_gas_price > max_gas_price_wei {
                        final_gas_price = max_gas_price_wei;
                    }
                }
                
                Ok(final_gas_price)
            }
        }
    }

    pub async fn calculate_gas_limit(
        provider: Arc<EthProvider>,
        config: &TransferConfig,
        from: Address,
        to: Address,
        value: Option<U256>,
    ) -> Result<U256, Box<dyn std::error::Error + Send + Sync>> {
        match config.limit_type {
            GasLimitType::Auto => {
                let tx = ethers::types::transaction::eip2718::TypedTransaction::Legacy(
                    ethers::types::transaction::request::TransactionRequest::new()
                        .from(from)
                        .to(to)
                        .value(value.unwrap_or(U256::zero()))
                );
                
                let gas_limit = provider.estimate_gas(&tx, None).await?;
                Ok(gas_limit)
            },
            GasLimitType::Fixed => {
                if let Some(limit) = config.limit_count {
                    Ok(U256::from(limit))
                } else {
                    Err("Gas limit not configured for fixed type".into())
                }
            },
            GasLimitType::Random => {
                if let Some([min, max]) = config.limit_count_list {
                    let mut rng = rand::thread_rng();
                    let gas_limit = rng.gen_range(min..=max);
                    Ok(U256::from(gas_limit))
                } else {
                    Err("Gas limit range not configured for random type".into())
                }
            }
        }
    }

    fn gwei_to_wei(gwei: f64) -> U256 {
        let wei = (gwei * 1_000_000_000.0) as u128;
        U256::from(wei)
    }
}
