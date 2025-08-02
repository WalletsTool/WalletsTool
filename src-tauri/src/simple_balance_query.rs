use anyhow::{anyhow, Result};
use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};
use tauri::Emitter;
use rand;
use crate::database::{get_database_manager, rpc_service::RpcService};

// 基于窗口ID的停止标志映射
static STOP_FLAGS: LazyLock<Mutex<HashMap<String, AtomicBool>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

// 辅助函数：获取窗口的停止状态
fn get_stop_flag(window_id: &str) -> bool {
    let flags = STOP_FLAGS.lock().unwrap();
    if let Some(flag) = flags.get(window_id) {
        flag.load(Ordering::Relaxed)
    } else {
        false
    }
}

// 辅助函数：设置窗口的停止状态
fn set_stop_flag(window_id: &str, value: bool) {
    let mut flags = STOP_FLAGS.lock().unwrap();
    if let Some(flag) = flags.get(window_id) {
        flag.store(value, Ordering::Relaxed);
    } else {
        flags.insert(window_id.to_string(), AtomicBool::new(value));
    }
}

// 辅助函数：重置窗口的停止状态
fn reset_stop_flag(window_id: &str) {
    set_stop_flag(window_id, false);
}

// 查询项目结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryItem {
    pub address: String,
    pub private_key: Option<String>,
    pub plat_balance: Option<String>,
    pub coin_balance: Option<String>,
    pub nonce: Option<u64>,
    pub exec_status: String, // "0"=未执行, "1"=执行中, "2"=成功, "3"=失败
    pub error_msg: Option<String>,
}

// 币种配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinConfig {
    pub coin_type: String, // "base" 或 "token"
    pub contract_address: Option<String>,
    pub abi: Option<String>,
}

// 查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    pub chain: String,
    pub coin_config: CoinConfig,
    pub items: Vec<QueryItem>,
    pub only_coin_config: bool,
    pub thread_count: usize,
}

// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub success: bool,
    pub items: Vec<QueryItem>,
    pub error_msg: Option<String>,
}

// RPC 请求结构
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: i32,
}

// RPC 响应结构
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    #[allow(dead_code)]
    id: i32,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

// RPC 提供商配置
#[allow(dead_code)]
pub struct ProviderConfig {
    pub rpc_urls: Vec<String>,
    pub chain_id: u64,
}

// 余额查询器
pub struct SimpleBalanceQueryService {
    client: Client,
}

impl SimpleBalanceQueryService {
    pub fn new() -> Self {
        let client = Client::new();
        Self { client }
    }

    // 从数据库获取RPC URL
    async fn get_rpc_url(&self, chain: &str) -> Result<String> {
        let db_manager = get_database_manager();
        let rpc_service = RpcService::new(db_manager.get_pool());
        
        rpc_service.get_random_rpc_url(chain).await
    }

    // 发送 JSON-RPC 请求（带超时）
    async fn send_rpc_request(&self, rpc_url: &str, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };

        // 设置10秒超时
        let timeout = Duration::from_secs(10);
        let response = tokio::time::timeout(timeout, 
            self.client
                .post(rpc_url)
                .json(&request)
                .send()
        ).await
        .map_err(|_| anyhow!("RPC请求超时（10秒），RPC地址: {}", rpc_url))??
        ;

        let json_response: JsonRpcResponse = tokio::time::timeout(timeout,
            response.json::<JsonRpcResponse>()
        ).await
        .map_err(|_| anyhow!("RPC响应解析超时（10秒），RPC地址: {}", rpc_url))??
        ;

        if let Some(error) = json_response.error {
            return Err(anyhow!("RPC Error: {} - {}", error.code, error.message));
        }

