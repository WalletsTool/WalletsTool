<script setup name="transfer">
import { IconDelete, IconDoubleLeft, IconPlus, IconUpload } from "@arco-design/web-vue/es/icon";
import { useRouter } from "vue-router";
import { nextTick, onBeforeMount, onBeforeUnmount, onMounted, reactive, ref, watch, } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Notification } from "@arco-design/web-vue";
import utils from "@/scripts/transfer/transfer_utils.js";
import { utils as rpcUtils } from "@/scripts/common/provider.js";
import base_coin_transfer from "@/scripts/transfer/base_coin_transfer.js";
import token_transfer from "@/scripts/transfer/token_transfer.js";
import { ethers } from "ethers";
import { debounce } from "throttle-debounce";
import { read, utils as xlUtils } from "xlsx";
import balance_utils from "@/scripts/balance/balance_utils.js";
import token_utils from "@/scripts/token/token_utils.js";
import { getCurrentWindow } from '@tauri-apps/api/window'

const router = useRouter();
// 窗口标题
const windowTitle = ref('Web3 Tools - 批量转账');
// table列名
const columns = [
  {
    title: "序号",
    align: "center",
    width: "60",
    slotName: "index",
  },
  {
    title: "发送秘钥",
    align: "center",
    width: "400",
    dataIndex: "private_key",
    ellipsis: "true",
    tooltip: "true",
  },
  {
    title: "接收地址",
    align: "center",
    dataIndex: "to_addr",
    ellipsis: "true",
    tooltip: "true",
  },
  {
    title: "转账数量",
    align: "center",
    dataIndex: "amount",
    width: "120",
    ellipsis: "true",
    tooltip: "true",
  },
  {
    title: "平台币余额",
    align: "center",
    dataIndex: "plat_balance",
    width: "120",
    ellipsis: "true",
    tooltip: "true",
  },
  {
    title: "代币余额",
    align: "center",
    dataIndex: "coin_balance",
    width: "120",
    ellipsis: "true",
    tooltip: "true",
  },
  {
    title: "状态",
    align: "center",
    slotName: "exec_status",
    width: "100",
    ellipsis: "true",
    tooltip: "true",
  },
  {
    title: "返回信息",
    align: "center",
    dataIndex: "error_msg",
    ellipsis: "true",
    tooltip: "true",
  },
  {
    title: "操作",
    align: "center",
    slotName: "optional",
    width: "60",
    ellipsis: "true",
    tooltip: "true",
  },
];
let tableLoading = ref(false);
const data = ref([]);
// 选中的数据key
const selectedKeys = ref([]);
// 选择配置
const rowSelection = reactive({
  type: "checkbox",
  showCheckedAll: true,
  onlyCurrent: false,
});

// 点击行实现选中和取消
function rowClick(record, event) {
  const index = selectedKeys.value.indexOf(record.key);
  index >= 0
    ? selectedKeys.value.splice(index, 1)
    : selectedKeys.value.push(record.key);
}

// 分页
const pagination = ref(false);
const scrollbar = ref(true);
// 滚动条设置
// 滚动配置现在通过 CSS calc() 动态计算
let tableBool = ref(true);
// rpc默认值
const chainValue = ref("");
// 当前rpc
const currentChain = ref({});
// rpc自定义字段名
const chainFieldNames = { value: "key", label: "scan_url" };
// 主网选择器
let chainOptions = ref([]);
// coin默认值
let coinValue = ref("");
// coin自定义字段名
const coinFieldNames = { value: "key", label: "label" };
// 币种选择器
const coinOptions = ref([]);
// 查询余额按钮loading
let balanceLoading = ref(false);
// 详细配置
const form = reactive({
  send_type: "3",
  amount_from: "1",
  send_count: "0",
  send_min_count: "1",
  send_max_count: "100",
  gas_price_type: "3",
  gas_price: "30",
  gas_price_rate: "5",
  max_gas_price: "",
  limit_type: "1",
  limit_count: "21000",
  limit_min_count: "21000",
  limit_max_count: "30000",
  min_interval: "1",
  max_interval: "3",
  amount_precision: "6",
  error_retry: "0",
});

// 录入 私钥 / 接收地址 弹窗
let visible = ref(false);
let importModalTitle = ref("");
let importModalType = ref("");
let importText = ref("");
// 添加代币弹窗
let addCoinVisible = ref(false);
let coinAddress = ref("");
// 删除代币弹窗
let deleteTokenVisible = ref(false);
// 链管理弹窗显示状态
let chainManageVisible = ref(false);
// 链编辑弹窗显示状态（新增和编辑共用）
let chainFormVisible = ref(false);
// 是否为编辑模式（false为新增，true为编辑）
let isEditMode = ref(false);
// 链管理表格数据
let chainManageData = ref([]);
// 当前编辑的链
let currentEditChain = ref(null);
// 链管理表格加载状态
let chainTableLoading = ref(false);
// 链信息表单（添加和编辑共用）
const chainForm = reactive({
  chain_key: '',
  chain_name: '',
  chain_id: '',
  native_currency_symbol: '',
  native_currency_name: '',
  native_currency_decimals: 18,
  pic_url: '',
  scan_url: '',
  scan_api: '',
  verify_api: '',
  check_verify_api: '',
  rpc_urls: ['']
});
// 删除信息弹窗
let deleteItemVisible = ref(false);
// 当前币种名称
let currentCoin = ref({});
// 当前数据的key
let currentItemKey = ref("");
// 开始执行按钮loading
let startLoading = ref(false);
// 转账中途停止
let stopFlag = ref(false);
// 转账是否已经停止
let stopStatus = ref(true);
// 线程数设置，默认为1
let threadCount = ref(1);
// 获取gas
const timer = setInterval(fetchGas, 5000);

watch(stopStatus, (newValue, oldValue) => {
  console.log(`count的值已从${oldValue}更新为${newValue}`);
});

// 初始化RPC列表
onBeforeMount(async () => {
  const chainList = await invoke("get_chain_list");
  // 过滤掉starknet
  chainOptions.value = chainList.filter((item) => item.key !== "starknet");
  chainValue.value = chainOptions.value[0].key;
  currentChain.value = chainOptions.value[0];
  // 获取rpc对应的代币列表
  await chainChange();
});

onMounted(async () => {
  // 获取窗口标题
  try {
    const currentWindow = getCurrentWindow();
    const title = await currentWindow.title();
    if (title) {
      windowTitle.value = title;
    }
  } catch (error) {
    console.error('Error getting window title:', error);
  }

  // 页面高度现在通过 CSS 自动调整，无需监听器

  // 监听余额查询更新事件
  await listen('balance_item_update', (event) => {
    console.log('Transfer页面收到余额更新:', event.payload);
    const { index, item } = event.payload;
    if (data.value[index]) {
      // 更新对应索引的数据
      Object.assign(data.value[index], item);
    }
  });
});

onBeforeUnmount(() => {
  if (timer) {
    clearInterval(timer);
  }
});

// 读取上传的文件
function UploadFile() {
  visible.value = false;
  let file = uploadInputRef.value.files[0];
  let reader = new FileReader();
  //提取excel中文件内容
  reader.readAsArrayBuffer(file);
  let fileData = [];
  data.value = [];
  tableLoading.value = true;
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
        // 从私钥生成地址
        let address = "";
        try {
          const wallet = new ethers.Wallet(String(i.私钥));
          address = wallet.address;
        } catch (error) {
          console.error('生成地址失败:', error);
          address = "无效私钥";
        }
        data.value.push({
          key: String(i.私钥) + String(i.地址),
          private_key: String(i.私钥),
          address: address,
          to_addr: String(i.地址),
          amount: String(i.转账数量),
          plat_balance: "",
          coin_balance: "",
          exec_status: "0",
          error_msg: "",
        });
      }
    });
  };
  reader.onloadend = function () {
    tableLoading.value = false;
    console.log("读取完成");
  };
}

const uploadInputRef = ref(null);

// 点击上传文件
function upload() {
  uploadInputRef.value.click();
}

// 下载模板文件
const downloadFile = debounce(1000, () => {
  let a = document.createElement("a");
  a.href = `/template/import_model.xlsx`;
  a.download = "导入模板.xlsx";
  a.click();
});

// RPC变化事件
async function chainChange() {
  coinOptions.value = await invoke("get_coin_list", {
    chain: chainValue.value
  });
  coinValue.value = coinOptions.value[0].key;
  currentCoin.value = coinOptions.value[0];
  currentChain.value = chainOptions.value.filter(
    (item) => item.key === chainValue.value
  )[0];
  currentChain.value.gas_price = "查询中...";
  // 查询gas
  fetchGas();
}

