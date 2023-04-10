<script setup>
import {ref} from "vue";
import {invoke} from "@tauri-apps/api/tauri";
import {ethers} from "ethers";
import {erc20_abi, claim_abi} from "./abis.js";
import {Alchemy, Network} from "alchemy-sdk";
import chalk from "chalk";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsg.value = await invoke("greet", {name: name.value});
}


// --------------------------------配置区域开始--------------------------------
// 设置Alchemy API 的配置信息
const settings = {
  apiKey: "vdFJRkQyX2rcaEB2ChCDxLYOuWGB35iN", // 在此处配置你的Alchemy API KEY
  network: Network.ETH_MAINNET,
};
const alchemy = new Alchemy(settings);

// 秘钥数组（配置多个私钥）
let prv_key_array = [
  "0x864441e52d0636dfe639741875e0e60ad59415ce0e32d5e79e14e69d9f4d8fe2",
  // "0x76120c9b5d2ffd5d969f2a7a6896db25e00eab4926c8d2b6ee18f94be1ccec7b",
  // "0x8369feb9e3991be53676b88479b5169843b2e0fd5954f315a8c38586feaaea04",
  // "0x2cd4050845e795c2817f67f9aa3a97bd52e7ff8c04f802e2ec36514709bbbb2a",
  // "0x78f233e69958cf3bd57f75ffe09b64e72c6a8500c15bc8ff9107d137f9941029"
]
// 目标地址数组（配置多个钱包地址，要求数量和前面秘钥的数量一致）
let destination_address_array = [
  "0x43051fc298defd3224a57dc9d4b188c862f67aee",
  // "0xaf5caed6f88cab0278d3b508c90c8fda780d62f7",
  // "0x0e351aa098b5dd3f713548b187dde1e1d54ed11a",
  // "0x523980233ca537c760730136df9efa636a7272b6",
  // "0x1c368e1afd42ad0de45208b9a925268058be2940"
]
// Arbitrum 的 rpc 节点数组
let rpc_array = [
  "https://open-platform.nodereal.io/0f6a7df001924b749c9466dc0bdb99c5/arbitrum-nitro/",
  "https://arb-mainnet.g.alchemy.com/v2/5-F_P0RDqrLDwQJo_o8g9XDqIhiIKKek",
  "https://indulgent-holy-wave.arbitrum-mainnet.discover.quiknode.pro/0f8965f469465c10e8efe4887e53bcf24f876a7e/",
  "https://arbitrum.blockpi.network/v1/rpc/e6d636b1830e242e23c62700ca034a9c1100a040"
]
// --------------------------------配置区域结束--------------------------------

let amountToClaim = [];
let currentNonce = [];

// claimContract合约
let claimContract = new ethers.Contract("0x67a24ce4321ab3af51c2d0a4801c3e111d88c9d9", claim_abi);
// ARB 代币合约
let token = new ethers.Contract("0x912CE59144191C1204E64559FE8253a0e49E6548", erc20_abi);

/**
 * 获取claim的数量以及nonce信息
 * @param rpc
 * @returns {Promise<void>}
 */
async function prepareToClaim(rpc) {
  for (let i = 0; i < prv_key_array.length; i++) {
    let provider = new ethers.providers.JsonRpcProvider(rpc, 42161);
    // 通过私钥创建钱包
    let wallet = new ethers.Wallet(prv_key_array[i], provider);
    // 获取钱包当前的nonce
    let current_nonce = await provider.getTransactionCount(wallet.address);
    // 获取钱包可以claim的数量
    let claimableAmount = await claimContract.connect(wallet).claimableTokens(wallet.address);
    amountToClaim.push(claimableAmount);
    currentNonce.push(current_nonce);
    console.log("amount to claim", amountToClaim);
    console.log("currentNonce", currentNonce);
  }
}

/**
 * claim和转账的核心方法
 * @param rpc
 * @param prv_key
 * @param current_nonce
 * @param to_addr
 * @param claimableAmount
 * @returns {Promise<void>}
 */
