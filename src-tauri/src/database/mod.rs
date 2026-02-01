pub mod models;
pub mod chain_service;
pub mod rpc_service;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use anyhow::Result;
use std::str::FromStr;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use std::path::Path;
use sha2::Sha256;
use hmac::Hmac;
use pbkdf2::pbkdf2;

/// 智能解析SQL语句，正确处理多行语句
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
        
        // 处理字符串字面量
        if !in_string && (ch == '\'' || ch == '"') {
            in_string = true;
            string_delimiter = ch;
            current_statement.push(ch);
        } else if in_string && ch == string_delimiter {
            // 检查是否是转义的引号
            if i + 1 < chars.len() && chars[i + 1] == string_delimiter {
                current_statement.push(ch);
                current_statement.push(chars[i + 1]);
                i += 1;
            } else {
                in_string = false;
                current_statement.push(ch);
            }
        } else if !in_string {
            // 处理注释
            if ch == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
                // 跳过单行注释
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                if i < chars.len() {
                    current_statement.push('\n');
                }
                i += 1;
                continue;
            }
            
            // 处理括号深度
            if ch == '(' {
                paren_depth += 1;
            } else if ch == ')' {
                paren_depth -= 1;
            }
            
            // 处理分号
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
    
    // 添加最后一个语句（如果有的话）
    let trimmed = current_statement.trim();
    if !trimmed.is_empty() && !trimmed.starts_with("--") {
        statements.push(trimmed.to_string());
    }
    
    statements
}

/// Package.json 配置结构
#[derive(Debug, Deserialize, Serialize)]
struct PackageJson {
    config: Option<Config>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    database: Option<DatabaseConfig>,
}

/// 数据库配置
#[derive(Debug, Deserialize, Serialize)]
struct DatabaseConfig {
    #[serde(rename = "forceInit")]
    force_init: Option<bool>,
    #[serde(rename = "enableDebugLog")]
    enable_debug_log: Option<bool>,
    #[serde(rename = "initSqlPath")]
    init_sql_path: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            force_init: Some(false),
            enable_debug_log: Some(true),
            init_sql_path: Some("data/init.sql".to_string()),
        }
    }
}

/// 读取package.json中的数据库配置
fn load_database_config() -> DatabaseConfig {
    let package_json_paths = vec!["package.json", "../package.json"];
    
    for path in package_json_paths {
        if Path::new(path).exists() {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    match serde_json::from_str::<PackageJson>(&content) {
                        Ok(package_json) => {
                            if let Some(config) = package_json.config {
                                if let Some(db_config) = config.database {
                                    return DatabaseConfig {
                                        force_init: db_config.force_init,
                                        enable_debug_log: db_config.enable_debug_log,
                                        init_sql_path: db_config.init_sql_path,
                                    };
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("解析 {path} 失败: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("读取 {path} 失败: {e}");
                }
            }
        }
    }
    
    // 如果找不到配置文件或配置项，返回默认配置
    DatabaseConfig::default()
}

/// 数据库管理器
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    /// 创建新的数据库管理器实例
    pub async fn new(database_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);
        
        let pool = SqlitePool::connect_with(options).await?;
        
        let manager = Self { pool };
        
        Ok(manager)
    }
    
    /// 创建加密的数据库管理器实例
    pub async fn new_encrypted(database_url: &str, db_password: &str) -> Result<Self> {
        let db_password_owned = db_password.to_string();
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .pragma("key", db_password_owned);  // SQLCipher 加密密钥
        
        let pool = SqlitePool::connect_with(options).await?;
        
        // 验证加密是否生效 - 执行一个简单查询
        match sqlx::query("SELECT 1").fetch_one(&pool).await {
            Ok(_) => {
                println!("数据库加密验证成功");
            }
            Err(e) => {
                return Err(anyhow::anyhow!("数据库解密失败，密码可能错误: {e}"));
            }
        }
        
        let manager = Self { pool };
        
        Ok(manager)
    }
    
    /// 获取数据库连接池
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// 从用户密码派生数据库加密密钥
/// 使用独立的 salt 和更高的迭代次数，与内部 MDK 派生分离
#[allow(dead_code)]
pub fn derive_database_key(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    let _ = pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, 600_000, &mut key);
    key
}

/// 生成数据库密码字符串
/// SQLCipher 直接使用字符串密码，不需要 hex 格式
pub fn generate_db_password(password: &str, _salt: &[u8; 16]) -> String {
    // 直接使用用户密码作为数据库密码
    // 这样更简单且兼容 SQLCipher
    password.to_string()
}

/// 从密码派生 salt（确定性方式）
pub fn derive_salt_from_password(password: &str) -> [u8; 16] {
    use sha2::Digest;
    let hash = Sha256::digest(password.as_bytes());
    let mut salt = [0u8; 16];
    salt.copy_from_slice(&hash[0..16]);
    salt
}

/// 执行数据库迁移
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // 检查chains表是否包含ecosystem列
    let ecosystem_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('chains') WHERE name = 'ecosystem'"
    )
    .fetch_one(pool)
    .await?;

    if ecosystem_exists == 0 {
        println!("正在迁移数据库: 添加 ecosystem 列到 chains 表");
        sqlx::query("ALTER TABLE chains ADD COLUMN ecosystem TEXT NOT NULL DEFAULT 'evm'")
            .execute(pool)
            .await?;
        println!("迁移完成: ecosystem 列已添加");
    }

    // 数据修复：将 Solana 链的 ecosystem 设置为 solana
    sqlx::query("UPDATE chains SET ecosystem = 'solana' WHERE chain_key IN ('sol', 'solana') AND ecosystem != 'solana'")
        .execute(pool)
        .await
        .ok();

    // 数据修复：确保没有空值
    sqlx::query("UPDATE chains SET ecosystem = 'evm' WHERE (ecosystem IS NULL OR ecosystem = '')")
        .execute(pool)
        .await
        .ok();

    Ok(())
}

