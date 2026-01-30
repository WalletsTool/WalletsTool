#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod utils;
mod wallets_tool;
mod plugins;
mod database;

use tauri::{WindowEvent, Manager, AppHandle, Runtime, Emitter, tray::TrayIconBuilder, menu::{MenuBuilder, MenuItemBuilder}};


// Tauri 命令：关闭所有子窗口
#[tauri::command]
async fn close_all_child_windows<R: Runtime>(app: AppHandle<R>, main_window_label: String) -> Result<Vec<String>, String> {
    let mut closed_windows = Vec::new();

    let windows = app.webview_windows();

    for (label, window) in windows {
        if label != main_window_label {  // 只排除主窗口
            match window.close() {
                Ok(_) => {
                    closed_windows.push(label);
                }
                Err(e) => {
                    eprintln!("关闭窗口 {label} 失败: {e}");
                }
            }
        }
    }

    Ok(closed_windows)
}

// Tauri 命令：获取所有子窗口
#[tauri::command]
async fn get_all_child_windows<R: Runtime>(app: AppHandle<R>, main_window_label: String) -> Result<Vec<String>, String> {
    let windows = app.webview_windows();
    let child_windows: Vec<String> = windows.keys()
        .filter(|&label| label != &main_window_label)
        .cloned()
        .collect();
    
    Ok(child_windows)
}

// Tauri 命令：强制关闭主窗口（跳过事件处理）
#[tauri::command]
async fn force_close_main_window<R: Runtime>(_app: AppHandle<R>) -> Result<(), String> {
    // 直接退出应用程序，跳过窗口关闭事件处理
    std::process::exit(0);
}

// Tauri 命令：显示主窗口
#[tauri::command]
async fn show_main_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;

        // 在Windows系统中强制窗口置顶，然后立即取消置顶状态
        // 这样可以确保窗口弹出到最上层而不会一直保持在最上层
        window.set_always_on_top(true).map_err(|e| e.to_string())?;
        window.set_always_on_top(false).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Tauri 命令：从托盘打开功能窗口
#[tauri::command]
async fn open_function_window<R: Runtime>(app: AppHandle<R>, page_name: String) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;
    
    let (title, _icon) = match page_name.as_str() {
        "transfer" => ("💸 批量转账", "transfer"),
        "balance" => ("💰 余额查询", "balance"),
        "monitor" => ("👁️ 链上监控", "monitor"),
        _ => ("❓ 未知功能", "unknown")
    };
    
    // let display_icon = match icon {
    //     "transfer" => "💸",
    //     "balance" => "💰",
    //     "monitor" => "👁️",
    //     _ => ""
    // };
    
    // 获取当前所有窗口的标签
    let existing_windows = app.webview_windows();
    let mut window_count = 1;
    
    // 循环查找可用的窗口标签，确保不与现有窗口冲突
    let window_label = loop {
        let candidate_label = format!("{page_name}{window_count}");
        
        // 检查这个标签是否已经存在
        if !existing_windows.contains_key(&candidate_label) {
            break candidate_label;
        }
        
        // 如果存在，递增计数器继续尝试
        window_count += 1;
        
        // 防止无限循环，设置一个合理的上限
        if window_count > 100 {
            return Err("无法找到可用的窗口标签，已达到最大窗口数量限制".to_string());
        }
    };
    
    let window_url = format!("/#/{page_name}?count={window_count}");
    
    // 生成窗口标题：统一格式为 "WalletsTool - {图标} {功能名} [{序号}]"
    let window_title = if window_count > 1 {
        format!("WalletsTool - {title} [{window_count}]")
    } else {
        format!("WalletsTool - {title}")
    };
    
    // 创建新窗口
    let webview = WebviewWindowBuilder::new(&app, &window_label, tauri::WebviewUrl::App(window_url.into()))
        .title(&window_title)
        .inner_size(1350.0, 900.0)
        .resizable(true)
        .center()
        .decorations(false)
        .visible(false)
        .skip_taskbar(false)
        .build()
        .map_err(|e| e.to_string())?;
    
    // 显示窗口
    webview.show().map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tokio::main]
