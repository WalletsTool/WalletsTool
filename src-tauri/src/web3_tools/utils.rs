use std::path::Path;
use std::fs;
use tauri::command;

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

// 保存链图标文件
#[command]
pub async fn save_chain_icon(file_name: String, file_data: Vec<u8>) -> Result<(), String> {
    // 获取当前工作目录
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("获取当前目录失败: {}", e))?;
    
    println!("当前工作目录: {:?}", current_dir);
    
    // 尝试多个可能的路径
    let possible_paths = vec![
        current_dir.join("public").join("chainIcons"),
        current_dir.join("..").join("public").join("chainIcons"),
        current_dir.join("..").join("..").join("public").join("chainIcons"),
    ];
    
    let mut chain_icons_dir = None;
    for path in possible_paths {
        let normalized_path = path.canonicalize().unwrap_or(path.clone());
        println!("尝试路径: {:?}", normalized_path);
        
        // 如果父目录存在，就使用这个路径
        if let Some(parent) = normalized_path.parent() {
            if parent.exists() {
                chain_icons_dir = Some(normalized_path);
                break;
            }
        }
    }
    
    let chain_icons_dir = chain_icons_dir.unwrap_or_else(|| {
        // 默认使用当前目录下的public/chainIcons
        current_dir.join("public").join("chainIcons")
    });
    
    println!("选择的目录: {:?}", chain_icons_dir);
    
    // 确保目录存在
    if !chain_icons_dir.exists() {
        fs::create_dir_all(&chain_icons_dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;
        println!("创建目录: {:?}", chain_icons_dir);
    }
    
    // 构建完整的文件路径
    let file_path = chain_icons_dir.join(&file_name);
    
    // 添加调试信息
    println!("保存图标文件到: {:?}", file_path);
    
    // 写入文件
    fs::write(&file_path, &file_data)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    println!("图标文件保存成功: {}", file_name);
    Ok(())
}