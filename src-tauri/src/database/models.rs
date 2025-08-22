use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// 链配置模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Chain {
    pub id: i64,
    pub chain_key: String,
    pub chain_name: String,
    pub chain_id: i64,
    pub native_currency_symbol: String,
    pub native_currency_name: String,
    pub native_currency_decimals: i32,
    pub pic_data: Option<String>,  // Base64编码的图标数据
    pub scan_url: Option<String>,
    pub scan_api: Option<String>,
    pub verify_api: Option<String>,
    pub check_verify_api: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// RPC提供商配置模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RpcProvider {
    pub id: i64,
    pub chain_id: i64,
    pub rpc_url: String,
    pub is_active: bool,
    pub priority: i32,  // 优先级，数值越小优先级越高
    pub last_success_at: Option<DateTime<Utc>>,
    pub failure_count: i32,
    pub avg_response_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 代币配置模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Token {
    pub id: i64,
    pub chain_id: i64,
    pub token_key: String,
    pub token_name: String,
    pub symbol: String,
    pub contract_address: Option<String>,
    pub decimals: i32,
    pub token_type: String, // "base" or "token"
    pub contract_type: Option<String>, // 合约类型
    pub abi: Option<String>, // 智能合约ABI JSON字符串
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}



/// 创建链的请求模型
#[derive(Debug, Deserialize)]
pub struct CreateChainRequest {
    pub chain_key: String,
    pub chain_name: String,
    pub chain_id: i64,
    pub native_currency_symbol: String,
    pub native_currency_name: String,
    pub native_currency_decimals: i32,
    pub pic_data: Option<String>,  // Base64编码的图标数据
    pub scan_url: Option<String>,
    pub scan_api: Option<String>,
    pub verify_api: Option<String>,
    pub check_verify_api: Option<String>,
    pub rpc_urls: Option<Vec<String>>,
}

/// 更新链的请求模型
#[derive(Debug, Deserialize)]
pub struct UpdateChainRequest {
    pub chain_name: String,
    pub chain_id: i64,
    pub native_currency_symbol: String,
    pub native_currency_name: String,
    pub native_currency_decimals: i32,
    pub pic_data: Option<String>,  // Base64编码的图标数据
    pub scan_url: Option<String>,
    pub scan_api: Option<String>,
    pub verify_api: Option<String>,
    pub check_verify_api: Option<String>,
    pub rpc_urls: Option<Vec<String>>,
}

/// 创建 RPC提供商的请求模型
#[derive(Debug, Deserialize)]
pub struct CreateRpcProviderRequest {
    #[allow(dead_code)]
    pub chain_key: String,
    #[allow(dead_code)]
    pub rpc_url: String,
    #[allow(dead_code)]
    pub priority: i32,
}

/// 创建代币的请求模型
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub chain_key: String,
    pub token_key: String,
    pub token_name: String,
    pub symbol: String,
    pub contract_address: Option<String>,
    pub decimals: i32,
    pub token_type: String,
    pub contract_type: Option<String>, // 合约类型
    pub abi: Option<String>, // 智能合约ABI JSON字符串
}

/// 更新代币的请求模型
#[derive(Debug, Deserialize)]
pub struct UpdateTokenRequest {
    pub token_name: String,
    pub symbol: String,
    pub contract_address: Option<String>,
    pub decimals: i32,
    pub token_type: String,
    pub contract_type: Option<String>, // 合约类型
    pub abi: Option<String>, // 智能合约ABI JSON字符串
}

/// 简化的链配置（用于API返回）
#[derive(Debug, Serialize)]
pub struct ChainInfo {
    pub key: String,
    pub chain: String,
    pub chain_id: i64,
    pub symbol: String,
    pub currency_name: String,
    pub decimals: i32,
    pub pic_data: Option<String>,
    pub scan_url: String,
    pub scan_api: String,
    pub verify_api: String,
    pub check_verify_api: String,
    pub rpc_urls: Vec<String>,
}
