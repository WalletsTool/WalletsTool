<script setup name="transfer">
import { IconDelete, IconDoubleLeft, IconPlus, IconUpload, IconShareExternal } from "@arco-design/web-vue/es/icon";
import { useRouter } from "vue-router";
import { onBeforeMount, onBeforeUnmount, onMounted, reactive, ref, watch, nextTick, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Notification } from "@arco-design/web-vue";
import { ethers } from "ethers";
import { debounce } from "throttle-debounce";
import { read, utils as xlUtils, writeFile } from "xlsx";
import ChainIcon from '../components/ChainIcon.vue'
import TitleBar from '../components/TitleBar.vue'
import ChainManagement from '../components/ChainManagement.vue'
import RpcManagement from '../components/RpcManagement.vue'
import TokenManagement from '../components/TokenManagement.vue'
import WalletImportModal from '../components/WalletImportModal.vue'

const router = useRouter();
// 窗口标题
const windowTitle = ref('Web3 Tools - 批量转账');

// table列名
const columns = [
  {
    title: "序号",
    align: "center",
    width: 60,
    slotName: "index",
  },
  {
    title: "发送方私钥",
    align: "center",
    width: 250,
    dataIndex: "private_key",
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "接收地址",
    align: "center",
    dataIndex: "to_addr",
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "转账数量",
    align: "center",
    dataIndex: "amount",
    width: 120,
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "平台币余额",
    align: "center",
    dataIndex: "plat_balance",
    width: 120,
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "代币余额",
    align: "center",
    dataIndex: "coin_balance",
    width: 120,
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "状态",
    align: "center",
    slotName: "exec_status",
    width: 100,
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "返回信息",
    align: "center",
    dataIndex: "error_msg",
    ellipsis: true,
    tooltip: { position: 'left' },
  },
  {
    title: "操作",
    align: "center",
    slotName: "optional",
    width: 60,
    ellipsis: true,
    tooltip: true,
  },
];
let tableLoading = ref(false);
// 全页面loading状态
let pageLoading = ref(false);
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

// 添加代币弹窗
let addCoinVisible = ref(false);
let coinAddress = ref("");
// 删除代币弹窗
let deleteTokenVisible = ref(false);
// 链管理组件引用
const chainManageRef = ref(null);
// RPC管理组件引用
const rpcManageRef = ref(null);
// 代币管理组件引用
const tokenManageRef = ref(null);
// 钱包导入组件引用
const walletImportRef = ref(null);
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

// 转账进度相关变量
const transferProgress = ref(0); // 转账进度百分比
const transferTotal = ref(0); // 总转账数量
const transferCompleted = ref(0); // 已完成转账数量
const showProgress = ref(false); // 是否显示进度条

// 更新转账进度
function updateTransferProgress() {
  if (!showProgress.value) return;

  // 计算已完成的转账数量（成功或失败都算完成）
  const completed = data.value.filter(item =>
    item.exec_status === '2' || item.exec_status === '3'
  ).length;

  transferCompleted.value = completed;
  // 计算进度百分比
  if (transferTotal.value > 0) {
    transferProgress.value = Number((completed / transferTotal.value).toFixed(2));
  } else {
    transferProgress.value = 0;
  }

  // 如果全部完成，延迟隐藏进度条
  if (completed === transferTotal.value && transferTotal.value > 0) {
    setTimeout(() => {
      showProgress.value = false;
    }, 3000); // 3秒后隐藏进度条
  }
}

// 代币管理相关变量
const tokenTableLoading = ref(false);
const tokenManageData = ref([]);
const tokenFormVisible = ref(false);
const isTokenEditMode = ref(false);
const currentEditToken = ref(null);
const tokenForm = reactive({
  key: '',
  name: '',
  symbol: '',
  decimals: 18,
  type: 'token',
  contract_type: '',
  contract_address: '',
  abi: ''
});



// 获取gas
const timer = setInterval(fetchGas, 5000);

watch(stopStatus, (newValue, oldValue) => {
  // 停止状态变化监听
});



// 初始化RPC列表
onBeforeMount(async () => {
  // 检查是否在Tauri环境中
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
    // 初始化加载链列表
    try {
      const result = await invoke('get_chain_list');
      chainOptions.value = result || [];

      // 设置默认选中第一个链
      if (chainOptions.value.length > 0) {
        chainValue.value = chainOptions.value[0].key;
        currentChain.value = chainOptions.value[0];
        // 获取对应的代币列表
        await chainChange();
      }
    } catch (error) {
      console.error('初始化链列表失败:', error);
    }
  } else {
    // 浏览器环境下的模拟数据
    chainOptions.value = [
      { key: 'eth', name: 'Ethereum', scan_url: 'etherscan.io', pic_url: 'eth.png' },
      { key: 'bnb', name: 'BNB Chain', scan_url: 'bscscan.com', pic_url: 'bnb.png' }
    ];
    chainValue.value = chainOptions.value[0].key;
    currentChain.value = chainOptions.value[0];
    // 获取rpc对应的代币列表
    await chainChange();
  }
});

onMounted(async () => {
  // 获取窗口标题
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow();
      const title = await currentWindow.title();
      if (title) {
        windowTitle.value = title;
      }
    } catch (error) {
      console.error('Error getting window title:', error);
    }
  } else {
    // 浏览器环境下设置默认标题
    windowTitle.value = 'Transfer - Web3 Tools';
  }

  // 页面高度现在通过 CSS 自动调整，无需监听器

  // 监听余额查询更新事件
  const isTauriMounted = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauriMounted) {
    await listen('balance_item_update', (event) => {
      const { index, item } = event.payload;
      if (data.value[index]) {
        // 更新对应索引的数据
        Object.assign(data.value[index], item);
      }
    });

    // 监听转账状态更新事件
    await listen('transfer_status_update', (event) => {
      const { index, error_msg, exec_status } = event.payload;
      if (data.value[index]) {
        // 更新对应索引的状态和返回信息
        data.value[index].error_msg = error_msg;
        data.value[index].exec_status = exec_status;

        // 更新进度条
        updateTransferProgress();
      }
    });
  }

  // 页面加载完成后发送事件
  nextTick(() => {
    setTimeout(() => {
      const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
      if (isTauri) {
        const currentWindow = getCurrentWindow();
        currentWindow.emit('page-loaded');
      }
    }, 50);
  });
});

