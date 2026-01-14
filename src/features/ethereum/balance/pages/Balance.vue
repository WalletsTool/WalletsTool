<script setup name="balance">
import { Icon } from '@iconify/vue';
import { IconDelete } from '@arco-design/web-vue/es/icon';
import { computed, defineAsyncComponent, nextTick, onBeforeMount, onMounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ethers } from "ethers";
import { Notification, Modal } from "@arco-design/web-vue";
import { utils as xlUtils, writeFile } from "xlsx";
import { getCurrentWindow } from '@tauri-apps/api/window'
import ChainIcon from '@/components/ChainIcon.vue'
import TitleBar from '@/components/TitleBar.vue'
import TableSkeleton from '@/components/TableSkeleton.vue'
import VirtualScrollerTable from '@/components/VirtualScrollerTable.vue'
import { debounce } from '@/utils/debounce.js'
import { WINDOW_CONFIG } from '@/utils/windowNames'

// 懒加载非关键组件
const ChainManagement = defineAsyncComponent(() => import('@/components/ChainManagement.vue'))
const RpcManagement = defineAsyncComponent(() => import('@/components/RpcManagement.vue'))
const TokenManagement = defineAsyncComponent(() => import('@/components/TokenManagement.vue'))
const CodeEditor = defineAsyncComponent(() => import('@/components/CodeEditor.vue'))
const ProxyConfigModal = defineAsyncComponent(() => import('@/components/ProxyConfigModal.vue'))

// 组件配置参数（props）：是否查询最后交易时间，默认 false
const props = defineProps({
  queryLastTransactionTime: { type: Boolean, default: false }
})

// table列名
const columns = [
  {
    title: '序号',
    align: 'center',
    width: 55,
    slotName: 'index'
  },
  {
    title: '钱包地址',
    align: 'center',
    width: 390,
    dataIndex: 'address',
    ellipsis: true,
    tooltip: true
  },
  {
    title: 'Nonce',
    align: 'center',
    width: 65,
    dataIndex: 'nonce',
    ellipsis: true,
    tooltip: true
  },
  {
    title: '平台币余额',
    align: 'center',
    dataIndex: 'plat_balance',
    width: 95,
    ellipsis: true,
    tooltip: true
  },
  {
    title: '代币余额',
    align: 'center',
    dataIndex: 'coin_balance',
    width: 85,
    ellipsis: true,
    tooltip: true
  },
  {
    title: '最后交易时间',
    align: 'center',
    dataIndex: 'last_transaction_time',
    width: 120,
    ellipsis: true,
    tooltip: true,
    slotName: 'last_transaction_time'
  },
  {
    title: '状态',
    align: 'center',
    slotName: 'exec_status',
    width: 90,
    ellipsis: true,
    tooltip: true
  },
  {
    title: '错误信息',
    align: 'center',
    dataIndex: 'error_msg',
    ellipsis: true,
    tooltip: true,
  },
  {
    title: '操作',
    align: 'center',
    slotName: 'optional',
    width: 55,
    ellipsis: true,
    tooltip: true
  }
]
const tableBool = ref(true)
const data = ref([])
// 选中的数据key
const selectedKeys = ref([]);

// 点击行实现选中和取消
function rowClick(record, event) {
  const index = selectedKeys.value.indexOf(record.address)
  index >= 0 ? selectedKeys.value.splice(index, 1) : selectedKeys.value.push(record.address)
}

// 仅查询目标代币
const onlyCoin = ref(true);
// 余额查询进度相关变量
const balanceProgress = ref(0); // 余额查询进度百分比
const balanceTotal = ref(0); // 总查询数量
const balanceCompleted = ref(0); // 已完成查询数量
const showProgress = ref(false); // 是否显示进度条
// 分页
const pagination = ref(false);
const scrollbar = ref(true);

// 窗口标题定义
const windowTitle = ref('余额查询');

// 窗口标题初始化
function initBalanceWindowTitle() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
    if (isTauri) {
      const windowLabel = getCurrentWindow().label
      const saved = WINDOW_CONFIG.getCustomTitle(windowLabel)
      if (saved) {
        windowTitle.value = saved
        return
      }
    }
  } catch (e) {
    console.error('初始化余额查询窗口标题失败:', e)
  }
  
  // 不再设置默认标题，由后端或调用方设置
}

initBalanceWindowTitle()

// chain默认值
const chainValue = ref('');
// 当前chain
const currentChain = ref({});
// chain自定义字段名
const chainFieldNames = { value: 'key', label: 'scan_url' }
// 主网选择器
let chainOptions = ref([])
// coin默认值
let coinValue = ref('');
// coin自定义字段名
const coinFieldNames = { value: 'key', label: 'name' }
// 币种选择器
const coinOptions = ref([]);
// 查询余额按钮loading
let balanceLoading = ref(false)
// 余额查询停止标志
let balanceStopFlag = ref(false)
// 详细配置
const form = reactive({
  thread_count: 3,
  // 是否查询最后交易时间（默认关闭），可由组件外部通过 props 控制初始值
  queryLastTransactionTime: props.queryLastTransactionTime
})
// 录入 钱包地址 弹窗
let visible = ref(false)
let importText = ref('')
// 导入loading状态
let importLoading = ref(false)
// 地址验证相关
const validationErrors = ref([])
const errorsExpanded = ref(false)
const addressErrorLines = ref([])
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
// 当前窗口ID
let currentWindowId = ref('')
// 链管理组件引用
const chainManageRef = ref(null);
// RPC管理组件引用
const rpcManageRef = ref(null);
// 代币管理组件引用
const tokenManageRef = ref(null);
// 代理配置组件引用
const proxyConfigRef = ref(null);
// 文件上传输入框引用
const uploadInputRef = ref(null);
const proxyConfigVisible = ref(false);
const proxyEnabled = ref(false);
const proxyStatus = ref('未配置');
const proxyCount = ref(0);
// 高级筛选相关变量
const advancedFilterVisible = ref(false);
const filterForm = reactive({
  platBalanceOperator: 'gt', // gt: 大于, eq: 等于, lt: 小于
  platBalanceValue: '',
  coinBalanceOperator: 'gt', // gt: 大于, eq: 等于, lt: 小于
  coinBalanceValue: '',
  nonceOperator: 'gt',
  nonceValue: '',
  errorMsg: ''
});

