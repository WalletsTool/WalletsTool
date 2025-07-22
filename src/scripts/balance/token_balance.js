import {utils} from "@/scripts/common/provider.js";
import {ethers, Wallet} from "ethers";
const {formatUnits} = ethers.utils;

const token_balance = {

    // 通过private_key查询余额方法
    query_balance(item, chain, contract) {
        return new Promise((resolve, reject) => {
            // 随机获取rpc服务
            const provider = utils.get_provider(chain)
            console.log('当前RPC地址：',provider.connection.url)
            // 通过私钥创建钱包
            let wallet = new Wallet(item.private_key, provider);
            let balance_wei = contract.connect(wallet).balanceOf(wallet.address);
            let decimals = contract.connect(wallet).decimals();
            Promise.all([balance_wei, decimals]).then(async (values) => {
                item.coin_balance = parseFloat(formatUnits(values[0], values[1])).toFixed(4).toString()
                resolve()
            }).catch((err) => {
                item.coin_balance = ''
                item.error_msg = '查询代币余额失败！'
                console.log('查询代币余额失败！', err)
                resolve('')
            })
        })
    },
    // 通过address查询余额方法
    query_balance_by_address(item, chain, contract) {
        return new Promise((resolve, reject) => {
            // 随机获取rpc服务
            const provider = utils.get_provider(chain)
            console.log('当前RPC地址：',provider.connection.url)
            let balance_wei = contract.connect(provider).balanceOf(item.address)
            let decimals = contract.connect(provider).decimals();
            let nonce = provider.getTransactionCount(item.address)
            Promise.all([balance_wei, decimals, nonce]).then(async (values) => {
                item.coin_balance = parseFloat(formatUnits(values[0], values[1])).toFixed(4).toString()
                item.nonce = values[2]
                resolve()
            }).catch((err) => {
                item.nonce = ''
                item.coin_balance = ''
                item.error_msg = '查询代币余额失败！'
                console.log('查询代币余额失败！', err)
                resolve('')
            })
        })
    }
}

export default token_balance