onBeforeUnmount(() => {
  if (timer) {
    clearInterval(timer);
  }
});

// 读取上传的文件
// 验证私钥格式
function validatePrivateKey(privateKey) {
  try {
    // 检查私钥是否为空或undefined
    if (!privateKey || typeof privateKey !== 'string') {
      return false;
    }

    // 去除首尾空格
    let cleanKey = privateKey.trim();

    // 如果以0x开头，去除0x前缀
    if (cleanKey.startsWith('0x') || cleanKey.startsWith('0X')) {
      cleanKey = cleanKey.slice(2);
    }

    // 检查长度是否为64位
    if (cleanKey.length !== 64) {
      return false;
    }

    // 检查是否为有效的十六进制字符
    if (!/^[0-9a-fA-F]{64}$/.test(cleanKey)) {
      return false;
    }
    // 尝试创建钱包实例验证私钥有效性
    new ethers.Wallet(privateKey);
    return true;
  } catch (error) {
    return false;
  }
}

// 验证地址格式
function validateAddress(address) {
  try {
    // 检查地址是否为空或undefined
    if (!address || typeof address !== 'string') {
      return false;
    }

    // 去除首尾空格
    const trimmedAddress = address.trim();

    // 检查是否以0x开头且长度为42
    if (!trimmedAddress.startsWith('0x') || trimmedAddress.length !== 42) {
      return false;
    }

    // 检查除0x外的部分是否为有效的十六进制字符
    const hexPart = trimmedAddress.slice(2);
    if (!/^[0-9a-fA-F]{40}$/.test(hexPart)) {
      return false;
    }

    // 使用ethers.js进行最终验证
    return ethers.utils.isAddress(trimmedAddress);
  } catch (error) {
    return false;
  }
}

// 导出不合规数据到Excel
function exportInvalidData(invalidData) {
  if (invalidData.length === 0) {
    return;
  }

  // 创建工作簿
  const wb = xlUtils.book_new();
  
  // 创建工作表数据
  const wsData = [
    ['私钥', '地址', '转账数量', '错误原因'], // 表头
    ...invalidData.map(item => [
      item.私钥 || '',
      item.地址 || '',
      item.转账数量 || '',
      item.错误原因 || ''
    ])
  ];
  
  // 创建工作表
  const ws = xlUtils.aoa_to_sheet(wsData);
  
  // 添加工作表到工作簿
  xlUtils.book_append_sheet(wb, ws, '不合规数据');
  
  // 生成文件名（包含时间戳）
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const fileName = `不合规数据_${timestamp}.xlsx`;
  
  // 导出文件
  writeFile(wb, fileName);
}

function UploadFile() {
  let file = uploadInputRef.value.files[0];
  let reader = new FileReader();
  //提取excel中文件内容
  reader.readAsArrayBuffer(file);
  data.value = [];
  // 开启全页面loading
  pageLoading.value = true;
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
    console.log(outdata);
    // 用于存储不合规数据
    const invalidData = [];
    let validCount = 0;
    let invalidCount = 0;
    
    // 这里for循环将excel表格数据转化成json数据
    outdata.forEach((i, index) => {
      const rowNumber = index + 2; // Excel行号（从第2行开始，第1行是表头）
      const privateKey = String(i.私钥 || '').trim();
      const toAddress = String(i.地址 || '').trim();
      const amount = i.转账数量;
      
      // 验证私钥和地址
      const isPrivateKeyValid = privateKey && validatePrivateKey(privateKey);
      const isAddressValid = toAddress && validateAddress(toAddress);
      
      if (isPrivateKeyValid && isAddressValid) {
        // 数据合规，添加到表格
        try {
          // 从私钥生成地址
          const wallet = new ethers.Wallet(privateKey);
          const address = wallet.address;
          
          data.value.push({
            key: privateKey + toAddress,
            private_key: privateKey,
            address: address,
            to_addr: toAddress,
            amount: amount ? String(amount) : "0", // 转账数量为空时显示为0
            plat_balance: "",
            coin_balance: "",
            exec_status: "0",
            error_msg: "",
          });
          validCount++;
        } catch (error) {
          // 私钥无效，添加到不合规数据
          const errorReasons = [];
          errorReasons.push('私钥无效');
          
          invalidData.push({
            私钥: privateKey,
            地址: toAddress,
            转账数量: amount || '',
            错误原因: errorReasons.join('; '),
            行号: rowNumber
          });
          invalidCount++;
        }
      } else {
        // 数据不合规，记录错误原因
        const errorReasons = [];
        if (!isPrivateKeyValid) {
          if (!privateKey) {
            errorReasons.push('私钥为空');
          } else {
            errorReasons.push('私钥格式错误');
          }
        }
        if (!isAddressValid) {
          if (!toAddress) {
            errorReasons.push('地址为空');
          } else {
            errorReasons.push('地址格式错误');
          }
        }
        
        invalidData.push({
          私钥: privateKey,
          地址: toAddress,
          转账数量: amount || '',
          错误原因: errorReasons.join('; '),
          行号: rowNumber
        });
        invalidCount++;
      }
    });
    
    // 如果有不合规数据，导出到本地
    if (invalidData.length > 0) {
      exportInvalidData(invalidData);
      
      // 显示导入结果通知
      if (validCount > 0) {
        Notification.warning({
          title: '导入完成',
          content: `成功导入 ${validCount} 条数据，${invalidCount} 条不合规数据已导出到本地文件`,
          duration: 5000
        });
      } else {
        Notification.error({
          title: '导入失败',
          content: `所有数据都不合规，共 ${invalidCount} 条数据已导出到本地文件`,
          duration: 5000
        });
      }
    } else {
      // 全部数据合规
      Notification.success({
        title: '导入成功',
        content: `成功导入 ${validCount} 条数据`,
        duration: 3000
      });
    }
  };
  reader.onloadend = function () {
    tableLoading.value = false;
    // 关闭全页面loading
    pageLoading.value = false;
    // 文件读取完成
  };
  if (uploadInputRef.value) {
    uploadInputRef.value.value = '';
  }
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
  const chainResult = chainOptions.value.filter(
    (item) => item.key === chainValue.value
  );

  if (chainResult.length > 0) {
    currentChain.value = chainResult[0];
    currentChain.value.gas_price = "查询中...";
    // 查询gas
    fetchGas();

    // 加载对应链的代币列表
    try {
      const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
      if (isTauri) {
        const tokenList = await invoke("get_coin_list", {
          chainKey: chainValue.value
        });

        coinOptions.value = tokenList.map(token => ({
          key: token.key,
          label: token.label,
          symbol: token.symbol,
          contract_address: token.contract_address,
          decimals: token.decimals,
          coin_type: token.coin_type,
          contract_type: token.contract_type,
          abi: token.abi
        }));

        // 设置默认选中第一个代币
        if (coinOptions.value.length > 0) {
          coinValue.value = coinOptions.value[0].key;
          currentCoin.value = coinOptions.value[0];
        } else {
          coinValue.value = '';
          currentCoin.value = null;
        }
      } else {
        // 浏览器环境下的模拟数据
        coinOptions.value = [
          { key: 'eth', label: 'ETH', symbol: 'ETH', coin_type: 'native', decimals: 18 },
          { key: 'usdt', label: 'USDT', symbol: 'USDT', coin_type: 'token', contract_address: '0x...', decimals: 6 }
        ];
        coinValue.value = coinOptions.value[0].key;
        currentCoin.value = coinOptions.value[0];
      }
    } catch (error) {
      console.error('加载代币列表失败:', error);
      coinOptions.value = [];
      coinValue.value = '';
      currentCoin.value = null;
    }
  } else {
    currentChain.value = null;
    coinOptions.value = [];
    coinValue.value = '';
    currentCoin.value = null;
  }
}

