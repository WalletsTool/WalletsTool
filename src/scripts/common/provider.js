import {ethers} from "ethers";
import {RpcProvider} from "starknet";
import * as web3 from "@solana/web3.js";

const providers = {

    eth_provider() {
        // rpc 节点
        const rpc_list = [
            // 'https://eth-mainnet.g.alchemy.com/v2/vdFJRkQyX2rcaEB2ChCDxLYOuWGB35iN', // alchemy_rpc
            'https://rpc.ankr.com/eth/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f', // ankr_rpc
            'https://eth-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5', // nodereal_rpc
            'https://lb.drpc.org/ogrpc?network=ethereum&dkey=Aj6S6lY4rEHYqHuH8SYuK888OpJEh1oR7qTOrkUU-y5L',
            'https://lb.drpc.org/ogrpc?network=ethereum&dkey=AkWhWFymO0apghWs6SF5_xkf0KP5iboR7qyeYkscDoZX',
            'https://lb.drpc.org/ogrpc?network=ethereum&dkey=AultnUOb_UljhLAc-67X1B0z3XyyiboR7qyfYkscDoZX',
            'https://lb.drpc.org/ogrpc?network=ethereum&dkey=AhUpGP3UjEWFi43jVViykOJE8l5ViboR7qygYkscDoZX',
            'https://lb.drpc.org/ogrpc?network=ethereum&dkey=AiFEuYjURUQhjCYFVsM-p21raG5CiboR7qyhYkscDoZX',
            'https://lb.drpc.org/ogrpc?network=ethereum&dkey=Aul6PCsUdU3qkN-S8I1_jc192qL7iboR7qyiYkscDoZX',
            'https://lb.drpc.org/ogrpc?network=ethereum&dkey=Ag9jpsjOeE93obtrzA2QU-dqVmXLibsR7qyjYkscDoZX',
            'https://lb.drpc.org/ogrpc?network=ethereum&dkey=Ai5Kc-Ex00urk5TinvqOthx7m4cBibsR7qykYkscDoZX',
            'https://lb.drpc.org/ogrpc?network=ethereum&dkey=ApdGLgvPqETzn8lU2dyrXfOM3dC0ibsR7qylYkscDoZX',
            'https://1rpc.io/3M4d7MevhTuT5ZLnv/eth',
            'https://1rpc.io/5XAZMkdvbiqyYBme/eth',
            'https://1rpc.io/45XiMaBJtP59Scipz/eth',
            'https://1rpc.io/48NBoEKWaum3t5c5T/eth',
            'https://1rpc.io/iN6L3qTAAwqFhB6r/eth',
            'https://1rpc.io/4JQrfLN2popHUByWB/eth',
            'https://1rpc.io/2n2mJNa2RUTGbRRDm/eth',
            'https://1rpc.io/cPRpCZQyf88TzQ4o/eth',
            'https://1rpc.io/2TapqZzuoodPZ86aH/eth',
            'https://1rpc.io/4qbGLbkHk2LbDP3pG/eth',
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
    base_provider() {
        // rpc 节点
        const rpc_list = [
            // 'https://base.blockpi.network/v1/rpc/public',
            'https://base.publicnode.com',
            'https://base.meowrpc.com',
            // 'https://1rpc.io/base',
            'https://1rpc.io/3MianwKGB8TzTeNwU/base',
            'https://1rpc.io/aVxuCztV8m2Wnc38/base',
            'https://1rpc.io/3RrFQ54LaLkSH5EFU/base',
            'https://base.blockpi.network/v1/rpc/0ae3552e01375f193365606394b71d5426c8bba5',
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 8453)
    },
    sol_provider() {
        // rpc 节点
        const rpc_list = [
            "https://practical-cosmological-water.solana-mainnet.quiknode.pro/125c32dc05fba63cfded06a571800d7dbda19ea9/",
            "https://go.getblock.io/419b279a3b614544b0d4801841fce66e",
            "https://mainnet.helius-rpc.com/?api-key=0b1e751c-26d1-457c-beeb-063b580d4484",
            "https://mainnet.helius-rpc.com/?api-key=9f11729e-2325-41f2-8d00-11a729b71dd2",
            "https://mainnet.helius-rpc.com/?api-key=fc75f223-8e95-4f79-8d5e-045766e19b92",
            "https://mainnet.helius-rpc.com/?api-key=fc75f223-8e95-4f79-8d5e-045766e19b92",
            "https://mainnet.helius-rpc.com/?api-key=86893688-2016-4d8f-949f-1cbdd5b7d077",
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]
        return new web3.Connection(rpc_url);
        // return new web3.Connection(web3.clusterApiUrl("testnet"));
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
    sepolia_provider() {
        // rpc 节点
        const rpc_list = [
            "https://lb.drpc.org/ogrpc?network=sepolia&dkey=As1VeG7g_U0olSnGHJaMTiWBnGj0jZYR7qzsYkscDoZX",
            "https://lb.drpc.org/ogrpc?network=sepolia&dkey=AhUpGP3UjEWFi43jVViykOJE8l5ViboR7qygYkscDoZX",
            "https://1rpc.io/45XiMaBJtP59Scipz/sepolia",
            "https://1rpc.io/48NBoEKWaum3t5c5T/sepolia",
            "https://sepolia.gateway.tenderly.co/2kEvvcRXQhP5UfscwWHs9h",
            "https://sepolia.infura.io/v3/45bfe9cba3d74311bb798ad9e52224cc"
            // 'https://api.zan.top/node/v1/eth/sepolia/public',
            // 'https://ethereum-sepolia.blockpi.network/v1/rpc/public',
            // 'https://rpc2.sepolia.org',
            // 'https://ethereum-sepolia.publicnode.com',
            // 'https://eth-sepolia.public.blastapi.io',
            // 'https://rpc.notadegen.com/eth/sepolia',
            // 'https://sepolia.gateway.tenderly.co',
            // 'https://gateway.tenderly.co/public/sepolia',
            // 'https://1rpc.io/sepolia',
            // 'https://rpc.sepolia.org'
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 11155111)
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
            'https://bsc-dataseed3.bnbchain.org',
            'https://bsc.publicnode.com',
            'https://bsc.drpc.org',
            // 'https://bsc-pokt.nodies.app',
            'https://bsc.rpc.blxrbdn.com',
            'https://bsc-dataseed1.ninicoin.io',
            'https://bsc-dataseed2.ninicoin.io',
            'https://bsc-dataseed3.ninicoin.io',
            'https://bsc-dataseed4.ninicoin.io',
            'https://bsc-dataseed1.bnbchain.org',
            'https://bsc-dataseed2.bnbchain.org',
            'https://bsc-dataseed3.bnbchain.org',
            'https://bsc-dataseed4.bnbchain.org',
            'https://bsc-dataseed1.defibit.io',
            'https://bsc-dataseed2.defibit.io',
            'https://bsc-dataseed3.defibit.io',
            'https://bsc-dataseed4.defibit.io',
            'https://bsc.blockpi.network/v1/rpc/public',
            // 'https://api.zan.top/node/v1/bsc/mainnet/public',
            'https://rpc.ankr.com/bsc',
            'https://bsc.rpc.blxrbdn.com',
            'https://binance.nodereal.io',
            'https://rpc-bsc.48.club',
            'https://koge-rpc-bsc.48.club',
            'https://1rpc.io/bnb',
            'https://rpc.ankr.com/bsc/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f',
            'https://binance.llamarpc.com',
            'https://1rpc.io/23AFBMJh4ZBfrMnw1/bnb'
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]
        console.log(rpc_url)
        return new ethers.providers.JsonRpcProvider(rpc_url, 56)
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
            'https://1rpc.io/2okGaNfQNRfFPgNnF/arb',
            'https://1rpc.io/5ZVMYTt5MvLGfEiWQ/arb',
            'https://1rpc.io/2ubJy9orGoKBFhUWS/arb',
            // 'https://lb.drpc.org/ogrpc?network=arbitrum&dkey=ApBlPnttBUG_v-Cq3zgItbXHK_CPdgYR7qPoxqxINsn1',
            // 'https://lb.drpc.org/ogrpc?network=arbitrum&dkey=ApjYQXk_j0HchUDGeOqTkB3hqHawdgYR7qPpxqxINsn1',
            // 'https://lb.drpc.org/ogrpc?network=arbitrum&dkey=ApbWc9z0mkqknrFOS-qrjxP0XaINdgYR7qPqxqxINsn1'
            // 'https://arbitrum.llamarpc.com',
            // 'https://api.zan.top/node/v1/arb/one/public',
            'https://arbitrum-one.public.blastapi.io',
            'https://arbitrum.blockpi.network/v1/rpc/public',
            // 'https://endpoints.omniatech.io/v1/arbitrum/one/public',
            // 'https://arbitrum.meowrpc.com',
            // 'https://rpc.arb1.arbitrum.gateway.fm',
            'https://1rpc.io/arb',
            // 'https://arb1.arbitrum.io/rpc',
            // 'https://arb-mainnet-public.unifra.io',
            // 'https://arbitrum.drpc.org',
            // 'https://arbitrum-one.publicnode.com',
            // 'https://arb-mainnet.g.alchemy.com/v2/LEr77rzSUl_f-aQIceeXUlkwmB6Pg9rE', // alchemy_rpc
            'https://arb-pokt.nodies.app',
            'https://rpc.ankr.com/arbitrum/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f', // ankr_rpc
            // 'https://open-platform.nodereal.io/0f6a7df001924b749c9466dc0bdb99c5/arbitrum-nitro/', // nodereal_rpc
            // 'https://arbitrum.blockpi.network/v1/rpc/e6d636b1830e242e23c62700ca034a9c1100a040', // blockpi_rpc
            // 'https://indulgent-holy-wave.arbitrum-mainnet.quiknode.pro/0f8965f469465c10e8efe4887e53bcf24f876a7e/', // quiknode_rpc
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
            'https://starknet-mainnet.public.blastapi.io',
            'https://starknet-mainnet.blastapi.io/f2ea9ebb-284e-420a-99a5-19b7e704fb7c',
            'https://rpc.starknet.lava.build',
            'https://g.w.lavanet.xyz:443/gateway/strk/rpc-http/61866b5d1622bcc43c94ce212c6e4670',
            'https://starknet-mainnet.g.alchemy.com/v2/VWNr68s6Mtu-64sFd9QCizSh_ouHhubR',
            'https://starknet-mainnet.infura.io/v3/45bfe9cba3d74311bb798ad9e52224cc',
            'https://starknet-mainnet.infura.io/v3/4d0122b8d16f414285430b8d4f1804c6',
            'https://starknet-mainnet.infura.io/v3/8f5e3edada1648059f5ef3267a329e44',
            'https://starknet-mainnet.core.chainstack.com/1e9ecb7dab8203e0bbc903bd2df07a39',
            'https://1rpc.io/starknet',
            'https://1rpc.io/45XiMaBJtP59Scipz/starknet',
            'https://1rpc.io/3xrU2Z2aXfPGWgQLJ/starknet',
            'https://1rpc.io/3JZuWQxZKuLd4cuLq/starknet',
            'https://1rpc.io/3T6KHnt8263GyK7wx/starknet',
            'https://starknet.blockpi.network/v1/rpc/e351fcf1891754a0daa12f7913f4f925908976aa'
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
        } else if (key === 'sepolia') {
            return providers.sepolia_provider()
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
        } else if (key === 'base') {
            return providers.base_provider()
        } else if (key === 'sol') {
            return providers.sol_provider()
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
