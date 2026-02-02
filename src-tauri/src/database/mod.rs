pub mod models;
pub mod chain_service;
pub mod rpc_service;
pub mod migrations;

/// 编译时嵌入 init.sql 内容，确保打包后也能正常恢复出厂设置
static INIT_SQL_CONTENT: &str = include_str!("../../data/init.sql");

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use anyhow::Result;
use std::env;
use std::fs;
use std::str::FromStr;
use std::sync::{OnceLock, RwLock};
use serde::{Deserialize, Serialize};
use std::path::Path;
use sha2::Sha256;
use hmac::Hmac;
use pbkdf2::pbkdf2;

async fn apply_versioned_migrations(pool: &SqlitePool) -> Result<Vec<migrations::MigrationApplied>> {
    let chains_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chains'",
    )
    .fetch_one(pool)
    .await?;

    if chains_table_exists == 0 {
        return Ok(Vec::new());
    }

    migrations::ensure_migrations_table(pool).await?;

    let app_version = env!("CARGO_PKG_VERSION");
    let mut applied = Vec::new();

    for m in migrations::all_migrations() {
        if migrations::is_migration_recorded(pool, m.version).await? {
            continue;
        }

        let checksum = migrations::checksum_sql(m.sql);

        let should_apply = if let Some(check_sql) = m.check_sql {
            let count: i64 = sqlx::query_scalar(check_sql).fetch_one(pool).await?;
            count == 0
        } else {
            true
        };

        let mut tx = pool.begin().await?;

        if should_apply {
            for statement in parse_sql_statements(m.sql) {
                let stmt_trimmed = statement.trim();
                if stmt_trimmed.is_empty() {
                    continue;
                }
                sqlx::query(&statement).execute(&mut *tx).await?;
            }
        }

        sqlx::query(
            "INSERT INTO schema_migrations(version, name, checksum, app_version, applied) VALUES(?, ?, ?, ?, ?)",
        )
        .bind(m.version)
        .bind(m.name)
        .bind(&checksum)
        .bind(app_version)
        .bind(if should_apply { 1 } else { 0 })
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        applied.push(migrations::MigrationApplied {
            version: m.version,
            name: m.name.to_string(),
            checksum,
            applied: should_apply,
        });
    }

    Ok(applied)
}

async fn apply_migrations_with_backup(
    pool: &SqlitePool,
    database_path: &str,
) -> Result<Vec<migrations::MigrationApplied>> {
    let chains_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chains'",
    )
    .fetch_one(pool)
    .await?;

    if chains_table_exists == 0 {
        return Ok(Vec::new());
    }

    migrations::ensure_migrations_table(pool).await?;

    let mut needs_backup = false;
    for m in migrations::all_migrations() {
        if migrations::is_migration_recorded(pool, m.version).await? {
            continue;
        }

        let should_apply = if let Some(check_sql) = m.check_sql {
            let count: i64 = sqlx::query_scalar(check_sql).fetch_one(pool).await?;
            count == 0
        } else {
            true
        };

        if should_apply {
            needs_backup = true;
            break;
        }
    }

    let backup_path = if needs_backup {
        Some(backup_database_file(pool, database_path).await?)
    } else {
        None
    };

    match apply_versioned_migrations(pool).await {
        Ok(applied) => Ok(applied),
        Err(e) => {
            if let Some(backup_path) = backup_path {
                pool.close().await;
                restore_database_file(database_path, &backup_path).await?;
            }
            Err(e)
        }
    }
}

fn database_url_to_path(database_url: &str) -> Option<String> {
    database_url
        .strip_prefix("sqlite://")
        .map(|s| s.trim_start_matches('/').to_string())
}

fn describe_db_file(database_path: &str) -> String {
    let cwd = env::current_dir()
        .ok()
        .and_then(|p| p.canonicalize().ok())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "<unknown>".to_string());

    let absolute = Path::new(database_path)
        .canonicalize()
        .ok()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| database_path.to_string());

    let meta = fs::metadata(database_path).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);

    let header_hex = fs::read(database_path)
        .ok()
        .and_then(|bytes| bytes.get(0..16).map(|h| h.to_vec()))
        .map(|h| hex::encode(h))
        .unwrap_or_else(|| "<unreadable>".to_string());

    format!("cwd={cwd}, db_path={absolute}, size={size}, header16={header_hex}")
}

