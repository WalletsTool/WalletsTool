use std::fs;
use tauri::command;
use serde_json::{json, Value};

use crate::utils::{chain_config_path, coin_config_path, read_json};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
pub fn get_chain_list() -> Vec<Value> {
    let path = chain_config_path();
    let content = fs::read_to_string(path).unwrap();
    let setting_json = read_json(&content).unwrap_or_else(|_| json!({}));
    let binding = Vec::new();
    let chain_list = setting_json["chain_list"].as_array().unwrap_or(&binding);
    chain_list.to_owned()
}

#[command]
pub fn get_coin_list(chain: &str) -> Vec<Value> {
    let path = coin_config_path(chain);
    let content = fs::read_to_string(path).unwrap();
    let setting_json = read_json(&content).unwrap_or_else(|_| json!({}));
    let binding = Vec::new();
    let coin_list = setting_json["coin_list"].as_array().unwrap_or(&binding);
    coin_list.to_owned()
}

#[command]
pub fn add_coin(chain: &str) -> Vec<Value> {
    let path = coin_config_path(chain);
    let content = fs::read_to_string(path).unwrap();
    let setting_json = read_json(&content).unwrap_or_else(|_| json!({}));
    let binding = Vec::new();
    let coin_list = setting_json["coin_list"].as_array().unwrap_or(&binding);
    coin_list.to_owned()
}
