use tauri::State;
use sqlx::SqlitePool;
use anyhow::Result;
use serde_json;

use crate::wallets_tool::airdrop::models::*;
use crate::wallets_tool::security;

/// 获取所有空投钱包
#[tauri::command]
pub async fn get_airdrop_wallets(pool: State<'_, SqlitePool>) -> Result<Vec<AirdropWallet>, String> {
    let wallets = sqlx::query_as::<_, AirdropWallet>(
        "SELECT * FROM airdrop_wallets ORDER BY id DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| format!("获取钱包列表失败: {}", e))?;

    Ok(wallets)
}

/// 创建空投钱包
#[tauri::command]
pub async fn create_airdrop_wallet(
    pool: State<'_, SqlitePool>,
    request: CreateAirdropWalletRequest,
) -> Result<AirdropWallet, String> {
    // 验证地址格式
    let address = request.address.trim();
    if !is_valid_address(address) {
        return Err("无效的钱包地址格式".to_string());
    }

    // 检查地址是否已存在
    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM airdrop_wallets WHERE LOWER(address) = LOWER(?)"
    )
    .bind(address)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| format!("检查地址失败: {}", e))?;

    if existing.is_some() {
        return Err("该地址已存在".to_string());
    }

    // 加密私钥
    let encrypted_key = security::memory::encrypt_string(&request.private_key)
        .map_err(|e| format!("加密私钥失败: {}", e))?;

    let name = request.name.trim();
    let label = request.label.as_ref().map(|s| s.trim().to_string());
    let group_name = request.group_name.unwrap_or_else(|| "Default".to_string());
    let proxy = request.proxy.unwrap_or_else(|| "Direct".to_string());
    let chain_type = request.chain_type.unwrap_or_else(|| "evm".to_string());

    let wallet = sqlx::query_as::<_, AirdropWallet>(
        r#"
        INSERT INTO airdrop_wallets (name, address, encrypted_private_key, label, group_name, proxy, chain_type)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#
    )
    .bind(name)
    .bind(address)
    .bind(&encrypted_key)
    .bind(&label)
    .bind(&group_name)
    .bind(&proxy)
    .bind(&chain_type)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("创建钱包失败: {}", e))?;

    Ok(wallet)
}

/// 更新空投钱包
#[tauri::command]
pub async fn update_airdrop_wallet(
    pool: State<'_, SqlitePool>,
    request: UpdateAirdropWalletRequest,
) -> Result<AirdropWallet, String> {
    // 检查钱包是否存在
    let existing: Option<AirdropWallet> = sqlx::query_as(
        "SELECT * FROM airdrop_wallets WHERE id = ?"
    )
    .bind(request.id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| format!("查询钱包失败: {}", e))?;

    if existing.is_none() {
        return Err("钱包不存在".to_string());
    }

    let existing = existing.unwrap();

    // 如果更新地址，验证格式
    if let Some(ref addr) = request.address {
        if !is_valid_address(addr.trim()) {
            return Err("无效的钱包地址格式".to_string());
        }

        // 检查新地址是否与其他钱包冲突
        let conflict: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM airdrop_wallets WHERE LOWER(address) = LOWER(?) AND id != ?"
        )
        .bind(addr.trim())
        .bind(request.id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| format!("检查地址冲突失败: {}", e))?;

        if conflict.is_some() {
            return Err("该地址已被其他钱包使用".to_string());
        }
    }

    // 构建更新SQL
    let mut updates = vec![];
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send + Sync>> = vec![];

    if let Some(name) = request.name {
        updates.push("name = ?");
        params.push(Box::new(name.trim().to_string()));
    }

    if let Some(address) = request.address {
        updates.push("address = ?");
        params.push(Box::new(address.trim().to_string()));
    }

    if let Some(private_key) = request.private_key {
        let encrypted_key = security::memory::encrypt_string(&private_key)
            .map_err(|e| format!("加密私钥失败: {}", e))?;
        updates.push("encrypted_private_key = ?");
        params.push(Box::new(encrypted_key));
    }

    if let Some(label) = request.label {
        updates.push("label = ?");
        params.push(Box::new(label.trim().to_string()));
    }

    if let Some(group_name) = request.group_name {
        updates.push("group_name = ?");
        params.push(Box::new(group_name.trim().to_string()));
    }

    if let Some(proxy) = request.proxy {
        updates.push("proxy = ?");
        params.push(Box::new(proxy.trim().to_string()));
    }

    if let Some(chain_type) = request.chain_type {
        updates.push("chain_type = ?");
        params.push(Box::new(chain_type.trim().to_string()));
    }

    if updates.is_empty() {
        return Ok(existing);
    }

    updates.push("updated_at = CURRENT_TIMESTAMP");

    let sql = format!(
        "UPDATE airdrop_wallets SET {} WHERE id = ? RETURNING *",
        updates.join(", ")
    );

    let mut query = sqlx::query_as::<_, AirdropWallet>(&sql);
    for param in params {
        query = query.bind(param);
    }
    query = query.bind(request.id);

    let wallet = query
        .fetch_one(&*pool)
        .await
        .map_err(|e| format!("更新钱包失败: {}", e))?;

    Ok(wallet)
}

