use chrono::{DateTime, Utc};
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs;
use url::Url;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// 代理配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub proxies: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            enabled: false,
            proxies: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// 代理统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStats {
    pub proxy_url: String,
    pub success_count: u32,
    pub failure_count: u32,
    pub avg_latency: f64,
    pub last_used: DateTime<Utc>,
}

/// 代理类型枚举
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ProxyType {
    Http(String),
    Https(String),
    Socks5(String),
}

impl ProxyType {
    pub fn from_url(url: &str) -> Result<Self, String> {
        let parsed_url = Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
        
        match parsed_url.scheme() {
            "http" => Ok(ProxyType::Http(url.to_string())),
            "https" => Ok(ProxyType::Https(url.to_string())),
            "socks5" => Ok(ProxyType::Socks5(url.to_string())),
            scheme => Err(format!("Unsupported proxy scheme: {}", scheme)),
        }
    }
    
    #[allow(dead_code)]
    pub fn url(&self) -> &str {
        match self {
            ProxyType::Http(url) | ProxyType::Https(url) | ProxyType::Socks5(url) => url,
        }
    }
}

/// 代理管理器 - 支持窗口隔离配置
pub struct ProxyManager {
    /// 当前窗口的配置
    current_window_label: Arc<Mutex<Option<String>>>,
    /// 所有窗口的配置缓存
    config_cache: Arc<Mutex<HashMap<String, ProxyConfig>>>,
    /// 所有窗口的客户端池
    client_pools: Arc<Mutex<HashMap<String, HashMap<String, Client>>>>,
    /// 所有窗口的统计信息
    stats: Arc<Mutex<HashMap<String, HashMap<String, ProxyStats>>>>,
}

impl ProxyManager {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            current_window_label: Arc::new(Mutex::new(None)),
            config_cache: Arc::new(Mutex::new(HashMap::new())),
            client_pools: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// 获取指定窗口的配置文件路径
    fn get_config_path_for_window(window_label: &str) -> Result<PathBuf, String> {
        let app_data_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?
            .join("WalletsTool");
        
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
        
        // 使用窗口ID生成配置文件名，将特殊字符替换为下划线
        let safe_label = window_label
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>();
        
        Ok(app_data_dir.join(format!("proxy_config_{}.json", safe_label)))
    }
    
    /// 设置当前窗口标签
    pub fn set_window_label(&self, window_label: String) {
        let mut current_label = self.current_window_label.lock().unwrap();
        *current_label = Some(window_label);
        println!("[DEBUG] ProxyManager - 设置当前窗口ID: {:?}", current_label);
    }
    
    /// 获取当前窗口标签
    pub fn get_current_window_label(&self) -> String {
        let current_label = self.current_window_label.lock().unwrap();
        match &*current_label {
            Some(label) => label.clone(),
            None => "default".to_string(),
        }
    }
    
    /// 加载指定窗口的配置
    pub async fn load_config_for_window(&self, window_label: &str) -> Result<ProxyConfig, String> {
        let config_path = Self::get_config_path_for_window(window_label)?;
        let config_path_clone = config_path.clone();
        
        println!("[DEBUG] load_config_for_window - 加载窗口 {} 的配置", window_label);
        
        if !config_path.exists() {
            println!("[DEBUG] load_config_for_window - 窗口 {} 的配置文件不存在，使用默认配置", window_label);
            let default_config = ProxyConfig::default();
            
            // 释放锁后再进行其他操作
            let mut config_cache = self.config_cache.lock().unwrap();
            config_cache.insert(window_label.to_string(), default_config.clone());
            return Ok(default_config);
        }
        
        // 读取配置文件内容
        let content = fs::read_to_string(&config_path_clone)
            .await
            .map_err(|e| format!("Failed to read config file for window {}: {}", window_label, e))?;
        
        let config: ProxyConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config for window {}: {}", window_label, e))?;
        
        println!("[DEBUG] load_config_for_window - 窗口 {} 的配置加载成功，代理启用: {}, 代理数量: {}", 
                 window_label, config.enabled, config.proxies.len());
        
        {
            let mut config_cache = self.config_cache.lock().unwrap();
            config_cache.insert(window_label.to_string(), config.clone());
        }
        
        // 为该窗口重建客户端池（释放锁后）
        self.rebuild_client_pool_for_window(window_label, &config).await?;
        
        Ok(config)
    }
    
