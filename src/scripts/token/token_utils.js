import axios from "axios";
import {ethers} from "ethers";
import {utils as provider_utils} from "@/scripts/common/provider.js";
import {utils as common_utils} from "@/scripts/common/utils.js";


// 等待结果
let pending = true
// proxy地址
let proxy_address = ''

// 通过guid获取proxy地址
function checkProxyVerification(guid, check_verify_api) {
    return new Promise((resolve, reject) => {
        axios({
            method: 'post',
            url: check_verify_api + guid,
        }).then(res => {
            if (res.data.status === '0') {
                pending = res.data.result === 'Pending in queue';
                resolve()
            } else if (res.data.status === '1') {
                let result = res.data.result.match(/[\s?](0x?)[^\s]+[\s?]/g)
                pending = false
                proxy_address = result[0]
                resolve()
            } else {
                pending = false
                console.log('res:', res)
                resolve()
            }
        }).catch(err => {
            pending = false
            console.log(err)
            resolve()
        })
    })
}

const token_utils = {
    getAbi(address, scan_api) {
        return new Promise((resolve, reject) => {
            axios({
                method: 'get',
                url: scan_api + address
            }).then(async res => {
                let contractABI;
                contractABI = JSON.parse(res.data.result);
                if (!contractABI) {
                    console.log(res.data)
                    reject('无法获取合约ABI，添加代币失败！')
                } else {
                    resolve(contractABI)
                }
            }).catch(err => {
                console.log(err)
                reject('无法获取合约ABI，添加代币失败！')
            })
        })
    },
    getProxyAddress(address, verify_api, check_verify_api) {
        return new Promise((resolve, reject) => {
            axios({
                method: 'post',
                url: verify_api,
                data: "address=" + address
            }).then(async res => {
                const guid = res.data.result
                pending = true
                proxy_address = ''
                while (pending) {
                    if (pending) {
                        await common_utils.sleep([0.5, 1.5])
                    }
                    await checkProxyVerification(guid, check_verify_api)
                }
                resolve(proxy_address.trim())
            }).catch(err => {
                console.log(err)
                reject('校验代理合约地址失败！')
            })
        })
    },
    async getTokenSymbol(chain, contract_address, abi) {
        const provider = await provider_utils.get_provider(chain)
        const contract = new ethers.Contract(contract_address, abi, provider);
        return contract.symbol()
    }
}

export default token_utils
