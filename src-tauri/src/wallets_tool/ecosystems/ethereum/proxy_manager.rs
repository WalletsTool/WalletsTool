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

/// 代理管理器
pub struct ProxyManager {
    config: Arc<Mutex<ProxyConfig>>,
    stats: Arc<Mutex<HashMap<String, ProxyStats>>>,
    client_pool: Arc<Mutex<HashMap<String, Client>>>,
    config_path: PathBuf,
}

impl ProxyManager {
    pub fn new() -> Result<Self, String> {
        let config_path = Self::get_config_path()?;
        
        Ok(Self {
            config: Arc::new(Mutex::new(ProxyConfig::default())),
            stats: Arc::new(Mutex::new(HashMap::new())),
            client_pool: Arc::new(Mutex::new(HashMap::new())),
            config_path,
        })
    }
    
    /// 获取配置文件路径
    fn get_config_path() -> Result<PathBuf, String> {
        let app_data_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?
            .join("WalletsTool");
        
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
        
        Ok(app_data_dir.join("proxy_config.json"))
    }
    
    /// 加载配置
    pub async fn load_config(&self) -> Result<(), String> {
        println!("[DEBUG] load_config - 开始加载代理配置");
        
        if !self.config_path.exists() {
            println!("[DEBUG] load_config - 配置文件不存在，创建默认配置");
            return self.save_config().await;
        }
        
        let content = fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let config: ProxyConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        
        println!("[DEBUG] load_config - 配置加载成功，代理启用: {}, 代理数量: {}", 
                 config.enabled, config.proxies.len());
        
        *self.config.lock().unwrap() = config;
        
        // 重要：加载配置后需要重建客户端池
        self.rebuild_client_pool().await?;
        
        Ok(())
    }
    
    /// 保存配置
    pub async fn save_config(&self) -> Result<(), String> {
        let config = self.config.lock().unwrap().clone();
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&self.config_path, content)
            .await
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        Ok(())
    }
    
    /// 更新代理配置
    pub async fn update_config(&self, proxies: Vec<String>, enabled: bool) -> Result<(), String> {
        {
            let mut config = self.config.lock().unwrap();
            config.proxies = proxies;
            config.enabled = enabled;
            config.updated_at = Utc::now();
        }
        
        self.save_config().await?;
        self.rebuild_client_pool().await?;
        Ok(())
    }
    
    /// 获取当前配置
    pub fn get_config(&self) -> ProxyConfig {
        self.config.lock().unwrap().clone()
    }
    
    /// 重建客户端池
    async fn rebuild_client_pool(&self) -> Result<(), String> {
        let config = self.config.lock().unwrap().clone();
        let mut client_pool = self.client_pool.lock().unwrap();
        
        println!("[DEBUG] rebuild_client_pool - 开始重建客户端池");
        println!("[DEBUG] rebuild_client_pool - 代理启用: {}, 代理数量: {}", 
                 config.enabled, config.proxies.len());
        
        client_pool.clear();
        
        if !config.enabled {
            println!("[DEBUG] rebuild_client_pool - 代理未启用，清空客户端池");
            return Ok(());
        }
        
        let mut success_count = 0;
        let mut fail_count = 0;
        
        for proxy_url in &config.proxies {
            match self.create_proxy_client(proxy_url) {
                Ok(client) => {
                    client_pool.insert(proxy_url.clone(), client);
                    success_count += 1;
                    println!("[DEBUG] rebuild_client_pool - 成功创建代理客户端: {}", proxy_url);
                }
                Err(e) => {
                    fail_count += 1;
                    println!("[ERROR] rebuild_client_pool - 创建代理客户端失败: {} - {}", proxy_url, e);
                }
            }
        }
        
        println!("[DEBUG] rebuild_client_pool - 完成，成功: {}, 失败: {}, 客户端池大小: {}", 
                 success_count, fail_count, client_pool.len());
        
        Ok(())
    }
    
    /// 创建代理客户端
    fn create_proxy_client(&self, proxy_url: &str) -> Result<Client, String> {
        let proxy_type = ProxyType::from_url(proxy_url)?;
        
        let proxy = match proxy_type {
            ProxyType::Http(_) | ProxyType::Https(_) => {
                Proxy::http(proxy_url).map_err(|e| format!("Failed to create HTTP proxy: {}", e))?
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
        let config = self.config.lock().unwrap();
        
        println!("[DEBUG] get_random_proxy_client - 代理启用状态: {}, 代理数量: {}", 
                 config.enabled, config.proxies.len());
        
        if !config.enabled || config.proxies.is_empty() {
            println!("[DEBUG] get_random_proxy_client - 代理未启用或无代理地址");
            return None;
        }
        
        let client_pool = self.client_pool.lock().unwrap();
        println!("[DEBUG] get_random_proxy_client - 客户端池大小: {}", client_pool.len());
        
        let available_proxies: Vec<_> = config.proxies.iter()
            .filter(|proxy| client_pool.contains_key(*proxy))
            .collect();
        
        println!("[DEBUG] get_random_proxy_client - 可用代理数量: {}", available_proxies.len());
        
        if available_proxies.is_empty() {
            println!("[WARN] get_random_proxy_client - 客户端池为空！代理配置: {:?}", config.proxies);
            return None;
        }
        
        let mut rng = thread_rng();
        let selected_proxy = available_proxies.choose(&mut rng)?;
        
        println!("[DEBUG] get_random_proxy_client - 选中代理: {}", selected_proxy);
        
        client_pool.get(*selected_proxy).cloned()
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
        let mut stats = self.stats.lock().unwrap();
        
        let proxy_stats = stats.entry(proxy_url.to_string()).or_insert_with(|| ProxyStats {
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
        self.stats.lock().unwrap().clone()
    }
}

lazy_static::lazy_static! {
    /// 全局代理管理器实例
    pub static ref PROXY_MANAGER: ProxyManager = ProxyManager::new().expect("Failed to create proxy manager");
}