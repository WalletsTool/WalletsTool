<script setup name="transfer-solana">
import { Icon } from '@iconify/vue';
import { useRouter, useRoute } from "vue-router";
import { IconDelete } from '@arco-design/web-vue/es/icon';
import { computed, defineAsyncComponent, onBeforeMount, onBeforeUnmount, onMounted, reactive, ref, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Notification, Modal } from "@arco-design/web-vue";

import { read, utils as xlUtils, writeFile } from "xlsx";
import { debounce as customDebounce } from '@/utils/debounce.js'
import ChainIcon from '@/components/ChainIcon.vue'
import TitleBar from '@/components/TitleBar.vue'
import TableSkeleton from '@/components/TableSkeleton.vue'
import VirtualScrollerTable from '@/components/VirtualScrollerTable.vue'

// 懒加载非关键组件
const ChainManagement = defineAsyncComponent(() => import('@/components/ChainManagement.vue'))
const RpcManagement = defineAsyncComponent(() => import('@/components/RpcManagement.vue'))
const TokenManagement = defineAsyncComponent(() => import('@/components/TokenManagement.vue'))
const WalletImportModal = defineAsyncComponent(() => import('@/components/WalletImportModal.vue'))
const router = useRouter();
// 窗口标题
const windowTitle = ref('Solana 批量转账');
// table列名
const columns = [
  {
    title: "序号",
    align: "center",
    width: 55,
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
    width: 250,
    dataIndex: "to_addr",
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "转账数量",
    align: "center",
    dataIndex: "amount",
    width: 95,
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "平台币余额",
    align: "center",
    dataIndex: "plat_balance",
    width: 95,
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "代币余额",
    align: "center",
    dataIndex: "coin_balance",
    width: 85,
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "状态",
    align: "center",
    slotName: "exec_status",
    width: 90,
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "返回信息",
    align: "center",
    dataIndex: "error_msg",
    ellipsis: true,
    tooltip: true,
  },
  {
    title: "操作",
    align: "center",
    slotName: "optional",
    width: 55,
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
// 当前rpc
const currentChain = ref({});
// rpc自定义字段名
const chainFieldNames = { value: "key", label: "scan_url" };
// 主网选择器
let chainOptions = ref([]);
// coin自定义字段名
const coinFieldNames = { value: "key", label: "label" };
// 币种选择器
const coinOptions = ref([]);
// 查询余额按钮loading
let balanceLoading = ref(false);
// 查询余额中途停止
let balanceStopFlag = ref(false);
// 查询余额是否已经停止
let balanceStopStatus = ref(true);
// 详细配置 - 去掉gas相关配置
const form = reactive({
  send_type: "3",
  amount_from: "1",
  send_count: "0",
  send_min_count: "1",
  send_max_count: "100",
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
const uploadInputRef = ref(null);

// 钱包导入相关
const walletImportVisible = ref(false);
const fileLoading = ref(false);
const advancedFilterVisible = ref(false);

// 进度相关
const progressData = ref({
  total: 0,
  success: 0,
  failed: 0,
  pending: 0
});

// 智能重试相关
const retryData = ref({
  total: 0,
  success: 0,
  failed: 0
});
const filterForm = reactive({
  platBalanceOperator: 'gt', // gt: 大于, eq: 等于, lt: 小于
  platBalanceValue: '',
  coinBalanceOperator: 'gt', // gt: 大于, eq: 等于, lt: 小于
  coinBalanceValue: '',
  errorMsg: ''
});
// 删除信息弹窗
let deleteItemVisible = ref(false);
// 当前币种名称
let currentCoin = ref({});
// 当前数据的key
let currentItemKey = ref("");
// 当前要删除项目的私钥
let currentItemPrivateKey = ref("");
// 开始执行按钮loading
let startLoading = ref(false);
// 转账中途停止
let stopFlag = ref(false);
// 转账是否已经停止
let stopStatus = ref(true);
// 是否执行过真正的转账操作（用于区分余额查询和转账）
let hasExecutedTransfer = ref(false);
// 转账确认弹窗相关变量
const transferConfirmVisible = ref(false);
const transferConfirmData = ref([]);
const transferConfirmLoading = ref(false);
// 线程数设置，默认为1
let threadCount = ref(1);
// 多窗口数量设置，默认为1
let multiWindowCount = ref(1);

// 链和币种选择
const chainValue = ref('');
const coinValue = ref('');

// 智能重试相关变量
const transferStartTime = ref(null); // 转账开始时间戳
const retryInProgress = ref(false); // 是否正在进行重试检查
const retryResults = ref([]); // 重试检查结果

// 监听线程数变化，自动调整间隔时间
watch(threadCount, (newValue) => {
  if (newValue > 1) {
    // 线程数大于1时，设置间隔时间为0
    form.min_interval = "0";
    form.max_interval = "0";
  } else {
    // 线程数等于1时，恢复默认间隔时间
    form.min_interval = "1";
    form.max_interval = "3";
  }
});

// 转账进度相关变量
const transferProgress = ref(0); // 转账进度百分比
const transferTotal = ref(0); // 总转账数量
const transferCompleted = ref(0); // 已完成转账数量
const showProgress = ref(false); // 是否显示进度条

// 计算属性：缓存转账配置 - 去掉gas相关配置
const transferConfig = computed(() => {
  return {
    error_count_limit: 3,
    error_retry: form.error_retry,
    chain: chainValue.value,
    chainLayer: currentChain.value.layer,
    l1: currentChain.value.l1,
    scalar: currentChain.value.scalar,
    delay: [
      form.min_interval && form.min_interval.trim() !== '' ? Number(form.min_interval) : 1,
      form.max_interval && form.max_interval.trim() !== '' ? Number(form.max_interval) : 3
    ],
    transfer_type: form.send_type,
    transfer_amount_list: [
      form.send_min_count && form.send_min_count.trim() !== '' ? Number(form.send_min_count) : 0,
      form.send_max_count && form.send_max_count.trim() !== '' ? Number(form.send_max_count) : 0
    ],
    left_amount_list: [
      form.send_min_count && form.send_min_count.trim() !== '' ? Number(form.send_min_count) : 0,
      form.send_max_count && form.send_max_count.trim() !== '' ? Number(form.send_max_count) : 0
    ],
    amount_precision: form.amount_precision && form.amount_precision.trim() !== '' ? Number(form.amount_precision) : 6
  };
});

// 计算属性：缓存统计数据
const transferStatistics = computed(() => {
  const total = data.value.length;
  const pending = data.value.filter(item => item.exec_status === '0').length;
  const processing = data.value.filter(item => item.exec_status === '1').length;
  const succeeded = data.value.filter(item => item.exec_status === '2').length;
  const failed = data.value.filter(item => item.exec_status === '3').length;

  return { total, pending, processing, succeeded, failed };
});

// 计算属性：缓存筛选后的数据
const filteredTransferData = computed(() => {
  if (!filterForm.platBalanceValue && !filterForm.coinBalanceValue && !filterForm.errorMsg) {
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

// 防抖的筛选更新函数
const debouncedFilterUpdate = customDebounce(() => {
  // 触发筛选数据的重新计算
  // filteredTransferData computed属性会自动响应filterForm的变化
}, 300);

// 代币管理相关变量
const tokenTableLoading = ref(false);
const tokenManageData = ref([]);

// Solana多窗口功能
function openMultipleWindow() {
  try {
    for (let i = 1; i < multiWindowCount.value; i++) {
      const windowLabel = `solana-transfer-${Date.now()}-${i}`;
      new WebviewWindow(windowLabel, {
        url: '/sol/transfer',
        title: `Solana 批量转账 - 窗口 ${i + 1}`,
        width: 1400,
        height: 900,
        resizable: true,
        maximized: false,
        decorations: false,
        transparent: true,
      });
    }
    Notification.success(`已打开 ${multiWindowCount.value - 1} 个新窗口`);
  } catch (error) {
    console.error('打开多窗口失败:', error);
    Notification.error('打开多窗口失败');
  }
}

// Solana配置共享功能
function applySharedConfig(config) {
  try {
    // 应用表单配置
    if (config.form) {
      Object.assign(form, config.form);
    }
    // 应用线程数配置
    if (config.threadCount) {
      threadCount.value = config.threadCount;
    }
    // 应用链配置
    if (config.chain) {
      chainValue.value = config.chain;
    }
    // 应用代币配置
    if (config.coin) {
      coinValue.value = config.coin;
    }
    // 应用数据
    if (config.data && Array.isArray(config.data)) {
      data.value = config.data;
    }
    Notification.success('配置应用成功');
  } catch (error) {
    console.error('应用配置失败:', error);
    Notification.error('应用配置失败');
  }
}

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

// Solana地址验证函数
function isValidSolanaAddress(address) {
  if (!address || typeof address !== 'string') {
    return false;
  }
  
  const trimmedAddress = address.trim();
  // Solana地址是Base58编码，长度通常为32-44个字符
  const base58Regex = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;
  return base58Regex.test(trimmedAddress);
}

// Solana私钥验证函数
function isValidSolanaPrivateKey(privateKey) {
  if (!privateKey || typeof privateKey !== 'string') {
    return false;
  }
  
  const trimmedKey = privateKey.trim();
  // Solana私钥通常是Base58编码的64字节数组
  const base58Regex = /^[1-9A-HJ-NP-Za-km-z]{87,88}$/;
  return base58Regex.test(trimmedKey);
}

// Solana特定的智能重试函数
async function performIntelligentRetry(failedData) {
  try {
    retryInProgress.value = true;
    retryResults.value = [];
    
    for (const item of failedData) {
      try {
        // 检查交易是否已经在链上成功
        const isSuccess = await checkRecentTransfer(
          item.private_key,
          item.to_addr,
          transferStartTime.value
        );
        
        if (isSuccess) {
          // 更新状态为成功
          item.exec_status = '2';
          item.error_msg = '链上检查发现交易已成功';
          retryResults.value.push({
            privateKey: item.private_key,
            action: '跳过重试',
            reason: '交易已在链上成功'
          });
        } else {
          // 加入重试队列
          item.exec_status = '0';
          item.error_msg = '';
          retryResults.value.push({
            privateKey: item.private_key,
            action: '加入重试',
            reason: '交易未在链上找到'
          });
        }
      } catch (error) {
        console.error('检查交易状态失败:', error);
        // 默认加入重试
        item.exec_status = '0';
        retryResults.value.push({
          privateKey: item.private_key,
          action: '加入重试',
          reason: '检查失败，默认重试'
        });
      }
    }
    
    retryInProgress.value = false;
    
    // 开始重试失败的交易
    const retryData = failedData.filter(item => item.exec_status === '0');
    if (retryData.length > 0) {
      transferFnc(retryData);
    }
  } catch (error) {
    console.error('智能重试失败:', error);
    retryInProgress.value = false;
    Notification.error('智能重试失败');
  }
}

// Solana特定的交易检查函数
async function checkRecentTransfer(privateKey, targetAddress, startTime) {
  try {
    // 调用后端API检查Solana交易
    const result = await invoke('check_solana_recent_transfer', {
      privateKey,
      targetAddress,
      startTime
    });
    return result || false;
  } catch (error) {
    console.error('检查Solana交易失败:', error);
    return false;
  }
}

// Solana转账核心函数
function transferFnc(transferData) {
  if (!transferData || transferData.length === 0) {
    Notification.warning('没有可转账的数据');
    return;
  }
  
  // 设置转账状态
  startLoading.value = true;
  stopStatus.value = false;
  stopFlag.value = false;
  hasExecutedTransfer.value = true;
  
  // 设置进度条
  transferTotal.value = transferData.length;
  transferCompleted.value = 0;
  transferProgress.value = 0;
  showProgress.value = true;
  transferStartTime.value = Date.now();
  
  // 开始转账
  startTransfer(transferData);
}

// 开始转账函数
async function startTransfer(transferData) {
  try {
    // 调用后端Solana转账API
    await invoke('start_solana_transfer', {
      transferData,
      config: transferConfig.value,
      threadCount: threadCount.value
    });
  } catch (error) {
    console.error('启动Solana转账失败:', error);
    Notification.error('启动转账失败: ' + error.message);
    // 重置状态
    startLoading.value = false;
    stopStatus.value = true;
    showProgress.value = false;
  }
}

// 防抖函数包装器
const debouncedOpenMultipleWindow = customDebounce(openMultipleWindow, 1000);
const debouncedStartTransfer = customDebounce(() => {
  const selectedData = data.value.filter(item => selectedKeys.value.includes(item.key));
  if (selectedData.length === 0) {
    Notification.warning('请先选择要转账的数据');
    return;
  }
  transferFnc(selectedData);
}, 1000);

const debouncedStopTransfer = customDebounce(() => {
  stopFlag.value = true;
  invoke('stop_solana_transfer').catch(error => {
    console.error('停止转账失败:', error);
  });
}, 500);

const debouncedQueryBalance = customDebounce(() => {
  queryBalance('from');
}, 1000);

const debouncedQueryToAddressBalance = customDebounce(() => {
  queryBalance('to');
}, 1000);

const debouncedStopBalanceQuery = customDebounce(() => {
  balanceStopFlag.value = true;
  invoke('stop_balance_query').catch(error => {
    console.error('停止余额查询失败:', error);
  });
}, 500);

const debouncedClearData = customDebounce(() => {
  Modal.confirm({
    title: '确认清空',
    content: '确定要清空所有数据吗？此操作不可撤销。',
    onOk: () => {
      data.value = [];
      selectedKeys.value = [];
      Notification.success('数据已清空');
    }
  });
}, 500);

const debouncedDeleteItemConfirm = customDebounce(() => {
  const index = data.value.findIndex(item => item.key === currentItemKey.value);
  if (index !== -1) {
    data.value.splice(index, 1);
    // 如果删除的项目在选中列表中，也要移除
    const selectedIndex = selectedKeys.value.indexOf(currentItemKey.value);
    if (selectedIndex !== -1) {
      selectedKeys.value.splice(selectedIndex, 1);
    }
    Notification.success('删除成功');
  }
  deleteItemVisible.value = false;
}, 500);

// 余额查询函数
async function queryBalance(type = 'from') {
  if (data.value.length === 0) {
    Notification.warning('请先导入数据');
    return;
  }
  
  if (!chainValue.value) {
    Notification.warning('请先选择区块链');
    return;
  }
  
  try {
    balanceLoading.value = true;
    balanceStopStatus.value = false;
    balanceStopFlag.value = false;
    
    await invoke('query_solana_balance', {
      data: data.value,
      chain: chainValue.value,
      coin: coinValue.value,
      type,
      threadCount: threadCount.value
    });
  } catch (error) {
    console.error('查询余额失败:', error);
    Notification.error('查询余额失败: ' + error.message);
    balanceLoading.value = false;
    balanceStopStatus.value = true;
  }
}

// 钱包导入函数
function importWallet() {
  walletImportVisible.value = true;
}

// 文件导入函数
async function importFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'CSV Files',
        extensions: ['csv']
      }, {
        name: 'Excel Files', 
        extensions: ['xlsx', 'xls']
      }]
    });
    
    if (selected) {
      fileLoading.value = true;
      const result = await invoke('import_solana_transfer_file', { filePath: selected });
      if (result && result.length > 0) {
        data.value = result.map((item, index) => ({
          ...item,
          key: `${Date.now()}-${index}`
        }));
        Notification.success(`成功导入 ${result.length} 条数据`);
      } else {
        Notification.warning('文件中没有有效数据');
      }
    }
  } catch (error) {
    console.error('导入文件失败:', error);
    Notification.error('导入文件失败: ' + error.message);
  } finally {
    fileLoading.value = false;
  }
}

// 下载模板函数
async function downloadTemplate() {
  try {
    const savePath = await save({
      defaultPath: 'solana_transfer_template.csv',
      filters: [{
        name: 'CSV Files',
        extensions: ['csv']
      }]
    });
    
    if (savePath) {
      await invoke('download_solana_transfer_template', { savePath });
      Notification.success('模板下载成功');
    }
  } catch (error) {
    console.error('下载模板失败:', error);
    Notification.error('下载模板失败: ' + error.message);
  }
}

// 删除选中项目
function deleteSelectedItems() {
  if (selectedKeys.value.length === 0) {
    Notification.warning('请先选择要删除的项目');
    return;
  }
  
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除选中的 ${selectedKeys.value.length} 个项目吗？`,
    onOk: () => {
      data.value = data.value.filter(item => !selectedKeys.value.includes(item.key));
      selectedKeys.value = [];
      Notification.success('删除成功');
    }
  });
}

// 删除单个项目
function deleteItem(record) {
  currentItemKey.value = record.key;
  deleteItemVisible.value = true;
}

// 表格选择处理
function handleSelectionChange(keys) {
  selectedKeys.value = keys;
}

// 全选/取消全选
function handleSelectAll(checked) {
  if (checked) {
    selectedKeys.value = data.value.map(item => item.key);
  } else {
    selectedKeys.value = [];
  }
}

// 筛选功能
function showAdvancedFilter() {
  advancedFilterVisible.value = true;
}

function applyAdvancedFilter() {
  // 应用高级筛选逻辑
  advancedFilterVisible.value = false;
  Notification.success('筛选条件已应用');
}
// 重置筛选条件
function resetAdvancedFilter() {
  // 重置筛选条件
  Notification.success('筛选条件已重置');
}

// 转账确认相关
function showTransferConfirm() {
  const selectedData = data.value.filter(item => selectedKeys.value.includes(item.key));
  if (selectedData.length === 0) {
    Notification.warning('请先选择要转账的数据');
    return;
  }
  transferConfirmData.value = selectedData;
  transferConfirmVisible.value = true;
}

function confirmTransfer() {
  transferConfirmVisible.value = false;
  transferFnc(transferConfirmData.value);
}

// 页面生命周期
onMounted(() => {
  // 监听后端事件
  listen('transfer_progress', (event) => {
    progressData.value = event.payload;
  });
  
  listen('transfer_complete', (event) => {
    startLoading.value = false;
    stopStatus.value = true;
    showProgress.value = false;
    Notification.success('转账完成');
  });
  
  listen('balance_progress', (event) => {
    // 更新余额查询进度
    const { index, total, result } = event.payload;
    if (result && data.value[index]) {
      data.value[index] = { ...data.value[index], ...result };
    }
  });
  
  listen('balance_complete', (event) => {
    balanceLoading.value = false;
    balanceStopStatus.value = true;
    Notification.success('余额查询完成');
  });
  
  listen('retry_progress', (event) => {
    retryData.value = event.payload;
  });
});

// 工具函数
function formatNumber(num) {
  if (!num) return '0';
  return Number(num).toLocaleString();
}

function formatAddress(address) {
  if (!address) return '';
  if (address.length <= 10) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

function getStatusColor(status) {
  const colorMap = {
    'pending': 'blue',
    'success': 'green', 
    'failed': 'red',
    'processing': 'orange'
  };
  return colorMap[status] || 'gray';
}

function getStatusText(status) {
  const textMap = {
    'pending': '待处理',
    'success': '成功',
    'failed': '失败', 
    'processing': '处理中'
  };
  return textMap[status] || '未知';
}

</script>

<template>
  <div class="container">
    <TitleBar :title="windowTitle" />
    <!-- 页面标题 -->
    <div class="pageTitle">Solana Transfer</div>
    <!-- 工具栏 -->
    <div class="toolBar">
      <a-button type="primary" @click="handleClick">
        <template #icon>
          <Icon icon="mdi:wallet" />
        </template>
        钱包录入
      </a-button>
      <a-button type="primary" style="margin-left: 10px" @click="upload">
        <template #icon>
          <Icon icon="mdi:upload" />
        </template>
        导入文件
      </a-button>
      <input ref="uploadInputRef" type="file" accept=".xlsx,.xls" style="display: none" @change="UploadFile" />
      <a-button type="outline" status="success" style="margin-left: 10px" @click="selectSuccess">
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
      <a-button type="outline" status="normal" style="margin-left: 10px" @click="InvertSelection">
        <template #icon>
          <Icon icon="mdi:swap-horizontal" />
        </template>
        反选
      </a-button>
      <!-- 高级筛选按钮 -->
      <a-button type="primary" status="normal" style="margin-left: 10px" @click="showAdvancedFilter">
        <template #icon>
          <Icon icon="mdi:filter" />
        </template>
        高级筛选
      </a-button>
      <a-button type="primary" status="danger" style="margin-left: 10px" @click="deleteSelected">
        <template #icon>
          <Icon icon="mdi:delete" />
        </template>
        删除选中
      </a-button>
      <a-button type="primary" status="danger" style="float: right; margin-right: 10px" @click="debouncedClearData">
        <template #icon>
          <Icon icon="mdi:delete" />
        </template>
        清空列表
      </a-button>
      <a-button type="outline" status="normal" style="float: right; margin-right: 10px" @click="downloadFile">
        <template #icon>
          <Icon icon="mdi:download" />
        </template>
        下载模板
      </a-button>
    </div>
    <!-- 操作账号表格 -->
    <div class="mainTable" style="flex: 1; overflow: hidden; display: flex; flex-direction: column;">
      <!-- 骨架屏 -->
      <TableSkeleton v-if="(tableLoading || balanceLoading) && data.length === 0" :rows="8" />

      <!-- 正常表格 -->
      <VirtualScrollerTable v-else-if="tableBool" :columns="columns" :data="filteredTransferData" :row-selection="rowSelection"
                            :loading="tableLoading" :selected-keys="selectedKeys" @row-click="rowClick"
                            @update:selected-keys="selectedKeys = $event" row-key="key" height="100%">

        <template #exec_status="{ record }">
          <a-tag v-if="record.exec_status === '0'" color="#86909c">等待执行
          </a-tag>
          <a-tag v-if="record.exec_status === '1'" color="#ff7d00">执行中
          </a-tag>
          <a-tag v-if="record.exec_status === '2'" color="#00b42a">执行成功
          </a-tag>
          <a-tag v-if="record.exec_status === '3'" color="#f53f3f">执行失败
          </a-tag>
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

    <!-- 转账进度条 - 悬浮在页面顶部 -->
    <Transition name="progress-slide" appear>
      <div v-if="showProgress" class="floating-progress-bar">
        <div class="progress-content">
          <div class="progress-header">
            <span class="progress-title">转账进度</span>
            <span class="progress-count">{{ transferCompleted }} / {{ transferTotal }}</span>
          </div>
          <a-progress :percent="transferProgress" :show-text="true" :stroke-width="6" :color="{
            '0%': '#00b42a',
            '100%': '#00b42a'
          }" class="progress-bar" />
        </div>
      </div>
    </Transition>

    <!-- 智能重试状态显示 -->
    <div v-if="retryInProgress"
         style="margin-top: 10px; padding: 10px; background: #f8f9fa; border-radius: 6px; border-left: 4px solid #1890ff; flex-shrink: 0;">
      <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 8px;">
        <a-spin size="small" />
        <span style="font-size: 14px; color: #1d2129; font-weight: 500;">智能重试检查中...</span>
      </div>
      <div style="font-size: 12px; color: #86909c;">
        正在检查失败交易的链上状态，判断是否需要重试
      </div>
    </div>

    <!-- 智能重试结果显示 -->
    <div v-if="retryResults.length > 0 && !retryInProgress"
         style="margin-top: 10px; padding: 10px; background: #f6ffed; border-radius: 6px; border-left: 4px solid #52c41a; flex-shrink: 0;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
        <span style="font-size: 14px; color: #1d2129; font-weight: 500;">智能重试检查完成</span>
        <a-button size="mini" type="text" @click="retryResults = []">
          <template #icon>
            <Icon icon="mdi:close" />
          </template>
        </a-button>
      </div>
      <div style="font-size: 12px; color: #52c41a; margin-bottom: 4px;">
        跳过重试: {{retryResults.filter(r => r.action === '跳过重试').length}} 笔 |
        加入重试: {{retryResults.filter(r => r.action === '加入重试').length}} 笔
      </div>
    </div>

    <!-- 管理代币按钮嵌入 -->
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
          <Icon icon="mdi:coin" />
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

    <!-- 细节配置 -->
    <div style="display: flex; padding-top: 5px; flex-shrink: 0;">
      <!-- 细节配置 -->
      <a-form ref="formRef" :model="form" :style="{ width: '100%' }" layout="vertical">
        <a-row style="height: 70px; display: flex;">
          <a-form-item field="send_type" label="发送方式" style="width: 330px;">
            <a-radio-group v-model="form.send_type" type="button">
              <a-radio value="1">全部</a-radio>
              <a-radio value="2">指定数量</a-radio>
              <a-radio value="3">范围随机</a-radio>
              <a-radio value="4">剩余数量</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item v-if="form.send_type === '2'" field="amount_from" label="数量来源" tooltip="如果选择表格数据则应导入带有转账数量的表格数据"
                       style="width: 190px;">
            <a-radio-group v-model="form.amount_from" type="button">
              <a-radio value="1">表格数据</a-radio>
              <a-radio value="2">当前指定</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item v-if="form.send_type === '2' && form.amount_from === '2'" field="send_count" label="发送数量"
                       style="width: 150px;">
            <a-input v-model="form.send_count" />
          </a-form-item>
          <a-form-item v-if="form.send_type === '3' || form.send_type === '4'" field="send_count_scope"
                       :label="form.send_type === '3' ? '发送数量范围' : '剩余数量范围'" style="width: 180px;">
            <a-input v-model="form.send_min_count" />
            <span style="padding: 0 5px">至</span>
            <a-input v-model="form.send_max_count" />
          </a-form-item>
          <a-form-item v-if="form.send_type === '3' || form.send_type === '4'" field="amount_precision" label="金额精度"
                       style="width: 110px;" tooltip="金额小数点位数">
            <a-input v-model="form.amount_precision" />
          </a-form-item>
          <a-divider direction="vertical" style="height: 50px; margin: 15px 10px 0 10px;" />
          <a-form-item field="interval_scope" label="发送间隔（秒）" style="width: 215px;">
            <a-input v-model="form.min_interval" :disabled="threadCount > 1" />
            <span style="padding: 0 5px">至</span>
            <a-input v-model="form.max_interval" :disabled="threadCount > 1" />
          </a-form-item>
          <a-form-item field="thread_count" label="线程数" style="width: 130px;" tooltip="同时执行的钱包数量">
            <a-input-number v-model="threadCount" :min="1" :max="99" :step="1" :default-value="1" mode="button" />
          </a-form-item>
          <a-form-item field="error_retry" label="失败自动重试" style="width: 125px;" tooltip="转账失败时是否自动重试">
            <a-switch v-model="form.error_retry" checked-value="1" unchecked-value="0" />
          </a-form-item>
          <a-divider direction="vertical" style="height: 50px; margin: 15px 10px 0 10px;" />
          <a-form-item field="multi_window" label="窗口多开" style="width: 100px;" tooltip="窗口配置相同">
            <a-input-group style="width: 100px;">
              <a-input-number v-model="multiWindowCount" :min="1" :max="9" :step="1" :default-value="1"
                              placeholder="窗口数" style="width: 50px;" />
              <a-button status="success" @click="debouncedOpenMultipleWindow">
                <template #icon>
                  <Icon icon="mdi:content-copy" />
                </template>
              </a-button>
            </a-input-group>
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
        <a-dropdown v-if="!balanceLoading && balanceStopStatus">
          <a-button type="primary" :style="{ width: '130px', height: '40px', fontSize: '14px' }">
            <template #icon>
              <Icon icon="mdi:magnify" />
            </template>
            查询余额
          </a-button>
          <template #content>
            <a-doption @click="debouncedQueryBalance">
              <Icon icon="mdi:export" style="margin-right: 8px;margin-bottom: -2px;" />
              查出账地址
            </a-doption>
            <a-doption @click="debouncedQueryToAddressBalance">
              <Icon icon="mdi:import" style="margin-right: 8px;margin-bottom: -2px;" />
              查到账地址
            </a-doption>
          </template>
        </a-dropdown>
        <a-tooltip v-else content="点击可以提前停止查询">
          <div @click="debouncedStopBalanceQuery">
            <a-button v-if="!balanceStopFlag" class="execute-btn executing" loading
                      :style="{ width: '130px', height: '40px', fontSize: '14px' }">
              <template #icon>
                <Icon icon="mdi:stop" />
              </template>
              查询中...
            </a-button>
          </div>
        </a-tooltip>
        <a-button v-if="balanceStopFlag && !balanceStopStatus" class="execute-btn stopping" loading
                  :style="{ width: '130px', height: '40px', fontSize: '14px' }">
          <template #icon>
            <Icon icon="mdi:stop" />
          </template>
          正在停止...
        </a-button>
      </div>

      <!-- 右侧区域 -->
      <div style="display: flex; align-items: center; gap: 20px;">
        <!-- 执行转账按钮 -->
        <a-button v-if="!startLoading && stopStatus" type="success" class="execute-btn"
                  @click="debouncedStartTransfer" :style="{ width: '130px', height: '40px', fontSize: '14px' }">
          <template #icon>
            <Icon icon="mdi:play" />
          </template>
          执行转账
        </a-button>
        <a-tooltip v-else content="点击可以提前停止执行">
          <div @click="debouncedStopTransfer">
            <a-button v-if="!stopFlag" class="execute-btn executing" loading
                      :style="{ width: '130px', height: '40px', fontSize: '14px' }">
              <template #icon>
                <Icon icon="mdi:stop" />
              </template>
              执行中...
            </a-button>
            <a-button v-if="stopFlag && !stopStatus" class="execute-btn stopping" loading
                      :style="{ width: '130px', height: '40px', fontSize: '14px' }">
              <template #icon>
                <icon-stop />
              </template>
              正在停止...
            </a-button>
          </div>
        </a-tooltip>
      </div>
    </div>
  </div>
  <!-- 钱包信息录入弹窗 -->
  <WalletImportModal ref="walletImportRef" @confirm="handleWalletImportConfirm" @cancel="handleWalletImportCancel" />
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
      {{ currentItemPrivateKey.substring(0, 15) + "......" }}
      】的数据？
    </div>
    <template #footer>
      <a-button @click="deleteItemCancel">取消</a-button>
      <a-button type="primary" status="danger" @click="debouncedDeleteItemConfirm" style="margin-left: 10px">确定
      </a-button>
    </template>
  </a-modal>

  <!-- 转账确认弹窗 -->
  <a-modal
      v-model:visible="transferConfirmVisible"
      title="转账确认"
      :mask-closable="false"
      :closable="true"
      @close="handleTransferConfirmClose"
      @cancel="handleTransferConfirmClose"
  >
    <div>检测到上次转账未完成，请选择操作方式：</div>
    <template #footer>
      <a-button @click="handleTransferConfirmClose">关闭</a-button>
      <a-button
          type="primary"
          @click="handleTransferConfirmCancel"
          :loading="transferConfirmLoading"
          style="margin-left: 10px"
      >
        重新开始转账
      </a-button>
      <a-button
          type="primary"
          status="success"
          @click="handleTransferConfirmOk"
          :loading="transferConfirmLoading"
          style="margin-left: 10px"
      >
        继续上次转账
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
          <a-input v-model="filterForm.platBalanceValue" placeholder="请输入平台币余额值" style="flex: 1;"
                   @input="debouncedFilterUpdate" />
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
          <a-input v-model="filterForm.coinBalanceValue" placeholder="请输入代币余额值" style="flex: 1;"
                   @input="debouncedFilterUpdate" />
        </div>
      </a-form-item>

      <!-- 错误信息模糊匹配 -->
      <a-form-item label="错误信息">
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
  margin-top: 45px;
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
  min-height: 400px;
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
  background-color: #a1970b;
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

.theme-icon {
  font-size: 16px;
  transition: transform 0.3s ease;
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

.progressBar {
  color: var(--text-color, #1d2129);
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

