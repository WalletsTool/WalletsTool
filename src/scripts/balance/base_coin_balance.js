import utils from "@/scripts/transfer/transfer_utils.js";
import {ethers} from "ethers";

const base_coin_balance = {

    // 转账方法
    query_balance(item, chain) {
        return new Promise(async (resolve, reject) => {
            // 随机获取rpc服务
            const provider = utils.get_provider(chain)
            // 通过私钥创建钱包
            let wallet = new ethers.Wallet(item.private_key, provider);
            wallet.getBalance().then((balance) => {
                item.plat_balance = parseFloat(ethers.utils.formatEther(balance)).toFixed(10).toString()
                resolve()
            }).catch((err) => {
                item.plat_balance = ''
                item.error_msg = '查询平台余额失败！'
                console.log('查询平台余额失败！', err)
                resolve()
            })
        })
    }
}

export default base_coin_balance