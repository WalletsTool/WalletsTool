use sqlx::SqlitePool;
use anyhow::Result;

#[allow(dead_code)]
/// 运行所有数据库迁移
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    create_chains_table(pool).await?;
    create_rpc_providers_table(pool).await?;
    create_tokens_table(pool).await?;
    create_balance_history_table(pool).await?;
    add_abi_column_to_tokens_table(pool).await?;
    insert_default_data(pool).await?;
    
    Ok(())
}

/// 创建链配置表
async fn create_chains_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chains (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chain_key TEXT NOT NULL UNIQUE,
            chain_name TEXT NOT NULL,
            chain_id INTEGER NOT NULL,
            native_currency_symbol TEXT NOT NULL,
            native_currency_name TEXT NOT NULL,
            native_currency_decimals INTEGER NOT NULL DEFAULT 18,
            pic_url TEXT,
            scan_url TEXT,
            scan_api TEXT,
            verify_api TEXT,
            check_verify_api TEXT,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#
    ).execute(pool).await?;
    
    Ok(())
}

/// 创建RPC提供商表
async fn create_rpc_providers_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS rpc_providers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chain_id INTEGER NOT NULL,
            rpc_url TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            priority INTEGER NOT NULL DEFAULT 100,
            last_success_at DATETIME,
            failure_count INTEGER NOT NULL DEFAULT 0,
            avg_response_time_ms INTEGER,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (chain_id) REFERENCES chains(id) ON DELETE CASCADE
        );
        "#
    ).execute(pool).await?;
    
    // 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_rpc_providers_chain_id ON rpc_providers(chain_id);")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_rpc_providers_priority ON rpc_providers(priority);")
        .execute(pool).await?;
    
    Ok(())
}

/// 创建代币表
async fn create_tokens_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chain_id INTEGER NOT NULL,
            token_key TEXT NOT NULL,
            token_name TEXT NOT NULL,
            symbol TEXT NOT NULL,
            contract_address TEXT,
            decimals INTEGER NOT NULL DEFAULT 18,
            token_type TEXT NOT NULL CHECK(token_type IN ('base', 'token')),
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (chain_id) REFERENCES chains(id) ON DELETE CASCADE,
            UNIQUE(chain_id, token_key)
        );
        "#
    ).execute(pool).await?;
    
    // 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tokens_chain_id ON tokens(chain_id);")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tokens_symbol ON tokens(symbol);")
        .execute(pool).await?;
    
    Ok(())
}

/// 创建余额历史表
async fn create_balance_history_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS balance_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            address TEXT NOT NULL,
            chain_id INTEGER NOT NULL,
            token_id INTEGER,
            balance TEXT NOT NULL,
            nonce INTEGER,
            query_status TEXT NOT NULL CHECK(query_status IN ('success', 'failed', 'timeout')),
            error_message TEXT,
            rpc_url TEXT NOT NULL,
            response_time_ms INTEGER,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (chain_id) REFERENCES chains(id) ON DELETE CASCADE,
            FOREIGN KEY (token_id) REFERENCES tokens(id) ON DELETE CASCADE
        );
        "#
    ).execute(pool).await?;
    
    // 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_balance_history_address ON balance_history(address);")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_balance_history_chain_id ON balance_history(chain_id);")
        .execute(pool).await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_balance_history_created_at ON balance_history(created_at);")
        .execute(pool).await?;
    
    Ok(())
}

/// 为tokens表添加abi列（数据库迁移）
async fn add_abi_column_to_tokens_table(pool: &SqlitePool) -> Result<()> {
    // 检查abi列是否已经存在
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('tokens') WHERE name = 'abi'"
    )
    .fetch_one(pool)
    .await?;
    
    let column_exists = count > 0;
    
    if !column_exists {
        // 添加abi列，用于存储智能合约的ABI JSON
        sqlx::query("ALTER TABLE tokens ADD COLUMN abi TEXT")
            .execute(pool)
            .await?;
    }
    
    Ok(())
}

