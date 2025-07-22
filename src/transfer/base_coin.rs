use std::str::FromStr;
use std::sync::Arc;

use ethers::{
    prelude::*,
    providers::Provider,
    signers::{LocalWallet, Signer},
    types::{transaction::eip2718::TypedTransaction, Address, TransactionRequest, U256},
    utils::{parse_ether, format_ether},
};
use rand::Rng;
use tokio::time::{sleep, Duration};

use crate::transfer::{
    config::*,
    provider::{EthProvider, GasCalculator, ProviderManager},
    utils::TransferUtils,
};

pub struct BaseCoinTransfer {
    provider_manager: ProviderManager,
}

impl BaseCoinTransfer {
    pub fn new() -> Self {
        Self {
            provider_manager: ProviderManager::new(),
        }
    }

    pub async fn single_transfer(
        &self,
        index: usize,
        item: &mut TransferItem,
        config: &TransferConfig,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        item.retry_flag = false;
        item.exec_status = ExecStatus::Executing;

        let provider = self.provider_manager.get_provider(&config.chain)?;
        
        // 创建钱包
        let wallet: LocalWallet = config.chain.parse::<LocalWallet>()?.with_chain_id(1u64); // 这里需要根据链设置正确的 chain_id
        let wallet = wallet.clone();
        let from_address = wallet.address();

        // 获取余额
        let balance = provider.get_balance(from_address, None).await?;
        let balance_ether = format_ether(balance);
        
        println!("序号：{}, 当前余额为: {} ETH", index + 1, balance_ether);

        if balance.is_zero() {
            return Err("当前余额不足，不做转账操作！".into());
        }

        // 计算转账金额
        let transfer_amount = self.calculate_transfer_amount(&config, &balance, &balance_ether)?;
        
        println!("序号：{}, 转账数量为: {} ETH", index + 1, format_ether(transfer_amount));

        // 计算 Gas
        let to_address = Address::from_str(&item.to_addr)?;
        let gas_price = GasCalculator::calculate_gas_price(provider.clone(), config).await?;
        let gas_limit = GasCalculator::calculate_gas_limit(
            provider.clone(),
            config,
            from_address,
            to_address,
            Some(transfer_amount),
        ).await?;

        let gas_fee = gas_price * gas_limit;
        let gas_fee_ether = format_ether(gas_fee);

        println!("序号：{}, 当前预估 gas_fee 为: {} ETH", index + 1, gas_fee_ether);

        // 构造交易
        let tx = TransactionRequest::new()
            .from(from_address)
            .to(to_address)
            .value(transfer_amount)
            .gas_price(gas_price)
            .gas(gas_limit);

        item.error_msg = "发送交易...".to_string();

        // 发送交易
        let signed_tx = wallet.sign_transaction(&TypedTransaction::Legacy(tx)).await?;
        let tx_hash = provider.send_raw_transaction(signed_tx.rlp()).await?;

        println!("序号：{}, 交易 hash 为：{:?}", index + 1, tx_hash);
        item.error_msg = "等待交易结果...".to_string();

        // 等待交易确认
        let receipt = provider
            .pending_transaction(tx_hash)
            .confirmations(1)
            .await?
            .ok_or("Transaction failed")?;

        if receipt.status == Some(U64::from(1)) {
            // 添加延迟
            TransferUtils::sleep(config.delay).await;
            Ok(format!("{:?}", tx_hash))
        } else {
            Err(format!("交易失败：{:?}", receipt).into())
        }
    }

    fn calculate_transfer_amount(
        &self,
        config: &TransferConfig,
        balance: &U256,
        balance_ether: &str,
    ) -> Result<U256, Box<dyn std::error::Error + Send + Sync>> {
        match &config.transfer_type {
            TransferType::All => {
                // 全部转账，需要预留 gas 费用
                // 这里简化处理，实际应该根据 gas 费用计算
                let gas_reserve = parse_ether("0.001")?; // 预留 0.001 ETH 作为 gas
                if *balance <= gas_reserve {
                    return Err("当前余额不足以支付 gas 费用".into());
                }
                Ok(*balance - gas_reserve)
            }
            TransferType::Fixed => {
                if let Some(amount) = &config.transfer_amount {
                    let transfer_amount = parse_ether(amount)?;
                    if transfer_amount >= *balance {
                        return Err("当前余额不足，不做转账操作！".into());
                    }
                    Ok(transfer_amount)
                } else {
                    Err("未设置固定转账数量".into())
                }
            }
            TransferType::Random => {
                if let Some([min, max]) = &config.transfer_amount_list {
                    let min_amount: f64 = min.parse()?;
                    let max_amount: f64 = max.parse()?;
                    
                    let mut rng = rand::thread_rng();
                    let random_amount = rng.gen_range(min_amount..=max_amount);
                    let precision = config.amount_precision as usize;
                    let formatted_amount = format!("{:.precision$}", random_amount, precision = precision);
                    
                    let transfer_amount = parse_ether(&formatted_amount)?;
                    
                    if transfer_amount >= *balance {
                        return Err("当前余额不足，不做转账操作！".into());
                    }
                    
                    Ok(transfer_amount)
                } else {
                    Err("未设置随机转账数量范围".into())
                }
            }
            TransferType::RemainRandom => {
                if let Some([min, max]) = &config.left_amount_list {
                    let balance_f64: f64 = balance_ether.parse()?;
                    let min_remain: f64 = min.parse()?;
                    let max_remain: f64 = max.parse()?;
                    
                    if balance_f64 >= min_remain && balance_f64 <= max_remain {
                        return Err(format!(
                            "当前余额为：{} 在设置的剩余范围内，不做转账操作！", 
                            balance_ether
                        ).into());
                    }
                    
                    let mut rng = rand::thread_rng();
                    let remain_amount = rng.gen_range(min_remain..=max_remain);
                    let precision = config.amount_precision as usize;
                    let transfer_amount_f64 = balance_f64 - remain_amount;
                    
                    if transfer_amount_f64 <= 0.0 {
                        return Err("当前余额不足，不做转账操作！".into());
                    }
                    
                    let formatted_amount = format!("{:.precision$}", transfer_amount_f64, precision = precision);
                    let transfer_amount = parse_ether(&formatted_amount)?;
                    
                    Ok(transfer_amount)
                } else {
                    Err("未设置剩余数量范围".into())
                }
            }
        }
    }
}

impl Default for BaseCoinTransfer {
    fn default() -> Self {
        Self::new()
    }
}