fn sqlite_quote_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

async fn backup_database_file(pool: &SqlitePool, database_path: &str) -> Result<String> {
    if !Path::new(database_path).exists() {
        return Err(anyhow::anyhow!("数据库文件不存在: {database_path}"));
    }

    let mut backup_path = format!(
        "{}.bak.{}",
        database_path,
        chrono::Utc::now().timestamp()
    );

    if Path::new(&backup_path).exists() {
        backup_path = format!(
            "{}.bak.{}.{}",
            database_path,
            chrono::Utc::now().timestamp(),
            rand::random::<u32>()
        );
    }

    let vacuum_sql = format!("VACUUM INTO {}", sqlite_quote_string(&backup_path));
    if sqlx::query(&vacuum_sql).execute(pool).await.is_err() {
        std::fs::copy(database_path, &backup_path)
            .map_err(|e| anyhow::anyhow!("备份数据库失败: {e}; src={database_path}, dst={backup_path}"))?;
    }

    Ok(backup_path)
}

async fn restore_database_file(database_path: &str, backup_path: &str) -> Result<()> {
    if !Path::new(backup_path).exists() {
        return Err(anyhow::anyhow!("备份文件不存在: {backup_path}"));
    }

    std::fs::copy(backup_path, database_path)
        .map_err(|e| anyhow::anyhow!("恢复数据库失败: {e}; src={backup_path}, dst={database_path}"))?;
    Ok(())
}