async fn main() {
    // 启动安全保护
    wallets_tool::security::enable_protection();

    // 初始化数据库
    if let Err(err) = database::init_database().await {
        eprintln!("数据库初始化失败: {err:?}");
        return;
    }
    
    // 创建数据库服务
    // Force rebuild: ecosystem field added
    let db_manager = database::get_database_manager();
    println!("Initializing WalletManagerService...");
    let wallet_manager_service = wallets_tool::wallet_manager::service::WalletManagerService::new(db_manager.get_pool().clone());
    
    let chain_service = database::chain_service::ChainService::new(db_manager.get_pool());
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(wallet_manager_service)
        .manage(chain_service)
        .setup(|app| {
            // 主窗口直接显示

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
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show_main" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = show_main_window(app_handle).await {
                                    eprintln!("显示主窗口失败: {e}");
                                }
                            });
                        }
                        "transfer" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = open_function_window(app_handle, "transfer".to_string()).await {
                                    eprintln!("打开批量转账窗口失败: {e}");
                                }
                            });
                        }
                        "balance" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Err(e) = open_function_window(app_handle, "balance".to_string()).await {
                                    eprintln!("打开余额查询窗口失败: {e}");
                                }
                            });
                        }
                        "quit" => {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                // 先显示主窗口
                                if let Err(e) = show_main_window(app_handle.clone()).await {
                                    eprintln!("显示主窗口失败: {e}");
                                }

                                // 发送退出确认事件到前端
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    if let Err(e) = window.emit("tray-quit-requested", ()) {
                                        eprintln!("发送托盘退出事件失败: {e}");
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
                                    eprintln!("左键点击托盘显示主窗口失败: {e}");
                                }
                            });
                        }
                        tauri::tray::TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Right,
                            button_state: tauri::tray::MouseButtonState::Up,
                            ..  
                        } => {
                            // 右键点击事件（菜单已在创建时设置）
                        }
                        _ => {}
                    }
                })
                .build(app)?;
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let window_label = window.label().to_string();

                if window_label == "main" {
                    // 阻止默认的关闭行为
                    api.prevent_close();
                    
                    // 将主窗口置于最前端，确保用户能看到确认对话框
                    if let Err(e) = window.show() {
                        eprintln!("显示主窗口失败: {e}");
                    }
                    if let Err(e) = window.set_focus() {
                        eprintln!("设置主窗口焦点失败: {e}");
                    }
                    
                    // 强制窗口置顶以确保在Windows系统中能够真正显示在最前端
                    if let Err(e) = window.set_always_on_top(true) {
                        eprintln!("设置窗口置顶失败: {e}");
                    }
                    
                    // 发送自定义事件到前端
                    if let Err(e) = window.emit("main-window-close-requested", ()) {
                        eprintln!("发送关闭事件失败: {e}");
                    }
                    
                    // 克隆窗口引用以便在异步任务中使用
                    let window_clone = window.clone();
                    
                    // 在短暂延迟后恢复窗口的正常状态
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                        if let Err(e) = window_clone.set_always_on_top(false) {
                            eprintln!("恢复窗口正常状态失败: {e}");
                        }
                    });
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            wallets_tool::ecosystems::ethereum::chain_config::get_chain_list,
            wallets_tool::ecosystems::ethereum::chain_config::get_coin_list,
            wallets_tool::ecosystems::ethereum::chain_config::add_coin,
            wallets_tool::ecosystems::ethereum::chain_config::remove_coin,
            wallets_tool::ecosystems::ethereum::chain_config::update_coin,
            wallets_tool::ecosystems::ethereum::chain_config::update_chain_pic_urls,
            wallets_tool::ecosystems::ethereum::chain_config::update_token_abi,
            // chain management commands
            wallets_tool::ecosystems::ethereum::chain_config::add_chain,
            wallets_tool::ecosystems::ethereum::chain_config::update_chain,
            wallets_tool::ecosystems::ethereum::chain_config::remove_chain,
            wallets_tool::ecosystems::ethereum::chain_config::get_chain_detail,
            wallets_tool::utils::download_file,
            wallets_tool::utils::save_chain_icon,
            wallets_tool::utils::get_chain_icon,
            wallets_tool::utils::read_resource_file,
            wallets_tool::utils::save_file,
            wallets_tool::utils::get_temp_dir,
            wallets_tool::utils::open_file_directory,
            // fs extra functions
            plugins::fs_extra::exists,
            plugins::fs_extra::open_file,
            // balance query functions
            wallets_tool::ecosystems::ethereum::simple_balance_query::query_balances_simple,
            wallets_tool::ecosystems::ethereum::simple_balance_query::query_balances_with_updates,
            wallets_tool::ecosystems::ethereum::simple_balance_query::stop_balance_query,
            wallets_tool::ecosystems::ethereum::simple_balance_query::reset_balance_query_stop,
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
            wallets_tool::transfer::base_coin_transfer,
            wallets_tool::transfer::base_coin_transfer_fast,
            wallets_tool::transfer::check_transactions_status_batch,
            wallets_tool::transfer::check_transaction_status,
            wallets_tool::transfer::query_balance,
            wallets_tool::transfer::check_wallet_recent_transfers,
            wallets_tool::transfer::stop_transfer,
            wallets_tool::transfer::reset_transfer_stop,
            // solana transfer functions
            wallets_tool::ecosystems::solana::transfer::sol_transfer,
            wallets_tool::ecosystems::solana::transfer::sol_token_transfer,
            wallets_tool::ecosystems::solana::transfer::sol_transfer_fast,
            wallets_tool::ecosystems::solana::transfer::sol_token_transfer_fast,
            wallets_tool::ecosystems::solana::transfer::sol_check_recent_transfers,
            wallets_tool::ecosystems::solana::transfer::sol_check_transactions_status_batch,
            wallets_tool::ecosystems::solana::transfer::sol_query_balances_with_updates,
            wallets_tool::ecosystems::solana::provider::test_solana_rpc_connection,

            // token transfer functions
            wallets_tool::token_transfer::token_transfer,
            wallets_tool::token_transfer::token_transfer_fast,
            wallets_tool::token_transfer::query_token_balance,
            wallets_tool::token_transfer::get_token_info,
            // provider functions
            wallets_tool::provider::get_chain_gas_price,
            wallets_tool::provider::test_rpc_url,
            wallets_tool::provider::get_multiple_gas_prices,
            // rpc management functions
            wallets_tool::ecosystems::ethereum::rpc_management::get_rpc_providers,
            wallets_tool::ecosystems::ethereum::rpc_management::add_rpc_provider,
            wallets_tool::ecosystems::ethereum::rpc_management::update_rpc_provider,
            wallets_tool::ecosystems::ethereum::rpc_management::delete_rpc_provider,
            wallets_tool::ecosystems::ethereum::rpc_management::test_rpc_connection,
            // proxy management functions
            wallets_tool::ecosystems::ethereum::proxy_commands::set_proxy_window_id,
            wallets_tool::ecosystems::ethereum::proxy_commands::save_proxy_config,
            wallets_tool::ecosystems::ethereum::proxy_commands::save_proxy_config_for_window,
            wallets_tool::ecosystems::ethereum::proxy_commands::get_proxy_config,
            wallets_tool::ecosystems::ethereum::proxy_commands::get_proxy_config_for_window,
            wallets_tool::ecosystems::ethereum::proxy_commands::test_proxy_connection,
            wallets_tool::ecosystems::ethereum::proxy_commands::get_proxy_stats,
            wallets_tool::ecosystems::ethereum::proxy_commands::get_proxy_stats_for_window,
            wallets_tool::ecosystems::ethereum::proxy_commands::clear_proxy_config_for_window,
            // wallet manager commands
            wallets_tool::wallet_manager::commands::init_wallet_manager_tables,
            wallets_tool::wallet_manager::commands::is_password_set,
            wallets_tool::wallet_manager::commands::init_password,
            wallets_tool::wallet_manager::commands::verify_password,
            wallets_tool::wallet_manager::commands::get_wallet_transport_public_key,
            wallets_tool::wallet_manager::commands::register_wallet_transport_key,
            wallets_tool::wallet_manager::commands::change_password,
            wallets_tool::wallet_manager::commands::create_group,
            wallets_tool::wallet_manager::commands::get_groups,
            wallets_tool::wallet_manager::commands::update_group,
            wallets_tool::wallet_manager::commands::delete_group,
            wallets_tool::wallet_manager::commands::create_wallet,
            wallets_tool::wallet_manager::commands::create_wallets,
            wallets_tool::wallet_manager::commands::get_wallets,
            wallets_tool::wallet_manager::commands::get_wallet_secrets,
            wallets_tool::wallet_manager::commands::export_wallets,
            wallets_tool::wallet_manager::commands::update_wallet,
            wallets_tool::wallet_manager::commands::delete_wallet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
