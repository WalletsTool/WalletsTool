import base_coin_balance from "@/scripts/balance/base_coin_balance.js";
import token_balance from "@/scripts/balance/token_balance.js";
import {ethers} from "ethers";

let chain = ''
let type = ''
let contract_address = ''
let contract_abi = []
let group = []

// 迭代执行
function recurrence(index) {
    iter_query(group[index ?? 0]).then(() => {
        if (index < group.length - 1) {
            recurrence(index + 1)
        } else {
            console.log('查询完成')
        }
    })
}

// 小组内遍历查询
function iter_query(items) {
    return new Promise((resolve, reject) => {
        let list = []
        if (type === 'base') {
            items.forEach(async obj => {
                list.push(base_coin_balance.query_balance(obj, chain))
            })
        } else if (type === 'token') {
            const contract = new ethers.Contract(currentCoin.value.contract_address, currentCoin.value.abi);
            items.forEach(async obj => {
                list.push(base_coin_balance.query_balance(obj, chain))
                list.push(token_balance.query_balance(obj, chain, contract))
            })
        }
        Promise.all(list).then(() => {
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
    exec_group_query(key, currentCoin, data) {
        chain = key
        type = currentCoin.type
        contract_address = currentCoin.contract_address
        contract_abi = currentCoin.abi
        // 分组
        group = divide_into_groups(data, 3)
        // 迭代查询
        recurrence()
    }
}

export default balance_utils
