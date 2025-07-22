use std::path::{Path, PathBuf};

use serde_json::{self};

// pub fn exists(path: &Path) -> bool {
//     Path::new(path).exists()
// }

#[allow(dead_code)]
pub fn chain_config_path() -> PathBuf {
    Path::new("conf/chain_setting.json".to_string().as_str()).to_owned()
}

#[allow(dead_code)]
pub fn coin_config_path(chain: &str) -> PathBuf {
    Path::new(format!("conf/chains/coin_{}_setting.json", chain).as_str()).to_owned()
}

#[allow(dead_code)]
pub fn read_json(content: &str) -> serde_json::Result<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_str(content)?;
    Ok(v)
}
