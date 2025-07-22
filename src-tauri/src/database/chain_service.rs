use anyhow::Result;
use sqlx::{SqlitePool, Row};
use crate::database::models::*;
use chrono::Utc;

/// 链服务
pub struct ChainService<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ChainService<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// 获取所有活跃链的配置信息
    pub async fn get_all_chains(&self) -> Result<Vec<ChainInfo>> {
        let chains = sqlx::query_as::<_, Chain>(
            "SELECT * FROM chains WHERE is_active = TRUE ORDER BY chain_name"
        ).fetch_all(self.pool).await?;

        let mut chain_infos = Vec::new();
        for chain in chains {
            let rpc_urls = self.get_chain_rpc_urls(chain.id).await?;
            chain_infos.push(ChainInfo {
                key: chain.chain_key,
                chain: chain.chain_name,
                chain_id: chain.chain_id,
                symbol: chain.native_currency_symbol,
                currency_name: chain.native_currency_name,
                decimals: chain.native_currency_decimals,
                pic_url: chain.pic_url.unwrap_or_default(),
                scan_url: chain.scan_url.unwrap_or_default(),
                scan_api: chain.scan_api.unwrap_or_default(),
                verify_api: chain.verify_api.unwrap_or_default(),
                check_verify_api: chain.check_verify_api.unwrap_or_default(),
                rpc_urls,
            });
        }

        Ok(chain_infos)
    }

    /// 根据chain_key获取链信息
    pub async fn get_chain_by_key(&self, chain_key: &str) -> Result<Option<Chain>> {
        let chain = sqlx::query_as::<_, Chain>(
            "SELECT * FROM chains WHERE chain_key = ? AND is_active = TRUE"
        )
        .bind(chain_key)
        .fetch_optional(self.pool)
        .await?;

        Ok(chain)
    }

    /// 获取链的RPC URLs
    pub async fn get_chain_rpc_urls(&self, chain_id: i64) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT rpc_url FROM rpc_providers WHERE chain_id = ? AND is_active = TRUE ORDER BY priority ASC"
        )
        .bind(chain_id)
        .fetch_all(self.pool)
        .await?;

        let rpc_urls: Vec<String> = rows.into_iter().map(|row| {
            row.get::<String, _>("rpc_url")
        }).collect();

        Ok(rpc_urls)
    }

    /// 获取链的所有代币
    pub async fn get_chain_tokens(&self, chain_key: &str) -> Result<Vec<TokenInfo>> {
        let tokens = sqlx::query_as::<_, Token>(
            r#"
            SELECT t.* FROM tokens t
            JOIN chains c ON t.chain_id = c.id
            WHERE c.chain_key = ? AND t.is_active = TRUE
            ORDER BY t.token_type DESC, t.token_name
            "#
        )
        .bind(chain_key)
        .fetch_all(self.pool)
        .await?;

        let token_infos = tokens.into_iter().map(|token| TokenInfo {
            key: token.token_key,
            coin: token.token_name,
            coin_type: token.token_type,
            contract_address: token.contract_address,
            decimals: token.decimals,
            abi: token.abi, // 使用数据库中的abi字段
        }).collect();

        Ok(token_infos)
    }

    /// 添加新链
    pub async fn add_chain(&self, request: CreateChainRequest) -> Result<i64> {
        let now = Utc::now();
        let chain_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO chains (
                chain_key, chain_name, chain_id, native_currency_symbol, 
                native_currency_name, native_currency_decimals, pic_url, 
                scan_url, scan_api, verify_api, check_verify_api, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#
        )
        .bind(&request.chain_key)
        .bind(&request.chain_name)
        .bind(request.chain_id)
        .bind(&request.native_currency_symbol)
        .bind(&request.native_currency_name)
        .bind(request.native_currency_decimals)
        .bind(&request.pic_url)
        .bind(&request.scan_url)
        .bind(&request.scan_api)
        .bind(&request.verify_api)
        .bind(&request.check_verify_api)
        .bind(now)
        .bind(now)
        .fetch_one(self.pool)
        .await?;

        // 添加基础代币
        sqlx::query(
            r#"
            INSERT INTO tokens (
                chain_id, token_key, token_name, symbol, decimals, token_type, abi, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, 'base', NULL, ?, ?)
            "#
        )
        .bind(chain_id)
        .bind(format!("{}_base", request.chain_key))
        .bind(&request.native_currency_name)
        .bind(&request.native_currency_symbol)
        .bind(request.native_currency_decimals)
        .bind(now)
        .bind(now)
        .execute(self.pool)
        .await?;
        
        // 如果提供了RPC URL，添加默认RPC提供商
        if let Some(rpc_urls) = request.rpc_urls {
            for (index, rpc_url) in rpc_urls.iter().enumerate() {
                sqlx::query(
                    r#"
                    INSERT INTO rpc_providers (
                        chain_id, rpc_url, priority, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?)
                    "#
                )
                .bind(chain_id)
                .bind(rpc_url)
                .bind(index as i32 + 1)
                .bind(now)
                .bind(now)
                .execute(self.pool)
                .await?;
            }
        }

        Ok(chain_id)
    }
    
    /// 更新链信息
    pub async fn update_chain(&self, chain_key: &str, request: UpdateChainRequest) -> Result<()> {
        let now = Utc::now();
        
        // 检查链是否存在
        let chain = self.get_chain_by_key(chain_key).await?
            .ok_or_else(|| anyhow::anyhow!("链不存在: {}", chain_key))?;
        
        // 更新链基本信息
        sqlx::query(
            r#"
            UPDATE chains SET 
                chain_name = ?, chain_id = ?, native_currency_symbol = ?, 
                native_currency_name = ?, native_currency_decimals = ?, pic_url = ?, 
                scan_url = ?, scan_api = ?, verify_api = ?, check_verify_api = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&request.chain_name)
        .bind(request.chain_id)
        .bind(&request.native_currency_symbol)
        .bind(&request.native_currency_name)
        .bind(request.native_currency_decimals)
        .bind(&request.pic_url)
        .bind(&request.scan_url)
        .bind(&request.scan_api)
        .bind(&request.verify_api)
        .bind(&request.check_verify_api)
        .bind(now)
        .bind(chain.id)
        .execute(self.pool)
        .await?;
        
        // 更新基础代币信息
        sqlx::query(
            r#"
            UPDATE tokens SET 
                token_name = ?, symbol = ?, decimals = ?, updated_at = ?
            WHERE chain_id = ? AND token_type = 'base'
            "#
        )
        .bind(&request.native_currency_name)
        .bind(&request.native_currency_symbol)
        .bind(request.native_currency_decimals)
        .bind(now)
        .bind(chain.id)
        .execute(self.pool)
        .await?;
        
        // 如果提供了新的RPC URLs，先删除旧的，再添加新的
        if let Some(rpc_urls) = request.rpc_urls {
            // 删除旧的RPC提供商
            sqlx::query(
                "UPDATE rpc_providers SET is_active = FALSE, updated_at = ? WHERE chain_id = ?"
            )
            .bind(now)
            .bind(chain.id)
            .execute(self.pool)
            .await?;
            
            // 添加新的RPC提供商
            for (index, rpc_url) in rpc_urls.iter().enumerate() {
                sqlx::query(
                    r#"
                    INSERT INTO rpc_providers (
                        chain_id, rpc_url, priority, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?)
                    "#
                )
                .bind(chain.id)
                .bind(rpc_url)
                .bind(index as i32 + 1)
                .bind(now)
                .bind(now)
                .execute(self.pool)
                .await?;
            }
        }
        
        Ok(())
    }
    
    /// 删除链（软删除）
    pub async fn remove_chain(&self, chain_key: &str) -> Result<()> {
        let now = Utc::now();
        
        // 检查链是否存在
        let chain = self.get_chain_by_key(chain_key).await?
            .ok_or_else(|| anyhow::anyhow!("链不存在: {}", chain_key))?;
        
        // 软删除链
        sqlx::query(
            "UPDATE chains SET is_active = FALSE, updated_at = ? WHERE id = ?"
        )
        .bind(now)
        .bind(chain.id)
        .execute(self.pool)
        .await?;
        
        // 同时软删除链下的所有代币
        sqlx::query(
            "UPDATE tokens SET is_active = FALSE, updated_at = ? WHERE chain_id = ?"
        )
        .bind(now)
        .bind(chain.id)
        .execute(self.pool)
        .await?;
        
        // 同时软删除链下的所有RPC提供商
        sqlx::query(
            "UPDATE rpc_providers SET is_active = FALSE, updated_at = ? WHERE chain_id = ?"
        )
        .bind(now)
        .bind(chain.id)
        .execute(self.pool)
        .await?;
        
        Ok(())
    }

    /// 添加代币
    pub async fn add_token(&self, request: CreateTokenRequest) -> Result<i64> {
        // 获取链ID
        let chain = self.get_chain_by_key(&request.chain_key).await?
            .ok_or_else(|| anyhow::anyhow!("链不存在: {}", request.chain_key))?;

        let now = Utc::now();
        let token_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO tokens (
                chain_id, token_key, token_name, symbol, contract_address, 
                decimals, token_type, abi, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#
        )
        .bind(chain.id)
        .bind(&request.token_key)
        .bind(&request.token_name)
        .bind(&request.symbol)
        .bind(&request.contract_address)
        .bind(request.decimals)
        .bind(&request.token_type)
        .bind(&request.abi)
        .bind(now)
        .bind(now)
        .fetch_one(self.pool)
        .await?;

        Ok(token_id)
    }

    /// 删除代币
    pub async fn remove_token(&self, chain_key: &str, token_key: &str) -> Result<()> {
        // SQLite不支持UPDATE...FROM语法，需要使用子查询
        sqlx::query(
            r#"
            UPDATE tokens SET is_active = FALSE, updated_at = ?
            WHERE tokens.chain_id IN (
                SELECT id FROM chains WHERE chain_key = ?
            ) AND tokens.token_key = ?
            "#
        )
        .bind(Utc::now())
        .bind(chain_key)
        .bind(token_key)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    /// 获取代币信息（用于查询余额）
    #[allow(dead_code)]
    pub async fn get_token_info(&self, chain_key: &str, token_key: &str) -> Result<Option<Token>> {
        let token = sqlx::query_as::<_, Token>(
            r#"
            SELECT t.* FROM tokens t
            JOIN chains c ON t.chain_id = c.id
            WHERE c.chain_key = ? AND t.token_key = ? AND t.is_active = TRUE
            "#
        )
        .bind(chain_key)
        .bind(token_key)
        .fetch_optional(self.pool)
        .await?;

        Ok(token)
    }
    
    /// 更新代币的ABI
    pub async fn update_token_abi(&self, chain_key: &str, token_key: &str, abi: Option<String>) -> Result<()> {
        // 获取链信息验证链是否存在
        let chain = self.get_chain_by_key(chain_key).await?
            .ok_or_else(|| anyhow::anyhow!("链不存在: {}", chain_key))?;
        
        let rows_affected = sqlx::query(
            r#"
            UPDATE tokens SET abi = ?, updated_at = ?
            WHERE chain_id = ? AND token_key = ? AND is_active = TRUE
            "#
        )
        .bind(&abi)
        .bind(Utc::now())
        .bind(chain.id)
        .bind(token_key)
        .execute(self.pool)
        .await?
        .rows_affected();
        
        if rows_affected == 0 {
            return Err(anyhow::anyhow!("代币不存在或已禁用: {}/{}", chain_key, token_key));
        }
        
        Ok(())
    }
}
