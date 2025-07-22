use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::transfer::{
    config::*,
    base_coin::BaseCoinTransfer,
    token::TokenTransfer,
    utils::TransferUtils,
    provider::ProviderManager,
};

pub struct TransferManager {
    base_coin_transfer: BaseCoinTransfer,
    token_transfer: TokenTransfer,
    provider_manager: ProviderManager,
    stop_flag: Arc<Mutex<bool>>,
}

impl TransferManager {
    pub fn new() -> Self {
        Self {
            base_coin_transfer: BaseCoinTransfer::new(),
            token_transfer: TokenTransfer::new(),
            provider_manager: ProviderManager::new(),
            stop_flag: Arc::new(Mutex::new(false)),
        }
    }

    /// 执行批量转账
    pub async fn execute_batch_transfer(
        &self,
        mut items: Vec<TransferItem>,
        config: TransferConfig,
        coin_config: Option<CoinConfig>, // None 表示基础币种转账
    ) -> Result<Vec<TransferItem>, Box<dyn std::error::Error + Send + Sync>> {
        // 验证配置
        TransferUtils::validate_config(&config)?;

        // 重置所有项目的状态
        for item in &mut items {
            item.exec_status = ExecStatus::Waiting;
            item.error_msg = String::new();
            item.retry_flag = false;
            item.error_count = 0;
        }

        // 执行转账
        self.iter_transfer(items, &config, coin_config.as_ref()).await
    }

    /// 迭代执行转账（支持重试）
    async fn iter_transfer(
        &self,
        mut items: Vec<TransferItem>,
        config: &TransferConfig,
        coin_config: Option<&CoinConfig>,
    ) -> Result<Vec<TransferItem>, Box<dyn std::error::Error + Send + Sync>> {
        let total_count = items.len();
        
        for (index, item) in items.iter_mut().enumerate() {
            // 检查停止标志
            if *self.stop_flag.lock().await {
                println!("转账已停止");
                break;
            }

            // 执行单个转账
            match self.single_transfer(index, item, config, coin_config).await {
                Ok(tx_hash) => {
                    item.exec_status = ExecStatus::Success;
                    item.error_msg = tx_hash;
                    println!("序号：{} 转账成功", index + 1);
                }
                Err(err) => {
                    if err.to_string().contains("base gas price 超出最大值限制") {
                        // Gas price 超限，停止执行
                        item.exec_status = ExecStatus::Waiting;
                        item.error_msg = String::new();
                        println!("Gas price 超出限制，停止执行");
                        break;
                    } else {
                        item.exec_status = ExecStatus::Failed;
                        item.error_msg = err.to_string();
                        
                        // 设置重试标志
                        if config.error_retry && item.error_count < config.error_count_limit {
                            item.error_count += 1;
                            item.retry_flag = true;
                            println!("序号：{} 转账失败，将重试。错误：{}", index + 1, err);
                        } else {
                            println!("序号：{} 转账失败。错误：{}", index + 1, err);
                        }
                    }
                }
            }

            // 添加延迟
            if index < total_count - 1 {
                TransferUtils::sleep(config.delay).await;
            }
        }

        // 处理重试
        if config.error_retry {
            let retry_items: Vec<_> = items.iter().filter(|item| item.retry_flag).collect();
            if !retry_items.is_empty() && !*self.stop_flag.lock().await {
                println!("开始重试失败的交易，数量：{}", retry_items.len());
                let retry_items_owned: Vec<TransferItem> = retry_items.into_iter().cloned().collect();
                return self.iter_transfer(retry_items_owned, config, coin_config).await;
            }
        }

        Ok(items)
    }

    /// 执行单个转账
    async fn single_transfer(
        &self,
        index: usize,
        item: &mut TransferItem,
        config: &TransferConfig,
        coin_config: Option<&CoinConfig>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match coin_config {
            None => {
                // 基础币种转账
                self.base_coin_transfer.single_transfer(index, item, config).await
            }
            Some(coin) => {
                // 代币转账
                if let Some(contract_address) = &coin.contract_address {
                    self.token_transfer
                        .single_transfer(index, item, config, contract_address)
                        .await
                } else {
                    Err("代币合约地址未配置".into())
                }
            }
        }
    }

    /// 停止转账
    pub async fn stop_transfer(&self) {
        let mut stop_flag = self.stop_flag.lock().await;
        *stop_flag = true;
    }

    /// 重置停止标志
    pub async fn reset_stop_flag(&self) {
        let mut stop_flag = self.stop_flag.lock().await;
        *stop_flag = false;
    }

    /// 获取当前 Gas 价格
    pub async fn get_gas_price(&self, chain: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        self.provider_manager.get_gas_price(chain).await
    }

    /// 验证转账数据
    pub fn validate_transfer_data(&self, items: &[TransferItem]) -> Result<(), String> {
        if items.is_empty() {
            return Err("转账数据不能为空".to_string());
        }

        for (index, item) in items.iter().enumerate() {
            // 验证私钥
            if !TransferUtils::is_valid_private_key(&item.private_key) {
                return Err(format!("序号 {} 的私钥格式不正确", index + 1));
            }

            // 验证接收地址
            if !TransferUtils::is_valid_address(&item.to_addr) {
                return Err(format!("序号 {} 的接收地址格式不正确", index + 1));
            }

            // 验证转账金额（如果指定）
            if let Some(amount) = &item.amount {
                if !TransferUtils::check_num(amount) {
                    return Err(format!("序号 {} 的转账金额格式不正确", index + 1));
                }
            }
        }

        Ok(())
    }

    /// 获取转账统计信息
    pub fn get_transfer_statistics(&self, items: &[TransferItem]) -> TransferStatistics {
        let total = items.len();
        let waiting = items.iter().filter(|item| matches!(item.exec_status, ExecStatus::Waiting)).count();
        let executing = items.iter().filter(|item| matches!(item.exec_status, ExecStatus::Executing)).count();
        let success = items.iter().filter(|item| matches!(item.exec_status, ExecStatus::Success)).count();
        let failed = items.iter().filter(|item| matches!(item.exec_status, ExecStatus::Failed)).count();

        TransferStatistics {
            total,
            waiting,
            executing,
            success,
            failed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransferStatistics {
    pub total: usize,
    pub waiting: usize,
    pub executing: usize,
    pub success: usize,
    pub failed: usize,
}

impl Default for TransferManager {
    fn default() -> Self {
        Self::new()
    }
}
