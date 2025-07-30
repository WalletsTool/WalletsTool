use tauri::State;
use crate::database::chain_service::ChainService;
use crate::database::models::CreateRpcProviderRequest;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use reqwest;

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcProviderInfo {
    pub id: i64,
    pub chain_id: i64,
    pub rpc_url: String,
    pub is_active: bool,
    pub priority: i32,
    pub last_success_at: Option<String>,
    pub failure_count: i32,
    pub avg_response_time_ms: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRpcProviderRequest {
    pub rpc_url: String,
    pub is_active: bool,
    pub priority: i32,
}

#[derive(Debug, Serialize)]
pub struct RpcTestResult {
    pub success: bool,
    pub response_time_ms: u64,
}

/// 获取指定链的 RPC 提供商
#[tauri::command]
pub async fn get_rpc_providers(
    chain_key: String,
    chain_service: State<'_, ChainService<'_>>,
) -> Result<Vec<RpcProviderInfo>, String> {
    let providers = chain_service.get_rpc_providers_by_chain(&chain_key).await
        .map_err(|e| format!("获取 RPC 提供商失败: {}", e))?;
    
    let provider_infos = providers.into_iter().map(|p| RpcProviderInfo {
        id: p.id,
        chain_id: p.chain_id,
        rpc_url: p.rpc_url,
        is_active: p.is_active,
        priority: p.priority,
        last_success_at: p.last_success_at.map(|dt| dt.to_rfc3339()),
        failure_count: p.failure_count,
        avg_response_time_ms: p.avg_response_time_ms,
    }).collect();
    
    Ok(provider_infos)
}

/// 添加 RPC 提供商
#[tauri::command]
pub async fn add_rpc_provider(
    chain_key: String,
    rpc_url: String,
    priority: i32,
    chain_service: State<'_, ChainService<'_>>,
) -> Result<RpcProviderInfo, String> {
    let request = CreateRpcProviderRequest {
        chain_key: chain_key.clone(),
        rpc_url: rpc_url.clone(),
        priority,
    };
    
    let provider = chain_service.add_rpc_provider_by_chain_key(&chain_key, &request).await
        .map_err(|e| format!("添加 RPC 提供商失败: {}", e))?;
    
    Ok(RpcProviderInfo {
        id: provider.id,
        chain_id: provider.chain_id,
        rpc_url: provider.rpc_url,
        is_active: provider.is_active,
        priority: provider.priority,
        last_success_at: provider.last_success_at.map(|dt| dt.to_rfc3339()),
        failure_count: provider.failure_count,
        avg_response_time_ms: provider.avg_response_time_ms,
    })
}

/// 更新 RPC 提供商
#[tauri::command]
pub async fn update_rpc_provider(
    id: i64,
    request: UpdateRpcProviderRequest,
    chain_service: State<'_, ChainService<'_>>,
) -> Result<RpcProviderInfo, String> {
    let provider = chain_service.update_rpc_provider(id, &request.rpc_url, request.is_active, request.priority).await
        .map_err(|e| format!("更新 RPC 提供商失败: {}", e))?;
    
    Ok(RpcProviderInfo {
        id: provider.id,
        chain_id: provider.chain_id,
        rpc_url: provider.rpc_url,
        is_active: provider.is_active,
        priority: provider.priority,
        last_success_at: provider.last_success_at.map(|dt| dt.to_rfc3339()),
        failure_count: provider.failure_count,
        avg_response_time_ms: provider.avg_response_time_ms,
    })
}

/// 删除 RPC 提供商
#[tauri::command]
pub async fn delete_rpc_provider(
    id: i64,
    chain_service: State<'_, ChainService<'_>>,
) -> Result<(), String> {
    chain_service.delete_rpc_provider(id).await
        .map_err(|e| format!("删除 RPC 提供商失败: {}", e))?;
    
    Ok(())
}

/// 测试 RPC 连接
#[tauri::command]
pub async fn test_rpc_connection(
    rpc_url: String,
) -> Result<RpcTestResult, String> {
    println!("[RPC测试] 开始测试RPC连接: {}", rpc_url);
    let start_time = std::time::Instant::now();
    
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| {
            let error_msg = format!("创建 HTTP 客户端失败: {}", e);
            println!("[RPC测试] {}", error_msg);
            error_msg
        })?;
    
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });
    
    println!("[RPC测试] 发送请求到: {}, 请求体: {}", rpc_url, payload);
    
    let success = match client.post(&rpc_url)
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            println!("[RPC测试] 收到响应，状态码: {}", response.status());
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        println!("[RPC测试] 响应JSON: {}", json);
                        let has_result = json.get("result").is_some();
                        println!("[RPC测试] 是否包含result字段: {}", has_result);
                        has_result
                    }
                    Err(e) => {
                        println!("[RPC测试] 解析JSON失败: {}", e);
                        false
                    }
                }
            } else {
                println!("[RPC测试] HTTP状态码不成功: {}", response.status());
                false
            }
        }
        Err(e) => {
            println!("[RPC测试] 请求失败: {}", e);
            false
        }
    };
    
    let response_time_ms = start_time.elapsed().as_millis() as u64;
    
    println!("[RPC测试] 测试完成 - 成功: {}, 响应时间: {}ms", success, response_time_ms);
    
    Ok(RpcTestResult {
        success,
        response_time_ms,
    })
}