function fetchGas() {
  const temp = chainValue.value;
  if (temp === "sol") {
    currentChain.value.gas_price = "";
    return
  }
  // 获取gas价格
  rpcUtils
    .get_base_gas_price(chainValue.value)
    .then((res) => {
      if (temp === chainValue.value) {
        if (chainValue.value === "eth") {
          currentChain.value.gas_price = res.toFixed(3);
        } else {
          currentChain.value.gas_price = res.toFixed(7);
        }
      } else {
        console.log("gas price 已失效");
      }
    })
    .catch((err) => {
      console.log(err);
      currentChain.value.gas_price = "查询错误";
    });
}

// coin变化事件
async function coinChange(value) {
  currentCoin.value = coinOptions.value.filter((item) => item.key === value)[0];
  console.log(currentCoin.value);
}

// 导入事件触发
function handleAddCoinClick() {
  if (chainValue.value === "okt") {
    Notification.warning(" OKT Chain 暂不支持添加代币！");
    return;
  }
  if (chainValue.value === "evmos") {
    Notification.warning(" EVMOS Chain 暂不支持添加代币！");
    return;
  }
  if (chainValue.value === "geth") {
    Notification.warning(" Goerli Ethereum 暂不支持添加代币！");
    return;
  }
  if (chainValue.value === "sepolia") {
    Notification.warning(" Sepolia Ethereum 暂不支持添加代币！");
    return;
  }
  if (chainValue.value === "scroll") {
    Notification.warning(" Scroll Alpha TestNet 暂不支持添加代币！");
    return;
  }
  if (chainValue.value === "linea") {
    Notification.warning(" Linea MainNet 暂不支持添加代币！");
    return;
  }
  if (chainValue.value === "base") {
    Notification.warning(" Base MainNet 暂不支持添加代币！");
    return;
  }
  addCoinVisible.value = true;
}

// 添加代币弹窗取消
function handleAddCoinCancel() {
  addCoinVisible.value = false;
}

// 添加代币核心方法
function addCoinFunc() {
  return new Promise((resolve, reject) => {
    const scan_api = currentChain.value.scan_api;
    const verify_api = currentChain.value.verify_api;
    const check_verify_api = currentChain.value.check_verify_api;

    console.log("校验是否存在代理合约");
    // 校验是否存在代理合约
    token_utils
      .getProxyAddress(coinAddress.value, verify_api, check_verify_api)
      .then((proxy_address) => {
        let address = coinAddress.value;
        if (proxy_address) {
          address = proxy_address;
        }
        console.log("获取合约ABI");
        // 获取合约ABI
        token_utils
          .getAbi(address, scan_api)
          .then((abi) => {
            console.log("获取代币名称");
            token_utils
              .getTokenSymbol(chainValue.value, coinAddress.value, abi)
              .then((symbol) => {
                let json = {
                  key: symbol.toLowerCase(),
                  coin: symbol,
                  type: "token",
                  contract_type: "",
                  contract_address: coinAddress.value,
                  abi: abi,
                };
                console.log("添加代币");
                // 添加代币
                invoke("add_coin", {
                  chain: chainValue.value,
                  objJson: JSON.stringify(json),
                })
                  .then(() => {
                    addCoinVisible.value = false;
                    coinAddress.value = "";
                    resolve();
                  })
                  .catch((err) => {
                    console.log(err);
                    reject("添加代币失败！");
                  });
              })
              .catch((err) => {
                console.log(err);
                reject("获取代币名称异常，添加代币失败！");
              });
          })
          .catch((err) => {
            reject(err);
          });
      })
      .catch(() => {
        reject("校验合约地址异常，添加代币失败！");
      });
  });
}

// 添加代币弹窗确认
const handleAddCoinBeforeOk = async () => {
  coinAddress.value = coinAddress.value.trim();
  if (!coinAddress.value) {
    Notification.warning("请输入代币地址！");
    return false;
  }
  let flag = false;
  await addCoinFunc()
    .then(() => {
      Notification.success("添加代币成功！");
      flag = true;
    })
    .catch((err) => {
      Notification.error(err);
    });
  // 删除成功后重新获取代币列表
  chainChange();
  return flag;
};

// 清空列表
function clearData() {
  data.value = [];
  Notification.success("清空列表成功！");
}

// 清空地址
function clearAddress() {
  data.value.forEach((item) => {
    item.to_addr = "";
  });
  Notification.success("清空地址成功！");
}

// 导入事件触发
function handleClick(type) {
  if (type === "send") {
    importModalTitle.value = "录入私钥";
  } else if (type === "receive") {
    importModalTitle.value = "录入接收地址";
  } else {
    Notification.warning("导入类型错误！");
    return;
  }
  importModalType.value = type;
  visible.value = true;
}

// 导入弹窗关闭事件
function handleCancel() {
  // TODO 判断是否正在进行数据处理 如果进行数据处理则提示
  visible.value = false;
  importText.value = "";
  importModalTitle.value = "";
  importModalType.value = "";
}

// 导入弹窗保存事件
const handleBeforeOk = () => {
  // 导入私钥
  if (importModalType.value === "send") {
    let importList = importText.value.split("\n").filter((item) => item !== "");
    const total_count = importList.length;
    // importList = importList.filter(item => data.value.length === 0 || !data.value.find(obj => obj.key === item))
    const success_count = importList.length;
    const fail_count = total_count - success_count;
    data.value.push(
      ...importList.map((item) => {
        // 从私钥生成地址
        let address = "";
        try {
          const wallet = new ethers.Wallet(item);
          address = wallet.address;
        } catch (error) {
          console.error('生成地址失败:', error);
          address = "无效私钥";
        }
        return {
          key: item,
          private_key: item,
          address: address,
          to_addr: "",
          plat_balance: "",
          coin_balance: "",
          exec_status: "0",
          error_msg: "",
        };
      })
    );
    if (fail_count > 0) {
      Notification.warning({
        title: "导入完成！",
        content: `执行${total_count}条，成功${success_count}条，失败${fail_count}条！`,
      });
    } else {
      Notification.success({
        title: "导入成功！",
        content: `成功导入${total_count}条`,
      });
    }
    importText.value = "";
    return true;
  } else if (importModalType.value === "receive") {
    // 导入接收地址
    const importList = importText.value
      .split("\n")
      .filter((item) => item !== "");
    if (data.value.length === 0) {
      Notification.warning("请先导入私钥！");
      return false;
    }
    // 如果私有都已经有接收地址了 则不导入
    if (!data.value.find((item) => !item.to_addr)) {
      Notification.warning("所有私钥均已有接收地址，无法导入！");
      return false;
    }
    let index = 0;
    data.value.forEach((item) => {
      if (!item.to_addr) {
        item.to_addr = importList[index];
        item.key = item.key + item.to_addr;
        index++;
      }
    });
    importText.value = "";
    return true;
  } else {
    Notification.warning("导入类型错误！");
    return false;
  }
};

// 删除数据
function deleteItem(item) {
  if (startLoading.value) {
    Notification.warning("请停止或等待执行完成后再删除数据！");
    return;
  }
  // 删除确认
  deleteItemModalShow(item.key);
}

// 删除数据弹窗显示
function deleteItemModalShow(key) {
  deleteItemVisible.value = true;
  currentItemKey.value = key;
}

// 删除item取消
function deleteItemCancel() {
  deleteItemVisible.value = false;
}

// 删除item确认
async function deleteItemConfirm() {
  deleteItemVisible.value = false;
  console.log(data.value.length);
  data.value = data.value.filter((obj) => currentItemKey.value !== obj.key);
  console.log(data.value.length);
  Notification.success("删除成功！");
}

// 删除代币取消
function deleteTokenCancel() {
  deleteTokenVisible.value = false;
}

