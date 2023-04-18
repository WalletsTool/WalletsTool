<script setup name="transfer">
import {IconDelete, IconDoubleLeft, IconPlus} from '@arco-design/web-vue/es/icon';
import {useRouter} from "vue-router";
import {nextTick, onBeforeMount, onMounted, reactive, ref, watch} from "vue";
import {invoke} from "@tauri-apps/api/tauri";
import {Notification} from "@arco-design/web-vue";
import utils from "@/scripts/transfer/transfer_utils.js";
import base_coin_transfer from "@/scripts/transfer/base_coin_transfer.js";
import token_transfer from "@/scripts/transfer/token_transfer.js";
import {ethers} from "ethers";
import {debounce} from "throttle-debounce";
import {read, utils as xlUtils} from "xlsx";
import balance_utils from "@/scripts/balance/balance_utils.js";
import token_utils from "@/scripts/token/token_utils.js";

const router = useRouter()
// table列名
const columns = [
    {
        title: '序号',
        align: 'center',
        width: '60',
        slotName: 'index'
    },
    {
        title: '发送秘钥',
        align: 'center',
        width: '400',
        dataIndex: 'private_key',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '接收地址',
        align: 'center',
        dataIndex: 'to_addr',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '平台币余额',
        align: 'center',
        dataIndex: 'plat_balance',
        width: '120',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '代币余额',
        align: 'center',
        dataIndex: 'coin_balance',
        width: '120',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '状态',
        align: 'center',
        slotName: 'exec_status',
        width: '100',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '返回信息',
        align: 'center',
        dataIndex: 'error_msg',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '操作',
        align: 'center',
        slotName: 'optional',
        width: '60',
        ellipsis: "true",
        tooltip: 'true'
    }
]
let tableLoading = ref(false)
const data = ref([])
// 选中的数据key
const selectedKeys = ref([]);
// 选择配置
const rowSelection = reactive({
    type: 'checkbox',
    showCheckedAll: true,
    onlyCurrent: false,
});

// 点击行实现选中和取消
function rowClick(record, event) {
    const index = selectedKeys.value.indexOf(record.private_key)
    index >= 0 ? selectedKeys.value.splice(index, 1) : selectedKeys.value.push(record.private_key)
}

// 分页
const pagination = ref(false);
const scrollbar = ref(true);
// 滚动条设置
let scroll = {
    y: document.documentElement.clientHeight >= 625 ? document.documentElement.clientHeight - 445 : 180
}
let tableBool = ref(true)
// rpc默认值
const rpcValue = ref('');
// 当前rpc
const currentRpc = ref({});
// rpc自定义字段名
const rpcFieldNames = {value: 'key', label: 'scan_url'}
// 主网选择器
let rpcOptions = ref([])
// coin默认值
let coinValue = ref('');
// coin自定义字段名
const coinFieldNames = {value: 'key', label: 'coin'}
// 币种选择器
const coinOptions = ref([]);
// 查询余额按钮loading
let balanceLoading = ref(false)
// 详细配置
const form = reactive({
    send_type: '3',
    send_count: '0',
    send_min_count: '1',
    send_max_count: '100',
    gas_price_type: '3',
    gas_price: '30',
    gas_price_rate: '5',
    max_gas_price: '',
    limit_type: '1',
    limit_count: '21000',
    limit_min_count: '21000',
    limit_max_count: '30000',
    min_interval: '1',
    max_interval: '3',
    amount_precision: '6'
})

// 录入 私钥 / 接收地址 弹窗
let visible = ref(false)
let importModalTitle = ref('')
let importModalType = ref('')
let importText = ref('')
// 添加代币弹窗
let addCoinVisible = ref(false)
let coinAddress = ref('')
// 删除代币弹窗
let deleteTokenVisible = ref(false)
// 删除信息弹窗
let deleteItemVisible = ref(false)
// 当前币种名称
let currentCoin = ref({})
// 当前数据的key
let currentItemKey = ref('')
// 开始执行按钮loading
let startLoading = ref(false)
// 转账中途停止
let stopFlag = ref(false)
// 转账是否已经停止
let stopStatus = ref(true)

watch(stopStatus, (newValue, oldValue) => {
    console.log(`count的值已从${oldValue}更新为${newValue}`);
})

// 初始化RPC列表
onBeforeMount(async () => {
    const rpcList = await invoke('get_chain_list')
    // 过滤掉starknet
    rpcOptions.value = rpcList.filter(item => item.key !== 'starknet')
    rpcValue.value = rpcOptions.value[0].key
    currentRpc.value = rpcOptions.value[0]
    // 获取rpc对应的代币列表
    await rpcChange()
})

