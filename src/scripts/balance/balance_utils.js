import base_coin_balance from "@/scripts/balance/base_coin_balance.js";
import token_balance from "@/scripts/balance/token_balance.js";
import starknet_balance from "@/scripts/balance/starknet_balance.js";
import {ethers} from "ethers";

let chain = ''
let onlyCoin = true
let type = ''
let contract_address = ''
let contract_abi = []
let group = []

// 迭代执行
function recurrence(callback, index) {
    balance_utils.progress = (index/group.length).toFixed(4)
    iter_query(group[index]).then(() => {
        if (index < group.length - 1) {
            recurrence(callback, index + 1)
        } else {
            callback()
        }
    })
}

// 小组内遍历查询
function iter_query(items) {
    return new Promise((resolve, reject) => {
        let list = []
        if (type === 'base') {
            items.forEach(async obj => {
                obj.exec_status = '1'
                obj.error_msg = ''
                if (obj.private_key) {
                    list.push(base_coin_balance.query_balance(obj, chain))
                } else {
                    list.push(base_coin_balance.query_balance_by_address(obj, chain))
                }

            })
        } else if (type === 'token') {
            if (chain === 'starknet') {
                items.forEach(async obj => {
                    obj.exec_status = '1'
                    obj.error_msg = ''
                    list.push(starknet_balance.query_balance_by_address(obj, contract_address, contract_abi))
                })
            } else {
                const contract = new ethers.Contract(contract_address, contract_abi);
                items.forEach(async obj => {
                    obj.exec_status = '1'
                    obj.error_msg = ''
                    if (obj.private_key) {
                        if (!onlyCoin) {
                            list.push(base_coin_balance.query_balance(obj, chain))
                        }
                        list.push(token_balance.query_balance(obj, chain, contract))
                    } else {
                        if (!onlyCoin) {
                            list.push(base_coin_balance.query_balance_by_address(obj, chain))
                        }
                        list.push(token_balance.query_balance_by_address(obj, chain, contract))
                    }
                })
            }
        }
        Promise.all(list).then(() => {
            items.forEach(obj => {
                obj.exec_status = obj.error_msg ? '3' : '2'
            })
            resolve()
        })
    })
}

// 数据分组方法
function divide_into_groups(array, subGroupLength) {
    let index = 0;
    let newArray = [];

    while (index < array.length) {
        newArray.push(array.slice(index, index += subGroupLength));
    }

    return newArray;
}

const balance_utils = {
    // 执行分组查询
    exec_group_query(key, currentCoin, data, onlyCoinConfig, callback, subGroupLength = 3) {
        chain = key
        onlyCoin = onlyCoinConfig
        type = currentCoin.type
        contract_address = currentCoin.contract_address
        contract_abi = currentCoin.abi
        // 分组
        group = divide_into_groups(data, subGroupLength)
        // 迭代查询
        recurrence(callback, 0)
    },
    progress: 0
}

export default balance_utils
