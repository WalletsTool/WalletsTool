use tauri::command;
use serde::{Serialize, Deserialize};
use crate::wallets_tool::ecosystems::ethereum::proxy_manager::{PROXY_MANAGER, ProxyConfig};

/// 代理测试结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyTestResult {
    pub success: bool,
    pub latency: f64,
    pub error: Option<String>,
}

/// Tauri命令：设置当前窗口ID
#[command]
pub async fn set_proxy_window_id(window_id: String) -> Result<String, String> {
    PROXY_MANAGER.set_window_label(window_id);
    
    Ok("窗口ID设置成功".to_string())
}

/// Tauri命令：保存代理配置（当前窗口）
#[command]
pub async fn save_proxy_config(proxies: Vec<String>, enabled: bool) -> Result<String, String> {
    PROXY_MANAGER.update_config(proxies, enabled).await?;
    Ok("代理配置保存成功".to_string())
}

/// Tauri命令：保存指定窗口的代理配置
#[command]
pub async fn save_proxy_config_for_window(window_id: String, proxies: Vec<String>, enabled: bool) -> Result<String, String> {
    PROXY_MANAGER.update_config_for_window(&window_id, proxies, enabled).await?;
    Ok(format!("窗口 {} 的代理配置保存成功", window_id))
}

/// Tauri命令：获取代理配置（当前窗口）
#[command]
pub async fn get_proxy_config() -> Result<ProxyConfig, String> {
    // 确保当前窗口的配置已加载
    if let Err(e) = PROXY_MANAGER.load_current_window_config().await {
        eprintln!("加载代理配置失败: {}", e);
    }
    
    Ok(PROXY_MANAGER.get_config())
}

/// Tauri命令：获取指定窗口的代理配置
#[command]
pub async fn get_proxy_config_for_window(window_id: String) -> Result<ProxyConfig, String> {
    // 确保指定窗口的配置已加载
    if let Err(e) = PROXY_MANAGER.load_config_for_window(&window_id).await {
        eprintln!("加载代理配置失败: {}", e);
    }
    
    Ok(PROXY_MANAGER.get_config_for_window(&window_id))
}

/// Tauri命令：测试代理连接
#[command]
pub async fn test_proxy_connection(proxy_url: String) -> Result<ProxyTestResult, String> {
    match PROXY_MANAGER.test_proxy(&proxy_url).await {
        Ok((success, latency)) => Ok(ProxyTestResult {
            success,
            latency,
            error: None,
        }),
        Err(error) => Ok(ProxyTestResult {
            success: false,
            latency: 0.0,
            error: Some(error),
        }),
    }
}

/// Tauri命令：获取代理统计信息（当前窗口）
#[command]
pub async fn get_proxy_stats() -> Result<std::collections::HashMap<String, crate::wallets_tool::ecosystems::ethereum::proxy_manager::ProxyStats>, String> {
    Ok(PROXY_MANAGER.get_proxy_stats())
}

/// Tauri命令：获取指定窗口的代理统计信息
#[command]
pub async fn get_proxy_stats_for_window(_window_id: String) -> Result<std::collections::HashMap<String, crate::wallets_tool::ecosystems::ethereum::proxy_manager::ProxyStats>, String> {
    let all_stats = PROXY_MANAGER.get_proxy_stats();
    // 对于单个窗口的统计，我们返回当前窗口的统计信息
    Ok(all_stats)
}

/// Tauri命令：清除指定窗口的代理配置
#[command]
pub async fn clear_proxy_config_for_window(window_id: String) -> Result<String, String> {
    PROXY_MANAGER.clear_config_for_window(&window_id).await?;
    Ok(format!("窗口 {} 的代理配置已完全清除", window_id))
}