/// 插入默认数据
async fn insert_default_data(pool: &SqlitePool) -> Result<()> {
    // 检查是否已经有数据
    let chain_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chains")
        .fetch_one(pool).await?;
    
    if chain_count == 0 {
        // 插入默认链配置
        let chains = vec![
            ("eth", "Ethereum", 1, "ETH", "Ethereum", 18),
            ("bsc", "BNB Smart Chain", 56, "BNB", "BNB", 18),
            ("avax", "Avalanche", 43114, "AVAX", "Avalanche", 18),
            ("zgs", "0G Network", 16600, "0G", "0G Network", 18),
            ("bevm", "BEVM", 11501, "BTC", "Bitcoin", 8),
            ("sahara", "Sahara AI", 0, "SAHARA", "Sahara", 18),
            ("story", "Story Protocol", 0, "IP", "Story", 18),
            ("manta", "Manta Pacific", 169, "ETH", "Ethereum", 18),
            ("linea", "Linea", 59144, "ETH", "Ethereum", 18),
            ("base", "Base", 8453, "ETH", "Ethereum", 18),
            ("sol", "Solana", 0, "SOL", "Solana", 9),
            ("opbnb", "opBNB", 204, "BNB", "BNB", 18),
            ("geth", "Goerli Testnet", 5, "ETH", "Ethereum", 18),
            ("sepolia", "Sepolia Testnet", 11155111, "ETH", "Ethereum", 18),
        ];
        
        for (key, name, chain_id, symbol, currency_name, decimals) in chains {
            // 获取每个链的额外配置信息
            let (pic_url, scan_url, scan_api, verify_api, check_verify_api) = get_chain_config(key);
            
            let chain_db_id: i64 = sqlx::query_scalar(
                "INSERT INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id"
            )
            .bind(key)
            .bind(name)
            .bind(chain_id)
            .bind(symbol)
            .bind(currency_name)
            .bind(decimals)
            .bind(pic_url)
            .bind(scan_url)
            .bind(scan_api)
            .bind(verify_api)
            .bind(check_verify_api)
            .fetch_one(pool)
            .await?;
            
            // 插入对应的RPC配置
            insert_default_rpc_providers(pool, key, chain_db_id).await?;
            
            // 插入基础代币配置
            sqlx::query(
                "INSERT INTO tokens (chain_id, token_key, token_name, symbol, decimals, token_type, abi) VALUES (?, ?, ?, ?, ?, 'base', NULL)"
            )
            .bind(chain_db_id)
            .bind(format!("{}_base", key))
            .bind(currency_name)
            .bind(symbol)
            .bind(decimals)
            .execute(pool)
            .await?;
        }
    }
    
    Ok(())
}

/// 插入默认RPC提供商
async fn insert_default_rpc_providers(pool: &SqlitePool, chain_key: &str, chain_id: i64) -> Result<()> {
    let rpc_urls = match chain_key {
        "eth" => vec![
            "https://rpc.ankr.com/eth/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f",
            "https://eth-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5",
            "https://1rpc.io/eth",
        ],
        "bsc" => vec![
            "https://bsc-dataseed1.bnbchain.org",
            "https://bsc.publicnode.com",
            "https://bsc.drpc.org",
        ],
        "avax" => vec![
            "https://ava-mainnet.public.blastapi.io/ext/bc/C/rpc",
            "https://api.avax.network/ext/bc/C/rpc",
            "https://avalanche.drpc.org",
            "https://avalanche-c-chain-rpc.publicnode.com",
            "https://avax-pokt.nodies.app/ext/bc/C/rpc",
            "https://1rpc.io/avax/c",
        ],
        "zgs" => vec![
            "https://0g-evm-rpc.tws.im/",
            "https://rpc-testnet.0g.ai",
        ],
        "bevm" => vec![
            "https://rpc-mainnet-2.bevm.io",
            "https://rpc-mainnet-1.bevm.io",
        ],
        "sahara" => vec![
            "https://testnet.saharalabs.ai",
        ],
        "story" => vec![
            "https://rpc.ankr.com/story_odyssey/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f",
        ],
        "manta" => vec![
            "https://1rpc.io/manta",
            "https://manta.nirvanalabs.xyz/mantapublic",
            "https://pacific-rpc.manta.network/http",
            "https://manta-pacific.drpc.org",
        ],
        "linea" => vec![
            "https://linea.decubate.com",
            "https://1rpc.io/linea",
            "https://rpc.linea.build",
            "https://linea.drpc.org",
        ],
        "base" => vec![
            "https://base.publicnode.com",
            "https://base.meowrpc.com",
            "https://1rpc.io/base",
            "https://base.llamarpc.com",
            "https://base.blockpi.network/v1/rpc/public",
        ],
        "sol" => vec![
            "https://rpc.ankr.com/solana/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f",
        ],
        "opbnb" => vec![
            "https://opbnb-mainnet-rpc.bnbchain.org",
            "https://opbnb-mainnet.nodereal.io/v1/ea08c11bd0874ce19cee7fc6f63b6cf8",
            "https://opbnb-mainnet.nodereal.io/v1/8a1c5fbe106c422ea9c9093570ce0af2",
        ],
        "geth" => vec![
            "https://eth-goerli.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5",
            "https://eth-goerli.nodereal.io/v1/ea08c11bd0874ce19cee7fc6f63b6cf8",
        ],
        "sepolia" => vec![
            "https://lb.drpc.org/ogrpc?network=sepolia",
        ],
        _ => vec![],
    };
    
    for (priority, rpc_url) in rpc_urls.iter().enumerate() {
        sqlx::query(
            "INSERT INTO rpc_providers (chain_id, rpc_url, priority) VALUES (?, ?, ?)"
        )
        .bind(chain_id)
        .bind(rpc_url)
        .bind(priority as i32)
        .execute(pool)
        .await?;
    }
    
    Ok(())
}

