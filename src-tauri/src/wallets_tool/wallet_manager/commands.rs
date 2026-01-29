use tauri::State;
use crate::wallets_tool::wallet_manager::service::WalletManagerService;
use super::models::*;

#[tauri::command]
pub async fn init_wallet_manager_tables(state: State<'_, WalletManagerService>) -> Result<(), String> {
    state.init_tables().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_password_set(state: State<'_, WalletManagerService>) -> Result<bool, String> {
    state.is_password_set().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn init_password(state: State<'_, WalletManagerService>, request: InitPasswordRequest) -> Result<(), String> {
    state.init_password(&request.password).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_password(state: State<'_, WalletManagerService>, request: VerifyPasswordRequest) -> Result<bool, String> {
    state.unlock(&request.password).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn change_password(state: State<'_, WalletManagerService>, request: ChangePasswordRequest) -> Result<(), String> {
    state.change_password(&request.old_password, &request.new_password).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_group(state: State<'_, WalletManagerService>, request: CreateGroupRequest) -> Result<i64, String> {
    state.create_group(request).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_groups(state: State<'_, WalletManagerService>) -> Result<Vec<WalletGroup>, String> {
    state.get_groups().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_group(state: State<'_, WalletManagerService>, request: UpdateGroupRequest) -> Result<(), String> {
    state.update_group(request).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_group(state: State<'_, WalletManagerService>, id: i64) -> Result<(), String> {
    state.delete_group(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_wallet(state: State<'_, WalletManagerService>, request: CreateWalletRequest) -> Result<i64, String> {
    state.create_wallet(request).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_wallets(state: State<'_, WalletManagerService>, request: CreateWalletsRequest) -> Result<Vec<WalletInfo>, String> {
    state.create_wallets(request, None).await.map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_wallets(
    state: State<'_, WalletManagerService>, 
    group_id: Option<i64>, 
    chain_type: Option<String>,
    password: Option<String>
) -> Result<Vec<WalletInfo>, String> {
    state.get_wallets(group_id, chain_type, password).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_wallet(state: State<'_, WalletManagerService>, request: UpdateWalletRequest) -> Result<(), String> {
    state.update_wallet(request).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_wallet(state: State<'_, WalletManagerService>, id: i64) -> Result<(), String> {
    state.delete_wallet(id).await.map_err(|e| e.to_string())
}