/// 全局数据库管理器实例
static DATABASE_MANAGER: OnceLock<DatabaseManager> = OnceLock::new();

/// 数据库加密密码存储
static DB_PASSWORD: OnceLock<String> = OnceLock::new();

/// 检查数据库文件是否存在
#[allow(dead_code)]
pub fn database_exists() -> bool {
    Path::new("data/wallets_tool.db").exists()
}

/// 检查数据库是否已加密（通过检查是否能无密码访问）
pub async fn is_database_encrypted() -> Result<bool> {
    let database_path = "data/wallets_tool.db";
    if !Path::new(database_path).exists() {
        return Ok(false); // 数据库不存在，视为未加密（新数据库）
    }
    
    // 尝试无密码打开数据库
    let database_url = format!("sqlite://{database_path}");
    let options = SqliteConnectOptions::from_str(&database_url)?;
    
    match SqlitePool::connect_with(options).await {
        Ok(pool) => {
            // 能打开，尝试执行查询
            match sqlx::query("SELECT 1").fetch_one(&pool).await {
                Ok(_) => Ok(false), // 无密码能访问，说明未加密
                Err(_) => Ok(true), // 需要密码才能访问
            }
        }
        Err(_) => Ok(true), // 无法打开，可能是加密的
    }
}

/// 初始化数据库（普通版本，用于应用启动时的基础数据库）
pub async fn init_database() -> Result<()> {
    let config = load_database_config();
    let enable_debug = config.enable_debug_log.unwrap_or(false);
    
    if enable_debug {
        println!("使用的数据库配置: {config:?}");
    }

    // 确保配置目录存在
    std::fs::create_dir_all("data")?;

    let database_path = "data/wallets_tool.db";
    let database_url = format!("sqlite://{database_path}");
    let db_exists = Path::new(&database_path).exists();
    
    // 检查是否已加密
    let is_encrypted = if db_exists {
        is_database_encrypted().await.unwrap_or(false)
    } else {
        false
    };
    
    if is_encrypted {
        println!("数据库已加密，等待密码解锁...");
        // 加密的数据库不在这里初始化，等待用户输入密码后解锁
        return Ok(());
    }
    
    // 创建普通数据库管理器
    let manager = DatabaseManager::new(&database_url).await?;
    
    // 执行自动迁移
    if let Err(e) = run_migrations(manager.get_pool()).await {
        eprintln!("数据库迁移失败: {e}");
    }

    DATABASE_MANAGER.set(manager)
        .map_err(|_| anyhow::anyhow!("Database already initialized"))?;

    // 如果需要初始化数据库
    if config.force_init.unwrap_or(false) || !db_exists {
        if enable_debug {
            println!("开始初始化数据库...");
            println!("数据库路径: {database_path}");
        }

        let init_sql_path = config.init_sql_path.unwrap_or_else(|| "data/init.sql".to_string());
        
        // 检查 init.sql 是否存在
        if !Path::new(&init_sql_path).exists() {
            if enable_debug {
                println!("init.sql 不存在，跳过初始化");
            }
            return Ok(());
        }
        
        let init_sql = std::fs::read_to_string(&init_sql_path)?;

        if enable_debug {
            println!("已加载 SQL 文件，大小: {} 字节", init_sql.len());
        }

        let pool = DATABASE_MANAGER.get().expect("Database manager not initialized").get_pool();

        // 直接执行整个SQL文件
        match sqlx::query(&init_sql).execute(pool).await {
            Ok(_) => {
                if enable_debug {
                    println!("数据库初始化SQL执行成功");
                }
            }
            Err(e) => {
                if enable_debug {
                    println!("整体执行失败，尝试逐条执行: {e}");
                }
                
                let mut executed_count = 0;
                let statements: Vec<&str> = init_sql
                    .split(';')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty() && !s.starts_with("--"))
                    .collect();
                
                for statement in statements {
                    if !statement.is_empty() {
                        match sqlx::query(statement).execute(pool).await {
                            Ok(_) => {
                                executed_count += 1;
                                if enable_debug {
                                    println!("执行 SQL 语句成功: {executed_count} 个");
                                }
                            }
                            Err(e) => {
                                if !e.to_string().contains("UNIQUE constraint failed") && !e.to_string().contains("already exists") {
                                    eprintln!("执行 SQL 语句错误: {e}");
                                    eprintln!("错误语句: {statement}");
                                    return Err(e.into());
                                } else if enable_debug {
                                    println!("忽略重复创建/插入错误");
                                }
                            }
                        }
                    }
                }
                if enable_debug {
                    println!("逐条执行完成，共执行 {executed_count} 个 SQL 语句");
                }
            }
        }

        println!("数据库初始化完成");
    }
    
    Ok(())
}