async function sendClaimAndTransfer(rpc, prv_key, current_nonce, to_addr, claimableAmount) {
  let provider = new ethers.providers.JsonRpcProvider(rpc, 42161);
  let wallet = new ethers.Wallet(prv_key, provider);

  try {
    // ① claim 通过claimContract合约来进行claim
    let claimTx = claimContract.connect(wallet).claim({
      gasLimit: "0x4C4B40",// 5kk in case gas on L1 is expensive.. Read about arbitrums 2D fees to learn more
      gasPrice: "0x3B9ACA00", // 1kkk = 1 gwei in case network is  overloaded
      // MAX GAS USED TO CLAIM = 0.005 eth ~= 9$
      nonce: current_nonce,
    })
  } catch (error) {
    console.log(chalk.red("error on claim occured"));
    console.log(chalk.red("wallet: ", wallet.address));
    console.log(error);
  }
  // ② 等待30ms
  await new Promise(r => setTimeout(r, 2000));
  try {
    // ③ 转发到目标地址
    let transferTx = token.connect(wallet).transferFrom(wallet.address, to_addr, claimableAmount, {
      gasLimit: "0x4C4B40",// 5kk in case gas on L1 is expensive.. Read about arbitrums 2D fees to learn more
      gasPrice: "0x3B9ACA00", // 1kkk = 1 gwei in case network is  overloaded
      // MAX GAS USED TO CLAIM = 0.005 eth ~= 9$
      nonce: current_nonce + 1,
    });
  } catch (error) {
    console.log(chalk.red("error on transfer occured.."));
    console.log(chalk.red("wallet: ", wallet.address));
    console.log(error);
  }
}

/**
 * 遍历私钥数组，对每个私钥进行claim和转账
 */
function sendMeMoneyBitch() {
  //  遍历私钥数组
  for (let i = 0; i < prv_key_array.length; i++) {
    // 调用claim和转账方法
    // 参数：rpc_array[i % rpc_array.length] 用于轮询rpc，防止某个rpc出现问题
    // 参数：prv_key_array[i] 私钥
    // 参数：currentNonce[i] 当前nonce
    // 参数：destination_address_array[i] 目标地址
    // 参数：amountToClaim[i] 要claim的数量
    sendClaimAndTransfer(rpc_array[i % rpc_array.length], prv_key_array[i], currentNonce[i], destination_address_array[i], amountToClaim[i]);
  }
}

/**
 * claim前的准备工作
 * @returns {Promise<void>}
 */
async function prepare() {
  await prepareToClaim(rpc_array[0]);
}


// ---------------------------------主流程开始---------------------------------
async function test() {
  // 输出作者信息
  console.log(chalk.green("--------------程序开始运行--------------"));

// 调用prepare方法，获取claim的数量以及nonce信息，完成准备工作
  await prepare();

// 输出目标区块高度
  console.log(chalk.blue("目标区块高度：16890400"));
  sendMeMoneyBitch();
// // 使用alchemy.ws建立长链接，监听区块高度
  alchemy.ws.on("block",
      async (blockNumber) => {
        // 输出当前区块高度
        console.log(chalk.blue("当前区块高度：", blockNumber));
        // 16890400是claim开始的区块高度，当区块高度达到16890400时，开始claim
        if (blockNumber >= 16890400) {
          // 等待700ms，确保区块高度已经达到16890400
          await new Promise(r => setTimeout(r, 700));
          // claim 并 转发
          console.log("hhhhhh  chenggongle!!!")
          // sendMeMoneyBitch();
          // 断开监听
          alchemy.ws.removeAllListeners();
          console.log(chalk.green("--------------程序结束运行--------------"));
        }
      })
}
// ---------------------------------主流程结束---------------------------------

</script>

<template>
  <div class="card">
    <input id="greet-input" v-model="name" placeholder="Enter a name..."/>
    <button type="button" @click="test()">Greet</button>
  </div>

  <p>{{ greetMsg }}</p>
</template>
