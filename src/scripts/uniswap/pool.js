import IUniswapV3PoolABI from '@uniswap/v3-core/artifacts/contracts/interfaces/IUniswapV3Pool.sol/IUniswapV3Pool.json'
import {computePoolAddress} from '@uniswap/v3-sdk'
import {POOL_FACTORY_CONTRACT_ADDRESS} from './constants'
import {ethers} from 'ethers'
import {utils as provider_utils} from "@/scripts/common/provider.js";
export async function getPoolInfo(CurrentConfig) {
    // 随机获取rpc服务
    const provider = provider_utils.get_provider(CurrentConfig.chain)
    if (!provider) {
        throw new Error('No provider')
    }

    const currentPoolAddress = computePoolAddress({
        factoryAddress: POOL_FACTORY_CONTRACT_ADDRESS,
        tokenA: CurrentConfig.tokens.in,
        tokenB: CurrentConfig.tokens.out,
        fee: CurrentConfig.tokens.poolFee,
    })

    const poolContract = new ethers.Contract(
        currentPoolAddress,
        IUniswapV3PoolABI.abi,
        provider
    )

    const [token0, token1, fee, tickSpacing, liquidity, slot0] =
        await Promise.all([
            poolContract.token0(),
            poolContract.token1(),
            poolContract.fee(),
            poolContract.tickSpacing(),
            poolContract.liquidity(),
            poolContract.slot0(),
        ])

    return {
        token0,
        token1,
        fee,
        tickSpacing,
        liquidity,
        sqrtPriceX96: slot0[0],
        tick: slot0[1],
    }
}
