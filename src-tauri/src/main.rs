#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod utils;
mod web3_tools;
mod plugins;
mod simple_balance_query;
mod database;

use tauri::{WindowEvent, Manager, AppHandle, Emitter};


// Tauri 命令：关闭所有子窗口
#[tauri::command]
async fn close_all_child_windows(app: AppHandle, main_window_label: String) -> Result<Vec<String>, String> {
    let mut closed_windows = Vec::new();
    
    let windows = app.webview_windows();
    
    for (label, window) in windows {
        if label != main_window_label {
            match window.close() {
                Ok(_) => {
                    closed_windows.push(label);
                }
                Err(e) => {
                    eprintln!("关闭窗口 {} 失败: {}", label, e);
                }
            }
        }
    }
    
    Ok(closed_windows)
}

// Tauri 命令：获取所有子窗口
#[tauri::command]
async fn get_all_child_windows(app: AppHandle, main_window_label: String) -> Result<Vec<String>, String> {
    let windows = app.webview_windows();
    let child_windows: Vec<String> = windows.keys()
        .filter(|&label| label != &main_window_label)
        .cloned()
        .collect();
    
    Ok(child_windows)
}

// Tauri 命令：强制关闭主窗口（跳过事件处理）
#[tauri::command]
async fn force_close_main_window(_app: AppHandle) -> Result<(), String> {
    // 直接退出应用程序，跳过窗口关闭事件处理
    std::process::exit(0);
}

#[tokio::main]
async fn main() {
    // 初始化数据库
    if let Err(err) = database::init_database().await {
        eprintln!("数据库初始化失败: {:?}", err);
        return;
    }
    
    tauri::Builder::default()
        .on_window_event(|window, event| {
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    let window_label = window.label().to_string();
                    
                    if window_label == "main" {
                        // 阻止默认的关闭行为
                        api.prevent_close();
                        
                        // 发送自定义事件到前端
                        if let Err(e) = window.emit("main-window-close-requested", ()) {
                            eprintln!("发送关闭事件失败: {}", e);
                        }
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            web3_tools::chain_config::get_chain_list,
            web3_tools::chain_config::get_coin_list,
            web3_tools::chain_config::add_coin,
            web3_tools::chain_config::remove_coin,
            web3_tools::chain_config::update_coin,
            web3_tools::chain_config::update_chain_pic_urls,
            web3_tools::chain_config::update_token_abi,
            // chain management commands
            web3_tools::chain_config::add_chain,
            web3_tools::chain_config::update_chain,
            web3_tools::chain_config::remove_chain,
            web3_tools::chain_config::get_chain_detail,
            web3_tools::utils::download_file,
            web3_tools::utils::save_chain_icon,
            // fs extra functions
            plugins::fs_extra::exists,
            plugins::fs_extra::open_file,
            // balance query functions
            simple_balance_query::query_balances_simple,
            simple_balance_query::query_balances_with_updates,
            // window management functions
            close_all_child_windows,
            get_all_child_windows,
            force_close_main_window,
            // database hot reload functions
            database::reload_database,
            database::check_database_schema,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