#[allow(dead_code)]
fn is_fatal_db_error(error: &anyhow::Error) -> bool {
    let msg = error.to_string();
    msg.contains("file is not a database") || msg.contains("code: 26")
}

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
        let db_password_for_hook = db_password.to_string();
        
        // 不使用 pragma 选项，完全依赖 after_connect 钩子来设置密钥
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);
        
        // 使用 PoolOptions 并配置 after_connect 钩子
        let pool = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
            .max_connections(1)  // 限制为单连接，避免多连接密钥问题
            .min_connections(1)
            .after_connect(move |conn, _meta| {
                let password = db_password_for_hook.clone();
                Box::pin(async move {
                    // SQLCipher: PRAGMA key 必须是第一条命令
                    sqlx::query(&format!("PRAGMA key = '{}'", password.replace("'", "''")))
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(options)
            .await?;
        
        // 验证数据库是否可以正常访问（执行实际查询）
        let schema_check: Result<i64, _> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master"
        )
        .fetch_one(&pool)
        .await;

        if let Err(e) = schema_check {
            let error_msg = e.to_string();
            let diag = database_url_to_path(database_url)
                .map(|p| describe_db_file(&p))
                .unwrap_or_else(|| format!("cwd={}", env::current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "<unknown>".to_string())));

            eprintln!("数据库解密失败详细信息: {}", error_msg);
            eprintln!("诊断信息: {}", diag);

            let db_path = database_url_to_path(database_url);
            let file_exists = db_path.as_ref().map(|p| Path::new(p).exists()).unwrap_or(false);
            let file_size = db_path.as_ref()
                .and_then(|p| fs::metadata(p).ok())
                .map(|m| m.len())
                .unwrap_or(0);

            if !file_exists || file_size == 0 {
                return Err(anyhow::anyhow!("数据库文件丢失或已损坏，请尝试恢复出厂设置"));
            }

            if error_msg.contains("database disk image is malformed") {
                return Err(anyhow::anyhow!("数据库文件损坏，请尝试恢复出厂设置"));
            }

            return Err(anyhow::anyhow!("密码错误，请检查后重试"));
        }

        println!("数据库加密验证成功");
        
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
#[allow(dead_code)]
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // 首先检查 chains 表是否存在
    let chains_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chains'"
    )
    .fetch_one(pool)
    .await?;

    // 如果 chains 表不存在，说明是新建数据库，跳过迁移
    // init.sql 会负责创建包含 ecosystem 列的新表结构
    if chains_table_exists == 0 {
        println!("数据库是新建的，跳过迁移，init.sql 将负责创建表结构");
        return Ok(());
    }

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

/// 全局数据库连接池（使用 RwLock 支持运行时更新）
static DATABASE_POOL: OnceLock<RwLock<SqlitePool>> = OnceLock::new();

/// 数据库加密密码存储
static DB_PASSWORD: OnceLock<String> = OnceLock::new();

/// 初始化全局数据库连接池
fn init_global_pool(pool: SqlitePool) {
    if DATABASE_POOL.get().is_none() {
        let _ = DATABASE_POOL.set(RwLock::new(pool));
    }
}

/// 更新全局数据库连接池
fn update_global_pool(pool: SqlitePool) {
    if let Some(lock) = DATABASE_POOL.get() {
        let mut guard = lock.write().unwrap();
        *guard = pool;
    } else {
        let _ = DATABASE_POOL.set(RwLock::new(pool));
    }
}

/// 获取全局数据库连接池的克隆
pub fn get_database_pool() -> SqlitePool {
    DATABASE_POOL
        .get()
        .expect("Database pool not initialized")
        .read()
        .unwrap()
        .clone()
}

/// 初始化测试用的全局数据库连接池（仅用于测试）
#[cfg(test)]
pub fn init_test_pool(pool: SqlitePool) {
    // 使用 update_global_pool 以支持多个测试
    update_global_pool(pool);
}

/// 检查数据库文件是否存在
#[allow(dead_code)]
pub fn database_exists() -> bool {
    Path::new("data/wallets_tool.db").exists()
}

/// 检查数据库是否已加密（通过检查是否能无密码访问）
/// 注意：需要区分普通SQLite文件（头部为"SQLite format 3"）和SQLCipher加密文件
pub async fn is_database_encrypted() -> Result<bool> {
    use tokio::fs::read as async_read;
    
    let database_path = "data/wallets_tool.db";
    if !Path::new(database_path).exists() {
        return Ok(false); // 数据库不存在，视为未加密（新数据库）
    }
    
    // 首先检查文件头，区分普通SQLite和SQLCipher
    // SQLCipher加密文件的文件头与普通SQLite不同（没有"SQLite format 3"）
    let file_header = async_read(database_path).await
        .ok()
        .and_then(|bytes| bytes.get(0..16).map(|h| h.to_vec()))
        .unwrap_or_default();
    
    let header_str = String::from_utf8_lossy(&file_header);
    let is_plain_sqlite = header_str.starts_with("SQLite format 3");
    
    if is_plain_sqlite {
        // 文件头显示是普通SQLite文件，不是SQLCipher加密的
        return Ok(false);
    }
    
    // 如果不是标准SQLite头部，说明是SQLCipher加密的
    // 无需尝试连接，直接返回 true
    Ok(true)
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
    let db_exists = Path::new(&database_path).exists();
    
    // 检查是否已加密
    let is_encrypted = if db_exists {
        is_database_encrypted().await.unwrap_or(false)
    } else {
        false
    };
    
    // 如果存在普通 SQLite 数据库（非加密），在创建连接前删除它
    // 这样可以避免文件锁定问题，用户设置密码后会创建新的加密数据库
    if db_exists && !is_encrypted {
        println!("检测到旧版本非加密数据库，正在删除以便创建加密版本...");
        if let Err(e) = std::fs::remove_file(database_path) {
            // 如果删除失败，尝试重命名为备份
            let backup_path = format!("{}.backup.{}", database_path, chrono::Utc::now().timestamp());
            if let Err(e2) = std::fs::rename(database_path, &backup_path) {
                return Err(anyhow::anyhow!(
                    "无法删除旧数据库文件: {}\n删除错误: {}\n重命名错误: {}",
                    database_path, e, e2
                ));
            }
            println!("已将旧数据库备份到: {backup_path}");
        } else {
            println!("已删除旧数据库文件");
        }
    }
    
    if is_encrypted {
        println!("数据库已加密，等待密码解锁...");
        // 加密数据库需要密码才能打开，创建内存数据库作为占位符
        // 同时加载基础配置数据，确保转账/余额查询等功能可用
        let memory_url = "sqlite::memory:";
        let manager = DatabaseManager::new(memory_url).await?;
        init_global_pool(manager.get_pool().clone());
        
        // 在内存数据库中加载基础配置数据
        load_init_sql_to_pool(enable_debug).await?;
        
        return Ok(());
    }
    
    // 数据库不存在或已删除，创建内存数据库并加载基础配置数据
    // 这样即使用户未初始化钱包管理，转账/余额查询等功能也能正常使用
    let memory_url = "sqlite::memory:";
    let manager = DatabaseManager::new(memory_url).await?;
    init_global_pool(manager.get_pool().clone());
    
    // 加载 init.sql 中的基础配置数据（区块链、RPC、代币等）
    load_init_sql_to_pool(enable_debug).await?;

    println!("数据库初始化完成（基础配置已加载，等待用户设置密码创建加密数据库）");
    
    Ok(())
}

/// 将 init.sql 内容加载到当前数据库连接池
async fn load_init_sql_to_pool(enable_debug: bool) -> Result<()> {
    let pool = get_database_pool();
    let init_sql = INIT_SQL_CONTENT;
    
    // 尝试整体执行
    match sqlx::query(init_sql).execute(&pool).await {
        Ok(_) => {
            if enable_debug {
                println!("init.sql 整体执行成功");
            }
        }
        Err(_) => {
            // 整体执行失败，逐条执行
            let statements = parse_sql_statements(init_sql);
            let mut executed_count = 0;
            
            for statement in statements {
                let stmt_trimmed = statement.trim();
                if !stmt_trimmed.is_empty() && !stmt_trimmed.starts_with("--") {
                    match sqlx::query(&statement).execute(&pool).await {
                        Ok(_) => {
                            executed_count += 1;
                        }
                        Err(e) => {
                            // 忽略"表/索引已存在"错误
                            let error_msg = e.to_string();
                            if error_msg.contains("already exists") {
                                continue;
                            }
                            if enable_debug {
                                eprintln!("执行 SQL 语句警告: {e}");
                            }
                        }
                    }
                }
            }
            
            if enable_debug {
                println!("init.sql 逐条执行完成，共 {executed_count} 条语句");
            }
        }
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
    
    // 检查数据库是否已经是加密的
    let is_encrypted = if db_exists {
        is_database_encrypted().await.unwrap_or(false)
    } else {
        false
    };
    
    let manager = if is_encrypted {
        // 数据库已经是加密的，用密码打开
        if enable_debug {
            println!("检测到已加密数据库，正在解锁...");
        }
        DatabaseManager::new_encrypted(&database_url, &db_password).await?
    } else {
        // 创建新的加密数据库
        if enable_debug {
            println!("创建新的加密数据库...");
        }
        DatabaseManager::new_encrypted(&database_url, &db_password).await?
    };
    
    if let Err(e) = apply_migrations_with_backup(manager.get_pool(), database_path).await {
        let diag = describe_db_file(database_path);
        return Err(anyhow::anyhow!("数据库迁移失败: {e}; {diag}"));
    }

    // 更新全局数据库连接池（替换内存数据库占位符）
    update_global_pool(manager.get_pool().clone());
    if enable_debug {
        println!("已更新全局数据库连接池");
    }

    // 初始化表结构和基础数据
    if !db_exists || config.force_init.unwrap_or(false) {
        if enable_debug {
            println!("开始初始化加密数据库（执行 init.sql）...");
        }

        let pool = get_database_pool();
        
        // 使用编译时嵌入的 init.sql 内容初始化数据库
        // 包含所有表结构和基础数据（区块链配置、RPC、代币等）
        let init_sql = INIT_SQL_CONTENT;
        
        // 尝试整体执行
        match sqlx::query(init_sql).execute(&pool).await {
            Ok(_) => {
                if enable_debug {
                    println!("init.sql 整体执行成功");
                }
            }
            Err(_) => {
                // 整体执行失败，逐条执行
                let statements = parse_sql_statements(init_sql);
                let mut executed_count = 0;
                
                for statement in statements {
                    let stmt_trimmed = statement.trim();
                    if !stmt_trimmed.is_empty() && !stmt_trimmed.starts_with("--") {
                        match sqlx::query(&statement).execute(&pool).await {
                            Ok(_) => {
                                executed_count += 1;
                            }
                            Err(e) => {
                                // 忽略"表/索引已存在"错误
                                let error_msg = e.to_string();
                                if error_msg.contains("already exists") {
                                    continue;
                                }
                                eprintln!("执行 SQL 语句错误: {e}");
                                eprintln!("错误语句: {statement}");
                                return Err(anyhow::anyhow!("初始化数据库失败: {e}"));
                            }
                        }
                    }
                }
                
                if enable_debug {
                    println!("init.sql 逐条执行完成，共 {executed_count} 条语句");
                }
            }
        }

        println!("加密数据库初始化完成（包含基础配置数据）");
    }

    let pool = get_database_pool();
    let wallet_manager_service =
        crate::wallets_tool::wallet_manager::service::WalletManagerService::new(pool.clone());
    wallet_manager_service.init_tables().await?;
    crate::wallets_tool::airdrop::db::init_airdrop_tables(&pool).await?;
    
    Ok(())
}

/// 使用密码解锁加密数据库（后续启动时调用）
pub async fn unlock_encrypted_database(password: &str) -> Result<()> {
    let database_path = "data/wallets_tool.db";
    let database_url = format!("sqlite://{database_path}");
    
    // 检查数据库是否真的是加密的
    let is_encrypted = is_database_encrypted().await?;
    
    let manager = if is_encrypted {
        // 数据库是加密的，尝试用密码解锁
        println!("检测到加密数据库，正在解锁...");
        let salt = derive_salt_from_password(password);
        let db_password = generate_db_password(password, &salt);
        
        // 存储数据库密码
        let _ = DB_PASSWORD.set(db_password.clone());
        
        DatabaseManager::new_encrypted(&database_url, &db_password).await?
    } else {
        // 数据库是普通的，无需加密
        println!("数据库未加密，创建普通连接...");
        
        // 存储空密码或原密码用于迁移
        let salt = derive_salt_from_password(password);
        let db_password = generate_db_password(password, &salt);
        let _ = DB_PASSWORD.set(db_password.clone());
        
        DatabaseManager::new(&database_url).await?
    };
    
    if let Err(e) = apply_migrations_with_backup(manager.get_pool(), database_path).await {
        let diag = describe_db_file(database_path);
        return Err(anyhow::anyhow!("数据库迁移失败: {e}; {diag}"));
    }

    // 更新全局数据库连接池（替换内存数据库占位符）
    update_global_pool(manager.get_pool().clone());

    let pool = get_database_pool();
    let wallet_manager_service =
        crate::wallets_tool::wallet_manager::service::WalletManagerService::new(pool.clone());
    wallet_manager_service.init_tables().await?;
    crate::wallets_tool::airdrop::db::init_airdrop_tables(&pool).await?;

    Ok(())
}

/// 获取数据库密码
#[allow(dead_code)]
pub fn get_db_password() -> Option<&'static String> {
    DB_PASSWORD.get()
}

/// 动态数据库管理器引用（每次调用 get_pool 时从全局获取最新的 pool）
pub struct DatabaseManagerRef;

impl DatabaseManagerRef {
    pub fn get_pool(&self) -> SqlitePool {
        get_database_pool()
    }
}

/// 全局数据库管理器引用（兼容旧 API）
static DATABASE_MANAGER_REF: DatabaseManagerRef = DatabaseManagerRef;

/// 获取全局数据库管理器（兼容旧 API，返回动态引用）
pub fn get_database_manager() -> &'static DatabaseManagerRef {
    &DATABASE_MANAGER_REF
}