onMounted(() => {
    // 监听页面高度
    window.onresize = () => {
        return (() => {
            window.screenHeight = document.documentElement.clientHeight
            setTimeout(() => {
                if (window.screenHeight >= 700) {
                    tableBool.value = false
                    scroll = {
                        y: window.screenHeight - 445
                    }
                    nextTick(() => {
                        tableBool.value = true
                    })
                } else {
                    tableBool.value = false
                    scroll = {
                        y: 180
                    }
                    nextTick(() => {
                        tableBool.value = true
                    })
                }
            }, 200)
        })()
    }
})

// 读取上传的文件
function UploadFile() {
    visible.value = false
    let file = uploadInputRef.value.files[0]
    let reader = new FileReader()
    //提取excel中文件内容
    reader.readAsArrayBuffer(file);
    let fileData = []
    data.value = []
    tableLoading.value = true
    reader.onload = function () {
        const buffer = reader.result;
        const bytes = new Uint8Array(buffer);
        const length = bytes.byteLength;
        let binary = "";
        for (let i = 0; i < length; i++) {
            binary += String.fromCharCode(bytes[i]);
        }
        //转换二进制
        const wb = read(binary, {
            type: "binary",
        });
        const outdata = xlUtils.sheet_to_json(wb.Sheets[wb.SheetNames[0]]);
        //这里for循环将excel表格数据转化成json数据
        outdata.forEach((i) => {
            if (i.私钥 && i.地址) {
                data.value.push({
                    private_key: String(i.私钥),
                    to_addr: String(i.地址),
                    plat_balance: '',
                    coin_balance: '',
                    exec_status: '0',
                    error_msg: ''
                })
            }
        })
    }
    reader.onloadend = function () {
        tableLoading.value = false
        console.log('读取完成')
    }
}

const uploadInputRef = ref(null)

// 点击上传文件
function upload() {
    uploadInputRef.value.click()
}

// 下载模板文件
const downloadFile = debounce(1000, () => {
    let a = document.createElement('a');
    a.href = `/template/import_model.xlsx`;
    a.download = '导入模板.xlsx';
    a.click();
})

// RPC变化事件
async function rpcChange() {
    coinOptions.value = await invoke("get_coin_list", {chain: rpcValue.value})
    coinValue.value = coinOptions.value[0].key
    currentCoin.value = coinOptions.value[0]
    currentRpc.value = rpcOptions.value.filter(item => item.key === rpcValue.value)[0]
}

// coin变化事件
async function coinChange(value) {
    currentCoin.value = coinOptions.value.filter(item => item.key === value)[0]
    console.log(currentCoin.value)
}

// 导入事件触发
function handleAddCoinClick() {
    addCoinVisible.value = true
}

// 添加代币弹窗取消
function handleAddCoinCancel() {
    addCoinVisible.value = false
}

// 添加代币核心方法
function addCoinFunc() {
    return new Promise((resolve, reject) => {
        const scan_api = currentRpc.value.scan_api
        const verify_api = currentRpc.value.verify_api
        const check_verify_api = currentRpc.value.check_verify_api

        console.log('校验是否存在代理合约')
        // 校验是否存在代理合约
        token_utils.getProxyAddress(coinAddress.value, verify_api, check_verify_api).then(proxy_address => {
            let address = coinAddress.value
            if (proxy_address) {
                address = proxy_address
            }
            console.log('获取合约ABI')
            // 获取合约ABI
            token_utils.getAbi(address, scan_api).then(abi => {
                console.log('获取代币名称')
                token_utils.getTokenSymbol(rpcValue.value, coinAddress.value, abi).then(symbol => {
                    let json = {
                        "key": symbol.toLowerCase(),
                        "coin": symbol,
                        "type": "token",
                        "contract_type": "",
                        "contract_address": coinAddress.value,
                        "abi": abi
                    }
                    console.log('添加代币')
                    // 添加代币
                    invoke('add_coin', {chain: rpcValue.value, objJson: JSON.stringify(json)}).then(() => {
                        addCoinVisible.value = false
                        coinAddress.value = ''
                        resolve()
                    }).catch(err => {
                        console.log(err)
                        reject('添加代币失败！')
                    })
                }).catch(err => {
                    console.log(err)
                    reject('获取代币名称异常，添加代币失败！')
                })
            }).catch(err => {
                reject(err)
            })
        }).catch(() => {
            reject('校验合约地址异常，添加代币失败！')
        })
    })
}

// 添加代币弹窗确认
const handleAddCoinBeforeOk = async () => {
    coinAddress.value = coinAddress.value.trim()
    if (!coinAddress.value) {
        Notification.warning('请输入代币地址！');
        return false
    }
    let flag = false
    await addCoinFunc().then(() => {
        Notification.success('添加代币成功！');
        flag = true
    }).catch(err => {
        Notification.error(err);
    })
    // 删除成功后重新获取代币列表
    rpcChange()
    return flag
}