// 查询余额
async function queryBalance() {
  if (!stopStatus.value) {
    Notification.warning("请停止或等待执行完成后再查询余额！");
    return;
  }
  if (data.value.length === 0) {
    Notification.warning("请先导入私钥！");
    return;
  }
  if (currentCoin.value.coin_type === "base" || currentCoin.value.coin_type === "token") {
    balanceLoading.value = true;

    // 重置所有项目状态
    data.value.forEach((item) => {
      item.plat_balance = "";
      item.coin_balance = "";
      item.error_msg = "";
      item.exec_status = "0";
    });

    console.log(`开始查询 ${data.value.length} 个地址的余额`);

    try {
      // 使用Rust后端进行查询 - 支持实时更新
      const params = {
        chain: chainValue.value,
        coin_config: {
          coin_type: currentCoin.value.coin_type,
          contract_address: currentCoin.value.contract_address || null,
          abi: currentCoin.value.abi || null
        },
        items: data.value.map(item => ({
          address: item.address || null,
          private_key: item.private_key || null,
          plat_balance: null,
          coin_balance: null,
          nonce: null,
          exec_status: '0',
          error_msg: null
        })),
        only_coin_config: false,
        thread_count: threadCount.value
      };

      const result = await invoke('query_balances_with_updates', { params });

      if (result.success || result.items) {
        // 更新数据 - 无论总体是否成功，都要更新单条记录的状态
        result.items.forEach((resultItem, index) => {
          if (data.value[index]) {
            Object.assign(data.value[index], resultItem);
          }
        });

        // 统计成功和失败的数量
        const successCount = result.items.filter(item => item.exec_status === '2').length;
        const failCount = result.items.filter(item => item.exec_status === '3').length;
        const totalCount = result.items.length;

        console.log(`查询完成: 成功 ${successCount}, 失败 ${failCount}, 总计 ${totalCount}`);

        if (successCount === totalCount) {
          Notification.success('查询成功！');
        } else if (successCount > 0) {
          Notification.warning(`查询完成！成功 ${successCount} 条，失败 ${failCount} 条`);
        } else {
          Notification.error('查询失败：所有记录都查询失败');
        }
      } else {
        // 只有在没有返回任何结果时才设置所有项目为失败状态
        data.value.forEach(item => {
          item.exec_status = '3';
          item.error_msg = result.error_msg || '查询失败！';
        });
        Notification.error('查询失败：' + (result.error_msg || '未知错误'));
      }

    } catch (error) {
      console.error('查询失败:', error);

      // 设置所有项目为失败状态
      data.value.forEach(item => {
        item.exec_status = '3';
        item.error_msg = '查询失败！';
      });

      Notification.error('查询失败：' + error.message);
    }

    balanceLoading.value = false;
  } else {
    Notification.warning("查询 coin 类型错误！");
  }
}

// 删除代币方法
function deleteToken() {
  if (chainValue.value === "okt") {
    Notification.warning(" OKT Chain 暂不支持删除代币！");
    return;
  }
  if (chainValue.value === "evmos") {
    Notification.warning(" EVMOS Chain 暂不支持删除代币！");
    return;
  }
  if (chainValue.value === "geth") {
    Notification.warning(" Goerli Ethereum 暂不支持删除代币！");
    return;
  }
  if (chainValue.value === "sepolia") {
    Notification.warning(" Sepolia Ethereum 暂不支持删除代币！");
    return;
  }
  if (chainValue.value === "scroll") {
    Notification.warning(" Scroll Alpha TestNet 暂不支持删除代币！");
    return;
  }
  if (chainValue.value === "linea") {
    Notification.warning(" Linea MainNet 暂不支持删除代币！");
    return;
  }
  if (chainValue.value === "base") {
    Notification.warning(" Base MainNet 暂不支持删除代币！");
    return;
  }
  deleteTokenVisible.value = true;
}

// 删除代币确认
async function deleteTokenConfirm() {
  console.log("确认删除代币");
  deleteTokenVisible.value = false;
  await invoke("remove_coin", {
    chain: chainValue.value,
    key: currentCoin.value.key,
  })
    .then(() => {
      Notification.success("删除成功！");
      // 删除成功后重新获取代币列表
      chainChange();
    })
    .catch(() => {
      Notification.error("删除失败！");
    });
}

//  转账方法
async function transferFnc(inputData) {
  // 执行转账
  await iterTransfer(inputData)
    .then(() => {
      if (stopFlag.value) {
        Notification.warning("已停止执行！");
      } else {
        const retryData = inputData.filter((item) => item.retry_flag === true);
        if (form.error_retry === "1" && retryData.length > 0) {
          //  存在重试数据
          transferFnc(retryData);
        } else {
          Notification.success("执行完成！");
          stopStatus.value = true;
        }
      }
      startLoading.value = false;
      stopFlag.value = false;
    })
    .catch(() => {
      Notification.error("执行失败！");
      startLoading.value = false;
      stopStatus.value = true;
    });
}

// 执行
function startTransfer() {
  if (balanceLoading.value) {
    Notification.warning("请等待余额查询完成后再执行！");
    return;
  }
  if (data.value.length === 0) {
    Notification.warning("请先导入私钥！");
    return;
  }
  if (data.value.find((item) => !item.private_key || !item.to_addr)) {
    Notification.warning("请检查是否所有私钥都有对应的转账地址！");
    return;
  }
  // 如果转账类型为指定数量并且且为表格指定数量则进行数据校验
  if (form.send_type === '2' && form.amount_from === '1' &&
    data.value.find((item) => !item.amount)) {
    Notification.warning("包含转账金额为空的错误数据请核实！");
    return;
  }
  validateForm()
    .then(async () => {
      console.log("验证通过");
      startLoading.value = true;
      stopFlag.value = false;
      stopStatus.value = false;
      data.value.forEach((item) => {
        item.exec_status = "0";
        item.error_msg = "";
        item.retry_flag = false;
        item.error_count = 0;
      });
      await transferFnc(data.value);
    })
    .catch(() => {
      console.log("验证失败");
      startLoading.value = false;
    });
}

// 执行转账
async function iterTransfer(accountData) {
  // 判断是主币转账还是代币转账
  let contract;
  if (currentCoin.value.coin_type === "token") {
    contract = new ethers.Contract(
      currentCoin.value.contract_address,
      currentCoin.value.abi
    );
  }
  // 遍历所有账户转账
  for (let i = 0; i < accountData.length; i++) {
    try {
      const config = {
        error_count_limit: 3, //  错误次数限制
        error_retry: form.error_retry, // 是否自动失败重试
        chain: chainValue.value,
        chainLayer: currentChain.value.layer,
        l1: currentChain.value.l1,
        scalar: currentChain.value.scalar,
        delay: [form.min_interval, form.max_interval], // 延迟时间
        transfer_type: form.send_type, // 转账类型 1：全部转账 2:转账固定数量 3：转账随机数量  4：剩余随机数量
        transfer_amount: form.amount_from === '1' ? accountData[i].amount : form.send_count, // 转账当前指定的固定金额
        transfer_amount_list: [form.send_min_count, form.send_max_count], // 转账数量 (transfer_type 为 1 时生效) 转账数量在5-10之间随机，第二个数要大于第一个数！！
        left_amount_list: [form.send_min_count, form.send_max_count], // 剩余数量 (transfer_type 为 2 时生效) 剩余数量在4-6之间随机，第二个数要大于第一个数！！
        amount_precision: Number(form.amount_precision), // 一般无需修改，转账个数的精确度 6 代表个数有6位小数
        limit_type: form.limit_type, // limit_type 限制类型 1：自动 2：指定数量 3：范围随机
        limit_count: form.limit_count, // limit_count 指定数量 (limit_type 为 2 时生效)
        limit_count_list: [form.limit_min_count, form.limit_max_count],
        gas_price_type: form.gas_price_type, // gas price类型 1: 自动 2：固定gas price 3：gas price溢价率
        gas_price_rate: Number(form.gas_price_rate) / 100, // gas price溢价率，0.05代表gas price是当前gas price的105%
        gas_price: form.gas_price, // 设置最大的gas price，单位gwei
        max_gas_price: form.max_gas_price, // 设置最大的gas price，单位gwei
      };
      if (currentCoin.value.coin_type === "base") {
        if (stopFlag.value) {
          stopStatus.value = true;
          return;
        }
        // 设置状态 为执行中
        accountData[i].exec_status = "1";
        await base_coin_transfer
          .single_transfer(i + 1, accountData[i], config)
          .then((res) => {
            accountData[i].exec_status = "2";
            accountData[i].error_msg = res;
          })
          .catch((err) => {
            if (err === "base gas price 超出最大值限制") {
              Notification.error("base gas price 超出最大值限制");
              // 停止
              stopTransfer();
              accountData[i].exec_status = "0";
              accountData[i].error_msg = "";
            } else {
              accountData[i].exec_status = "3";
              accountData[i].error_msg = err;
            }
          });
      } else if (currentCoin.value.coin_type === "token") {
        if (stopFlag.value) {
          stopStatus.value = true;
          return;
        }
        // 设置状态 为执行中
        accountData[i].exec_status = "1";
        await token_transfer
          .single_transfer(i + 1, accountData[i], config, contract)
          .then((res) => {
            accountData[i].exec_status = "2";
            accountData[i].error_msg = res;
          })
          .catch((err) => {
            if (err === "base gas price 超出最大值限制") {
              Notification.error("base gas price 超出最大值限制");
              // 停止
              stopTransfer();
              accountData[i].exec_status = "0";
              accountData[i].error_msg = "";
            } else {
              accountData[i].exec_status = "3";
              accountData[i].error_msg = err;
            }
          });
      } else {
        Notification.error("未知币种类型");
        console.log("未知币种类型：", currentCoin.value.coin_type);
        return;
      }
    } catch (e) {
      console.log("序号：", i + 1, "交易失败！");
      console.log(e);
    }
  }
}

// 停止执行
function stopTransfer() {
  startLoading.value = false;
  stopFlag.value = true;
}