/// 获取链的额外配置信息
fn get_chain_config(chain_key: &str) -> (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) {
    match chain_key {
        "eth" => (
            Some("https://assets.coingecko.com/coins/images/279/small/ethereum.png".to_string()),
            Some("https://etherscan.io".to_string()),
            Some("https://api.etherscan.io/api".to_string()),
            Some("https://api.etherscan.io/api".to_string()),
            Some("https://api.etherscan.io/api".to_string()),
        ),
        "bsc" => (
            Some("https://assets.coingecko.com/coins/images/825/small/bnb-icon2_2x.png".to_string()),
            Some("https://bscscan.com".to_string()),
            Some("https://api.bscscan.com/api".to_string()),
            Some("https://api.bscscan.com/api".to_string()),
            Some("https://api.bscscan.com/api".to_string()),
        ),
        "avax" => (
            Some("https://assets.coingecko.com/coins/images/12559/small/Avalanche_Circle_RedWhite_Trans.png".to_string()),
            Some("https://snowscan.xyz".to_string()),
            Some("https://api.routescan.io/v2/network/mainnet/evm/43114/etherscan/api".to_string()),
            Some("https://api.routescan.io/v2/network/mainnet/evm/43114/etherscan/api".to_string()),
            Some("https://api.routescan.io/v2/network/mainnet/evm/43114/etherscan/api".to_string()),
        ),
        "base" => (
            Some("https://assets.coingecko.com/coins/images/279/small/ethereum.png".to_string()),
            Some("https://basescan.org".to_string()),
            Some("https://api.basescan.org/api".to_string()),
            Some("https://api.basescan.org/api".to_string()),
            Some("https://api.basescan.org/api".to_string()),
        ),
        "linea" => (
            Some("https://assets.coingecko.com/coins/images/279/small/ethereum.png".to_string()),
            Some("https://lineascan.build".to_string()),
            Some("https://api.lineascan.build/api".to_string()),
            Some("https://api.lineascan.build/api".to_string()),
            Some("https://api.lineascan.build/api".to_string()),
        ),
        "manta" => (
            Some("https://assets.coingecko.com/coins/images/279/small/ethereum.png".to_string()),
            Some("https://pacific-explorer.manta.network".to_string()),
            Some("https://pacific-explorer.manta.network/api".to_string()),
            Some("https://pacific-explorer.manta.network/api".to_string()),
            Some("https://pacific-explorer.manta.network/api".to_string()),
        ),
        "opbnb" => (
            Some("https://assets.coingecko.com/coins/images/825/small/bnb-icon2_2x.png".to_string()),
            Some("https://opbnbscan.com".to_string()),
            Some("https://api-opbnb.bscscan.com/api".to_string()),
            Some("https://api-opbnb.bscscan.com/api".to_string()),
            Some("https://api-opbnb.bscscan.com/api".to_string()),
        ),
        "sol" => (
            Some("https://assets.coingecko.com/coins/images/4128/small/solana.png".to_string()),
            Some("https://solscan.io".to_string()),
            Some("https://api.solscan.io".to_string()),
            Some("https://api.solscan.io".to_string()),
            Some("https://api.solscan.io".to_string()),
        ),
        "geth" => (
            Some("https://assets.coingecko.com/coins/images/279/small/ethereum.png".to_string()),
            Some("https://goerli.etherscan.io".to_string()),
            Some("https://api-goerli.etherscan.io/api".to_string()),
            Some("https://api-goerli.etherscan.io/api".to_string()),
            Some("https://api-goerli.etherscan.io/api".to_string()),
        ),
        "sepolia" => (
            Some("https://assets.coingecko.com/coins/images/279/small/ethereum.png".to_string()),
            Some("https://sepolia.etherscan.io".to_string()),
            Some("https://api-sepolia.etherscan.io/api".to_string()),
            Some("https://api-sepolia.etherscan.io/api".to_string()),
            Some("https://api-sepolia.etherscan.io/api".to_string()),
        ),
        "zgs" => (
            Some("https://0g.ai/favicon.ico".to_string()),
            Some("https://chainscan.0g.ai".to_string()),
            Some("https://chainscan.0g.ai/api".to_string()),
            Some("https://chainscan.0g.ai/api".to_string()),
            Some("https://chainscan.0g.ai/api".to_string()),
        ),
        "bevm" => (
            Some("https://assets.coingecko.com/coins/images/1/small/bitcoin.png".to_string()),
            Some("https://scan-mainnet.bevm.io".to_string()),
            Some("https://scan-mainnet.bevm.io/api".to_string()),
            Some("https://scan-mainnet.bevm.io/api".to_string()),
            Some("https://scan-mainnet.bevm.io/api".to_string()),
        ),
        "sahara" => (
            Some("https://www.saharalabs.ai/favicon.ico".to_string()),
            Some("https://explorer.saharalabs.ai".to_string()),
            Some("https://explorer.saharalabs.ai/api".to_string()),
            Some("https://explorer.saharalabs.ai/api".to_string()),
            Some("https://explorer.saharalabs.ai/api".to_string()),
        ),
        "story" => (
            Some("https://story.foundation/favicon.ico".to_string()),
            Some("https://testnet.storyscan.xyz".to_string()),
            Some("https://testnet.storyscan.xyz/api".to_string()),
            Some("https://testnet.storyscan.xyz/api".to_string()),
            Some("https://testnet.storyscan.xyz/api".to_string()),
        ),
        _ => (None, None, None, None, None),
    }
}
