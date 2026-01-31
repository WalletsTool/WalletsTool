use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WalletGroup {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub chain_type: Option<String>, // 'evm' or 'solana' or NULL for old data (migration) or if flexible
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Wallet {
    pub id: i64,
    pub group_id: Option<i64>,
    pub name: Option<String>,
    pub address: String,
    pub chain_type: String, // 'evm' or 'solana'
    pub wallet_type: String, // 'full_wallet' or 'address_only'
    #[serde(skip_serializing)]
    pub encrypted_private_key: Option<String>,
    #[serde(skip_serializing)]
    pub encrypted_mnemonic: Option<String>,
    pub mnemonic_index: Option<i64>,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: i64,
    pub group_id: Option<i64>,
    pub name: Option<String>,
    pub address: String,
    pub chain_type: String,
    #[serde(default)]
    pub wallet_type: String,
    #[serde(default)]
    pub has_private_key: bool,
    #[serde(default)]
    pub has_mnemonic: bool,
    pub sealed_private_key: Option<String>,
    pub sealed_mnemonic: Option<String>,
    pub mnemonic_index: Option<i64>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletsResult {
    pub total: u32,
    #[serde(default)]
    pub preview: Vec<WalletInfo>,
    pub sealed_mnemonic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSecrets {
    pub id: i64,
    pub name: Option<String>,
    pub address: String,
    pub sealed_private_key: Option<String>,
    pub sealed_mnemonic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AppConfig {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub parent_id: Option<i64>,
    pub name: String,
    pub chain_type: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupRequest {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub group_id: Option<i64>,
    pub name: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    pub chain_type: String,
    pub sealed_private_key: Option<String>,
    pub sealed_mnemonic: Option<String>,
    pub remark: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub transport_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateWalletsMode {
    MnemonicImport,
    PrivateKeyImport,
    GenerateSameMnemonic,
    GenerateDifferentMnemonic,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWalletsRequest {
    pub group_id: Option<i64>,
    pub name: Option<String>,
    pub chain_type: String,
    pub mode: CreateWalletsMode,
    pub sealed_mnemonic: Option<String>,
    pub sealed_private_key: Option<String>,
    pub count: u32,
    #[serde(default)]
    pub start_index: Option<u32>,
    #[serde(default)]
    pub word_count: Option<u32>,
    pub remark: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub preview_limit: Option<u32>,
    #[serde(default)]
    pub include_secrets: Option<bool>,
    #[serde(default)]
    pub transport_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWalletRequest {
    pub id: i64,
    pub group_id: Option<i64>,
    pub name: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InitPasswordRequest {
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub encrypted_password_b64: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyPasswordRequest {
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub encrypted_password_b64: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletExportData {
    pub id: i64,
    pub name: Option<String>,
    pub address: String,
    pub chain_type: String,
    pub private_key: Option<String>,
    pub mnemonic: Option<String>,
    pub mnemonic_index: Option<i64>,
    pub remark: Option<String>,
    pub group_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ExportWalletsRequest {
    pub ids: Vec<i64>,
    pub password: String,
}

// ==================== Watch Address (Read-only Address) Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchAddressInfo {
    pub id: i64,
    pub group_id: Option<i64>,
    pub group_name: Option<String>,
    pub name: Option<String>,
    pub address: String,
    pub chain_type: String,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWatchAddressRequest {
    pub group_id: Option<i64>,
    pub name: Option<String>,
    pub address: String,
    pub chain_type: String,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWatchAddressesRequest {
    pub group_id: Option<i64>,
    pub name_prefix: Option<String>,
    pub chain_type: String,
    pub addresses: Vec<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWatchAddressRequest {
    pub id: i64,
    pub group_id: Option<i64>,
    pub name: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WatchAddressExportData {
    pub id: i64,
    pub name: Option<String>,
    pub address: String,
    pub chain_type: String,
    pub remark: Option<String>,
    pub group_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ExportWatchAddressesRequest {
    pub ids: Vec<i64>,
}
