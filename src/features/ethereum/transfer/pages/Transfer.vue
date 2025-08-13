<script setup name="transfer">
import { Icon } from '@iconify/vue';
import { useRouter, useRoute } from "vue-router";
import { IconDelete } from '@arco-design/web-vue/es/icon';
import { computed, defineAsyncComponent, onBeforeMount, onBeforeUnmount, onMounted, reactive, ref, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Notification, Modal } from "@arco-design/web-vue";
import { ethers } from "ethers";

import { read, utils as xlUtils, writeFile } from "xlsx";
import { debounce as customDebounce } from '@/utils/debounce.js'
import ChainIcon from '@/components/ChainIcon.vue'
import TitleBar from '@/components/TitleBar.vue'
import TableSkeleton from '@/components/TableSkeleton.vue'
import VirtualScrollerTable from '@/components/VirtualScrollerTable.vue'
import * as party from 'party-js'

// 懒加载非关键组件
const ChainManagement = defineAsyncComponent(() => import('@/components/ChainManagement.vue'))
const RpcManagement = defineAsyncComponent(() => import('@/components/RpcManagement.vue'))
const TokenManagement = defineAsyncComponent(() => import('@/components/TokenManagement.vue'))
const WalletImportModal = defineAsyncComponent(() => import('@/components/WalletImportModal.vue'))
const router = useRouter();
// 窗口标题
const windowTitle = ref('批量转账');
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
// 查询余额中途停止
let balanceStopFlag = ref(false);
// 查询余额是否已经停止
let balanceStopStatus = ref(true);
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
  amount_precision: "3",
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
// 高级筛选相关变量
const advancedFilterVisible = ref(false);
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
// 转账会话是否完全结束（用于区分正常完成和中断）
let transferSessionCompleted = ref(true);
// 转账确认弹窗相关变量
const transferConfirmVisible = ref(false);
const transferConfirmLoading = ref(false);
// 线程数设置，默认为1
let threadCount = ref(1);
// 多窗口数量设置，默认为1
let multiWindowCount = ref(1);

// 庆祝和打赏相关变量
const showCelebration = ref(false);
const showTipModal = ref(false);
const tipAmount = ref('');
const tipPrivateKey = ref(''); // 用户输入的私钥
const tipLoading = ref(false);
const developerAddress = ref('0x298E1bE50Ba5f50CF23cFA6b5F1dF347cFBef40A'); // 开发者收款地址
const tipAmountOptions = ['0.001', '0.005', '0.01', '0.05', '0.1']; // 预设打赏金额选项

// 数据验证缓存 - 避免重复验证
const dataValidationCache = ref({
  lastDataLength: 0,
  lastFormState: '',
  isValid: false,
  invalidReason: '',
  cacheTime: 0
});

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

// 余额查询进度相关变量
const balanceProgress = ref(0); // 余额查询进度百分比
const balanceTotal = ref(0); // 总查询数量
const balanceCompleted = ref(0); // 已完成查询数量
const showBalanceProgress = ref(false); // 是否显示余额查询进度条

// 查到账地址余额查询进度相关变量
const toAddressBalanceProgress = ref(0); // 查到账地址余额查询进度百分比
const toAddressBalanceTotal = ref(0); // 查到账地址总查询数量
const toAddressBalanceCompleted = ref(0); // 查到账地址已完成查询数量
const showToAddressBalanceProgress = ref(false); // 是否显示查到账地址余额查询进度条

// 计算属性：缓存转账配置
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
    amount_precision: form.amount_precision && form.amount_precision.trim() !== '' ? Number(form.amount_precision) : 6,
    limit_type: form.limit_type,
    limit_count: form.limit_count && form.limit_count.trim() !== '' ? Number(form.limit_count) : 21000,
    limit_count_list: [
      form.limit_min_count && form.limit_min_count.trim() !== '' ? Number(form.limit_min_count) : 21000,
      form.limit_max_count && form.limit_max_count.trim() !== '' ? Number(form.limit_max_count) : 30000
    ],
    gas_price_type: form.gas_price_type,
    gas_price_rate: form.gas_price_rate && form.gas_price_rate.trim() !== '' ? Number(form.gas_price_rate) / 100 : 0.05,
    gas_price: form.gas_price && form.gas_price.trim() !== '' ? Number(form.gas_price) : 30,
    max_gas_price: form.max_gas_price && form.max_gas_price.trim() !== '' ? Number(form.max_gas_price) : 0
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

// Gas价格监控相关变量
const gasPriceMonitoring = ref(false); // 是否正在监控gas价格
const gasPriceCountdown = ref(0); // gas价格查询倒计时
const currentGasPrice = ref(0); // 当前gas价格
const gasPriceTimer = ref(null); // gas价格监控定时器
const transferPaused = ref(false); // 转账是否因gas价格过高而暂停
const pausedTransferData = ref(null); // 暂停时的转账数据
const pausedTransferIndex = ref(0); // 暂停时的转账索引

// 窗口多开相关函数
function openMultipleWindow() {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (!isTauri) {
    Notification.warning('此功能仅在桌面应用中可用');
    return;
  }

  try {
    // 获取要打开的窗口数量
    const windowCount = multiWindowCount.value;
    if (windowCount < 1 || windowCount > 9) {
      Notification.warning('窗口数量必须在1-9之间');
      return;
    }

    // 收集当前窗口的配置数据
    const currentConfig = {
      chainValue: chainValue.value,
      coinValue: coinValue.value,
      form: { ...form },
      threadCount: threadCount.value,
      data: data.value.map(item => ({ ...item })) // 深拷贝数据
    };

    // 创建多个窗口
    const baseTimestamp = new Date().toISOString().replace(/[:.]/g, '').slice(0, 17); // 格式化为年月日时分秒毫秒
    const configKeys = [];

    // 为每个窗口创建唯一的configKey
    for (let i = 0; i < windowCount; i++) {
      const windowId = baseTimestamp + i;
      const configKey = `transfer_config_${windowId}`;
      configKeys.push({
        configKey,
        windowId,
        windowLabel: `${getCurrentWindow().label}_multi_${windowId}`
      });

      // 将配置存储到localStorage中，使用唯一key
      localStorage.setItem(configKey, JSON.stringify(currentConfig));
    }

    // 打开所有窗口
    let openedCount = 0;
    let errorCount = 0;

    for (const { configKey, windowId, windowLabel } of configKeys) {
      const windowUrl = `/#/transfer?configKey=${configKey}`; // 通过URL参数传递配置key

      // 打开新窗口
      const webview = new WebviewWindow(windowLabel, {
        url: windowUrl,
        title: `（多开窗口）批量转账 ${windowId}`,
        width: 1350,
        height: 900,
        // center: true,
        resizable: true,
        decorations: false,  // 移除Windows原生窗口边框
        backgroundColor: document.documentElement.getAttribute('data-theme') === 'light' ? '#FFFFFF' : '#2A2A2B',  // 根据当前主题动态设置窗口背景色
        skipTaskbar: false
      });

      webview.once('tauri://created', () => {
        openedCount++;
        Notification.success(`已打开新窗口: 批量转账 ${windowId} (${openedCount}/${windowCount})`);

        // 所有窗口都已打开，清理不需要的configKey
        if (openedCount + errorCount === windowCount) {
          // 所有窗口都已处理完毕，不需要清理localStorage
          // localStorage中的数据会在各窗口读取后自动清理
        }
      });

      webview.once('tauri://error', (e) => {
        errorCount++;
        console.error(`打开窗口 ${windowId} 失败:`, e);
        Notification.error(`打开窗口 ${windowId} 失败`);

        // 清理对应的localStorage数据
        localStorage.removeItem(configKey);

        // 所有窗口都已处理完毕
        if (openedCount + errorCount === windowCount) {
          // 所有窗口都已处理完毕
        }
      });
    }
  } catch (error) {
    console.error('窗口多开失败:', error);
    Notification.error('窗口多开失败');
  }
}

// 应用共享配置
function applySharedConfig(config) {
  if (!config) return;

  try {
    // 应用链选择
    if (config.chainValue) {
      chainValue.value = config.chainValue;
      // 找到对应的链对象
      const chain = chainOptions.value.find(c => c.key === config.chainValue);
      if (chain) {
        currentChain.value = chain;
      }
    }

    // 应用币种选择
    if (config.coinValue) {
      coinValue.value = config.coinValue;
      // 找到对应的币种对象
      const coin = coinOptions.value.find(c => c.key === config.coinValue);
      if (coin) {
        currentCoin.value = coin;
      }
    }

    // 应用表单配置
    if (config.form) {
      Object.assign(form, config.form);
    }

    // 应用线程数
    if (config.threadCount) {
      threadCount.value = config.threadCount;
    }

    // 应用数据
    if (config.data && Array.isArray(config.data)) {
      data.value = config.data.map((item, index) => ({
        ...item,
        key: index + 1 // 重新生成key
      }));
    }

    Notification.success('已应用共享配置');
  } catch (error) {
    console.error('应用共享配置失败:', error);
    Notification.error('应用共享配置失败');
  }
}

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

// 更新余额查询进度
function updateBalanceProgress() {
  if (!showBalanceProgress.value) return;

  // 计算已完成的查询数量（有余额数据或查询失败都算完成）
  const completed = data.value.filter(item =>
    (item.plat_balance !== '' && item.plat_balance !== null) ||
    (item.coin_balance !== '' && item.coin_balance !== null) ||
    item.exec_status === '3'
  ).length;

  balanceCompleted.value = completed;
  // 计算进度百分比
  if (balanceTotal.value > 0) {
    balanceProgress.value = Number((completed / balanceTotal.value).toFixed(4));
  } else {
    balanceProgress.value = 0;
  }

  // 如果全部完成，延迟隐藏进度条
  if (completed === balanceTotal.value && balanceTotal.value > 0) {
    setTimeout(() => {
      showBalanceProgress.value = false;
    }, 3000); // 3秒后隐藏进度条
  }
}

// 更新查到账地址余额查询进度
function updateToAddressBalanceProgress() {
  if (!showToAddressBalanceProgress.value) return;

  // 只计算有到账地址的项目的完成情况
  const itemsWithToAddr = data.value.filter(item => item.to_addr);
  const completed = itemsWithToAddr.filter(item =>
    (item.plat_balance !== '' && item.plat_balance !== null) ||
    (item.coin_balance !== '' && item.coin_balance !== null) ||
    item.exec_status === '3'
  ).length;

  toAddressBalanceCompleted.value = completed;
  // 计算进度百分比
  if (toAddressBalanceTotal.value > 0) {
    toAddressBalanceProgress.value = Number((completed / toAddressBalanceTotal.value).toFixed(4));
  } else {
    toAddressBalanceProgress.value = 0;
  }

  // 如果全部完成，延迟隐藏进度条
  if (completed === toAddressBalanceTotal.value && toAddressBalanceTotal.value > 0) {
    setTimeout(() => {
      showToAddressBalanceProgress.value = false;
    }, 3000); // 3秒后隐藏进度条
  }
}

// 代币管理相关变量
const tokenTableLoading = ref(false);
const tokenManageData = ref([]);

// 智能重试核心函数
async function performIntelligentRetry(failedData) {
  if (!transferStartTime.value) {
    console.warn('未找到转账开始时间，使用传统重试方式');
    transferFnc(failedData);
    return;
  }

  retryInProgress.value = true;
  retryResults.value = [];

  Notification.info(`开始智能重试检查，共 ${failedData.length} 笔失败交易`);

  try {
    // 对每个失败的交易进行检查
    const retryList = [];

    for (const item of failedData) {
      try {
        // 查询该私钥钱包在转账开始时间之后的交易历史
        const hasRecentTransfer = await checkRecentTransfer(item.private_key, item.to_addr, transferStartTime.value);

        if (hasRecentTransfer) {
          // 发现在开始时间之后有包含目标接收地址的交易，不重试
          const realIndex = data.value.findIndex(dataItem => dataItem.key === item.key);
          if (realIndex !== -1) {
            data.value[realIndex].error_msg = '检测到链上已有相关交易，跳过重试';
            data.value[realIndex].exec_status = '2'; // 标记为成功
          }
          retryResults.value.push({
            key: item.key,
            address: item.to_addr,
            action: '跳过重试',
            reason: '检测到链上已有相关交易'
          });
        } else {
          // 没有发现相关交易，加入重试列表
          retryList.push(item);
          retryResults.value.push({
            key: item.key,
            address: item.to_addr,
            action: '加入重试',
            reason: '未检测到相关链上交易'
          });
        }
      } catch (error) {
        console.error(`检查交易失败 ${item.to_addr}:`, error);
        // 检查失败时，保守起见加入重试列表
        retryList.push(item);
        retryResults.value.push({
          key: item.key,
          address: item.to_addr,
          action: '加入重试',
          reason: '检查失败，保守重试'
        });
      }
    }

    retryInProgress.value = false;

    if (retryList.length > 0) {
      Notification.info(`智能重试检查完成，将重试 ${retryList.length} 笔交易，跳过 ${failedData.length - retryList.length} 笔交易`);
      // 执行重试
      transferFnc(retryList);
    } else {
      Notification.success('智能重试检查完成，所有失败交易均检测到链上已有相关交易，无需重试');
      stopStatus.value = true;
      // 标记转账会话完全结束
      transferSessionCompleted.value = true;
    }

  } catch (error) {
    console.error('智能重试检查失败:', error);
    retryInProgress.value = false;
    Notification.error('智能重试检查失败，使用传统重试方式');
    transferFnc(failedData);
  }
}