// 校验数据是否合规
function validateForm() {
  return new Promise((resolve, reject) => {
    if (
      checkSendType() &&
      checkPrecision() &&
      checkDelay() &&
      checkGasLimit() &&
      checkGasPrice()
    ) {
      resolve();
    } else {
      reject();
    }
  });
}

const formRef = ref(null);

// 检验发送类型
function checkSendType() {
  if (form.send_type === "1") {
    return true;
  } else if (form.send_type === "2") {
    const bool = utils.checkNum(form.send_count) && Number(form.send_count) > 0;
    if (form.amount_from === "2" && !bool) {
      Notification.error("发送数量必须为数字且大于0");
      formRef.value.setFields({
        send_count: {
          status: "error",
          message: "发送数量必须为数字且大于0",
        },
      });
      return false;
    } else {
      return true;
    }
  } else if (form.send_type === "3" || form.send_type === "4") {
    const bool =
      utils.checkNum(form.send_min_count) &&
      utils.checkNum(form.send_max_count) &&
      Number(form.send_min_count) > 0 &&
      Number(form.send_max_count) > 0;
    if (!bool) {
      const msg =
        form.send_type === "4"
          ? "剩余数量必须为数字且大于0"
          : "发送数量必须为数字且大于0";
      Notification.error(msg);
      formRef.value.setFields({
        send_count_scope: {
          status: "error",
          message: "数量范围错误",
        },
      });
      return false;
    }
    if (Number(form.send_min_count) > Number(form.send_max_count)) {
      const msg =
        form.send_type === "4"
          ? "最大剩余数量应该大于等于最小剩余数量"
          : "最大发送数量应该大于等于最小发送数量";
      Notification.error(msg);
      formRef.value.setFields({
        send_count_scope: {
          status: "error",
          message: "数量范围错误",
        },
      });
      return false;
    }
    return true;
  } else {
    Notification.error("发送类型错误");
    return false;
  }
}

// 检验精度
function checkPrecision() {
  const bool =
    utils.checkNum(form.amount_precision) &&
    Number(form.amount_precision) > 0 &&
    Number(form.amount_precision) < 18;
  if (!bool) {
    Notification.error("金额精度必须为数字且大于0小于18");
    formRef.value.setFields({
      amount_precision: {
        status: "error",
        message: "应大于0小于18",
      },
    });
    return false;
  } else {
    return true;
  }
}

// 检验 Gas Price
function checkGasPrice() {
  if (form.gas_price_type === "1") {
    return true;
  } else if (form.gas_price_type === "2") {
    const bool = utils.checkNum(form.gas_price) && Number(form.gas_price) > 0;
    if (!bool) {
      Notification.error("Gas Price必须为数字且大于0");
      formRef.value.setFields({
        gas_price: {
          status: "error",
          message: "必须为数字且大于0",
        },
      });
      return false;
    } else {
      return true;
    }
  } else if (form.gas_price_type === "3") {
    const bool = utils.checkPositiveInteger(form.gas_price_rate);
    if (!bool) {
      Notification.error("Gas Price 提高比例应为正整数");
      formRef.value.setFields({
        gas_price_rate: {
          status: "error",
          message: "比例应为正整数",
        },
      });
      return false;
    }
    // 如果有最大Gas Price
    if (form.max_gas_price) {
      const bool1 =
        utils.checkNum(form.max_gas_price) && Number(form.max_gas_price) > 0;
      if (!bool1) {
        Notification.error("最大 Gas Price 设置必须为数字且大于0");
        formRef.value.setFields({
          max_gas_price: {
            status: "error",
            message: "必须为数字且大于0",
          },
        });
        return false;
      } else {
        return true;
      }
    } else {
      return true;
    }
  } else {
    Notification.error("Gas Price 方式错误");
    return false;
  }
}

// 检验 Gas Limit
function checkGasLimit() {
  if (form.limit_type === "1") {
    return true;
  } else if (form.limit_type === "2") {
    const bool = utils.checkPositiveInteger(form.limit_count);
    if (!bool) {
      Notification.error("Gas Limit 数量必须为正整数");
      formRef.value.setFields({
        limit_count: {
          status: "error",
          message: "数量必须为正整数",
        },
      });
      return false;
    } else {
      return true;
    }
  } else if (form.limit_type === "3") {
    const bool =
      utils.checkPositiveInteger(form.limit_min_count) &&
      utils.checkPositiveInteger(form.limit_max_count);
    if (!bool) {
      Notification.error("Gas Limit 数量范围必须为正整数");
      formRef.value.setFields({
        limit_count_scope: {
          status: "error",
          message: "数量范围必须为正整数",
        },
      });
      return false;
    }
    if (Number(form.limit_min_count) > Number(form.limit_max_count)) {
      Notification.error("最大 Gas Limit 数量应该大于等于最小 Gas Limit 数量");
      formRef.value.setFields({
        limit_count_scope: {
          status: "error",
          message: "范围错误",
        },
      });
      return false;
    }
    return true;
  } else {
    Notification.error("Gas Limit 类型错误");
    return false;
  }
}

// 检验 间隔时间
function checkDelay() {
  const bool =
    (form.min_interval === "0" ||
      utils.checkPositiveInteger(form.min_interval)) &&
    (form.max_interval === "0" ||
      utils.checkPositiveInteger(form.max_interval));
  if (!bool) {
    Notification.error("发送间隔必须为正整数或者0");
    formRef.value.setFields({
      interval_scope: {
        status: "error",
        message: "发送间隔必须为正整数或者0",
      },
    });
    return false;
  }
  if (Number(form.min_interval) > Number(form.max_interval)) {
    Notification.error("最大间隔应该大于等于最小间隔");
    formRef.value.setFields({
      interval_scope: {
        status: "error",
        message: "最大间隔应该大于等于最小间隔",
      },
    });
    return false;
  }
  return true;
}

function selectSucceeded() {
  selectedKeys.value = data.value
    .filter((item) => item.exec_status === "2")
    .map((item) => item.key);
}

function selectFailed() {
  selectedKeys.value = data.value
    .filter((item) => item.exec_status === "3")
    .map((item) => item.key);
}

// 反选
function InvertSelection() {
  selectedKeys.value = data.value
    .filter((item) => selectedKeys.value.indexOf(item.key) < 0)
    .map((item) => item.key);
}

function deleteSelected() {
  if (startLoading.value) {
    Notification.warning("请停止或等待执行完成后再删除数据！");
    return;
  }
  data.value = data.value.filter(
    (item) => !selectedKeys.value.includes(item.key)
  );
  Notification.success("删除成功");
}

// 返回首页
function goHome() {
  router.push({
    name: "home",
  });
}

// 链管理相关方法
// 显示链管理弹窗
function showChainManage() {
  chainManageVisible.value = true;
  loadChainManageData();
}

// 加载链管理数据
async function loadChainManageData() {
  chainTableLoading.value = true;
  try {
    const chainList = await invoke("get_chain_list");
    chainManageData.value = chainList.map(chain => ({
      key: chain.key,
      chain_key: chain.key,
      chain_name: chain.name,
      chain_id: chain.chain_id,
      symbol: chain.symbol,
      currency_name: chain.currency_name,
      pic_url: chain.pic_url,
      scan_url: chain.scan_url,
      scan_api: chain.scan_api,
      verify_api: chain.verify_api,
      check_verify_api: chain.check_verify_api,
      rpc_urls: chain.rpc_urls
    }));
  } catch (error) {
    console.error('加载链数据失败:', error);
    Notification.error('加载链数据失败：' + error);
  } finally {
    chainTableLoading.value = false;
  }
}

// 显示添加链弹窗
function showAddChain() {
  // 设置为添加模式
  isEditMode.value = false;
  currentEditChain.value = null;
  
  // 重置表单
  Object.assign(chainForm, {
    chain_key: '',
    chain_name: '',
    chain_id: '',
    native_currency_symbol: '',
    native_currency_name: '',
    native_currency_decimals: 18,
    pic_url: '',
    scan_url: '',
    scan_api: '',
    verify_api: '',
    check_verify_api: '',
    rpc_urls: ['']
  });
  chainFormVisible.value = true;
}

// 添加RPC URL输入框
function addRpcUrl() {
  chainForm.rpc_urls.push('');
}

// 删除RPC URL输入框
function removeRpcUrl(index) {
  if (chainForm.rpc_urls.length > 1) {
    chainForm.rpc_urls.splice(index, 1);
  }
}

