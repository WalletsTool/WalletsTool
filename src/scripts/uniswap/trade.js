import {CurrencyAmount, Percent, SupportedChainId, Token, TradeType,} from "@uniswap/sdk-core";
import {FeeAmount, Pool, Route, SwapQuoter, SwapRouter, Trade} from "@uniswap/v3-sdk";
import JSBI from "jsbi";
import {BigNumber, ethers} from "ethers";
import {
    ERC20_ABI,
    MAX_FEE_PER_GAS, MAX_PRIORITY_FEE_PER_GAS,
    QUOTER_CONTRACT_ADDRESS,
    SWAP_ROUTER_ADDRESS,
    TOKEN_AMOUNT_TO_APPROVE_FOR_TRANSFER
} from "./constants";
import {getPoolInfo} from "@/scripts/uniswap/pool.js";
import {utils as provider_utils} from "@/scripts/common/provider.js";

const WETH_TOKEN = new Token(
    SupportedChainId.MAINNET,
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    18,
    "WETH",
    "Wrapped Ether"
);

const USDC_TOKEN = new Token(
    SupportedChainId.MAINNET,
    "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    6,
    "USDC",
    "USD//C"
);

const CurrentConfig = {
    chain: 'eth',
    slippageTolerance: new Percent(50, 10_000),
    deadline: Math.floor(Date.now() / 1000) + 60 * 20,
    wallet: {
        privateKey: '0x',
    },
    tokens: {
        in: WETH_TOKEN,
        amountIn: 1,
        out: USDC_TOKEN,
        poolFee: FeeAmount.MEDIUM,
    },
};

// 创建交易
async function createAndExecTrade(CurrentConfig) {
    const provider = provider_utils.get_provider(CurrentConfig.chain)
    const wallet = new ethers.Wallet(CurrentConfig.wallet.privateKey, provider)
    const walletAddress = wallet.address

    if (!walletAddress || !provider) {
        throw new Error('创建 provider 或 wallet 失败')
    }

    const poolInfo = await getPoolInfo(CurrentConfig);

    const pool = new Pool(
        CurrentConfig.tokens.in,
        CurrentConfig.tokens.out,
        CurrentConfig.tokens.poolFee,
        poolInfo.sqrtPriceX96.toString(),
        poolInfo.liquidity.toString(),
        poolInfo.tick
    );

    const swapRoute = new Route(
        [pool],
        CurrentConfig.tokens.in,
        CurrentConfig.tokens.out
    );

    const amountOut = await getOutputQuote(provider, CurrentConfig, swapRoute);
    // 创建交易
    const trade = Trade.createUncheckedTrade({
        route: swapRoute,
        inputAmount: CurrencyAmount.fromRawAmount(
            CurrentConfig.tokens.in,
            ethers.utils
                .parseUnits(
                    CurrentConfig.tokens.amountIn.toString(),
                    CurrentConfig.tokens.in.decimals
                )
                .toString()
        ),
        outputAmount: CurrencyAmount.fromRawAmount(
            CurrentConfig.tokens.out,
            JSBI.BigInt(amountOut)
        ),
        tradeType: TradeType.EXACT_INPUT,
    });

    // 授权
    const tokenApproval = await getTokenTransferApproval(wallet, provider, CurrentConfig.tokens.in)
    // 如果授权失败，直接返回
    if (tokenApproval !== 'Sent') {
        return 'Failed'
    }
    // 设置交易参数 slippageTolerance: 滑点容忍度，deadline: 交易截止时间
    const options = {
        slippageTolerance: CurrentConfig.slippageTolerance, // 50 bips, or 0.50%
        deadline: CurrentConfig.deadline, // 20 minutes from the current Unix time
        recipient: walletAddress,
    }
    // 获取交易参数
    const methodParameters = SwapRouter.swapCallParameters([trade], options)

    const tx = {
        data: methodParameters.calldata,
        to: SWAP_ROUTER_ADDRESS,
        value: methodParameters.value,
        from: walletAddress,
        maxFeePerGas: MAX_FEE_PER_GAS,
        maxPriorityFeePerGas: MAX_PRIORITY_FEE_PER_GAS,
    }
    // 发送交易
    return await sendTransaction(tx)
}


async function getOutputQuote(provider, CurrentConfig, route) {

    const {calldata} = await SwapQuoter.quoteCallParameters(
        route,
        CurrencyAmount.fromRawAmount(
            CurrentConfig.tokens.in,
            ethers.utils
                .parseUnits(
                    CurrentConfig.tokens.amountIn.toString(),
                    CurrentConfig.tokens.in.decimals
                )
                .toString()
        ),
        TradeType.EXACT_INPUT,
        {
            useQuoterV2: true,
        }
    );

    const quoteCallReturnData = await provider.call({
        to: QUOTER_CONTRACT_ADDRESS,
        data: calldata,
    });

    return ethers.utils.defaultAbiCoder.decode(["uint256"], quoteCallReturnData);
}

async function getTokenTransferApproval(wallet, provider, token) {
    try {
        const tokenContract = new ethers.Contract(
            token.address,
            ERC20_ABI,
            provider
        )

        const transaction = await tokenContract.populateTransaction.approve(
            SWAP_ROUTER_ADDRESS,
            ethers.utils.parseUnits(TOKEN_AMOUNT_TO_APPROVE_FOR_TRANSFER.toString(), token.decimals).toString()
        )

        return sendTransaction(wallet, provider, {
            ...transaction,
            from: wallet.address,
        })
    } catch (e) {
        console.error(e)
        return 'Failed'
    }
}

async function sendTransaction(wallet, provider, transaction) {
    if (transaction.value) {
        transaction.value = BigNumber.from(transaction.value)
    }
    const txRes = await wallet.sendTransaction(transaction)

    let receipt = null

    while (receipt === null) {
        try {
            receipt = await provider.getTransactionReceipt(txRes.hash)
        } catch (e) {
            console.log(`Receipt error:`, e)
            break
        }
    }

    // Transaction was successful if status === 1
    if (receipt) {
        return 'Sent'
    } else {
        return 'Failed'
    }
}
