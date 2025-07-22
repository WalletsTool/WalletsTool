use std::sync::Arc;
use ethers::{
    prelude::*,
    providers::Provider,
    signers::{LocalWallet, Signer},
    types::{Address, U256},
    utils::parse_units,
};
use rand::Rng;

use crate::transfer::{
    config::*,
    provider::{EthProvider, GasCalculator, ProviderManager},
};

pub struct TokenTransfer {
    provider_manager: ProviderManager,
}

impl TokenTransfer {
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
        contract_address: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        item.retry_flag = false;
        item.exec_status = ExecStatus::Executing;

        let provider = self.provider_manager.get_provider(&config.chain)?;
        
        let wallet: LocalWallet = config.chain.parse::<LocalWallet>()?.with_chain_id(1u64); // 需要根据实际链条设置
        let wallet = wallet.clone();
        let from_address = wallet.address();
        let contract_address = Address::from_str(contract_address)?;
        
        // 创建合约实例
        let contract = Contract::new(contract_address, include_bytes!("../../abi/erc20.json"), wallet.clone())?;
        let decimals: u8 = contract.method("decimals", ())?.call().await?;

        // 获取代币余额
        let balance: U256 = contract.method("balanceOf", from_address)?.call().await?;
        let balance_formatted = balance.as_u128() as f64 / (10_u64.pow(decimals.into()) as f64);

        println!("序号：{}, 当前余额为: {} tokens", index + 1, balance_formatted);
        
        if balance.is_zero() {
            return Err("当前余额不足，不做转账操作！".into());
        }

        // 计算转账数量
        let transfer_amount = self.calculate_transfer_amount(&config, balance_formatted)?;
        let transfer_amount_u256 = parse_units(transfer_amount, decimals)?;
        
        println!("序号：{}, 转账数量为: {} tokens", index + 1, transfer_amount);

        // 计算 Gas
        let to_address = Address::from_str(&item.to_addr)?;
        let gas_price = GasCalculator::calculate_gas_price(provider.clone(), config).await?;
        let gas_limit = GasCalculator::calculate_gas_limit(
            provider.clone(),
            config,
            from_address,
            to_address,
            Some(transfer_amount_u256),
        ).await?;

        // 执行交易
        item.error_msg = "发送交易...".to_string();
        let tx = contract
            .method("transfer", (to_address, transfer_amount_u256))?
            .gas(gas_limit)
            .gas_price(gas_price)
            .send()
            .await?;

        println!("序号：{}, 交易 hash 为：{:?}", index + 1, tx);
        item.error_msg = "等待交易结果...".to_string();

        let receipt = provider
            .pending_transaction(tx)
            .confirmations(1)
            .await?
            .ok_or("Transaction failed")?;

        if receipt.status == Some(1) {
            // 添加延迟
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            Ok(format!("{:?}", tx))
        } else {
            Err(format!("交易失败：{:?}", receipt).into())
        }
    }

    fn calculate_transfer_amount(
        &self,
        config: &TransferConfig,
        balance: f64,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        match &config.transfer_type {
            TransferType::All => {
                let gas_reserve = 0.001; // 预留 gas
                if balance <= gas_reserve {
                    return Err("当前余额不足以支付 gas 费用".into());
                }
                Ok(balance - gas_reserve)
            }
            TransferType::Fixed => {
                if let Some(amount_str) = &config.transfer_amount {
                    let transfer_amount: f64 = amount_str.parse()?;
                    if transfer_amount >= balance {
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

                    if random_amount >= balance {
                        return Err("当前余额不足，不做转账操作！".into());
                    }

                    Ok(random_amount)
                } else {
                    Err("未设置随机转账数量范围".into())
                }
            }
            TransferType::RemainRandom => {
                if let Some([min, max]) = &config.left_amount_list {
                    let min_remain: f64 = min.parse()?;
                    let max_remain: f64 = max.parse()?;

                    if balance >= min_remain && balance <= max_remain {
                        return Err("当前余额在设定范围内，不做转账操作！".into());
                    }

                    let mut rng = rand::thread_rng();
                    let remain = rng.gen_range(min_remain..=max_remain);

                    if balance <= remain {
                        return Err("当前余额不足以支付费用".into());
                    }

                    Ok(balance - remain)
                } else {
                    Err("未设置剩余数量范围".into())
                }
            }
        }
    }
}

impl Default for TokenTransfer {
    fn default() -> Self {
        Self::new()
    }
}
