pub mod models;
pub mod migrations;
pub mod chain_service;
pub mod rpc_service;
pub mod json_migration;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use anyhow::Result;
use std::str::FromStr;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use std::path::Path;

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
            init_sql_path: Some("sql/init.sql".to_string()),
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
                            eprintln!("解析 {} 失败: {}", path, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("读取 {} 失败: {}", path, e);
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
    
    /// 获取数据库连接池
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
    
}

/// 全局数据库管理器实例
static DATABASE_MANAGER: OnceLock<DatabaseManager> = OnceLock::new();

/// 初始化数据库
pub async fn init_database() -> Result<()> {
    let config = load_database_config();
    println!("使用的数据库配置: {:?}", config);

    // 确保配置目录存在
    std::fs::create_dir_all("conf")?;

    let database_path = "conf/web3_tools.db";
    let database_url = format!("sqlite://{}", database_path);
    let db_exists = Path::new(&database_path).exists();
    
    // 始终创建数据库管理器
    let manager = DatabaseManager::new(&database_url).await?;
    DATABASE_MANAGER.set(manager)
        .map_err(|_| anyhow::anyhow!("Database already initialized"))?;

    // 如果需要初始化数据库
    if config.force_init.unwrap_or(false) || !db_exists {
        println!("开始初始化数据库...");
        println!("数据库路径: {}", database_path);

        // 使用自定义路径加载初始 SQL 文件
        let init_sql_path = config.init_sql_path.unwrap_or_else(|| "sql/init.sql".to_string());
        let init_sql = std::fs::read_to_string(&init_sql_path)?;

        println!("已加载 SQL 文件，大小: {} 字节", init_sql.len());

        let pool = DATABASE_MANAGER.get().expect("Database manager not initialized").get_pool();

        // 将SQL文件分割为多个语句并执行
        let mut executed_count = 0;
        for statement in init_sql.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() && !statement.starts_with("--") {
                match sqlx::query(statement).execute(pool).await {
                    Ok(_) => {
                        executed_count += 1;
                        if config.enable_debug_log.unwrap_or(false) {
                            println!("执行 SQL 语句成功: {} 个", executed_count);
                        }
                    }
                    Err(e) => {
                        // 忽略重复插入错误（UNIQUE constraint failed）
                        if !e.to_string().contains("UNIQUE constraint failed") {
                            eprintln!("执行 SQL 语句错误: {}", e);
                            eprintln!("错误语句: {}", statement);
                            return Err(e.into());
                        } else if config.enable_debug_log.unwrap_or(false) {
                            println!("忽略重复插入错误");
                        }
                    }
                }
            }
        }

        println!("数据库初始化完成，共执行 {} 个 SQL 语句", executed_count);
    } else {
        println!("数据库已存在，跳过初始化。");
    }
    Ok(())
}

/// 获取全局数据库管理器
pub fn get_database_manager() -> &'static DatabaseManager {
    DATABASE_MANAGER.get().expect("Database not initialized")
}
