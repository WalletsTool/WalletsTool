use anyhow::Result;
use sqlx::SqlitePool;
use std::path::Path;

use super::dual_database::{DualDatabaseManager, PUBLIC_DB_PATH};
use super::encryption::create_unencrypted_pool;

static PUBLIC_INIT_SQL: &str = include_str!("../../data/public_init.sql");

pub async fn init_public_database() -> Result<()> {
    std::fs::create_dir_all("data")?;
    
    let db_exists = Path::new(PUBLIC_DB_PATH).exists();
    
    let pool = create_unencrypted_pool(PUBLIC_DB_PATH).await?;
    
    if !db_exists {
        execute_init_sql(&pool, PUBLIC_INIT_SQL).await?;
        load_default_chain_data(&pool).await?;
    }
    
    DualDatabaseManager::init_public_pool(pool);
    DualDatabaseManager::init_secure_pool_placeholder();
    
    Ok(())
}

async fn execute_init_sql(pool: &SqlitePool, sql: &str) -> Result<()> {
    for statement in parse_sql_statements(sql) {
        let stmt_trimmed = statement.trim();
        if !stmt_trimmed.is_empty() && !stmt_trimmed.starts_with("--") {
            if let Err(e) = sqlx::query(&statement).execute(pool).await {
                let error_msg = e.to_string();
                if !error_msg.contains("already exists") {
                    eprintln!("SQL warning: {}", error_msg);
                }
            }
        }
    }
    Ok(())
}

fn parse_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_string = false;
    let mut string_delimiter = '\0';
    let mut paren_depth = 0;
    
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        if !in_string && (ch == '\'' || ch == '"') {
            in_string = true;
            string_delimiter = ch;
            current_statement.push(ch);
        } else if in_string && ch == string_delimiter {
            if i + 1 < chars.len() && chars[i + 1] == string_delimiter {
                current_statement.push(ch);
                current_statement.push(chars[i + 1]);
                i += 1;
            } else {
                in_string = false;
                current_statement.push(ch);
            }
        } else if !in_string {
            if ch == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                if i < chars.len() {
                    current_statement.push('\n');
                }
                i += 1;
                continue;
            }
            
            if ch == '(' {
                paren_depth += 1;
            } else if ch == ')' {
                paren_depth -= 1;
            }
            
            if ch == ';' && paren_depth == 0 {
                let trimmed = current_statement.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    statements.push(current_statement.trim().to_string());
                }
                current_statement.clear();
            } else {
                current_statement.push(ch);
            }
        } else {
            current_statement.push(ch);
        }
        
        i += 1;
    }
    
    let trimmed = current_statement.trim();
    if !trimmed.is_empty() && !trimmed.starts_with("--") {
        statements.push(trimmed.to_string());
    }
    
    statements
}

async fn load_default_chain_data(pool: &SqlitePool) -> Result<()> {
    let chains_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM chains")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    
    if chains_count > 0 {
        return Ok(());
    }
    
    let default_chains = vec![
        ("eth", "Ethereum", 1, "ETH", "ETH", 18, "evm"),
        ("bsc", "BSC", 56, "BNB", "BNB", 18, "evm"),
        ("base", "BASE", 8453, "ETH", "ETH", 18, "evm"),
        ("arb", "Arbitrum One", 42161, "ETH", "ETH", 18, "evm"),
        ("optimism", "Optimism", 10, "ETH", "ETH", 18, "evm"),
        ("polygon", "Polygon", 137, "POL", "POL", 18, "evm"),
        ("linea", "Linea", 59144, "ETH", "ETH", 18, "evm"),
        ("sol", "Solana", 101, "SOL", "Solana", 9, "solana"),
        ("soldev", "Solana-dev", 103, "SOL", "Solana", 9, "solana"),
    ];
    
    for (key, name, chain_id, symbol, currency_name, decimals, ecosystem) in default_chains {
        sqlx::query(
            "INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, ecosystem) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(key)
        .bind(name)
        .bind(chain_id)
        .bind(symbol)
        .bind(currency_name)
        .bind(decimals)
        .bind(ecosystem)
        .execute(pool)
        .await?;
    }
    
    let default_rpcs = vec![
        ("eth", "https://eth.llamarpc.com"),
        ("eth", "https://rpc.ankr.com/eth"),
        ("bsc", "https://bsc-dataseed.binance.org"),
        ("bsc", "https://bsc-dataseed1.defibit.io"),
        ("base", "https://mainnet.base.org"),
        ("base", "https://1rpc.io/base"),
        ("arb", "https://arb1.arbitrum.io/rpc"),
        ("arb", "https://1rpc.io/arb"),
        ("optimism", "https://mainnet.optimism.io"),
        ("polygon", "https://polygon-rpc.com"),
        ("linea", "https://rpc.linea.build"),
        ("sol", "https://api.mainnet-beta.solana.com"),
        ("soldev", "https://api.devnet.solana.com"),
    ];
    
    for (chain_key, rpc_url) in default_rpcs {
        let chain_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM chains WHERE chain_key = ?"
        )
        .bind(chain_key)
        .fetch_optional(pool)
        .await?;
        
        if let Some(id) = chain_id {
            sqlx::query(
                "INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url) VALUES (?, ?)"
            )
            .bind(id)
            .bind(rpc_url)
            .execute(pool)
            .await?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sql_statements() {
        let sql = "CREATE TABLE a (id INT); INSERT INTO a VALUES (1);";
        let stmts = parse_sql_statements(sql);
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn test_parse_sql_with_comments() {
        let sql = "-- comment\nCREATE TABLE a (id INT);";
        let stmts = parse_sql_statements(sql);
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].contains("CREATE TABLE"));
    }
}
