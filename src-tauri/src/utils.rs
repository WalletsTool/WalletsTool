use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{self};

pub fn exists(path: &Path) -> bool {
    Path::new(path).exists()
}

pub fn chain_config_path() -> PathBuf {
    Path::new(format!("conf/chain_setting.json").as_str()).to_owned()
}

pub fn coin_config_path(chain: &str,page:&str) -> PathBuf {
    Path::new(format!("conf/{}/coin_{}_setting.json", page, chain).as_str()).to_owned()
}

pub fn read_json(content: &str) -> serde_json::Result<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_str(content)?;
    Ok(v)
}
