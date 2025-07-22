use rand::Rng;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

pub struct TransferUtils;

impl TransferUtils {
    /// 检查字符串是否为数字
    pub fn check_num(num: &str) -> bool {
        if num.is_empty() {
            return false;
        }
        
        // 正则表达式：匹配数字（可带小数点）
        let regex = regex::Regex::new(r"^[0-9]+\.?[0-9]*$").unwrap();
        regex.is_match(num)
    }

    /// 检查字符串是否为正整数
    pub fn check_positive_integer(num: &str) -> bool {
        if num.is_empty() {
            return false;
        }
        
        // 正则表达式：匹配正整数
        let regex = regex::Regex::new(r"^[1-9]+[0-9]*$").unwrap();
        regex.is_match(num)
    }

    /// 随机延迟
    pub async fn sleep(delay: [u64; 2]) {
        let mut rng = rand::thread_rng();
        let sleep_duration = rng.gen_range(delay[0]..=delay[1]);
        
        println!("延迟: {} 秒", sleep_duration);
        sleep(Duration::from_secs(sleep_duration)).await;
    }

    /// 生成随机数量
    pub fn generate_random_amount(min: f64, max: f64, precision: u8) -> String {
        let mut rng = rand::thread_rng();
        let amount = rng.gen_range(min..=max);
        format!("{:.precision$}", amount, precision = precision as usize)
    }

    /// 验证地址格式
    pub fn is_valid_address(address: &str) -> bool {
        // 以太坊地址格式：0x开头，42位十六进制
        if address.len() != 42 {
            return false;
        }
        
        if !address.starts_with("0x") {
            return false;
        }
        
        // 检查是否为有效的十六进制
        address[2..].chars().all(|c| c.is_ascii_hexdigit())
    }

    /// 验证私钥格式
    pub fn is_valid_private_key(private_key: &str) -> bool {
        // 私钥格式：64位十六进制（可能带0x前缀）
        let key = if private_key.starts_with("0x") {
            &private_key[2..]
        } else {
            private_key
        };
        
        if key.len() != 64 {
            return false;
        }
        
        key.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// 转换 Gwei 到 Wei
    pub fn gwei_to_wei(gwei: f64) -> u128 {
        (gwei * 1_000_000_000.0) as u128
    }

    /// 转换 Wei 到 Gwei
    pub fn wei_to_gwei(wei: u128) -> f64 {
        wei as f64 / 1_000_000_000.0
    }

    /// 转换 Wei 到 Ether
    pub fn wei_to_ether(wei: u128) -> f64 {
        wei as f64 / 1_000_000_000_000_000_000.0
    }

    /// 转换 Ether 到 Wei
    pub fn ether_to_wei(ether: f64) -> u128 {
        (ether * 1_000_000_000_000_000_000.0) as u128
    }

    /// 格式化余额显示
    pub fn format_balance(balance: f64, precision: usize) -> String {
        format!("{:.precision$}", balance, precision = precision)
    }

    /// 验证转账配置
    pub fn validate_config(config: &crate::transfer::config::TransferConfig) -> Result<(), String> {
        // 验证链名称
        if config.chain.is_empty() {
            return Err("链名称不能为空".to_string());
        }

        // 验证延迟设置
        if config.delay[0] > config.delay[1] {
            return Err("最小延迟不能大于最大延迟".to_string());
        }

        // 验证转账类型相关配置
        match config.transfer_type {
            crate::transfer::config::TransferType::Fixed => {
                if config.transfer_amount.is_none() {
                    return Err("固定转账模式必须设置转账数量".to_string());
                }
                if let Some(amount) = &config.transfer_amount {
                    if !Self::check_num(amount) {
                        return Err("转账数量必须为有效数字".to_string());
                    }
                }
            }
            crate::transfer::config::TransferType::Random => {
                if config.transfer_amount_list.is_none() {
                    return Err("随机转账模式必须设置数量范围".to_string());
                }
                if let Some([min, max]) = &config.transfer_amount_list {
                    if !Self::check_num(min) || !Self::check_num(max) {
                        return Err("转账数量范围必须为有效数字".to_string());
                    }
                    if min.parse::<f64>().unwrap() >= max.parse::<f64>().unwrap() {
                        return Err("最小转账数量不能大于等于最大转账数量".to_string());
                    }
                }
            }
            crate::transfer::config::TransferType::RemainRandom => {
                if config.left_amount_list.is_none() {
                    return Err("剩余随机模式必须设置剩余数量范围".to_string());
                }
                if let Some([min, max]) = &config.left_amount_list {
                    if !Self::check_num(min) || !Self::check_num(max) {
                        return Err("剩余数量范围必须为有效数字".to_string());
                    }
                    if min.parse::<f64>().unwrap() >= max.parse::<f64>().unwrap() {
                        return Err("最小剩余数量不能大于等于最大剩余数量".to_string());
                    }
                }
            }
            _ => {}
        }

        // 验证 Gas 价格设置
        if let crate::transfer::config::GasPriceType::Fixed = config.gas_price_type {
            if config.gas_price.is_none() {
                return Err("固定 Gas 价格模式必须设置 Gas 价格".to_string());
            }
            if let Some(price) = &config.gas_price {
                if !Self::check_num(price) {
                    return Err("Gas 价格必须为有效数字".to_string());
                }
            }
        }

        // 验证 Gas 限制设置
        if let crate::transfer::config::GasLimitType::Fixed = config.limit_type {
            if config.limit_count.is_none() {
                return Err("固定 Gas 限制模式必须设置 Gas 限制".to_string());
            }
        }

        if let crate::transfer::config::GasLimitType::Random = config.limit_type {
            if config.limit_count_list.is_none() {
                return Err("随机 Gas 限制模式必须设置 Gas 限制范围".to_string());
            }
            if let Some([min, max]) = config.limit_count_list {
                if min >= max {
                    return Err("最小 Gas 限制不能大于等于最大 Gas 限制".to_string());
                }
            }
        }

        Ok(())
    }
}
