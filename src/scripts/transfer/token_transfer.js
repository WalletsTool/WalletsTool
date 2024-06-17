import {BigNumber, ethers} from "ethers";
import utils from "@/scripts/transfer/transfer_utils.js";
import {utils as provider_utils} from "@/scripts/common/provider.js";
import {utils as common_utils} from "@/scripts/common/utils.js";


// 转账配置说明
const config = {
    chain: '', // arb, op, eth, bsc 等
    delay: [1, 3],
    transfer_type: 0,  // 转账类型 1：全部转账 2:转账固定数量 3：转账随机数量  4：剩余随机数量
    transfer_amount: 5, // 转账固定金额
    transfer_amount_list: [5, 10],  // 转账数量 (transfer_type 为 1 时生效) 转账数量在5-10之间随机，第二个数要大于第一个数！！
    left_amount_list: [4, 6],  // 剩余数量 (transfer_type 为 2 时生效) 剩余数量在4-6之间随机，第二个数要大于第一个数！！
    amount_precision: 6,  // 无需修改，转账个数的精确度 6 代表个数有6位小数
    limit_type: '1', // limit_type 限制类型 1：自动 2：指定数量 3：范围随机
    limit_count_list: [21111, 30000],
    gas_price_type: '2',
    gas_price_rate: 0.05,  // gas price溢价率，0.05代表gas price是当前gas price的105%
    max_gas_price: 1  // 设置最大的gas price，单位gwei
}

const token_transfer = {
    single_transfer(index, item, config, contract) {
        // 随机获取rpc服务
        const provider = provider_utils.get_provider(config.chain)
        return new Promise((resolve, reject) => {
            item.retry_flag = false
            // 通过私钥创建钱包
            let wallet = new ethers.Wallet(item.private_key, provider);
            let balance_wei = contract.connect(wallet).balanceOf(wallet.address);
            let decimals = contract.connect(wallet).decimals();
            let gasPrice = utils.getGasPrice(config, provider);
            Promise.all([balance_wei, decimals, gasPrice]).then(async (values) => {
                // 如果当前gas fee太高
                if (values[2] === 'base gas price 超出最大值限制') {
                    reject('base gas price 超出最大值限制')
                    return
                }
                const balance = ethers.utils.formatUnits(values[0], values[1])

                console.log('序号：', index, '当前余额为:', balance)
                console.log('序号：', index, '当前 gasPrice 为:', ethers.utils.formatUnits(values[2], 'gwei'))

                if (Number(balance) > 0) {
                    let transfer_amount = BigNumber.from(0)
                    if (config.transfer_type === '1') {
                        // 全部转账
                        transfer_amount = values[0]
                    } else if (config.transfer_type === '2') {
                        if (parseFloat(config.transfer_amount) >= parseFloat(balance)) {
                            reject('当前余额不足，不做转账操作！')
                            return
                        }
                        // 转账固定数量
                        transfer_amount = ethers.utils.parseUnits(config.transfer_amount, values[1])
                    } else if (config.transfer_type === '3') {
                        const temp = (Math.random() * (Number(config.transfer_amount_list[1]) - Number(config.transfer_amount_list[0])) + Number(config.transfer_amount_list[0])).toFixed(Number(config.amount_precision))
                        if (parseFloat(temp) >= parseFloat(balance)) {
                            reject('当前余额不足，不做转账操作！')
                            return
                        }
                        // 转账随机数量
                        transfer_amount = ethers.utils.parseUnits(temp, values[1])
                    } else if (config.transfer_type === '4') {
                        if (parseFloat(balance) >= Number(config.left_amount_list[0]) && parseFloat(balance) <= Number(config.left_amount_list[1])) {
                            reject('当前余额为：' + balance + ' 在设置的剩余范围内，不做转账操作！')
                            return
                        }
                        let left_amount = (Math.random() * (Number(config.left_amount_list[1]) - Number(config.left_amount_list[0])) + Number(config.left_amount_list[0])).toFixed(Number(config.amount_precision));

                        if (parseFloat(left_amount) >= parseFloat(balance)) {
                            reject('当前余额不足，不做转账操作！')
                            return
                        }
                        // 剩余固定数量
                        transfer_amount = ethers.utils.parseUnits((parseFloat(balance) - parseFloat(left_amount)).toFixed(Number(config.amount_precision)), values[1])
                    }

                    console.log('序号：', index, '转账数量为:', ethers.utils.formatUnits(transfer_amount, values[1]))

                    const gasLimit = await utils.getContractGasLimit(config, provider, contract, wallet, item.to_addr, transfer_amount)
                    console.log('序号：', index, 'gasLimit:', gasLimit)

                    item.error_msg = '发送交易...'
                    contract.connect(wallet).transfer(item.to_addr, transfer_amount, {
                        gasPrice: values[2],
                        gasLimit: gasLimit
                    }).then(async res => {
                        console.log('序号：', index, '交易 hash 为：', res.hash)
                        item.error_msg = '等待交易结果...'
                        provider.waitForTransaction(res.hash, null, 30000).then(async receipt => {
                            if (receipt.status === 1) {
                                await common_utils.sleep(config.delay)
                                resolve(res.hash)
                            } else {
                                if (config.error_retry === '1' && item.error_count < config.error_count_limit) {
                                    item.error_count = item.error_count + 1
                                    item.retry_flag = true
                                }
                                reject('交易失败：' + JSON.stringify(receipt))
                            }
                        }).catch(err => {
                            if (config.error_retry === '1' && item.error_count < config.error_count_limit) {
                                item.error_count = item.error_count + 1
                                item.retry_flag = true
                            }
                            reject(err)
                        })
                    }).catch(err => {
                        if (config.error_retry === '1' && item.error_count < config.error_count_limit) {
                            item.error_count = item.error_count + 1
                            item.retry_flag = true
                        }
                        reject(err)
                    })
                } else {
                    reject('当前余额不足，不做转账操作！')
                }
            }).catch(err => {
                if (config.error_retry === '1' && item.error_count < config.error_count_limit) {
                    item.error_count = item.error_count + 1
                    item.retry_flag = true
                }
                console.log(err)
                reject('获取基础信息失败：' + err)
            })
        })
    }
}

export default token_transfer