/// 使用密码初始化加密数据库（首次设置密码时调用）
pub async fn init_encrypted_database(password: &str) -> Result<()> {
    let config = load_database_config();
    let enable_debug = config.enable_debug_log.unwrap_or(false);
    
    if enable_debug {
        println!("初始化加密数据库...");
    }

    // 确保配置目录存在
    std::fs::create_dir_all("data")?;

    let database_path = "data/wallets_tool.db";
    let database_url = format!("sqlite://{database_path}");
    let db_exists = Path::new(&database_path).exists();
    
    // 从密码派生 salt
    let salt = derive_salt_from_password(password);
    let db_password = generate_db_password(password, &salt);
    
    // 存储数据库密码
    let _ = DB_PASSWORD.set(db_password.clone());
    
    // 创建加密数据库管理器
    let manager = DatabaseManager::new_encrypted(&database_url, &db_password).await?;
    
    // 执行自动迁移
    if let Err(e) = run_migrations(manager.get_pool()).await {
        eprintln!("数据库迁移失败: {e}");
    }

    // 存储数据库管理器（仅在未初始化时）
    if DATABASE_MANAGER.get().is_none() {
        DATABASE_MANAGER.set(manager)
            .map_err(|_| anyhow::anyhow!("Database already initialized"))?;
    } else if enable_debug {
        println!("数据库管理器已初始化，跳过设置");
    }

    // 初始化表结构
    if !db_exists || config.force_init.unwrap_or(false) {
        if enable_debug {
            println!("开始初始化加密数据库表结构...");
        }

        let pool = DATABASE_MANAGER.get().expect("Database manager not initialized").get_pool();
        
        // 创建钱包管理器所需的表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                parent_id INTEGER,
                name TEXT NOT NULL,
                chain_type TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (parent_id) REFERENCES wallet_groups(id) ON DELETE CASCADE
            )
            "#
        ).execute(pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                group_id INTEGER,
                name TEXT,
                address TEXT NOT NULL,
                chain_type TEXT NOT NULL,
                encrypted_private_key TEXT,
                encrypted_mnemonic TEXT,
                mnemonic_index INTEGER,
                remark TEXT,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (group_id) REFERENCES wallet_groups(id) ON DELETE SET NULL
            )
            "#
        ).execute(pool).await?;

        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_wallets_chain_address ON wallets(chain_type, address)"
        ).execute(pool).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#
        ).execute(pool).await?;

        println!("加密数据库初始化完成");
    }
    
    Ok(())
}