// 清空列表
function clearData() {
    data.value = []
    Notification.success('清空列表成功！');
}

// 清空地址
function clearAddress() {
    data.value.forEach(
        item => {
            item.to_addr = ''
        }
    )
    Notification.success('清空地址成功！');
}

// 导入事件触发
function handleClick(type) {
    if (type === 'send') {
        importModalTitle.value = '录入私钥'
    } else if (type === 'receive') {
        importModalTitle.value = '录入接收地址'
    } else {
        Notification.warning('导入类型错误！');
        return
    }
    importModalType.value = type
    visible.value = true
}

// 导入弹窗关闭事件
function handleCancel() {
    // TODO 判断是否正在进行数据处理 如果进行数据处理则提示
    visible.value = false
    importText.value = ''
    importModalTitle.value = ''
    importModalType.value = ''
}

// 导入弹窗保存事件
const handleBeforeOk = () => {
    // 导入私钥
    if (importModalType.value === 'send') {
        let importList = importText.value.split('\n').filter(item => item !== '')
        const total_count = importList.length
        importList = importList.filter(item => data.value.length === 0 || !data.value.find(obj => obj.private_key === item))
        const success_count = importList.length
        const fail_count = total_count - success_count
        data.value.push(...importList.map(item => {
            return {
                private_key: item,
                to_addr: '',
                plat_balance: '',
                coin_balance: '',
                exec_status: '0',
                error_msg: ''
            }
        }))
        if (fail_count > 0) {
            Notification.warning({
                title: '导入完成！',
                content: `执行${total_count}条，成功${success_count}条，失败${fail_count}条！`,
            })
        } else {
            Notification.success({
                title: '导入成功！',
                content: `成功导入${total_count}条`,
            })
        }
        importText.value = ''
        return true
    } else if (importModalType.value === 'receive') {
        // 导入接收地址
        const importList = importText.value.split('\n')
        if (data.value.length === 0) {
            Notification.warning('请先导入私钥！');
            return false
        }
        // 如果私有都已经有接收地址了 则不导入
        if (!data.value.find(item => !item.to_addr)) {
            Notification.warning('所有私钥均已有接收地址，无法导入！');
            return false
        }
        let index = 0
        data.value.forEach(item => {
            if (!item.to_addr) {
                item.to_addr = importList[index]
                index++
            }
        })
        importText.value = ''
        return true
    } else {
        Notification.warning('导入类型错误！');
        return false
    }
}

// 删除数据
function deleteItem(item) {
    if (startLoading.value) {
        Notification.warning('请停止或等待执行完成后再删除数据！');
        return
    }
    // 删除确认
    deleteItemModalShow(item.private_key)
}

// 删除数据弹窗显示
function deleteItemModalShow(private_key) {
    deleteItemVisible.value = true
    currentItemKey.value = private_key
}

// 删除item取消
function deleteItemCancel() {
    deleteItemVisible.value = false
}

// 删除item确认
async function deleteItemConfirm() {
    deleteItemVisible.value = false
    console.log(data.value.length)
    data.value = data.value.filter(obj => currentItemKey.value !== obj.private_key)
    console.log(data.value.length)
    Notification.success('删除成功！');
}

// 删除代币取消
function deleteTokenCancel() {
    deleteTokenVisible.value = false
}

// 查询余额
async function queryBalance() {
    if (!stopStatus.value) {
        Notification.warning('请停止或等待执行完成后再查询余额！');
        return
    }
    if (data.value.length === 0) {
        Notification.warning('请先导入私钥！');
        return
    }
    if (currentCoin.value.type === 'base' || currentCoin.value.type === 'token') {
        balanceLoading.value = true
        data.value.forEach(item => {
            item.plat_balance = ''
            item.coin_balance = ''
            item.error_msg = ''
        })
        balance_utils.exec_group_query(rpcValue.value, currentCoin.value, data.value, () => {
            Notification.success('查询成功！');
            balanceLoading.value = false
        })
    } else {
        Notification.warning('查询 coin 类型错误！');
    }
}

// 删除代币方法
function deleteToken() {
    deleteTokenVisible.value = true
}

// 删除代币确认
async function deleteTokenConfirm() {
    console.log('确认删除代币')
    deleteTokenVisible.value = false
    await invoke("remove_coin", {chain: rpcValue.value, key: currentCoin.value.key}).then(() => {
        Notification.success('删除成功！');
        // 删除成功后重新获取代币列表
        rpcChange()
    }).catch(() => {
        Notification.error('删除失败！');
    })
}

