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

const base_coin_transfer = {

    // 转账方法
    single_transfer(index, item, config) {
        return new Promise((resolve, reject) => {
            // 随机获取rpc服务
            const provider = provider_utils.get_provider(config.chain)
            // 通过私钥创建钱包
            let wallet = new ethers.Wallet(item.private_key, provider);
            let balance_wei = wallet.getBalance();
            let gas_limit = utils.getWalletGasLimit(config, wallet, item.to_addr);
            let nonce = wallet.getTransactionCount();
            let gas_price = utils.getGasPrice(config, provider);
            Promise.all([balance_wei, gas_price, nonce, gas_limit]).then(async (values) => {
                // 当设置的上限小于当前的base gas price时，不做转账操作
                if (config.max_gas_price && ethers.utils.parseUnits(config.max_gas_price, 'gwei').lt(values[1])) {
                    reject('当前设置的gas price上限小于当前的base gas price，不做转账操作！')
                    return
                }

                const balance = ethers.utils.formatEther(values[0])
                const gas_fee = ethers.utils.formatEther(values[1].mul(values[3]))

                console.log('序号：', index, '当前余额为:', balance)
                console.log('序号：', index, '当前 gas_limit 为:', values[3].toNumber())
                console.log('序号：', index, '当前设置 gas_price 为:', ethers.utils.formatUnits(values[1], 'gwei'), ' Gwei')
                console.log('序号：', index, '当前预估 gas_fee 为:', gas_fee)

                if (Number(balance) > 0) {
                    let transfer_amount = BigNumber.from(0)
                    if (config.transfer_type === '1') {
                        if (parseFloat(gas_fee) >= parseFloat(balance)) {
                            reject('当前余额不足，不做转账操作！')
                            return
                        }
                        // 全部转账
                        transfer_amount = values[0].sub(values[1].mul(values[3]))
                        // 处理scroll无法转账为0的问题
                        if (config.chain === 'scroll') transfer_amount = transfer_amount.sub(ethers.utils.parseEther('0.0000000001'))
                    } else if (config.transfer_type === '2') {
                        if ((parseFloat(config.transfer_amount) + parseFloat(gas_fee)) >= parseFloat(balance)) {
                            reject('当前余额不足，不做转账操作！')
                            return
                        }
                        // 转账固定数量
                        transfer_amount = ethers.utils.parseEther(config.transfer_amount)
                    } else if (config.transfer_type === '3') {
                        const temp = (Math.random() * (Number(config.transfer_amount_list[1]) - Number(config.transfer_amount_list[0])) + Number(config.transfer_amount_list[0])).toFixed(Number(config.amount_precision))
                        if ((parseFloat(temp) + parseFloat(gas_fee)) >= parseFloat(balance)) {
                            reject('当前余额不足，不做转账操作！')
                            return
                        }
                        // 转账范围随机
                        transfer_amount = ethers.utils.parseEther(temp)
                    } else if (config.transfer_type === '4') {
                        if (parseFloat(balance) >= Number(config.left_amount_list[0]) && parseFloat(balance) <= Number(config.left_amount_list[1])) {
                            reject('当前余额为：' + balance + ' 在设置的剩余范围内，不做转账操作！')
                            return
                        }
                        let left_amount = (Math.random() * (Number(config.left_amount_list[1]) - Number(config.left_amount_list[0])) + Number(config.left_amount_list[0])).toFixed(Number(config.amount_precision));
                        if ((parseFloat(left_amount) + parseFloat(gas_fee)) >= parseFloat(balance)) {
                            reject('当前余额不足，不做转账操作！')
                            return
                        }
                        // 剩余随机数量
                        transfer_amount = ethers.utils.parseEther((parseFloat(balance) - parseFloat(gas_fee) - parseFloat(left_amount)).toFixed(Number(config.amount_precision)))
                    }

                    console.log('序号：', index, '转账数量为:', ethers.utils.formatEther(transfer_amount))

                    const tx = {
                        from: wallet.address,
                        to: item.to_addr,
                        nonce: values[2],
                        value: transfer_amount,
                        gasPrice: values[1],
                        gasLimit: values[3]
                    }
                    item.error_msg = '发送交易...'
                    wallet.sendTransaction(tx).then(async res => {
                        console.log('序号：', index, '交易 hash 为：', res.hash)
                        item.error_msg = '等待交易结果...'
                        provider.waitForTransaction(res.hash).then(async receipt => {
                            if(receipt.status === 1) {
                                await common_utils.sleep(config.delay)
                                resolve(res.hash)
                            }else {
                                reject('交易失败：' + JSON.stringify(receipt))
                            }
                        }).catch(err => {
                            reject(err)
                        })
                    }).catch(err => {
                        reject(err)
                    })
                } else {
                    reject('当前余额不足，不做转账操作！')
                }
            }).catch(err => {
                console.log(err)
                reject('获取基础信息失败：' + err)
            })
        })
    }
}

export default base_coin_transfer

