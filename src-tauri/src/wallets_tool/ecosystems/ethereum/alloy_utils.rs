use alloy_primitives::U256;

pub fn parse_ether_to_wei_f64(ether: f64) -> Result<U256, String> {
    let wei = (ether * 1e18) as u128;
    Ok(U256::from(wei))
}

pub fn parse_gwei_to_wei(gwei: f64) -> U256 {
    let wei = (gwei * 1e9) as u128;
    U256::from(wei)
}

pub fn format_wei_to_ether(wei: U256) -> String {
    let wei_str = wei.to_string();
    let wei_u128: u128 = wei_str.parse().unwrap_or(0);
    let ether = wei_u128 as f64 / 1e18;
    format!("{:.6}", ether)
}

pub fn format_wei_to_gwei(wei: U256) -> String {
    let wei_str = wei.to_string();
    let wei_u128: u128 = wei_str.parse().unwrap_or(0);
    let gwei = wei_u128 as f64 / 1e9;
    format!("{:.2}", gwei)
}

pub fn u256_to_f64(wei: U256) -> f64 {
    let wei_str = wei.to_string();
    wei_str.parse::<f64>().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ether_to_wei_f64() {
        assert_eq!(parse_ether_to_wei_f64(1.0).unwrap(), U256::from(1000000000000000000u128));
        assert_eq!(parse_ether_to_wei_f64(0.5).unwrap(), U256::from(500000000000000000u128));
    }

    #[test]
    fn test_parse_gwei_to_wei() {
        assert_eq!(parse_gwei_to_wei(1.0), U256::from(1000000000u128));
        assert_eq!(parse_gwei_to_wei(0.5), U256::from(500000000u128));
    }

    #[test]
    fn test_format_wei_to_ether() {
        let wei = U256::from(1000000000000000000u128);
        assert_eq!(format_wei_to_ether(wei), "1.000000");
        
        let wei = U256::from(500000000000000000u128);
        assert_eq!(format_wei_to_ether(wei), "0.500000");
    }

    #[test]
    fn test_format_wei_to_gwei() {
        let wei = U256::from(1000000000u128);
        assert_eq!(format_wei_to_gwei(wei), "1.00");
        
        let wei = U256::from(500000000u128);
        assert_eq!(format_wei_to_gwei(wei), "0.50");
    }
}