// 检查指定私钥钱包在指定时间之后是否有包含目标地址的转账交易
async function checkRecentTransfer(privateKey, targetAddress, startTime) {
  try {
    // 调用后端接口查询链上交易历史
    const result = await invoke('check_wallet_recent_transfers', {
      chain: chainValue.value,
      private_key: privateKey,
      target_address: targetAddress.toLowerCase(),
      start_timestamp: startTime,
      coin_type: currentCoin.value.coin_type,
      contract_address: currentCoin.value.coin_type === 'token' ? currentCoin.value.contract_address : null
    });

    return result.has_recent_transfer || false;
  } catch (error) {
    console.error('查询链上交易失败:', error);
    // 查询失败时返回false，让重试逻辑决定
    throw error;
  }
}
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



// 庆祝函数
function triggerCelebration() {
  // 使用party.js创建庆祝效果
  try {
    // 从页面中心发射彩带
    party.confetti(document.body, {
      count: party.variation.range(40, 100),
      spread: party.variation.range(50, 100),
    });

    // 延迟一点再发射第二波
    setTimeout(() => {
      party.sparkles(document.body, {
        count: party.variation.range(20, 40),
      });
    }, 500);

    // 从按钮位置发射
    const executeButton = document.querySelector('.execute-btn');
    if (executeButton) {
      party.confetti(executeButton, {
        count: party.variation.range(20, 40),
        spread: party.variation.range(30, 60),
      });
    }
  } catch (error) {
    console.log('Party.js庆祝效果加载失败:', error);
  }

  // 显示庆祝状态
  showCelebration.value = true;

  // 3秒后隐藏庆祝状态并显示打赏弹窗
  setTimeout(() => {
    showCelebration.value = false;
    showTipModal.value = true;
  }, 3000);
}

// 打赏函数
async function sendTip() {
  if (!tipAmount.value || parseFloat(tipAmount.value) <= 0) {
    Notification.warning('请输入有效的打赏金额');
    return;
  }

  if (!tipPrivateKey.value || !tipPrivateKey.value.trim()) {
    Notification.warning('请输入私钥');
    return;
  }

  // 验证私钥格式
  if (!validatePrivateKey(tipPrivateKey.value.trim())) {
    Notification.warning('私钥格式不正确');
    return;
  }

  tipLoading.value = true;

  try {
    // 从私钥生成地址
    const wallet = new ethers.Wallet(tipPrivateKey.value.trim());
    const fromAddress = wallet.address;

    // 构建转账数据（符合 TransferItem 结构）
    const tipData = {
      private_key: tipPrivateKey.value.trim(),
      to_addr: developerAddress.value,
      error_msg: "",
      error_count: 0,
      retry_flag: false
    };

    // 执行打赏转账 - 使用完整的配置结构
    const config = {
      error_count_limit: 3,
      error_retry: "0",
      chain: chainValue.value,
      chainLayer: currentChain.value.layer,
      l1: currentChain.value.l1,
      scalar: currentChain.value.scalar,
      delay: [1, 3],
      transfer_type: "2", // 指定数量转账
      transfer_amount: parseFloat(tipAmount.value),
      transfer_amount_list: [0, 0], // 随机转账范围（transfer_type为2时不使用）
      left_amount_list: [0, 0], // 剩余数量范围（transfer_type为4时使用）
      amount_precision: 6,
      limit_type: "1", // 自动gas limit
      limit_count: 21000,
      limit_count_list: [21000, 30000], // gas limit范围
      gas_price_type: form.gas_price_type || "3",
      gas_price_rate: form.gas_price_rate && form.gas_price_rate.trim() !== '' ? Number(form.gas_price_rate) / 100 : 0.05,
      gas_price: form.gas_price && form.gas_price.trim() !== '' ? Number(form.gas_price) : 30,
      max_gas_price: form.max_gas_price && form.max_gas_price.trim() !== '' ? Number(form.max_gas_price) : 0,
    };

    let result;
    // 使用特殊的 index 值 999999 来标识打赏转账（usize 类型需要正整数）
    const tipTransferIndex = 999999;

    if (currentCoin.value.coin_type === "base") {
      result = await invoke("base_coin_transfer", {
        index: tipTransferIndex,
        item: tipData,
        config: config
      });
    } else if (currentCoin.value.coin_type === "token") {
      result = await invoke("token_transfer", {
        index: tipTransferIndex,
        item: tipData,
        config: {
          ...config,
          contract_address: currentCoin.value.contract_address,
          abi: currentCoin.value.abi
        }
      });
    }

    // 处理结果
    if (typeof result === 'object' && result !== null) {
      if (result.success && result.tx_hash) {
        Notification.success({
          title: '打赏成功！',
          content: `感谢您的支持！`,
          duration: 5000
        });

        // 再次触发小型庆祝
        try {
          party.sparkles(document.body, {
            count: party.variation.range(10, 20),
          });
        } catch (error) {
          console.log('打赏庆祝效果加载失败:', error);
        }
      } else {
        throw new Error(result.error || '打赏失败');
      }
    } else {
      Notification.success({
        title: '打赏成功！',
        content: '感谢您的支持！',
        duration: 3000
      });
    }

    showTipModal.value = false;
    tipAmount.value = '';
    tipPrivateKey.value = '';

  } catch (error) {
    console.error('打赏失败:', error);
    Notification.error('打赏失败: ' + error.message);
  } finally {
    tipLoading.value = false;
  }
}

// 跳过打赏
function skipTip() {
  showTipModal.value = false;
  tipAmount.value = '';
  tipPrivateKey.value = '';
  Notification.info('感谢您使用本工具！');
}

// 获取成功转账的钱包数量（响应式）
const successfulWallets = computed(() => {
  return data.value.filter(item => item.exec_status === '2');
});

// 验证用户输入的私钥对应的地址余额
const tipWalletBalance = ref({
  valid: false,
  balance: 0,
  address: '',
  loading: false,
  error: null,
  hasAttempted: false // 是否已经尝试过查询
});

// 查询钱包余额的函数
async function queryTipWalletBalance() {
  if (!tipPrivateKey.value || !tipPrivateKey.value.trim()) {
    tipWalletBalance.value = {
      valid: false,
      balance: 0,
      address: '',
      loading: false,
      error: null,
      hasAttempted: false
    };
    return;
  }

  try {
    // 验证私钥格式
    if (!validatePrivateKey(tipPrivateKey.value.trim())) {
      tipWalletBalance.value = {
        valid: false,
        balance: 0,
        address: '',
        error: '私钥格式不正确',
        loading: false,
        hasAttempted: true
      };
      return;
    }

    // 从私钥生成地址
    const wallet = new ethers.Wallet(tipPrivateKey.value.trim());
    const address = wallet.address;

    // 设置加载状态
    tipWalletBalance.value = {
      valid: false,
      balance: 0,
      address,
      loading: true,
      error: null,
      hasAttempted: true
    };

    let balance = 0;

    if (currentCoin.value?.coin_type === "base") {
      // 查询主币余额
      const result = await invoke("query_balance", {
        chain: chainValue.value,
        address: address
      });

      if (typeof result === 'string') {
        balance = parseFloat(result || 0);
      } else if (typeof result === 'number') {
        balance = result;
      }
    } else if (currentCoin.value?.coin_type === "token") {
      // 查询代币余额 - 使用现有的余额查询系统
      const params = {
        chain: chainValue.value,
        coin_config: {
          coin_type: currentCoin.value.coin_type,
          contract_address: currentCoin.value.contract_address || null,
          abi: currentCoin.value.abi || null
        },
        items: [{
          key: address,
          address: address,
          private_key: null,
          plat_balance: null,
          coin_balance: null,
          nonce: null,
          exec_status: '0',
          error_msg: null,
          retry_flag: false
        }],
        only_coin_config: true, // 只查询代币余额
        thread_count: 1
      };

      const result = await invoke('query_balances_simple', { params });

      if (result && result.success && result.items && result.items.length > 0) {
        const item = result.items[0];
        if (item.exec_status === '2') {
          balance = parseFloat(item.coin_balance || 0);
        } else {
          throw new Error(item.error_msg || '代币余额查询失败');
        }
      } else {
        throw new Error('代币余额查询失败');
      }
    }

    tipWalletBalance.value = {
      valid: true,
      balance,
      address,
      loading: false,
      error: null,
      hasAttempted: true,
      sufficient: tipAmount.value ? balance >= parseFloat(tipAmount.value) : true
    };

  } catch (error) {
    console.error('查询打赏钱包余额失败:', error);
    tipWalletBalance.value = {
      valid: false,
      balance: 0,
      address: tipWalletBalance.value.address || '',
      error: '余额查询失败: ' + error.message,
      loading: false,
      hasAttempted: true
    };
  }
}

// 监听私钥变化，自动查询余额
watch(tipPrivateKey, customDebounce(queryTipWalletBalance, 1000));
watch(currentCoin, queryTipWalletBalance);

// 计算余额充足性（响应式）
const tipBalanceSufficient = computed(() => {
  if (!tipWalletBalance.value.valid || !tipAmount.value) return true;
  return tipWalletBalance.value.balance >= parseFloat(tipAmount.value);
});

// 判断是否应该显示私钥验证状态
const shouldShowTipWalletStatus = computed(() => {
  return tipPrivateKey.value && tipPrivateKey.value.trim().length > 0;
});

// 获取gas
const timer = setInterval(fetchGas, 5000);

// Gas价格监控函数
async function startGasPriceMonitoring() {
  if (gasPriceMonitoring.value) return;

  gasPriceMonitoring.value = true;
  gasPriceCountdown.value = 10;

  // 立即检查一次gas价格
  await checkGasPriceForTransfer();

  // 启动定时器，每10秒检查一次
  gasPriceTimer.value = setInterval(async () => {
    gasPriceCountdown.value = 10;
    await checkGasPriceForTransfer();

    // 倒计时
    const countdownInterval = setInterval(() => {
      gasPriceCountdown.value--;
      if (gasPriceCountdown.value <= 0) {
        clearInterval(countdownInterval);
      }
    }, 1000);
  }, 10000);
}

// 停止gas价格监控
function stopGasPriceMonitoring() {
  gasPriceMonitoring.value = false;
  gasPriceCountdown.value = 0;
  if (gasPriceTimer.value) {
    clearInterval(gasPriceTimer.value);
    gasPriceTimer.value = null;
  }
}

// 检查gas价格是否超过限制
async function checkGasPriceForTransfer() {
  if (!form.max_gas_price || !form.max_gas_price.trim()) {
    return true; // 没有设置最大gas价格限制
  }

  const maxGasPrice = Number(form.max_gas_price);
  if (maxGasPrice <= 0) {
    return true; // 无效的最大gas价格设置
  }

  try {
    // 获取当前gas价格
    const res = await invoke("get_chain_gas_price", { chain: chainValue.value });
    const gasPrice = res?.gas_price_gwei || 0;
    currentGasPrice.value = gasPrice;

    if (gasPrice > maxGasPrice) {
      // Gas价格超过限制
      if (!transferPaused.value && !stopFlag.value && startLoading.value) {
        // 暂停转账
        transferPaused.value = true;
        Notification.warning(`Gas价格 ${gasPrice.toFixed(3)} Gwei 超过设定上限 ${maxGasPrice} Gwei，转账已暂停`);
      }
      return false;
    } else {
      // Gas价格在限制范围内
      if (transferPaused.value) {
        // 恢复转账
        transferPaused.value = false;
        stopGasPriceMonitoring();
        Notification.success(`Gas价格 ${gasPrice.toFixed(3)} Gwei 已降至设定范围内，转账将自动恢复`);

        // 恢复转账
        if (pausedTransferData.value) {
          await resumeTransfer();
        }
      }
      return true;
    }
  } catch (error) {
    console.error('获取gas价格失败:', error);
    currentGasPrice.value = 0;
    return true; // 获取失败时不阻止转账
  }
}

// 恢复转账
async function resumeTransfer() {
  if (!pausedTransferData.value) return;

  const { accountData, index } = pausedTransferData.value;
  pausedTransferData.value = null;

  // 从暂停的位置继续执行转账
  await continueTransferFromIndex(accountData, index);
}