// 执行
function startTransfer() {
    if (balanceLoading.value) {
        Notification.warning('请等待余额查询完成后再执行！');
        return
    }
    if (data.value.length === 0) {
        Notification.warning('请先导入私钥！');
        return
    }
    startLoading.value = true
    stopFlag.value = false
    stopStatus.value = false
    validateForm().then(async () => {
        console.log('验证通过')
        data.value.forEach(item => {
            item.exec_status = '0'
            item.error_msg = ''
        })
        // 执行转账
        await iter_transfer().then(() => {
            if (stopFlag.value) {
                Notification.warning('已停止执行！')
            } else {
                Notification.success('执行完成！');
                stopStatus.value = true
            }
            startLoading.value = false
            stopFlag.value = false
        }).catch(() => {
            Notification.error('执行失败！');
            startLoading.value = false
            stopStatus.value = true
        })
    }).catch(() => {
        console.log('验证失败')
        startLoading.value = false
    })
}

// 执行转账
async function iter_transfer() {
    // TODO 中途停止功能
    // 判断是主币转账还是代币转账
    let contract
    if (currentCoin.value.type === 'token') {
        contract = new ethers.Contract(currentCoin.value.contract_address, currentCoin.value.abi);
    }
    // 遍历所有账户转账
    for (let i = 0; i < data.value.length; i++) {
        try {
            const config = {
                chain: rpcValue.value,
                delay: [form.min_interval, form.max_interval],    // 延迟时间
                transfer_type: form.send_type,  // 转账类型 1：全部转账 2:转账固定数量 3：转账随机数量  4：剩余随机数量
                transfer_amount: form.send_count, // 转账固定金额
                transfer_amount_list: [form.send_min_count, form.send_max_count],  // 转账数量 (transfer_type 为 1 时生效) 转账数量在5-10之间随机，第二个数要大于第一个数！！
                left_amount_list: [form.send_min_count, form.send_max_count],  // 剩余数量 (transfer_type 为 2 时生效) 剩余数量在4-6之间随机，第二个数要大于第一个数！！
                amount_precision: Number(form.amount_precision),  // 一般无需修改，转账个数的精确度 6 代表个数有6位小数
                limit_type: form.limit_type, // limit_type 限制类型 1：自动 2：指定数量 3：范围随机
                limit_count_list: [form.limit_min_count, form.limit_max_count],
                gas_price_type: form.gas_price_type, // gas price类型 1: 自动 2：固定gas price 3：gas price溢价率
                gas_price_rate: Number(form.gas_price_rate) / 100,  // gas price溢价率，0.05代表gas price是当前gas price的105%
                max_gas_price: form.max_gas_price  // 设置最大的gas price，单位gwei
            }
            if (currentCoin.value.type === 'base') {
                if (stopFlag.value) {
                    stopStatus.value = true
                    return
                }
                // 设置状态 为执行中
                data.value[i].exec_status = '1'
                await base_coin_transfer.single_transfer(i + 1, data.value[i], config)
                    .then(res => {
                        data.value[i].exec_status = '2'
                        data.value[i].error_msg = res
                    }).catch(err => {
                        data.value[i].exec_status = '3'
                        data.value[i].error_msg = err
                    })
            } else if (currentCoin.value.type === 'token') {
                if (stopFlag.value) {
                    stopStatus.value = true
                    return
                }
                // 设置状态 为执行中
                data.value[i].exec_status = '1'
                await token_transfer.single_transfer(i + 1, data.value[i], config, contract)
                    .then(res => {
                        data.value[i].exec_status = '2'
                        data.value[i].error_msg = res
                    }).catch(err => {
                        data.value[i].exec_status = '3'
                        data.value[i].error_msg = err
                    })
            } else {
                Notification.error('未知币种类型')
                console.log('未知币种类型：', currentCoin.value.type)
                return
            }
        } catch (e) {
            console.log('序号：', i + 1, '交易失败！')
            console.log(e)
        }
    }
}

// 停止执行
function stopTransfer() {
    startLoading.value = false
    stopFlag.value = true
}

// 校验数据是否合规
function validateForm() {
    return new Promise((resolve, reject) => {
        if (checkSendType() && checkPrecision() && checkDelay() && checkGasLimit() && checkGasPrice()) {
            resolve()
        } else {
            reject()
        }
    })
}

const formRef = ref(null)

