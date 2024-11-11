import {BigNumber, ethers} from "ethers";
import { asL2Provider } from "@constellation-labs/bedrock-sdk";
const transfer_utils = {
    //判断字符串是否为数字
    checkNum(num) {
        if (!num) {
            return false
        }
        const reg = /^[0-9]+.?[0-9]*$/;
        return reg.test(num);
    },
    //判断字符串是否为正整数
    checkPositiveInteger(num) {
        if (!num) {
            return false
        }
        const reg = /^[1-9]+[0-9]*]*$/;
        return reg.test(num);
    },
    // 获取 GasPrice 设置
    getGasPrice(config, provider) {
        return new Promise(async (resolve, reject) => {
            if (config.gas_price_type === '1') {
                provider.getGasPrice().then((gas_price) => {
                    let gas_price_final = gas_price
                    // 如果存在最大值 gas price 限制
                    if (config.max_gas_price) {
                        if (Number(ethers.utils.formatUnits(gas_price, 'gwei')) > Number(config.max_gas_price)) {
                            resolve('base gas price 超出最大值限制')
                        } else {
                            resolve(gas_price_final)
                        }
                    } else {
                        resolve(gas_price_final)
                    }
                }).catch((err) => {
                    console.log('获取gas_price 失败，', err)
                    reject('获取gas_price 失败，' + err)
                })
            } else if (config.gas_price_type === '2') {
                resolve(ethers.utils.parseUnits(config.gas_price, 'gwei'))
            } else if (config.gas_price_type === '3') {
                // 计算 gas_price 溢价
                let gas_price_final = BigNumber.from('0')
                provider.getGasPrice().then((gas_price) => {
                    const gas_price_by_rate = BigNumber.from(Math.ceil(Number(gas_price.toString()) * (1 + Number(config.gas_price_rate))).toString())
                    gas_price_final = gas_price_by_rate
                    let flag = false
                    if (config.max_gas_price) {
                        if (Number(ethers.utils.formatUnits(gas_price, 'gwei')) > Number(config.max_gas_price)) {
                            flag = true
                        } else {
                            if (Number(ethers.utils.formatUnits(gas_price_by_rate, 'gwei')) >= Number(config.max_gas_price)) {
                                gas_price_final = ethers.utils.parseUnits(config.max_gas_price, 'gwei')
                            }
                        }
                    }
                    if (flag) {
                        resolve('base gas price 超出最大值限制')
                    } else {
                        resolve(gas_price_final)
                    }
                }).catch((err) => {
                    console.log('获取gas_price 失败', err)
                    reject('获取gas_price 失败')
                })
            } else {
                reject('gas price type error')
            }
        })
    },
    // 获取 GasLimit 设置
    getWalletGasLimit(config, wallet, to_address) {
        return new Promise(async (resolve, reject) => {
            // 计算 gas_limit
            if (config.limit_type === '1') {
                wallet.estimateGas({from: wallet.address, to: to_address}).then((gas_limit) => {
                    resolve(gas_limit)
                }).catch((err) => {
                    console.log('获取gas_limit 失败，', err)
                    reject('获取gas_limit 失败，' + err)
                })
            } else if (config.limit_type === '2') {
                resolve(BigNumber.from(config.limit_count))
            } else if (config.limit_type === '3') {
                let gas_limit_final = Math.floor(Math.random() * (Number(config.limit_count_list[1]) - Number(config.limit_count_list[0])) + Number(config.limit_count_list[0]));
                resolve(BigNumber.from(gas_limit_final.toString()))
            } else {
                reject('gas limit type error')
            }
        })
    },
    // 获取 gas_fee ZKS
    getWalletFeeZks(config, provider, wallet, to_address, amount) {
        return new Promise(async (resolve, reject) => {
            // 获取费用信息
            provider.estimateFee({
                from: await wallet.getAddress(),
                to: to_address,
                value: amount.toHexString()
            }).then(async (fee) => {
                // 计算gas_price的设置
                const gas_price = await this.getGasPrice(config, provider)
                // 计算 gas_limit
                if (config.limit_type === '1') {
                    resolve({
                        gas_fee: ethers.utils.formatEther(gas_price.mul(BigNumber.from(fee.gas_limit))).toString(),
                        gas_price: gas_price,
                        gas_limit: BigNumber.from(fee.gas_limit)
                    })
                } else if (config.limit_type === '2') {
                    resolve({
                        gas_fee: ethers.utils.formatEther(gas_price.mul(BigNumber.from(config.limit_count))).toString(),
                        gas_price: gas_price,
                        gas_limit: BigNumber.from(config.limit_count)
                    })
                } else if (config.limit_type === '3') {
                    let gas_limit_final = Math.floor(Math.random() * (Number(config.limit_count_list[1]) - Number(config.limit_count_list[0])) + Number(config.limit_count_list[0]));
                    resolve({
                        gas_fee: ethers.utils.formatEther(gas_price.mul(BigNumber.from(gas_limit_final.toString()))).toString(),
                        gas_price: gas_price,
                        gas_limit: BigNumber.from(gas_limit_final.toString())
                    })
                } else {
                    reject('gas limit type error')
                }
            }).catch((err) => {
                console.log('获取gas_fee 失败，', err)
                reject('获取gas_fee 失败，' + err)
            })
        })
    },
    // 获取 gas_fee Manta
    getWalletFeeManta(config, provider, wallet, to_address) {
        return new Promise(async (resolve, reject) => {
            try {
                const l2RpcProvider = asL2Provider(provider);
                // 获取费用信息
                l2RpcProvider.estimateTotalGasCost({
                    from: await wallet.getAddress(),
                    to: to_address,
                    type: 2,
                }).then(async (fee) => {
                    resolve(ethers.utils.formatEther(fee).toString())
                }).catch((err) => {
                    console.log('获取gas_fee 失败，', err)
                    reject('获取gas_fee 失败，' + err)
                })
            } catch (e){
                console.log('获取gas_fee 失败，', e)
                reject('获取gas_fee 失败，' + e)
            }
        })
    },
    // 获取 GasLimit 设置
    getContractGasLimit(config, provider, contract, wallet, to_address, transfer_amount) {
        return new Promise(async (resolve, reject) => {
            // 计算 gas_limit
            if (config.limit_type === '1') {
                if (config.chain === 'zksync') {
                    provider.estimateGasTransfer({
                        from: await wallet.getAddress(),
                        to: to_address,
                        token: contract.address,
                        amount: transfer_amount
                    }).then((gas_limit) => {
                        resolve(gas_limit)
                    }).catch((err) => {
                        console.log('获取gas_limit 失败，', err)
                        reject('获取gas_limit 失败，' + err)
                    })
                } else {
                    contract.connect(wallet).estimateGas.transfer(to_address, transfer_amount).then((gas_limit) => {
                        resolve(gas_limit)
                    }).catch((err) => {
                        console.log('获取gas_limit 失败，', err)
                        reject('获取gas_limit 失败，' + err)
                    })
                }
            } else if (config.limit_type === '2') {
                resolve(BigNumber.from(config.limit_count))
            } else if (config.limit_type === '3') {
                let gas_limit_final = Math.floor(Math.random() * (Number(config.limit_count_list[1]) - Number(config.limit_count_list[0])) + Number(config.limit_count_list[0]));
                resolve(BigNumber.from(gas_limit_final.toString()))
            } else {
                reject('gas limit type error')
            }
        })
    }
}

export default transfer_utils
