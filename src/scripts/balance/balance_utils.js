// 余额查询工具类 - 仅使用 Rust 后端

import { invoke } from '@tauri-apps/api/core';
import starknet_balance from "@/scripts/balance/starknet_balance.js";

class BalanceUtils {
    constructor() {
        // 仅保留必要的属性
    }

    // 销毁方法（保留接口兼容性）
    async destroyPool() {
        console.log('清理完成');
    }

    // 执行余额查询 - 仅使用 Rust 后端
    async exec_group_query(key, currentCoin, data, onlyCoinConfig, callback, threadCount = 3) {
        try {
            // 特殊处理StarkNet
            if (key === 'starknet' && currentCoin.coin_type === 'token') {
                return this.execStarknetQuery(data, currentCoin, callback);
            }
            
            // 使用 Rust 后端查询
            console.log(`使用 Rust 后端查询，线程数: ${threadCount}`);
            return this.execRustBackendQuery(key, currentCoin, data, onlyCoinConfig, callback, threadCount);
            
        } catch (error) {
            console.error('执行查询失败:', error);
            // 设置所有项目为失败状态
            data.forEach(item => {
                item.exec_status = '3';
                item.error_msg = '查询失败！';
            });
            callback();
        }
    }

    // 使用 Rust 后端执行查询
    async execRustBackendQuery(key, currentCoin, data, onlyCoinConfig, callback, threadCount) {
        try {
            // 格式化参数
            const params = {
                chain: key,
                coin_config: {
                    coin_type: currentCoin.coin_type,
                    contract_address: currentCoin.contract_address || null,
                    abi: currentCoin.abi || null
                },
                items: data.map(item => ({
                    address: item.address || null,
                    private_key: item.private_key || null,
                    plat_balance: null,
                    coin_balance: null,
                    nonce: null,
                    exec_status: '0',
                    error_msg: null
                })),
                only_coin_config: onlyCoinConfig,
                thread_count: threadCount
            };
            
            // 调用 Rust 后端
            const result = await invoke('query_balances_simple', { params });
            
            // 将结果合并回原始数据
            if (result.success && result.items) {
                result.items.forEach((rustItem, index) => {
                    if (data[index]) {
                        Object.assign(data[index], rustItem);
                    }
                });
            } else {
                // 如果查询失败，设置所有项目为失败状态
                data.forEach(item => {
                    item.exec_status = '3';
                    item.error_msg = result.error_msg || '查询失败！';
                });
            }
            
            // 执行回调
            callback();
            
        } catch (error) {
            console.error('Rust 后端查询失败:', error);
            // 设置所有项目为失败状态
            data.forEach(item => {
                item.exec_status = '3';
                item.error_msg = '查询失败！';
            });
            callback();
        }
    }
    
    // 处理StarkNet查询（保持原有逻辑）
    async execStarknetQuery(data, currentCoin, callback) {
        try {
            const tasks = data.map(item => {
                item.exec_status = '1';
                item.error_msg = '';
                return starknet_balance.query_balance_by_address(item, currentCoin.contract_address, currentCoin.abi);
            });
            
            await Promise.all(tasks);
            
            data.forEach(item => {
                item.exec_status = item.error_msg ? '3' : '2';
            });
            
            callback();
        } catch (error) {
            console.error('StarkNet查询失败:', error);
            data.forEach(item => {
                item.exec_status = '3';
                item.error_msg = '查询失败！';
            });
            callback();
        }
    }
}

// 创建单例实例
const balance_utils = new BalanceUtils();

export default balance_utils;