/// 删除空投钱包
#[tauri::command]
pub async fn delete_airdrop_wallet(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM airdrop_wallets WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| format!("删除钱包失败: {}", e))?;

    Ok(())
}

/// 批量导入空投钱包
#[tauri::command]
pub async fn import_airdrop_wallets(
    pool: State<'_, SqlitePool>,
    request: ImportWalletsRequest,
) -> Result<(Vec<AirdropWallet>, Vec<String>), String> {
    let mut wallets = vec![];
    let mut errors = vec![];

    for (idx, req) in request.wallets.into_iter().enumerate() {
        // 验证地址
        let address = req.address.trim();
        if !is_valid_address(address) {
            errors.push(format!("第{}行: 无效地址 {}", idx + 1, address));
            continue;
        }

        // 检查重复
        let existing: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM airdrop_wallets WHERE LOWER(address) = LOWER(?)"
        )
        .bind(address)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| format!("检查地址失败: {}", e))?;

        if existing.is_some() {
            errors.push(format!("第{}行: 地址 {} 已存在", idx + 1, address));
            continue;
        }

        // 加密私钥
        let encrypted_key = match security::memory::encrypt_string(&req.private_key) {
            Ok(key) => key,
            Err(e) => {
                errors.push(format!("第{}行: 加密私钥失败 {}", idx + 1, e));
                continue;
            }
        };

        let name = req.name.trim();
        let label = req.label.as_ref().map(|s| s.trim().to_string());
        let group_name = req.group_name.unwrap_or_else(|| "Default".to_string());
        let proxy = req.proxy.unwrap_or_else(|| "Direct".to_string());
        let chain_type = req.chain_type.unwrap_or_else(|| "evm".to_string());

        match sqlx::query_as::<_, AirdropWallet>(
            r#"
            INSERT INTO airdrop_wallets (name, address, encrypted_private_key, label, group_name, proxy, chain_type)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
        .bind(name)
        .bind(address)
        .bind(&encrypted_key)
        .bind(&label)
        .bind(&group_name)
        .bind(&proxy)
        .bind(&chain_type)
        .fetch_one(&*pool)
        .await
        {
            Ok(wallet) => wallets.push(wallet),
            Err(e) => errors.push(format!("第{}行: 导入失败 {}", idx + 1, e)),
        }
    }

    Ok((wallets, errors))
}