async function fetchGas() {
  const temp = chainValue.value;

  // 检查 currentChain 是否为 null
  if (!currentChain.value) {
    return;
  }

  if (temp === "sol") {
    currentChain.value.gas_price = "";
    return
  }
  // 获取gas价格
  try {
    const res = await invoke("get_chain_gas_price", { chain: chainValue.value });
    if (temp === chainValue.value && currentChain.value) {
      // 确保 res 是数字类型
      const gasPrice = res?.gas_price_gwei || undefined;
      if (isNaN(gasPrice)) {
        currentChain.value.gas_price = "数据格式错误";
      } else {
        if (chainValue.value === "eth") {
          currentChain.value.gas_price = gasPrice.toFixed(3);
        } else {
          currentChain.value.gas_price = gasPrice.toFixed(7);
        }
      }
    } else {
      // gas price 已失效
    }
  } catch (err) {
    if (currentChain.value) {
      currentChain.value.gas_price = "查询错误";
    }
  }
}

// coin变化事件
async function coinChange(value) {
  currentCoin.value = coinOptions.value.filter((item) => item.key === value)[0];
}

// 打开区块链浏览器
function openBlockchainScan() {
  if (currentChain.value?.scan_url) {
    // 检查是否在Tauri环境中
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      // 在Tauri环境中使用shell打开默认浏览器
      import('@tauri-apps/plugin-shell').then(({ open }) => {
        open(currentChain.value.scan_url);
      }).catch(error => {
        console.error('打开浏览器失败:', error);
        Notification.error('打开浏览器失败');
      });
    } else {
      // 在浏览器环境中直接打开新窗口
      window.open(currentChain.value.scan_url, '_blank');
    }
  }
}

// 添加代币弹窗取消
function handleAddCoinCancel() {
  addCoinVisible.value = false;
}

// 添加代币核心方法
async function addCoinFunc() {
  try {
    // 使用Rust后端获取代币信息
    const tokenInfo = await invoke("get_token_info", {
      chain: chainValue.value,
      contractAddress: coinAddress.value
    });

    let json = {
      key: tokenInfo.symbol.toLowerCase(),
      coin: tokenInfo.symbol,
      type: "token",
      contract_type: "",
      contract_address: coinAddress.value,
      abi: tokenInfo.abi,
    };

    // 添加代币
    await invoke("add_coin", {
      chain: chainValue.value,
      objJson: JSON.stringify(json),
    });

    addCoinVisible.value = false;
    coinAddress.value = "";
    return Promise.resolve();
  } catch (err) {
    return Promise.reject(err.toString() || "添加代币失败！");
  }
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
  // 重置文件输入的value，确保再次选择相同文件时能触发change事件
  if (uploadInputRef.value) {
    uploadInputRef.value.value = '';
  }
  // 重置页面loading状态
  pageLoading.value = false;
  Notification.success("清空列表成功！");
}

// 清空私钥
function clearPrivateKey() {
  data.value.forEach((item) => {
    item.private_key = "";
  });
  // 删除私钥和地址都为空的行
  data.value = data.value.filter((item) => {
    return item.private_key.trim() !== "" || item.to_addr.trim() !== "";
  });
  Notification.success("清空私钥成功！");
}