// 从指定索引继续转账
async function continueTransferFromIndex(accountData, startIndex) {
  // 判断是主币转账还是代币转账
  let contract;
  if (currentCoin.value.coin_type === "token") {
    contract = new ethers.Contract(
      currentCoin.value.contract_address,
      currentCoin.value.abi
    );
  }

  // 从指定索引开始继续执行转账
  for (let index = startIndex; index < accountData.length; index++) {
    if (stopFlag.value) {
      stopStatus.value = true;
      return;
    }

    const item = accountData[index];

    // 跳过已完成或失败的记录，只处理等待执行的记录
    if (item.exec_status !== '0') {
      continue;
    }

    // 检查gas价格是否超过限制
    if (form.max_gas_price && form.max_gas_price.trim()) {
      const gasPriceOk = await checkGasPriceForTransfer();
      if (!gasPriceOk) {
        // Gas价格超过限制，暂停转账并启动监控
        pausedTransferData.value = { accountData, index };
        await startGasPriceMonitoring();

        // 等待gas价格降低
        while (transferPaused.value && !stopFlag.value) {
          await new Promise(resolve => setTimeout(resolve, 1000));
        }

        // 如果用户手动停止了转账，退出
        if (stopFlag.value) {
          stopStatus.value = true;
          return;
        }
      }
    }

    // 找到该item在原始data.value数组中的真实索引
    const realIndex = data.value.findIndex(dataItem => dataItem.key === item.key);
    if (realIndex === -1) {
      console.error('无法找到对应的数据项');
      continue;
    }
    const config = {
      error_count_limit: 3, //  错误次数限制
      error_retry: form.error_retry, // 是否自动失败重试
      chain: chainValue.value,
      chainLayer: currentChain.value.layer,
      l1: currentChain.value.l1,
      scalar: currentChain.value.scalar,
      delay: [form.min_interval && form.min_interval.trim() !== '' ? Number(form.min_interval) : 1, form.max_interval && form.max_interval.trim() !== '' ? Number(form.max_interval) : 3], // 延迟时间
      transfer_type: form.send_type, // 转账类型 1：全部转账 2:转账固定数量 3：转账随机数量  4：剩余随机数量
      transfer_amount: form.amount_from === '1' ? (item.amount && item.amount.trim() !== '' ? Number(item.amount) : 0) : (form.send_count && form.send_count.trim() !== '' ? Number(form.send_count) : 0), // 转账当前指定的固定金额
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

    try {
      if (currentCoin.value.coin_type === "base") {
        // 设置状态 为执行中
        data.value[realIndex].exec_status = "1";
        try {
          const res = await invoke("base_coin_transfer", {
            index: realIndex + 1,
            item: item,
            config: config
          });
          data.value[realIndex].exec_status = "2";
          // 转账成功时只显示tx_hash
          if (typeof res === 'object' && res !== null) {
            if (res.success && res.tx_hash) {
              data.value[realIndex].error_msg = res.tx_hash;
            } else {
              data.value[realIndex].error_msg = res.error || '转账失败';
            }
          } else {
            data.value[realIndex].error_msg = String(res || '转账成功');
          }
          // 更新进度条
          updateTransferProgress();
        } catch (err) {
          if (err === "base gas price 超出最大值限制") {
            Notification.error("base gas price 超出最大值限制");
            // 停止
            stopTransfer();
            data.value[realIndex].exec_status = "0";
            data.value[realIndex].error_msg = "";
            return;
          } else {
            data.value[realIndex].exec_status = "3";
            data.value[realIndex].error_msg = err;
            // 更新进度条
            updateTransferProgress();
          }
        }
      } else if (currentCoin.value.coin_type === "token") {
        // 设置状态 为执行中
        data.value[realIndex].exec_status = "1";
        try {
          const res = await invoke("token_transfer", {
            index: realIndex + 1,
            item: item,
            config: {
              ...config,
              contract_address: contract.address,
              abi: contract.abi
            }
          });
          data.value[realIndex].exec_status = "2";
          // 转账成功时只显示tx_hash
          if (typeof res === 'object' && res !== null) {
            if (res.success && res.tx_hash) {
              data.value[realIndex].error_msg = res.tx_hash;
            } else {
              data.value[realIndex].error_msg = res.error || '转账失败';
            }
          } else {
            data.value[realIndex].error_msg = String(res || '转账成功');
          }
          // 更新进度条
          updateTransferProgress();
        } catch (err) {
          if (err === "base gas price 超出最大值限制") {
            Notification.error("base gas price 超出最大值限制");
            // 停止
            stopTransfer();
            data.value[realIndex].exec_status = "0";
            data.value[realIndex].error_msg = "";
            return;
          } else {
            data.value[realIndex].exec_status = "3";
            data.value[realIndex].error_msg = err;
            // 更新进度条
            updateTransferProgress();
          }
        }
      } else {
        Notification.error("未知币种类型");
        return;
      }
    } catch (e) {
      // 交易失败
      data.value[realIndex].exec_status = "3";
      data.value[realIndex].error_msg = e.message || '转账异常';
      updateTransferProgress();
    }

    // 添加延迟等待（只在实际执行了转账后才延迟，跳过的记录不延迟）
    if (index < accountData.length - 1 && !stopFlag.value) {
      const minDelay = form.min_interval && form.min_interval.trim() !== '' ? Number(form.min_interval) * 1000 : 1000;
      const maxDelay = form.max_interval && form.max_interval.trim() !== '' ? Number(form.max_interval) * 1000 : 3000;
      const randomDelay = Math.floor(Math.random() * (maxDelay - minDelay + 1)) + minDelay;

      // 找到下一条待执行的数据
      let nextPendingIndex = -1;
      for (let i = index + 1; i < accountData.length; i++) {
        if (accountData[i].exec_status === '0') {
          nextPendingIndex = data.value.findIndex(dataItem => dataItem.key === accountData[i].key);
          break;
        }
      }

      // 如果找到下一条待执行的数据，在其error_msg字段显示倒计时
      if (nextPendingIndex !== -1) {
        const originalErrorMsg = data.value[nextPendingIndex].error_msg;
        let remainingTime = Math.ceil(randomDelay / 1000);

        // 每秒更新倒计时
        const countdownInterval = setInterval(() => {
          if (stopFlag.value) {
            clearInterval(countdownInterval);
            // 恢复原始错误信息
            data.value[nextPendingIndex].error_msg = originalErrorMsg;
            return;
          }

          data.value[nextPendingIndex].error_msg = `等待中...${remainingTime}秒`;
          remainingTime--;

          if (remainingTime < 0) {
            clearInterval(countdownInterval);
            // 恢复原始错误信息
            data.value[nextPendingIndex].error_msg = originalErrorMsg;
          }
        }, 1000);

        await new Promise(resolve => {
          setTimeout(() => {
            clearInterval(countdownInterval);
            resolve();
          }, randomDelay);
        });
      } else {
        // 没有找到下一条待执行的数据，直接延迟
        await new Promise(resolve => setTimeout(resolve, randomDelay));
      }
    }
  }

  // 转账完成
  startLoading.value = false;
  stopStatus.value = true;
  // 标记转账会话完全结束
  transferSessionCompleted.value = true;
}

watch(stopStatus, (newValue, oldValue) => {
  // 停止状态变化监听
});



// 初始化RPC列表
onBeforeMount(async () => {
  // 检查是否在Tauri环境中
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;

  // 从URL参数获取配置key并读取共享配置
  let sharedConfig = null;
  const route = useRoute();
  const configKey = route.query.configKey;
  if (configKey) {
    try {
      const configData = localStorage.getItem(configKey);
      if (configData) {
        sharedConfig = JSON.parse(configData);
        // 读取后立即清除localStorage中的数据
        localStorage.removeItem(configKey);
        console.log('从localStorage读取到共享配置:', sharedConfig);
      }
    } catch (error) {
      console.error('读取共享配置失败:', error);
    }
  }
  if (isTauri) {
    // 初始化加载链列表
    try {
      const result = await invoke('get_chain_list');
      chainOptions.value = result || [];

      // 按照name字段排序
      chainOptions.value.sort((a, b) => {
        const nameA = a.name || '';
        const nameB = b.name || '';
        return nameA.localeCompare(nameB);
      });

      // 如果有共享配置，优先使用共享配置
      if (sharedConfig) {
        // 应用链选择
        if (sharedConfig.chainValue) {
          chainValue.value = sharedConfig.chainValue;
          const chain = chainOptions.value.find(c => c.key === sharedConfig.chainValue);
          if (chain) {
            currentChain.value = chain;
          }
        }

        // 应用表单配置
        if (sharedConfig.form) {
          Object.assign(form, sharedConfig.form);
        }

        // 应用线程数
        if (sharedConfig.threadCount) {
          threadCount.value = sharedConfig.threadCount;
        }

        // 获取对应的代币列表
        await chainChange();

        // 应用币种选择（需要在chainChange之后）
        if (sharedConfig.coinValue) {
          coinValue.value = sharedConfig.coinValue;
          const coin = coinOptions.value.find(c => c.key === sharedConfig.coinValue);
          if (coin) {
            currentCoin.value = coin;
          }
        }

        // 应用数据
        if (sharedConfig.data && Array.isArray(sharedConfig.data)) {
          data.value = sharedConfig.data.map((item, index) => ({
            ...item,
            key: index + 1 // 重新生成key
          }));
        }
      } else {
        // 没有共享配置时设置默认值
        if (chainOptions.value.length > 0) {
          chainValue.value = chainOptions.value[0].key;
          currentChain.value = chainOptions.value[0];
          // 获取对应的代币列表
          await chainChange();
        }
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

    // 按照name字段排序
    chainOptions.value.sort((a, b) => {
      const nameA = a.name || '';
      const nameB = b.name || '';
      return nameA.localeCompare(nameB);
    });

    // 如果有共享配置，优先使用共享配置
    if (sharedConfig) {
      // 应用链选择
      if (sharedConfig.chainValue) {
        chainValue.value = sharedConfig.chainValue;
        const chain = chainOptions.value.find(c => c.key === sharedConfig.chainValue);
        if (chain) {
          currentChain.value = chain;
        }
      }

      // 应用表单配置
      if (sharedConfig.form) {
        Object.assign(form, sharedConfig.form);
      }

      // 应用线程数
      if (sharedConfig.threadCount) {
        threadCount.value = sharedConfig.threadCount;
      }

      // 获取对应的代币列表
      await chainChange();

      // 应用币种选择（需要在chainChange之后）
      if (sharedConfig.coinValue) {
        coinValue.value = sharedConfig.coinValue;
        const coin = coinOptions.value.find(c => c.key === sharedConfig.coinValue);
        if (coin) {
          currentCoin.value = coin;
        }
      }

      // 应用数据
      if (sharedConfig.data && Array.isArray(sharedConfig.data)) {
        data.value = sharedConfig.data.map((item, index) => ({
          ...item,
          key: index + 1 // 重新生成key
        }));
      }
    } else {
      // 没有共享配置时设置默认值
      chainValue.value = chainOptions.value[0].key;
      currentChain.value = chainOptions.value[0];
      // 获取rpc对应的代币列表
      await chainChange();
    }
  }
});

onMounted(async () => {
  // 获取窗口标题和ID
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow();
      const title = await currentWindow.title();
      if (title) {
        windowTitle.value = title;
      }

      // 获取当前窗口ID
      currentWindowId.value = currentWindow.label;


    } catch (error) {
      console.error('Error getting window title or setting close listener:', error);
    }
  } else {
    // 浏览器环境下设置默认标题和ID
    windowTitle.value = '批量转账';
    currentWindowId.value = 'browser_transfer_window';
  }

  // 配置应用已经在onBeforeMount中完成，这里不再需要重复应用

  // 页面高度现在通过 CSS 自动调整，无需监听器

  // 监听余额查询更新事件
  const isTauriMounted = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauriMounted) {
    await listen('balance_item_update', (event) => {
      const { item, window_id } = event.payload;
      // 检查是否是本窗口的事件
      if (window_id && window_id !== currentWindowId.value) {
        return; // 不是本窗口的事件，直接返回
      }
      // 使用address查找对应的数据项，而不是使用index
      const targetIndex = data.value.findIndex(dataItem => dataItem.key === item.key);
      if (targetIndex !== -1) {
        // 更新对应地址的数据（只更新指定字段）
        data.value[targetIndex].plat_balance = item.plat_balance;
        data.value[targetIndex].coin_balance = item.coin_balance;
        data.value[targetIndex].exec_status = item.exec_status;
        data.value[targetIndex].error_msg = item.error_msg;
        
        // 实时更新余额查询进度
        updateBalanceProgress();
        updateToAddressBalanceProgress();
      }
    });

    // 监听转账状态更新事件
    await listen('transfer_status_update', (event) => {
      const { index, error_msg, exec_status, item } = event.payload;

      // 检查是否是打赏转账（通过特殊 index 值识别）
      if (index === 999999) {
        // 这是打赏转账，不更新主表格数据
        console.log('打赏转账状态更新:', { index, error_msg, exec_status });
        return;
      }

      // 使用private_key查找对应的数据项，而不是使用index
      let targetIndex = -1;
      if (item && item.private_key) {
        targetIndex = data.value.findIndex(dataItem => dataItem.private_key === item.private_key);
      } else {
        // 如果没有item信息，仍然使用index作为备用方案
        targetIndex = index;
      }

      if (targetIndex !== -1 && data.value[targetIndex]) {
        // 更新对应数据项的状态和返回信息
        data.value[targetIndex].error_msg = error_msg;
        data.value[targetIndex].exec_status = exec_status;

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

onBeforeUnmount(async () => {
  // 停止转账操作
  if (startLoading.value) {
    stopFlag.value = true;
    startLoading.value = false;
    stopStatus.value = true;
    Notification.warning('窗口关闭，已自动停止转账操作');
  }

  // 停止gas价格监控
  stopGasPriceMonitoring();

  // 清理所有定时器
  if (timer) {
    clearInterval(timer);
  }
  if (gasPriceTimer.value) {
    clearInterval(gasPriceTimer.value);
    gasPriceTimer.value = null;
  }

  // 重置相关状态
  transferPaused.value = false;
  pausedTransferData.value = null;
  gasPriceMonitoring.value = false;
  gasPriceCountdown.value = 0;
  currentGasPrice.value = 0;

  console.log('Transfer页面清理完成，所有后台操作已停止');
});

// 导入进度相关变量
const importProgress = ref(0); // 导入进度百分比
const importTotal = ref(0); // 总导入数量
const importCompleted = ref(0); // 已完成导入数量
const showImportProgress = ref(false); // 是否显示导入进度条
const importProgressText = ref(''); // 导入进度文本

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

// 更新导入进度
function updateImportProgress() {
  if (!showImportProgress.value) return;

  // 计算进度百分比
  if (importTotal.value > 0) {
    importProgress.value = Number((importCompleted.value / importTotal.value).toFixed(4));
  } else {
    importProgress.value = 0;
  }

  // 如果全部完成，延迟隐藏进度条
  if (importCompleted.value === importTotal.value && importTotal.value > 0) {
    setTimeout(() => {
      showImportProgress.value = false;
    }, 1000); // 2秒后隐藏进度条
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

// 异步分批处理数据的函数
async function processBatchData(batchData, batchIndex, totalBatches) {
  const validItems = [];
  const invalidItems = [];

  for (let i = 0; i < batchData.length; i++) {
    const item = batchData[i];
    const rowNumber = item._originalIndex + 2; // Excel行号（从第2行开始，第1行是表头）
    const privateKey = String(item.私钥 || '').trim();
    const toAddress = String(item.地址 || '').trim();
    const amount = item.转账数量;

    // 验证私钥和地址
    const isPrivateKeyValid = privateKey && validatePrivateKey(privateKey);
    const isAddressValid = toAddress && validateAddress(toAddress);

    if (isPrivateKeyValid && isAddressValid) {
      // 数据合规，添加到表格
      try {
        // 从私钥生成地址
        const wallet = new ethers.Wallet(privateKey);
        const address = wallet.address;

        validItems.push({
          key: `transfer_${validItems.length}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
          private_key: privateKey,
          address: address,
          to_addr: toAddress,
          amount: amount ? String(amount) : "0", // 转账数量为空时显示为0
          plat_balance: "",
          coin_balance: "",
          exec_status: "0",
          error_msg: "",
        });
      } catch (error) {
        // 私钥无效，添加到不合规数据
        invalidItems.push({
          私钥: privateKey,
          地址: toAddress,
          转账数量: amount || '',
          错误原因: '私钥无效',
          行号: rowNumber
        });
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

      invalidItems.push({
        私钥: privateKey,
        地址: toAddress,
        转账数量: amount || '',
        错误原因: errorReasons.join('; '),
        行号: rowNumber
      });
    }

    // 更新进度
    importCompleted.value++;
    updateImportProgress();

    // 每处理10条数据就让出一次控制权，避免阻塞UI
    if (i % 10 === 0) {
      await new Promise(resolve => setTimeout(resolve, 0));
    }
  }

  return { validItems, invalidItems };
}

function UploadFile() {
  // 检查是否有文件被选择
  if (!uploadInputRef.value.files || !uploadInputRef.value.files[0]) {
    return; // 没有文件被选择，直接返回
  }

  // 开启全页面loading
  pageLoading.value = true;
  tableLoading.value = true;

  // 添加100毫秒延迟，确保loading窗口显示
  setTimeout(async () => {
    try {
      let file = uploadInputRef.value.files[0];
      let reader = new FileReader();

      // 清空现有数据
      data.value = [];
      clearValidationCache(); // 清除验证缓存

      // 读取文件
      reader.readAsArrayBuffer(file);

      reader.onload = async function () {
        try {
          const buffer = reader.result;
          const bytes = new Uint8Array(buffer);
          const length = bytes.byteLength;
          let binary = "";
          for (let i = 0; i < length; i++) {
            binary += String.fromCharCode(bytes[i]);
          }

          // 转换二进制
          const wb = read(binary, {
            type: "binary",
          });
          const outdata = xlUtils.sheet_to_json(wb.Sheets[wb.SheetNames[0]]);

          // 为每个数据项添加原始索引
          const indexedData = outdata.map((item, index) => ({
            ...item,
            _originalIndex: index
          }));

          // 初始化进度
          importTotal.value = indexedData.length;
          importCompleted.value = 0;
          importProgress.value = 0;
          importProgressText.value = '正在处理数据...';
          showImportProgress.value = true;

          // 用于存储所有结果
          const allValidData = [];
          const allInvalidData = [];

          // 分批处理数据，每批处理50条
          const batchSize = 50;
          const totalBatches = Math.ceil(indexedData.length / batchSize);

          for (let i = 0; i < totalBatches; i++) {
            const start = i * batchSize;
            const end = Math.min(start + batchSize, indexedData.length);
            const batchData = indexedData.slice(start, end);

            // 处理当前批次
            const { validItems, invalidItems } = await processBatchData(batchData, i, totalBatches);

            // 累积结果
            allValidData.push(...validItems);
            allInvalidData.push(...invalidItems);

            // 更新进度文本
            importProgressText.value = `正在处理数据...`;
          }

          // 数据处理完成，更新进度文本
          importProgressText.value = '数据处理完成，正在渲染表格...';

          // 等待一个微任务，确保进度更新显示
          await new Promise(resolve => setTimeout(resolve, 100));

          // 重新生成key确保唯一性
          const finalValidData = allValidData.map((item, index) => ({
            ...item,
            key: index + 1
          }));

          // 一次性更新数据，触发表格渲染
          data.value = finalValidData;

          // 处理不合规数据
          if (allInvalidData.length > 0) {
            exportInvalidData(allInvalidData);

            // 显示导入结果通知
            if (allValidData.length > 0) {
              Notification.warning({
                title: '导入完成',
                content: `成功导入 ${allValidData.length} 条数据，${allInvalidData.length} 条不合规数据已导出到本地文件`,
                duration: 5000
              });
            } else {
              Notification.error({
                title: '导入失败',
                content: `所有数据都不合规，共 ${allInvalidData.length} 条数据已导出到本地文件`,
                duration: 5000
              });
            }
          } else {
            // 全部数据合规
            Notification.success({
              title: '导入成功',
              content: `成功导入 ${allValidData.length} 条数据`,
              duration: 3000
            });
          }

        } catch (error) {
          console.error('文件处理失败:', error);
          Notification.error({
            title: '文件处理失败',
            content: '文件处理过程中发生错误，请检查文件格式是否正确',
            duration: 5000
          });
        } finally {
          // 关闭loading
          tableLoading.value = false;
          pageLoading.value = false;

          // 延迟隐藏导入进度条
          setTimeout(() => {
            showImportProgress.value = false;
          }, 2000);
        }
      };

      reader.onerror = function () {
        tableLoading.value = false;
        pageLoading.value = false;
        showImportProgress.value = false;

        Notification.error({
          title: '文件读取失败',
          content: '文件读取过程中发生错误，请检查文件格式是否正确',
          duration: 5000
        });
      };

    } catch (error) {
      console.error('导入文件失败:', error);
      tableLoading.value = false;
      pageLoading.value = false;
      showImportProgress.value = false;

      Notification.error({
        title: '导入失败',
        content: '导入过程中发生错误，请重试',
        duration: 5000
      });
    } finally {
      // 清空文件输入
      if (uploadInputRef.value) {
        uploadInputRef.value.value = '';
      }
    }
  }, 100);
}

const uploadInputRef = ref(null);

// 点击上传文件
function upload() {
  uploadInputRef.value.click();
}

// 下载模板文件
const downloadFile = customDebounce(() => {
  let a = document.createElement("a");
  a.href = `/template/import_model.xlsx`;
  a.download = "导入模板.xlsx";
  a.click();
}, 1000);

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

        // 对代币列表进行排序：base coin排在第一位，其他代币按label字母顺序排序
        coinOptions.value.sort((a, b) => {
          // base coin (coin_type为'base') 排在第一位
          if (a.coin_type === 'base' && b.coin_type !== 'base') {
            return -1;
          }
          if (a.coin_type !== 'base' && b.coin_type === 'base') {
            return 1;
          }
          // 如果都是base coin或都不是base coin，按label字母顺序排序
          return a.label.localeCompare(b.label);
        });

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
          { key: 'eth', label: 'ETH', symbol: 'ETH', coin_type: 'base', decimals: 18 },
          { key: 'usdt', label: 'USDT', symbol: 'USDT', coin_type: 'token', contract_address: '0x...', decimals: 6 }
        ];

        // 对代币列表进行排序：base coin排在第一位，其他代币按label字母顺序排序
        coinOptions.value.sort((a, b) => {
          // base coin (coin_type为'base') 排在第一位
          if (a.coin_type === 'base' && b.coin_type !== 'base') {
            return -1;
          }
          if (a.coin_type !== 'base' && b.coin_type === 'base') {
            return 1;
          }
          // 如果都是base coin或都不是base coin，按label字母顺序排序
          return a.label.localeCompare(b.label);
        });

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
  if (startLoading.value) {
    Notification.warning('请停止或等待转账完成后再清空列表！');
    return;
  }
  if (balanceLoading.value) {
    Notification.warning("请停止或等待查询完成后再清空列表！");
    return;
  }
  if (data.value.length === 0) {
    Notification.warning('当前列表无数据！');
    return;
  }

  Modal.confirm({
    title: '确认清空',
    content: '确定要清空所有列表数据吗？此操作不可撤销。',
    onOk: () => {
      data.value = [];
      clearValidationCache(); // 清除验证缓存
      // 重置转账执行标识
      hasExecutedTransfer.value = false;
      // 重置文件输入的value，确保再次选择相同文件时能触发change事件
      if (uploadInputRef.value) {
        uploadInputRef.value.value = '';
      }
      // 重置页面loading状态
      pageLoading.value = false;
      Notification.success("清空列表成功！");
    }
  });
}

// 导入事件触发
function handleClick() {
  if (walletImportRef.value) {
    walletImportRef.value.show();
  }
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
        key: `transfer_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
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
  clearValidationCache(); // 清除验证缓存

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


// 删除数据
function deleteItem(item) {
  if (startLoading.value) {
    Notification.warning("请停止或等待执行完成后再删除数据！");
    return;
  }
  // 删除确认
  deleteItemModalShow(item);
}

// 删除数据弹窗显示
function deleteItemModalShow(item) {
  deleteItemVisible.value = true;
  currentItemKey.value = item.key;
  currentItemPrivateKey.value = item.private_key || "";
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

// 当前窗口ID
let currentWindowId = ref('');

// 创建防抖版本的操作函数
const debouncedQueryBalance = customDebounce(queryBalance, 500);
const debouncedQueryToAddressBalance = customDebounce(queryToAddressBalance, 500);
const debouncedStartTransfer = customDebounce(startTransfer, 800);
const debouncedStopBalanceQuery = customDebounce(stopBalanceQuery, 300);
const debouncedStopTransfer = customDebounce(stopTransfer, 300);
const debouncedHandleClick = customDebounce(handleClick, 500);
const debouncedClearData = customDebounce(clearData, 600);
const debouncedDeleteItemConfirm = customDebounce(deleteItemConfirm, 400);
const debouncedOpenMultipleWindow = customDebounce(openMultipleWindow, 600);

// 查询余额 - 支持分批处理
async function queryBalance() {
  if (!stopStatus.value) {
    Notification.warning("请停止或等待执行完成后再查询余额！");
    return;
  }
  if (!balanceStopStatus.value) {
    Notification.warning("请停止或等待查询完成后再查询余额！");
    return;
  }
  if (data.value.length === 0) {
    Notification.warning("请先导入私钥！");
    return;
  }
  hasExecutedTransfer.value = false;
  transferTotal.value = data.value.length;
  transferCompleted.value = 0;
  transferProgress.value = 0;
  if (currentCoin.value.coin_type === "base" || currentCoin.value.coin_type === "token") {
    balanceLoading.value = true;
    balanceStopFlag.value = false;
    balanceStopStatus.value = false;

    // 初始化余额查询进度
    const totalItems = data.value.length;
    balanceTotal.value = totalItems;
    balanceCompleted.value = 0;
    balanceProgress.value = 0;
    showBalanceProgress.value = totalItems > 0;

    // 重置所有项目状态
    data.value.forEach((item) => {
      item.plat_balance = "";
      item.coin_balance = "";
      item.error_msg = "";
      item.exec_status = "0";
    });

    // 分批处理大数据集
    await queryBalanceInBatches();
  } else {
    Notification.warning("查询 coin 类型错误！");
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
        balanceStopStatus.value = true;
        // 隐藏查出账地址进度条
        showBalanceProgress.value = false;
        return;
      }

      const startIndex = batchIndex * BATCH_SIZE;
      const endIndex = Math.min(startIndex + BATCH_SIZE, totalItems);
      const batchData = data.value.slice(startIndex, endIndex);

      console.log(`处理第 ${batchIndex + 1}/${totalBatches} 批，索引 ${startIndex}-${endIndex - 1}`);

      await queryBalanceBatch(batchData, startIndex);

      // 批次间添加短暂延迟，避免过于频繁的请求
      if (batchIndex < totalBatches - 1) {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }

    // 所有批次完成后的统计
    const successCount = data.value.filter(item => item.exec_status === '2').length;
    const failCount = data.value.filter(item => item.exec_status === '3').length;
    const totalCount = data.value.length;

    if (successCount === totalCount) {
      Notification.success('查询成功！');
    } else if (successCount > 0) {
      Notification.warning(`查询完成！成功 ${successCount} 条，失败 ${failCount} 条`);
    } else {
      Notification.error('查询失败：所有记录都查询失败');
    }

  } catch (error) {
    console.error('分批查询失败:', error);
    Notification.error('查询失败：' + error.message);
  } finally {
    balanceLoading.value = false;
    balanceStopStatus.value = true;
    // 隐藏查出账地址进度条
    showBalanceProgress.value = false;
  }
}

// 查询单个批次的余额
async function queryBalanceBatch(batchData, startIndex) {
  try {
    // 使用窗口感知的余额查询
    const params = {
      chain: chainValue.value,
      coin_config: {
        coin_type: currentCoin.value.coin_type,
        contract_address: currentCoin.value.contract_address || null,
        abi: currentCoin.value.abi || null
      },
      items: batchData.map(item => ({
        key: item.key,
        address: item.address || null,
        private_key: item.private_key || null,
        plat_balance: null,
        coin_balance: null,
        nonce: null,
        exec_status: '0',
        error_msg: null,
        retry_flag: false
      })),
      only_coin_config: false,
      thread_count: Number(threadCount.value)
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
          // 保存原始私钥，避免被覆盖
          const originalPrivateKey = data.value[dataIndex].private_key;
          Object.assign(data.value[dataIndex], resultItem);
          // 恢复私钥字段
          data.value[dataIndex].private_key = originalPrivateKey;
        }
      });
    } else {
      // 只有在没有返回任何结果时才设置批次项目为失败状态
      batchData.forEach((item, index) => {
        const dataIndex = startIndex + index;
        if (data.value[dataIndex]) {
          // 保护私钥字段，只更新状态相关字段
          data.value[dataIndex].exec_status = '3';
          data.value[dataIndex].error_msg = result.error_msg || '查询失败！';
        }
      });
    }

    // 更新余额查询进度
    updateBalanceProgress();

  } catch (error) {
    console.error('批次查询失败:', error);

    // 设置批次项目为失败状态，保护私钥字段
    batchData.forEach((item, index) => {
      const dataIndex = startIndex + index;
      if (data.value[dataIndex]) {
        data.value[dataIndex].exec_status = '3';
        data.value[dataIndex].error_msg = '查询失败！';
      }
    });

    // 更新余额查询进度
    updateBalanceProgress();
  }
}



// 查询到账地址余额
async function queryToAddressBalance() {
  if (!stopStatus.value) {
    Notification.warning("请停止或等待执行完成后再查询余额！");
    return;
  }
  if (!balanceStopStatus.value) {
    Notification.warning("请停止或等待查询完成后再查询余额！");
    return;
  }
  if (data.value.length === 0) {
    Notification.warning("请先导入地址！");
    return;
  }

  hasExecutedTransfer.value = false;
  transferTotal.value = data.value.length;
  transferCompleted.value = 0;
  transferProgress.value = 0;

  // 检查是否有到账地址
  const itemsWithToAddr = JSON.parse(JSON.stringify(data.value.filter(item => item.to_addr)));
  if (itemsWithToAddr.length === 0) {
    Notification.warning("请先设置到账地址！");
    return;
  }

  if (currentCoin.value.coin_type === "base" || currentCoin.value.coin_type === "token") {
    balanceLoading.value = true;
    balanceStopFlag.value = false;
    balanceStopStatus.value = false;

    // 初始化查到账地址余额查询进度
    const totalItems = itemsWithToAddr.length;
    toAddressBalanceTotal.value = totalItems;
    toAddressBalanceCompleted.value = 0;
    toAddressBalanceProgress.value = 0;
    showToAddressBalanceProgress.value = totalItems > 0;

    // 重置所有项目状态
    data.value.forEach((item) => {
      item.plat_balance = "";
      item.coin_balance = "";
      item.error_msg = "";
      item.exec_status = "0";
    });

    // 分批处理大数据集
    await queryToAddressBalanceInBatches();
  } else {
    Notification.warning("查询 coin 类型错误！");
  }
}

// 分批查询到账地址余额
async function queryToAddressBalanceInBatches() {
  const BATCH_SIZE = 50; // 每批处理50个地址
  const itemsWithToAddr = data.value.filter(item => item.to_addr);
  const totalItems = itemsWithToAddr.length;
  const totalBatches = Math.ceil(totalItems / BATCH_SIZE);

  console.log(`开始分批查询到账地址余额，总数: ${totalItems}, 批次数: ${totalBatches}, 每批大小: ${BATCH_SIZE}`);

  try {
    for (let batchIndex = 0; batchIndex < totalBatches; batchIndex++) {
      // 检查是否需要停止查询
      if (balanceStopFlag.value) {
        balanceLoading.value = false;
        balanceStopStatus.value = true;
        // 隐藏查到账地址进度条
        showToAddressBalanceProgress.value = false;
        return;
      }

      const startIndex = batchIndex * BATCH_SIZE;
      const endIndex = Math.min(startIndex + BATCH_SIZE, totalItems);
      const batchData = itemsWithToAddr.slice(startIndex, endIndex);

      console.log(`处理第 ${batchIndex + 1}/${totalBatches} 批到账地址，索引 ${startIndex}-${endIndex - 1}`);

      await queryToAddressBalanceBatch(batchData, startIndex);

      // 批次间添加短暂延迟，避免过于频繁的请求
      if (batchIndex < totalBatches - 1) {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
    }

    // 所有批次完成后的统计
    const successCount = data.value.filter(item => item.exec_status === '2').length;
    const failCount = data.value.filter(item => item.exec_status === '3').length;
    const totalCount = itemsWithToAddr.length;

    if (successCount === totalCount) {
      Notification.success(`到账地址余额查询成功！共查询 ${totalCount} 个地址`);
    } else if (successCount > 0) {
      Notification.warning(`到账地址余额查询完成！成功 ${successCount} 条，失败 ${failCount} 条`);
    } else {
      Notification.error('到账地址余额查询失败：所有地址都查询失败');
    }

  } catch (error) {
    console.error('分批查询到账地址失败:', error);
    Notification.error('到账地址余额查询失败：' + error.message);
  } finally {
    balanceLoading.value = false;
    balanceStopStatus.value = true;
    // 隐藏查到账地址进度条
    showToAddressBalanceProgress.value = false;
  }
}

// 查询单个批次的到账地址余额
async function queryToAddressBalanceBatch(batchData, startIndex) {
  try {
    // 创建独立的查询数据，避免影响原始数据
    const queryItems = batchData.map(item => ({
      key: item.key,
      address: item.to_addr, // 使用到账地址而不是发送地址
      private_key: null, // 到账地址不需要私钥
      plat_balance: null,
      coin_balance: null,
      nonce: null,
      exec_status: '0',
      error_msg: null,
      retry_flag: false
    }));

    // 使用窗口感知的余额查询
    const params = {
      chain: chainValue.value,
      coin_config: {
        coin_type: currentCoin.value.coin_type,
        contract_address: currentCoin.value.contract_address || null,
        abi: currentCoin.value.abi || null
      },
      items: queryItems,
      only_coin_config: false,
      thread_count: Number(threadCount.value)
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
        items: queryItems.map(item => ({
          ...item,
          plat_balance: '2.5',
          coin_balance: '250.0',
          nonce: 1,
          exec_status: '2',
          error_msg: null
        }))
      };
    }

    if (result.success || result.items) {
      // 更新数据 - 根据key匹配原始数据项
      result.items.forEach((resultItem, index) => {
        const originalItem = batchData[index];
        const dataIndex = data.value.findIndex(item => item.key === originalItem.key);
        if (dataIndex !== -1) {
          // 保存原始私钥和到账地址，避免被覆盖
          const originalPrivateKey = data.value[dataIndex].private_key;
          const originalToAddr = data.value[dataIndex].to_addr;
          Object.assign(data.value[dataIndex], resultItem);
          // 恢复私钥和到账地址字段
          data.value[dataIndex].private_key = originalPrivateKey;
          data.value[dataIndex].to_addr = originalToAddr;
        }
      });
    } else {
      // 只有在没有返回任何结果时才设置批次项目为失败状态
      batchData.forEach((item, index) => {
        const dataIndex = data.value.findIndex(dataItem => dataItem.key === item.key);
        if (dataIndex !== -1) {
          // 保护私钥字段，只更新状态相关字段
          data.value[dataIndex].exec_status = '3';
          data.value[dataIndex].error_msg = result.error_msg || '查询失败！';
        }
      });
    }

    // 更新查到账地址余额查询进度
    updateToAddressBalanceProgress();

  } catch (error) {
    console.error('批次查询到账地址失败:', error);

    // 设置批次项目为失败状态，保护私钥字段
    batchData.forEach((item, index) => {
      const dataIndex = data.value.findIndex(dataItem => dataItem.key === item.key);
      if (dataIndex !== -1) {
        data.value[dataIndex].exec_status = '3';
        data.value[dataIndex].error_msg = '查询失败！';
      }
    });

    // 更新查到账地址余额查询进度
    updateToAddressBalanceProgress();
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
    .then(async () => {
      if (stopFlag.value) {
        Notification.warning("已停止执行！");
      } else {
        const retryData = inputData.filter((item) => item.retry_flag === true);
        if (form.error_retry === "1" && retryData.length > 0) {
          //  存在重试数据，使用智能重试逻辑
          await performIntelligentRetry(retryData);
        } else {
          // 计算成功的转账数量
          const successCount = inputData.filter(item => item.exec_status === '2').length;
          const totalCount = inputData.length;

          if (successCount > 0) {
            Notification.success(`执行完成！成功转账 ${successCount}/${totalCount} 笔`);

            // 如果有成功的转账，触发庆祝效果
            if (successCount >= totalCount * 0.5) { // 成功率超过50%就庆祝
              setTimeout(() => {
                triggerCelebration();
              }, 1000); // 延迟1秒触发庆祝，让用户先看到完成通知
            }
          } else {
            Notification.warning("执行完成，但没有成功的转账");
          }

          stopStatus.value = true;
          // 标记转账会话完全结束
          transferSessionCompleted.value = true;
        }
      }
      startLoading.value = false;
      stopFlag.value = false;
      // 隐藏进度条
      showProgress.value = false;
    })
    .catch(() => {
      Notification.error("执行失败！");
      startLoading.value = false;
      stopStatus.value = true;
      // 隐藏进度条
      showProgress.value = false;
    });
}

// 执行
function startTransfer() {
  // 基础验证检查
  if (balanceLoading.value) {
    startLoading.value = false;
    Notification.warning("请等待余额查询完成后再执行！");
    return;
  }
  if (data.value.length === 0) {
    startLoading.value = false;
    Notification.warning("请先导入私钥！");
    return;
  }

  // 立即设置loading状态，提供即时反馈
  startLoading.value = true;

  // 使用 requestIdleCallback 或 setTimeout 来异步执行数据验证，避免阻塞UI
  const performValidationAndStart = () => {
    try {
      // 首先进行快速验证
      const quickValidation = quickValidateData();
      if (!quickValidation.isValid) {
        startLoading.value = false;
        Notification.warning(quickValidation.reason === '存在私钥或地址为空的数据'
          ? "请检查是否所有私钥都有对应的转账地址！"
          : "包含转账金额为空的错误数据请核实！");
        return;
      }

      // 检查未完成的转账记录
      let hasIncompleteTransfers = false;
      if (hasExecutedTransfer.value && !transferSessionCompleted.value) {
        // 如果执行过转账且转账会话未完成，则认为有未完成的转账
        hasIncompleteTransfers = true;
      }

      if (hasIncompleteTransfers && stopStatus.value) {
        // 暂时重置loading状态，等待用户选择
        startLoading.value = false;
        // 显示转账确认弹窗
        transferConfirmVisible.value = true;
      } else {
        // 首次转账或正在进行中，直接开始
        executeTransfer(data.value, true);
      }
    } catch (error) {
      console.error('数据验证过程中发生错误:', error);
      startLoading.value = false;
      Notification.error('数据验证失败，请重试');
    }
  };

  // 使用 requestIdleCallback 在浏览器空闲时执行，如果不支持则使用 setTimeout
  if (window.requestIdleCallback) {
    window.requestIdleCallback(performValidationAndStart, { timeout: 100 });
  } else {
    setTimeout(performValidationAndStart, 0);
  }
}

// 处理转账确认弹窗的函数
function handleTransferConfirmOk() {
  transferConfirmLoading.value = true;

  setTimeout(() => {
    // 继续上次转账 - 只处理等待执行的项目
    const incompleteData = data.value.filter(item =>
      item.exec_status === "0"
    );
    if (incompleteData.length === 0) {
      transferConfirmVisible.value = false;
      transferConfirmLoading.value = false;
      startLoading.value = false;
      Notification.info("所有转账已完成！");
      return;
    }

    transferConfirmVisible.value = false;
    transferConfirmLoading.value = false;
    startLoading.value = true;
    executeTransfer(incompleteData, false);
  }, 100)
}

function handleTransferConfirmCancel() {
  transferConfirmLoading.value = true;

  transferConfirmVisible.value = false;
  transferConfirmLoading.value = false;
  startLoading.value = true;
  setTimeout(() => {
    // 重新开始转账 - 重置所有状态
    executeTransfer(data.value, true);
  }, 100)
}

function handleTransferConfirmClose() {
  transferConfirmVisible.value = false;
  transferConfirmLoading.value = false;
  startLoading.value = false;
}

// 执行转账的通用方法
function executeTransfer(transferData, resetStatus = true) {
  validateForm()
    .then(async () => {
      // 验证通过，loading状态已在startTransfer中设置
      stopFlag.value = false;
      stopStatus.value = false;

      // 标记已执行过转账操作（用于区分余额查询和转账）
      hasExecutedTransfer.value = true;
      // 标记转账会话开始，未完成
      transferSessionCompleted.value = false;

      // 记录转账开始时间（仅在重新开始时记录）
      if (resetStatus) {
        transferStartTime.value = Date.now();
        console.log('转账开始时间:', new Date(transferStartTime.value).toLocaleString());
      }

      // 初始化进度条
      if (resetStatus) {
        // 重新开始时，总数为所有数据
        transferTotal.value = data.value.length;
        transferCompleted.value = 0;
        transferProgress.value = 0;

        // 重新开始时重置所有状态 - 使用异步批处理优化性能
        await resetDataStatusAsync();
      } else {
        // // 继续转账时，总数为实际要处理的数据量
        // transferTotal.value = transferData.length;
        // transferCompleted.value = 0;
        // transferProgress.value = 0;

        // 继续转账时不需要重置状态，因为只处理等待执行的项目
      }

      showProgress.value = true;

      await transferFnc(transferData);
    })
    .catch(() => {
      // 验证失败
      startLoading.value = false;
    });
}

// 执行转账 - 基于钱包地址的队列管理系统
async function iterTransfer(accountData) {
  // 判断是主币转账还是代币转账
  let contract;
  if (currentCoin.value.coin_type === "token") {
    contract = new ethers.Contract(
      currentCoin.value.contract_address,
      currentCoin.value.abi
    );
  }

  // 如果线程数为1，则按照table中的顺序逐一执行，无需分组
  if (threadCount.value === 1) {
    for (let index = 0; index < accountData.length; index++) {
      if (stopFlag.value) {
        stopStatus.value = true;
        return;
      }

      const item = accountData[index];

      // 跳过已完成或失败的记录，只处理等待执行的记录
      if (item.exec_status !== '0') {
        continue;
      }

      // 检查gas价格是否超过限制
      if (form.max_gas_price && form.max_gas_price.trim()) {
        const gasPriceOk = await checkGasPriceForTransfer();
        if (!gasPriceOk) {
          // Gas价格超过限制，暂停转账并启动监控
          pausedTransferData.value = { accountData, index };
          await startGasPriceMonitoring();

          // 等待gas价格降低
          while (transferPaused.value && !stopFlag.value) {
            await new Promise(resolve => setTimeout(resolve, 1000));
          }

          // 如果用户手动停止了转账，退出
          if (stopFlag.value) {
            stopStatus.value = true;
            return;
          }
        }
      }

      // 找到该item在原始data.value数组中的真实索引
      const realIndex = data.value.findIndex(dataItem => dataItem.key === item.key);
      if (realIndex === -1) {
        console.error('无法找到对应的数据项');
        continue;
      }
      const config = {
        ...transferConfig.value,
        transfer_amount: form.amount_from === '1' ? (item.amount && item.amount.trim() !== '' ? Number(item.amount) : 0) : (form.send_count && form.send_count.trim() !== '' ? Number(form.send_count) : 0), // 转账当前指定的固定金额
      };

      try {
        if (currentCoin.value.coin_type === "base") {
          // 设置状态 为执行中
          data.value[realIndex].exec_status = "1";
          try {
            console.log("config:", config);
            const res = await invoke("base_coin_transfer", {
              index: realIndex + 1,
              item: item,
              config: config
            });
            console.log("base_coin_transfer 返回信息:", res);
            data.value[realIndex].exec_status = "2";
            // 转账成功时只显示tx_hash
            if (typeof res === 'object' && res !== null) {
              if (res.success && res.tx_hash) {
                data.value[realIndex].error_msg = res.tx_hash;
              } else {
                data.value[realIndex].error_msg = res.error || '转账失败';
              }
            } else {
              data.value[realIndex].error_msg = String(res || '转账成功');
            }
            // 更新进度条
            updateTransferProgress();
          } catch (err) {
            if (err === "base gas price 超出最大值限制") {
              Notification.error("base gas price 超出最大值限制");
              // 停止
              stopTransfer();
              data.value[realIndex].exec_status = "0";
              data.value[realIndex].error_msg = "";
              return;
            } else {
              data.value[realIndex].exec_status = "3";
              data.value[realIndex].error_msg = err;
              // 更新进度条
              updateTransferProgress();
            }
          }
        } else if (currentCoin.value.coin_type === "token") {
          // 设置状态 为执行中
          data.value[realIndex].exec_status = "1";
          try {
            const res = await invoke("token_transfer", {
              index: realIndex + 1,
              item: item,
              config: {
                ...config,
                contract_address: contract.address,
                abi: contract.abi
              }
            });
            console.log("token_transfer 返回信息:", res);
            data.value[realIndex].exec_status = "2";
            // 转账成功时只显示tx_hash
            if (typeof res === 'object' && res !== null) {
              if (res.success && res.tx_hash) {
                data.value[realIndex].error_msg = res.tx_hash;
              } else {
                data.value[realIndex].error_msg = res.error || '转账失败';
              }
            } else {
              data.value[realIndex].error_msg = String(res || '转账成功');
            }
            // 更新进度条
            updateTransferProgress();
          } catch (err) {
            if (err === "base gas price 超出最大值限制") {
              Notification.error("base gas price 超出最大值限制");
              // 停止
              stopTransfer();
              data.value[realIndex].exec_status = "0";
              data.value[realIndex].error_msg = "";
              return;
            } else {
              data.value[realIndex].exec_status = "3";
              data.value[realIndex].error_msg = err;
              // 更新进度条
              updateTransferProgress();
            }
          }
        } else {
          Notification.error("未知币种类型");
          return;
        }
      } catch (e) {
        // 交易失败
        data.value[realIndex].exec_status = "3";
        data.value[realIndex].error_msg = e.message || '转账异常';
        updateTransferProgress();
      }

      // 添加延迟等待（只在实际执行了转账后才延迟，跳过的记录不延迟）
      if (index < accountData.length - 1 && !stopFlag.value) {
        const minDelay = form.min_interval && form.min_interval.trim() !== '' ? Number(form.min_interval) * 1000 : 1000;
        const maxDelay = form.max_interval && form.max_interval.trim() !== '' ? Number(form.max_interval) * 1000 : 3000;
        const randomDelay = Math.floor(Math.random() * (maxDelay - minDelay + 1)) + minDelay;

        // 找到下一条待执行的数据
        let nextPendingIndex = -1;
        for (let i = index + 1; i < accountData.length; i++) {
          if (accountData[i].exec_status === '0') {
            nextPendingIndex = data.value.findIndex(dataItem => dataItem.key === accountData[i].key);
            break;
          }
        }

        // 如果找到下一条待执行的数据，在其error_msg字段显示倒计时
        if (nextPendingIndex !== -1) {
          const originalErrorMsg = data.value[nextPendingIndex].error_msg;
          let remainingTime = Math.ceil(randomDelay / 1000);

          // 每秒更新倒计时
          const countdownInterval = setInterval(() => {
            if (stopFlag.value) {
              clearInterval(countdownInterval);
              // 恢复原始错误信息
              data.value[nextPendingIndex].error_msg = originalErrorMsg;
              return;
            }

            data.value[nextPendingIndex].error_msg = `等待中...${remainingTime}秒`;
            remainingTime--;

            if (remainingTime < 0) {
              clearInterval(countdownInterval);
              // 恢复原始错误信息
              data.value[nextPendingIndex].error_msg = originalErrorMsg;
            }
          }, 1000);

          await new Promise(resolve => {
            setTimeout(() => {
              clearInterval(countdownInterval);
              // 确保恢复原始错误信息
              if (nextPendingIndex !== -1 && data.value[nextPendingIndex]) {
                data.value[nextPendingIndex].error_msg = originalErrorMsg;
              }
              resolve();
            }, randomDelay);
          });
        } else {
          // 如果没有找到下一条待执行的数据，使用原来的延迟方式
          await new Promise(resolve => setTimeout(resolve, randomDelay));
        }
      }
    }
    return;
  }

  // 多线程模式：按钱包地址分组数据，避免nonce冲突
  // 性能优化：预先构建索引映射，避免重复的findIndex操作
  const keyToIndexMap = new Map();
  data.value.forEach((dataItem, index) => {
    keyToIndexMap.set(dataItem.key, index);
  });

  const walletGroups = new Map();
  accountData.forEach((item, index) => {
    const walletAddress = item.address || item.private_key; // 使用地址或私钥作为分组键
    if (!walletGroups.has(walletAddress)) {
      walletGroups.set(walletAddress, []);
    }
    // 使用预构建的索引映射快速查找真实索引
    const realIndex = keyToIndexMap.get(item.key) ?? -1;
    walletGroups.get(walletAddress).push({ ...item, originalIndex: index, realIndex: realIndex });
  });

  // 将钱包组转换为数组，便于并发处理
  const walletGroupsArray = Array.from(walletGroups.values());

  // 并发处理不同钱包的转账，但同一钱包内的交易串行执行
  const processWalletGroup = async (walletTransactions) => {
    // 同一钱包的交易必须串行执行，避免nonce冲突
    for (const item of walletTransactions) {
      if (stopFlag.value) {
        stopStatus.value = true;
        return;
      }

      // 跳过已完成或失败的记录，只处理等待执行的记录
      if (item.exec_status !== '0') {
        continue;
      }

      const originalIndex = item.originalIndex;
      const realIndex = item.realIndex;

      if (realIndex === -1) {
        console.error('无法找到对应的数据项');
        continue;
      }
      const config = {
        error_count_limit: 3, //  错误次数限制
        error_retry: form.error_retry, // 是否自动失败重试
        chain: chainValue.value,
        chainLayer: currentChain.value.layer,
        l1: currentChain.value.l1,
        scalar: currentChain.value.scalar,
        delay: [form.min_interval && form.min_interval.trim() !== '' ? Number(form.min_interval) : 1, form.max_interval && form.max_interval.trim() !== '' ? Number(form.max_interval) : 3], // 延迟时间
        transfer_type: form.send_type, // 转账类型 1：全部转账 2:转账固定数量 3：转账随机数量  4：剩余随机数量
        transfer_amount: form.amount_from === '1' ? (item.amount && item.amount.trim() !== '' ? Number(item.amount) : 0) : (form.send_count && form.send_count.trim() !== '' ? Number(form.send_count) : 0), // 转账当前指定的固定金额
        transfer_amount_list: [form.send_min_count && form.send_min_count.trim() !== '' ? Number(form.send_min_count) : 0, form.send_max_count && form.send_max_count.trim() !== '' ? Number(form.send_max_count) : 0], // 转账数量 (transfer_type 为 1 时生效) 转账数量在5-10之间随机，第二个数要大于第一个数！！
        left_amount_list: [form.send_min_count && form.send_min_count.trim() !== '' ? Number(form.send_min_count) : 0, form.send_max_count && form.send_max_count.trim() !== '' ? Number(form.send_max_count) : 0], // 剩余数量 (transfer_type 为 2 时生效) 剩余数量在4-6之间随机，第二个数要大于第一个数！！
        amount_precision: form.amount_precision && form.amount_precision.trim() !== '' ? Number(form.amount_precision) : 3, // 一般无需修改，转账个数的精确度 6 代表个数有6位小数
        limit_type: form.limit_type, // limit_type 限制类型 1：自动 2：指定数量 3：范围随机
        limit_count: form.limit_count && form.limit_count.trim() !== '' ? Number(form.limit_count) : 21000, // limit_count 指定数量 (limit_type 为 2 时生效)
        limit_count_list: [form.limit_min_count && form.limit_min_count.trim() !== '' ? Number(form.limit_min_count) : 21000, form.limit_max_count && form.limit_max_count.trim() !== '' ? Number(form.limit_max_count) : 30000],
        gas_price_type: form.gas_price_type, // gas price类型 1: 自动 2：固定gas price 3：gas price溢价率
        gas_price_rate: form.gas_price_rate && form.gas_price_rate.trim() !== '' ? Number(form.gas_price_rate) / 100 : 0.05, // gas price溢价率，0.05代表gas price是当前gas price的105%
        gas_price: form.gas_price && form.gas_price.trim() !== '' ? Number(form.gas_price) : 30, // 设置最大的gas price，单位gwei
        max_gas_price: form.max_gas_price && form.max_gas_price.trim() !== '' ? Number(form.max_gas_price) : 0, // 设置最大的gas price，单位gwei
      };

      try {
        if (currentCoin.value.coin_type === "base") {
          // 设置状态 为执行中
          data.value[realIndex].exec_status = "1";
          try {
            console.log("config:", config);
            const res = await invoke("base_coin_transfer", {
              index: realIndex + 1,
              item: item,
              config: config
            });
            console.log("base_coin_transfer 返回信息:", res);
            data.value[realIndex].exec_status = "2";
            // 转账成功时只显示tx_hash
            if (typeof res === 'object' && res !== null) {
              if (res.success && res.tx_hash) {
                data.value[realIndex].error_msg = res.tx_hash;
              } else {
                data.value[realIndex].error_msg = res.error || '转账失败';
              }
            } else {
              data.value[realIndex].error_msg = String(res || '转账成功');
            }
            // 更新进度条
            updateTransferProgress();
          } catch (err) {
            if (err === "base gas price 超出最大值限制") {
              Notification.error("base gas price 超出最大值限制");
              // 停止
              stopTransfer();
              data.value[realIndex].exec_status = "0";
              data.value[realIndex].error_msg = "";
              return; // 停止当前钱包组的处理
            } else {
              data.value[realIndex].exec_status = "3";
              data.value[realIndex].error_msg = err;
              // 更新进度条
              updateTransferProgress();
            }
          }
        } else if (currentCoin.value.coin_type === "token") {
          // 设置状态 为执行中
          data.value[realIndex].exec_status = "1";
          try {
            const res = await invoke("token_transfer", {
              index: realIndex + 1,
              item: item,
              config: {
                ...config,
                contract_address: contract.address,
                abi: contract.abi
              }
            });
            console.log("token_transfer 返回信息:", res);
            data.value[realIndex].exec_status = "2";
            // 转账成功时只显示tx_hash
            if (typeof res === 'object' && res !== null) {
              if (res.success && res.tx_hash) {
                data.value[realIndex].error_msg = res.tx_hash;
              } else {
                data.value[realIndex].error_msg = res.error || '转账失败';
              }
            } else {
              data.value[realIndex].error_msg = String(res || '转账成功');
            }
            // 更新进度条
            updateTransferProgress();
          } catch (err) {
            if (err === "base gas price 超出最大值限制") {
              Notification.error("base gas price 超出最大值限制");
              // 停止
              stopTransfer();
              data.value[realIndex].exec_status = "0";
              data.value[realIndex].error_msg = "";
              return; // 停止当前钱包组的处理
            } else {
              data.value[realIndex].exec_status = "3";
              data.value[realIndex].error_msg = err;
              // 更新进度条
              updateTransferProgress();
            }
          }
        } else {
          Notification.error("未知币种类型");
          return;
        }
      } catch (e) {
        // 交易失败
        data.value[realIndex].exec_status = "3";
        data.value[realIndex].error_msg = e.message || '转账异常';
        updateTransferProgress();
      }
    }
  };

  // 使用Promise.all并发处理不同钱包组，但限制并发数量
  const maxConcurrency = Math.min(threadCount.value, walletGroupsArray.length);
  const chunks = [];

  // 将钱包组分批处理，每批最多threadCount个
  for (let i = 0; i < walletGroupsArray.length; i += maxConcurrency) {
    chunks.push(walletGroupsArray.slice(i, i + maxConcurrency));
  }

  // 逐批处理钱包组
  for (const chunk of chunks) {
    if (stopFlag.value) {
      stopStatus.value = true;
      return;
    }

    // 并发处理当前批次的钱包组
    await Promise.all(chunk.map(processWalletGroup));
  }
}

// 停止执行
function stopTransfer() {
  startLoading.value = false;
  stopFlag.value = true;
  // 隐藏进度条
  showProgress.value = false;
}

// 快速数据验证 - 使用缓存避免重复验证
function quickValidateData() {
  const currentDataLength = data.value.length;
  const currentFormState = `${form.send_type}_${form.amount_from}`;
  const currentTime = Date.now();

  // 检查缓存是否有效（数据长度和表单状态未变，且缓存时间在5秒内）
  if (dataValidationCache.value.lastDataLength === currentDataLength &&
      dataValidationCache.value.lastFormState === currentFormState &&
      currentTime - dataValidationCache.value.cacheTime < 5000) {
    return {
      isValid: dataValidationCache.value.isValid,
      reason: dataValidationCache.value.invalidReason
    };
  }

  // 执行快速验证
  let isValid = true;
  let reason = '';

  // 快速检查：只验证前100条和随机抽样
  const sampleSize = Math.min(100, currentDataLength);
  const step = Math.max(1, Math.floor(currentDataLength / sampleSize));

  for (let i = 0; i < currentDataLength; i += step) {
    const item = data.value[i];

    if (!item.private_key || !item.to_addr) {
      isValid = false;
      reason = '存在私钥或地址为空的数据';
      break;
    }

    if (form.send_type === '2' && form.amount_from === '1' && !item.amount) {
      isValid = false;
      reason = '存在转账金额为空的数据';
      break;
    }
  }

  // 更新缓存
  dataValidationCache.value = {
    lastDataLength: currentDataLength,
    lastFormState: currentFormState,
    isValid,
    invalidReason: reason,
    cacheTime: currentTime
  };

  return { isValid, reason };
}

// 清除数据验证缓存
function clearValidationCache() {
  dataValidationCache.value = {
    lastDataLength: 0,
    lastFormState: '',
    isValid: false,
    invalidReason: '',
    cacheTime: 0
  };
}

// 异步批处理重置数据状态 - 性能优化
async function resetDataStatusAsync() {
  const totalItems = data.value.length;

  // 对于小数据量，直接同步处理
  if (totalItems <= 500) {
    for (let i = 0; i < totalItems; i++) {
      const item = data.value[i];
      item.exec_status = "0";
      item.error_msg = "";
      item.retry_flag = false;
      item.error_count = 0;
    }
    return;
  }

  // 对于大数据量，使用批处理
  const batchSize = Math.max(50, Math.min(200, Math.floor(totalItems / 20))); // 动态调整批次大小
  let processedCount = 0;

  for (let i = 0; i < totalItems; i += batchSize) {
    const endIndex = Math.min(i + batchSize, totalItems);

    // 批量重置当前批次的数据状态
    for (let j = i; j < endIndex; j++) {
      const item = data.value[j];
      item.exec_status = "0";
      item.error_msg = "";
      item.retry_flag = false;
      item.error_count = 0;
    }

    processedCount = endIndex;

    // 每处理一定数量后让出控制权
    if (i > 0 && i % (batchSize * 5) === 0) {
      await new Promise(resolve => setTimeout(resolve, 0));
    }

    // 显示进度（仅在大数据量时）
    if (totalItems > 2000 && processedCount % 1000 === 0) {
      console.log(`数据重置进度: ${processedCount}/${totalItems} (${Math.round(processedCount / totalItems * 100)}%)`);
    }
  }
}

// 停止查询余额
async function stopBalanceQuery() {
  console.log('stopBalanceQuery方法被调用');
  try {
    // 调用后端停止接口
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
  balanceStopStatus.value = true;
  // 隐藏两个进度条
  showBalanceProgress.value = false;
  showToAddressBalanceProgress.value = false;
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

// 显示高级筛选弹窗
function showAdvancedFilter() {
  advancedFilterVisible.value = true;
}

// 应用高级筛选
function applyAdvancedFilter() {
  let filteredItems = [];

  data.value.forEach(item => {
    let shouldSelect = true;

    // 平台币余额筛选
    if (filterForm.platBalanceValue && filterForm.platBalanceValue.trim() !== '') {
      const platBalanceValue = parseFloat(filterForm.platBalanceValue);
      const itemPlatBalance = parseFloat(item.plat_balance || 0);

      if (filterForm.platBalanceOperator === 'gt' && itemPlatBalance <= platBalanceValue) {
        shouldSelect = false;
      } else if (filterForm.platBalanceOperator === 'eq' && itemPlatBalance !== platBalanceValue) {
        shouldSelect = false;
      } else if (filterForm.platBalanceOperator === 'lt' && itemPlatBalance >= platBalanceValue) {
        shouldSelect = false;
      }
    }

    // 代币余额筛选
    if (shouldSelect && filterForm.coinBalanceValue && filterForm.coinBalanceValue.trim() !== '') {
      const coinBalanceValue = parseFloat(filterForm.coinBalanceValue);
      const itemCoinBalance = parseFloat(item.coin_balance || 0);

      if (filterForm.coinBalanceOperator === 'gt' && itemCoinBalance <= coinBalanceValue) {
        shouldSelect = false;
      } else if (filterForm.coinBalanceOperator === 'eq' && itemCoinBalance !== coinBalanceValue) {
        shouldSelect = false;
      } else if (filterForm.coinBalanceOperator === 'lt' && itemCoinBalance >= coinBalanceValue) {
        shouldSelect = false;
      }
    }

    // 错误信息模糊匹配筛选
    if (shouldSelect && filterForm.errorMsg && filterForm.errorMsg.trim()) {
      const errorMsg = item.error_msg || '';
      if (!errorMsg.toLowerCase().includes(filterForm.errorMsg.toLowerCase())) {
        shouldSelect = false;
      }
    }

    if (shouldSelect) {
      filteredItems.push(item.key);
    }
  });

  // 更新选中的项
  selectedKeys.value = filteredItems;

  // 关闭弹窗
  advancedFilterVisible.value = false;

  // 显示筛选结果
  Notification.success(`筛选完成，共选中 ${filteredItems.length} 条数据`);
}

function deleteSelected() {
  if (startLoading.value) {
    Notification.warning("请停止或等待执行完成后再删除数据！");
    return;
  }

  // 检查是否有选中的项目
  if (selectedKeys.value.length === 0) {
    Notification.warning("请先选择要删除的项目！");
    return;
  }

  // 显示确认对话框
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除选中的 ${selectedKeys.value.length} 个项目吗？此操作不可撤销。`,
    okText: '确认删除',
    cancelText: '取消',
    okButtonProps: {
      status: 'danger'
    },
    onOk: () => {
      // 执行删除操作
      data.value = data.value.filter(
        (item) => !selectedKeys.value.includes(item.key)
      );
      selectedKeys.value = []; // 清空选中状态
      Notification.success("删除成功");
    }
  });
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
        { key: 'eth', coin: 'ETH', type: 'base', decimals: 18 },
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

// 处理TitleBar的before-close事件
async function handleBeforeClose() {
  try {
    console.log('TitleBar触发关闭事件，正在停止后台操作...');

    // 停止余额查询操作
    if (balanceLoading.value) {
      await stopBalanceQuery();
      console.log('已停止余额查询操作');
    }

    // 停止转账操作
    if (startLoading.value) {
      await stopTransfer();
      console.log('已停止转账操作');
    }

    // 停止gas价格监控
    if (gasPriceMonitoring.value && gasPriceTimer.value) {
      clearInterval(gasPriceTimer.value);
      gasPriceTimer.value = null;
      gasPriceMonitoring.value = false;
      console.log('已清理gas价格监控定时器');
    }

    // 重置相关状态
    transferPaused.value = false;
    pausedTransferData.value = null;
    gasPriceCountdown.value = 0;
    currentGasPrice.value = 0;

    console.log('TitleBar窗口关闭清理完成，所有后台操作已停止');
  } catch (error) {
    console.error('处理窗口关闭事件时发生错误:', error);
  }
}
</script>

<template>
  <!-- 标题栏组件 -->
  <TitleBar :title="windowTitle" @before-close="handleBeforeClose" />

  <div class="container transfer" style="height: 100vh; display: flex; flex-direction: column; overflow: hidden;">
    <!-- <span class="pageTitle">批量转账</span> -->
    <!-- 工具栏 -->
    <div class="toolBar" style="flex-shrink: 0;">
      <a-button type="primary" @click="debouncedHandleClick">
        <template #icon>
          <Icon icon="mdi:wallet" />
        </template>
        钱包录入
      </a-button>
      <a-tooltip content="导入按照“模板文件”填写的文件" position="bottom">
        <a-button type="primary" status="success" style="margin-left: 10px" @click="upload">
          <template #icon>
            <Icon icon="mdi:upload" />
          </template>
          导入文件（推荐）
        </a-button>
      </a-tooltip>
      <input type="file" ref="uploadInputRef" @change="UploadFile" id="btn_file" style="display: none" />
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
      <VirtualScrollerTable v-else-if="tableBool" :columns="columns" :data="data" :row-selection="rowSelection"
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

    <!-- 导入进度条 - 悬浮在页面顶部 -->
    <Transition name="progress-slide" appear>
      <div v-if="showImportProgress" class="floating-progress-bar">
        <div class="progress-content">
          <div class="progress-header">
            <span class="progress-title">{{ importProgressText }}</span>
            <span class="progress-count">{{ importCompleted }} / {{ importTotal }}</span>
          </div>
          <a-progress :percent="importProgress" :show-text="true" :stroke-width="6" :color="{
            '0%': '#722ed1',
            '100%': '#722ed1'
          }" class="progress-bar" />
        </div>
      </div>
    </Transition>

    <!-- 转账进度条 - 悬浮在页面顶部 -->
    <Transition name="progress-slide" appear>
      <div v-if="showProgress" class="floating-progress-bar" :style="{ top: showImportProgress ? '120px' : '45px' }">
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

    <!-- 余额查询进度条 - 悬浮在页面顶部 -->
    <Transition name="progress-slide" appear>
      <div v-if="showBalanceProgress" class="floating-progress-bar" :style="{
        top: (showImportProgress && showProgress) ? '220px' :
             (showImportProgress || showProgress) ? '120px' : '45px'
      }">
        <div class="progress-content">
          <div class="progress-header">
            <span class="progress-title">查出账地址进度</span>
            <span class="progress-count">{{ balanceCompleted }} / {{ balanceTotal }}</span>
          </div>
          <a-progress :percent="balanceProgress" :show-text="true" :stroke-width="6" :color="{
            '0%': '#1890ff',
            '100%': '#1890ff'
          }" class="progress-bar" />
        </div>
      </div>
    </Transition>

    <!-- 查到账地址余额查询进度条 - 悬浮在页面顶部 -->
    <Transition name="progress-slide" appear>
      <div v-if="showToAddressBalanceProgress" class="floating-progress-bar" :style="{
        top: (showImportProgress && showProgress && showBalanceProgress) ? '320px' :
             ((showImportProgress && showProgress) || (showImportProgress && showBalanceProgress) || (showProgress && showBalanceProgress)) ? '220px' :
             (showImportProgress || showProgress || showBalanceProgress) ? '120px' : '45px'
      }">
        <div class="progress-content">
          <div class="progress-header">
            <span class="progress-title">查到账地址进度</span>
            <span class="progress-count">{{ toAddressBalanceCompleted }} / {{ toAddressBalanceTotal }}</span>
          </div>
          <a-progress :percent="toAddressBalanceProgress" :show-text="true" :stroke-width="6" :color="{
            '0%': '#52c41a',
            '100%': '#52c41a'
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
            <span v-show="chainValue !== 'sol'" style="flex: 1; text-align: end; color: #00b42a">Gas Price: {{
              data?.gas_price ?? "未知" }}</span>
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
        <a-row v-show="chainValue !== 'sol'" style="height: 70px; display: flex;">
          <a-form-item field="limit_type" label="Gas Limit" style="width: 245px;">
            <a-radio-group v-model="form.limit_type" type="button">
              <a-radio value="1">自动</a-radio>
              <a-radio value="2">指定数量</a-radio>
              <a-radio value="3">范围随机</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item v-if="form.limit_type === '2'" style="width: 150px;" field="limit_count" label="Gas Limit">
            <a-input v-model="form.limit_count" />
          </a-form-item>
          <a-form-item v-if="form.limit_type === '3'" style="width: 265px;" field="limit_count_scope"
            label="Gas Limit 范围">
            <a-input v-model="form.limit_min_count" />
            <span style="padding: 0 5px">至</span>
            <a-input v-model="form.limit_max_count" />
          </a-form-item>
          <a-divider direction="vertical" style="height: 50px; margin: 15px 10px 0 10px;" />
          <a-form-item field="gas_price_type" label="Gas Price 方式" style="width: 230px;">
            <a-radio-group v-model="form.gas_price_type" type="button">
              <a-radio value="1">自动</a-radio>
              <a-radio value="2">固定值</a-radio>
              <a-radio value="3">指定比例</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item v-if="form.gas_price_type === '2'" field="gas_price" style="width: 120px;" label="Gas Price">
            <a-input v-model="form.gas_price" />
          </a-form-item>
          <a-form-item v-if="form.gas_price_type === '3'" field="gas_price_rate" style="width: 100px;" label="提高比例">
            <a-input v-model="form.gas_price_rate">
              <template #append> %</template>
            </a-input>
          </a-form-item>
          <a-form-item v-if="form.gas_price_type === '1' || form.gas_price_type === '3'" field="max_gas_price"
            style="width: 130px;" label="最大 Gas Price" tooltip="为空时则不设置上限（单位：Gwei）">
            <a-input v-model="form.max_gas_price" />
            <!-- Gas监控状态显示 -->
            <div v-if="gasPriceMonitoring" class="gas-monitoring-info"
              style="position: absolute; left: 140px; top: 0; width: 300px; font-size: 12px; color: #666; background: #f8f9fa; padding: 8px; border-radius: 4px; border: 1px solid #e8e9ea; z-index: 10;">
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px;">
                <span style="color: #ff4d4f;">⏸️ 转账已暂停</span>
                <span>Gas监控中...</span>
                <span style="color: #1890ff;">{{ gasPriceCountdown }}秒后查询</span>
              </div>
              <div>
                <span>当前Gas: {{ currentGasPrice }} Gwei</span>
                <span style="margin-left: 12px;">目标: ≤{{ form.max_gas_price }} Gwei</span>
              </div>
            </div>
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
        <a-button v-if="!startLoading && stopStatus" type="success" class="execute-btn" @click="debouncedStartTransfer"
          :style="{ width: '130px', height: '40px', fontSize: '14px' }">
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
  <a-modal v-model:visible="transferConfirmVisible" title="转账确认" :mask-closable="false" :closable="true"
    @close="handleTransferConfirmClose" @cancel="handleTransferConfirmClose">
    <div>检测到上次转账未完成，请选择操作方式：</div>
    <template #footer>
      <a-button @click="handleTransferConfirmClose">关闭</a-button>
      <a-button type="primary" @click="handleTransferConfirmCancel" :loading="transferConfirmLoading"
        style="margin-left: 10px">
        重新开始转账
      </a-button>
      <a-button type="primary" status="success" @click="handleTransferConfirmOk" :loading="transferConfirmLoading"
        style="margin-left: 10px">
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

  <!-- 庆祝状态覆盖层 -->
  <div v-if="showCelebration" class="celebration-overlay">
    <div class="celebration-content">
      <div class="celebration-icon">🎉</div>
      <div class="celebration-title">转账完成！</div>
      <div class="celebration-subtitle">恭喜您成功完成批量转账</div>
      <div class="celebration-sparkle">✨ 即将为您展示打赏选项 ✨</div>
    </div>
  </div>

  <!-- 打赏弹窗 -->
  <a-modal v-model:visible="showTipModal" title="💝 支持开发者" width="580px" :mask-closable="false">
    <div class="tip-modal-content">
      <div class="tip-header">
        <div class="tip-description">
          <p>感谢使用批量转账工具！</p>
          <p>如果对您有帮助，欢迎给开发者一点小小的支持～</p>
        </div>
      </div>

      <div class="tip-info">
        <div class="tip-info-row">
          <span class="tip-label">开发者地址:</span>
          <span class="tip-address">{{ developerAddress.substring(0, 10) }}...{{ developerAddress.slice(-8) }}</span>
        </div>
        <div class="tip-info-row">
          <span class="tip-label">当前链:</span>
          <span>{{ currentChain?.name || '未知' }}</span>
          <span class="tip-label" style="margin-left: 16px;">币种:</span>
          <span>{{ currentCoin?.symbol || '未知' }}</span>
        </div>
      </div>
      <div class="tip-note">
        <Icon icon="mdi:information" style="color: #1890ff; margin-right: 4px;" />
        请输入您要用于打赏的钱包私钥，系统会验证地址和余额
      </div>
      <!-- 私钥输入区域 -->
      <div class="tip-private-key-section">
        <div class="tip-label">
          <Icon icon="mdi:key" style="margin-right: 4px;" />
          打赏账号私钥:
        </div>
        <a-input
          v-model="tipPrivateKey"
          type="password"
          placeholder="请输入用于打赏的钱包私钥"
          show-password
          class="tip-private-key-input"
        />

        <!-- 私钥验证状态 -->
        <div v-if="shouldShowTipWalletStatus" class="tip-wallet-status">
          <div v-if="tipWalletBalance.loading" class="wallet-info-loading">
            <Icon icon="mdi:loading" class="loading-icon" style="color: #1890ff; margin-right: 4px;" />
            正在查询余额...
          </div>
          <div v-else-if="tipWalletBalance.valid" class="wallet-info-valid">
            <div class="wallet-address">
              <Icon icon="mdi:wallet" style="color: #00b42a; margin-right: 4px;" />
              {{ tipWalletBalance.address?.substring(0, 10) }}...{{ tipWalletBalance.address?.slice(-8) }}
            </div>
            <div class="wallet-balance" :class="{ 'insufficient': !tipBalanceSufficient }">
              <Icon icon="mdi:coins" style="margin-right: 4px;" />
              {{ currentCoin?.coin_type === 'base' ? '平台币' : '代币' }}余额:
              {{ tipWalletBalance.balance }} {{ currentCoin?.symbol || 'Token' }}
            </div>
            <div v-if="tipAmount && !tipBalanceSufficient" class="balance-warning">
              <Icon icon="mdi:alert" style="color: #f53f3f; margin-right: 4px;" />
              余额不足，需要 {{ tipAmount }} {{ currentCoin?.symbol || 'Token' }}
            </div>
          </div>
          <div v-else-if="tipWalletBalance.error && tipWalletBalance.hasAttempted" class="wallet-info-invalid">
            <Icon icon="mdi:alert-circle" style="color: #f53f3f; margin-right: 4px;" />
            {{ tipWalletBalance.error }}
          </div>
        </div>
      </div>

      <div class="tip-amount-section">
        <div class="tip-label">打赏金额:</div>
        <div class="tip-amount-options">
          <a-button
            v-for="amount in tipAmountOptions"
            :key="amount"
            type="outline"
            size="mini"
            @click="tipAmount = amount"
            :class="{ 'selected': tipAmount === amount }"
            class="tip-amount-btn"
          >
            {{ amount }}
          </a-button>
        </div>
        <a-input
          v-model="tipAmount"
          placeholder="自定义金额"
          :suffix="currentCoin?.symbol || 'Token'"
          size="small"
          style="margin-top: 8px;"
        >
          <template #suffix>
            {{ currentCoin?.symbol || '未知' }}
          </template>
        </a-input>
      </div>
    </div>

    <template #footer>
      <div class="tip-footer">
        <a-button @click="skipTip" size="large">
          <template #icon>
            <Icon icon="mdi:heart-outline" />
          </template>
          下次一定
        </a-button>
        <a-button
          type="primary"
          @click="sendTip"
          :loading="tipLoading"
          :disabled="!tipWalletBalance.valid || !tipBalanceSufficient || !tipAmount || tipWalletBalance.loading"
          size="large"
          style="margin-left: 12px;"
        >
          <template #icon>
            <Icon icon="mdi:gift" />
          </template>
          {{ tipLoading ? '打赏中...' : '立即打赏' }}
        </a-button>
      </div>
    </template>
  </a-modal>

  <!-- 全页面Loading覆盖层 -->
  <div v-if="pageLoading" class="page-loading-overlay" :class="{ 'with-progress': showImportProgress }">
    <div class="loading-content">
      <a-spin size="large" />
      <div class="loading-text">正在导入文件，请稍候...</div>
      <div v-if="showImportProgress" class="loading-hint">
        请查看页面顶部的进度条了解详细进度
      </div>
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
  background-color: rgb(255, 125, 0);
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

.loading-hint {
  font-size: 14px;
  color: var(--text-color-secondary, #86909c);
  text-align: center;
  margin-top: 8px;
}

/* 当有进度条时，调整loading遮罩层的透明度 */
.page-loading-overlay.with-progress {
  background-color: var(--overlay-bg, rgba(0, 0, 0, 0.3));
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
  z-index: 10000; /* 设置比loading遮罩层(9999)更高的层级 */
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

/* 庆祝覆盖层样式 */
.celebration-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.9), rgba(16, 185, 129, 0.9));
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 10001;
  animation: celebrationFadeIn 0.5s ease-out;
}

.celebration-content {
  text-align: center;
  color: white;
  animation: celebrationBounce 1s ease-out;
}

.celebration-icon {
  font-size: 120px;
  margin-bottom: 120px;
  animation: celebrationRotate 2s ease-in-out infinite;
}

.celebration-title {
  font-size: 48px;
  font-weight: bold;
  margin-bottom: 16px;
  text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
}

.celebration-subtitle {
  font-size: 24px;
  margin-bottom: 20px;
  opacity: 0.9;
}

.celebration-sparkle {
  font-size: 18px;
  opacity: 0.8;
  animation: celebrationPulse 1.5s ease-in-out infinite;
}

/* 庆祝动画 */
@keyframes celebrationFadeIn {
  from {
    opacity: 0;
    transform: scale(0.8);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes celebrationBounce {
  0%, 20%, 50%, 80%, 100% {
    transform: translateY(0);
  }
  40% {
    transform: translateY(-30px);
  }
  60% {
    transform: translateY(-15px);
  }
}

@keyframes celebrationRotate {
  0%, 100% {
    transform: rotate(0deg);
  }
  25% {
    transform: rotate(-10deg);
  }
  75% {
    transform: rotate(10deg);
  }
}

@keyframes celebrationPulse {
  0%, 100% {
    opacity: 0.8;
    transform: scale(1);
  }
  50% {
    opacity: 1;
    transform: scale(1.05);
  }
}

/* 打赏弹窗样式 */
.tip-modal-content {
  padding: 12px 0;
}

.tip-header {
  text-align: center;
  margin-bottom: 16px;
}

.tip-description {
  font-size: 14px;
  color: var(--text-color-secondary, #86909c);
  line-height: 1.4;
}

.tip-info {
  background: var(--color-fill-2, #f7f8fa);
  padding: 12px;
  border-radius: 6px;
  margin-bottom: 16px;
}

.tip-info-row {
  display: flex;
  align-items: center;
  margin-bottom: 6px;
  font-size: 13px;
}

.tip-info-row:last-child {
  margin-bottom: 0;
}

.tip-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-color, #1d2129);
  margin-right: 8px;
  white-space: nowrap;
}

.tip-address {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: var(--text-color-secondary, #86909c);
}

/* 私钥输入区域样式 */
.tip-private-key-section {
  margin-bottom: 16px;
  margin-top: 10px;
}

.tip-private-key-input {
  margin-top: 6px;
  margin-bottom: 8px;
}

.tip-wallet-status {
  padding: 8px;
  border-radius: 4px;
  font-size: 12px;
}

.wallet-info-loading {
  background: var(--color-primary-light-1, #e8f4ff);
  border: 1px solid var(--color-primary-light-3, #7bc7ff);
  color: var(--color-primary, #165dff);
  display: flex;
  align-items: center;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.wallet-info-valid {
  padding: 8px 15px;
  background: var(--color-success-light-1, #e8f5e8);
  border: 1px solid var(--color-success-light-3, #7bc77b);
}

.wallet-info-invalid {
  background: var(--color-danger-light-1, #ffece8);
  border: 1px solid var(--color-danger-light-3, #f7a9a9);
  color: var(--color-danger, #f53f3f);
  display: flex;
  align-items: center;
}

.wallet-address {
  display: flex;
  align-items: center;
  margin-bottom: 4px;
  font-family: 'Courier New', monospace;
  color: var(--color-success, #00b42a);
  font-size: 12px;
}

.wallet-balance {
  display: flex;
  align-items: center;
  margin-bottom: 4px;
  color: var(--color-success, #00b42a);
  font-size: 12px;
}

.wallet-balance.insufficient {
  color: var(--color-warning, #ff7d00);
}

.balance-warning {
  display: flex;
  align-items: center;
  color: var(--color-danger, #f53f3f);
  font-weight: 500;
  font-size: 12px;
}

.tip-amount-section {
  margin-bottom: 16px;
}

.tip-amount-options {
  display: flex;
  gap: 6px;
  margin: 8px 0;
  flex-wrap: wrap;
}

.tip-amount-btn {
  flex: 1;
  min-width: 60px;
  font-size: 12px;
}

.tip-amount-btn.selected {
  background-color: var(--color-primary-light-1, #e8f4ff);
  border-color: var(--color-primary, #165dff);
  color: var(--color-primary, #165dff);
}

.tip-note {
  display: flex;
  align-items: flex-start;
  font-size: 12px;
  color: var(--text-color-secondary, #86909c);
  background: var(--color-primary-light-1, #e8f4ff);
  padding: 8px;
  border-radius: 4px;
  line-height: 1.3;
}

.tip-footer {
  display: flex;
  justify-content: center;
  gap: 12px;
}


</style>