/// 获取钱包私钥（解密）
#[tauri::command]
pub async fn get_wallet_private_key(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<String, String> {
    let encrypted_key: String = sqlx::query_scalar(
        "SELECT encrypted_private_key FROM airdrop_wallets WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| format!("查询钱包失败: {}", e))?
    .ok_or_else(|| "钱包不存在".to_string())?;

    let private_key = security::memory::decrypt_string(&encrypted_key)
        .map_err(|e| format!("解密私钥失败: {}", e))?;

    Ok(private_key)
}

/// 获取所有浏览器环境配置
#[tauri::command]
pub async fn get_browser_profiles(pool: State<'_, SqlitePool>) -> Result<Vec<BrowserProfile>, String> {
    let profiles = sqlx::query_as::<_, BrowserProfile>(
        "SELECT * FROM browser_profiles ORDER BY id DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| format!("获取环境配置列表失败: {}", e))?;

    Ok(profiles)
}

/// 创建浏览器环境配置
#[tauri::command]
pub async fn create_browser_profile(
    pool: State<'_, SqlitePool>,
    request: CreateBrowserProfileRequest,
) -> Result<BrowserProfile, String> {
    let profile = sqlx::query_as::<_, BrowserProfile>(
        r#"
        INSERT INTO browser_profiles (
            name, description, user_agent, viewport_width, viewport_height,
            device_scale_factor, locale, timezone_id, proxy_type, proxy_host,
            proxy_port, proxy_username, proxy_password, canvas_spoof, webgl_spoof,
            audio_spoof, timezone_spoof, geolocation_spoof, font_spoof, webrtc_spoof,
            navigator_override, webdriver_override, custom_headers, headless,
            extensions, is_default
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.user_agent)
    .bind(request.viewport_width.unwrap_or(1920))
    .bind(request.viewport_height.unwrap_or(1080))
    .bind(request.device_scale_factor.unwrap_or(1.0))
    .bind(request.locale.unwrap_or_else(|| "en-US".to_string()))
    .bind(request.timezone_id.unwrap_or_else(|| "America/New_York".to_string()))
    .bind(request.proxy_type.unwrap_or_else(|| "direct".to_string()))
    .bind(&request.proxy_host)
    .bind(request.proxy_port)
    .bind(&request.proxy_username)
    .bind(&request.proxy_password)
    .bind(request.canvas_spoof.unwrap_or(true))
    .bind(request.webgl_spoof.unwrap_or(true))
    .bind(request.audio_spoof.unwrap_or(true))
    .bind(request.timezone_spoof.unwrap_or(true))
    .bind(request.geolocation_spoof.unwrap_or(true))
    .bind(request.font_spoof.unwrap_or(true))
    .bind(request.webrtc_spoof.unwrap_or(true))
    .bind(request.navigator_override.unwrap_or(true))
    .bind(request.webdriver_override.unwrap_or(true))
    .bind(&request.custom_headers)
    .bind(request.headless.unwrap_or(false))
    .bind(&request.extensions)
    .bind(request.is_default.unwrap_or(false))
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("创建环境配置失败: {}", e))?;

    Ok(profile)
}

/// 更新浏览器环境配置
#[tauri::command]
pub async fn update_browser_profile(
    pool: State<'_, SqlitePool>,
    request: UpdateBrowserProfileRequest,
) -> Result<BrowserProfile, String> {
    let profile = sqlx::query_as::<_, BrowserProfile>(
        r#"
        UPDATE browser_profiles SET
            name = COALESCE(?, name),
            description = COALESCE(?, description),
            user_agent = COALESCE(?, user_agent),
            viewport_width = COALESCE(?, viewport_width),
            viewport_height = COALESCE(?, viewport_height),
            device_scale_factor = COALESCE(?, device_scale_factor),
            locale = COALESCE(?, locale),
            timezone_id = COALESCE(?, timezone_id),
            proxy_type = COALESCE(?, proxy_type),
            proxy_host = COALESCE(?, proxy_host),
            proxy_port = COALESCE(?, proxy_port),
            proxy_username = COALESCE(?, proxy_username),
            proxy_password = COALESCE(?, proxy_password),
            canvas_spoof = COALESCE(?, canvas_spoof),
            webgl_spoof = COALESCE(?, webgl_spoof),
            audio_spoof = COALESCE(?, audio_spoof),
            timezone_spoof = COALESCE(?, timezone_spoof),
            geolocation_spoof = COALESCE(?, geolocation_spoof),
            font_spoof = COALESCE(?, font_spoof),
            webrtc_spoof = COALESCE(?, webrtc_spoof),
            navigator_override = COALESCE(?, navigator_override),
            webdriver_override = COALESCE(?, webdriver_override),
            custom_headers = COALESCE(?, custom_headers),
            headless = COALESCE(?, headless),
            extensions = COALESCE(?, extensions),
            is_default = COALESCE(?, is_default),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.user_agent)
    .bind(request.viewport_width)
    .bind(request.viewport_height)
    .bind(request.device_scale_factor)
    .bind(&request.locale)
    .bind(&request.timezone_id)
    .bind(&request.proxy_type)
    .bind(&request.proxy_host)
    .bind(request.proxy_port)
    .bind(&request.proxy_username)
    .bind(&request.proxy_password)
    .bind(request.canvas_spoof)
    .bind(request.webgl_spoof)
    .bind(request.audio_spoof)
    .bind(request.timezone_spoof)
    .bind(request.geolocation_spoof)
    .bind(request.font_spoof)
    .bind(request.webrtc_spoof)
    .bind(request.navigator_override)
    .bind(request.webdriver_override)
    .bind(&request.custom_headers)
    .bind(request.headless)
    .bind(&request.extensions)
    .bind(request.is_default)
    .bind(request.id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("更新环境配置失败: {}", e))?;

    Ok(profile)
}

/// 删除浏览器环境配置
#[tauri::command]
pub async fn delete_browser_profile(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM browser_profiles WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| format!("删除环境配置失败: {}", e))?;

    Ok(())
}

/// 批量生成浏览器环境配置
#[tauri::command]
pub async fn batch_generate_profiles(
    pool: State<'_, SqlitePool>,
    request: BatchGenerateProfilesRequest,
) -> Result<Vec<BrowserProfile>, String> {
    use rand::seq::SliceRandom;
    use rand::Rng;

    let user_agents = vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:121.0) Gecko/20100101 Firefox/121.0",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
    ];

    let viewports = vec![
        (1920, 1080),
        (1366, 768),
        (1440, 900),
        (1536, 864),
        (2560, 1440),
        (1280, 720),
        (390, 844),
        (414, 896),
    ];

    let locales = vec!["en-US", "zh-CN", "ja-JP", "ko-KR", "de-DE", "fr-FR", "en-GB"];
    let timezones = vec![
        "America/New_York", "America/Los_Angeles", "Europe/London",
        "Europe/Paris", "Asia/Tokyo", "Asia/Shanghai", "Asia/Singapore",
    ];

    let mut profiles = vec![];
    let mut rng = rand::thread_rng();

    for i in 0..request.count {
        let ua = user_agents.choose(&mut rng).unwrap();
        let (width, height) = viewports.choose(&mut rng).unwrap();
        let locale = locales.choose(&mut rng).unwrap();
        let timezone = timezones.choose(&mut rng).unwrap();

        let proxy_type = request.proxy_type.clone().unwrap_or_else(|| "direct".to_string());
        let proxy_host = request.proxy_host_prefix.as_ref().map(|prefix| {
            format!("{}.{}", prefix, i + 1)
        });
        let proxy_port = request.proxy_port_start.map(|start| start + i);

        let enable_all = request.enable_all_spoofs.unwrap_or(true);

        let profile = sqlx::query_as::<_, BrowserProfile>(
            r#"
            INSERT INTO browser_profiles (
                name, user_agent, viewport_width, viewport_height,
                locale, timezone_id, proxy_type, proxy_host, proxy_port,
                canvas_spoof, webgl_spoof, audio_spoof, timezone_spoof,
                geolocation_spoof, font_spoof, webrtc_spoof,
                navigator_override, webdriver_override
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
        .bind(format!("Auto-Profile-{}", i + 1))
        .bind(*ua)
        .bind(*width)
        .bind(*height)
        .bind(*locale)
        .bind(*timezone)
        .bind(&proxy_type)
        .bind(&proxy_host)
        .bind(proxy_port)
        .bind(enable_all)
        .bind(enable_all)
        .bind(enable_all)
        .bind(enable_all)
        .bind(enable_all)
        .bind(enable_all)
        .bind(enable_all)
        .bind(enable_all)
        .bind(enable_all)
        .fetch_one(&*pool)
        .await
        .map_err(|e| format!("生成环境配置失败: {}", e))?;

        profiles.push(profile);
    }

    Ok(profiles)
}

/// 获取所有自动化脚本
#[tauri::command]
pub async fn get_automation_scripts(pool: State<'_, SqlitePool>) -> Result<Vec<AutomationScript>, String> {
    let scripts = sqlx::query_as::<_, AutomationScript>(
        "SELECT * FROM automation_scripts ORDER BY is_system DESC, id DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| format!("获取脚本列表失败: {}", e))?;

    Ok(scripts)
}

/// 创建自动化脚本
#[tauri::command]
pub async fn create_automation_script(
    pool: State<'_, SqlitePool>,
    request: CreateAutomationScriptRequest,
) -> Result<AutomationScript, String> {
    let required_apis = request.required_apis.map(|apis| serde_json::to_string(&apis).unwrap_or_default());
    let tags = request.tags.map(|t| serde_json::to_string(&t).unwrap_or_default());

    let script = sqlx::query_as::<_, AutomationScript>(
        r#"
        INSERT INTO automation_scripts (name, description, content, required_apis, author, tags)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *
        "#
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.content)
    .bind(&required_apis)
    .bind(&request.author)
    .bind(&tags)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("创建脚本失败: {}", e))?;

    Ok(script)
}

/// 更新自动化脚本
#[tauri::command]
pub async fn update_automation_script(
    pool: State<'_, SqlitePool>,
    request: UpdateAutomationScriptRequest,
) -> Result<AutomationScript, String> {
    let required_apis = request.required_apis.map(|apis| serde_json::to_string(&apis).unwrap_or_default());
    let tags = request.tags.map(|t| serde_json::to_string(&t).unwrap_or_default());

    let script = sqlx::query_as::<_, AutomationScript>(
        r#"
        UPDATE automation_scripts SET
            name = COALESCE(?, name),
            description = COALESCE(?, description),
            content = COALESCE(?, content),
            required_apis = COALESCE(?, required_apis),
            author = COALESCE(?, author),
            tags = COALESCE(?, tags),
            version = version + 1,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ? AND is_system = 0
        RETURNING *
        "#
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.content)
    .bind(&required_apis)
    .bind(&request.author)
    .bind(&tags)
    .bind(request.id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("更新脚本失败: {}", e))?;

    Ok(script)
}

/// 删除自动化脚本
#[tauri::command]
pub async fn delete_automation_script(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM automation_scripts WHERE id = ? AND is_system = 0")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| format!("删除脚本失败: {}", e))?;

    if result.rows_affected() == 0 {
        return Err("无法删除系统脚本或脚本不存在".to_string());
    }

    Ok(())
}

/// 获取所有自动化任务
#[tauri::command]
pub async fn get_automation_tasks(pool: State<'_, SqlitePool>) -> Result<Vec<AutomationTask>, String> {
    let tasks = sqlx::query_as::<_, AutomationTask>(
        "SELECT * FROM automation_tasks ORDER BY id DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| format!("获取任务列表失败: {}", e))?;

    Ok(tasks)
}

/// 创建自动化任务
#[tauri::command]
pub async fn create_automation_task(
    pool: State<'_, SqlitePool>,
    request: CreateAutomationTaskRequest,
) -> Result<AutomationTask, String> {
    let wallet_ids = serde_json::to_string(&request.wallet_ids)
        .map_err(|e| format!("序列化钱包ID失败: {}", e))?;
    let schedule_config = serde_json::to_string(&request.schedule_config)
        .map_err(|e| format!("序列化调度配置失败: {}", e))?;

    let task = sqlx::query_as::<_, AutomationTask>(
        r#"
        INSERT INTO automation_tasks (
            name, description, script_id, wallet_ids, profile_strategy,
            specific_profile_id, schedule_type, schedule_config, concurrency,
            timeout_seconds, retry_times, retry_interval_seconds
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(request.script_id)
    .bind(&wallet_ids)
    .bind(request.profile_strategy.unwrap_or_else(|| "random".to_string()))
    .bind(request.specific_profile_id)
    .bind(request.schedule_type.unwrap_or_else(|| "once".to_string()))
    .bind(&schedule_config)
    .bind(request.concurrency.unwrap_or(1))
    .bind(request.timeout_seconds.unwrap_or(300))
    .bind(request.retry_times.unwrap_or(3))
    .bind(request.retry_interval_seconds.unwrap_or(60))
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("创建任务失败: {}", e))?;

    Ok(task)
}

/// 更新自动化任务
#[tauri::command]
pub async fn update_automation_task(
    pool: State<'_, SqlitePool>,
    request: UpdateAutomationTaskRequest,
) -> Result<AutomationTask, String> {
    let wallet_ids = request.wallet_ids.map(|ids| {
        serde_json::to_string(&ids).unwrap_or_default()
    });
    let schedule_config = request.schedule_config.map(|cfg| {
        serde_json::to_string(&cfg).unwrap_or_default()
    });

    let task = sqlx::query_as::<_, AutomationTask>(
        r#"
        UPDATE automation_tasks SET
            name = COALESCE(?, name),
            description = COALESCE(?, description),
            script_id = COALESCE(?, script_id),
            wallet_ids = COALESCE(?, wallet_ids),
            profile_strategy = COALESCE(?, profile_strategy),
            specific_profile_id = COALESCE(?, specific_profile_id),
            schedule_type = COALESCE(?, schedule_type),
            schedule_config = COALESCE(?, schedule_config),
            concurrency = COALESCE(?, concurrency),
            timeout_seconds = COALESCE(?, timeout_seconds),
            retry_times = COALESCE(?, retry_times),
            retry_interval_seconds = COALESCE(?, retry_interval_seconds),
            status = COALESCE(?, status),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        RETURNING *
        "#
    )
    .bind(&request.name)
    .bind(&request.description)
    .bind(request.script_id)
    .bind(&wallet_ids)
    .bind(&request.profile_strategy)
    .bind(request.specific_profile_id)
    .bind(&request.schedule_type)
    .bind(&schedule_config)
    .bind(request.concurrency)
    .bind(request.timeout_seconds)
    .bind(request.retry_times)
    .bind(request.retry_interval_seconds)
    .bind(&request.status)
    .bind(request.id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("更新任务失败: {}", e))?;

    Ok(task)
}

/// 删除自动化任务
#[tauri::command]
pub async fn delete_automation_task(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM automation_tasks WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| format!("删除任务失败: {}", e))?;

    Ok(())
}

/// 切换任务状态
#[tauri::command]
pub async fn toggle_task_status(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<AutomationTask, String> {
    let task: AutomationTask = sqlx::query_as(
        "SELECT * FROM automation_tasks WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("查询任务失败: {}", e))?;

    let new_status = match task.status.as_str() {
        "enabled" => "paused",
        "paused" => "enabled",
        "draft" => "enabled",
        _ => "enabled",
    };

    let updated = sqlx::query_as::<_, AutomationTask>(
        "UPDATE automation_tasks SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? RETURNING *"
    )
    .bind(new_status)
    .bind(id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("切换任务状态失败: {}", e))?;

    Ok(updated)
}

/// 获取任务执行记录
#[tauri::command]
pub async fn get_task_executions(
    pool: State<'_, SqlitePool>,
    task_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<TaskExecution>, String> {
    let mut sql = "SELECT * FROM task_executions".to_string();
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send + Sync>> = vec![];

    if let Some(tid) = task_id {
        sql.push_str(" WHERE task_id = ?");
        params.push(Box::new(tid));
    }

    sql.push_str(" ORDER BY created_at DESC");

    if let Some(lim) = limit {
        sql.push_str(" LIMIT ?");
        params.push(Box::new(lim));
    }

    let mut query = sqlx::query_as::<_, TaskExecution>(&sql);
    for param in params {
        query = query.bind(param);
    }

    let executions = query
        .fetch_all(&*pool)
        .await
        .map_err(|e| format!("获取执行记录失败: {}", e))?;

    Ok(executions)
}

/// 获取任务统计信息
#[tauri::command]
pub async fn get_task_execution_stats(
    pool: State<'_, SqlitePool>,
    task_id: Option<i64>,
) -> Result<TaskExecutionStats, String> {
    let (total, success, failed): (i64, i64, i64) = if let Some(tid) = task_id {
        sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(CASE WHEN status = 'success' THEN 1 END) as success,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed
            FROM task_executions
            WHERE task_id = ?
            "#
        )
        .bind(tid)
        .fetch_one(&*pool)
        .await
        .map_err(|e| format!("获取统计信息失败: {}", e))?
    } else {
        sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(CASE WHEN status = 'success' THEN 1 END) as success,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed
            FROM task_executions
            "#
        )
        .fetch_one(&*pool)
        .await
        .map_err(|e| format!("获取统计信息失败: {}", e))?
    };

    let success_rate = if total > 0 {
        (success as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(TaskExecutionStats {
        total_executions: total,
        success_count: success,
        failed_count: failed,
        success_rate,
    })
}

/// 初始化浏览器自动化表
#[tauri::command]
pub async fn init_browser_automation_tables(pool: State<'_, SqlitePool>) -> Result<(), String> {
    crate::wallets_tool::airdrop::db::init_airdrop_tables(&*pool)
        .await
        .map_err(|e| format!("初始化表失败: {}", e))?;

    Ok(())
}

/// 验证地址格式
fn is_valid_address(address: &str) -> bool {
    // EVM地址格式: 0x + 40个十六进制字符
    let evm_regex = regex::Regex::new(r"^0x[a-fA-F0-9]{40}$").unwrap();
    // Solana地址格式: 32-44个base58字符
    let solana_regex = regex::Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$").unwrap();

    evm_regex.is_match(address) || solana_regex.is_match(address)
}