// 检验发送类型
function checkSendType() {
    if (form.send_type === '1') {
        return true
    } else if (form.send_type === '2') {
        const bool = utils.checkNum(form.send_count) && Number(form.send_count) > 0
        if (!bool) {
            Notification.error('发送数量必须为数字且大于0')
            formRef.value.setFields({
                send_count: {
                    status: 'error',
                    message: '发送数量必须为数字且大于0'
                }
            })
            return false
        } else {
            return true
        }
    } else if (form.send_type === '3' || form.send_type === '4') {
        const bool = utils.checkNum(form.send_min_count) && utils.checkNum(form.send_max_count) && Number(form.send_min_count) > 0 && Number(form.send_max_count) > 0
        if (!bool) {
            const msg = form.send_type === '4' ? '剩余数量必须为数字且大于0' : '发送数量必须为数字且大于0'
            Notification.error(msg)
            formRef.value.setFields({
                send_count_scope: {
                    status: 'error',
                    message: '数量范围错误'
                }
            })
            return false
        }
        if (Number(form.send_min_count) > Number(form.send_max_count)) {
            const msg = form.send_type === '4' ? '最大剩余数量应该大于等于最小剩余数量' : '最大发送数量应该大于等于最小发送数量'
            Notification.error(msg)
            formRef.value.setFields({
                send_count_scope: {
                    status: 'error',
                    message: '数量范围错误'
                }
            })
            return false
        }
        return true
    } else {
        Notification.error('发送类型错误')
        return false
    }
}

// 检验精度
function checkPrecision() {
    const bool = utils.checkNum(form.amount_precision) && Number(form.amount_precision) > 0 && Number(form.amount_precision) < 18
    if (!bool) {
        Notification.error('金额精度必须为数字且大于0小于18')
        formRef.value.setFields({
            amount_precision: {
                status: 'error',
                message: '应大于0小于18'
            }
        })
        return false
    } else {
        return true
    }
}

// 检验 Gas Price
function checkGasPrice() {
    if (form.gas_price_type === '1') {
        return true
    } else if (form.gas_price_type === '2') {
        const bool = utils.checkNum(form.gas_price) && Number(form.gas_price) > 0
        if (!bool) {
            Notification.error('Gas Price必须为数字且大于0')
            formRef.value.setFields({
                gas_price: {
                    status: 'error',
                    message: '必须为数字且大于0'
                }
            })
            return false
        } else {
            return true
        }
    } else if (form.gas_price_type === '3') {
        const bool = utils.checkPositiveInteger(form.gas_price_rate)
        if (!bool) {
            Notification.error('Gas Price 提高比例应为正整数')
            formRef.value.setFields({
                gas_price_rate: {
                    status: 'error',
                    message: '比例应为正整数'
                }
            })
            return false
        }
        // 如果有最大Gas Price
        if (form.max_gas_price) {
            const bool1 = utils.checkNum(form.max_gas_price) && Number(form.max_gas_price) > 0
            if (!bool1) {
                Notification.error('最大 Gas Price 设置必须为数字且大于0')
                formRef.value.setFields({
                    max_gas_price: {
                        status: 'error',
                        message: '必须为数字且大于0'
                    }
                })
                return false
            } else {
                return true
            }
        } else {
            return true
        }
    } else {
        Notification.error('Gas Price 方式错误')
        return false
    }
}

// 检验 Gas Limit
function checkGasLimit() {
    if (form.limit_type === '1') {
        return true
    } else if (form.limit_type === '2') {
        const bool = utils.checkPositiveInteger(form.limit_count)
        if (!bool) {
            Notification.error('Gas Limit 数量必须为正整数')
            formRef.value.setFields({
                limit_count: {
                    status: 'error',
                    message: '数量必须为正整数'
                }
            })
            return false
        } else {
            return true
        }
    } else if (form.limit_type === '3') {
        const bool = utils.checkPositiveInteger(form.limit_min_count) && utils.checkPositiveInteger(form.limit_max_count)
        if (!bool) {
            Notification.error('Gas Limit 数量范围必须为正整数')
            formRef.value.setFields({
                limit_count_scope: {
                    status: 'error',
                    message: '数量范围必须为正整数'
                }
            })
            return false
        }
        if (Number(form.limit_min_count) > Number(form.limit_max_count)) {
            Notification.error('最大 Gas Limit 数量应该大于等于最小 Gas Limit 数量')
            formRef.value.setFields({
                limit_count_scope: {
                    status: 'error',
                    message: '范围错误'
                }
            })
            return false
        }
        return true
    } else {
        Notification.error('Gas Limit 类型错误')
        return false
    }
}