        json_response.result.ok_or_else(|| anyhow!("No result in RPC response"))
    }

    // 查询基础币种余额
    async fn query_base_balance(&self, item: &mut QueryItem, chain: &str) -> Result<()> {
        let rpc_url = self.get_rpc_url(chain).await?;
        
        
        // 查询余额
        let balance_result = self.send_rpc_request(
            &rpc_url,
            "eth_getBalance",
            serde_json::json!([item.address, "latest"])
        ).await?;

        if let Some(balance_hex) = balance_result.as_str() {
            
            // 检查十六进制字符串长度
            let hex_without_prefix = &balance_hex[2..];
            
            // 将十六进制转换为十进制，使用u128避免溢出
            match u128::from_str_radix(hex_without_prefix, 16) {
                Ok(balance_wei) => {
                    let balance_eth = balance_wei as f64 / 1e18;
                    item.plat_balance = Some(format!("{:.6}", balance_eth));
                }
                Err(e) => {
                    println!("[ERROR] 十六进制转换失败 - 链: {}, 地址: {}, 十六进制: {}, 错误: {}", 
                            chain, item.address, balance_hex, e);
                    return Err(anyhow!("余额数值转换失败: {} (原始值: {})", e, balance_hex));
                }
            }
        } else {
            println!("[WARNING] 余额查询返回空值 - 链: {}, 地址: {}", chain, item.address);
        }

        // 查询 nonce
        let nonce_result = self.send_rpc_request(
            &rpc_url,
            "eth_getTransactionCount",
            serde_json::json!([item.address, "latest"])
        ).await?;

        if let Some(nonce_hex) = nonce_result.as_str() {
            let nonce_without_prefix = &nonce_hex[2..];
            
            match u64::from_str_radix(nonce_without_prefix, 16) {
                Ok(nonce) => {
                    item.nonce = Some(nonce);
                }
                Err(e) => {
                    println!("[ERROR] nonce转换失败 - 链: {}, 地址: {}, 十六进制: {}, 错误: {}", 
                            chain, item.address, nonce_hex, e);
                    return Err(anyhow!("nonce数值转换失败: {} (原始值: {})", e, nonce_hex));
                }
            }
        } else {
            println!("[WARNING] nonce查询返回空值 - 链: {}, 地址: {}", chain, item.address);
        }

        Ok(())
    }

    // 查询代币余额
    async fn query_token_balance(&self, item: &mut QueryItem, chain: &str, contract_address: &str) -> Result<()> {
        let rpc_url = self.get_rpc_url(chain).await?;
        
        // ERC20 balanceOf 函数的方法ID
        let balance_of_method = "70a08231";
        
        // 编码地址参数（去掉0x前缀，左填充到32字节）
        let address_param = format!("{:0>64}", &item.address[2..]);
        let data = format!("0x{}{}", balance_of_method, address_param);

        let balance_result = self.send_rpc_request(
            &rpc_url,
            "eth_call",
            serde_json::json!([{
                "to": contract_address,
                "data": data
            }, "latest"])
        ).await?;

        if let Some(balance_hex) = balance_result.as_str() {
            
            // 检查十六进制字符串长度
            let hex_without_prefix = &balance_hex[2..];
            
            // 将十六进制转换为十进制，使用u128避免溢出
            match u128::from_str_radix(hex_without_prefix, 16) {
                Ok(balance_wei) => {
                    let balance_tokens = balance_wei as f64 / 1e18; // 假设18位小数
                    item.coin_balance = Some(format!("{:.4}", balance_tokens));
                }
                Err(e) => {
                    println!("[ERROR] 代币余额十六进制转换失败 - 链: {}, 地址: {}, 合约: {}, 十六进制: {}, 错误: {}", 
                            chain, item.address, contract_address, balance_hex, e);
                    return Err(anyhow!("代币余额数值转换失败: {} (原始值: {})", e, balance_hex));
                }
            }
        } else {
            println!("[WARNING] 代币余额查询返回空值 - 链: {}, 地址: {}, 合约: {}", 
                    chain, item.address, contract_address);
        }

        Ok(())
    }

    // 查询单个项目的余额（带超时控制）
    async fn query_single_item(&self, mut item: QueryItem, params: &QueryParams, window_id: &str) -> QueryItem {
        // 检查是否需要停止查询
        if get_stop_flag(window_id) {
            item.exec_status = "3".to_string(); // 失败
            item.error_msg = Some("查询已被用户停止".to_string());
            return item;
        }

        item.exec_status = "1".to_string(); // 执行中
        item.error_msg = None;

        // 设置单个查询任务的超时时间为15秒
        let query_timeout = Duration::from_secs(15);
        
        // 获取当前使用的RPC地址用于错误信息
        let rpc_url = if let Ok(url) = self.get_rpc_url(&params.chain).await {
            url
        } else {
            "未知RPC地址".to_string()
        };
        
        let result = tokio::time::timeout(query_timeout, async {
            if params.coin_config.coin_type == "base" {
                self.query_base_balance(&mut item, &params.chain).await?;
            } else if params.coin_config.coin_type == "token" {
                // 如果不是仅查询目标代币，也要查询平台币
                if !params.only_coin_config {
                    if let Err(e) = self.query_base_balance(&mut item, &params.chain).await {
                        println!("查询平台币失败: {}", e);
                    }
                }
                
                if let Some(contract_address) = &params.coin_config.contract_address {
                    self.query_token_balance(&mut item, &params.chain, contract_address).await?;
                }
            }
            Ok::<(), anyhow::Error>(())
        }).await;

        match result {
            Ok(Ok(_)) => {
                item.exec_status = "2".to_string(); // 成功
            }
            Ok(Err(e)) => {
                item.exec_status = "3".to_string(); // 失败
                item.error_msg = Some(format!("查询失败: {}", e));
                println!("查询失败: {}", e);
            }
            Err(_) => {
                item.exec_status = "3".to_string(); // 超时
                item.error_msg = Some(format!("查询超时（15秒），RPC地址: {}", rpc_url));
                println!("查询超时: {}, RPC地址: {}", item.address, rpc_url);
            }
        }

        item
    }

    // 批量查询余额（多线程）
    pub async fn query_balances(&self, params: QueryParams) -> QueryResult {
        let thread_count = params.thread_count.max(1).min(99); // 限制线程数在1-99之间
        let semaphore = Arc::new(Semaphore::new(thread_count));
        
        println!("开始批量查询余额，线程数: {}, 总任务数: {}", thread_count, params.items.len());
        
        let items = params.items.clone();
        let tasks: Vec<_> = items.into_iter().map(|item| {
            let semaphore = semaphore.clone();
            let params = params.clone();
            let service = self;
            
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                // 添加随机延迟，避免过快请求
                let delay = Duration::from_millis(rand::random::<u64>() % 100);
                sleep(delay).await;
                
                service.query_single_item(item, &params, "default").await
            }
        }).collect();

        let results = join_all(tasks).await;
        
        let success = results.iter().all(|item| item.exec_status == "2");
        let error_msg = if success {
            None
        } else {
            Some("部分查询失败".to_string())
        };

        println!("查询完成，成功: {}", success);
        
        QueryResult {
            success,
            items: results,
            error_msg,
        }
    }

    // 批量查询余额（带实时更新）
    pub async fn query_balances_with_updates(
        &self, 
        params: QueryParams, 
        app_handle: tauri::AppHandle,
        window_id: String
    ) -> QueryResult {
        // 重置停止标志
        reset_stop_flag(&window_id);
        
        let thread_count = params.thread_count.max(1).min(99); // 限制线程数在1-99之间
        let semaphore = Arc::new(Semaphore::new(thread_count));
        
        println!("开始批量查询余额（实时更新），线程数: {}, 总任务数: {}", thread_count, params.items.len());
        
        let items = params.items.clone();
        let tasks: Vec<_> = items.into_iter().enumerate().map(|(index, item)| {
            let semaphore = semaphore.clone();
            let params = params.clone();
            let service = self;
            let app_handle = app_handle.clone();
            let window_id = window_id.clone();
            
            async move {
                // 在获取信号量前检查停止标志
                if get_stop_flag(&window_id) {
                    let mut stopped_item = item.clone();
                    stopped_item.exec_status = "3".to_string();
                    stopped_item.error_msg = Some("查询已被用户停止".to_string());
                    return stopped_item;
                }
                
                let _permit = semaphore.acquire().await.unwrap();
                
                // 获取信号量后再次检查停止标志
                if get_stop_flag(&window_id) {
                    let mut stopped_item = item.clone();
                    stopped_item.exec_status = "3".to_string();
                    stopped_item.error_msg = Some("查询已被用户停止".to_string());
                    return stopped_item;
                }
                
                // 添加随机延迟，避免过快请求
                let delay = Duration::from_millis(rand::random::<u64>() % 100);
                sleep(delay).await;
                
                // 通知前端该项目开始执行
                let mut updating_item = item.clone();
                updating_item.exec_status = "1".to_string();
                if let Err(e) = app_handle.emit("balance_item_update", serde_json::json!({
                    "index": index,
                    "item": updating_item,
                    "window_id": window_id
                })) {
                    println!("发送开始执行事件失败: {}", e);
                }
                
                let result = service.query_single_item(item, &params, &window_id).await;
                
                // 通知前端该项目查询完成
                if let Err(e) = app_handle.emit("balance_item_update", serde_json::json!({
                    "index": index,
                    "item": result.clone(),
                    "window_id": window_id
                })) {
                    println!("发送查询完成事件失败: {}", e);
                }
                
                result
            }
        }).collect();

        let results = join_all(tasks).await;
        
        let success = results.iter().all(|item| item.exec_status == "2");
        let error_msg = if success {
            None
        } else {
            Some("部分查询失败".to_string())
        };

        println!("查询完成，成功: {}", success);
        
        QueryResult {
            success,
            items: results,
            error_msg,
        }
    }
}

// Tauri 命令：查询余额
#[tauri::command]
pub async fn query_balances_simple(params: QueryParams) -> Result<QueryResult, String> {
    let service = SimpleBalanceQueryService::new();
    
    let result = service.query_balances(params).await;
    Ok(result)
}

// 带有实时更新的查询余额命令
#[tauri::command]
pub async fn query_balances_with_updates(
    params: QueryParams,
    app_handle: tauri::AppHandle,
    window_id: String,
) -> Result<QueryResult, String> {
    let service = SimpleBalanceQueryService::new();
    
    let result = service.query_balances_with_updates(params, app_handle, window_id).await;
    Ok(result)
}

// 停止余额查询命令
#[tauri::command]
pub async fn stop_balance_query(window_id: String) -> Result<(), String> {
    set_stop_flag(&window_id, true);
    println!("收到停止余额查询请求，窗口ID: {}", window_id);
    Ok(())
}

// 重置停止标志
#[tauri::command]
pub async fn reset_balance_query_stop(window_id: String) -> Result<(), String> {
    reset_stop_flag(&window_id);
    println!("重置余额查询停止标志，窗口ID: {}", window_id);
    Ok(())
}
