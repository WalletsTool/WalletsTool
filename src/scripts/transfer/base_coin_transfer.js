import {BigNumber, ethers} from "ethers";
import utils from "@/scripts/transfer/transfer_utils.js";
import {utils as provider_utils} from "@/scripts/common/provider.js";
import {utils as common_utils} from "@/scripts/common/utils.js";
import * as web3 from "@solana/web3.js";
import bs58 from "bs58";
import {Wallet} from "zksync-ethers";


// 转账配置说明
const config = {
    chain: "", // arb, op, eth, bsc 等
    delay: [1, 3],
    transfer_type: 0, // 转账类型 1：全部转账 2:转账固定数量 3：转账随机数量  4：剩余随机数量
    transfer_amount: 5, // 转账固定金额
    transfer_amount_list: [5, 10], // 转账数量 (transfer_type 为 1 时生效) 转账数量在5-10之间随机，第二个数要大于第一个数！！
    left_amount_list: [4, 6], // 剩余数量 (transfer_type 为 2 时生效) 剩余数量在4-6之间随机，第二个数要大于第一个数！！
    amount_precision: 6, // 无需修改，转账个数的精确度 6 代表个数有6位小数
    limit_type: "1", // limit_type 限制类型 1：自动 2：指定数量 3：范围随机
    limit_count_list: [21111, 30000],
    gas_price_type: "2",
    gas_price_rate: 0.05, // gas price溢价率，0.05代表gas price是当前gas price的105%
    max_gas_price: 1, // 设置最大的gas price，单位gwei
};