// 计算属性：缓存筛选后的数据
const filteredData = computed(() => {
  if (!filterForm.platBalanceValue && !filterForm.coinBalanceValue && 
      !filterForm.nonceValue && !filterForm.errorMsg) {
    return data.value;
  }
  
  return data.value.filter(item => {
    // 平台币余额筛选
    if (filterForm.platBalanceValue && filterForm.platBalanceValue.trim() !== '') {
      const platBalanceValue = parseFloat(filterForm.platBalanceValue);
      const itemPlatBalance = parseFloat(item.plat_balance || 0);
      
      if (filterForm.platBalanceOperator === 'gt' && itemPlatBalance <= platBalanceValue) {
        return false;
      } else if (filterForm.platBalanceOperator === 'eq' && itemPlatBalance !== platBalanceValue) {
        return false;
      } else if (filterForm.platBalanceOperator === 'lt' && itemPlatBalance >= platBalanceValue) {
        return false;
      }
    }
    
    // 代币余额筛选
    if (filterForm.coinBalanceValue && filterForm.coinBalanceValue.trim() !== '') {
      const coinBalanceValue = parseFloat(filterForm.coinBalanceValue);
      const itemCoinBalance = parseFloat(item.coin_balance || 0);
      
      if (filterForm.coinBalanceOperator === 'gt' && itemCoinBalance <= coinBalanceValue) {
        return false;
      } else if (filterForm.coinBalanceOperator === 'eq' && itemCoinBalance !== coinBalanceValue) {
        return false;
      } else if (filterForm.coinBalanceOperator === 'lt' && itemCoinBalance >= coinBalanceValue) {
        return false;
      }
    }
    
    // Nonce值筛选
    if (filterForm.nonceValue && filterForm.nonceValue.trim() !== '') {
      const nonceValue = parseInt(filterForm.nonceValue);
      const itemNonce = parseInt(item.nonce || 0);
      
      if (filterForm.nonceOperator === 'gt' && itemNonce <= nonceValue) {
        return false;
      } else if (filterForm.nonceOperator === 'eq' && itemNonce !== nonceValue) {
        return false;
      } else if (filterForm.nonceOperator === 'lt' && itemNonce >= nonceValue) {
        return false;
      }
    }
    
    // 错误信息模糊匹配
    if (filterForm.errorMsg && filterForm.errorMsg.trim() !== '') {
      const errorMsg = item.error_msg || '';
      if (!errorMsg.toLowerCase().includes(filterForm.errorMsg.toLowerCase())) {
        return false;
      }
    }
    
    return true;
  });
});

// 计算属性：缓存统计数据
const statisticsData = computed(() => {
  const total = data.value.length;
  const succeeded = data.value.filter(item => item.exec_status === '2').length;
  const failed = data.value.filter(item => item.exec_status === '3').length;
  const pending = data.value.filter(item => item.exec_status === '0' || item.exec_status === '1').length;
  
  return { total, succeeded, failed, pending };
});

// 防抖的筛选更新函数
const debouncedFilterUpdate = debounce(() => {
  // 触发筛选数据的重新计算
  // filteredData computed属性会自动响应filterForm的变化
}, 300);

// 时间格式化函数
function formatTransactionTime(timestamp) {
  if (!timestamp || timestamp === 0) {
    return '暂无交易';
  }
  
  try {
    // 将Unix时间戳转换为毫秒
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    
    if (diffMinutes < 60) {
      return `${diffMinutes}分钟前`;
    } else if (diffHours < 24) {
      return `${diffHours}小时前`;
    } else if (diffDays < 30) {
      return `${diffDays}天前`;
    } else {
      // 超过30天显示具体日期
      return date.toLocaleDateString('zh-CN');
    }
  } catch (error) {
    console.error('时间格式化错误:', error);
    return '时间错误';
  }
}

// a-table行选择配置
const rowSelection = reactive({
  type: 'checkbox',
  showCheckedAll: true,
  selectedRowKeys: selectedKeys,
  onSelect: (rowKeys) => {
    selectedKeys.value = rowKeys
  },
  onSelectAll: (selected) => {
    if (selected) {
      selectedKeys.value = data.value.map(item => item.address)
    } else {
      selectedKeys.value = []
    }
  }
})

// 初始化Chain列表
onBeforeMount(async () => {
  chainOptions.value = await invoke('get_chain_list')
  if (chainOptions.value && chainOptions.value.length > 0) {
    chainValue.value = chainOptions.value[0].key
    currentChain.value = chainOptions.value[0]
    // 获取chain对应的代币列表
    await chainChange()
  }
})

onMounted(async () => {
  // 获取窗口标题和ID
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow();
      
      // 获取窗口标题
      const title = await currentWindow.title();
      if (title) {
        windowTitle.value = title;
      }
      
      // 获取当前窗口ID
      currentWindowId.value = currentWindow.label;
      console.log('当前窗口ID:', currentWindowId.value);

      // 添加Tauri窗口关闭事件监听器
      await currentWindow.onCloseRequested(async (event) => {
        console.log('窗口关闭事件触发，正在停止后台操作...');

        // 停止余额查询操作
        if (balanceLoading.value) {
          await stopBalanceQuery();
          console.log('已停止余额查询操作');
        }

        console.log('窗口关闭清理完成，所有后台操作已停止');
      });
    } catch (error) {
      console.error('Error getting window info:', error);
    }
  } else {
    // 浏览器环境下设置默认标题和ID
    windowTitle.value = '余额查询';
    currentWindowId.value = 'browser_window';
  }

  // 页面高度现在通过 CSS 自动调整，无需监听器

  // 监听余额查询更新事件
  await listen('balance_item_update', (event) => {
    const { index, item, window_id } = event.payload
    // 检查是否是本窗口的事件
    if (window_id && window_id !== currentWindowId.value) {
      return; // 不是本窗口的事件，直接返回
    }
    // 使用address查找对应的数据项，而不是使用index
    const targetIndex = data.value.findIndex(dataItem => dataItem.address === item.address)
    if (targetIndex !== -1) {
      // 更新对应地址的数据
      Object.assign(data.value[targetIndex], item)

      // 更新进度
      updateBalanceProgress()
    }
  })

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

  // 初始化代理状态
  await initProxyStatus();
})


// RPC变化事件
async function chainChange() {
  coinOptions.value = await invoke("get_coin_list", { chainKey: chainValue.value })
  if (coinOptions.value && coinOptions.value.length > 0) {
    coinValue.value = coinOptions.value[0].key
    currentCoin.value = coinOptions.value[0]
  }
  currentChain.value = chainOptions.value.find(item => item.key === chainValue.value) || {}
}

// coin变化事件
async function coinChange(value) {
  currentCoin.value = coinOptions.value.filter(item => item.key === value)[0]
}