    /// 加载当前窗口的配置
    pub async fn load_current_window_config(&self) -> Result<ProxyConfig, String> {
        let window_label = self.get_current_window_label();
        self.load_config_for_window(&window_label).await
    }
    
    /// 保存指定窗口的配置
    pub async fn save_config_for_window(&self, window_label: &str, config: &ProxyConfig) -> Result<(), String> {
        let config_path = Self::get_config_path_for_window(window_label)?;
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&config_path, content)
            .await
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        println!("[DEBUG] save_config_for_window - 窗口 {} 的配置已保存", window_label);
        
        Ok(())
    }
    
    /// 更新当前窗口的代理配置
    pub async fn update_config(&self, proxies: Vec<String>, enabled: bool) -> Result<(), String> {
        let window_label = self.get_current_window_label();
        self.update_config_for_window(&window_label, proxies, enabled).await
    }
    
    /// 更新指定窗口的代理配置
    pub async fn update_config_for_window(&self, window_label: &str, proxies: Vec<String>, enabled: bool) -> Result<(), String> {
        // 先获取配置，克隆后释放锁
        let config = {
            let config_cache = self.config_cache.lock().unwrap();
            config_cache.get(window_label)
                .cloned()
                .unwrap_or_else(ProxyConfig::default)
        };
        
        // 创建新配置
        let mut new_config = config;
        new_config.proxies = proxies;
        new_config.enabled = enabled;
        new_config.updated_at = Utc::now();
        
        // 插入到缓存中（释放锁后）
        {
            let mut config_cache = self.config_cache.lock().unwrap();
            config_cache.insert(window_label.to_string(), new_config.clone());
        }
        
        self.save_config_for_window(window_label, &new_config).await?;
        self.rebuild_client_pool_for_window(window_label, &new_config).await?;
        
        Ok(())
    }
    
    /// 获取当前窗口的配置
    pub fn get_config(&self) -> ProxyConfig {
        let window_label = self.get_current_window_label();
        self.get_config_for_window(&window_label)
    }
    
    /// 获取指定窗口的配置
    pub fn get_config_for_window(&self, window_label: &str) -> ProxyConfig {
        let config_cache = self.config_cache.lock().unwrap();
        match config_cache.get(window_label) {
            Some(config) => config.clone(),
            None => ProxyConfig::default(),
        }
    }
    
    /// 重建指定窗口的客户端池
    async fn rebuild_client_pool_for_window(&self, window_label: &str, config: &ProxyConfig) -> Result<(), String> {
        let mut client_pools = self.client_pools.lock().unwrap();
        let window_client_pool = client_pools.entry(window_label.to_string()).or_insert_with(HashMap::new);
        
        println!("[DEBUG] rebuild_client_pool_for_window - 窗口 {}, 代理启用: {}, 代理数量: {}", 
                 window_label, config.enabled, config.proxies.len());
        
        window_client_pool.clear();
        
        if !config.enabled {
            println!("[DEBUG] rebuild_client_pool_for_window - 窗口 {} 代理未启用，清空客户端池", window_label);
            return Ok(());
        }
        
        let mut success_count = 0;
        let mut fail_count = 0;
        
        for proxy_url in &config.proxies {
            match self.create_proxy_client(proxy_url) {
                Ok(client) => {
                    window_client_pool.insert(proxy_url.clone(), client);
                    success_count += 1;
                    println!("[DEBUG] rebuild_client_pool_for_window - 窗口 {} 成功创建代理客户端: {}", window_label, proxy_url);
                }
                Err(e) => {
                    fail_count += 1;
                    println!("[ERROR] rebuild_client_pool_for_window - 窗口 {} 创建代理客户端失败: {} - {}", window_label, proxy_url, e);
                }
            }
        }
        
        println!("[DEBUG] rebuild_client_pool_for_window - 窗口 {} 完成，成功: {}, 失败: {}, 客户端池大小: {}", 
                 window_label, success_count, fail_count, window_client_pool.len());
        
        Ok(())
    }
    
    /// 创建代理客户端
    fn create_proxy_client(&self, proxy_url: &str) -> Result<Client, String> {
        let proxy_type = ProxyType::from_url(proxy_url)?;
        
        let proxy = match proxy_type {
            ProxyType::Http(_) | ProxyType::Https(_) => {
                Proxy::all(proxy_url).map_err(|e| format!("Failed to create HTTP/HTTPS proxy: {}", e))?
            }
            ProxyType::Socks5(_) => {
                Proxy::all(proxy_url).map_err(|e| format!("Failed to create SOCKS5 proxy: {}", e))?
            }
        };
        
        Client::builder()
            .proxy(proxy)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to build proxy client: {}", e))
    }
    
    /// 获取随机代理客户端
    pub fn get_random_proxy_client(&self) -> Option<Client> {
        let window_label = self.get_current_window_label();
        let config = self.get_config_for_window(&window_label);
        
        println!("[DEBUG] get_random_proxy_client - 窗口: {}, 代理启用状态: {}, 代理数量: {}", 
                 window_label, config.enabled, config.proxies.len());
        
        if !config.enabled || config.proxies.is_empty() {
            println!("[DEBUG] get_random_proxy_client - 代理未启用或无代理地址");
            return None;
        }
        
        let client_pools = self.client_pools.lock().unwrap();
        let window_client_pool = match client_pools.get(&window_label) {
            Some(pool) => pool,
            None => {
                println!("[DEBUG] get_random_proxy_client - 窗口 {} 没有客户端池", window_label);
                return None;
            }
        };
        
        println!("[DEBUG] get_random_proxy_client - 窗口 {} 客户端池大小: {}", window_label, window_client_pool.len());
        
        let available_proxies: Vec<_> = config.proxies.iter()
            .filter(|proxy| window_client_pool.contains_key(*proxy))
            .collect();
        
        println!("[DEBUG] get_random_proxy_client - 可用代理数量: {}", available_proxies.len());
        
        if available_proxies.is_empty() {
            println!("[WARN] get_random_proxy_client - 窗口 {} 客户端池为空！代理配置: {:?}", window_label, config.proxies);
            return None;
        }
        
        let mut rng = thread_rng();
        let selected_proxy = available_proxies.choose(&mut rng)?;
        
        println!("[DEBUG] get_random_proxy_client - 窗口 {} 选中代理: {}", window_label, selected_proxy);
        
        window_client_pool.get(*selected_proxy).cloned()
    }
    
    pub fn get_random_proxy_client_for_window(&self, window_id: &str) -> Option<Client> {
        let config = self.get_config_for_window(window_id);
        
        println!("[DEBUG] get_random_proxy_client_for_window - 窗口: {}, 代理启用状态: {}, 代理数量: {}", 
                 window_id, config.enabled, config.proxies.len());
        
        if !config.enabled || config.proxies.is_empty() {
            println!("[DEBUG] get_random_proxy_client_for_window - 代理未启用或无代理地址");
            return None;
        }
        
        let client_pools = self.client_pools.lock().unwrap();
        let window_client_pool = match client_pools.get(window_id) {
            Some(pool) => pool,
            None => {
                println!("[DEBUG] get_random_proxy_client_for_window - 窗口 {} 没有客户端池", window_id);
                return None;
            }
        };
        
        println!("[DEBUG] get_random_proxy_client_for_window - 窗口 {} 客户端池大小: {}", window_id, window_client_pool.len());
        
        let available_proxies: Vec<_> = config.proxies.iter()
            .filter(|proxy| window_client_pool.contains_key(*proxy))
            .collect();
        
        println!("[DEBUG] get_random_proxy_client_for_window - 可用代理数量: {}", available_proxies.len());
        
        if available_proxies.is_empty() {
            println!("[WARN] get_random_proxy_client_for_window - 窗口 {} 客户端池为空！代理配置: {:?}", window_id, config.proxies);
            return None;
        }
        
        let mut rng = thread_rng();
        let selected_proxy = available_proxies.choose(&mut rng)?;
        
        println!("[DEBUG] get_random_proxy_client_for_window - 窗口 {} 选中代理: {}", window_id, selected_proxy);
        
        window_client_pool.get(*selected_proxy).cloned()
    }
    
    /// 获取随机代理URL
    pub fn get_random_proxy(&self) -> Option<String> {
        let window_label = self.get_current_window_label();
        let config = self.get_config_for_window(&window_label);
        
        if !config.enabled || config.proxies.is_empty() {
            return None;
        }
        
        let mut rng = thread_rng();
        config.proxies.choose(&mut rng).cloned()
    }
    
    /// 获取指定窗口的随机代理URL
    pub fn get_random_proxy_for_window(&self, window_id: &str) -> Option<String> {
        let config = self.get_config_for_window(window_id);
        
        if !config.enabled || config.proxies.is_empty() {
            return None;
        }
        
        let mut rng = thread_rng();
        config.proxies.choose(&mut rng).cloned()
    }
    
    /// 测试代理连接
    pub async fn test_proxy(&self, proxy_url: &str) -> Result<(bool, f64), String> {
        let start_time = std::time::Instant::now();
        
        let client = self.create_proxy_client(proxy_url)?;
        
        // 使用多个测试URL，提高成功率
        let test_urls = vec![
            "http://www.baidu.com",           // 国内网站
            "http://www.google.com",          // 国外网站
            "https://httpbin.org/ip",         // HTTP测试服务
        ];
        
        // 尝试所有测试URL，只要有一个成功就认为代理可用
        let mut last_error = String::new();
        
        for test_url in test_urls {
            match client.get(test_url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await 
            {
                Ok(response) => {
                    let latency = start_time.elapsed().as_millis() as f64;
                    if response.status().is_success() {
                        return Ok((true, latency));
                    } else {
                        last_error = format!("HTTP error from {}: {}", test_url, response.status());
                    }
                }
                Err(e) => {
                    last_error = format!("Request to {} failed: {}", test_url, e);
                }
            }
        }
        
        // 所有测试URL都失败
        Err(last_error)
    }
    
    /// 更新代理统计
    #[allow(dead_code)]
    pub fn update_proxy_stats(&self, proxy_url: &str, success: bool, latency: f64) {
        let window_label = self.get_current_window_label();
        let mut all_stats = self.stats.lock().unwrap();
        let window_stats = all_stats.entry(window_label.clone()).or_insert_with(HashMap::new);
        
        let proxy_stats = window_stats.entry(proxy_url.to_string()).or_insert_with(|| ProxyStats {
            proxy_url: proxy_url.to_string(),
            success_count: 0,
            failure_count: 0,
            avg_latency: 0.0,
            last_used: Utc::now(),
        });
        
        if success {
            proxy_stats.success_count += 1;
            let total_requests = proxy_stats.success_count + proxy_stats.failure_count;
            proxy_stats.avg_latency = (proxy_stats.avg_latency * (total_requests - 1) as f64 + latency) / total_requests as f64;
        } else {
            proxy_stats.failure_count += 1;
        }
        
        proxy_stats.last_used = Utc::now();
    }
    
    /// 获取代理统计信息
    pub fn get_proxy_stats(&self) -> HashMap<String, ProxyStats> {
        let window_label = self.get_current_window_label();
        let all_stats = self.stats.lock().unwrap();
        all_stats.get(&window_label).cloned().unwrap_or_else(HashMap::new)
    }
    
    /// 清除指定窗口的代理配置（文件、内存缓存、客户端池、统计）
    pub async fn clear_config_for_window(&self, window_label: &str) -> Result<(), String> {
        let config_path = Self::get_config_path_for_window(window_label)?;
        
        if config_path.exists() {
            fs::remove_file(&config_path)
                .await
                .map_err(|e| format!("Failed to remove config file: {}", e))?;
            println!("[DEBUG] clear_config_for_window - 已删除配置文件: {:?}", config_path);
        }
        
        {
            let mut config_cache = self.config_cache.lock().unwrap();
            config_cache.remove(window_label);
            println!("[DEBUG] clear_config_for_window - 已清除内存配置缓存: {}", window_label);
        }
        
        {
            let mut client_pools = self.client_pools.lock().unwrap();
            client_pools.remove(window_label);
            println!("[DEBUG] clear_config_for_window - 已清除客户端池: {}", window_label);
        }
        
        {
            let mut stats = self.stats.lock().unwrap();
            stats.remove(window_label);
            println!("[DEBUG] clear_config_for_window - 已清除统计信息: {}", window_label);
        }
        
        Ok(())
    }
}

lazy_static::lazy_static! {
    /// 全局代理管理器实例
    pub static ref PROXY_MANAGER: ProxyManager = ProxyManager::new().expect("Failed to create proxy manager");
}