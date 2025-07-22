use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferConfig {
    pub chain: String,
    pub chain_layer: Option<String>,
    pub l1: Option<String>,
    pub scalar: Option<f64>,
    pub delay: [u64, u64],
    pub transfer_type: TransferType,
    pub transfer_amount: Option<String>,
    pub transfer_amount_list: Option<[String; 2]>,
    pub left_amount_list: Option<[String; 2]>,
    pub amount_precision: u8,
    pub limit_type: GasLimitType,
    pub limit_count: Option<u64>,
    pub limit_count_list: Option<[u64; 2]>,
    pub gas_price_type: GasPriceType,
    pub gas_price_rate: Option<f64>,
    pub gas_price: Option<String>,
    pub max_gas_price: Option<String>,
    pub error_retry: bool,
    pub error_count_limit: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    #[serde(rename = "1")]
    All,
    #[serde(rename = "2")]
    Fixed,
    #[serde(rename = "3")]
    Random,
    #[serde(rename = "4")]
    RemainRandom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GasLimitType {
    #[serde(rename = "1")]
    Auto,
    #[serde(rename = "2")]
    Fixed,
    #[serde(rename = "3")]
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GasPriceType {
    #[serde(rename = "1")]
    Auto,
    #[serde(rename = "2")]
    Fixed,
    #[serde(rename = "3")]
    Rate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferItem {
    pub key: String,
    pub private_key: String,
    pub to_addr: String,
    pub amount: Option<String>,
    pub plat_balance: String,
    pub coin_balance: String,
    pub exec_status: ExecStatus,
    pub error_msg: String,
    pub retry_flag: bool,
    pub error_count: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecStatus {
    #[serde(rename = "0")]
    Waiting,
    #[serde(rename = "1")]
    Executing,
    #[serde(rename = "2")]
    Success,
    #[serde(rename = "3")]
    Failed,
}

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub key: String,
    pub chain: String,
    pub scan_url: String,
    pub scan_api: String,
    pub verify_api: String,
    pub check_verify_api: String,
    pub layer: Option<String>,
    pub l1: Option<String>,
    pub scalar: Option<f64>,
    pub pic_url: String,
    pub gas_price: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CoinConfig {
    pub key: String,
    pub coin: String,
    pub label: String,
    pub coin_type: String,
    pub contract_type: Option<String>,
    pub contract_address: Option<String>,
    pub abi: Option<String>,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            chain: "eth".to_string(),
            chain_layer: None,
            l1: None,
            scalar: None,
            delay: [1, 3],
            transfer_type: TransferType::Random,
            transfer_amount: None,
            transfer_amount_list: Some(["1".to_string(), "100".to_string()]),
            left_amount_list: Some(["1".to_string(), "100".to_string()]),
            amount_precision: 6,
            limit_type: GasLimitType::Auto,
            limit_count: None,
            limit_count_list: None,
            gas_price_type: GasPriceType::Rate,
            gas_price_rate: Some(0.05),
            gas_price: Some("30".to_string()),
            max_gas_price: None,
            error_retry: true,
            error_count_limit: 3,
        }
    }
}

impl Default for ExecStatus {
    fn default() -> Self {
        Self::Waiting
    }
}