/// 恢复出厂设置（删除所有数据并重新初始化数据库）
/// 这是一个彻底的数据库重置操作，会删除所有数据
#[tauri::command]
pub async fn reload_database() -> Result<String, String> {
    let config = load_database_config();
    let enable_debug = config.enable_debug_log.unwrap_or(false);
    
    if enable_debug {
        println!("开始恢复出厂设置...");
    }
    
    // 获取当前连接池
    let pool = get_database_pool();
    
    // 禁用外键约束
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&pool)
        .await
        .map_err(|e| format!("禁用外键约束失败: {e}"))?;
    
    // 获取所有表名
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("获取表名失败: {e}"))?;
    
    // 删除所有表
    for table in &tables {
        let drop_sql = format!("DROP TABLE IF EXISTS {}", table);
        if let Err(e) = sqlx::query(&drop_sql).execute(&pool).await {
            if enable_debug {
                println!("警告：删除表 {} 失败: {}", table, e);
            }
        } else if enable_debug {
            println!("已删除表: {}", table);
        }
    }
    
    // 删除所有索引
    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("获取索引名失败: {e}"))?;
    
    for index in &indexes {
        let drop_sql = format!("DROP INDEX IF EXISTS {}", index);
        if let Err(e) = sqlx::query(&drop_sql).execute(&pool).await {
            if enable_debug {
                println!("警告：删除索引 {} 失败: {}", index, e);
            }
        }
    }
    
    if enable_debug {
        println!("已删除所有表和索引");
    }
    
    // 使用编译时嵌入的 init.sql 内容（确保打包后也能正常工作）
    let init_sql = INIT_SQL_CONTENT;
    
    if enable_debug {
        println!("已加载 SQL 内容，大小: {} 字节", init_sql.len());
    }
    
    // 执行SQL语句
    let mut executed_count = 0;
    
    match sqlx::query(init_sql).execute(&pool).await {
        Ok(_) => {
            executed_count = 1;
            if enable_debug {
                println!("整体执行SQL成功");
            }
        }
        Err(_) => {
            let statements = parse_sql_statements(&init_sql);
            
            for statement in statements {
                let stmt_trimmed = statement.trim();
                if !stmt_trimmed.is_empty() && !stmt_trimmed.starts_with("--") {
                    match sqlx::query(&statement).execute(&pool).await {
                        Ok(_) => {
                            executed_count += 1;
                            if enable_debug {
                                println!("执行 SQL 语句成功: {executed_count} 个");
                            }
                        }
                        Err(e) => {
                            // 忽略"表/索引已存在"错误
                            let error_msg = e.to_string();
                            if error_msg.contains("already exists") {
                                if enable_debug {
                                    println!("跳过已存在的对象: {statement}");
                                }
                                continue;
                            }
                            eprintln!("执行 SQL 语句错误: {e}");
                            eprintln!("错误语句: {statement}");
                            return Err(format!("执行SQL语句失败: {e}"));
                        }
                    }
                }
            }
        }
    }
    
    // 重新启用外键约束
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .map_err(|e| format!("启用外键约束失败: {e}"))?;
    
    if enable_debug {
        println!("恢复出厂设置完成，共执行 {executed_count} 个 SQL 语句");
    }
    
    Ok(format!("恢复出厂设置成功，共执行 {executed_count} 个 SQL 语句"))
}