const base_coin_transfer = {
    async getSolFee(provider, feePayer, item, balance) {
        const {blockhash} = await provider.getLatestBlockhash()
        const transaction_temp = new web3.Transaction({
            feePayer: feePayer.publicKey,
            recentBlockhash: blockhash,
        }).add(
            web3.SystemProgram.transfer({
                fromPubkey: feePayer.publicKey,
                toPubkey: item.to_addr,
                lamports: parseInt(balance * web3.LAMPORTS_PER_SOL),
            })
        );
        const response = await provider.getFeeForMessage(
            transaction_temp.compileMessage(),
            'confirmed',
        );
        const feeInLamports = response.value;
        return feeInLamports / web3.LAMPORTS_PER_SOL;
    },
    // 转账方法
    single_transfer(index, item, config) {
        return new Promise(async (resolve, reject) => {
            item.retry_flag = false;
            if (config.chain === "sol") {
                // 随机获取rpc服务
                const provider = provider_utils.get_provider(config.chain);
                try {
                    const feePayer = web3.Keypair.fromSecretKey(
                        bs58.decode(item.private_key)
                    );
                    let balance_lam = await provider.getBalance(feePayer.publicKey);
                    // 获取余额
                    const balance = balance_lam / web3.LAMPORTS_PER_SOL;
                    console.log("序号：", index, "当前余额为:", balance, " SOL");
                    if (Number(balance) > 0) {
                        let transfer_amount = BigNumber.from(0);
                        if (config.transfer_type === "1") {
                            const fee = await this.getSolFee(provider, feePayer, item, parseFloat(balance));
                            if (parseFloat(fee) >= parseFloat(balance)) {
                                reject("当前余额不足，不做转账操作！");
                                return;
                            }
                            // 全部转账
                            transfer_amount = parseFloat(balance) - parseFloat(fee);
                        } else if (config.transfer_type === "2") {
                            const fee = await this.getSolFee(provider, feePayer, item, parseFloat(config.transfer_amount));
                            if (
                                parseFloat(config.transfer_amount) + parseFloat(fee) >=
                                parseFloat(balance)
                            ) {
                                reject("当前余额不足，不做转账操作！");
                                return;
                            }
                            // 转账固定数量
                            transfer_amount = config.transfer_amount;
                        } else if (config.transfer_type === "3") {
                            const temp = (
                                Math.random() *
                                (Number(config.transfer_amount_list[1]) -
                                    Number(config.transfer_amount_list[0])) +
                                Number(config.transfer_amount_list[0])
                            ).toFixed(Number(config.amount_precision));
                            const fee = await this.getSolFee(provider, feePayer, item, parseFloat(temp));
                            if (
                                parseFloat(temp) + parseFloat(fee) >=
                                parseFloat(balance)
                            ) {
                                reject("当前余额不足，不做转账操作！");
                                return;
                            }
                            // 转账范围随机
                            transfer_amount = temp;
                        } else if (config.transfer_type === "4") {
                            if (
                                parseFloat(balance) >= Number(config.left_amount_list[0]) &&
                                parseFloat(balance) <= Number(config.left_amount_list[1])
                            ) {
                                reject(
                                    "当前余额为：" +
                                    balance +
                                    " 在设置的剩余范围内，不做转账操作！"
                                );
                                return;
                            }
                            let left_amount = (
                                Math.random() *
                                (Number(config.left_amount_list[1]) -
                                    Number(config.left_amount_list[0])) +
                                Number(config.left_amount_list[0])
                            ).toFixed(Number(config.amount_precision));
                            const fee = await this.getSolFee(provider, feePayer, item, parseFloat(balance) - parseFloat(left_amount));
                            if (
                                parseFloat(left_amount) + parseFloat(fee) >=
                                parseFloat(balance)
                            ) {
                                reject("当前余额不足，不做转账操作！");
                                return;
                            }
                            // 剩余随机数量
                            transfer_amount = (parseFloat(balance) - parseFloat(left_amount)).toFixed(Number(config.amount_precision));
                        }

                        console.log(
                            "序号：",
                            index,
                            "转账数量为:",
                            transfer_amount
                        );
                        const transaction = new web3.Transaction().add(
                            web3.SystemProgram.transfer({
                                fromPubkey: feePayer.publicKey,
                                toPubkey: item.to_addr,
                                lamports: parseInt(transfer_amount * web3.LAMPORTS_PER_SOL),
                            })
                        );
                        item.error_msg = "发送交易...";
                        web3.sendAndConfirmTransaction(
                            provider,
                            transaction,
                            [feePayer]
                        ).then(async (res) => {
                            console.log("序号：", index, "交易 hash 为：", res);
                            item.error_msg = "交易完成...";
                            resolve(res)
                        }).catch((err) => {
                            if (
                                config.error_retry === "1" &&
                                item.error_count < config.error_count_limit
                            ) {
                                item.error_count = item.error_count + 1;
                                item.retry_flag = true;
                            }
                            reject(err);
                        });
                    } else {
                        reject("当前余额不足，不做转账操作！");
                    }
                } catch (err) {
                    if (
                        config.error_retry === "1" &&
                        item.error_count < config.error_count_limit
                    ) {
                        item.error_count = item.error_count + 1;
                        item.retry_flag = true;
                    }
                    console.log(err);
                    reject("获取基础信息失败：" + err);
                }
            }
            else if (config.chain === "zksync") {
                try {
                    // 随机获取rpc服务
                    const provider = provider_utils.get_provider(config.chain);
                    // 通过私钥创建钱包
                    let wallet = new Wallet(item.private_key, provider);
                    let balance_wei = await wallet.getBalance();
                    let fee_info = await utils.getWalletFeeZks(config, provider, wallet, item.to_addr, balance_wei);
                    const balance = ethers.utils.formatEther(balance_wei);
                    const gas_fee = fee_info.gas_fee;

                    console.log("序号：", index, "当前余额为:", balance);
                    console.log("序号：", index, "当前预估 gas_fee 为:", gas_fee);

                    if (Number(balance) > 0) {
                        let transfer_amount = BigNumber.from(0);
                        if (config.transfer_type === "1") {
                            if (parseFloat(gas_fee) >= parseFloat(balance)) {
                                reject("当前余额不足，不做转账操作！");
                                return;
                            }
                            // 全部转账
                            transfer_amount = balance_wei.sub(ethers.utils.parseEther(gas_fee));
                        } else if (config.transfer_type === "2") {
                            if (
                                parseFloat(config.transfer_amount) + parseFloat(gas_fee) >=
                                parseFloat(balance)
                            ) {
                                reject("当前余额不足，不做转账操作！");
                                return;
                            }
                            // 转账固定数量
                            transfer_amount = ethers.utils.parseEther(
                                config.transfer_amount
                            );
                        } else if (config.transfer_type === "3") {
                            const temp = (
                                Math.random() *
                                (Number(config.transfer_amount_list[1]) -
                                    Number(config.transfer_amount_list[0])) +
                                Number(config.transfer_amount_list[0])
                            ).toFixed(Number(config.amount_precision));
                            if (
                                parseFloat(temp) + parseFloat(gas_fee) >=
                                parseFloat(balance)
                            ) {
                                reject("当前余额不足，不做转账操作！");
                                return;
                            }
                            // 转账范围随机
                            transfer_amount = ethers.utils.parseEther(temp);
                        } else if (config.transfer_type === "4") {
                            if (
                                parseFloat(balance) >= Number(config.left_amount_list[0]) &&
                                parseFloat(balance) <= Number(config.left_amount_list[1])
                            ) {
                                reject(
                                    "当前余额为：" +
                                    balance +
                                    " 在设置的剩余范围内，不做转账操作！"
                                );
                                return;
                            }
                            let left_amount = (
                                Math.random() *
                                (Number(config.left_amount_list[1]) -
                                    Number(config.left_amount_list[0])) +
                                Number(config.left_amount_list[0])
                            ).toFixed(Number(config.amount_precision));
                            if (
                                parseFloat(left_amount) + parseFloat(gas_fee) >=
                                parseFloat(balance)
                            ) {
                                reject("当前余额不足，不做转账操作！");
                                return;
                            }
                            // 剩余随机数量
                            transfer_amount = ethers.utils.parseEther(
                                (
                                    parseFloat(balance) -
                                    parseFloat(gas_fee) -
                                    parseFloat(left_amount)
                                ).toFixed(Number(config.amount_precision))
                            );
                        }

                        console.log(
                            "序号：",
                            index,
                            "转账数量为:",
                            ethers.utils.formatEther(transfer_amount)
                        );
                        const tx = {
                            from: wallet.address,
                            to: item.to_addr,
                            value: transfer_amount,
                            gasPrice: fee_info.gas_price,
                            gasLimit: fee_info.gas_limit
                        };
                        item.error_msg = "发送交易...";
                        wallet.sendTransaction(tx).then(async (res) => {
                            console.log("序号：", index, "交易 hash 为：", res.hash);
                            item.error_msg = "等待交易结果...";
                            const txHandle = await provider.getTransaction(res.hash)
                            txHandle.wait().then(async () => {
                                resolve(res.hash);
                            }).catch((err) => {
                                if (
                                    config.error_retry === "1" &&
                                    item.error_count < config.error_count_limit
                                ) {
                                    item.error_count = item.error_count + 1;
                                    item.retry_flag = true;
                                }
                                reject(err);
                            });
                        }).catch((err) => {
                            if (
                                config.error_retry === "1" &&
                                item.error_count < config.error_count_limit
                            ) {
                                item.error_count = item.error_count + 1;
                                item.retry_flag = true;
                            }
                            reject(err);
                        });
                    } else {
                        reject("当前余额不足，不做转账操作！");
                    }
                } catch (err) {
                    if (
                        config.error_retry === "1" &&
                        item.error_count < config.error_count_limit
                    ) {
                        item.error_count = item.error_count + 1;
                        item.retry_flag = true;
                    }
                    console.log(err);
                    reject("获取基础信息失败：" + err);
                }

            }
            else if (config.chain === "manta"){
                try {
                    // 随机获取rpc服务
                    const provider = provider_utils.get_provider(config.chain);
                    // 通过私钥创建钱包
                    let wallet = new ethers.Wallet(item.private_key, provider);
                    let balance_wei = wallet.getBalance();
                    let gas_limit = utils.getWalletGasLimit(config, wallet, item.to_addr);
                    let nonce = wallet.getTransactionCount();
                    let gas_price = utils.getGasPrice(config, provider);
                    Promise.all([balance_wei, gas_price, nonce, gas_limit])
                        .then(async (values) => {
                            // 如果当前gas fee太高
                            if (values[1] === "base gas price 超出最大值限制") {
                                reject("base gas price 超出最大值限制");
                                return;
                            }

                            const balance = ethers.utils.formatEther(values[0]);
                            let gas_fee = await utils.getWalletFeeManta(config, provider, wallet, item.to_addr);

                            console.log("序号：", index, "当前余额为:", balance);
                            console.log(
                                "序号：",
                                index,
                                "当前 gas_limit 为:",
                                values[3].toNumber()
                            );
                            console.log(
                                "序号：",
                                index,
                                "当前设置 gas_price 为:",
                                ethers.utils.formatUnits(values[1], "gwei"),
                                " Gwei"
                            );
                            console.log("序号：", index, "当前预估 gas_fee 为:", gas_fee, "按比例提升为:", parseFloat(gas_fee)*1.2);
                            // 按比例提高下预估fee
                            gas_fee = parseFloat(gas_fee)*1.2
                            if (Number(balance) > 0) {
                                let transfer_amount = BigNumber.from(0);
                                if (config.transfer_type === "1") {
                                    if (parseFloat(gas_fee) >= parseFloat(balance)) {
                                        reject("当前余额不足，不做转账操作！");
                                        return;
                                    }
                                    // 全部转账
                                    transfer_amount = ethers.utils.parseEther(
                                        (
                                            parseFloat(balance) -
                                            parseFloat(gas_fee)
                                        ).toFixed(8)
                                    );
                                } else if (config.transfer_type === "2") {
                                    if (
                                        parseFloat(config.transfer_amount) + parseFloat(gas_fee) >=
                                        parseFloat(balance)
                                    ) {
                                        reject("当前余额不足，不做转账操作！");
                                        return;
                                    }
                                    // 转账固定数量
                                    transfer_amount = ethers.utils.parseEther(
                                        config.transfer_amount
                                    );
                                } else if (config.transfer_type === "3") {
                                    const temp = (
                                        Math.random() *
                                        (Number(config.transfer_amount_list[1]) -
                                            Number(config.transfer_amount_list[0])) +
                                        Number(config.transfer_amount_list[0])
                                    ).toFixed(Number(config.amount_precision));
                                    if (
                                        parseFloat(temp) + parseFloat(gas_fee) >=
                                        parseFloat(balance)
                                    ) {
                                        reject("当前余额不足，不做转账操作！");
                                        return;
                                    }
                                    // 转账范围随机
                                    transfer_amount = ethers.utils.parseEther(temp);
                                } else if (config.transfer_type === "4") {
                                    if (
                                        parseFloat(balance) >= Number(config.left_amount_list[0]) &&
                                        parseFloat(balance) <= Number(config.left_amount_list[1])
                                    ) {
                                        reject(
                                            "当前余额为：" +
                                            balance +
                                            " 在设置的剩余范围内，不做转账操作！"
                                        );
                                        return;
                                    }
                                    let left_amount = (
                                        Math.random() *
                                        (Number(config.left_amount_list[1]) -
                                            Number(config.left_amount_list[0])) +
                                        Number(config.left_amount_list[0])
                                    ).toFixed(Number(config.amount_precision));
                                    if (
                                        parseFloat(left_amount) + parseFloat(gas_fee) >=
                                        parseFloat(balance)
                                    ) {
                                        reject("当前余额不足，不做转账操作！");
                                        return;
                                    }
                                    // 剩余随机数量
                                    transfer_amount = ethers.utils.parseEther(
                                        (
                                            parseFloat(balance) -
                                            parseFloat(gas_fee) -
                                            parseFloat(left_amount)
                                        ).toFixed(Number(config.amount_precision))
                                    );
                                }

                                console.log(
                                    "序号：",
                                    index,
                                    "转账数量为:",
                                    ethers.utils.formatEther(transfer_amount)
                                );

                                const tx = {
                                    from: wallet.address,
                                    to: item.to_addr,
                                    nonce: values[2],
                                    value: transfer_amount,
                                    gasPrice: values[1],
                                    gasLimit: values[3],
                                };
                                item.error_msg = "发送交易...";
                                wallet
                                    .sendTransaction(tx)
                                    .then(async (res) => {
                                        console.log("序号：", index, "交易 hash 为：", res.hash);
                                        item.error_msg = "等待交易结果...";
                                        provider
                                            .waitForTransaction(res.hash, null, 30000)
                                            .then(async (receipt) => {
                                                if (receipt.status === 1) {
                                                    await common_utils.sleep(config.delay);
                                                    resolve(res.hash);
                                                } else {
                                                    if (
                                                        config.error_retry === "1" &&
                                                        item.error_count < config.error_count_limit
                                                    ) {
                                                        item.error_count = item.error_count + 1;
                                                        item.retry_flag = true;
                                                    }
                                                    reject("交易失败：" + JSON.stringify(receipt));
                                                }
                                            })
                                            .catch((err) => {
                                                if (
                                                    config.error_retry === "1" &&
                                                    item.error_count < config.error_count_limit
                                                ) {
                                                    item.error_count = item.error_count + 1;
                                                    item.retry_flag = true;
                                                }
                                                reject(err);
                                            });
                                    })
                                    .catch((err) => {
                                        if (
                                            config.error_retry === "1" &&
                                            item.error_count < config.error_count_limit
                                        ) {
                                            item.error_count = item.error_count + 1;
                                            item.retry_flag = true;
                                        }
                                        reject(err);
                                    });
                            } else {
                                reject("当前余额不足，不做转账操作！");
                            }
                        })
                        .catch((err) => {
                            if (
                                config.error_retry === "1" &&
                                item.error_count < config.error_count_limit
                            ) {
                                item.error_count = item.error_count + 1;
                                item.retry_flag = true;
                            }
                            console.log(err);
                            reject("获取基础信息失败：" + err);
                        });
                } catch (err) {
                    if (
                        config.error_retry === "1" &&
                        item.error_count < config.error_count_limit
                    ) {
                        item.error_count = item.error_count + 1;
                        item.retry_flag = true;
                    }
                    console.log(err);
                    reject("获取基础信息失败：" + err);
                }
            }
            else {
                // 随机获取rpc服务
                const provider = provider_utils.get_provider(config.chain);
                // 通过私钥创建钱包
                let wallet = new ethers.Wallet(item.private_key, provider);
                let balance_wei = wallet.getBalance();
                let gas_limit = utils.getWalletGasLimit(config, wallet, item.to_addr);
                let nonce = wallet.getTransactionCount();
                let gas_price = utils.getGasPrice(config, provider);
                Promise.all([balance_wei, gas_price, nonce, gas_limit])
                    .then(async (values) => {
                        // 如果当前gas fee太高
                        if (values[1] === "base gas price 超出最大值限制") {
                            reject("base gas price 超出最大值限制");
                            return;
                        }

                        const balance = ethers.utils.formatEther(values[0]);
                        const gas_fee = ethers.utils.formatEther(values[1].mul(values[3]));

                        console.log("序号：", index, "当前余额为:", balance);
                        console.log(
                            "序号：",
                            index,
                            "当前 gas_limit 为:",
                            values[3].toNumber()
                        );
                        console.log(
                            "序号：",
                            index,
                            "当前设置 gas_price 为:",
                            ethers.utils.formatUnits(values[1], "gwei"),
                            " Gwei"
                        );
                        console.log("序号：", index, "当前预估 gas_fee 为:", gas_fee);

                        if (Number(balance) > 0) {
                            let transfer_amount = BigNumber.from(0);
                            if (config.transfer_type === "1") {
                                if (parseFloat(gas_fee) >= parseFloat(balance)) {
                                    reject("当前余额不足，不做转账操作！");
                                    return;
                                }
                                if (config.chainLayer === "L2") {
                                    const l1Provider = provider_utils.get_provider(config.l1);
                                    let l1_gas_limit = await utils.getWalletGasLimit(
                                        config,
                                        new ethers.Wallet(item.private_key, l1Provider),
                                        item.to_addr
                                    );
                                    let l1_gas_price = await utils.getGasPrice(
                                        config,
                                        l1Provider
                                    );
                                    // 如果当前gas fee太高
                                    if (l1_gas_price === "base gas price 超出最大值限制") {
                                        reject("l1 base gas price 超出最大值限制");
                                        return;
                                    }
                                    if (
                                        parseFloat(gas_fee) +
                                        parseFloat(
                                            ethers.utils.formatEther(l1_gas_price.mul(l1_gas_limit))
                                        ) >=
                                        parseFloat(balance)
                                    ) {
                                        reject("当前余额不足，不做转账操作！");
                                        return;
                                    }
                                    // 计算l1的gas消耗
                                    const rate = Math.round((config.scalar ?? 1) * 1000);
                                    const l1_gas_fee_wei = l1_gas_price
                                        .mul(l1_gas_limit)
                                        .mul(rate)
                                        .div(1000);
                                    // 全部转账
                                    transfer_amount = values[0]
                                        .sub(values[1].mul(values[3])) // l2 的gas费
                                        .sub(l1_gas_fee_wei); // l1 的gas费
                                    // 处理scroll无法转账为0的问题
                                    if (config.chain === "scroll") {
                                        transfer_amount = transfer_amount.sub(
                                            ethers.utils.parseEther("0.0000000001")
                                        );
                                    }
                                } else {
                                    // 全部转账
                                    transfer_amount = values[0].sub(values[1].mul(values[3]));
                                }
                            } else if (config.transfer_type === "2") {
                                if (
                                    parseFloat(config.transfer_amount) + parseFloat(gas_fee) >=
                                    parseFloat(balance)
                                ) {
                                    reject("当前余额不足，不做转账操作！");
                                    return;
                                }
                                // 转账固定数量
                                transfer_amount = ethers.utils.parseEther(
                                    config.transfer_amount
                                );
                            } else if (config.transfer_type === "3") {
                                const temp = (
                                    Math.random() *
                                    (Number(config.transfer_amount_list[1]) -
                                        Number(config.transfer_amount_list[0])) +
                                    Number(config.transfer_amount_list[0])
                                ).toFixed(Number(config.amount_precision));
                                if (
                                    parseFloat(temp) + parseFloat(gas_fee) >=
                                    parseFloat(balance)
                                ) {
                                    reject("当前余额不足，不做转账操作！");
                                    return;
                                }
                                // 转账范围随机
                                transfer_amount = ethers.utils.parseEther(temp);
                            } else if (config.transfer_type === "4") {
                                if (
                                    parseFloat(balance) >= Number(config.left_amount_list[0]) &&
                                    parseFloat(balance) <= Number(config.left_amount_list[1])
                                ) {
                                    reject(
                                        "当前余额为：" +
                                        balance +
                                        " 在设置的剩余范围内，不做转账操作！"
                                    );
                                    return;
                                }
                                let left_amount = (
                                    Math.random() *
                                    (Number(config.left_amount_list[1]) -
                                        Number(config.left_amount_list[0])) +
                                    Number(config.left_amount_list[0])
                                ).toFixed(Number(config.amount_precision));
                                if (
                                    parseFloat(left_amount) + parseFloat(gas_fee) >=
                                    parseFloat(balance)
                                ) {
                                    reject("当前余额不足，不做转账操作！");
                                    return;
                                }
                                // 剩余随机数量
                                transfer_amount = ethers.utils.parseEther(
                                    (
                                        parseFloat(balance) -
                                        parseFloat(gas_fee) -
                                        parseFloat(left_amount)
                                    ).toFixed(Number(config.amount_precision))
                                );
                            }

                            console.log(
                                "序号：",
                                index,
                                "转账数量为:",
                                ethers.utils.formatEther(transfer_amount)
                            );

                            const tx = {
                                from: wallet.address,
                                to: item.to_addr,
                                nonce: values[2],
                                value: transfer_amount,
                                gasPrice: values[1],
                                gasLimit: values[3],
                            };
                            item.error_msg = "发送交易...";
                            wallet
                                .sendTransaction(tx)
                                .then(async (res) => {
                                    console.log("序号：", index, "交易 hash 为：", res.hash);
                                    item.error_msg = "等待交易结果...";
                                    provider
                                        .waitForTransaction(res.hash, null, 30000)
                                        .then(async (receipt) => {
                                            if (receipt.status === 1) {
                                                await common_utils.sleep(config.delay);
                                                resolve(res.hash);
                                            } else {
                                                if (
                                                    config.error_retry === "1" &&
                                                    item.error_count < config.error_count_limit
                                                ) {
                                                    item.error_count = item.error_count + 1;
                                                    item.retry_flag = true;
                                                }
                                                reject("交易失败：" + JSON.stringify(receipt));
                                            }
                                        })
                                        .catch((err) => {
                                            if (
                                                config.error_retry === "1" &&
                                                item.error_count < config.error_count_limit
                                            ) {
                                                item.error_count = item.error_count + 1;
                                                item.retry_flag = true;
                                            }
                                            reject(err);
                                        });
                                })
                                .catch((err) => {
                                    if (
                                        config.error_retry === "1" &&
                                        item.error_count < config.error_count_limit
                                    ) {
                                        item.error_count = item.error_count + 1;
                                        item.retry_flag = true;
                                    }
                                    reject(err);
                                });
                        } else {
                            reject("当前余额不足，不做转账操作！");
                        }
                    })
                    .catch((err) => {
                        if (
                            config.error_retry === "1" &&
                            item.error_count < config.error_count_limit
                        ) {
                            item.error_count = item.error_count + 1;
                            item.retry_flag = true;
                        }
                        console.log(err);
                        reject("获取基础信息失败：" + err);
                    });
            }
        });
    },
};

export default base_coin_transfer;
