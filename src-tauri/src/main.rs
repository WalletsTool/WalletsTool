#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod utils;
mod wallet_manager;
mod plugins;
mod simple_balance_query;
mod database;

use tauri::{WindowEvent, Manager, AppHandle, Emitter, tray::TrayIconBuilder, menu::{MenuBuilder, MenuItemBuilder}};


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

// Tauri 命令：显示主窗口
#[tauri::command]
async fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("wallet_manager") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Tauri 命令：从托盘打开功能窗口
#[tauri::command]
async fn open_function_window(app: AppHandle, page_name: String) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;
    
    let title = match page_name.as_str() {
        "transfer" => "批量转账",
        "balance" => "余额查询",
        "monitor" => "链上地址监控",
        _ => "未知功能"
    };
    
    // 检查是否已有同类型窗口打开
    let existing_windows: Vec<String> = app.webview_windows().keys().cloned().collect();
    let mut window_count = 1;
    
    // 计算当前页面类型的窗口数量
    for label in &existing_windows {
        if label.starts_with(&page_name) {
            window_count += 1;
        }
    }
    
    let window_label = format!("{}{}", page_name, window_count);
    let window_url = format!("/#/{}", page_name);
    
    // 创建新窗口
    let _webview = WebviewWindowBuilder::new(&app, &window_label, tauri::WebviewUrl::App(window_url.into()))
        .title(&format!("▶ 窗口 {} 🧡 {}", window_count, title))
        .inner_size(1275.0, 850.0)
        .resizable(true)
        .center()
        .decorations(false)
        .visible(false)
        .skip_taskbar(false)
        .build()
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tokio::main]
async fn main() {
    // 初始化数据库
    if let Err(err) = database::init_database().await {
        eprintln!("数据库初始化失败: {:?}", err);
        return;
    }
    
    // 创建数据库服务
    let db_manager = database::get_database_manager();
    let chain_service = database::chain_service::ChainService::new(db_manager.get_pool());
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(chain_service)
        .setup(|app| {
            // 构建托盘菜单
            let show_main = MenuItemBuilder::new("显示主窗口").id("show_main").build(app)?;
            let separator1 = tauri::menu::PredefinedMenuItem::separator(app)?;
            let batch_transfer = MenuItemBuilder::new("批量转账").id("transfer").build(app)?;
            let balance_query = MenuItemBuilder::new("余额查询").id("balance").build(app)?;
            let separator2 = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit = MenuItemBuilder::new("退出程序").id("quit").build(app)?;
            
            let menu = MenuBuilder::new(app)
                .item(&show_main)
                .item(&separator1)
                .item(&batch_transfer)
                .item(&balance_query)
                .item(&separator2)
                .item(&quit)
                .build()?;
            
            // 创建托盘图标
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show_main" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = show_main_window(app_handle).await {
                                    eprintln!("显示主窗口失败: {}", e);
                                }
                            });
                        }
                        "transfer" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = open_function_window(app_handle, "transfer".to_string()).await {
                                    eprintln!("打开批量转账窗口失败: {}", e);
                                }
                            });
                        }
                        "balance" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = open_function_window(app_handle, "balance".to_string()).await {
                                    eprintln!("打开余额查询窗口失败: {}", e);
                                }
                            });
                        }
                        "quit" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                // 先显示主窗口
                                if let Err(e) = show_main_window(app_handle.clone()).await {
                                    eprintln!("显示主窗口失败: {}", e);
                                }
                                
                                // 发送退出确认事件到前端
                                if let Some(window) = app_handle.get_webview_window("wallet_manager") {
                                    if let Err(e) = window.emit("tray-quit-requested", ()) {
                                        eprintln!("发送托盘退出事件失败: {}", e);
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |tray, event| {
                    match event {
                        tauri::tray::TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Left,
                            button_state: tauri::tray::MouseButtonState::Up,
                            ..  
                        } => {
                            // 左键点击显示主窗口
                            let app_handle = tray.app_handle().clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = show_main_window(app_handle).await {
                                    eprintln!("左键点击托盘显示主窗口失败: {}", e);
                                }
                            });
                        }
                        tauri::tray::TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Right,
                            button_state: tauri::tray::MouseButtonState::Up,
                            ..  
                        } => {
                            // 右键点击显示菜单
                            if let Err(e) = tray.set_menu(Some(menu.clone())) {
                                eprintln!("设置托盘菜单失败: {}", e);
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;
            
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    let window_label = window.label().to_string();
                    
                    if window_label == "wallet_manager" {
                        // 阻止默认的关闭行为
                        api.prevent_close();
                        
                        // 将主窗口置于最前端，确保用户能看到确认对话框
                        if let Err(e) = window.show() {
                            eprintln!("显示主窗口失败: {}", e);
                        }
                        if let Err(e) = window.set_focus() {
                            eprintln!("设置主窗口焦点失败: {}", e);
                        }
                        
                        // 强制窗口置顶以确保在Windows系统中能够真正显示在最前端
                        if let Err(e) = window.set_always_on_top(true) {
                            eprintln!("设置窗口置顶失败: {}", e);
                        }
                        
                        // 发送自定义事件到前端
                        if let Err(e) = window.emit("main-window-close-requested", ()) {
                            eprintln!("发送关闭事件失败: {}", e);
                        }
                        
                        // 克隆窗口引用以便在异步任务中使用
                        let window_clone = window.clone();
                        
                        // 在短暂延迟后恢复窗口的正常状态
                        tokio::spawn(async move {
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                            if let Err(e) = window_clone.set_always_on_top(false) {
                                eprintln!("恢复窗口正常状态失败: {}", e);
                            }
                        });
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            wallet_manager::chain_config::get_chain_list,
            wallet_manager::chain_config::get_coin_list,
            wallet_manager::chain_config::add_coin,
            wallet_manager::chain_config::remove_coin,
            wallet_manager::chain_config::update_coin,
            wallet_manager::chain_config::update_chain_pic_urls,
            wallet_manager::chain_config::update_token_abi,
            // chain management commands
            wallet_manager::chain_config::add_chain,
            wallet_manager::chain_config::update_chain,
            wallet_manager::chain_config::remove_chain,
            wallet_manager::chain_config::get_chain_detail,
            wallet_manager::utils::download_file,
            wallet_manager::utils::save_chain_icon,
            wallet_manager::utils::get_chain_icon,
            // fs extra functions
            plugins::fs_extra::exists,
            plugins::fs_extra::open_file,
            // balance query functions
            simple_balance_query::query_balances_simple,
            simple_balance_query::query_balances_with_updates,
            simple_balance_query::stop_balance_query,
            simple_balance_query::reset_balance_query_stop,
            // window management functions
            close_all_child_windows,
            get_all_child_windows,
            force_close_main_window,
            show_main_window,
            open_function_window,
            // database hot reload functions
            database::reload_database,
            database::check_database_schema,
            database::export_database_to_init_sql,
            // transfer functions
            wallet_manager::transfer::base_coin_transfer,
            wallet_manager::transfer::query_balance,
            wallet_manager::transfer::check_wallet_recent_transfers,
            // token transfer functions
            wallet_manager::token_transfer::token_transfer,
            wallet_manager::token_transfer::query_token_balance,
            wallet_manager::token_transfer::get_token_info,
            // provider functions
            wallet_manager::provider::get_chain_gas_price,
            wallet_manager::provider::test_rpc_url,
            wallet_manager::provider::get_multiple_gas_prices,
            // rpc management functions
            wallet_manager::rpc_management::get_rpc_providers,
            wallet_manager::rpc_management::add_rpc_provider,
            wallet_manager::rpc_management::update_rpc_provider,
            wallet_manager::rpc_management::delete_rpc_provider,
            wallet_manager::rpc_management::test_rpc_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