// 清空地址
function clearAddress() {
  data.value.forEach((item) => {
    item.to_addr = "";
  });
  // 删除私钥和地址都为空的行
  data.value = data.value.filter((item) => {
    return item.private_key.trim() !== "" || item.to_addr.trim() !== "";
  });
  Notification.success("清空地址成功！");
}

// 导入事件触发
function handleClick() {
  if (walletImportRef.value) {
    walletImportRef.value.show();
  }
}

// 导入弹窗关闭事件
function handleCancel() {
  handleWalletImportCancel();
}



// 处理钱包导入确认事件
function handleWalletImportConfirm(importData) {
  const { privateKeys, addresses } = importData;
  
  const newData = [];
  let successCount = 0;
  let failCount = 0;

  for (let i = 0; i < privateKeys.length; i++) {
    const privateKey = privateKeys[i];
    const toAddress = addresses[i];

    try {
      // 从私钥生成发送方地址
      const wallet = new ethers.Wallet(privateKey);
      const fromAddress = wallet.address;

      newData.push({
        key: privateKey + toAddress,
        private_key: privateKey,
        address: fromAddress,
        to_addr: toAddress,
        amount: "",
        plat_balance: "",
        coin_balance: "",
        exec_status: "0",
        error_msg: "",
      });
      successCount++;
    } catch (error) {
      console.error('处理数据失败:', error);
      failCount++;
    }
  }

  // 添加到表格数据
  data.value.push(...newData);

  // 计算重复数据信息
  const uniqueKeys = new Set(privateKeys);
  const duplicateKeysCount = privateKeys.length - uniqueKeys.size;
  const uniqueAddresses = new Set(addresses);
  const duplicateAddressesCount = addresses.length - uniqueAddresses.size;

  // 显示结果通知
  const totalCount = privateKeys.length;
  let notificationContent = `成功导入${successCount}条数据`;

  // 添加重复数据提示
  if (duplicateKeysCount > 0 || duplicateAddressesCount > 0) {
    const duplicateInfo = [];
    if (duplicateKeysCount > 0) duplicateInfo.push(`${duplicateKeysCount}个重复私钥`);
    if (duplicateAddressesCount > 0) duplicateInfo.push(`${duplicateAddressesCount}个重复地址`);
    notificationContent += `（包含${duplicateInfo.join('、')}）`;
  }

  if (failCount > 0) {
    Notification.warning({
      title: "导入完成！",
      content: `总计${totalCount}条，成功${successCount}条，失败${failCount}条（格式错误）。${duplicateKeysCount > 0 || duplicateAddressesCount > 0 ? '注意：已允许重复数据导入。' : ''}`,
    });
  } else {
    Notification.success({
      title: "导入成功！",
      content: notificationContent,
    });
  }
  
  // 弹窗关闭现在由组件内部管理
}

// 处理钱包导入取消事件
function handleWalletImportCancel() {
  // 弹窗关闭现在由组件内部管理
}

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
  // 数据长度记录
  data.value = data.value.filter((obj) => currentItemKey.value !== obj.key);
  // 数据长度记录
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

    // 开始查询地址余额

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

      let result;
      const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
      if (isTauri) {
        result = await invoke('query_balances_with_updates', { params });
      } else {
        // 浏览器环境下的模拟数据
        result = {
          success: true,
          items: data.value.map(item => ({
            ...item,
            plat_balance: '1.0',
            coin_balance: '100.0',
            nonce: 1,
            exec_status: '2',
            error_msg: null
          }))
        };
      }

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

        // 查询完成统计

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


