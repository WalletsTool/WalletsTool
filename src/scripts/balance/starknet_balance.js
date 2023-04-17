import {Contract, Account, Provider, uint256} from "starknet";
import {BigNumber, ethers} from "ethers";

const starknet_balance = {

    // 转账方法
    async query_balance_by_address(item, contractAddress, proxyAddress) {
        return new Promise(async (resolve, reject) => {
            const provider = new Provider({sequencer: {network: 'mainnet-alpha'}})
            let contractABI
            if (proxyAddress) {
                const {abi} = await provider.getClassAt(proxyAddress);
                contractABI = abi
            } else {
                const {abi} = await provider.getClassAt(contractAddress);
                contractABI = abi
            }
            if (contractABI === undefined) {
                item.coin_balance = ''
                item.error_msg = '查询合约ABI失败！'
                console.log("查询合约ABI失败！")
                resolve()
            }
            const ethContract = new Contract(contractABI, contractAddress, provider);
            const balance_result = ethContract.balanceOf(item.address);
            const decimals_result = ethContract.decimals();
            Promise.all([balance_result, decimals_result]).then(async ([balance_result, decimals_result]) => {
                const balance_str = uint256.uint256ToBN(balance_result.balance).toString()
                const decimals_str = decimals_result.decimals.toString()
                item.coin_balance = parseFloat(ethers.utils.formatUnits(BigNumber.from(balance_str), decimals_str)).toFixed(6).toString()
                resolve()
            }).catch((err) => {
                item.coin_balance = ''
                item.error_msg = '查询代币余额失败！'
                console.log('查询代币余额失败！', err)
                resolve()
            })
        })
    }
}


export default starknet_balance
