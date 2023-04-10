import {ethers} from "ethers";
import {op_abi} from "./abis.js";
import utils from "../scripts/transfer/transfer_utils.js";


const private_key = "afad87223fbf1ac1c0b6ac73d76508d80ada7dee9a16b0eb74e654720321008f"
const address = "0x92377cc62E24DAeC0246ba4b9EB9CE70822fb63D"


let provider = utils.get_provider('op');
let contract = new ethers.Contract("0x4200000000000000000000000000000000000042", op_abi);

let wallet = new ethers.Wallet(private_key, provider);

const dd = await contract.connect(wallet).estimateGas.transfer(address,await wallet.getBalance())
console.log(dd)
//
// const data = [
//     {
//         demo: '1'
//     }
// ]
//
// function test(item) {
//     item.demo = '2'
// }
//
// test(data[0])
// console.log(data)
