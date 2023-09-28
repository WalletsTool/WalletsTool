import {ethers} from "ethers";
import {RpcProvider} from "starknet";

const providers = {

    eth_provider() {
        // rpc 节点
        const rpc_list = [
            'https://eth-mainnet.g.alchemy.com/v2/vdFJRkQyX2rcaEB2ChCDxLYOuWGB35iN', // alchemy_rpc
            'https://rpc.ankr.com/eth/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f', // ankr_rpc
            'https://eth-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5', // nodereal_rpc
            // 'https://ethereum.blockpi.network/v1/rpc/23d1f2352ff1e80e0e5579a19b69529f01af8065', // blockpi_rpc
            // 'https://mainnet.infura.io/v3/45bfe9cba3d74311bb798ad9e52224cc', // infura_rpc
            // 'https://old-nameless-dew.quiknode.pro/8963d0cdc581f3ddf8d6a53a82b20fa308fc806d/', // quiknode_rpc
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 1)
    },
    linea_provider() {
        // rpc 节点
        const rpc_list = [
            'https://linea.blockpi.network/v1/rpc/public',
            'https://1rpc.io/linea',
            'https://rpc.linea.build',
            'https://linea.drpc.org'
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 59144)
    },
    opbnb_provider() {
        // rpc 节点
        const rpc_list = [
            'https://opbnb-mainnet-rpc.bnbchain.org',
            // 'https://opbnb-mainnet.nodereal.io/v1/64a9df0874fb4a93b9d0a3849de012d3',
            // 'https://opbnb-mainnet.nodereal.io/v1/e9a36765eb8a40b9bd12e680a1fd2bc5',
            'https://opbnb-mainnet.nodereal.io/v1/ea08c11bd0874ce19cee7fc6f63b6cf8', // yeah.net
            'https://opbnb-mainnet.nodereal.io/v1/8a1c5fbe106c422ea9c9093570ce0af2', // yahoo.com
            'https://opbnb-mainnet.nodereal.io/v1/205f773b951e46468f684b460e971093' // gmail.com
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 204)
    },
    geth_provider() {
        // rpc 节点
        const rpc_list = [
            'https://eth-goerli.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5', // nodereal_rpc
            'https://eth-goerli.nodereal.io/v1/ea08c11bd0874ce19cee7fc6f63b6cf8', // nodereal_rpc
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 5)
    },
    scroll_eth_provider() {
        // rpc 节点
        const rpc_list = [
            // 'https://scroll-alphanet.blastapi.io/d6df6b9f-6b9a-470e-b529-708c36a65c32', // blastapi_rpc
            'https://scroll-alphanet.public.blastapi.io',
            'https://alpha-rpc.scroll.io/l2',
            'https://scroll-testnet.blockpi.network/v1/rpc/public',
            'https://scroll-alpha-public.unifra.io',
            // 'https://scroll-alpha.unifra.io/v1/5c1ce5f1ab9b4fc6b063b8e9afc58f5c', // unifra_rpc
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 534353)
    },
    binance_provider() {
        // rpc 节点
        const rpc_list = [
            'https://rpc.ankr.com/bsc/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f', // ankr_rpc
            'https://bsc-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5', // nodereal_rpc
            // 'https://bsc.blockpi.network/v1/rpc/fc05a6d785ae099ec79e488f1a6168d192ac8db9', // blockpi_rpc
            'https://blue-late-rain.bsc.quiknode.pro/cbe5c8e3b1198b2cfadfe1dc6f4107c4a3de6938/', // quiknode_rpc
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 56)

        // const rpc_list = [
        //     'https://rpc.ankr.com/bsc_testnet_chapel/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f'
        // ]
        // const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]
        //
        // return new ethers.providers.JsonRpcProvider(rpc_url, 97)
    },
    polygon_provider() {
        // rpc 节点
        const rpc_list = [
            'https://polygon-mainnet.g.alchemy.com/v2/3I459iRjefnHAY8H6zQ-3mQWycNEJh25', // alchemy_rpc
            'https://rpc.ankr.com/polygon/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f', // ankr_rpc
            'https://polygon-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5', // nodereal_rpc
            // 'https://polygon.blockpi.network/v1/rpc/b81b8e70c73f7ed6794ce2091ee9b340b4c1ebb5', // blockpi_rpc
            // 'https://skilled-black-gas.matic.quiknode.pro/79466695ef6e44dd393e18b5e69b27a30523db3b/', // quiknode_rpc
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 137)
    },
    arb_provider() {
        // rpc 节点
        const rpc_list = [
            'https://arb-mainnet.g.alchemy.com/v2/LEr77rzSUl_f-aQIceeXUlkwmB6Pg9rE', // alchemy_rpc
            'https://rpc.ankr.com/arbitrum/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f', // ankr_rpc
            'https://open-platform.nodereal.io/0f6a7df001924b749c9466dc0bdb99c5/arbitrum-nitro/', // nodereal_rpc
            // 'https://arbitrum.blockpi.network/v1/rpc/e6d636b1830e242e23c62700ca034a9c1100a040', // blockpi_rpc
            'https://indulgent-holy-wave.arbitrum-mainnet.quiknode.pro/0f8965f469465c10e8efe4887e53bcf24f876a7e/', // quiknode_rpc
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 42161)
    },
    op_provider() {
        // rpc 节点
        const rpc_list = [
            'https://opt-mainnet.g.alchemy.com/v2/vnCby8geAM4QtKzZo-r4-80pyZpfb9bU', // alchemy_rpc
            'https://rpc.ankr.com/optimism/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f', // ankr_rpc
            'https://opt-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5', // nodereal_rpc
            // 'https://optimism.blockpi.network/v1/rpc/3874ee74b9c803af050b67d125f273840144394c', // blockpi_rpc
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 10)
    },
    starknet_provider() {
        // rpc 节点
        const rpc_list = [
            // 'https://starknet.w3node.com/041c6b76eaeee394d8222542a4a1ec9fe655c9072386be63161ccf5de3e4d5d6/api', // rockx
            'https://starknet-mainnet.blastapi.io/c4e6e6fb-9364-44f2-a814-d6d6dafd36ae', // blastapi
            'https://starknet-mainnet.blastapi.io/461b72a6-8955-4f1d-a558-f3ec9043b2c1', // blastapi
            'https://starknet-mainnet.gateway.pokt.network/v1/lb/0d6939d2cea79d6f19b49e8f', // protal
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]
        return new RpcProvider({
            nodeUrl: rpc_url
        })
        //     return rpc_list[Math.floor(Math.random() * rpc_list.length)]
    },
    okt_provider() {
        // rpc 节点
        const rpc_list = [
            // 'https://exchainrpc.okex.org'
            'https://oKc-mainnet.gateway.pokt.network/v1/lb/0d6939d2cea79d6f19b49e8f' // pokt
            // 'https://exchaintestrpc.okex.org'
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]
        return new ethers.providers.JsonRpcProvider(rpc_url, 66)
        // return new ethers.providers.JsonRpcProvider(rpc_url, 65)
    },
}

export const utils = {
    get_provider(key) {
        if (key === 'eth') {
            return providers.eth_provider()
        } else if (key === 'linea') {
            return providers.linea_provider()
        } else if (key === 'opbnb') {
            return providers.opbnb_provider()
        } else if (key === 'geth') {
            return providers.geth_provider()
        } else if (key === 'scroll') {
            return providers.scroll_eth_provider()
        } else if (key === 'arb') {
            return providers.arb_provider()
        } else if (key === 'op') {
            return providers.op_provider()
        } else if (key === 'binance') {
            return providers.binance_provider()
        } else if (key === 'polygon') {
            return providers.polygon_provider()
        } else if (key === 'starknet') {
            return providers.starknet_provider()
        } else if (key === 'okt') {
            return providers.okt_provider()
        }
    },
    get_base_gas_price(key) {
        return new Promise((resolve, reject) => {
            this.get_provider(key)
                .getGasPrice()
                .then((gas_price) => {
                    resolve(Number(ethers.utils.formatUnits(gas_price, 'gwei')))
                }).catch((err) => {
                reject(err)
            })
        })
    }
}