// 检验 间隔时间
function checkDelay() {
    const bool = (form.min_interval === '0' || utils.checkPositiveInteger(form.min_interval)) && (form.max_interval === '0' || utils.checkPositiveInteger(form.max_interval))
    if (!bool) {
        Notification.error('发送间隔必须为正整数或者0')
        formRef.value.setFields({
            interval_scope: {
                status: 'error',
                message: '发送间隔必须为正整数或者0'
            }
        })
        return false
    }
    if (Number(form.min_interval) > Number(form.max_interval)) {
        Notification.error('最大间隔应该大于等于最小间隔')
        formRef.value.setFields({
            interval_scope: {
                status: 'error',
                message: '最大间隔应该大于等于最小间隔'
            }
        })
        return false
    }
    return true
}

function selectSucceeded() {
    selectedKeys.value = data.value.filter(item => item.exec_status === '2').map(item => item.private_key)
}

function selectFailed() {
    selectedKeys.value = data.value.filter(item => item.exec_status === '3').map(item => item.private_key)
}

function deleteSelected() {
    if (startLoading.value) {
        Notification.warning('请停止或等待执行完成后再删除数据！');
        return
    }
    data.value = data.value.filter(item => !selectedKeys.value.includes(item.private_key))
    Notification.success('删除成功')
}

// 返回首页
function goHome() {
    router.push({
        name: 'home'
    })
}
</script>