// 删除代币确认
async function deleteTokenConfirm() {
  deleteTokenVisible.value = false;
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
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
  } else {
    // 浏览器环境下模拟成功
    Notification.success("删除成功！");
    chainChange();
  }
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
      // 验证通过
      startLoading.value = true;
      stopFlag.value = false;
      stopStatus.value = false;

      // 初始化进度条
      transferTotal.value = data.value.length;
      transferCompleted.value = 0;
      transferProgress.value = 0;
      showProgress.value = true;

      data.value.forEach((item) => {
        item.exec_status = "0";
        item.error_msg = "";
        item.retry_flag = false;
        item.error_count = 0;
      });
      await transferFnc(data.value);
    })
    .catch(() => {
      // 验证失败
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
        delay: [form.min_interval && form.min_interval.trim() !== '' ? Number(form.min_interval) : 1, form.max_interval && form.max_interval.trim() !== '' ? Number(form.max_interval) : 3], // 延迟时间
        transfer_type: form.send_type, // 转账类型 1：全部转账 2:转账固定数量 3：转账随机数量  4：剩余随机数量
        transfer_amount: form.amount_from === '1' ? (accountData[i].amount && accountData[i].amount.trim() !== '' ? Number(accountData[i].amount) : 0) : (form.send_count && form.send_count.trim() !== '' ? Number(form.send_count) : 0), // 转账当前指定的固定金额
        transfer_amount_list: [form.send_min_count && form.send_min_count.trim() !== '' ? Number(form.send_min_count) : 0, form.send_max_count && form.send_max_count.trim() !== '' ? Number(form.send_max_count) : 0], // 转账数量 (transfer_type 为 1 时生效) 转账数量在5-10之间随机，第二个数要大于第一个数！！
        left_amount_list: [form.send_min_count && form.send_min_count.trim() !== '' ? Number(form.send_min_count) : 0, form.send_max_count && form.send_max_count.trim() !== '' ? Number(form.send_max_count) : 0], // 剩余数量 (transfer_type 为 2 时生效) 剩余数量在4-6之间随机，第二个数要大于第一个数！！
        amount_precision: form.amount_precision && form.amount_precision.trim() !== '' ? Number(form.amount_precision) : 6, // 一般无需修改，转账个数的精确度 6 代表个数有6位小数
        limit_type: form.limit_type, // limit_type 限制类型 1：自动 2：指定数量 3：范围随机
        limit_count: form.limit_count && form.limit_count.trim() !== '' ? Number(form.limit_count) : 21000, // limit_count 指定数量 (limit_type 为 2 时生效)
        limit_count_list: [form.limit_min_count && form.limit_min_count.trim() !== '' ? Number(form.limit_min_count) : 21000, form.limit_max_count && form.limit_max_count.trim() !== '' ? Number(form.limit_max_count) : 30000],
        gas_price_type: form.gas_price_type, // gas price类型 1: 自动 2：固定gas price 3：gas price溢价率
        gas_price_rate: form.gas_price_rate && form.gas_price_rate.trim() !== '' ? Number(form.gas_price_rate) / 100 : 0.05, // gas price溢价率，0.05代表gas price是当前gas price的105%
        gas_price: form.gas_price && form.gas_price.trim() !== '' ? Number(form.gas_price) : 30, // 设置最大的gas price，单位gwei
        max_gas_price: form.max_gas_price && form.max_gas_price.trim() !== '' ? Number(form.max_gas_price) : 0, // 设置最大的gas price，单位gwei
      };
      if (currentCoin.value.coin_type === "base") {
        if (stopFlag.value) {
          stopStatus.value = true;
          return;
        }
        // 设置状态 为执行中
        accountData[i].exec_status = "1";
        try {
          console.log("config:", config);
          const res = await invoke("base_coin_transfer", {
            index: i + 1,
            item: accountData[i],
            config: config
          });
          console.log("base_coin_transfer 返回信息:", res);
          accountData[i].exec_status = "2";
          // 转账成功时只显示tx_hash
          if (typeof res === 'object' && res !== null) {
            if (res.success && res.tx_hash) {
              accountData[i].error_msg = res.tx_hash;
            } else {
              accountData[i].error_msg = res.error || '转账失败';
            }
          } else {
            accountData[i].error_msg = String(res || '转账成功');
          }
          // 更新进度条
          updateTransferProgress();
        } catch (err) {
          if (err === "base gas price 超出最大值限制") {
            Notification.error("base gas price 超出最大值限制");
            // 停止
            stopTransfer();
            accountData[i].exec_status = "0";
            accountData[i].error_msg = "";
          } else {
            accountData[i].exec_status = "3";
            accountData[i].error_msg = err;
            // 更新进度条
            updateTransferProgress();
          }
        }
      } else if (currentCoin.value.coin_type === "token") {
        if (stopFlag.value) {
          stopStatus.value = true;
          return;
        }
        // 设置状态 为执行中
        accountData[i].exec_status = "1";
        try {
          const res = await invoke("token_transfer", {
            index: i + 1,
            item: accountData[i],
            config: {
              ...config,
              contract_address: contract.address,
              abi: contract.abi
            }
          });
          console.log("token_transfer 返回信息:", res);
          accountData[i].exec_status = "2";
          // 转账成功时只显示tx_hash
          if (typeof res === 'object' && res !== null) {
            if (res.success && res.tx_hash) {
              accountData[i].error_msg = res.tx_hash;
            } else {
              accountData[i].error_msg = res.error || '转账失败';
            }
          } else {
            accountData[i].error_msg = String(res || '转账成功');
          }
          // 更新进度条
          updateTransferProgress();
        } catch (err) {
          if (err === "base gas price 超出最大值限制") {
            Notification.error("base gas price 超出最大值限制");
            // 停止
            stopTransfer();
            accountData[i].exec_status = "0";
            accountData[i].error_msg = "";
          } else {
            accountData[i].exec_status = "3";
            accountData[i].error_msg = err;
            // 更新进度条
            updateTransferProgress();
          }
        }
      } else {
        Notification.error("未知币种类型");
        // 未知币种类型
        return;
      }
    } catch (e) {
      // 交易失败
      // 错误信息
    }
  }
}

// 停止执行
function stopTransfer() {
  startLoading.value = false;
  stopFlag.value = true;
  // 隐藏进度条
  setTimeout(() => {
    showProgress.value = false;
  }, 1000);
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
    const bool = /^\d+(\.\d+)?$/.test(form.send_count) && Number(form.send_count) > 0;
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
      /^\d+(\.\d+)?$/.test(form.send_min_count) &&
      /^\d+(\.\d+)?$/.test(form.send_max_count) &&
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
    /^\d+(\.\d+)?$/.test(form.amount_precision) &&
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
    const bool = /^\d+(\.\d+)?$/.test(form.gas_price) && Number(form.gas_price) > 0;
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
    const bool = /^\d+$/.test(form.gas_price_rate) && Number(form.gas_price_rate) > 0;
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
        /^\d+(\.\d+)?$/.test(form.max_gas_price) && Number(form.max_gas_price) > 0;
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
    const bool = /^\d+$/.test(form.limit_count) && Number(form.limit_count) > 0;
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
      /^\d+$/.test(form.limit_min_count) && Number(form.limit_min_count) > 0 &&
      /^\d+$/.test(form.limit_max_count) && Number(form.limit_max_count) > 0;
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
      /^\d+$/.test(form.min_interval) && Number(form.min_interval) >= 0) &&
    (form.max_interval === "0" ||
      /^\d+$/.test(form.max_interval) && Number(form.max_interval) >= 0);
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

// 代币管理相关方法
// 显示代币管理弹窗
function showTokenManage() {
  if (!chainValue.value) {
    Notification.warning("请先选择区块链！");
    return;
  }
  tokenManageRef.value?.show();
}

