pub mod models;
pub mod chain_service;
pub mod rpc_service;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions, Row};
use anyhow::Result;
use std::str::FromStr;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use std::path::Path;

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

    let database_path = "data/wallet_manager.db";
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
    
    Ok(())
}

/// 获取全局数据库管理器
pub fn get_database_manager() -> &'static DatabaseManager {
    DATABASE_MANAGER.get().expect("Database not initialized")
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
        .map_err(|e| format!("禁用外键约束失败: {}", e))?;
    
    // 获取所有表名
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("获取表名失败: {}", e))?;
    
    // 删除所有表（包括表结构）
    for table in &tables {
        let drop_sql = format!("DROP TABLE IF EXISTS {}", table);
        sqlx::query(&drop_sql)
            .execute(pool)
            .await
            .map_err(|e| format!("删除表 {} 失败: {}", table, e))?;
        
        if enable_debug {
            println!("已删除表: {}", table);
        }
    }
    
    // 清理sqlite_sequence表
    sqlx::query("DELETE FROM sqlite_sequence")
        .execute(pool)
        .await
        .ok(); // 忽略错误，因为表可能不存在
    
    if enable_debug {
        println!("已删除所有表，开始重新创建...");
    }
    
    // 从init.sql重新导入数据
    let init_sql_path = config.init_sql_path.unwrap_or_else(|| "data/init.sql".to_string());
    let init_sql = std::fs::read_to_string(&init_sql_path)
        .map_err(|e| format!("读取init.sql文件失败: {}", e))?;
    
    if enable_debug {
        println!("已加载 SQL 文件，大小: {} 字节", init_sql.len());
    }
    
    // 执行SQL语句 - 使用更智能的解析方式
    let mut executed_count = 0;
    
    // 先尝试整体执行
    match sqlx::query(&init_sql).execute(pool).await {
        Ok(_) => {
            executed_count = 1;
            if enable_debug {
                println!("整体执行SQL成功");
            }
        }
        Err(_) => {
            // 整体执行失败，尝试智能分割
            if enable_debug {
                println!("整体执行失败，尝试智能分割执行");
            }
            
            let statements = parse_sql_statements(&init_sql);
            
            for statement in statements {
                if !statement.trim().is_empty() && !statement.trim().starts_with("--") {
                    match sqlx::query(&statement).execute(pool).await {
                        Ok(_) => {
                            executed_count += 1;
                            if enable_debug {
                                println!("执行 SQL 语句成功: {} 个", executed_count);
                            }
                        }
                        Err(e) => {
                            eprintln!("执行 SQL 语句错误: {}", e);
                            eprintln!("错误语句: {}", statement);
                            // 重新启用外键约束检查
                            sqlx::query("PRAGMA foreign_keys = ON")
                                .execute(pool)
                                .await
                                .ok();
                            return Err(format!("执行SQL语句失败: {}", e));
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
        .map_err(|e| format!("启用外键约束失败: {}", e))?;
    
    if enable_debug {
        println!("数据库重新加载完成，共执行 {} 个 SQL 语句", executed_count);
    }
    
    Ok(format!("数据库重新加载成功，共执行 {} 个 SQL 语句", executed_count))
}

/// 检查数据库结构是否需要更新
#[tauri::command]
pub async fn check_database_schema() -> Result<serde_json::Value, String> {
    let pool = get_database_manager().get_pool();
    
    // 检查数据库中是否存在必要的表
    let chains_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chains'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {}", e))?;
    
    let tokens_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tokens'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {}", e))?;
    
    let rpc_table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='rpc_providers'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("检查数据库结构失败: {}", e))?;
    
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
        "db_exists": true, // 数据库连接成功意味着数据库文件存在
        "chains_table_exists": chains_table_exists > 0,
        "tokens_table_exists": tokens_table_exists > 0,
        "rpc_table_exists": rpc_table_exists > 0,
        "contract_type_column_exists": contract_type_exists > 0,
        "abi_column_exists": abi_exists > 0,
        "needs_migration": contract_type_exists == 0 || abi_exists == 0
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
    
    // 添加文件头注释
    sql_content.push_str("-- Wallet Manager 数据库初始化脚本\n");
    sql_content.push_str("-- 此文件由系统自动生成，包含当前数据库的所有数据\n\n");
    
    // 获取所有表名
    let all_tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("获取表名失败: {}", e))?;
    
    // 按照外键依赖关系排序表，确保被引用的表先创建
    let mut tables = Vec::new();
    
    // 定义表的依赖顺序：被依赖的表在前
    let table_order = vec![
        "chains",           // 基础表，被其他表引用
        "rpc_providers",    // 依赖 chains
        "tokens",           // 依赖 chains
    ];
    
    // 按照预定义顺序添加存在的表
    for table_name in &table_order {
        if all_tables.contains(&table_name.to_string()) {
            tables.push(table_name.to_string());
        }
    }
    
    // 添加其他未在预定义顺序中的表
    for table in &all_tables {
        if !tables.contains(table) {
            tables.push(table.clone());
        }
    }
    
    if enable_debug {
        println!("表创建顺序: {:?}", tables);
    }
    
    // 为每个表生成CREATE TABLE语句和INSERT语句
    for table in &tables {
        if enable_debug {
            println!("正在导出表: {}", table);
        }
        
        // 获取表结构
        let create_sql: String = sqlx::query_scalar(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name = ?"
        )
        .bind(table)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("获取表 {} 结构失败: {}", table, e))?;
        
        sql_content.push_str(&format!("-- 创建{}表\n", table));
        sql_content.push_str(&create_sql);
        sql_content.push_str(";\n\n");
        
        // 获取表数据
        let rows = sqlx::query(&format!("SELECT * FROM {}", table))
            .fetch_all(pool)
            .await
            .map_err(|e| format!("获取表 {} 数据失败: {}", table, e))?;
        
        if !rows.is_empty() {
            // 获取列名
            let columns: Vec<String> = sqlx::query_scalar(
                "SELECT name FROM pragma_table_info(?) ORDER BY cid"
            )
            .bind(table)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("获取表 {} 列名失败: {}", table, e))?;
            
            sql_content.push_str(&format!("-- 插入{}表数据\n", table));
            
            for row in rows {
                let mut values = Vec::new();
                for (i, _column) in columns.iter().enumerate() {
                    // 尝试获取不同类型的值
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
                            // 转义单引号并添加引号
                            let escaped = v.replace("'", "''");
                            values.push(format!("'{}'", escaped));
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
            sql_content.push_str("\n");
        }
    }
    
    // 获取索引
    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE type='index' AND sql IS NOT NULL AND name NOT LIKE 'sqlite_%'"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("获取索引失败: {}", e))?;
    
    if !indexes.is_empty() {
        sql_content.push_str("-- 创建索引\n");
        for index_sql in indexes {
            sql_content.push_str(&index_sql);
            sql_content.push_str(";\n");
        }
    }
    
    // 写入文件
    let init_sql_path = config.init_sql_path.unwrap_or_else(|| "data/init.sql".to_string());
    std::fs::write(&init_sql_path, &sql_content)
        .map_err(|e| format!("写入init.sql文件失败: {}", e))?;
    
    if enable_debug {
        println!("数据库导出完成，文件大小: {} 字节", sql_content.len());
    }
    
    Ok(format!("数据库数据已成功导出到 {}，共 {} 个表，文件大小: {} 字节", 
        init_sql_path, tables.len(), sql_content.len()))
}
