use reqwest::Client;
use serde_json::{json, Value};
use solana_sdk::{
    hash::Hash,
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

pub struct SolanaProvider {
    client: Client,
    rpc_url: String,
}

impl SolanaProvider {
    pub fn new(rpc_url: String) -> Self {
        Self {
            client: Client::new(),
            rpc_url,
        }
    }

    async fn request(&self, method: &str, params: Value) -> Result<Value, String> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let res = self.client.post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json: Value = res.json().await.map_err(|e| e.to_string())?;
        
        if let Some(err) = json.get("error") {
            return Err(err.to_string());
        }
        
        json.get("result").cloned().ok_or_else(|| "No result in response".to_string())
    }

    pub async fn get_latest_blockhash(&self) -> Result<Hash, String> {
        let res = self.request("getLatestBlockhash", json!([{"commitment": "finalized"}])).await?;
        let hash_str = res["value"]["blockhash"].as_str().ok_or("Invalid blockhash format")?;
        Hash::from_str(hash_str).map_err(|e| e.to_string())
    }

    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, String> {
        let res = self.request("getBalance", json!([pubkey.to_string()])).await?;
        res["value"].as_u64().ok_or_else(|| "Invalid balance format".to_string())
    }

    pub async fn get_account(&self, pubkey: &Pubkey) -> Result<Value, String> {
         self.request("getAccountInfo", json!([pubkey.to_string(), {"encoding": "base64"}])).await
    }

    pub async fn send_transaction(&self, transaction: &Transaction) -> Result<Signature, String> {
        let serialized = bincode::serialize(transaction).map_err(|e| e.to_string())?;
        let base64_tx = base64::encode(serialized);
        
        let res = self.request("sendTransaction", json!([base64_tx, {"encoding": "base64"}])).await?;
        let sig_str = res.as_str().ok_or("Invalid signature format")?;
        Signature::from_str(sig_str).map_err(|e| e.to_string())
    }

    pub async fn confirm_transaction(&self, signature: &Signature) -> Result<bool, String> {
        // Simple polling for confirmation
        for _ in 0..30 { // 30 attempts, 1s interval
            let res = self.request("getSignatureStatuses", json!([[signature.to_string()]])).await?;
            if let Some(statuses) = res["value"].as_array() {
                if let Some(status) = statuses.get(0) {
                     if !status.is_null() {
                        if let Some(confirmation) = status.get("confirmationStatus") {
                             let s = confirmation.as_str().unwrap_or("");
                             if s == "confirmed" || s == "finalized" {
                                 return Ok(true);
                             }
                        }
                     }
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        Err("Transaction confirmation timed out".to_string())
    }

    pub async fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<Signature, String> {
        let sig = self.send_transaction(transaction).await?;
        self.confirm_transaction(&sig).await?;
        Ok(sig)
    }
}

pub async fn get_rpc_client(chain: &str) -> Result<SolanaProvider, String> {
    let url = if chain == "sol" {
        "https://api.mainnet-beta.solana.com".to_string()
    } else {
        "https://api.devnet.solana.com".to_string()
    };
    Ok(SolanaProvider::new(url))
}