// 上传链图标
async function uploadChainIcon(option) {
  try {
    // 从 option 中解构出 file 对象，兼容不同的参数格式
    let file = option.file || option;
    
    // 检查 file 对象是否存在
    if (!file) {
      Notification.error('未选择文件');
      return;
    }
    
    // 尝试从不同的属性中获取真正的文件对象
    if (file.fileItem) {
      file = file.fileItem;
    } else if (file.originFileObj) {
      file = file.originFileObj;
    } else if (file.file) {
      file = file.file;
    }
    
    // 检查文件对象是否有必要的属性（name, size等）
    if (!file || typeof file !== 'object' || !file.name) {
      console.error('无法获取有效的文件对象:', file);
      Notification.error('文件对象无效，请重新选择文件');
      return;
    }
    
    // 如果不是标准的File或Blob对象，但有必要的属性，尝试重构为File对象
    if (!(file instanceof File) && !(file instanceof Blob)) {
      console.warn('文件对象不是标准的File或Blob类型，尝试重构为File对象:', file);
      
      // 检查是否有原始文件数据
      let fileData = null;
      let fileName = file.name || 'unknown';
      let fileType = file.type || '';
      
      // 尝试从不同属性获取文件数据
      if (file.raw) {
        fileData = file.raw;
      } else if (file.originFileObj) {
        fileData = file.originFileObj;
      } else if (file.file) {
        fileData = file.file;
      } else if (file instanceof ArrayBuffer) {
        fileData = new Blob([file], { type: fileType });
      } else if (file.buffer) {
        fileData = new Blob([file.buffer], { type: fileType });
      }
      
      // 如果找到了文件数据，创建新的File对象
      if (fileData && (fileData instanceof File || fileData instanceof Blob)) {
        file = fileData;
        console.log('成功重构文件对象:', file);
      } else if (fileData instanceof ArrayBuffer) {
        file = new Blob([fileData], { type: fileType });
        console.log('从ArrayBuffer创建Blob对象:', file);
      } else {
        console.error('无法重构文件对象，没有找到有效的文件数据:', file);
        Notification.error('文件对象格式不支持，请重新选择文件');
        return;
      }
    }

    console.log('上传文件：', file);
    console.log('文件名：', file.name);
    console.log('文件类型：', file.type);
    console.log('文件大小：', file.size);
    console.log('文件对象类型：', file.constructor.name);
    console.log('是否为File实例：', file instanceof File);
    console.log('是否为Blob实例：', file instanceof Blob);
    
    // 如果file.type为undefined，尝试从文件名推断类型
    let fileType = file.type;
    if (!fileType && file.name) {
      const extension = file.name.split('.').pop().toLowerCase();
      const typeMap = {
        'png': 'image/png',
        'jpg': 'image/jpeg',
        'jpeg': 'image/jpeg',
        'gif': 'image/gif',
        'svg': 'image/svg+xml',
        'webp': 'image/webp'
      };
      fileType = typeMap[extension] || '';
      console.log('从文件扩展名推断的类型：', fileType);
    }
    
    // 检查文件类型
    if (!fileType || !fileType.startsWith('image/')) {
      Notification.error('请选择图片文件（支持PNG、JPG、JPEG、GIF、SVG、WEBP格式）');
      return;
    }
    
    // 检查是否输入了图标文件名
    if (!chainForm.pic_url.trim()) {
      Notification.error('请先输入图标文件名');
      return;
    }
    
    // 获取文件扩展名
    const fileExtension = file.name.split('.').pop().toLowerCase();
    const allowedExtensions = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'];
    
    if (!allowedExtensions.includes(fileExtension)) {
      Notification.error('不支持的图片格式，请选择 PNG、JPG、JPEG、GIF、SVG 或 WEBP 格式');
      return;
    }
    
    // 构建目标文件名（使用输入的文件名，但保持原始扩展名）
    let targetFileName = chainForm.pic_url.trim();
    if (!targetFileName.includes('.')) {
      targetFileName += '.' + fileExtension;
      chainForm.pic_url = targetFileName; // 更新表单中的文件名
    }
    
    // 读取文件内容 - 确保文件对象兼容性
    let uint8Array;
    
    // 最后检查：确保文件对象是可读取的
    if (!(file instanceof File) && !(file instanceof Blob)) {
      console.error('经过重构后文件对象仍然不是File或Blob类型:', file);
      Notification.error('文件对象类型不兼容，无法读取');
      return;
    }
    
    try {
      // 方法1: 直接使用 FileReader
      uint8Array = await new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => {
          const arrayBuffer = reader.result;
          resolve(new Uint8Array(arrayBuffer));
        };
        reader.onerror = (error) => {
          console.error('FileReader错误详情:', error);
          reject(new Error('FileReader读取失败: ' + (error.target?.error?.message || '未知错误')));
        };
        
        try {
          reader.readAsArrayBuffer(file);
        } catch (readError) {
          console.error('FileReader.readAsArrayBuffer调用失败:', readError);
          reject(new Error('FileReader调用失败: ' + readError.message));
        }
      });
    } catch (error1) {
      console.warn('FileReader读取失败，尝试其他方法:', error1);
      try {
        // 方法2: 如果文件有 arrayBuffer 方法
        if (file.arrayBuffer && typeof file.arrayBuffer === 'function') {
          const arrayBuffer = await file.arrayBuffer();
          uint8Array = new Uint8Array(arrayBuffer);
        } else {
          throw new Error('文件对象不支持 arrayBuffer 方法');
        }
      } catch (error2) {
        console.warn('arrayBuffer方法失败，尝试最后的方法:', error2);
        try {
          // 方法3: 如果文件有 stream 方法
          if (file.stream && typeof file.stream === 'function') {
            const stream = file.stream();
            const response = new Response(stream);
            const arrayBuffer = await response.arrayBuffer();
            uint8Array = new Uint8Array(arrayBuffer);
          } else {
            throw new Error('文件对象不支持任何读取方法');
          }
        } catch (error3) {
          console.error('所有文件读取方法都失败:', { error1, error2, error3 });
          throw new Error('无法读取文件内容：文件对象不支持读取操作');
        }
      }
    }
    
    // 调用 Tauri 命令保存文件
    await invoke('save_chain_icon', {
      fileName: targetFileName,
      fileData: Array.from(uint8Array)
    });
    
    Notification.success('图标上传成功！');
  } catch (error) {
    console.error('上传图标失败:', error);
    Notification.error('上传图标失败：' + error);
  }
}

// 提交链表单（统一处理添加和编辑）
async function submitChainForm() {
  try {
    // 验证必填项
    if (!chainForm.chain_key.trim()) {
      Notification.warning('请输入链标识符');
      return false;
    }
    if (!chainForm.chain_name.trim()) {
      Notification.warning('请输入链名称');
      return false;
    }
    if (!chainForm.chain_id) {
      Notification.warning('请输入链ID');
      return false;
    }
    if (!chainForm.native_currency_symbol.trim()) {
      Notification.warning('请输入原生代币符号');
      return false;
    }
    if (!chainForm.native_currency_name.trim()) {
      Notification.warning('请输入原生代币名称');
      return false;
    }
    
    // 过滤空的RPC URLs
    const filteredRpcUrls = chainForm.rpc_urls.filter(url => url.trim());
    
    const requestData = {
      chain_name: chainForm.chain_name,
      chain_id: parseInt(chainForm.chain_id),
      native_currency_symbol: chainForm.native_currency_symbol,
      native_currency_name: chainForm.native_currency_name,
      native_currency_decimals: chainForm.native_currency_decimals,
      pic_url: chainForm.pic_url,
      scan_url: chainForm.scan_url,
      scan_api: chainForm.scan_api,
      verify_api: chainForm.verify_api,
      check_verify_api: chainForm.check_verify_api,
      rpc_urls: filteredRpcUrls.length > 0 ? filteredRpcUrls : null
    };
    
    if (isEditMode.value) {
      // 更新链
      await invoke('update_chain', {
        chainKey: chainForm.chain_key,
        requestJson: JSON.stringify(requestData)
      });
      Notification.success('编辑链成功！');
    } else {
      // 添加链
      await invoke('add_chain', { requestJson: JSON.stringify({
        ...requestData,
        chain_key: chainForm.chain_key
      })});
      Notification.success('添加链成功！');
    }
    
    // 刷新链列表
    loadChainManageData();
    
    // 重新加载主页面的链选择器
    const chainList = await invoke("get_chain_list");
    chainOptions.value = chainList.filter((item) => item.key !== "starknet");
    
    // 如果编辑的是当前选中的链，更新当前链的信息
    if (isEditMode.value && chainValue.value === chainForm.chain_key) {
      currentChain.value = chainOptions.value.find(item => item.key === chainForm.chain_key);
    }
    
    chainFormVisible.value = false;
    return true;
  } catch (error) {
    console.error('链操作失败:', error);
    Notification.error('链操作失败：' + error);
    return false;
  }
}

// 删除链
async function deleteChain(chainKey) {
  try {
    await invoke('remove_chain', { chainKey });
    Notification.success('删除链成功！');
    
    // 刷新链列表
    loadChainManageData();
    
    // 重新加载主页面的链选择器
    const chainList = await invoke("get_chain_list");
    chainOptions.value = chainList.filter((item) => item.key !== "starknet");
    
    // 如果删除的是当前选中的链，切换到第一个链
    if (chainValue.value === chainKey && chainOptions.value.length > 0) {
      chainValue.value = chainOptions.value[0].key;
      currentChain.value = chainOptions.value[0];
      await chainChange();
    }
  } catch (error) {
    console.error('删除链失败:', error);
    Notification.error('删除链失败：' + error);
  }
}