// 加载代币管理数据
async function loadTokenManageData() {
  tokenTableLoading.value = true;
  try {
    let tokenList;
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      tokenList = await invoke("get_coin_list", {
        chainKey: chainValue.value
      });
    } else {
      // 浏览器环境下的模拟数据
      tokenList = [
        { key: 'eth', coin: 'ETH', type: 'native', decimals: 18 },
        { key: 'usdt', coin: 'USDT', type: 'token', contract_address: '0x...', decimals: 6 }
      ];
    }
    tokenManageData.value = tokenList.map(token => ({
      key: token.key,
      coin: token.symbol || token.coin || token.label, // 使用symbol作为显示的代币符号
      name: token.label || token.coin, // 添加name字段映射
      symbol: token.symbol || token.coin || token.label, // 正确映射symbol字段
      type: token.type || token.coin_type,
      contract_type: token.contract_type || '',
      contract_address: token.contract_address || '',
      abi: token.abi || '',
      decimals: token.decimals || 18,
      label: token.label || token.coin
    }));
  } catch (error) {
    console.error('加载代币数据失败:', error);
    Notification.error('加载代币数据失败：' + error);
  } finally {
    tokenTableLoading.value = false;
  }
}

// 显示添加代币弹窗
function showAddToken() {
  // 设置为添加模式
  isTokenEditMode.value = false;
  currentEditToken.value = null;

  // 重置表单
  Object.assign(tokenForm, {
    key: '',
    name: '',
    symbol: '',
    decimals: 18,
    type: 'token',
    contract_type: '',
    contract_address: '',
    abi: ''
  });
  tokenFormVisible.value = true;
}

// 显示编辑代币弹窗
function showEditToken(record) {
  // 设置为编辑模式
  isTokenEditMode.value = true;
  currentEditToken.value = record;

  // 填充表单数据，确保所有字段都有默认值
  Object.assign(tokenForm, {
    key: record.key || '',
    name: record.name || record.coin || record.label || '',
    symbol: record.symbol || record.coin || record.label || '',
    decimals: record.decimals || 18,
    type: record.type || 'token',
    contract_type: record.contract_type || '',
    contract_address: record.contract_address || '',
    abi: record.abi || ''
  });
  tokenFormVisible.value = true;
}

// 提交代币表单（统一处理添加和编辑）
async function submitTokenForm() {
  try {
    // 验证必填项，确保字段存在且不为空
    if (!tokenForm.name || !tokenForm.name.trim()) {
      Notification.warning('请输入代币名称');
      return false;
    }
    if (!tokenForm.symbol || !tokenForm.symbol.trim()) {
      Notification.warning('请输入代币符号');
      return false;
    }
    if (!tokenForm.key || !tokenForm.key.trim()) {
      Notification.warning('请输入代币标识');
      return false;
    }
    if (tokenForm.type === 'token' && (!tokenForm.contract_address || !tokenForm.contract_address.trim())) {
      Notification.warning('代币类型为token时，合约地址不能为空');
      return false;
    }
    if (tokenForm.type === 'token' && (!tokenForm.abi || !tokenForm.abi.trim())) {
      Notification.warning('代币类型为合约代币时，ABI不能为空');
      return false;
    }
    if (!tokenForm.decimals || tokenForm.decimals < 0) {
      Notification.warning('请输入有效的小数位数');
      return false;
    }

    // 如果是添加模式且没有输入key，自动生成
    if (!isTokenEditMode.value && !tokenForm.key.trim()) {
      tokenForm.key = tokenForm.symbol.toLowerCase();
    }

    const requestData = {
      key: tokenForm.key,
      name: tokenForm.name,
      symbol: tokenForm.symbol,
      coin_type: tokenForm.type,
      contract_address: tokenForm.contract_address,
      decimals: tokenForm.decimals,
      abi: tokenForm.abi
    };

    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      if (isTokenEditMode.value) {
        // 更新代币
        await invoke('update_coin', {
          chain: chainValue.value,
          key: tokenForm.key,
          objJson: JSON.stringify(requestData)
        });
        Notification.success('编辑代币成功！');
      } else {
        // 添加代币
        await invoke('add_coin', {
          chain: chainValue.value,
          objJson: JSON.stringify(requestData)
        });
        Notification.success('添加代币成功！');
      }
    } else {
      // 浏览器环境下模拟成功
      if (isTokenEditMode.value) {
        Notification.success('编辑代币成功！');
      } else {
        Notification.success('添加代币成功！');
      }
    }

    // 刷新代币列表
    loadTokenManageData();

    // 重新加载主页面的代币选择器
    await chainChange();

    tokenFormVisible.value = false;
    return true;
  } catch (error) {
    console.error('代币操作失败:', error);
    Notification.error('代币操作失败：' + error);
    return false;
  }
}

// 删除代币
async function deleteTokenFromManage(tokenKey) {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      await invoke('remove_coin', {
        chain: chainValue.value,
        key: tokenKey
      });
    }

    Notification.success('删除代币成功！');

    // 刷新代币列表
    loadTokenManageData();

    // 重新加载主页面的代币选择器
    await chainChange();
  } catch (error) {
    console.error('删除代币失败:', error);
    Notification.error('删除代币失败：' + error);
  }
}

// 事件处理函数
// 处理链更新事件
async function handleChainUpdated() {
  // 重新加载链数据
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      // 从后端重新获取链列表
      const result = await invoke('get_chain_list');
      chainOptions.value = result || [];

      // 检查当前选中的链是否还存在
      const currentChainExists = chainOptions.value.find(chain => chain.key === chainValue.value);
      if (!currentChainExists && chainOptions.value.length > 0) {
        // 如果当前选中的链不存在了，选择第一个可用的链
        chainValue.value = chainOptions.value[0].key;
        await chainChange();
      } else if (currentChainExists) {
        // 如果当前链仍然存在，更新当前链信息
        currentChain.value = currentChainExists;
      } else {
        // 如果没有可用的链，清空选择
        chainValue.value = '';
        currentChain.value = null;
        coinOptions.value = [];
        coinValue.value = '';
        currentCoin.value = null;
      }

      console.log('链列表已更新');
    }
  } catch (error) {
    console.error('更新链列表失败:', error);
    Notification.error('更新链列表失败');
  }
}