/// 使用密码解锁加密数据库（后续启动时调用）
pub async fn unlock_encrypted_database(password: &str) -> Result<()> {
    let database_path = "data/wallets_tool.db";
    let database_url = format!("sqlite://{database_path}");
    
    // 从密码派生 salt 和数据库密码
    let salt = derive_salt_from_password(password);
    let db_password = generate_db_password(password, &salt);
    
    // 存储数据库密码
    let _ = DB_PASSWORD.set(db_password.clone());
    
    // 创建加密数据库管理器
    let manager = DatabaseManager::new_encrypted(&database_url, &db_password).await?;

    // 存储数据库管理器（仅在未初始化时）
    if DATABASE_MANAGER.get().is_none() {
        DATABASE_MANAGER.set(manager)
            .map_err(|_| anyhow::anyhow!("Database already initialized"))?;
    }

    Ok(())
}

/// 获取全局数据库管理器
pub fn get_database_manager() -> &'static DatabaseManager {
    DATABASE_MANAGER.get().expect("Database not initialized")
}

/// 获取数据库密码
#[allow(dead_code)]
pub fn get_db_password() -> Option<&'static String> {
    DB_PASSWORD.get()
}

/// 重新加载数据库（删除所有表并从init.sql重新导入）
#[tauri::command]
pub async fn reload_database() -> Result<String, String> {
    let config = load_database_config();
    let enable_debug = config.enable_debug_log.unwrap_or(false);
    let pool = get_database_manager().get_pool();
    
    if enable_debug {
        println!("开始重新加载数据库...");
    }
    
    // 禁用外键约束检查
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(pool)
        .await
        .map_err(|e| format!("禁用外键约束失败: {e}"))?;
    
    // 获取所有表名
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("获取表名失败: {e}"))?;
    
    // 删除所有表（包括表结构）
    for table in &tables {
        let drop_sql = format!("DROP TABLE IF EXISTS {table}");
        sqlx::query(&drop_sql)
            .execute(pool)
            .await
            .map_err(|e| format!("删除表 {table} 失败: {e}"))?;
        
        if enable_debug {
            println!("已删除表: {table}");
        }
    }
    
    // 清理sqlite_sequence表
    sqlx::query("DELETE FROM sqlite_sequence")
        .execute(pool)
        .await
        .ok();
    
    if enable_debug {
        println!("已删除所有表，开始重新创建...");
    }
    
    // 从init.sql重新导入数据
    let init_sql_path = config.init_sql_path.unwrap_or_else(|| "data/init.sql".to_string());
    let init_sql = std::fs::read_to_string(&init_sql_path)
        .map_err(|e| format!("读取init.sql文件失败: {e}"))?;
    
    if enable_debug {
        println!("已加载 SQL 文件，大小: {} 字节", init_sql.len());
    }
    
    // 执行SQL语句
    let mut executed_count = 0;
    
    match sqlx::query(&init_sql).execute(pool).await {
        Ok(_) => {
            executed_count = 1;
            if enable_debug {
                println!("整体执行SQL成功");
            }
        }
        Err(_) => {
            let statements = parse_sql_statements(&init_sql);
            
            for statement in statements {
                if !statement.trim().is_empty() && !statement.trim().starts_with("--") {
                    match sqlx::query(&statement).execute(pool).await {
                        Ok(_) => {
                            executed_count += 1;
                            if enable_debug {
                                println!("执行 SQL 语句成功: {executed_count} 个");
                            }
                        }
                        Err(e) => {
                            eprintln!("执行 SQL 语句错误: {e}");
                            eprintln!("错误语句: {statement}");
                            sqlx::query("PRAGMA foreign_keys = ON")
                                .execute(pool)
                                .await
                                .ok();
                            return Err(format!("执行SQL语句失败: {e}"));
                        }
                    }
                }
            }
        }
    }
    
    // 重新启用外键约束检查
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .map_err(|e| format!("启用外键约束失败: {e}"))?;
    
    if enable_debug {
        println!("数据库重新加载完成，共执行 {executed_count} 个 SQL 语句");
    }
    
    Ok(format!("数据库重新加载成功，共执行 {executed_count} 个 SQL 语句"))
}

/// 检查数据库结构是否需要更新
#[tauri::command]
pub async fn check_database_schema() -> Result<serde_json::Value, String> {
    let pool = get_database_manager().get_pool();
    
    let chains_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chains'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;
    
    let tokens_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tokens'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;
    
    let rpc_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='rpc_providers'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;
    
    let contract_type_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('tokens') WHERE name = 'contract_type'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;
    
    let abi_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('tokens') WHERE name = 'abi'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;

    let ecosystem_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('chains') WHERE name = 'ecosystem'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;
    
    let schema_info = serde_json::json!({
        "db_exists": true,
        "chains_table_exists": chains_table_exists > 0,
        "tokens_table_exists": tokens_table_exists > 0,
        "rpc_table_exists": rpc_table_exists > 0,
        "contract_type_column_exists": contract_type_exists > 0,
        "abi_column_exists": abi_exists > 0,
        "ecosystem_column_exists": ecosystem_exists > 0,
        "needs_migration": contract_type_exists == 0 || abi_exists == 0 || ecosystem_exists == 0
    });
    
    Ok(schema_info)
}

