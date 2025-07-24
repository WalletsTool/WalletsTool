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
    let enable_debug = config.enable_debug_log.unwrap_or(false);
    
    if enable_debug {
        println!("使用的数据库配置: {:?}", config);
    }

    // 确保配置目录存在
    std::fs::create_dir_all("data")?;

    let database_path = "data/web3_tools.db";
    let database_url = format!("sqlite://{}", database_path);
    let db_exists = Path::new(&database_path).exists();
    
    // 始终创建数据库管理器
    let manager = DatabaseManager::new(&database_url).await?;
    DATABASE_MANAGER.set(manager)
        .map_err(|_| anyhow::anyhow!("Database already initialized"))?;

    // 如果需要初始化数据库
    if config.force_init.unwrap_or(false) || !db_exists {
        if enable_debug {
            println!("开始初始化数据库...");
            println!("数据库路径: {}", database_path);
        }

        // 使用自定义路径加载初始 SQL 文件
        let init_sql_path = config.init_sql_path.unwrap_or_else(|| "data/init.sql".to_string());
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
                // 如果整体执行失败，尝试逐条执行
                if enable_debug {
                    println!("整体执行失败，尝试逐条执行: {}", e);
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
                                    println!("执行 SQL 语句成功: {} 个", executed_count);
                                }
                            }
                            Err(e) => {
                                // 忽略重复插入错误（UNIQUE constraint failed）
                                if !e.to_string().contains("UNIQUE constraint failed") && !e.to_string().contains("already exists") {
                                    eprintln!("执行 SQL 语句错误: {}", e);
                                    eprintln!("错误语句: {}", statement);
                                    return Err(e.into());
                                } else if enable_debug {
                                    println!("忽略重复创建/插入错误");
                                }
                            }
                        }
                    }
                }
                if enable_debug {
                    println!("逐条执行完成，共执行 {} 个 SQL 语句", executed_count);
                }
            }
        }

        println!("数据库初始化完成");
    }
    
    // 运行数据库迁移
    let pool = DATABASE_MANAGER.get().expect("Database manager not initialized").get_pool();
    if let Err(e) = migrations::run_migrations(pool).await {
        eprintln!("数据库迁移失败: {}", e);
        return Err(e);
    }
    
    if enable_debug {
        println!("数据库迁移完成");
    }
    
    Ok(())
}

/// 获取全局数据库管理器
pub fn get_database_manager() -> &'static DatabaseManager {
    DATABASE_MANAGER.get().expect("Database not initialized")
}

/// 手动触发数据库迁移（热重载功能）
#[tauri::command]
pub async fn reload_database() -> Result<String, String> {
    let pool = get_database_manager().get_pool();
    
    match migrations::run_migrations(pool).await {
        Ok(_) => {
            println!("数据库迁移完成");
            Ok("数据库迁移成功完成".to_string())
        }
        Err(e) => {
            eprintln!("数据库迁移失败: {}", e);
            Err(format!("数据库迁移失败: {}", e))
        }
    }
}

/// 检查数据库结构是否需要更新
#[tauri::command]
pub async fn check_database_schema() -> Result<serde_json::Value, String> {
    let pool = get_database_manager().get_pool();
    
    // 检查tokens表是否包含contract_type列
    let contract_type_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('tokens') WHERE name = 'contract_type'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {}", e))?;
    
    // 检查tokens表是否包含abi列
    let abi_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('tokens') WHERE name = 'abi'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {}", e))?;
    
    let schema_info = serde_json::json!({
        "contract_type_column_exists": contract_type_exists > 0,
        "abi_column_exists": abi_exists > 0,
        "needs_migration": contract_type_exists == 0 || abi_exists == 0
    });
    
    Ok(schema_info)
}
