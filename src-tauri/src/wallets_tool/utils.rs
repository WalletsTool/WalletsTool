use std::path::Path;
use tauri::command;
use crate::database::chain_service::ChainService;
use base64::{Engine as _, engine::general_purpose};
use tauri::State;
use sqlx;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// 下载文件到指定路径
#[command]
pub fn download_file(_url: &str, _file_path: &Path) -> () {
    // let client = ClientBuilder::new().build().unwrap();
    // // 请求文件
    // let response = client.send(
    //     HttpRequestBuilder::new("GET", url)
    //         .unwrap()
    //         .response_type(ResponseType::Binary)
    // ).await;
    // // 写入文件
    // return if let Ok(mut response) = response {
    //     response::write_to_file(&mut response, file_path)?;
    //     Ok(())
    // } else {
    //     Err("获取文件失败！".into())
    // }
}

// 保存链图标数据到数据库
#[command]
pub async fn save_chain_icon(
    chain_key: String,
    file_name: String, 
    file_data: Vec<u8>,
    chain_service: State<'_, ChainService<'_>>
) -> Result<String, String> {
    // 将文件数据转换为Base64编码
    let base64_data = general_purpose::STANDARD.encode(&file_data);
    
    // 检查链是否存在
    let chain = chain_service.get_chain_by_key(&chain_key).await
        .map_err(|e| format!("获取链信息失败: {}", e))?;
    
    if let Some(chain) = chain {
        // 链已存在，更新图标数据
        sqlx::query(
            "UPDATE chains SET pic_data = ?, pic_url = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&base64_data)
        .bind(&file_name)
        .bind(chrono::Utc::now())
        .bind(chain.id)
        .execute(chain_service.get_pool())
        .await
        .map_err(|e| format!("更新图标数据失败: {}", e))?;
        
        println!("图标数据更新成功: {} -> {}", chain_key, file_name);
    } else {
        // 链不存在，这是新增链的情况，直接返回base64数据供前端使用
        println!("为新增链准备图标数据: {} -> {}", chain_key, file_name);
    }
    
    Ok(base64_data)
}

// 获取链图标数据
#[command]
pub async fn get_chain_icon(
    chain_key: String,
    chain_service: State<'_, ChainService<'_>>
) -> Result<Option<String>, String> {
    let chain = chain_service.get_chain_by_key(&chain_key).await
        .map_err(|e| format!("获取链信息失败: {}", e))?;
    
    match chain {
        Some(chain) => Ok(chain.pic_data),
        None => Ok(None)
    }
}