/// 导出数据库数据到init.sql文件
#[tauri::command]
pub async fn export_database_to_init_sql() -> Result<String, String> {
    let config = load_database_config();
    let enable_debug = config.enable_debug_log.unwrap_or(false);
    let pool = get_database_manager().get_pool();
    
    if enable_debug {
        println!("开始导出数据库数据到init.sql...");
    }
    
    let mut sql_content = String::new();
    
    sql_content.push_str("-- Wallet Manager 数据库初始化脚本\n");
    sql_content.push_str("-- 此文件由系统自动生成，包含当前数据库的所有数据\n\n");
    
    let all_tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("获取表名失败: {e}"))?;
    
    let mut tables = Vec::new();
    let table_order = vec!["chains", "rpc_providers", "tokens"];
    
    for table_name in &table_order {
        if all_tables.contains(&table_name.to_string()) {
            tables.push(table_name.to_string());
        }
    }
    
    for table in &all_tables {
        if !tables.contains(table) {
            tables.push(table.clone());
        }
    }
    
    if enable_debug {
        println!("表创建顺序: {tables:?}");
    }
    
    for table in &tables {
        if enable_debug {
            println!("正在导出表: {table}");
        }
        
        let create_sql: String = sqlx::query_scalar(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name = ?"
        )
        .bind(table)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("获取表 {table} 结构失败: {e}"))?;
        
        sql_content.push_str(&format!("-- 创建{table}表\n"));
        sql_content.push_str(&create_sql);
        sql_content.push_str(";\n\n");
        
        let rows = sqlx::query(&format!("SELECT * FROM {table}"))
            .fetch_all(pool)
            .await
            .map_err(|e| format!("获取表 {table} 数据失败: {e}"))?;
        
        if !rows.is_empty() {
            let columns: Vec<String> = sqlx::query_scalar(
                "SELECT name FROM pragma_table_info(?) ORDER BY cid"
            )
            .bind(table)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("获取表 {table} 列名失败: {e}"))?;
            
            sql_content.push_str(&format!("-- 插入{table}表数据\n"));
            
            for row in rows {
                let mut values = Vec::new();
                for (i, _column) in columns.iter().enumerate() {
                    let value = if let Ok(v) = row.try_get::<Option<String>, _>(i) {
                        v
                    } else if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
                        v.map(|x| x.to_string())
                    } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
                        v.map(|x| x.to_string())
                    } else if let Ok(v) = row.try_get::<Option<bool>, _>(i) {
                        v.map(|x| if x { "1" } else { "0" }.to_string())
                    } else {
                        None
                    };
                    
                    match value {
                        Some(v) => {
                            let escaped = v.replace("'", "''");
                            values.push(format!("'{escaped}'"));
                        }
                        None => values.push("NULL".to_string()),
                    }
                }
                
                let insert_sql = format!(
                    "INSERT OR IGNORE INTO {} ({}) VALUES ({});\n",
                    table,
                    columns.join(", "),
                    values.join(", ")
                );
                sql_content.push_str(&insert_sql);
            }
            sql_content.push('\n');
        }
    }
    
    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE type='index' AND sql IS NOT NULL AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("获取索引失败: {e}"))?;
    
    if !indexes.is_empty() {
        sql_content.push_str("-- 创建索引\n");
        for index_sql in indexes {
            sql_content.push_str(&index_sql);
            sql_content.push_str(";\n");
        }
    }
    
    let init_sql_path = config.init_sql_path.unwrap_or_else(|| "data/init.sql".to_string());
    std::fs::write(&init_sql_path, &sql_content)
        .map_err(|e| format!("写入init.sql文件失败: {e}"))?;
    
    if enable_debug {
        println!("数据库导出完成，文件大小: {} 字节", sql_content.len());
    }
    
    Ok(format!("数据库数据已成功导出到 {}，共 {} 个表，文件大小: {} 字节", 
        init_sql_path, tables.len(), sql_content.len()))
}
