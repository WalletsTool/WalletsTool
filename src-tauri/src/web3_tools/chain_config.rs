use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Index;

use serde_json::{json, Value};
use tauri::command;

use crate::utils::{chain_config_path, coin_config_path, read_json};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
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
pub fn get_coin_list(chain: &str, page: &str) -> Vec<Value> {
    let path = coin_config_path(chain, page);
    let content = fs::read_to_string(path).unwrap();
    let setting_json = read_json(&content).unwrap_or_else(|_| json!({}));
    let binding = Vec::new();
    let coin_list = setting_json["coin_list"].as_array().unwrap_or(&binding);
    coin_list.to_owned()
}

#[command]
pub fn add_coin(chain: &str, page: &str, obj_json: &str) -> () {
    let path = coin_config_path(chain, page);
    let content = fs::read_to_string(path.clone()).unwrap();
    let mut setting_json = read_json(&content).unwrap_or_else(|_| json!({}));
    let binding = Vec::new();
    let mut coin_list = setting_json["coin_list"].as_array().unwrap_or(&binding).to_owned();
    coin_list.push(read_json(obj_json).unwrap());
    setting_json["coin_list"] = json!(coin_list);
    // 写入文件
    let mut file = File::create(path.to_str().unwrap()).expect("create failed");
    file.write_all(setting_json.to_string().as_bytes()).expect("write failed");
}

#[command]
pub fn remove_coin(chain: &str, page: &str, key: &str) -> () {
    let path = coin_config_path(chain, page);
    let content = fs::read_to_string(path.clone()).unwrap();
    let mut setting_json = read_json(&content).unwrap_or_else(|_| json!({}));
    let binding = Vec::new();
    let mut coin_list = setting_json["coin_list"].as_array().unwrap_or(&binding).to_owned();
    let coin_list_new = coin_list.iter().filter(|x| x["key"] != key).collect::<Vec<&Value>>();
    setting_json["coin_list"] = json!(coin_list_new);
    // 写入文件
    let mut file = File::create(path.to_str().unwrap()).expect("create failed");
    file.write_all(setting_json.to_string().as_bytes()).expect("write failed");
}
