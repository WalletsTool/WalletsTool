use serde::{Deserialize, Serialize};
use crate::wallets_tool::security::SecureMemory;
#[allow(deprecated)]
use solana_sdk::{
    signature::{Keypair, Signer},
    pubkey::Pubkey,
    system_instruction,
    transaction::Transaction,
    compute_budget::ComputeBudgetInstruction,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use std::str::FromStr;
use crate::wallets_tool::ecosystems::solana::provider::get_rpc_client;
use tauri::Emitter;
use serde_json::json;

#[derive(Deserialize)]
pub struct TransferItem {
    pub private_key: SecureMemory,
    pub to_addr: String,
    pub amount: Option<String>,
}

#[derive(Deserialize)]
pub struct TransferConfig {
    pub transfer_amount: f64,
    pub gas_price: Option<u64>,
    pub contract_address: Option<String>,
    pub amount_precision: Option<u8>,
}

#[derive(Serialize)]
pub struct TransferResult {
    success: bool,
    tx_hash: Option<String>,
    error: Option<String>,
}

#[tauri::command]
pub async fn sol_transfer(
    _index: usize,
    item: TransferItem,
    config: TransferConfig,
) -> Result<TransferResult, String> {
    let client = get_rpc_client("sol").await?;
    
    let keypair = item.private_key.use_secret(|secret_str| {
        let bytes = bs58::decode(secret_str).into_vec().map_err(|e| e.to_string())?;
        Keypair::try_from(bytes.as_slice()).map_err(|e| e.to_string())
    }).map_err(|e| e.to_string())??;

    let to_pubkey = Pubkey::from_str(&item.to_addr).map_err(|e| e.to_string())?;
    
    let amount_sol = if let Some(amt) = &item.amount {
         if !amt.is_empty() { amt.parse::<f64>().unwrap_or(config.transfer_amount) } else { config.transfer_amount }
    } else {
        config.transfer_amount
    };
    let lamports = (amount_sol * 1_000_000_000.0) as u64;

    let mut instructions = vec![];
    if let Some(fee) = config.gas_price {
        if fee > 0 {
             instructions.push(ComputeBudgetInstruction::set_compute_unit_price(fee));
        }
    }
    instructions.push(system_instruction::transfer(&keypair.pubkey(), &to_pubkey, lamports));

    let recent_blockhash = client.get_latest_blockhash().await?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(sig) => Ok(TransferResult { success: true, tx_hash: Some(sig.to_string()), error: None }),
        Err(e) => Ok(TransferResult { success: false, tx_hash: None, error: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn sol_token_transfer(
    _index: usize,
    item: TransferItem,
    config: TransferConfig,
) -> Result<TransferResult, String> {
    let client = get_rpc_client("sol").await?;
    let mint_str = config.contract_address.ok_or("Missing Mint Address")?;
    let mint = Pubkey::from_str(&mint_str).map_err(|_| "Invalid Mint Address")?;
    
    let keypair = item.private_key.use_secret(|secret_str| {
        let bytes = bs58::decode(secret_str).into_vec().map_err(|e| e.to_string())?;
        Keypair::try_from(bytes.as_slice()).map_err(|e| e.to_string())
    }).map_err(|e| e.to_string())??;

    let to_pubkey = Pubkey::from_str(&item.to_addr).map_err(|_| "Invalid To Address")?;
    
    let from_ata = get_associated_token_address(&keypair.pubkey(), &mint);
    let to_ata = get_associated_token_address(&to_pubkey, &mint);
    
    let amount_val = if let Some(amt) = &item.amount {
         if !amt.is_empty() { amt.parse::<f64>().unwrap_or(config.transfer_amount) } else { config.transfer_amount }
    } else {
        config.transfer_amount
    };
    let decimals = config.amount_precision.unwrap_or(6);
    let amount_u64 = (amount_val * 10f64.powi(decimals as i32)) as u64;

    let mut instructions = vec![];
    if let Some(fee) = config.gas_price {
        if fee > 0 {
             instructions.push(ComputeBudgetInstruction::set_compute_unit_price(fee));
        }
    }

    // Check if destination account exists
    let should_create_ata = match client.get_account(&to_ata).await {
        Ok(res) => res["value"].is_null(),
        Err(_) => true,
    };

    if should_create_ata {
        instructions.push(create_associated_token_account(
            &keypair.pubkey(),
            &to_pubkey,
            &mint,
            &spl_token::id(),
        ));
    }
    
    instructions.push(spl_token::instruction::transfer(
        &spl_token::id(),
        &from_ata,
        &to_ata,
        &keypair.pubkey(),
        &[],
        amount_u64,
    ).map_err(|e| e.to_string())?);

    let recent_blockhash = client.get_latest_blockhash().await?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(sig) => Ok(TransferResult { success: true, tx_hash: Some(sig.to_string()), error: None }),
        Err(e) => Ok(TransferResult { success: false, tx_hash: None, error: Some(e.to_string()) }),
    }
}

#[derive(Serialize)]
pub struct CheckResult {
    has_recent_transfer: bool,
}

#[tauri::command]
pub async fn sol_check_recent_transfers(
    _chain: String,
    _private_key: SecureMemory,
    _target_address: String,
    _start_timestamp: Option<i64>,
    _coin_type: String,
    _contract_address: Option<String>,
    _amount: Option<String>,
) -> Result<CheckResult, String> {
    Ok(CheckResult { has_recent_transfer: false })
}

#[tauri::command]
pub async fn sol_check_transactions_status_batch(
    _chain: String,
    _tx_hashes: Vec<String>,
) -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn sol_transfer_fast(
    index: usize,
    item: TransferItem,
    config: TransferConfig,
) -> Result<TransferResult, String> {
    sol_transfer(index, item, config).await
}

#[tauri::command]
pub async fn sol_token_transfer_fast(
    index: usize,
    item: TransferItem,
    config: TransferConfig,
) -> Result<TransferResult, String> {
    sol_token_transfer(index, item, config).await
}

#[derive(Deserialize)]
pub struct BalanceQueryParams {
    pub items: Vec<BalanceItem>,
    pub window_id: String,
    pub query_id: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BalanceItem {
    pub key: String,
    pub address: String,
    pub private_key: Option<String>, // We don't need private key for balance, but struct has it
    pub plat_balance: Option<String>,
    pub coin_balance: Option<String>,
    pub exec_status: String,
    pub error_msg: Option<String>,
}

#[tauri::command]
pub async fn sol_query_balances_with_updates(
    params: BalanceQueryParams,
    window: tauri::Window,
) -> Result<serde_json::Value, String> {
    let client = match get_rpc_client("sol").await {
        Ok(c) => c,
        Err(e) => return Err(e),
    };
    
    let mut results = vec![];
    
    for mut item in params.items {
        if let Ok(pubkey) = Pubkey::from_str(&item.address) {
            match client.get_balance(&pubkey).await {
                Ok(bal) => {
                    item.plat_balance = Some((bal as f64 / 1_000_000_000.0).to_string());
                    item.exec_status = "2".to_string();
                },
                Err(e) => {
                    item.exec_status = "3".to_string();
                    item.error_msg = Some(e.to_string());
                }
            }
        } else {
            item.exec_status = "3".to_string();
            item.error_msg = Some("Invalid Address".to_string());
        }
        
        let _ = window.emit("balance_item_update", json!({
            "item": item,
            "window_id": params.window_id,
            "query_id": params.query_id
        }));
        
        results.push(item);
    }
    
    Ok(json!({ "success": true, "items": results }))
}
