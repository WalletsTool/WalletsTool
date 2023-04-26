import {BigNumber, ethers} from "ethers";

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
                resolve(await provider.getGasPrice())
            } else if (config.gas_price_type === '2') {
                resolve(ethers.utils.parseUnits(config.max_gas_price, 'gwei'))
            } else if (config.gas_price_type === '3') {
                // 计算 gas_price 溢价
                let gas_price_final = BigNumber.from('0')
                await provider.getGasPrice().then((gas_price) => {
                    const gas_price_by_rate = BigNumber.from(Math.ceil(Number(gas_price.toString()) * (1 + Number(config.gas_price_rate))).toString())
                    if (config.max_gas_price && Number(ethers.utils.formatUnits(gas_price_by_rate, 'gwei')) >= Number(config.max_gas_price)) {
                        gas_price_final = ethers.utils.parseUnits(config.max_gas_price, 'gwei')
                    } else {
                        gas_price_final = gas_price_by_rate
                    }
                    resolve(gas_price_final)
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
                resolve(await wallet.estimateGas({from: wallet.address, to: to_address}))
            } else if (config.limit_type === '2') {
                resolve(BigNumber.from(config.limit_value))
            } else if (config.limit_type === '3') {
                let gas_limit_final = Math.floor(Math.random() * (Number(config.limit_count_list[1]) - Number(config.limit_count_list[0])) + Number(config.limit_count_list[0]));
                resolve(BigNumber.from(gas_limit_final.toString()))
            } else {
                reject('gas limit type error')
            }
        })
    },
    // 获取 GasLimit 设置
    getContractGasLimit(config, contract, wallet, to_address, transfer_amount) {
        return new Promise(async (resolve, reject) => {
            // 计算 gas_limit
            if (config.limit_type === '1') {
                resolve(await contract.connect(wallet).estimateGas.transfer(to_address, transfer_amount))
            } else if (config.limit_type === '2') {
                resolve(BigNumber.from(config.limit_value))
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