// 关闭链管理弹窗
function closeChainManage() {
  chainManageVisible.value = false;
}

// 关闭添加链弹窗
function closeAddChain() {
  chainFormVisible.value = false;
}

// 显示编辑链弹窗
async function showEditChain(record) {
  try {
    // 设置为编辑模式
    isEditMode.value = true;

    // 获取链详情
    const chainDetail = await invoke('get_chain_detail', { chainKey: record.chain_key });
    if (chainDetail) {
      // 填充表单数据
      Object.assign(chainForm, {
        chain_key: chainDetail.chain_key,
        chain_name: chainDetail.chain_name,
        chain_id: chainDetail.chain_id.toString(),
        native_currency_symbol: chainDetail.native_currency_symbol,
        native_currency_name: chainDetail.native_currency_name,
        native_currency_decimals: chainDetail.native_currency_decimals,
        pic_url: chainDetail.pic_url || '',
        scan_url: chainDetail.scan_url || '',
        scan_api: chainDetail.scan_api || '',
        verify_api: chainDetail.verify_api || '',
        check_verify_api: chainDetail.check_verify_api || '',
        rpc_urls: chainDetail.rpc_urls && chainDetail.rpc_urls.length > 0 ? chainDetail.rpc_urls : ['']
      });
      currentEditChain.value = record;
      chainFormVisible.value = true;
    }
  } catch (error) {
    console.error('获取链详情失败:', error);
    Notification.error('获取链详情失败：' + error);
  }
}

// 关闭编辑链弹窗
function closeEditChain() {
  editChainVisible.value = false;
  currentEditChain.value = null;
}

// 保存编辑的链
async function saveEditedChain() {
  try {
    // 验证必填项
    if (!editChainForm.chain_name.trim()) {
      Notification.warning('请输入链名称');
      return false;
    }
    if (!editChainForm.chain_id) {
      Notification.warning('请输入链ID');
      return false;
    }
    if (!editChainForm.native_currency_symbol.trim()) {
      Notification.warning('请输入原生代币符号');
      return false;
    }
    if (!editChainForm.native_currency_name.trim()) {
      Notification.warning('请输入原生代币名称');
      return false;
    }
    
    // 过滤空的RPC URLs
    const filteredRpcUrls = editChainForm.rpc_urls.filter(url => url.trim());
    
    const requestData = {
      chain_name: editChainForm.chain_name,
      chain_id: parseInt(editChainForm.chain_id),
      native_currency_symbol: editChainForm.native_currency_symbol,
      native_currency_name: editChainForm.native_currency_name,
      native_currency_decimals: editChainForm.native_currency_decimals,
      pic_url: editChainForm.pic_url,
      scan_url: editChainForm.scan_url,
      scan_api: editChainForm.scan_api,
      verify_api: editChainForm.verify_api,
      check_verify_api: editChainForm.check_verify_api,
      rpc_urls: filteredRpcUrls.length > 0 ? filteredRpcUrls : null
    };
    
    await invoke('update_chain', { 
      chainKey: editChainForm.chain_key, 
      requestJson: JSON.stringify(requestData) 
    });
    Notification.success('更新链成功！');
    
    // 刷新链列表
    loadChainManageData();
    
    // 重新加载主页面的链选择器
    const chainList = await invoke("get_chain_list");
    chainOptions.value = chainList.filter((item) => item.key !== "starknet");
    
    // 如果编辑的是当前选中的链，更新当前链的信息
    if (chainValue.value === editChainForm.chain_key) {
      currentChain.value = chainOptions.value.find(item => item.key === editChainForm.chain_key);
    }
    
    editChainVisible.value = false;
    return true;
  } catch (error) {
    console.error('更新链失败:', error);
    Notification.error('更新链失败：' + error);
    return false;
  }
}

// 编辑链时添加RPC URL输入框
function addEditRpcUrl() {
  editChainForm.rpc_urls.push('');
}

// 编辑链时删除RPC URL输入框
function removeEditRpcUrl(index) {
  if (editChainForm.rpc_urls.length > 1) {
    editChainForm.rpc_urls.splice(index, 1);
  }
}

// 标题栏控制方法
async function minimizeWindow() {
  try {
    const currentWindow = getCurrentWindow()
    await currentWindow.minimize()
  } catch (error) {
    console.error('Error minimizing window:', error)
  }
}

async function maximizeWindow() {
  try {
    const currentWindow = getCurrentWindow()
    await currentWindow.toggleMaximize()
  } catch (error) {
    console.error('Error maximizing window:', error)
  }
}

async function closeWindow() {
  try {
    const currentWindow = getCurrentWindow()
    await currentWindow.close()
  } catch (error) {
    console.error('Error closing window:', error)
  }
}
</script>

