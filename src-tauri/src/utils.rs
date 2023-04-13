use std::path::{Path, PathBuf};

use serde_json::{self};
use anyhow::Result;

pub fn exists(path: &Path) -> bool {
    Path::new(path).exists()
}

pub fn chain_config_path() -> PathBuf {
    Path::new(format!("{}/conf/chain_setting.json", env!("CARGO_MANIFEST_DIR")).as_str()).to_owned()
}

pub fn coin_config_path(chain:&str) -> PathBuf {
    Path::new(format!("{}/conf/coin_{}_setting.json", env!("CARGO_MANIFEST_DIR"),chain).as_str()).to_owned()
}

pub fn read_json(content: &str) -> serde_json::Result<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_str(content)?;
    Ok(v)
}
