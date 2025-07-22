use anyhow::Result;
use sqlx::{SqlitePool, Row};
use crate::database::models::*;
use chrono::Utc;
use rand::prelude::*;

/// RPC服务
pub struct RpcService<'a> {
    pool: &'a SqlitePool,
}

impl<'a> RpcService<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// 为指定链获取最佳RPC URL（按优先级和成功率）
    pub async fn get_best_rpc_url(&self, chain_key: &str) -> Result<String> {
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
        .fetch_optional(self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("没有可用的RPC提供商: {}", chain_key))?;

        Ok(row.get::<String, _>("rpc_url"))
    }

    /// 获取指定链的随机RPC URL（用于负载均衡）
    pub async fn get_random_rpc_url(&self, chain_key: &str) -> Result<String> {
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
        .fetch_all(self.pool)
        .await?;

        if rows.is_empty() {
            return Err(anyhow::anyhow!("没有可用的RPC提供商: {}", chain_key));
        }

        let rpc_urls: Vec<String> = rows.into_iter().map(|row| {
            row.get::<String, _>("rpc_url")
        }).collect();

        let mut rng = thread_rng();
        let index = rng.gen_range(0..rpc_urls.len());
        Ok(rpc_urls[index].clone())
    }

    /// 获取链的所有RPC URLs
    pub async fn get_all_rpc_urls(&self, chain_key: &str) -> Result<Vec<String>> {
        let rpc_urls = sqlx::query!(
            r#"
            SELECT rp.rpc_url 
            FROM rpc_providers rp
            JOIN chains c ON rp.chain_id = c.id
            WHERE c.chain_key = ? AND rp.is_active = TRUE
            ORDER BY rp.priority ASC
            "#,
            chain_key
        )
        .fetch_all(self.pool)
        .await?;

        Ok(rpc_urls.into_iter().map(|r| r.rpc_url).collect())
    }

    /// 添加RPC提供商
    pub async fn add_rpc_provider(&self, request: CreateRpcProviderRequest) -> Result<i64> {
        // 获取链ID
        let chain = sqlx::query!(
            "SELECT id FROM chains WHERE chain_key = ? AND is_active = TRUE",
            request.chain_key
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("链不存在: {}", request.chain_key))?;

        let now = Utc::now();
        let rpc_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO rpc_providers (
                chain_id, rpc_url, priority, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "#
        )
        .bind(chain.id)
        .bind(&request.rpc_url)
        .bind(request.priority)
        .bind(now)
        .bind(now)
        .fetch_one(self.pool)
        .await?;

        Ok(rpc_id)
    }

    /// 记录RPC请求成功
    pub async fn record_rpc_success(&self, rpc_url: &str, response_time_ms: i32) -> Result<()> {
        let now = Utc::now();
        
        // 更新成功时间和响应时间，重置失败计数
        sqlx::query!(
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
            "#,
            now,
            response_time_ms,
            response_time_ms,
            now,
            rpc_url
        ).execute(self.pool).await?;

        Ok(())
    }

    /// 记录RPC请求失败
    pub async fn record_rpc_failure(&self, rpc_url: &str) -> Result<()> {
        let now = Utc::now();
        
        // 增加失败计数
        sqlx::query!(
            r#"
            UPDATE rpc_providers 
            SET failure_count = failure_count + 1,
                updated_at = ?
            WHERE rpc_url = ?
            "#,
            now,
            rpc_url
        ).execute(self.pool).await?;

        // 如果失败次数过多（比如超过10次），暂时禁用该RPC
        sqlx::query!(
            r#"
            UPDATE rpc_providers 
            SET is_active = FALSE,
                updated_at = ?
            WHERE rpc_url = ? AND failure_count >= 10
            "#,
            now,
            rpc_url
        ).execute(self.pool).await?;

        Ok(())
    }

    /// 重新激活失败的RPC提供商（可以定期调用）
    pub async fn reactivate_failed_rpcs(&self) -> Result<()> {
        let now = Utc::now();
        
        // 重新激活24小时前失败的RPC（给它们重新尝试的机会）
        sqlx::query!(
            r#"
            UPDATE rpc_providers 
            SET is_active = TRUE,
                failure_count = 0,
                updated_at = ?
            WHERE is_active = FALSE AND updated_at < datetime('now', '-1 day')
            "#,
            now
        ).execute(self.pool).await?;

        Ok(())
    }

    /// 记录余额查询历史
    pub async fn record_balance_query(
        &self,
        address: &str,
        chain_key: &str,
        token_key: Option<&str>,
        balance: &str,
        nonce: Option<i64>,
        status: &str,
        error_message: Option<&str>,
        rpc_url: &str,
        response_time_ms: Option<i32>,
    ) -> Result<()> {
        // 获取链ID
        let chain = sqlx::query!(
            "SELECT id FROM chains WHERE chain_key = ?",
            chain_key
        )
        .fetch_one(self.pool)
        .await?;

        // 获取代币ID（如果指定了代币）
        let token_id = if let Some(token_key) = token_key {
            let token = sqlx::query!(
                r#"
                SELECT t.id FROM tokens t
                JOIN chains c ON t.chain_id = c.id
                WHERE c.chain_key = ? AND t.token_key = ?
                "#,
                chain_key,
                token_key
            )
            .fetch_optional(self.pool)
            .await?;
            
            token.map(|t| t.id)
        } else {
            None
        };

        sqlx::query!(
            r#"
            INSERT INTO balance_history (
                address, chain_id, token_id, balance, nonce, query_status, 
                error_message, rpc_url, response_time_ms
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            address,
            chain.id,
            token_id,
            balance,
            nonce,
            status,
            error_message,
            rpc_url,
            response_time_ms
        ).execute(self.pool).await?;

        Ok(())
    }

    /// 获取地址的余额查询历史
    pub async fn get_balance_history(&self, address: &str, limit: i32) -> Result<Vec<BalanceHistory>> {
        let history = sqlx::query_as::<_, BalanceHistory>(
            r#"
            SELECT * FROM balance_history 
            WHERE address = ? 
            ORDER BY created_at DESC 
            LIMIT ?
            "#
        )
        .bind(address)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(history)
    }

    /// 获取RPC提供商统计信息
    pub async fn get_rpc_stats(&self, chain_key: &str) -> Result<Vec<RpcProvider>> {
        let stats = sqlx::query_as::<_, RpcProvider>(
            r#"
            SELECT rp.* FROM rpc_providers rp
            JOIN chains c ON rp.chain_id = c.id
            WHERE c.chain_key = ?
            ORDER BY rp.priority ASC, rp.failure_count ASC
            "#
        )
        .bind(chain_key)
        .fetch_all(self.pool)
        .await?;

        Ok(stats)
    }
}
