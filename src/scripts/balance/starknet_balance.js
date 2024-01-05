import {Contract, number, uint256} from "starknet";
import {BigNumber, ethers} from "ethers";
import {utils} from "@/scripts/common/provider.js";
import axios from "axios";

const starknet_balance = {

    // 转账方法
    async query_balance_by_address(item, contractAddress, contractABI) {
        const provider = utils.get_provider('starknet')
        console.log('当前RPC地址：',provider.nodeUrl)
        return new Promise(async (resolve) => {
            // 屏蔽主网api使用第三方rpc服务商的api
            // const provider = new Provider({sequencer: {network: 'mainnet-alpha'}})
            // const provider = utils.get_provider('starknet')
            // console.log('当前RPC地址：',provider.nodeUrl)
            const ethContract = new Contract(contractABI, contractAddress, provider);
            const balance_result = ethContract.balanceOf(item.address);
            const decimals_result = ethContract.decimals();
            const nonce_result = provider.getNonceForAddress(item.address);
            // const rpc_url = utils.get_provider('starknet')
            // const balance_result = this.query_balance(rpc_url, contractAddress, item.address);
            // const decimals_result = this.query_decimals(rpc_url, contractAddress);
            // const nonce_result = this.query_nonce(rpc_url, item.address);
            Promise.all([balance_result, decimals_result, nonce_result]).then(async ([balance_result, decimals_result, nonce_result]) => {
                let balance_str
                if (contractAddress === '0x0030c42f4c0a094ea1eda7e3086056a225a464c43dd7da48bd2083fc3114a4db') {
                    balance_str = balance_result.balance.toString()
                } else {
                    balance_str = uint256.uint256ToBN(balance_result.balance).toString()
                }
                const decimals_str = decimals_result.decimals.toString()
                const nonce_str = number.toBN(nonce_result).toString()
                // const balance_str = number.toBN(balance_result.data.result[0]).toString()
                // const decimals_str = number.toBN(decimals_result.data.result).toString()
                // const nonce_str = number.toBN(nonce_result.data.result).toString()
                item.coin_balance = parseFloat(ethers.utils.formatUnits(BigNumber.from(balance_str), decimals_str)).toFixed(6).toString()
                item.nonce = nonce_str
                resolve()
            }).catch((err) => {
                item.coin_balance = ''
                item.error_msg = '查询代币余额失败！'
                console.log('查询代币余额失败！',provider.nodeUrl, err)
                resolve()
            })
        })
    },
    query_balance(rpc_url, contract_address, address) {
        return axios.post(
            rpc_url,
            JSON.stringify({
                method: 'starknet_call', jsonrpc: '2.0', params: {
                    request: {
                        contract_address: contract_address,
                        entry_point_selector: "0x2e4263afad30923c891518314c3c95dbe830a16874e8abc5777a9a20b54c76e",
                        calldata: [address]
                    },
                    block_id: 'latest',
                }, id: 0
            }), {
                headers: {
                    'content-type': 'application/json'
                }
            })
    },
    query_decimals(rpc_url, contract_address) {
        return axios.post(
            rpc_url,
            JSON.stringify({
                method: 'starknet_call', jsonrpc: '2.0', params: {
                    request: {
                        contract_address: contract_address,
                        entry_point_selector: '0x4c4fb1ab068f6039d5780c68dd0fa2f8742cceb3426d19667778ca7f3518a9',
                        calldata: []
                    },
                    block_id: 'latest',
                }, id: 0
            }), {
                headers: {
                    'content-type': 'application/json'
                }
            })
    },
    query_nonce(rpc_url, address) {
        return axios.post(
            rpc_url,
            JSON.stringify({
                method: 'starknet_getNonce', jsonrpc: '2.0', params: {
                    contract_address: address,
                    block_id: 'latest'
                }, id: 0
            }), {
                headers: {
                    'content-type': 'application/json'
                }
            });
    }
}


export default starknet_balance