<template>
    <div class="container transfer">
        <span class="pageTitle">钱包多对多转账</span>
        <!-- 工具栏 -->
        <div class="toolBar">
            <a-button type="primary" @click="handleClick('send')">录入发送方</a-button>
            <a-button type="primary" style="margin-left: 10px" @click="handleClick('receive')">录入接收地址</a-button>
            <a-divider direction="vertical"/>
            <a-button type="outline" status="normal" @click="downloadFile">下载模板</a-button>
            <a-button type="primary" status="success" style="margin-left: 10px" @click="upload">导入文件</a-button>
            <input type="file" ref="uploadInputRef" @change="UploadFile" id="btn_file" style="display:none">
            <a-divider direction="vertical"/>
            <!-- 选择操作区按钮 -->
            <a-button type="outline" status="normal" @click="selectSucceeded">选中成功</a-button>
            <a-button type="outline" status="danger" style="margin-left: 10px" @click="selectFailed">选中失败</a-button>
            <a-button type="primary" status="danger" style="margin-left: 10px" @click="deleteSelected">删除选中
            </a-button>
            <a-button class="goHome" type="outline" status="success" @click="goHome">
                <template #icon>
                    <icon-double-left/>
                </template>
                返回首页
            </a-button>
            <a-button type="outline" status="normal" style="float: right;margin-right: 10px"
                      @click="clearData">清空列表
            </a-button>
            <a-button type="outline" status="normal" style="float: right;margin-right: 10px"
                      @click="clearAddress">清空地址
            </a-button>
        </div>
        <!-- 操作账号表格 -->
        <div class="mainTable">
            <a-table v-if="tableBool" row-key="private_key" :columns="columns" :column-resizable="true" :data="data"
                     :row-selection="rowSelection" :loading="tableLoading"
                     :scroll="scroll"
                     :scrollbar="scrollbar" @row-click="rowClick"
                     v-model:selectedKeys="selectedKeys" :pagination="pagination">
                <template #index="{ rowIndex }">
                    {{ rowIndex + 1 }}
                </template>
                <template #exec_status="{ rowIndex }">
                    <a-tag v-if="data[rowIndex].exec_status === '0'" color="#86909c">等待执行</a-tag>
                    <a-tag v-if="data[rowIndex].exec_status === '1'" color="#ff7d00">执行中</a-tag>
                    <a-tag v-if="data[rowIndex].exec_status === '2'" color="#00b42a">执行成功</a-tag>
                    <a-tag v-if="data[rowIndex].exec_status === '3'" color="#f53f3f">执行失败</a-tag>
                </template>
                <template #optional="{ record }">
                    <icon-delete style="font-size: 16px" @click.prevent="deleteItem(record)"/>
                </template>
            </a-table>
        </div>
        <!-- RPC 选择器 -->
        <a-select v-model="rpcValue" :options="rpcOptions" @change="rpcChange" :field-names="rpcFieldNames" size="large"
                  :style="{'margin-top':'10px'}">
            <template #label="{ data }">
                <img alt="" :src="data?.pic_url" style="width: 18px;height: 18px">
                <span style="margin-left: 10px">{{ data?.chain }}</span>
                <span style="margin-left: 50px">{{ data?.scan_url }}</span>
            </template>
            <template #option="{ data }">
                <img alt="" :src="data?.pic_url" style="width: 16px;height: 16px">
                <span style="margin-left: 10px">{{ data?.chain }}</span>
                <span style="position: absolute;right: 10px;">{{ data?.scan_url }}</span>
            </template>
        </a-select>
        <!-- 管理代币 -->
        <div>
            <div class="subTitle">
                选择代币：
            </div>
            <div style="display: flex;flex-direction: row;align-items: center;margin-top: 10px">
                <a-button type="outline" status="normal" @click="handleAddCoinClick"
                          style="margin-left: 10px">
                    <icon-plus/>
                    <span style="margin-left: 5px">添加代币</span>
                </a-button>
                <!-- 代币 选择器 -->
                <a-select v-model="coinValue" :options="coinOptions" :field-names="coinFieldNames"
                          :style="{'margin-left':'10px',flex:'1'}" @change="coinChange">
                    <template #label="{ data }">
                        <span style="margin-left: 10px">{{ data?.coin }}</span>
                    </template>
                </a-select>
                <a-button type="outline" status="normal" style="margin-left: 10px" @click="queryBalance"
                          :loading="balanceLoading">查询余额
                </a-button>
                <a-button type="primary" status="danger" @click="deleteToken" style="margin-left: 10px">删除代币
                </a-button>
            </div>
        </div>
        <div style="display: flex;padding-top: 10px">
            <!-- 细节配置 -->
            <a-form ref="formRef" :model="form" :style="{width: '85%' }" layout="vertical">
                <a-row style="height: 80px">
                    <a-form-item field="send_type" label="发送方式" style="width: 315px;padding: 10px">
                        <a-radio-group v-model="form.send_type" type="button">
                            <a-radio value="1">全部</a-radio>
                            <a-radio value="2">指定数量</a-radio>
                            <a-radio value="3">范围随机</a-radio>
                            <a-radio value="4">剩余数量</a-radio>
                        </a-radio-group>
                    </a-form-item>
                    <a-form-item v-if="form.send_type === '2'" field="send_count" label="发送数量"
                                 style="width: 150px;padding: 10px">
                        <a-input v-model="form.send_count"/>
                    </a-form-item>
                    <a-form-item v-if="form.send_type === '3' || form.send_type === '4'" field="send_count_scope"
                                 :label="form.send_type === '3'?'发送数量范围':'剩余数量范围'"
                                 style="width: 180px;padding: 10px">
                        <a-input v-model="form.send_min_count"/>
                        <span style="padding: 0 5px">至</span>
                        <a-input v-model="form.send_max_count"/>
                    </a-form-item>
                    <a-form-item v-if="form.send_type === '3' || form.send_type === '4'" field="amount_precision"
                                 label="金额精度" style="width: 110px;padding: 10px"
                                 tooltip="金额小数点位数">
                        <a-input v-model="form.amount_precision"/>
                    </a-form-item>
                    <a-form-item field="interval_scope" label="发送间隔（秒）" style="width: 210px;padding: 10px">
                        <a-input v-model="form.min_interval"/>
                        <span style="padding: 0 5px">至</span>
                        <a-input v-model="form.max_interval"/>
                    </a-form-item>
                </a-row>
                <a-row style="height: 80px">
                    <a-form-item field="limit_type" label="Gas Limit" style="width: 230px;padding: 10px">
                        <a-radio-group v-model="form.limit_type" type="button">
                            <a-radio value="1">自动</a-radio>
                            <a-radio value="2">指定数量</a-radio>
                            <a-radio value="3">范围随机</a-radio>
                        </a-radio-group>
                    </a-form-item>
                    <a-form-item v-if="form.limit_type === '2'" style="width: 150px;padding: 10px" field="limit_count"
                                 label="Gas Limit">
                        <a-input v-model="form.limit_count"/>
                    </a-form-item>
                    <a-form-item v-if="form.limit_type === '3'" style="width: 265px;padding: 10px"
                                 field="limit_count_scope"
                                 label="Gas Limit 范围">
                        <a-input v-model="form.limit_min_count"/>
                        <span style="padding: 0 5px">至</span>
                        <a-input v-model="form.limit_max_count"/>
                    </a-form-item>
                    <a-form-item field="gas_price_type" label="Gas Price 方式" style="width: 225px;padding: 10px">
                        <a-radio-group v-model="form.gas_price_type" type="button">
                            <a-radio value="1">自动</a-radio>
                            <a-radio value="2">固定值</a-radio>
                            <a-radio value="3">指定比例</a-radio>
                        </a-radio-group>
                    </a-form-item>
                    <a-form-item v-if="form.gas_price_type === '2'" field="gas_price"
                                 style="width: 120px;padding: 10px" label="Gas Price">
                        <a-input v-model="form.gas_price"/>
                    </a-form-item>
                    <a-form-item v-if="form.gas_price_type === '3'" field="gas_price_rate"
                                 style="width: 100px;padding: 10px" label="提高比例">
                        <a-input v-model="form.gas_price_rate">
                            <template #append>
                                %
                            </template>
                        </a-input>
                    </a-form-item>
                    <a-form-item v-if="form.gas_price_type === '3'" field="max_gas_price"
                                 style="width: 130px;padding: 10px" label="最大 Gas Price"
                                 tooltip="为空时则不设置上限（单位：Gwei）">
                        <a-input v-model="form.max_gas_price"/>
                    </a-form-item>
                </a-row>
            </a-form>
            <!-- 提交按钮 -->
            <div style="display: flex;flex: 1;align-items: center;justify-content: center;">
                <a-button v-if="!startLoading &&  stopStatus" :class="['submitBtn']" @click="startTransfer">执行转账
                </a-button>
                <a-tooltip v-else content="点击可以提前停止执行">
                    <div @click="stopTransfer">
                        <a-button v-if="!stopFlag" class="submitBtn" loading>执行中...</a-button>
                        <a-button v-if="stopFlag && !stopStatus" class="submitBtn" loading>正在停止...</a-button>
                    </div>
                </a-tooltip>
            </div>
        </div>
    </div>
  <!-- 录入弹窗 -->
    <a-modal v-model:visible="visible" :width="700" :title="importModalTitle" @cancel="handleCancel"
             :on-before-ok="handleBeforeOk">
        <a-textarea v-model="importText" style="margin-top: 10px" placeholder="格式：一行一个" allow-clear :auto-size="{
            minRows:15,
            maxRows:20
          }"/>
    </a-modal>
  <!-- 添加代币弹窗 -->
    <a-modal v-model:visible="addCoinVisible" :width="700" title="添加代币" @cancel="handleAddCoinCancel"
             :on-before-ok="handleAddCoinBeforeOk" unmountOnClose>
        <a-input v-model="coinAddress" placeholder="请输入代币合约地址" allow-clear/>
    </a-modal>
  <!-- 删除代币确认框 -->
    <a-modal v-model:visible="deleteTokenVisible" title="删除确认">
        <div>确认删除【 {{ currentCoin.coin }} 】代币？</div>
        <template #footer>
            <a-button @click="deleteTokenCancel">取消</a-button>
            <a-button type="primary" status="danger" @click="deleteTokenConfirm" style="margin-left: 10px">确定
            </a-button>
        </template>
    </a-modal>
  <!-- 删除数据确认框 -->
    <a-modal v-model:visible="deleteItemVisible" title="删除确认">
        <div>确认删除私钥为【
            {{ currentItemKey.substring(0, 15) + '......' + currentItemKey.substring(currentItemKey.length - 15) }}
            】的数据？
        </div>
        <template #footer>
            <a-button @click="deleteItemCancel">取消</a-button>
            <a-button type="primary" status="danger" @click="deleteItemConfirm" style="margin-left: 10px">确定
            </a-button>
        </template>
    </a-modal>
