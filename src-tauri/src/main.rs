#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod utils;
mod web3_tools;
mod plugins;


fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            web3_tools::chain_config::get_chain_list,
            web3_tools::chain_config::get_coin_list,
            web3_tools::chain_config::add_coin,
            web3_tools::chain_config::remove_coin,
            web3_tools::utils::download_file])
        .plugin(plugins::ToolsExtra::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