<template>
  <!-- 自定义标题栏 -->
  <div class="title-bar">
    <div class="title-bar-text">{{ windowTitle }}</div>
    <div class="title-bar-controls">
      <button class="title-bar-control" @click="minimizeWindow" title="最小化">
        <span class="minimize-icon">—</span>
      </button>
      <button class="title-bar-control" @click="maximizeWindow" title="最大化">
        <span class="maximize-icon">□</span>
      </button>
      <button class="title-bar-control close" @click="closeWindow" title="关闭">
        <span class="close-icon">×</span>
      </button>
    </div>
  </div>

  <div class="container transfer" style="height: calc(100vh - 30px); display: flex; flex-direction: column; overflow: hidden;">
    <!-- <span class="pageTitle">批量转账</span> -->
    <!-- 工具栏 -->
    <div class="toolBar" style="flex-shrink: 0;">
      <a-button type="primary" @click="handleClick('send')">录入发送方
      </a-button>
      <a-button type="primary" style="margin-left: 10px" @click="handleClick('receive')">录入接收地址
      </a-button>
      <a-divider direction="vertical" />
      <a-button type="outline" status="normal" @click="downloadFile">下载模板
      </a-button>
      <a-button type="primary" status="success" style="margin-left: 10px" @click="upload">导入文件
      </a-button>
      <input type="file" ref="uploadInputRef" @change="UploadFile" id="btn_file" style="display: none" />
      <a-divider direction="vertical" />
      <!-- 选择操作区按钮 -->
      <a-button type="outline" status="success" @click="selectSucceeded">选中成功
      </a-button>
      <a-button type="outline" status="danger" style="margin-left: 10px" @click="selectFailed">选中失败
      </a-button>
      <a-button type="outline" status="normal" style="margin-left: 10px" @click="InvertSelection">反选
      </a-button>
      <a-button type="primary" status="danger" style="margin-left: 10px" @click="deleteSelected">删除选中
      </a-button>
      <a-button v-show="false" class="goHome" type="outline" status="success" @click="goHome">
        <template #icon>
          <icon-double-left />
        </template>
        返回首页
      </a-button>
      <a-button type="outline" status="normal" style="float: right; margin-right: 10px" @click="clearData">清空列表
      </a-button>
      <a-button type="outline" status="normal" style="float: right; margin-right: 10px" @click="clearAddress">清空地址
      </a-button>
    </div>
    <!-- 操作账号表格 -->
    <div class="mainTable" style="flex: 1; overflow: hidden; display: flex; flex-direction: column; min-height: 0;">
      <a-table v-if="tableBool" row-key="key" :columns="columns" :column-resizable="true" :data="data"
        :row-selection="rowSelection" :loading="tableLoading" :scrollbar="scrollbar"
        @row-click="rowClick" v-model:selectedKeys="selectedKeys" :pagination="pagination" style="height: 100%;">
        <template #index="{ rowIndex }">
          {{ rowIndex + 1 }}
        </template>
        <template #exec_status="{ rowIndex }">
          <a-tag v-if="data[rowIndex].exec_status === '0'" color="#86909c">等待执行
          </a-tag>
          <a-tag v-if="data[rowIndex].exec_status === '1'" color="#ff7d00">执行中
          </a-tag>
          <a-tag v-if="data[rowIndex].exec_status === '2'" color="#00b42a">执行成功
          </a-tag>
          <a-tag v-if="data[rowIndex].exec_status === '3'" color="#f53f3f">执行失败
          </a-tag>
        </template>
        <template #optional="{ record }">
          <icon-delete style="font-size: 16px" @click.prevent="deleteItem(record)" />
        </template>
      </a-table>
    </div>
    <!-- 管理代币按钮嵌入 -->
    <div style="display: flex; gap: 10px; align-items: center; margin-top: 10px; flex-shrink: 0;">
      <!-- 链管理按钮 -->
      <a-button type="outline" @click="showChainManage" style="white-space: nowrap;">
        区块链管理
      </a-button>
      <!-- 链 选择器 -->
      <a-select v-model="chainValue" :options="chainOptions" @change="chainChange" :field-names="chainFieldNames"
        size="large" :style="{ width: '65%' }">
        <template #label="{ data }">
          <div style="
            display: flex;
            flex-direction: row;
            align-items: center;
            width: 100%;
          ">
            <span style="color: gray;">区块链：</span>
            <img alt="" :src="`/chainIcons/${data?.pic_url}`" style="width: 18px; height: 18px" />
            <span style="margin-left: 10px">{{ data?.name }}</span>
            <span style="margin-left: 20px;color: #c3c3c3;">{{ data?.scan_url }}</span>
            <span v-show="chainValue !== 'sol'" style="flex: 1; text-align: end; color: #00b42a">Gas Price: {{
              data?.gas_price ?? "未知" }}</span>
          </div>
        </template>
        <template #option="{ data }">
          <div style="display: flex; flex-direction: row; align-items: center;height: 32px;">
            <img alt="" :src="`/chainIcons/${data?.pic_url}`" style="width: 18px; height: 18px" />
            <span style="margin-left: 10px">{{ data?.name }}</span>
            <span style="margin-left: 20px;color: #c3c3c3;">{{ data?.scan_url }}</span>
          </div>
        </template>
      </a-select>
      <a-button type="outline" @click="handleAddCoinClick" style="white-space: nowrap;" >
        代币管理
      </a-button>
      <!-- 代币 选择器 -->
      <a-select v-model="coinValue" :options="coinOptions" :field-names="coinFieldNames" :style="{ width: '30%' }"
        @change="coinChange">
        <template #label="{ data }">
          <span style="color: gray;">代币：</span>
          <span style="margin-left: 10px">{{ data?.label }}</span>
        </template>
        <template #option="{ data }">
          <span style="margin-left: 10px">{{ data?.label }}</span>
        </template>
      </a-select>
    </div>
    
    <!-- 细节配置 -->
    <div style="display: flex; padding-top: 5px; flex-shrink: 0;">
    <!-- 细节配置 -->
    <a-form ref="formRef" :model="form" :style="{ width: '100%' }" layout="vertical">
      <a-row style="height: 70px">
        <a-form-item field="send_type" label="发送方式" style="width: 315px; padding: 5px 10px;">
          <a-radio-group v-model="form.send_type" type="button">
            <a-radio value="1">全部</a-radio>
            <a-radio value="2">指定数量</a-radio>
            <a-radio value="3">范围随机</a-radio>
            <a-radio value="4">剩余数量</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item v-if="form.send_type === '2'" field="amount_from" label="数量来源" tooltip="如果选择表格数据则应导入带有转账数量的表格数据"
          style="width: 180px; padding: 5px 10px;">
          <a-radio-group v-model="form.amount_from" type="button">
            <a-radio value="1">表格数据</a-radio>
            <a-radio value="2">当前指定</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item v-if="form.send_type === '2' && form.amount_from === '2'" field="send_count" label="发送数量"
          style="width: 150px; padding: 5px 10px;">
          <a-input v-model="form.send_count" />
        </a-form-item>
        <a-form-item v-if="form.send_type === '3' || form.send_type === '4'" field="send_count_scope"
          :label="form.send_type === '3' ? '发送数量范围' : '剩余数量范围'" style="width: 180px; padding: 5px 10px;">
          <a-input v-model="form.send_min_count" />
          <span style="padding: 0 5px">至</span>
          <a-input v-model="form.send_max_count" />
        </a-form-item>
        <a-form-item v-if="form.send_type === '3' || form.send_type === '4'" field="amount_precision" label="金额精度"
          style="width: 110px; padding: 5px 10px;" tooltip="金额小数点位数">
          <a-input v-model="form.amount_precision" />
        </a-form-item>
        <a-divider direction="vertical" style="height: 50px; margin: 15px 10px 0 10px;" />
        <a-form-item field="interval_scope" label="发送间隔（秒）" style="width: 215px; padding: 5px 10px;">
          <a-input v-model="form.min_interval" />
          <span style="padding: 0 5px">至</span>
          <a-input v-model="form.max_interval" />
        </a-form-item>
        <a-form-item field="thread_count" label="线程数" style="width: 130px; padding: 5px 10px;" tooltip="同时执行的钱包数量">
          <a-input-number v-model="threadCount" :min="1" :max="100" :step="1" :default-value="1" mode="button" />
        </a-form-item>
        <a-form-item field="error_retry" label="失败自动重试" style="width: 110px; padding: 5px 10px;" tooltip="转账失败时是否自动重试">
          <a-switch v-model="form.error_retry" checked-value="1" unchecked-value="0" />
        </a-form-item>
      </a-row>
      <a-row v-show="chainValue !== 'sol'" style="height: 70px">
        <a-form-item field="limit_type" label="Gas Limit" style="width: 230px; padding: 5px 10px;">
          <a-radio-group v-model="form.limit_type" type="button">
            <a-radio value="1">自动</a-radio>
            <a-radio value="2">指定数量</a-radio>
            <a-radio value="3">范围随机</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item v-if="form.limit_type === '2'" style="width: 150px; padding: 5px 10px;" field="limit_count"
          label="Gas Limit">
          <a-input v-model="form.limit_count" />
        </a-form-item>
        <a-form-item v-if="form.limit_type === '3'" style="width: 265px; padding: 5px 10px;" field="limit_count_scope"
          label="Gas Limit 范围">
          <a-input v-model="form.limit_min_count" />
          <span style="padding: 0 5px">至</span>
          <a-input v-model="form.limit_max_count" />
        </a-form-item>
        <a-divider direction="vertical" style="height: 50px; margin: 15px 10px 0 10px;" />
        <a-form-item field="gas_price_type" label="Gas Price 方式" style="width: 225px; padding: 5px 10px;">
          <a-radio-group v-model="form.gas_price_type" type="button">
            <a-radio value="1">自动</a-radio>
            <a-radio value="2">固定值</a-radio>
            <a-radio value="3">指定比例</a-radio>
          </a-radio-group>
        </a-form-item>
        <a-form-item v-if="form.gas_price_type === '2'" field="gas_price" style="width: 120px; padding: 5px 10px;"
          label="Gas Price">
          <a-input v-model="form.gas_price" />
        </a-form-item>
        <a-form-item v-if="form.gas_price_type === '3'" field="gas_price_rate" style="width: 100px; padding: 5px 10px;"
          label="提高比例">
          <a-input v-model="form.gas_price_rate">
            <template #append> %</template>
          </a-input>
        </a-form-item>
        <a-form-item v-if="form.gas_price_type === '1' || form.gas_price_type === '3'" field="max_gas_price"
          style="width: 130px; padding: 5px 10px;" label="最大 Gas Price" tooltip="为空时则不设置上限（单位：Gwei）">
          <a-input v-model="form.max_gas_price" />
        </a-form-item>
      </a-row>
    </a-form>
  </div>
    <!-- 核心操作区 -->
    <div style="display: flex; align-items: center; padding: 10px 20px; margin-top: 5px; justify-content: center; gap: 30px; flex-shrink: 0;">
    <!-- 左侧区域 -->
    <div style="display: flex; align-items: center; gap: 20px;">
      <!-- 查询余额 -->
      <a-dropdown>
        <a-button type="primary" :loading="balanceLoading" :style="{ width: '130px', height: '40px', fontSize: '14px' }">查询余额</a-button>
        <template #content>
          <a-doption @click="queryBalance">⤴️ 查出账地址</a-doption>
          <a-doption disabled>⤵️ 查到账地址</a-doption>
        </template>
      </a-dropdown>
    </div>
    
    <!-- 右侧区域 -->
    <div style="display: flex; align-items: center; gap: 20px;">
      <!-- 执行转账按钮 -->
      <a-button v-if="!startLoading && stopStatus" class="execute-btn" @click="startTransfer(data.value)" :style="{ width: '130px', height: '40px', fontSize: '14px' }">执行转账</a-button>
      <a-tooltip v-else content="点击可以提前停止执行">
        <div @click="stopTransfer">
          <a-button v-if="!stopFlag" class="execute-btn executing" loading :style="{ width: '130px', height: '40px', fontSize: '14px' }">执行中...</a-button>
          <a-button v-if="stopFlag && !stopStatus" class="execute-btn stopping" loading :style="{ width: '130px', height: '40px', fontSize: '14px' }">正在停止...</a-button>
        </div>
      </a-tooltip>
    </div>
  </div>
  </div>
  <a-modal v-model:visible="visible" :width="700" :title="importModalTitle" @cancel="handleCancel"
    :on-before-ok="handleBeforeOk">
    <a-textarea v-model="importText" style="margin-top: 10px" placeholder="格式：一行一个" allow-clear :auto-size="{
        minRows: 15,
        maxRows: 20,
      }" />
  </a-modal>
  <!-- 添加代币弹窗 -->
  <a-modal v-model:visible="addCoinVisible" :width="700" title="添加代币" @cancel="handleAddCoinCancel"
    :on-before-ok="handleAddCoinBeforeOk" unmountOnClose>
    <a-input v-model="coinAddress" placeholder="请输入代币合约地址" allow-clear />
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
    <div>
      确认删除私钥为【
      {{ currentItemKey.substring(0, 15) + "......" }}
      】的数据？
    </div>
    <template #footer>
      <a-button @click="deleteItemCancel">取消</a-button>
      <a-button type="primary" status="danger" @click="deleteItemConfirm" style="margin-left: 10px">确定
      </a-button>
    </template>
  </a-modal>
  
  <!-- 链管理弹窗 -->
  <a-modal v-model:visible="chainManageVisible" title="区块链管理" :width="1100" @cancel="closeChainManage">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <h3 style="margin: 0;">链配置管理</h3>
      <a-button type="primary" @click="showAddChain">
        <icon-plus />
        添加新链
      </a-button>
    </div>
    
    <a-table :data="chainManageData" :loading="chainTableLoading" :pagination="false" :scroll="{ y: 500 }">
      <template #columns>
        <a-table-column title="链标识" data-index="chain_key" :width="80" />
        <a-table-column title="链名称" data-index="chain_name" :width="100" />
        <a-table-column title="链ID" data-index="chain_id" :width="80" />
        <a-table-column title="原生代币" data-index="symbol" :width="80" />
        <a-table-column title="图标" :width="50">
          <template #cell="{ record }">
            <img v-if="record.pic_url" :src="`/chainIcons/${record.pic_url}`" 
                 style="width: 24px; height: 24px; border-radius: 50%;" 
                 :alt="record.chain_name" />
            <span v-else>-</span>
          </template>
        </a-table-column>
        <a-table-column title="浏览器" data-index="scan_url" :width="150" :ellipsis="true" :tooltip="true" />
        <a-table-column title="RPC数量" :width="70">
          <template #cell="{ record }">
            {{ record.rpc_urls ? record.rpc_urls.length : 0 }} 个
          </template>
        </a-table-column>
        <a-table-column title="操作" :width="150">
          <template #cell="{ record }">
            <a-button type="text" @click="showEditChain(record)" size="small">
              编辑
            </a-button>
            <a-popconfirm content="确定要删除这个链吗？这将同时删除该链下的所有代币！" 
                         @ok="deleteChain(record.chain_key)">
              <a-button type="text" status="danger" size="small">
                <icon-delete />
                删除
              </a-button>
            </a-popconfirm>
          </template>
        </a-table-column>
      </template>
    </a-table>
    
    <template #footer>
      <a-button @click="closeChainManage">关闭</a-button>
    </template>
  </a-modal>
  
  <!-- 添加/编辑链弹窗 -->
  <a-modal v-model:visible="chainFormVisible" :title="isEditMode ? '编辑链' : '添加新链'" :width="800" :body-style="{ maxHeight: '400px', overflowY: 'auto' }" @cancel="closeAddChain" :on-before-ok="submitChainForm">
    <a-form :model="chainForm" layout="vertical">
      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="链标识符" required>
            <a-input v-model="chainForm.chain_key" placeholder="例如：eth, bsc, polygon" :disabled="isEditMode" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="链名称" required>
            <a-input v-model="chainForm.chain_name" placeholder="例如：Ethereum, BSC" />
          </a-form-item>
        </a-col>
      </a-row>
      
      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="链ID" required>
            <a-input v-model="chainForm.chain_id" placeholder="例如：1, 56, 137" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="原生代币符号" required>
            <a-input v-model="chainForm.native_currency_symbol" placeholder="例如：ETH, BNB, MATIC" />
          </a-form-item>
        </a-col>
      </a-row>
      
      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="原生代币名称" required>
            <a-input v-model="chainForm.native_currency_name" placeholder="例如：Ethereum, Binance Coin" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="小数位数">
            <a-input-number v-model="chainForm.native_currency_decimals" :min="0" :max="18" />
          </a-form-item>
        </a-col>
      </a-row>
      
      <a-form-item label="图标文件名">
        <div style="display: flex; gap: 8px; align-items: center;">
          <a-input v-model="chainForm.pic_url" placeholder="例如：ethereum.png" style="flex: 1;" />
          <a-upload
            :custom-request="uploadChainIcon"
            :show-file-list="false"
            accept="image/*"
          >
            <template #upload-button>
              <a-button type="outline">
                <icon-upload />
                上传图标
              </a-button>
            </template>
          </a-upload>
        </div>
      </a-form-item>
      
      <a-form-item label="区块链浏览器URL">
        <a-input v-model="chainForm.scan_url" placeholder="例如：https://etherscan.io" />
      </a-form-item>
      
      <a-form-item label="浏览器API">
        <a-input v-model="chainForm.scan_api" placeholder="例如：https://api.etherscan.io/api" />
      </a-form-item>
      
      <a-form-item label="验证API">
        <a-input v-model="chainForm.verify_api" placeholder="验证合约的API地址" />
      </a-form-item>
      
      <a-form-item label="检查验证API">
        <a-input v-model="chainForm.check_verify_api" placeholder="检查合约验证状态的API地址" />
      </a-form-item>
      
      <a-form-item label="RPC URLs">
        <div class="rpc-urls"> 
          <div v-for="(url, index) in chainForm.rpc_urls" :key="index" style="margin-bottom: 8px; display: flex; align-items: center; gap: 8px;">
          <a-input v-model="chainForm.rpc_urls[index]" placeholder="RPC节点地址" style="flex: 1;" />
          <a-button v-if="chainForm.rpc_urls.length > 1" @click="removeRpcUrl(index)" type="outline" status="danger" size="small">
            <icon-delete />
          </a-button>
        </div>
        <div style="margin-top: 8px;">
          <a-button @click="addRpcUrl" type="outline" size="small">
            <icon-plus />
            添加RPC
          </a-button>
        </div>
        </div>
      </a-form-item>
    </a-form>
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
  background-clip: text;
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
  margin-top: 35px;
}