</template>

<style scoped>
.container {
    background-color: white;
    padding: 10px;
    min-width: 1240px;
}

.pageTitle {
    position: fixed;
    padding: 0 30px;
    user-select: none;
    text-align: start;
    line-height: 100px;
    font-size: 100px;
    background-image: linear-gradient(to bottom, #f2f3f5, #ffffff);
    -webkit-background-clip: text;
    font-weight: 600;
    height: 120px;
    right: -10px;
    border-radius: 30px;
    color: transparent;
    top: 15px;
    z-index: 0;
}

.toolBar {
    margin-top: 10px;
}

.goHome {
    float: right;
    background-color: white;
}

.goHome:hover {
    background-color: #ffffffa6;
}

.mainTable {
    margin-top: 20px;
}

.subTitle {
    font-weight: 600;
    font-size: 16px;
    margin: 10px 0 0 10px;
}

.arco-form-item {
    padding: 20px 10px 0 10px;
    margin-bottom: 10px;
}

.submitBtn {
    width: 150px;
    height: 80px;
    font-size: 22px;
    color: #ffffff;
    background-color: #0fa962;
}

.arco-btn-secondary.arco-btn-loading {
    color: #ffffff;
    background-color: #11c06f;
}

.submitBtn:hover {
    color: #ffffff;
    background-color: #11c06f;
}

.arco-btn-secondary.arco-btn-loading:hover {
    color: #ffffff;
    background-color: #fc0934;
}

.arco-btn-outline.arco-btn-loading:hover {
    color: #ffffff;
    background-color: #fc0934;
    border: none;
}

.arco-radio-button.arco-radio-checked {
    color: #ffffff;
    background-color: #165dff;
}

.importBar {
    display: flex;
}
</style>
<style lang="less">
.transfer {
  .arco-table-body {
    min-height: 355px;

    .arco-table-element .arco-empty {
      min-height: 330px;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
    }
  }
}
</style>