// 处理RPC更新事件
function handleRpcUpdated() {
  // 可以在这里处理RPC更新后的逻辑
  console.log('RPC已更新');
}

// 处理代币更新事件
function handleTokenUpdated() {
  // 重新加载代币选择器
  chainChange();
}

// 显示RPC管理弹窗
function showRpcManage() {
  if (!chainValue.value) {
    Notification.warning("请先选择区块链！");
    return;
  }
  rpcManageRef.value?.show();
}

// 显示链管理弹窗
function showChainManage() {
  if (chainManageRef.value) {
    chainManageRef.value.show();
  }
}
</script>

<template>
  <!-- 标题栏组件 -->
  <TitleBar :title="windowTitle" />

  <div class="container transfer" style="height: 100vh; display: flex; flex-direction: column; overflow: hidden;">
    <!-- <span class="pageTitle">批量转账</span> -->
    <!-- 工具栏 -->
    <div class="toolBar" style="flex-shrink: 0;">
      <a-button type="primary" @click="handleClick()">钱包信息录入
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
      <!-- <a-button type="outline" status="normal" style="float: right; margin-right: 10px" @click="clearPrivateKey">清空私钥
      </a-button>
      <a-button type="outline" status="normal" style="float: right; margin-right: 10px" @click="clearAddress">清空地址
      </a-button> -->
    </div>
    <!-- 操作账号表格 -->
    <div class="mainTable" style="flex: 1; overflow: hidden; display: flex; flex-direction: column; min-height: 0;">
      <a-table v-if="tableBool" row-key="key" :columns="columns" :column-resizable="true" :data="data"
        :row-selection="rowSelection" :loading="tableLoading" :scrollbar="scrollbar" :scroll="{ y: '100%' }"
        @row-click="rowClick" v-model:selectedKeys="selectedKeys" :pagination="pagination" style="height: 100%;">
        <template #index="{ rowIndex }">
          {{ rowIndex + 1 }}
        </template>
        <template #exec_status="{ rowIndex }">
          <a-tag v-if="data[rowIndex].exec_status === '0'" color="#86909c">等待执行
          </a-tag>
          <a-tag v-if="data[rowIndex].exec_status === '1'" color="#ff7d00">执行中
          </a-tag>
          <a-tag v-if="data[rowIndex].exec_status === '2'" color="#00b42a">转账成功
          </a-tag>
          <a-tag v-if="data[rowIndex].exec_status === '3'" color="#f53f3f">转账失败
          </a-tag>
        </template>
        <template #optional="{ record }">
          <icon-delete style="font-size: 16px" @click.prevent="deleteItem(record)" />
        </template>
      </a-table>
    </div>

    <!-- 转账进度条 -->
    <div v-if="showProgress" style="margin-top: 10px; padding: 0 10px; flex-shrink: 0;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
        <span style="font-size: 14px; color: #1d2129; font-weight: 500;">转账进度</span>
        <span style="font-size: 14px; color: #86909c;">{{ transferCompleted }} / {{ transferTotal }}</span>
      </div>
      <a-progress :percent="transferProgress" :show-text="true" :stroke-width="8" :color="{
        '0%': '#00b42a',
        '100%': '#00b42a'
      }" style="width: 100%;" />
    </div>

    <!-- 管理代币按钮嵌入 -->
    <div style="display: flex; gap: 10px; align-items: center; margin-top: 10px; flex-shrink: 0;">
      <!-- 链管理按钮 -->
      <a-button type="primary" @click="showChainManage" style="white-space: nowrap;">
        区块链管理
      </a-button>
      <a-button type="primary" @click="showRpcManage" :disabled="!chainValue" style="white-space: nowrap;">
        RPC管理
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
            <ChainIcon :chain-key="data?.key" :pic-data="data?.pic_data" :alt="data?.name"
              style="width: 18px; height: 18px;" />
            <span style="margin-left: 10px">{{ data?.name }}</span>
            <span style="margin-left: 20px;color: #c3c3c3;">{{ data?.scan_url }}</span>
            <span v-show="chainValue !== 'sol'" style="flex: 1; text-align: end; color: #00b42a">Gas Price: {{
              data?.gas_price ?? "未知" }}</span>
          </div>
        </template>
        <template #option="{ data }">
          <div style="display: flex; flex-direction: row; align-items: center;height: 32px;">
            <ChainIcon :chain-key="data?.key" :pic-data="data?.pic_data" :alt="data?.name"
              style="width: 18px; height: 18px;" />
            <span style="margin-left: 10px">{{ data?.name }}</span>
            <span style="margin-left: 20px;color: #c3c3c3;">{{ data?.scan_url }}</span>
          </div>
        </template>
      </a-select>
      <!-- 区块链浏览器跳转按钮 -->
      <a-tooltip v-if="currentChain?.scan_url" content="在浏览器中打开区块链浏览器">
        <a-button type="primary" @click="openBlockchainScan" shape="round" style="white-space: nowrap; padding: 0 8px;">
          <icon-share-external />
        </a-button>
      </a-tooltip>
      <a-button type="primary" @click="showTokenManage" :disabled="!chainValue" style="white-space: nowrap;">
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
          <a-form-item field="send_type" label="发送方式" style="width: 330px; padding: 5px 10px;">
            <a-radio-group v-model="form.send_type" type="button">
              <a-radio value="1">全部</a-radio>
              <a-radio value="2">指定数量</a-radio>
              <a-radio value="3">范围随机</a-radio>
              <a-radio value="4">剩余数量</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item v-if="form.send_type === '2'" field="amount_from" label="数量来源" tooltip="如果选择表格数据则应导入带有转账数量的表格数据"
            style="width: 190px; padding: 5px 10px;">
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
          <a-form-item field="error_retry" label="失败自动重试" style="width: 125px; padding: 5px 10px;"
            tooltip="转账失败时是否自动重试">
            <a-switch v-model="form.error_retry" checked-value="1" unchecked-value="0" />
          </a-form-item>
        </a-row>
        <a-row v-show="chainValue !== 'sol'" style="height: 70px">
          <a-form-item field="limit_type" label="Gas Limit" style="width: 245px; padding: 5px 10px;">
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
          <a-form-item field="gas_price_type" label="Gas Price 方式" style="width: 230px; padding: 5px 10px;">
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
          <a-form-item v-if="form.gas_price_type === '3'" field="gas_price_rate"
            style="width: 100px; padding: 5px 10px;" label="提高比例">
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
    <div
      style="display: flex; align-items: center; padding: 10px 20px; margin-top: 5px; justify-content: center; gap: 30px; flex-shrink: 0;">
      <!-- 左侧区域 -->
      <div style="display: flex; align-items: center; gap: 20px;">


        <!-- 查询余额 -->
        <a-dropdown>
          <a-button type="primary" :loading="balanceLoading"
            :style="{ width: '130px', height: '40px', fontSize: '14px' }">查询余额</a-button>
          <template #content>
            <a-doption @click="queryBalance">⤴️ 查出账地址</a-doption>
            <a-doption disabled>⤵️ 查到账地址</a-doption>
          </template>
        </a-dropdown>
      </div>

      <!-- 右侧区域 -->
      <div style="display: flex; align-items: center; gap: 20px;">
        <!-- 执行转账按钮 -->
        <a-button v-if="!startLoading && stopStatus" type="success" class="execute-btn"
          @click="startTransfer(data.value)"
          :style="{ width: '130px', height: '40px', fontSize: '14px' }">执行转账</a-button>
        <a-tooltip v-else content="点击可以提前停止执行">
          <div @click="stopTransfer">
            <a-button v-if="!stopFlag" class="execute-btn executing" loading
              :style="{ width: '130px', height: '40px', fontSize: '14px' }">执行中...</a-button>
            <a-button v-if="stopFlag && !stopStatus" class="execute-btn stopping" loading
              :style="{ width: '130px', height: '40px', fontSize: '14px' }">正在停止...</a-button>
          </div>
        </a-tooltip>
      </div>
    </div>
  </div>
  <!-- 钱包信息录入弹窗 -->
  <WalletImportModal 
    ref="walletImportRef"
    @confirm="handleWalletImportConfirm"
    @cancel="handleWalletImportCancel" />
  <!-- 添加代币弹窗 -->
  <a-modal v-model:visible="addCoinVisible" :width="700" title="添加代币" @cancel="handleAddCoinCancel"
    :on-before-ok="handleAddCoinBeforeOk" unmountOnClose>
    <a-input v-model="coinAddress" placeholder="请输入代币合约地址" allow-clear />
  </a-modal>
  <!-- 删除代币确认框 -->
  <a-modal v-model:visible="deleteTokenVisible" title="删除确认">
    <div>确认删除【 {{ currentCoin?.coin || '未知' }} 】代币？</div>
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

  <!-- 链管理组件 -->
  <ChainManagement ref="chainManageRef" @chain-updated="handleChainUpdated" />

  <!-- 代币管理组件 -->
  <TokenManagement ref="tokenManageRef" :chain-value="chainValue" :chain-options="chainOptions"
    @token-updated="handleTokenUpdated" />

  <!-- RPC管理组件 -->
  <RpcManagement ref="rpcManageRef" :chain-value="chainValue" :chain-options="chainOptions"
    @rpc-updated="handleRpcUpdated" />

  <!-- 全页面Loading覆盖层 -->
  <div v-if="pageLoading" class="page-loading-overlay">
    <div class="loading-content">
      <a-spin size="large" />
      <div class="loading-text">正在导入文件，请稍候...</div>
    </div>
  </div>