.goHome {
  float: right;
  background-color: white;
}

.goHome:hover {
  background-color: #ffffffa6;
}

.mainTable {
  margin-top: 15px;
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
  margin-top: 10px;
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

/* 执行转账按钮样式 */
.execute-btn {
  width: 150px;
  height: 50px;
  font-size: 16px;
  color: #ffffff;
  background-color: #0fa962;
  border: none;
}

.execute-btn:hover {
  color: #ffffff;
  background-color: #11c06f;
}

.execute-btn.executing {
  background-color: #11c06f;
}

.execute-btn.executing:hover {
  color: #ffffff;
  background-color: #fc0934;
}

.execute-btn.stopping {
  background-color: #11c06f;
}

.execute-btn.stopping:hover {
  color: #ffffff;
  background-color: #fc0934;
}

.arco-radio-button.arco-radio-checked {
  color: #ffffff;
  background-color: #165dff;
}

.importBar {
  display: flex;
}

/* 自定义标题栏样式 */
.title-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 30px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 0 10px;
  -webkit-app-region: drag;
  user-select: none;
  font-size: 14px;
  font-weight: 500;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
}

.title-bar-text {
  font-size: 13px;
  font-weight: 500;
  margin-left: 8px;
}

.title-bar-controls {
  display: flex;
  -webkit-app-region: no-drag;
}

.title-bar-control {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 30px;
  background: transparent;
  border: none;
  color: white;
  cursor: pointer;
  font-size: 16px;
  transition: background-color 0.2s;
  -webkit-app-region: no-drag;
}

.title-bar-control:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.title-bar-control.close:hover {
  background-color: #e81123;
}

.minimize-icon {
  font-size: 14px;
  font-weight: bold;
  margin-top: -2px;
}

.maximize-icon {
  font-size: 14px;
  font-weight: normal;
}

.close-icon {
  font-size: 18px;
  font-weight: normal;
  line-height: 1;
}

/* 调整容器以适应自定义标题栏 */
.container {
  height: calc(100vh - 30px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 隐藏滚动条但保持滚动功能 */
.container::-webkit-scrollbar {
  display: none;
}

.container {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

/* 隐藏表格滚动条 */
.arco-table-content::-webkit-scrollbar {
  display: none;
}

.arco-table-content {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.arco-scrollbar::-webkit-scrollbar {
  display: none;
}

.arco-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.rpc-urls {
  width: 100%;
}
</style>
<style lang="less">
.transfer {
  .arco-table-body {
    min-height: 150px;

    .arco-table-element .arco-empty {
      min-height: 130px;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
    }
  }
}
</style>