// 删除代币方法
function deleteToken() {
  if (chainValue.value === 'starknet') {
    Notification.warning({ content: ' StarkNet 暂不支持删除代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'okt') {
    Notification.warning({ content: ' OKT Chain 暂不支持删除代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === "evmos") {
    Notification.warning({ content: " EVMOS Chain 暂不支持删除代币！", position: 'topLeft' });
    return;
  }
  if (chainValue.value === 'geth') {
    Notification.warning({ content: ' Goerli Ethereum 暂不支持删除代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'sepolia') {
    Notification.warning({ content: ' Sepolia Ethereum 暂不支持删除代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'scroll') {
    Notification.warning({ content: ' Scroll Alpha TestNet 暂不支持删除代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'linea') {
    Notification.warning({ content: ' Linea MainNet 暂不支持删除代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'base') {
    Notification.warning({ content: ' Base MainNet 暂不支持删除代币！', position: 'topLeft' });
    return
  }
  deleteTokenVisible.value = true
}

// 删除代币取消
function deleteTokenCancel() {
  deleteTokenVisible.value = false
}

// 删除代币确认
async function deleteTokenConfirm() {
  deleteTokenVisible.value = false
  await invoke("remove_coin", { chain: chainValue.value, key: currentCoin.value.key }).then(() => {
    Notification.success({ content: '删除成功！', position: 'topLeft' });
    // 删除成功后重新获取代币列表
    chainChange()
  }).catch(() => {
    Notification.error({ content: '删除失败！', position: 'topLeft' });
  })
}

// 导入事件触发
function handleAddCoinClick() {
  if (chainValue.value === 'starknet') {
    Notification.warning({ content: ' StarkNet 暂不支持添加代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'okt') {
    Notification.warning({ content: ' OKT Chain 暂不支持添加代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === "evmos") {
    Notification.warning({ content: " EVMOS Chain 暂不支持添加代币！", position: 'topLeft' });
    return;
  }
  if (chainValue.value === 'geth') {
    Notification.warning({ content: ' Goerli Ethereum 暂不支持添加代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'sepolia') {
    Notification.warning({ content: ' Sepolia Ethereum 暂不支持添加代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'scroll') {
    Notification.warning({ content: ' Scroll Alpha TestNet 暂不支持添加代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'linea') {
    Notification.warning({ content: ' Linea MainNet 暂不支持添加代币！', position: 'topLeft' });
    return
  }
  if (chainValue.value === 'base') {
    Notification.warning({ content: ' Base MainNet 暂不支持添加代币！', position: 'topLeft' });
    return
  }
  addCoinVisible.value = true
}

// 添加代币弹窗取消
function handleAddCoinCancel() {
  addCoinVisible.value = false
}

// 添加代币核心方法 - 使用Rust后端
function addCoinFunc() {
  return new Promise(async (resolve, reject) => {
    try {
      // 直接使用Rust后端获取代币信息
      const tokenInfo = await invoke('get_token_info', {
        chain: chainValue.value,
        contractAddress: coinAddress.value
      })

      let json = {
        "key": tokenInfo.symbol.toLowerCase(),
        "coin": tokenInfo.symbol,
        "type": "token",
        "contract_type": "",
        "contract_address": coinAddress.value,
        "abi": null // 使用标准ERC20 ABI
      }

      // 添加代币
      await invoke('add_coin', {
        chain: chainValue.value,
        objJson: JSON.stringify(json)
      })

      addCoinVisible.value = false
      coinAddress.value = ''
      resolve()
    } catch (err) {
      reject('添加代币失败：' + err)
    }
  })
}

// 添加代币弹窗确认
const handleAddCoinBeforeOk = async () => {
  coinAddress.value = coinAddress.value.trim()
  if (!coinAddress.value) {
    Notification.warning({ content: '请输入代币地址！', position: 'topLeft' });
    return false
  }
  let flag = false
  await addCoinFunc().then(() => {
    Notification.success({ content: '添加代币成功！', position: 'topLeft' });
    flag = true
  }).catch(err => {
    Notification.error(err);
  })
  // 删除成功后重新获取代币列表
  chainChange()
  return flag
}

// 清空列表
function clearData() {
  if (balanceLoading.value) {
    Notification.warning({ content: '请停止或等待查询完成后再清空列表！', position: 'topLeft' });
    return;
  }
   if(data.value.length === 0){
    Notification.warning({ content: '当前列表无数据！', position: 'topLeft' });
    return;
  }
  Modal.confirm({
    title: '确认清空',
    content: '确定要清空所有列表数据吗？此操作不可撤销。',
    onOk: () => {
      data.value = [];
      Notification.success({ content: '清空列表成功！', position: 'topLeft' });
    }
  });
}

// 导入事件触发
function handleClick() {
  visible.value = true
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
    return ethers.isAddress(trimmedAddress);
  } catch (error) {
    return false;
  }
}

// 验证导入的地址数据
function validateImportData() {
  const addresses = importText.value.split('\n').filter(line => line.trim() !== '');
  
  validationErrors.value = [];
  const errorLines = new Set();
  
  // 验证地址格式
  addresses.forEach((addr, index) => {
    const trimmedAddr = addr.trim();
    if (trimmedAddr && !validateAddress(trimmedAddr)) {
      validationErrors.value.push(`第${index + 1}行地址格式错误`);
      errorLines.add(index + 1);
    }
  });
  
  // 更新错误行号
  addressErrorLines.value = Array.from(errorLines);
}

// 切换错误信息展开状态
function toggleErrorsExpanded() {
  errorsExpanded.value = !errorsExpanded.value;
}

// 计算显示的错误信息
const displayedErrors = computed(() => {
  if (errorsExpanded.value || validationErrors.value.length <= 3) {
    return validationErrors.value;
  }
  return validationErrors.value.slice(0, 3);
});

// 导入弹窗关闭事件
function handleCancel() {
  // 如果正在导入，不允许关闭
  if (importLoading.value) {
    Notification.warning({ content: '正在导入数据，请稍候...', position: 'topLeft' });
    return false;
  }
  
  visible.value = false
  importText.value = ''
  // 重置验证状态
  validationErrors.value = []
  errorsExpanded.value = false
  addressErrorLines.value = []
}

// 手动录入钱包
function handleManualImport() {
  visible.value = true;
}

// 上传文件导入
function handleFileUpload() {
  uploadInputRef.value.click();
}

// 下载模板
function downloadTemplate() {
  let a = document.createElement("a");
  a.href = `/template/import_model.xlsx`;
  a.download = "导入模板.xlsx";
  a.click();
}

// 处理文件变化

// 导入弹窗保存事件
const handleBeforeOk = async () => {
  // 验证数据
  validateImportData();
  
  if (validationErrors.value.length > 0) {
    return false;
  }
  
  // 开始loading
  importLoading.value = true;
  
  try {
    // 模拟处理延迟，特别是对于大量数据
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // 第一步：获取所有非空地址
    let importList = importText.value.split('\n').filter(item => item.trim() !== '')
    const original_count = importList.length
    
    // 第二步：去除导入文本中的重复地址（保持原始顺序，去除后面的重复项）
    const uniqueAddresses = new Set()
    importList = importList.filter(item => {
      const trimmedAddr = item.trim()
      if (uniqueAddresses.has(trimmedAddr)) {
        return false // 重复地址，过滤掉
      }
      uniqueAddresses.add(trimmedAddr)
      return true
    })
    const after_dedup_count = importList.length
    const internal_dup_count = original_count - after_dedup_count
    
    // 第三步：过滤与现有数据重复的地址
    const beforeFilterCount = importList.length
    importList = importList.filter(item => data.value.length === 0 || !data.value.find(obj => obj.address === item.trim()))
    const success_count = importList.length
    const existing_dup_count = beforeFilterCount - success_count
    const total_filtered_count = original_count - success_count
    
    // 批量处理数据，避免UI阻塞
    const batchSize = 100;
    for (let i = 0; i < importList.length; i += batchSize) {
      const batch = importList.slice(i, i + batchSize);
      data.value.push(...batch.map(item => {
        return {
          address: item.trim(),
          nonce: '',
          plat_balance: '',
          coin_balance: '',
          exec_status: '0',
          error_msg: '',
          last_transaction_time: null
        }
      }));
      // 让UI有机会更新
      await new Promise(resolve => setTimeout(resolve, 10));
    }
    
    // 显示详细的导入统计信息
    if (total_filtered_count > 0) {
      let message = `原始地址${original_count}条，成功导入${success_count}条`
      const details = []
      
      if (internal_dup_count > 0) {
        details.push(`内部重复${internal_dup_count}条`)
      }
      if (existing_dup_count > 0) {
        details.push(`与现有数据重复${existing_dup_count}条`)
      }
      
      if (details.length > 0) {
        message += `，已过滤：${details.join('、')}`
      }
      
      Notification.warning({ 
        title: '导入完成！',
        content: message
      , position: 'topLeft' })
    } else {
      Notification.success({
        title: '导入成功！',
        content: `成功导入${success_count}条地址`,
      })
    }
    
    importText.value = ''
    // 重置验证状态
    validationErrors.value = []
    errorsExpanded.value = false
    addressErrorLines.value = []
    
    return true;
  } catch (error) {
    console.error('导入失败:', error);
    Notification.error('导入失败：' + error.message);
    return false;
  } finally {
    // 结束loading
    importLoading.value = false;
  }
}

// 处理文件变化
function handleFileChange(event) {
  const file = event.target.files[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = async (e) => {
    try {
      const data = new Uint8Array(e.target.result);
      const workbook = xlUtils.read(data, { type: 'array' });
      const firstSheet = workbook.Sheets[workbook.SheetNames[0]];
      const jsonData = xlUtils.sheet_to_json(firstSheet, { header: 1 });

      const addresses = [];
      jsonData.forEach((row, index) => {
        if (row && row[0]) {
          const addr = String(row[0]).trim();
          if (validateAddress(addr)) {
            addresses.push(addr);
          }
        }
      });

      if (addresses.length > 0) {
        importText.value = addresses.join('\n');
        validateImportData();
        visible.value = true;
        Notification.success({
          content: `成功解析 ${addresses.length} 个地址`,
          position: 'topLeft',
        });
      } else {
        Notification.error({
          content: '未在文件中找到有效的地址数据',
          position: 'topLeft',
        });
      }
    } catch (error) {
      console.error('解析文件失败:', error);
      Notification.error({
        content: '解析文件失败，请确保文件格式正确',
        position: 'topLeft',
      });
    } finally {
      event.target.value = '';
    }
  };
  reader.readAsArrayBuffer(file);
}

// 删除数据
function deleteItem(item) {
  if (balanceLoading.value) {
    Notification.warning({ content: '请停止或等待查询完成后再删除数据！', position: 'topLeft' });
    return
  }
  // 删除确认
  deleteItemModalShow(item.address)
}

// 删除数据弹窗显示
function deleteItemModalShow(address) {
  deleteItemVisible.value = true
  currentItemKey.value = address
}

// 删除item取消
function deleteItemCancel() {
  deleteItemVisible.value = false
}

// 删除item确认
async function deleteItemConfirm() {
  deleteItemVisible.value = false
  data.value = data.value.filter(obj => currentItemKey.value !== obj.address)
  Notification.success({ content: '删除成功！', position: 'topLeft' });
}

// 更新余额查询进度
function updateBalanceProgress() {
  balanceCompleted.value = data.value.filter(item =>
    item.exec_status === '2' || item.exec_status === '3'
  ).length
  balanceProgress.value = balanceTotal.value > 0 ? Number((balanceCompleted.value / balanceTotal.value).toFixed(2)) : 0
}

// 创建防抖版本的操作函数
const debouncedQueryBalance = debounce(queryBalance, 500);
const debouncedStopBalanceQuery = debounce(stopBalanceQuery, 300);
const debouncedDeleteSelected = debounce(deleteSelected, 400);
const debouncedExportAllToExcel = debounce(exportAllToExcel, 600);
const debouncedExportSelectToExcel = debounce(exportSelectToExcel, 600);
const debouncedClearData = debounce(clearData, 600);
const debouncedDeleteItemConfirm = debounce(deleteItemConfirm, 400);

// 查询余额（改为使用Rust后端）
async function queryBalance() {
  if (data.value.length === 0) {
    Notification.warning({ content: '请先导入地址！', position: 'topLeft' });
    return
  }
  if (!chainValue.value) {
    Notification.warning({ content: '请先选择区块链！', position: 'topLeft' });
    return
  }
  if (!coinValue.value) {
    Notification.warning({ content: '请先选择代币！', position: 'topLeft' });
    return
  }

  // 每次查询都重新开始，重置所有状态
  executeBalanceQuery(data.value, true);
}

// 查询余额的通用方法 - 支持分批处理
async function executeBalanceQuery(queryData) {
  if (currentCoin.value.coin_type === 'base' || currentCoin.value.coin_type === 'token') {
    balanceLoading.value = true
    balanceStopFlag.value = false

    // 初始化进度条
    balanceTotal.value = data.value.length
    balanceCompleted.value = 0
    balanceProgress.value = 0

    // 重置所有项目状态和进度
    data.value.forEach(item => {
      item.nonce = ''
      item.plat_balance = ''
      item.coin_balance = ''
      item.error_msg = ''
      item.exec_status = '0'
      item.last_transaction_time = null
    })

    showProgress.value = true

    // 分批处理大数据集
    await queryBalanceInBatches()
  } else {
    Notification.warning({ content: '查询 coin 类型错误！', position: 'topLeft' });
  }
}

// 分批查询余额
async function queryBalanceInBatches() {
  const BATCH_SIZE = 50; // 每批处理50个地址
  const totalItems = data.value.length;
  const totalBatches = Math.ceil(totalItems / BATCH_SIZE);
  
  console.log(`开始分批查询余额，总数: ${totalItems}, 批次数: ${totalBatches}, 每批大小: ${BATCH_SIZE}`);
  
  try {
    for (let batchIndex = 0; batchIndex < totalBatches; batchIndex++) {
      // 检查是否需要停止查询
      if (balanceStopFlag.value) {
        balanceLoading.value = false;
        showProgress.value = false;
        return;
      }
      
      const startIndex = batchIndex * BATCH_SIZE;
      const endIndex = Math.min(startIndex + BATCH_SIZE, totalItems);
      const batchData = data.value.slice(startIndex, endIndex);
      
      console.log(`处理第 ${batchIndex + 1}/${totalBatches} 批，索引 ${startIndex}-${endIndex - 1}`);
      
      await queryBalanceBatch(batchData, startIndex);
      
      // 更新进度条
      updateBalanceProgress();
      
      // 批次间添加短暂延迟，避免过于频繁的请求
      if (batchIndex < totalBatches - 1) {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }
    
    // 所有批次完成后的统计
    const successCount = data.value.filter(item => item.exec_status === '2').length;
    const failCount = data.value.filter(item => item.exec_status === '3').length;
    const totalCount = data.value.length;
    
    // 确保进度条显示100%
    balanceProgress.value = 1;
    
    // 延迟隐藏进度条
    setTimeout(() => {
      showProgress.value = false;
    }, 3000); // 3秒后隐藏进度条
    
    if (successCount === totalCount) {
      Notification.success({ content: '查询成功！', position: 'topLeft' });
    } else if (successCount > 0) {
      Notification.warning({ content: `查询完成！成功 ${successCount} 条，失败 ${failCount} 条`, position: 'topLeft' });
    } else {
      Notification.error({ content: '查询失败：所有记录都查询失败', position: 'topLeft' });
    }
    
  } catch (error) {
    console.error('分批查询失败:', error);
    
    // 设置所有项目为失败状态
    data.value.forEach(item => {
      item.exec_status = '3';
      item.error_msg = '查询失败！';
    });
    
    // 隐藏进度条
    showProgress.value = false;
    Notification.error('查询失败：' + error.message);
  } finally {
    balanceLoading.value = false;
  }
}

// 查询单个批次的余额
async function queryBalanceBatch(batchData, startIndex) {
  try {
    // 使用Rust后端进行查询
    const params = {
      chain: chainValue.value,
      coin_config: {
        coin_type: currentCoin.value.coin_type,
        contract_address: currentCoin.value.contract_address || null,
        abi: currentCoin.value.abi || null
      },
      items: batchData.map(item => ({
        key: item.address,
        address: item.address,
        private_key: item.private_key || null,
        plat_balance: null,
        coin_balance: null,
        nonce: null,
        exec_status: '0',
        error_msg: null,
        retry_flag: false
      })),
      only_coin_config: onlyCoin.value,
      thread_count: form.thread_count,
      // 将配置参数传递给后端，按需查询最后交易时间
      query_last_transaction_time: !!form.queryLastTransactionTime
    };

    // 检查是否需要停止查询
    if (balanceStopFlag.value) {
      return;
    }

    let result;
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      result = await invoke('query_balances_with_updates', {
        params,
        windowId: currentWindowId.value
      });
    } else {
      // 浏览器环境下的模拟数据
      result = {
        success: true,
        items: batchData.map(item => ({
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
        const dataIndex = startIndex + index;
        if (data.value[dataIndex]) {
          Object.assign(data.value[dataIndex], resultItem);
        }
      });
    } else {
      // 只有在没有返回任何结果时才设置批次项目为失败状态
      batchData.forEach((item, index) => {
        const dataIndex = startIndex + index;
        if (data.value[dataIndex]) {
          data.value[dataIndex].exec_status = '3';
          data.value[dataIndex].error_msg = result.error_msg || '查询失败！';
        }
      });
    }

  } catch (error) {
    console.error('批次查询失败:', error);
    
    // 设置批次项目为失败状态
    batchData.forEach((item, index) => {
      const dataIndex = startIndex + index;
      if (data.value[dataIndex]) {
        data.value[dataIndex].exec_status = '3';
        data.value[dataIndex].error_msg = '查询失败！';
      }
    });
  }
}

// 停止余额查询
async function stopBalanceQuery() {
  console.log('停止查询按钮被点击');
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      await invoke('stop_balance_query', {
        windowId: currentWindowId.value
      });
      console.log('已发送停止查询请求到后端，窗口ID:', currentWindowId.value);
    }
  } catch (error) {
    console.error('停止查询请求失败:', error);
  }

  balanceLoading.value = false;
  balanceStopFlag.value = true;
  // 隐藏进度条
  showProgress.value = false;
  console.log('停止查询成功');
}

// 选中成功
function selectSucceeded() {
  selectedKeys.value = data.value.filter(item => item.exec_status === '2').map(item => item.address)
}

// 选中失败
function selectFailed() {
  selectedKeys.value = data.value.filter(item => item.exec_status === '3').map(item => item.address)
}

// 反选
function InvertSelection() {
  selectedKeys.value = data.value.filter(item => selectedKeys.value.indexOf(item.address) < 0).map(item => item.address)
}

// 显示高级筛选弹窗
function showAdvancedFilter() {
  advancedFilterVisible.value = true;
}

// 应用高级筛选
function applyAdvancedFilter() {
  const filteredItems = filteredData.value.map(item => item.address);
  selectedKeys.value = filteredItems;
  advancedFilterVisible.value = false;
  
  Notification.success({ content: `已筛选并选中 ${filteredItems.length} 条数据`, position: 'topLeft' });
}

function deleteSelected() {
  if (balanceLoading.value) {
    Notification.warning({ content: '请停止或等待查询完成后再删除数据！', position: 'topLeft' });
    return
  }
  data.value = data.value.filter(item => !selectedKeys.value.includes(item.address))
  Notification.success({ content: '删除成功', position: 'topLeft' })
}

function exportAllToExcel() {
  exportExcel(data.value)
}

function exportSelectToExcel() {
  const select_data = data.value.filter(item => selectedKeys.value.includes(item.address))
  exportExcel(select_data)
}

function exportExcel(target_data) {
  if (target_data.length === 0) {
    Notification.warning({ content: '无法导出空列表！', position: 'topLeft' });
    return
  }
  let export_data = [['地址', 'Nonce', '平台余额', '代币余额', '查询状态', '错误信息', '最后交易时间']]
  target_data.forEach(item => {
    const transactionTime = formatTransactionTime(item.last_transaction_time)
    export_data.push([item.address, item.nonce, item.plat_balance, item.coin_balance, item.exec_status, item.error_msg, transactionTime])
  })
  // 创建工作簿
  const workbook = xlUtils.book_new();
  // 创建工作表
  const worksheet = xlUtils.aoa_to_sheet(export_data);
  // 将工作表添加到工作簿
  xlUtils.book_append_sheet(workbook, worksheet, 'Sheet1');
  // 导出文件
  writeFile(workbook, 'balance_data.xlsx');
}

// 链管理相关方法
// 显示链管理弹窗
function showChainManage() {
  if (chainManageRef.value) {
    chainManageRef.value.show();
  }
}

// 显示RPC管理弹窗
function showRpcManage() {
  if (!chainValue.value) {
    Notification.warning({ content: "请先选择区块链！", position: 'topLeft' });
    return;
  }
  rpcManageRef.value?.show();
}

// 显示代币管理弹窗
function showTokenManage() {
  if (!chainValue.value) {
    Notification.warning({ content: "请先选择区块链！", position: 'topLeft' });
    return;
  }
  tokenManageRef.value?.show();
}

// 打开区块链浏览器
function openBlockchainScan() {
  if (!currentChain.value?.scan_url) {
    Notification.warning({ content: '当前链没有配置区块链浏览器地址', position: 'topLeft' });
    return;
  }

  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
    // 在Tauri环境中使用shell打开默认浏览器
    import('@tauri-apps/plugin-shell').then(({ open }) => {
      open(currentChain.value.scan_url);
    }).catch(error => {
      console.error('打开浏览器失败:', error);
      Notification.error({ content: '打开浏览器失败', position: 'topLeft' });
    });
  } else {
    // 在浏览器环境中直接打开新窗口
    window.open(currentChain.value.scan_url, '_blank');
  }
}

// 处理链更新事件
async function handleChainUpdated() {
  // 重新加载链数据
  try {
    chainOptions.value = await invoke('get_chain_list');

    // 检查当前选中的链是否还存在
    const currentChainExists = chainOptions.value.find(chain => chain.key === chainValue.value);

    if (!currentChainExists && chainOptions.value.length > 0) {
      // 如果当前链不存在，选择第一个可用的链
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
  } catch (error) {
    console.error('更新链列表失败:', error);
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

// 代理配置相关函数
async function openProxyConfig() {
  proxyConfigVisible.value = true;
}

// 监听代理配置变化
function handleProxyConfigChange(config) {
  proxyEnabled.value = config.enabled;
  proxyCount.value = config.proxies ? config.proxies.length : 0;
  
  // 只有在启用代理且有代理可用时才设置为'已配置'
  if (config.enabled && proxyCount.value > 0) {
    proxyStatus.value = '已配置';
  } else {
    proxyStatus.value = '未配置';
  }
  
  // 调试日志
  console.log('[代理状态更新]', {
    enabled: proxyEnabled.value,
    count: proxyCount.value,
    status: proxyStatus.value
  });
}

// 获取代理状态颜色
const proxyStatusColor = computed(() => {
  switch (proxyStatus.value) {
    case '已配置':
      return '#00b42a';
    case '连接中':
      return '#ff7d00';
    case '已连接':
      return '#00b42a';
    case '连接失败':
      return '#f53f3f';
    default:
      return '#86909c';
  }
});

// 初始化代理状态
async function initProxyStatus() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      const config = await invoke('get_proxy_config');
      handleProxyConfigChange(config);
    }
  } catch (error) {
    console.error('初始化代理状态失败:', error);
  }
}

// 处理TitleBar的before-close事件
async function handleBeforeClose() {
  try {
    console.log('TitleBar触发关闭事件，正在停止后台操作...');

    // 停止余额查询操作
    if (balanceLoading.value) {
      await stopBalanceQuery();
      console.log('已停止余额查询操作');
    }

    console.log('TitleBar窗口关闭清理完成，所有后台操作已停止');
  } catch (error) {
    console.error('处理窗口关闭事件时发生错误:', error);
  }
}
</script>

<template>
  <!-- 标题栏组件 -->
  <TitleBar :title="windowTitle" @before-close="handleBeforeClose" />

  <div class="container balance" style="height: 100vh; display: flex; flex-direction: column; overflow: hidden;">
    <!-- <span class="pageTitle">余额查询</span> -->
    <!-- 工具栏 -->
    <div class="toolBar" style="flex-shrink: 0;">
      <a-button type="primary" @click="handleClick()">
        <template #icon>
          <Icon icon="mdi:wallet" />
        </template>
        录入钱包地址
      </a-button>
      <a-divider direction="vertical" />
      <!-- 选择操作区按钮 -->
      <a-button type="outline" status="success" @click="selectSucceeded">
        <template #icon>
          <Icon icon="mdi:check" />
        </template>
        选中成功
      </a-button>
      <a-button type="outline" status="danger" style="margin-left: 10px" @click="selectFailed">
        <template #icon>
          <Icon icon="mdi:close" />
        </template>
        选中失败
      </a-button>
      <!-- 高级筛选按钮 -->
      <a-button type="outline" status="normal" style="margin-left: 10px" @click="showAdvancedFilter">
        <template #icon>
          <Icon icon="mdi:filter" />
        </template>
        高级筛选
      </a-button>
      <a-button type="outline" status="normal" style="margin-left: 10px" @click="InvertSelection">
        <template #icon>
          <Icon icon="mdi:swap-horizontal" />
        </template>
        反选
      </a-button>
      <a-button type="primary" status="danger" style="margin-left: 10px" @click="debouncedDeleteSelected">
        <template #icon>
          <Icon icon="mdi:delete" />
        </template>
        删除选中
      </a-button>
      <!-- 代理配置按钮 -->
      <a-button type="outline" status="normal" style="margin-left: 10px" @click="openProxyConfig">
        <template #icon>
          <Icon icon="mdi:proxy" />
        </template>
        代理配置
        <a-tag :color="proxyEnabled ? proxyStatusColor : '#86909c'" size="small" style="margin-left: 4px;">
          {{ proxyEnabled ? proxyCount + '个' : '未启用' }}
        </a-tag>
      </a-button>
      <a-divider direction="vertical" />
      <a-button type="primary" status="success" @click="debouncedExportAllToExcel">
        <template #icon>
          <Icon icon="mdi:download" />
        </template>
        导出全表
      </a-button>
      <a-button type="outline" status="normal" style="margin-left: 10px" @click="debouncedExportSelectToExcel">
        <template #icon>
          <Icon icon="mdi:download" />
        </template>
        导出选中
      </a-button>
      <a-button type="primary" status="danger" style="float: right;margin-right: 10px" @click="debouncedClearData">
        <template #icon>
          <Icon icon="mdi:delete" />
        </template>
        清空列表
      </a-button>
    </div>
    <!-- 操作账号表格 -->
    <div class="mainTable" style="flex: 1; overflow: hidden; display: flex; flex-direction: column; min-height: 0;">
      <!-- 骨架屏 -->
      <TableSkeleton v-if="balanceLoading && data.length === 0" :rows="8" />
      
      <!-- 正常表格 -->
      <VirtualScrollerTable 
        v-else-if="tableBool" 
        :columns="columns" 
        :data="filteredData"
        :row-selection="rowSelection" 
        :selected-keys="selectedKeys"
        @row-click="rowClick"
        @update:selected-keys="selectedKeys = $event"
        @open-manual-import="handleManualImport"
        @open-file-upload="handleFileUpload"
        @download-template="downloadTemplate"
        row-key="address"
        height="100%"
        page-type="balance"
        :empty-data="filteredData.length === 0"
      >

        <template #exec_status="{ record }">
          <a-tag v-if="record.exec_status === '0'" color="#86909c">等待查询
          </a-tag>
          <a-tag v-if="record.exec_status === '1'" color="#ff7d00">查询中
          </a-tag>
          <a-tag v-if="record.exec_status === '2'" color="#00b42a">查询成功
          </a-tag>
          <a-tag v-if="record.exec_status === '3'" color="#f53f3f">查询失败
          </a-tag>
        </template>
        <template #last_transaction_time="{ record }">
          <span>{{ formatTransactionTime(record.last_transaction_time) }}</span>
        </template>
        <template #optional="{ record }">
          <a-button type="text" size="small" @click.stop="deleteItem(record)" status="danger">
            <template #icon>
              <icon-delete />
            </template>
          </a-button>
        </template>
      </VirtualScrollerTable>
    </div>

    <!-- 余额查询进度条 - 悬浮在页面顶部 -->
    <Transition name="progress-slide" appear>
      <div v-if="showProgress" class="floating-progress-bar">
        <div class="progress-content">
          <div class="progress-header">
            <span class="progress-title">查询进度</span>
            <span class="progress-count">{{ balanceCompleted }}/{{ balanceTotal }}</span>
          </div>
          <a-progress 
            :percent="balanceProgress" 
            :stroke-width="6" 
            :animation="true" 
            :color="{
              '0%': '#37ecba',
              '100%': '#009efd',
            }" 
            class="progress-bar"
          />
        </div>
      </div>
    </Transition>

    <!-- 链管理按钮嵌入 -->
    <div style="display: flex; gap: 10px; align-items: center; margin-top: 10px; flex-shrink: 0;">
      <!-- 链管理按钮 -->
      <a-button type="primary" @click="showChainManage" style="white-space: nowrap;">
        <template #icon>
          <Icon icon="mdi:settings" />
        </template>
        区块链管理
      </a-button>
      <a-button type="primary" @click="showRpcManage" :disabled="!chainValue" style="white-space: nowrap;">
        <template #icon>
          <Icon icon="mdi:link" />
        </template>
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
              style="width: 20px; height: 20px;" />
            <span style="margin-left: 10px">{{ data?.name }}</span>
            <span style="margin-left: 20px;color: #c3c3c3;">{{ data?.scan_url }}</span>
          </div>
        </template>
        <template #option="{ data }">
          <div style="display: flex; flex-direction: row; align-items: center;height: 32px;">
            <ChainIcon :chain-key="data?.key" :pic-data="data?.pic_data" :alt="data?.name"
              style="width: 20px; height: 20px;" />
            <span style="margin-left: 10px">{{ data?.name }}</span>
            <span style="margin-left: 20px;color: #c3c3c3;">{{ data?.scan_url }}</span>
          </div>
        </template>
      </a-select>
      <!-- 区块链浏览器跳转按钮 -->
      <a-tooltip v-if="currentChain?.scan_url" content="在浏览器中打开区块链浏览器">
        <a-button type="primary" @click="openBlockchainScan" shape="round" style="white-space: nowrap; padding: 0 8px;">
          <Icon icon="mdi:open-in-new" />
        </a-button>
      </a-tooltip>
      <a-button type="primary" @click="showTokenManage" :disabled="!chainValue" style="white-space: nowrap;">
        <template #icon>
          <Icon icon="mdi:cog" />
        </template>
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
    <!-- 相关设置 -->
    <div style="display: flex; padding-top: 5px; align-items: center; flex-shrink: 0;">
      <!-- 表单配置 -->
      <a-form :model="form" auto-label-width="true">
        <div style="display: flex; align-items: end; gap: 20px;">
          <!-- 仅查询目标代币开关 -->
          <!-- 是否查询最后交易时间 -->
          <a-form-item label="查询最后交易时间" style="width: 160px;margin-bottom: 0;">
            <a-switch v-model="form.queryLastTransactionTime" />
          </a-form-item>
           <a-form-item label="仅查询目标代币" style="width: 160px;margin-bottom: 0;">
            <a-switch v-model="onlyCoin" />
          </a-form-item>
          <!-- 线程数配置 -->
          <a-form-item field="thread_count" label="线程数" style="width: 260px; margin-bottom: 0;"
            tooltip="同时查询的钱包数量（1-99）之间">
            <a-input-number :max="99" :min="1" mode="button" v-model="form.thread_count" style="width: 100%;" />
          </a-form-item>
        </div>
      </a-form>
    </div>
    <div style="display: flex; gap: 10px; align-items: center; justify-content: center; margin-top: 5px; flex-shrink: 0;">
      <!-- 查询按钮 -->
      <a-tooltip v-if="balanceLoading" content="点击可以提前停止查询">
        <a-button type="primary" status="danger" class="execute-btn" style="height: 40px;width: 130px;font-size: 14px;" @click="debouncedStopBalanceQuery">
          <template #icon>
            <Icon icon="mdi:stop" />
          </template>
          查询中...
        </a-button>
      </a-tooltip>
      <a-button v-else type="primary" status="success" class="execute-btn" style="height: 40px;width: 130px;font-size: 14px;" @click="debouncedQueryBalance">
        <template #icon>
          <Icon icon="mdi:play" />
        </template>
        查询余额
      </a-button>
    </div>
  </div>
  <!-- 录入弹窗 -->
  <a-modal v-model:visible="visible" :width="700" title="录入钱包地址" @cancel="handleCancel" :on-before-ok="handleBeforeOk"
    :confirm-loading="importLoading"
    :cancel-button-props="{ disabled: importLoading }"
    :mask-closable="!importLoading"
    :closable="!importLoading">
    <div style="margin-top: 10px; height: 400px; position: relative;">
      <!-- Loading 遮罩层 -->
      <div v-if="importLoading" class="loading-overlay">
        <a-spin size="large" />
        <div class="loading-text">正在导入数据，请稍候...</div>
      </div>
      
      <CodeEditor 
        v-model="importText" 
        :error-lines="addressErrorLines"
        :disabled="importLoading"
        placeholder="格式：一行一个地址&#10;示例：0x742d35Cc6634C0532925a3b8D4..."
        @input="validateImportData"
        style="height: 100%;"
      />
    </div>
    
    <!-- 验证错误提示 -->
    <div v-if="validationErrors.length > 0" style="margin-top: 15px;">
      <a-alert style="padding: 5px 15px;" type="error" :title="`发现 ${validationErrors.length} 个问题`" :show-icon="true">
        <ul style="margin: 8px 0 0 0; padding-left: 20px;">
          <li v-for="(error, index) in displayedErrors" :key="error"
            style="margin-bottom: 4px; color: #f53f3f; font-size: 12px;">{{ error }}</li>
        </ul>
        <div v-if="validationErrors.length > 3" style="margin-top: 10px; text-align: center;">
          <a-button type="text" size="small" @click="toggleErrorsExpanded" style="color: #165dff;font-size: 12px;">
            {{ errorsExpanded ? '⬆️收起' : '⬇️展开全部' }}
          </a-button>
        </div>
      </a-alert>
    </div>
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
    <div>确认删除地址为【
      {{ currentItemKey.substring(0, 15) + '......' + currentItemKey.substring(currentItemKey.length - 15) }}
      】的数据？
    </div>
    <template #footer>
      <a-button @click="deleteItemCancel">取消</a-button>
      <a-button type="primary" status="danger" @click="debouncedDeleteItemConfirm" style="margin-left: 10px">确定
      </a-button>
    </template>
  </a-modal>

  <!-- 高级筛选弹窗 -->
  <a-modal v-model:visible="advancedFilterVisible" title="高级筛选" width="500px">
    <a-form :model="filterForm" layout="vertical">
      <!-- 平台币余额筛选 -->
      <a-form-item label="平台币余额筛选">
        <div style="display: flex; gap: 10px; align-items: center;">
          <a-select v-model="filterForm.platBalanceOperator" style="width: 100px;">
            <a-option value="gt">大于</a-option>
            <a-option value="eq">等于</a-option>
            <a-option value="lt">小于</a-option>
          </a-select>
          <a-input v-model="filterForm.platBalanceValue" placeholder="请输入平台币余额值" style="flex: 1;" @input="debouncedFilterUpdate" />
        </div>
      </a-form-item>
      
      <!-- 代币余额筛选 -->
      <a-form-item label="代币余额筛选">
        <div style="display: flex; gap: 10px; align-items: center;">
          <a-select v-model="filterForm.coinBalanceOperator" style="width: 100px;">
            <a-option value="gt">大于</a-option>
            <a-option value="eq">等于</a-option>
            <a-option value="lt">小于</a-option>
          </a-select>
          <a-input v-model="filterForm.coinBalanceValue" placeholder="请输入代币余额值" style="flex: 1;" @input="debouncedFilterUpdate" />
        </div>
      </a-form-item>
      
      <!-- Nonce值筛选 -->
      <a-form-item label="Nonce值筛选">
        <div style="display: flex; gap: 10px; align-items: center;">
          <a-select v-model="filterForm.nonceOperator" style="width: 100px;">
            <a-option value="gt">大于</a-option>
            <a-option value="eq">等于</a-option>
            <a-option value="lt">小于</a-option>
          </a-select>
          <a-input v-model="filterForm.nonceValue" placeholder="请输入数值" style="flex: 1;" @input="debouncedFilterUpdate" />
        </div>
      </a-form-item>
      
      <!-- 错误信息模糊匹配 -->
      <a-form-item label="错误信息模糊匹配">
        <a-input v-model="filterForm.errorMsg" placeholder="请输入要匹配的错误信息" @input="debouncedFilterUpdate" />
      </a-form-item>
    </a-form>
    
    <template #footer>
      <a-button @click="advancedFilterVisible = false">取消</a-button>
      <a-button type="primary" @click="applyAdvancedFilter" style="margin-left: 10px;">应用筛选</a-button>
    </template>
  </a-modal>

  <!-- 链管理组件 -->
  <ChainManagement ref="chainManageRef" @chain-updated="handleChainUpdated" />
  <!-- RPC管理组件 -->
  <RpcManagement ref="rpcManageRef" :chain-value="chainValue" @rpc-updated="handleRpcUpdated" />
  <!-- 代币管理组件 -->
  <TokenManagement ref="tokenManageRef" :chain-value="chainValue" @token-updated="handleTokenUpdated" />
  
  <!-- 代理配置弹窗 -->
  <ProxyConfigModal 
    v-if="proxyConfigVisible" 
    :visible="proxyConfigVisible" 
    @update:visible="proxyConfigVisible = $event"
    @config-change="handleProxyConfigChange"
    ref="proxyConfigRef"
  />

  <!-- 隐藏的文件输入框 -->
  <input
    type="file"
    ref="uploadInputRef"
    accept=".xlsx,.xls,.csv"
    style="display: none"
    @change="handleFileChange"
  />
</template>

<style scoped lang="less">
/* Loading 遮罩层样式 */
.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--loading-bg, #f7f8fa);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  border-radius: 6px;
}

.loading-text {
  margin-top: 16px;
  color: #4e5969;
  font-size: 14px;
  font-weight: 500;
}

.minimize-icon {
  font-size: 14px;
}

.close-icon {
  font-size: 14px;
}

/* 调整容器以适应自定义标题栏 */
.container {
  padding: 10px;
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

.pageTitle {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 20px;
  background: linear-gradient(120deg, #11c06f 0%, #165dff 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  position: relative;
  display: inline-block;
}

.pageTitle::after {
  content: "";
  position: absolute;
  bottom: -5px;
  left: 0;
  width: 100%;
  height: 3px;
  background: linear-gradient(120deg, #11c06f 0%, #165dff 100%);
  border-radius: 3px;
}

.pageTitle::before {
  content: "💰";
  margin-right: 10px;
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

.mainTable {
  margin-top: 15px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.subTitle {
  font-weight: 600;
  font-size: 16px;
  margin: 10px 0 0 10px;
}

.arco-form-item {
  padding: 5px 10px;
  margin-bottom: 10px;
}

.arco-btn-secondary.arco-btn-loading {
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

.execute-btn {
  width: 150px;
  height: 50px;
  font-size: 16px;
  color: #ffffff;
  background-color: #0fa962;
  border: none;
}

.toolBar {
  margin-top: 45px;
}

/* 悬浮进度条样式 */
.floating-progress-bar {
  position: fixed;
  top: 45px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 1000;
  width: 90%;
  max-width: 600px;
  background: var(--card-bg, #ffffff);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12), 0 2px 8px rgba(0, 0, 0, 0.08);
  border: 1px solid var(--border-color, #e5e6eb);
  backdrop-filter: blur(8px);
}

.progress-content {
  padding: 5px 20px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-color, #1d2129);
}

.progress-count {
  font-size: 13px;
  color: var(--text-color-secondary, #86909c);
  font-weight: 500;
}

.progress-bar {
  width: 100%;
}

/* 进度条动画 */
.progress-slide-enter-active {
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.progress-slide-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.6, 1);
}

.progress-slide-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(-100%);
}

.progress-slide-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-100%);
}

.progress-slide-enter-to,
.progress-slide-leave-from {
  opacity: 1;
  transform: translateX(-50%) translateY(0);
}
</style>