</template>

<style scoped>
.container {
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
  height: calc(100vh - 200px);
  min-height: 400px;
  display: flex;
  flex-direction: column;
}

.mainTable .arco-table {
  flex: 1;
  height: 100%;
}

.mainTable .arco-table-container {
  height: 100%;
}

.mainTable .arco-table-content {
  height: 100%;
}

.mainTable .arco-empty {
  height: 100%;
  min-height: 400px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

.mainTable .arco-table-cell {
  height: 100%;
}

.mainTable .arco-table-td-content {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 确保表格列宽度和省略号生效 */
.mainTable .arco-table-td {
  max-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mainTable .arco-table-cell {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 强制表格布局为固定布局 */
.mainTable .arco-table-element {
  table-layout: fixed;
  width: 100%;
}

.mainTable .arco-table-tbody {
  height: 100%;
}

.mainTable .arco-table-tr {
  height: 100%;
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

/* 全页面Loading覆盖层样式 */
.page-loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: var(--overlay-bg, rgba(0, 0, 0, 0.5));
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 9999;
}

.loading-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  background-color: var(--card-bg, white);
  padding: 32px;
  border-radius: 8px;
  box-shadow: 0 4px 12px var(--shadow-color, rgba(0, 0, 0, 0.15));
  border: 1px solid var(--border-color, transparent);
}

.loading-text {
  font-size: 16px;
  color: var(--text-color, #1d2129);
  font-weight: 500;
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
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(30, 58, 138, 0.3);
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

.title-bar-control.theme-toggle {
  width: 40px;
  margin-right: 5px;
}

.title-bar-control.theme-toggle:hover {
  background-color: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
}

.theme-icon {
  font-size: 16px;
  transition: transform 0.3s ease;
}

.title-bar-control.theme-toggle:hover .theme-icon {
  transform: scale(1.1);
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