/// 检查数据库结构是否需要更新
#[tauri::command]
pub async fn check_database_schema() -> Result<serde_json::Value, String> {
    let pool = get_database_pool();
    
    let chains_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chains'"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;
    
    let tokens_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tokens'"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;
    
    let rpc_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='rpc_providers'"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;
    
    let contract_type_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('tokens') WHERE name = 'contract_type'"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;
    
    let abi_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('tokens') WHERE name = 'abi'"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {e}"))?;

    let ecosystem_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('chains') WHERE name = 'ecosystem'"
    )
    .fetch_one(&pool)
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
    let pool = get_database_pool();
    
    if enable_debug {
        println!("开始导出数据库数据到init.sql...");
    }
    
    let mut sql_content = String::new();
    
    sql_content.push_str("-- Wallet Manager 数据库初始化脚本\n");
    sql_content.push_str("-- 此文件由系统自动生成，包含当前数据库的所有数据\n\n");
    
    let all_tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(&pool)
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
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("获取表 {table} 结构失败: {e}"))?;
        
        sql_content.push_str(&format!("-- 创建{table}表\n"));
        sql_content.push_str(&create_sql);
        sql_content.push_str(";\n\n");
        
        let rows = sqlx::query(&format!("SELECT * FROM {table}"))
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("获取表 {table} 数据失败: {e}"))?;
        
        if !rows.is_empty() {
            let columns: Vec<String> = sqlx::query_scalar(
                "SELECT name FROM pragma_table_info(?) ORDER BY cid"
            )
            .bind(table)
            .fetch_all(&pool)
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
    .fetch_all(&pool)
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

/// 检查钱包数据库是否已就绪（加密数据库已初始化，即用户已设置密码）
#[tauri::command]
pub async fn is_wallet_db_ready() -> Result<bool, String> {
    // 检查加密数据库文件是否存在
    let database_path = "data/wallets_tool.db";
    if !Path::new(database_path).exists() {
        return Ok(false);
    }
    
    // 检查数据库是否是加密的（用户已设置密码）
    let is_encrypted = is_database_encrypted().await.unwrap_or(false);
    if !is_encrypted {
        return Ok(false);
    }
    
    // 检查数据库密码是否已设置（数据库已解锁）
    if DB_PASSWORD.get().is_none() {
        return Ok(false);
    }
    
    // 检查数据库池是否已初始化
    if DATABASE_POOL.get().is_none() {
        return Ok(false);
    }
    
    let pool = get_database_pool();
    
    // 检查 wallets 表是否存在
    let wallets_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='wallets'"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("检查数据库状态失败: {e}"))?;
    
    Ok(wallets_table_exists > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqliteConnectOptions;
    use std::str::FromStr;

    #[tokio::test]
    async fn new_encrypted_failure_contains_diag() {
        let mut path = std::env::temp_dir();
        path.push(format!("walletstool_plain_{}.db", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
        let database_url = format!("sqlite://{}", path.display());

        let options = SqliteConnectOptions::from_str(&database_url)
            .unwrap()
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS chains (id INTEGER PRIMARY KEY)").execute(&pool).await.unwrap();
        pool.close().await;

        let result = DatabaseManager::new_encrypted(&database_url, "passphrase").await;
        if let Err(e) = result {
            assert!(e.to_string().contains("cwd="));
        }

        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn versioned_migration_adds_ecosystem_column() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE chains (id INTEGER PRIMARY KEY AUTOINCREMENT, chain_key TEXT NOT NULL UNIQUE)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO chains(chain_key) VALUES('sol')")
            .execute(&pool)
            .await
            .unwrap();

        let applied = apply_versioned_migrations(&pool).await.unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].version, 1);
        assert!(applied[0].applied);

        let ecosystem: String = sqlx::query_scalar("SELECT ecosystem FROM chains WHERE chain_key = 'sol'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(ecosystem, "solana");

        let recorded: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations WHERE version = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(recorded, 1);

        let second = apply_versioned_migrations(&pool).await.unwrap();
        assert!(second.is_empty());
    }

    #[tokio::test]
    async fn versioned_migration_records_when_already_satisfied() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE chains (id INTEGER PRIMARY KEY AUTOINCREMENT, chain_key TEXT NOT NULL UNIQUE, ecosystem TEXT NOT NULL DEFAULT 'evm')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let applied = apply_versioned_migrations(&pool).await.unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].version, 1);
        assert!(!applied[0].applied);

        let recorded_applied: i64 = sqlx::query_scalar(
            "SELECT applied FROM schema_migrations WHERE version = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(recorded_applied, 0);
    }
}
