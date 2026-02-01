use sqlx::SqlitePool;
use anyhow::Result;

/// 初始化浏览器自动化相关的数据库表
pub async fn init_airdrop_tables(pool: &SqlitePool) -> Result<()> {
    // 空投钱包表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS airdrop_wallets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            encrypted_private_key TEXT NOT NULL,
            label TEXT,
            group_name TEXT NOT NULL DEFAULT 'Default',
            proxy TEXT NOT NULL DEFAULT 'Direct',
            chain_type TEXT NOT NULL DEFAULT 'evm',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;

    // 创建索引
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_airdrop_wallets_address ON airdrop_wallets(address)"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_airdrop_wallets_group ON airdrop_wallets(group_name)"
    )
    .execute(pool)
    .await?;

    // 浏览器环境配置表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS browser_profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            user_agent TEXT,
            viewport_width INTEGER NOT NULL DEFAULT 1920,
            viewport_height INTEGER NOT NULL DEFAULT 1080,
            device_scale_factor REAL NOT NULL DEFAULT 1.0,
            locale TEXT NOT NULL DEFAULT 'en-US',
            timezone_id TEXT NOT NULL DEFAULT 'America/New_York',
            proxy_type TEXT NOT NULL DEFAULT 'direct',
            proxy_host TEXT,
            proxy_port INTEGER,
            proxy_username TEXT,
            proxy_password TEXT,
            canvas_spoof BOOLEAN NOT NULL DEFAULT 1,
            webgl_spoof BOOLEAN NOT NULL DEFAULT 1,
            audio_spoof BOOLEAN NOT NULL DEFAULT 1,
            timezone_spoof BOOLEAN NOT NULL DEFAULT 1,
            geolocation_spoof BOOLEAN NOT NULL DEFAULT 1,
            font_spoof BOOLEAN NOT NULL DEFAULT 1,
            webrtc_spoof BOOLEAN NOT NULL DEFAULT 1,
            navigator_override BOOLEAN NOT NULL DEFAULT 1,
            webdriver_override BOOLEAN NOT NULL DEFAULT 1,
            custom_headers TEXT,
            headless BOOLEAN NOT NULL DEFAULT 0,
            extensions TEXT,
            is_default BOOLEAN NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_browser_profiles_default ON browser_profiles(is_default)"
    )
    .execute(pool)
    .await?;

    // 自动化脚本表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS automation_scripts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            content TEXT NOT NULL,
            compiled_content TEXT,
            version INTEGER NOT NULL DEFAULT 1,
            is_system BOOLEAN NOT NULL DEFAULT 0,
            required_apis TEXT,
            author TEXT,
            tags TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_automation_scripts_tags ON automation_scripts(tags)"
    )
    .execute(pool)
    .await?;

    // 自动化任务表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS automation_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            script_id INTEGER NOT NULL,
            wallet_ids TEXT NOT NULL,
            profile_strategy TEXT NOT NULL DEFAULT 'random',
            specific_profile_id INTEGER,
            schedule_type TEXT NOT NULL DEFAULT 'once',
            schedule_config TEXT NOT NULL,
            concurrency INTEGER NOT NULL DEFAULT 1,
            timeout_seconds INTEGER NOT NULL DEFAULT 300,
            retry_times INTEGER NOT NULL DEFAULT 3,
            retry_interval_seconds INTEGER NOT NULL DEFAULT 60,
            status TEXT NOT NULL DEFAULT 'draft',
            last_run_time DATETIME,
            next_run_time DATETIME,
            total_runs INTEGER NOT NULL DEFAULT 0,
            success_runs INTEGER NOT NULL DEFAULT 0,
            failed_runs INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (script_id) REFERENCES automation_scripts(id),
            FOREIGN KEY (specific_profile_id) REFERENCES browser_profiles(id)
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_automation_tasks_status ON automation_tasks(status)"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_automation_tasks_next_run ON automation_tasks(next_run_time) WHERE status = 'enabled'"
    )
    .execute(pool)
    .await?;

    // 任务执行记录表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS task_executions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            wallet_id INTEGER NOT NULL,
            profile_id INTEGER,
            status TEXT NOT NULL,
            start_time DATETIME,
            end_time DATETIME,
            duration_ms INTEGER,
            error_message TEXT,
            result_data TEXT,
            logs TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES automation_tasks(id),
            FOREIGN KEY (wallet_id) REFERENCES airdrop_wallets(id),
            FOREIGN KEY (profile_id) REFERENCES browser_profiles(id)
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_task_executions_task ON task_executions(task_id)"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_task_executions_status ON task_executions(status)"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_task_executions_time ON task_executions(created_at DESC)"
    )
    .execute(pool)
    .await?;

    // 插入默认脚本
    insert_default_scripts(pool).await?;

    Ok(())
}

/// 插入默认的自动化脚本
async fn insert_default_scripts(pool: &SqlitePool) -> Result<()> {
    // 检查是否已有系统脚本
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM automation_scripts WHERE is_system = 1"
    )
    .fetch_one(pool)
    .await?;

    if count > 0 {
        return Ok(());
    }

    // 默认OKX Daily Claim脚本
    let okx_script = r#"// OKX Daily Claim Script
async function run({ page, wallet, api }) {
    api.log('info', '开始执行 OKX Daily Claim');
    
    // 1. 打开OKX官网
    await page.goto('https://www.okx.com');
    await api.waitForSelector('body');
    await api.randomDelay(2000, 4000);
    
    api.log('info', '页面加载完成');
    
    // 2. 连接钱包
    api.log('info', '连接 OKX Wallet...');
    // await api.connectOKXWallet({ chainId: '0x1' });
    
    api.log('success', '脚本执行完成');
    return { success: true };
}"#;

    sqlx::query(
        r#"
        INSERT INTO automation_scripts (name, description, content, is_system, required_apis, author, tags)
        VALUES (?, ?, ?, 1, ?, 'System', ?)
        "#
    )
    .bind("OKX Daily Claim")
    .bind("OKX每日签到脚本")
    .bind(okx_script)
    .bind(r#"["connectOKXWallet", "waitForSelector", "randomDelay"]"#)
    .bind(r#"["okx", "daily", "claim"]"#)
    .execute(pool)
    .await?;

    // 默认Uniswap脚本
    let uniswap_script = r#"// Uniswap V3 Swap Script
async function run({ page, wallet, api }) {
    api.log('info', '开始执行 Uniswap Swap');
    
    // 1. 连接钱包
    // await api.connectMetaMask({ expectedChainId: '0x1' });
    
    // 2. 打开Uniswap
    await page.goto('https://app.uniswap.org');
    await api.waitForSelector('body');
    await api.randomDelay(2000, 3000);
    
    api.log('info', 'Uniswap页面加载完成');
    
    api.log('success', '脚本执行完成');
    return { success: true };
}"#;

    sqlx::query(
        r#"
        INSERT INTO automation_scripts (name, description, content, is_system, required_apis, author, tags)
        VALUES (?, ?, ?, 1, ?, 'System', ?)
        "#
    )
    .bind("Uniswap Swap")
    .bind("Uniswap交换脚本")
    .bind(uniswap_script)
    .bind(r#"["connectMetaMask", "waitForSelector", "randomDelay"]"#)
    .bind(r#"["uniswap", "swap", "dex"]"#)
    .execute(pool)
    .await?;

    Ok(())
}
