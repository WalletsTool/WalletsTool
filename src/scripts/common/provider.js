import {ethers} from "ethers";
import {RpcProvider} from "starknet";
import * as web3 from "@solana/web3.js";
import {Provider} from "zksync-ethers";

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
            // "https://api.mainnet-beta.solana.com",
            // "https://rpc.ankr.com/solana",
            // "https://api.tatum.io/v3/blockchain/node/solana-mainnet",
            "https://rpc.ankr.com/solana/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f"
            // "https://practical-cosmological-water.solana-mainnet.quiknode.pro/125c32dc05fba63cfded06a571800d7dbda19ea9/",
            // "https://practical-cosmological-water.solana-mainnet.quiknode.pro/125c32dc05fba63cfded06a571800d7dbda19ea9/",
            // "https://go.getblock.io/419b279a3b614544b0d4801841fce66e",
            // "https://mainnet.helius-rpc.com/?api-key=0b1e751c-26d1-457c-beeb-063b580d4484",
            // "https://mainnet.helius-rpc.com/?api-key=9f11729e-2325-41f2-8d00-11a729b71dd2",
            // "https://mainnet.helius-rpc.com/?api-key=fc75f223-8e95-4f79-8d5e-045766e19b92",
            // "https://mainnet.helius-rpc.com/?api-key=fc75f223-8e95-4f79-8d5e-045766e19b92",
            // "https://mainnet.helius-rpc.com/?api-key=86893688-2016-4d8f-949f-1cbdd5b7d077",
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
    holesky_provider() {
        // rpc 节点
        const rpc_list = [
            'https://ethereum-holesky.blockpi.network/v1/rpc/public',
            'https://rpc.holesky.ethpandaops.io',
            'https://holesky.drpc.org',
            'https://rpc-holesky.rockx.com',
            'https://endpoints.omniatech.io/v1/eth/holesky/public',
            'https://ethereum-holesky-rpc.publicnode.com',
            // 'https://1rpc.io/holesky'
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 17000)
    },
    scroll_eth_provider() {
        // rpc 节点
        const rpc_list = [
            'https://scroll-mainnet-public.unifra.io',
            'https://scroll.blockpi.network/v1/rpc/public',
            'https://scroll.drpc.org',
            'https://scroll-mainnet.public.blastapi.io',
            'https://scroll-mainnet.rpc.grove.city/v1/a7a7c8e2',
            'https://1rpc.io/scroll',
            'https://rpc.ankr.com/scroll',
            'https://scroll-mainnet.chainstacklabs.com',
            // 'https://scroll-alphanet.blastapi.io/d6df6b9f-6b9a-470e-b529-708c36a65c32', // blastapi_rpc
            // 'https://scroll-alphanet.public.blastapi.io',
            // 'https://alpha-rpc.scroll.io/l2',
            // 'https://scroll-testnet.blockpi.network/v1/rpc/public',
            // 'https://scroll-alpha-public.unifra.io',
            // 'https://scroll-alpha.unifra.io/v1/5c1ce5f1ab9b4fc6b063b8e9afc58f5c', // unifra_rpc
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 534352)
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
    dym_provider() {
        // rpc 节点
        const rpc_list = [
            'https://dymension.rpc.thirdweb.com',
            'https://dymension.rpc.thirdweb.com/4a85f2ac23cbbd2e6a5769192e22b003',
            'https://dymension.rpc.thirdweb.com/8c03e317e55e7c1096d139af8a828b57',
            'https://dymension.rpc.thirdweb.com/1d746d0ca54d6206f9abb519b0c8b83f',
            'https://dymension-evm.blockpi.network/v1/rpc/public',
            'https://dymension.blockpi.network/rpc/v1/public',
            'https://dymension.blockpi.network/rpc/v1/2fcd3420fc2747ccd2ca767a237320ea9f76c04a',
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 1100)
    },
    zksync_provider() {
        // rpc 节点
        const rpc_list = [
            'https://mainnet.era.zksync.io',
            'https://zksync.drpc.org',
            'https://mainnet.era.zksync.io',
            'https://1rpc.io/zksync2-era',
            'https://zksync.meowrpc.com',
            'https://zksync-era.blockpi.network/v1/rpc/public'
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new Provider(rpc_url)
    },
    evmos_provider() {
        // rpc 节点
        const rpc_list = [
            'https://evmos.lava.build/lava-referer-f0790e27-cdd3-43ae-b782-899ee1e4e85a/',
            'https://evmos.lava.build/lava-referer-d800d161-70ca-4622-b021-14c66b239abb',
            'https://evmos.lava.build/lava-referer-0740b2e8-589e-4b9b-952d-35e6d81a2200',
            'https://evmos.lava.build/lava-referer-6a87a9bb-3c94-4bd2-a952-698d172fa977',
            'https://evmos.lava.build/lava-referer-5d35ec37-5a32-46d1-9cb7-ed6ca6fc4cb9',
            'https://evmos.lava.build/lava-referer-71975c3d-f9c1-49c3-980b-5f6488514cc4',
            'https://evmos.lava.build/lava-referer-c2f3aa87-afea-4186-8bf5-f4bddc11984e',
            'https://evmos.lava.build/lava-referer-7cb6c08d-b76d-4189-a031-5f9cfddcc59a',
            'https://evmos.lava.build/lava-referer-ae702181-a9b5-4d14-8620-f93f85edfc56',
            'https://evmos.lava.build/lava-referer-0b5f89e5-d88d-410d-851f-21d31e61059c',
            'https://evmos.lava.build/lava-referer-2ac29134-318f-4aaf-9499-0ef6e6648ad4',
            'https://evmos.lava.build/lava-referer-5b58c9d0-ba5a-469d-94d8-f493f4c6cb77',
            'https://evmos.lava.build/lava-referer-1c87f12c-9bae-42a6-af5a-c759e30dac44',
            'https://evmos.lava.build/lava-referer-83c29fde-dfd3-4d8e-9529-0e1b3f910262',
            'https://evmos.lava.build/lava-referer-f318f334-3944-4632-b2dc-feec14a9eab9',
            'https://evmos.lava.build/lava-referer-ffe60a9a-8fe4-4a22-9878-fcf191e77f2c',
            'https://evmos.lava.build/lava-referer-7fa0df7d-4d55-4689-b105-a8425f2ec5bc',
            'https://evmos.lava.build/lava-referer-e6188861-fc54-4a96-bb03-2ca289733cda',
            'https://evmos.lava.build/lava-referer-016ee2bd-f449-4e60-928a-a20d9a72b185',
            'https://evmos.lava.build/lava-referer-ee8c3ad2-00aa-48f0-aee3-2f100b26e30d',
            'https://evmos.lava.build/lava-referer-a73531ec-eac4-459a-b382-fc0a269a2596',
            'https://evmos.lava.build/lava-referer-0f42fb33-29e1-4b72-9211-3dc9bb249ebf',
            'https://evmos.lava.build/lava-referer-9135517f-baa9-41b8-a5da-7c9ede3bcc65',
            'https://evmos.lava.build/lava-referer-6b56d47a-6942-4903-a47f-4e2fa42dae38',
            'https://evmos.lava.build/lava-referer-2153e643-32da-4b67-9be7-2f0560eda1ca',
            'https://evmos.lava.build/lava-referer-71fc6633-016a-45dd-b298-b2078f7987c2',
            'https://evmos.lava.build/lava-referer-c82d3b69-5c52-446d-af0b-a3620e56a800',
            'https://evmos.lava.build/lava-referer-625dcd47-23f7-4adc-9147-9acdb818bb92',
            'https://evmos.lava.build/lava-referer-52616f4d-c098-439d-9f6a-c2cb03f980f6',
            'https://evmos.lava.build/lava-referer-58597052-67e7-4d41-8e69-685c51f4873f',
            'https://evmos.lava.build/lava-referer-59355cb3-959e-4ea1-a328-6b662d994e83',
            'https://evmos.lava.build/lava-referer-4c6a5538-9c22-47d5-82c6-9f3f833861a7',
            'https://evmos.lava.build/lava-referer-087cbae2-ce96-4c48-8534-d8c61cdff81c',
            'https://evmos.lava.build/lava-referer-43aa7161-6c88-43da-beb7-f3dd0529b652',
            'https://evmos.lava.build/lava-referer-27e37e53-87af-49d8-a472-cd3ced852b21',
            'https://evmos.lava.build/lava-referer-661bb6c0-dfd4-4045-8ca9-927d8abe8992',
            'https://evmos.lava.build/lava-referer-41eb8b4e-1ca6-48a4-ba50-7ab3d4df9f19',
            'https://evmos.lava.build/lava-referer-206e54f3-716a-4c57-a979-e3ad8439b1b8',
            'https://evmos.lava.build/lava-referer-4d061805-5676-4805-9cf4-30e506d37be7',
            'https://evmos.lava.build/lava-referer-a27cba2f-b517-4c8d-88b0-d133d74c574e',
            'https://evmos.lava.build/lava-referer-b7756956-7534-498f-8364-375c9088ffa5',
            'https://evmos.lava.build/lava-referer-21cf1092-bd1a-4422-9cde-16f96f232e54',
            'https://evmos.lava.build/lava-referer-ac4c2340-bdce-4314-8123-7e2fb73a5cdc',
            'https://evmos.lava.build/lava-referer-d87a7420-8250-4c35-9b17-1376822b9c96',
            'https://evmos.lava.build/lava-referer-fd13ebe6-7ad2-406e-b598-995d4036b4e3',
            'https://evmos.lava.build/lava-referer-067208cc-1117-474c-8fb6-016fc07b71f2',
            'https://evmos.lava.build/lava-referer-d058201c-14be-4e85-9e1d-3f8a41450b9e',
            'https://evmos.lava.build/lava-referer-8b40a2a8-f76a-477c-a5ae-8b418f168b3c',
            'https://evmos.lava.build/lava-referer-f91d5e50-931e-4eeb-a59c-b3fec81956ee',
            'https://evmos.lava.build/lava-referer-16cecf5b-55e0-4380-a89e-745fb1877b12',
            'https://evmos.lava.build/lava-referer-2fbea34c-8121-4e0b-9d23-6b199c149092',
            'https://evmos.lava.build/lava-referer-e162ad75-b3da-4ac7-a452-a9bc919a400d',
            'https://evmos.lava.build/lava-referer-1c88a9a2-caec-43e7-99d0-7966df907934',
            'https://evmos.lava.build/lava-referer-ad5828a1-f7f7-4b83-a78b-4cd2bf0d5d20',
            'https://evmos.lava.build/lava-referer-40640ef0-79c8-4ac7-8624-a1aaa5b4493c',
            'https://evmos.lava.build/lava-referer-05435a5a-3576-48df-8379-b733f5e87655',
            'https://evmos.lava.build/lava-referer-968858ca-29a6-4513-8b8f-851bc2a263be',
            'https://evmos.lava.build/lava-referer-d6548e49-0be2-49b1-aae2-45d742bda137',
            'https://evmos.lava.build/lava-referer-a9d8a634-303a-45b1-9fac-d77218fcc1fc',
            'https://evmos.lava.build/lava-referer-be11e59f-18ab-4127-b58f-e71a925c6a77',
            'https://evmos.lava.build/lava-referer-452f84f5-d529-4756-9bdd-29225f2fda61',
            'https://evmos.lava.build/lava-referer-5df1e083-80f7-4896-9fad-6dc5872b3843',
            'https://evmos.lava.build/lava-referer-515735e7-e75d-45cc-9584-4d89c59fc08f',
            'https://evmos.lava.build/lava-referer-50055057-5b3e-4b30-9ee1-1c56800e8a35',
            'https://evmos.lava.build/lava-referer-998bc6ad-95ed-4beb-ab19-a31e2c5dc3f3',
            'https://evmos.lava.build/lava-referer-d512912d-c408-4d6f-8f50-77b75f009478',
            'https://evmos.lava.build/lava-referer-1c3a901b-9357-4184-913d-0a2ddf217745',
            'https://evmos.lava.build/lava-referer-26a427e7-256a-4f75-a7a7-6ca57eb0ad4c',
            'https://evmos.lava.build/lava-referer-a5992b55-50d0-40ac-bacb-a35b7d16e11a',
            'https://evmos.lava.build/lava-referer-e7f308b8-65b6-4a46-ba5a-f951a25d1ae4',
            'https://evmos.lava.build/lava-referer-f5f36f06-0b9c-42dd-8d10-3383e5ef0586',
            'https://evmos.lava.build/lava-referer-bbf4b03e-a495-4898-ab5b-81803aa4a8e2',
            'https://evmos.lava.build/lava-referer-8e3d7a2b-0d67-42b6-9c69-5631ab271a0b',
            'https://evmos.lava.build/lava-referer-5d3c5e4e-093c-47c9-9c3c-3679c344c6cb',
            'https://evmos.lava.build/lava-referer-a222c58c-5054-4e79-962a-122d117768f0',
            'https://evmos.lava.build/lava-referer-b66becc2-8ccd-4fc2-bd49-fc762de1a57b',
            'https://evmos.lava.build/lava-referer-5b36b81f-b80c-4d5e-8d4d-05ea9d50e8be',
            'https://evmos.lava.build/lava-referer-4e99972d-1306-4ed8-84db-9d6b38beea74',
            'https://evmos.lava.build/lava-referer-14917c24-cc5a-46aa-9ff3-8d00584982e5',
            'https://evmos.lava.build/lava-referer-8e60ec75-da67-4fc7-95a5-4981fc0ee0a7',
            'https://evmos.lava.build/lava-referer-18347b29-33a8-4973-91b7-5f7f9ed3c87e',
            'https://evmos.lava.build/lava-referer-1b0c1603-2db7-46cd-9fbf-7e4fe2886b7b',
            'https://evmos.lava.build/lava-referer-7069b314-3a63-46d5-95ac-80c9cc8f426a',
            'https://evmos.lava.build/lava-referer-410e82a4-133d-467f-8980-8e09720081d8',
            'https://evmos.lava.build/lava-referer-c8de0496-7d9d-4dd7-9ef6-092e71dc68d8',
            'https://evmos.lava.build/lava-referer-0ced04f6-a5d1-4152-a9f6-0180e4faa46e',
            'https://evmos.lava.build/lava-referer-3376cc99-219f-4331-8cfd-d1b89d194e84',
            'https://evmos.lava.build/lava-referer-820a411f-c668-444d-bdc5-cc487debd343',
            'https://evmos.lava.build/lava-referer-05f9d0bf-049a-4cc3-8434-1028db320285',
            'https://evmos.lava.build/lava-referer-4fca38a3-200f-49ec-a61e-8fcc7ab03b28',
            'https://evmos.lava.build/lava-referer-58f1f6f6-300b-4d66-86a6-32b363d7a3c2',
            'https://evmos.lava.build/lava-referer-b1c112a2-bda5-47f1-8854-31054590839b',
            'https://evmos.lava.build/lava-referer-d0d0bac7-7699-4548-a24d-24bf421d4395',
            'https://evmos.lava.build/lava-referer-58850613-dbed-42e9-bacc-9a284c14ebf3',
            'https://evmos.lava.build/lava-referer-7f2f757f-1028-4a05-bc30-a51f21e25845',
            'https://evmos.lava.build/lava-referer-a94caf6f-5d53-4cd5-bad5-5bae7fccbaa6',
            'https://evmos.lava.build/lava-referer-636b4507-33cd-421a-a272-1299f5ff30d6',
            'https://evmos.lava.build/lava-referer-8eb793f3-61a0-4bf7-9830-cb8d3754f3f7',
            'https://evmos.lava.build/lava-referer-d62dc620-777b-4aee-aabc-3e93d819097d',
            'https://evmos.lava.build/lava-referer-f11ed669-79b6-4f90-96b9-d83dc09d7eaa',
            'https://evmos.lava.build/lava-referer-905c27e1-bdf4-4547-9959-4e70fa8806bd',
            'https://evmos.lava.build/lava-referer-a38ba2f4-eb18-4f6c-99d3-00675e57c1af',
            'https://evmos.lava.build/lava-referer-c419a5a3-e54a-4927-a19b-adc41925349a',
            'https://evmos.lava.build/lava-referer-1663c51f-acdb-44cb-8f32-8ddf478e459e',
            'https://evmos.lava.build/lava-referer-26fc419e-9dd9-42b4-99c2-6244a258ae6b',
            'https://evmos.lava.build/lava-referer-b8577187-16b5-4d63-ae7f-21e26eaa2a89',
            'https://evmos.lava.build/lava-referer-74e7d485-2525-4fa2-9e28-00a42ff72084',
            'https://evmos.lava.build/lava-referer-8cf8eb7c-232f-4932-b189-89fe0ec9119a',
            'https://evmos.lava.build/lava-referer-d5988b10-1d0e-4bd0-89c6-aba4f8ba9d9a',
            'https://evmos.lava.build/lava-referer-e84ed555-3b76-42f9-9254-ccbcc5f4dfa8',
            'https://evmos.lava.build/lava-referer-7c6e93f7-8f9b-4f66-aae1-24eee8aa3838',
            'https://evmos.lava.build/lava-referer-d30e2bd5-cef5-412e-af87-3c7a42d75b2c',
            'https://evmos.lava.build/lava-referer-f0d0d127-9cdf-4b4c-a061-8511c4852449',
            'https://evmos.lava.build/lava-referer-c5b89aba-514a-4f52-8f8c-49248009bb00',
            'https://evmos.lava.build/lava-referer-0df5339e-f013-4167-9c49-291b62307f75',
            'https://evmos.lava.build/lava-referer-790eefab-46ea-4eb8-9668-bc036aa398ce',
            'https://evmos.lava.build/lava-referer-51c7162f-1ffa-4a2c-baa4-2358d1af20f0',
            'https://evmos.lava.build/lava-referer-03d43c4a-0e3a-4a85-88d4-92bcf563ff94',
            'https://evmos.lava.build/lava-referer-8a1cfe5a-c56c-499d-a8f4-a2c953e75d3f',
            'https://evmos.lava.build/lava-referer-78ee0518-bc26-4899-87ef-639bef9efd75',
            'https://evmos.lava.build/lava-referer-2fcb61f6-bd70-419b-831a-993d3fe1d451',
            'https://evmos.lava.build/lava-referer-12695be0-8ff0-4cbd-af0f-f51d9d0f1de8',
            'https://evmos.lava.build/lava-referer-837c4089-9717-436c-abad-244492aa8bef',
            'https://evmos.lava.build/lava-referer-61f6904f-1715-48f2-9016-f644de1824f9',
            'https://evmos.lava.build/lava-referer-8a4364be-52b4-44d6-bf6d-39f367e2e9a8',
            'https://evmos.lava.build/lava-referer-2e8cc37b-4146-4e91-bd2d-873e81d4af69',
            'https://evmos.lava.build/lava-referer-ac9069df-1aec-49c6-bcc1-0b4ff1f3aa8c',
            'https://evmos.lava.build/lava-referer-53e0eb9a-d6e7-40a5-95d6-36dff0338b8d',
            'https://evmos.lava.build/lava-referer-cb162533-bfc2-4a9b-9699-2dd3fdb08088',
            'https://evmos.lava.build/lava-referer-6c4b68df-741b-4e2a-a978-0aa70e553460',
            'https://evmos.lava.build/lava-referer-4c606644-2750-4d2e-bf75-bfc487e3a3f0',
            'https://evmos.lava.build/lava-referer-955ba372-3e10-4501-801f-0ec4df70d7d3',
            'https://evmos.lava.build/lava-referer-49e02c4f-b316-485d-aa16-ea2655f64d8f',
            'https://evmos.lava.build/lava-referer-be4371d4-ddaf-4276-9f4a-5958da9329cf',
            'https://evmos.lava.build/lava-referer-e57a170d-67c5-49b0-86ad-58624cd1a016',
            'https://evmos.lava.build/lava-referer-f8321639-3b94-4f2d-8da2-4ae425f1e614',
            'https://evmos.lava.build/lava-referer-27411673-2920-42e9-9186-d7f73b80c6be',
            'https://evmos.lava.build/lava-referer-3fec0c7d-9b5f-4e1a-b0bb-f288fb468a63',
            'https://evmos.lava.build/lava-referer-aede1341-7212-4884-99a2-50119512bebc',
            'https://evmos.lava.build/lava-referer-49f754f1-8dc5-49c9-bdfa-411e03ab1a9f',
            'https://evmos.lava.build/lava-referer-41e078ca-3b89-473a-812e-ecc1ba99e204',
            'https://evmos.lava.build/lava-referer-3eb78870-b934-4363-957d-27c3b1f7b260',
            'https://evmos.lava.build/lava-referer-ca8a9c57-08aa-4290-8abe-8686ea925f39',
            'https://evmos.lava.build/lava-referer-492e919c-a883-4db1-8c31-04a6cc75ed12',
            'https://evmos.lava.build/lava-referer-8aba0007-cefd-4783-ae88-32515b07f912',
            'https://evmos.lava.build/lava-referer-b05fe7ff-f3fb-4a30-a241-304437c154eb',
            'https://evmos.lava.build/lava-referer-6f9560c7-aec0-4056-b016-180c12dce5ac',
        ]
        const rpc_url = rpc_list[Math.floor(Math.random() * rpc_list.length)]

        return new ethers.providers.JsonRpcProvider(rpc_url, 9001)
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
        } else if (key === 'holesky') {
            return providers.holesky_provider()
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
        } else if (key === 'dym') {
            return providers.dym_provider()
        } else if (key === 'evmos') {
            return providers.evmos_provider()
        } else if (key === 'zksync') {
            return providers.zksync_provider()
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
