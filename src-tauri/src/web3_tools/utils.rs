use std::fs::File;
use std::io::copy;
use std::path::Path;

use tauri::api::http::{ClientBuilder, HttpRequestBuilder, ResponseType};
use tauri::command;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
// 下载文件到指定路径
#[command]
pub fn download_file(url: &str, file_path: &Path) -> () {
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