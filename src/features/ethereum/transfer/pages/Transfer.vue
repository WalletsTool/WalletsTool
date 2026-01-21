<script setup name="Transfer">
import { ref, reactive, computed, watch, onBeforeMount, onMounted, onBeforeUnmount, nextTick } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { Icon } from '@iconify/vue';
import { IconDelete } from '@arco-design/web-vue/es/icon';
import { defineAsyncComponent } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Notification, Modal } from '@arco-design/web-vue';
import { debounce as customDebounce } from '@/utils/debounce.js';
import TitleBar from '@/components/TitleBar.vue';
import TableSkeleton from '@/components/TableSkeleton.vue';
import VirtualScrollerTable from '@/components/VirtualScrollerTable.vue';
import ChainIcon from '@/components/ChainIcon.vue';

import { useTransfer } from '../composables/useTransfer';
import { useBalanceQuery } from '../composables/useBalanceQuery';
import { useValidation } from '../composables/useValidation';
import { useDataOperations } from '../composables/useDataOperations';
import { useTip } from '../composables/useTip';
import { WINDOW_CONFIG } from '@/utils/windowNames';

const TransferGuide = defineAsyncComponent(() => import('../components/TransferGuide.vue'));

const ChainManagement = defineAsyncComponent(() => import('@/components/ChainManagement.vue'));
const RpcManagement = defineAsyncComponent(() => import('@/components/RpcManagement.vue'));
const TokenManagement = defineAsyncComponent(() => import('@/components/TokenManagement.vue'));
const WalletImportModal = defineAsyncComponent(() => import('@/components/WalletImportModal.vue'));
const ProxyConfigModal = defineAsyncComponent(() => import('@/components/ProxyConfigModal.vue'));

const router = useRouter();
const route = useRoute();

const windowTitle = ref('批量转账');

// 窗口标题初始化
function initWindowTitle() {
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
    console.error('初始化窗口标题失败:', e)
  }
  
  // 不再设置默认标题，由后端设置或页面多开功能设置
  // 后端会设置正确格式的标题，前端无需覆盖
}

initWindowTitle()

const columns = [
  { title: '序号', align: 'center', width: 55, slotName: 'index' },
  { title: '发送方私钥', align: 'center', width: 230, dataIndex: 'private_key', ellipsis: true, tooltip: true },
  { title: '接收地址', align: 'center', width: 230, dataIndex: 'to_addr', ellipsis: true, tooltip: true },
  { title: '转账数量', align: 'center', dataIndex: 'amount', width: 85, ellipsis: true, tooltip: true },
  { title: '平台币余额', align: 'center', dataIndex: 'plat_balance', width: 95, ellipsis: true, tooltip: true },
  { title: '代币余额', align: 'center', dataIndex: 'coin_balance', width: 85, ellipsis: true, tooltip: true },
  { title: '状态', align: 'center', slotName: 'exec_status', width: 100, ellipsis: true, tooltip: true },
  { title: '返回信息', align: 'center', dataIndex: 'error_msg', ellipsis: true, tooltip: true },
  { title: '操作', align: 'center', slotName: 'optional', width: 55, ellipsis: true, tooltip: true },
];

let tableLoading = ref(false);
let pageLoading = ref(false);
const data = ref([]);
const selectedKeys = ref([]);
const rowSelection = reactive({ type: 'checkbox', showCheckedAll: true, onlyCurrent: false });

function rowClick(record, event) {
  const index = selectedKeys.value.indexOf(record.key);
  index >= 0 ? selectedKeys.value.splice(index, 1) : selectedKeys.value.push(record.key);
}

let tableBool = ref(true);
const chainValue = ref('');
const currentChain = ref({});
const chainFieldNames = { value: 'key', label: 'scan_url' };
let chainOptions = ref([]);
let coinValue = ref('');
const coinFieldNames = { value: 'key', label: 'label' };
const coinOptions = ref([]);
let balanceLoading = ref(false);
let balanceStopFlag = ref(false);
let balanceStopStatus = ref(true);

const form = reactive({
  send_type: '3',
  amount_from: '2',
  send_count: '0',
  send_min_count: '1',
  send_max_count: '100',
  gas_price_type: '1',
  gas_price: '30',
  gas_price_rate: '5',
  max_gas_price: '',
  limit_type: '1',
  limit_count: '21000',
  limit_min_count: '21000',
  limit_max_count: '30000',
  min_interval: '1',
  max_interval: '3',
  amount_precision: '6',
  error_retry: '0',
});

let addCoinVisible = ref(false);
let coinAddress = ref('');
let deleteTokenVisible = ref(false);
const chainManageRef = ref(null);
const rpcManageRef = ref(null);
const tokenManageRef = ref(null);
const walletImportRef = ref(null);
const proxyConfigRef = ref(null);
const proxyConfigVisible = ref(false);
const proxyEnabled = ref(false);
const proxyStatus = ref('未配置');
const proxyCount = ref(0);
const guideVisible = ref(false);
const advancedFilterVisible = ref(false);
const filterForm = reactive({
  platBalanceOperator: 'gt',
  platBalanceValue: '',
  coinBalanceOperator: 'gt',
  coinBalanceValue: '',
  errorMsg: '',
});

let deleteItemVisible = ref(false);
let currentCoin = ref({});
let currentItemKey = ref('');
let currentItemPrivateKey = ref('');
let startLoading = ref(false);
let stopFlag = ref(false);
let stopStatus = ref(true);
let hasExecutedTransfer = ref(false);
let transferSessionCompleted = ref(true);
const transferConfirmVisible = ref(false);
const transferConfirmLoading = ref(false);
const isTransferConfirmAction = ref(false);
let threadCount = ref(1);
let enableMultiThread = ref(false);
let multiWindowCount = ref(1);

const isSidePanelExpanded = ref(false);

const floatingActionBarStyle = computed(() => {
  const sidePanelWidth = isSidePanelExpanded.value ? 60 : 0;
  return { '--side-panel-offset': `${sidePanelWidth}px` };
});

function expandSidePanel() { isSidePanelExpanded.value = true; }
function collapseSidePanel() { isSidePanelExpanded.value = false; }

function handleGlobalPaste(event) {
  const clipboardData = event.clipboardData || window.clipboardData;
  const hasImage = clipboardData && clipboardData.types && clipboardData.types.includes('Files');
  if (hasImage) {
    event.preventDefault();
    Notification.error({ title: '不支持粘贴图片', content: 'ERROR: Cannot read "clipboard"' });
  }
}

const dataValidationCache = ref({ lastDataLength: 0, lastFormState: '', isValid: false, invalidReason: '', cacheTime: 0 });
const transferStartTime = ref(null);

const transferProgress = ref(0);
const transferTotal = ref(0);
const transferCompleted = ref(0);
const showProgress = ref(false);

const balanceProgress = ref(0);
const balanceTotal = ref(0);
const balanceCompleted = ref(0);
const showBalanceProgress = ref(false);

const toAddressBalanceProgress = ref(0);
const toAddressBalanceTotal = ref(0);
const toAddressBalanceCompleted = ref(0);
const showToAddressBalanceProgress = ref(false);

const actionClickStates = ref({});
const rowHoverStates = ref({});

function setActionClickState(record, actionType) {
  const key = record.key;
  if (!actionClickStates.value[key]) actionClickStates.value[key] = {};
  actionClickStates.value[key][actionType] = true;
  setTimeout(() => {
    if (actionClickStates.value[key]) actionClickStates.value[key][actionType] = false;
  }, 500);
}

const transferConfig = computed(() => ({
  error_count_limit: 3,
  error_retry: form.error_retry,
  chain: chainValue.value,
  chainLayer: currentChain.value.layer,
  l1: currentChain.value.l1,
  scalar: currentChain.value.scalar,
  delay: [Number(form.min_interval) || 1, Number(form.max_interval) || 3],
  transfer_type: form.send_type,
  transfer_amount_list: [Number(form.send_min_count) || 0, Number(form.send_max_count) || 0],
  left_amount_list: [Number(form.send_min_count) || 0, Number(form.send_max_count) || 0],
  amount_precision: Number(form.amount_precision) || 6,
  limit_type: form.limit_type,
  limit_count: Number(form.limit_count) || 21000,
  limit_count_list: [Number(form.limit_min_count) || 21000, Number(form.limit_max_count) || 30000],
  gas_price_type: form.gas_price_type,
  gas_price_rate: (Number(form.gas_price_rate) || 5) / 100,
  gas_price: Number(form.gas_price) || 30,
  max_gas_price: Number(form.max_gas_price) || 0,
  window_id: currentWindowId.value || null,
}));

const transferStatistics = computed(() => ({
  total: data.value.length,
  pending: data.value.filter((item) => item.exec_status === '0').length,
  processing: data.value.filter((item) => item.exec_status === '1').length,
  succeeded: data.value.filter((item) => item.exec_status === '2').length,
  failed: data.value.filter((item) => item.exec_status === '3').length,
}));

const filteredTransferData = computed(() => {
  if (!filterForm.platBalanceValue && !filterForm.coinBalanceValue && !filterForm.errorMsg) return data.value;
  return data.value.filter((item) => {
    if (filterForm.platBalanceValue && filterForm.platBalanceValue.trim() !== '') {
      const platBalanceValue = parseFloat(filterForm.platBalanceValue);
      const itemPlatBalance = parseFloat(item.plat_balance || 0);
      if (filterForm.platBalanceOperator === 'gt' && itemPlatBalance <= platBalanceValue) return false;
      if (filterForm.platBalanceOperator === 'eq' && itemPlatBalance !== platBalanceValue) return false;
      if (filterForm.platBalanceOperator === 'lt' && itemPlatBalance >= platBalanceValue) return false;
    }
    if (filterForm.coinBalanceValue && filterForm.coinBalanceValue.trim() !== '') {
      const coinBalanceValue = parseFloat(filterForm.coinBalanceValue);
      const itemCoinBalance = parseFloat(item.coin_balance || 0);
      if (filterForm.coinBalanceOperator === 'gt' && itemCoinBalance <= coinBalanceValue) return false;
      if (filterForm.coinBalanceOperator === 'eq' && itemCoinBalance !== coinBalanceValue) return false;
      if (filterForm.coinBalanceOperator === 'lt' && itemCoinBalance >= coinBalanceValue) return false;
    }
    if (filterForm.errorMsg && filterForm.errorMsg.trim() !== '') {
      const errorMsg = item.error_msg || '';
      if (!errorMsg.toLowerCase().includes(filterForm.errorMsg.toLowerCase())) return false;
    }
    return true;
  });
});

const chainSearchKeyword = ref('');
const tokenSearchKeyword = ref('');
const chainSelectorExpanded = ref(false);
const tokenSelectorExpanded = ref(false);

const filteredChainOptions = computed(() => {
  if (!chainSearchKeyword.value.trim()) {
    return chainOptions.value || [];
  }
  const keyword = chainSearchKeyword.value.toLowerCase();
  return (chainOptions.value || []).filter(
    (chain) =>
      (chain.name && chain.name.toLowerCase().includes(keyword)) ||
      (chain.key && chain.key.toLowerCase().includes(keyword)) ||
      (chain.scan_url && chain.scan_url.toLowerCase().includes(keyword))
  );
});

const filteredCoinOptions = computed(() => {
  if (!tokenSearchKeyword.value.trim()) {
    return coinOptions.value || [];
  }
  const keyword = tokenSearchKeyword.value.toLowerCase();
  return (coinOptions.value || []).filter(
    (coin) =>
      (coin.label && coin.label.toLowerCase().includes(keyword)) ||
      (coin.symbol && coin.symbol.toLowerCase().includes(keyword)) ||
      (coin.key && coin.key.toLowerCase().includes(keyword))
  );
});

const isOperationInProgress = computed(() => balanceLoading.value || startLoading.value);

function toggleChainSelector() {
  if (isOperationInProgress.value) {
    Notification.warning({ content: '执行过程中无法切换区块链', position: 'topLeft' });
    return;
  }
  chainSelectorExpanded.value = !chainSelectorExpanded.value;
  tokenSelectorExpanded.value = false;
  if (chainSelectorExpanded.value) {
    nextTick(() => {
      chainSearchInputRef.value?.focus();
    });
  }
}

function toggleTokenSelector() {
  if (isOperationInProgress.value) {
    Notification.warning({ content: '执行过程中无法切换代币', position: 'topLeft' });
    return;
  }
  if (!chainValue.value) {
    return;
  }
  tokenSelectorExpanded.value = !tokenSelectorExpanded.value;
  chainSelectorExpanded.value = false;
  if (tokenSelectorExpanded.value) {
    nextTick(() => {
      tokenSearchInputRef.value?.focus();
    });
  }
}

function handleChainSelect(chainKey) {
  chainValue.value = chainKey;
  const chain = chainOptions.value.find((c) => c.key === chainKey);
  if (chain) currentChain.value = chain;
  chainSelectorExpanded.value = false;
  tokenSelectorExpanded.value = true;
  chainChange();
}

function handleTokenSelect(tokenKey) {
  coinValue.value = tokenKey;
  const coin = coinOptions.value.find((c) => c.key === tokenKey);
  if (coin) currentCoin.value = coin;
  tokenSelectorExpanded.value = false;
}

const debouncedFilterUpdate = customDebounce(() => {}, 300);

const gasPriceMonitoring = ref(false);
const gasPriceCountdown = ref(0);
const currentGasPrice = ref(0);
const gasPriceTimer = ref(null);
const transferPaused = ref(false);
const pausedTransferData = ref(null);

let timer = null;
let currentWindowId = ref('');

const uploadInputRef = ref(null);
const formRef = ref(null);
const chainSearchInputRef = ref(null);
const tokenSearchInputRef = ref(null);

function updateTransferProgress() {
  if (!showProgress.value) return;
  const completed = data.value.filter((item) => item.exec_status === '2' || item.exec_status === '3').length;
  transferCompleted.value = completed;
  transferProgress.value = transferTotal.value > 0 ? Number((completed / transferTotal.value).toFixed(2)) : 0;
  if (completed === transferTotal.value && transferTotal.value > 0) {
    setTimeout(() => { showProgress.value = false; }, 3000);
  }
}

function updateBalanceProgress() {
  if (!showBalanceProgress.value) return;
  const completed = data.value.filter((item) => (item.plat_balance !== '' && item.plat_balance !== null) || (item.coin_balance !== '' && item.coin_balance !== null) || item.exec_status === '3').length;
  balanceCompleted.value = completed;
  balanceProgress.value = balanceTotal.value > 0 ? Number((completed / balanceTotal.value).toFixed(4)) : 0;
  if (completed === balanceTotal.value && balanceTotal.value > 0) {
    setTimeout(() => { showBalanceProgress.value = false; }, 3000);
  }
}

function updateToAddressBalanceProgress() {
  if (!showToAddressBalanceProgress.value) return;
  const itemsWithToAddr = data.value.filter((item) => item.to_addr);
  const completed = itemsWithToAddr.filter((item) => (item.plat_balance !== '' && item.plat_balance !== null) || (item.coin_balance !== '' && item.coin_balance !== null) || item.exec_status === '3').length;
  toAddressBalanceCompleted.value = completed;
  toAddressBalanceProgress.value = toAddressBalanceTotal.value > 0 ? Number((completed / toAddressBalanceTotal.value).toFixed(4)) : 0;
  if (completed === toAddressBalanceTotal.value && toAddressBalanceTotal.value > 0) {
    setTimeout(() => { showToAddressBalanceProgress.value = false; }, 3000);
  }
}

function openMultipleWindow() {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (!isTauri) {
    Notification.warning({ content: '此功能仅在桌面应用中可用', position: 'topLeft' });
    return;
  }
  const windowCount = multiWindowCount.value;
  if (windowCount < 1 || windowCount > 9) {
    Notification.warning({ content: '窗口数量必须在1-9之间', position: 'topLeft' });
    return;
  }
  const currentConfig = { chainValue: chainValue.value, coinValue: coinValue.value, form: { ...form }, threadCount: threadCount.value, enableMultiThread: enableMultiThread.value, data: data.value.map((item) => ({ ...item })) };
  const configKeys = [];
  
  // 使用localStorage获取已使用的业务标签，避免跨窗口访问问题
  let usedLabels = [];
  try {
    const storedLabels = localStorage.getItem('transfer_used_business_labels');
    if (storedLabels) {
      usedLabels = JSON.parse(storedLabels);
    }
  } catch (e) {
    console.error('读取已使用的标签失败:', e);
  }
  
  for (let i = 0; i < windowCount; i++) {
    const windowId = generateWindowId();
    const configKey = `transfer_config_${windowId}`;
    const windowLabel = `${getCurrentWindow().label}_multi_${windowId}`;
    
    // 为每个窗口生成唯一的业务标签
    const businessLabel = WINDOW_CONFIG.suggestBusinessLabel('transfer', usedLabels);
    if (businessLabel && !usedLabels.includes(businessLabel)) {
      usedLabels.push(businessLabel);
    }
    
    configKeys.push({ configKey, windowId, windowLabel, businessLabel });
    localStorage.setItem(configKey, JSON.stringify(currentConfig));
  }
  
  // 保存更新后的标签列表到localStorage
  try {
    localStorage.setItem('transfer_used_business_labels', JSON.stringify(usedLabels));
  } catch (e) {
    console.error('保存标签列表失败:', e);
  }
  
  let openedCount = 0;
  let errorCount = 0;
  
  for (let i = 0; i < configKeys.length; i++) {
    const { configKey, windowId, windowLabel, businessLabel } = configKeys[i];
    const windowUrl = `/#/transfer?configKey=${configKey}&count=${i + 1}`;
    
    // 使用配置生成窗口标题：统一格式 "WalletsTool - {图标} {功能名} [{序号}]"
    const windowTitle = `WalletsTool - 💸 批量转账 [${i + 1}]`;
    
    const webview = new WebviewWindow(windowLabel, {
      url: windowUrl,
      title: windowTitle,
      width: 1350,
      height: 900,
      resizable: true,
      decorations: false,
      backgroundColor: document.documentElement.getAttribute('data-theme') === 'light' ? '#FFFFFF' : '#2A2A2B',
      skipTaskbar: false,
    });
    
    webview.once('tauri://created', async () => {
      openedCount++;
      
      // 先复制代理配置（同步执行，确保在新窗口加载前完成）
      const sourceWindowLabel = getCurrentWindow().label;
      try {
        console.log('开始复制代理配置到新窗口:', { windowId, windowLabel, sourceWindowLabel });
        await copyProxyConfigToWindow(windowId, windowLabel, sourceWindowLabel);
        console.log('代理配置复制完成');
      } catch (error) {
        console.error('复制代理配置失败:', error);
      }
      
      Notification.success({ content: `已打开新窗口: ${windowTitle} (${openedCount}/${windowCount})`, position: 'topLeft' });
    });
    webview.once('tauri://error', (e) => {
      errorCount++;
      console.error(`打开窗口 ${windowId} 失败:`, e);
      Notification.error({ content: `打开窗口 ${windowTitle} 失败`, position: 'topLeft' });
      localStorage.removeItem(configKey);
    });
  }
}

function applySharedConfig(config) {
  if (!config) return;
  if (config.chainValue) {
    chainValue.value = config.chainValue;
    const chain = chainOptions.value.find((c) => c.key === config.chainValue);
    if (chain) currentChain.value = chain;
  }
  if (config.coinValue) {
    coinValue.value = config.coinValue;
    const coin = coinOptions.value.find((c) => c.key === config.coinValue);
    if (coin) currentCoin.value = coin;
  }
  if (config.form) Object.assign(form, config.form);
  if (config.threadCount) threadCount.value = config.threadCount;
  if (config.enableMultiThread !== undefined) enableMultiThread.value = config.enableMultiThread;
  if (config.data && Array.isArray(config.data)) {
    data.value = config.data.map((item, index) => ({ ...item, key: String(index + 1) }));
  }
  Notification.success({ content: '已应用共享配置', position: 'topLeft' });
}

async function checkGasPriceForTransfer() {
  if (!form.max_gas_price || !form.max_gas_price.trim()) return true;
  const maxGasPrice = Number(form.max_gas_price);
  if (maxGasPrice <= 0) return true;
  try {
    const res = await invoke('get_chain_gas_price', { chain: chainValue.value });
    const gasPrice = res?.gas_price_gwei || 0;
    currentGasPrice.value = gasPrice;
    if (gasPrice > maxGasPrice) {
      if (!transferPaused.value && !stopFlag.value && startLoading.value) {
        transferPaused.value = true;
        Notification.warning({ content: `Gas价格 ${gasPrice.toFixed(3)} Gwei 超过设定上限 ${maxGasPrice} Gwei，转账已暂停`, position: 'topLeft' });
      }
      return false;
    } else {
      if (transferPaused.value) {
        transferPaused.value = false;
        stopGasPriceMonitoring();
        Notification.success({ content: `Gas价格 ${gasPrice.toFixed(3)} Gwei 已降至设定范围内，转账将自动恢复`, position: 'topLeft' });
        if (pausedTransferData.value) await resumeTransfer();
      }
      return true;
    }
  } catch (error) {
    currentGasPrice.value = 0;
    return true;
  }
}

async function startGasPriceMonitoring() {
  if (gasPriceMonitoring.value) return;
  gasPriceMonitoring.value = true;
  gasPriceCountdown.value = 10;
  await checkGasPriceForTransfer();
  gasPriceTimer.value = setInterval(async () => {
    gasPriceCountdown.value = 10;
    await checkGasPriceForTransfer();
    const countdownInterval = setInterval(() => {
      gasPriceCountdown.value--;
      if (gasPriceCountdown.value <= 0) clearInterval(countdownInterval);
    }, 1000);
  }, 10000);
}

function stopGasPriceMonitoring() {
  gasPriceMonitoring.value = false;
  gasPriceCountdown.value = 0;
  if (gasPriceTimer.value) {
    clearInterval(gasPriceTimer.value);
    gasPriceTimer.value = null;
  }
}

async function resumeTransfer() {
  if (!pausedTransferData.value) return;
  const { accountData, index } = pausedTransferData.value;
  pausedTransferData.value = null;
  await continueTransferFromIndex(accountData, index);
}

async function continueTransferFromIndex(accountData, startIndex) {
  for (let index = startIndex; index < accountData.length; index++) {
    if (stopFlag.value) { stopStatus.value = true; return; }
    const item = accountData[index];
    if (item.exec_status !== '0') continue;
    if (form.max_gas_price && form.max_gas_price.trim()) {
      const gasPriceOk = await checkGasPriceForTransfer();
      if (!gasPriceOk) {
        pausedTransferData.value = { accountData, index };
        await startGasPriceMonitoring();
        while (transferPaused.value && !stopFlag.value) await new Promise((resolve) => setTimeout(resolve, 1000));
        if (stopFlag.value) { stopStatus.value = true; return; }
      }
    }
    const realIndex = data.value.findIndex((dataItem) => dataItem.key === item.key);
    if (realIndex === -1) continue;
    const config = { ...transferConfig.value };
    try {
      if (currentCoin.value.coin_type === 'base') {
        data.value[realIndex].exec_status = '1';
        const res = await invoke('base_coin_transfer', { index: realIndex + 1, item, config });
        if (typeof res === 'object' && res !== null) {
          if (res.success && res.tx_hash) { data.value[realIndex].exec_status = '2'; data.value[realIndex].error_msg = res.tx_hash; }
          else { data.value[realIndex].exec_status = '3'; data.value[realIndex].error_msg = res.error || '转账失败'; }
        } else { data.value[realIndex].exec_status = '2'; data.value[realIndex].error_msg = String(res || '转账成功'); }
        updateTransferProgress();
      } else if (currentCoin.value.coin_type === 'token') {
        data.value[realIndex].exec_status = '1';
        const res = await invoke('token_transfer', { index: realIndex + 1, item, config: { ...config, contract_address: currentCoin.value.contract_address, abi: currentCoin.value.abi } });
        if (typeof res === 'object' && res !== null) {
          if (res.success && res.tx_hash) { data.value[realIndex].exec_status = '2'; data.value[realIndex].error_msg = res.tx_hash; }
          else { data.value[realIndex].exec_status = '3'; data.value[realIndex].error_msg = res.error || '转账失败'; }
        } else { data.value[realIndex].exec_status = '2'; data.value[realIndex].error_msg = String(res || '转账成功'); }
        updateTransferProgress();
      }
    } catch (e) {
      data.value[realIndex].exec_status = '3';
      data.value[realIndex].error_msg = e.message || '转账异常';
      updateTransferProgress();
    }
    if (index < accountData.length - 1 && !stopFlag.value) {
      const minDelay = (form.min_interval && form.min_interval.trim() !== '') ? Number(form.min_interval) * 1000 : 1000;
      const maxDelay = (form.max_interval && form.max_interval.trim() !== '') ? Number(form.max_interval) * 1000 : 3000;
      const randomDelay = Math.floor(Math.random() * (maxDelay - minDelay + 1)) + minDelay;
      await new Promise((resolve) => setTimeout(resolve, randomDelay));
    }
  }
  startLoading.value = false;
  stopStatus.value = true;
  transferSessionCompleted.value = true;
}

watch(enableMultiThread, (newValue) => { if (newValue === '0' || newValue === false) threadCount.value = 1; });

const tokenTableLoading = ref(false);
const tokenManageData = ref([]);
const tokenFormVisible = ref(false);
const isTokenEditMode = ref(false);
const currentEditToken = ref(null);
const tokenForm = reactive({ key: '', name: '', symbol: '', decimals: 18, type: 'token', contract_type: '', contract_address: '', abi: '' });

const { validateForm: validateFormFn, checkSendType, checkPrecision, checkGasPrice, checkGasLimit, checkDelay } = useValidation({ form, formRef });

const { importProgress, importTotal, importCompleted, showImportProgress, importProgressText, validatePrivateKey, validateAddress, updateImportProgress, processBatchData, UploadFile, upload, triggerFileUpload, downloadFile, downloadTemplate: downloadTemplateFn, exportPrivateKeyAddress, clearData: clearDataFn, deleteItem: deleteItemFn } = useDataOperations({
  data, uploadInputRef, clearValidationCache,
});

const { showCelebration, showTipModal, tipAmount, tipPrivateKey, tipLoading, developerAddress, tipAmountOptions, tipMode, showQRCode, showPrivateKeyInput, qrCodeDataURL, tipWalletBalance, tipBalanceSufficient, shouldShowTipWalletStatus, generateQRCode, switchTipMode, copyAddressToClipboard, triggerCelebration, skipTip, sendTip, copyDeveloperAddress, queryTipWalletBalance } = useTip({
  chainValue, currentChain, currentCoin,
});

function fetchGas() {
  const temp = chainValue.value;
  if (!currentChain.value) return;
  if (temp === 'sol') { currentChain.value.gas_price = ''; return; }
  invoke('get_chain_gas_price', { chain: chainValue.value })
    .then((res) => {
      if (temp === chainValue.value && currentChain.value) {
        const gasPrice = res?.gas_price_gwei;
        currentChain.value.gas_price = isNaN(gasPrice) ? '数据格式错误' : chainValue.value === 'eth' ? gasPrice.toFixed(3) : gasPrice.toFixed(7);
      }
    })
    .catch((err) => { if (currentChain.value) currentChain.value.gas_price = '查询错误'; });
}

function startGasTimer() {
  if (timer) clearInterval(timer);
  timer = setInterval(fetchGas, 5000);
}

function stopGasTimer() {
  if (timer) { clearInterval(timer); timer = null; }
}

async function chainChange() {
  const chainResult = chainOptions.value.filter((item) => item.key === chainValue.value);
  if (chainResult.length > 0) {
    currentChain.value = chainResult[0];
    currentChain.value.gas_price = '查询中...';
    fetchGas();
    startGasTimer();
    try {
      const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
      if (isTauri) {
        const tokenList = await invoke('get_coin_list', { chainKey: chainValue.value });
        coinOptions.value = tokenList.map((token) => ({ key: token.key, label: token.label, symbol: token.symbol, contract_address: token.contract_address, decimals: token.decimals, coin_type: token.coin_type, contract_type: token.contract_type, abi: token.abi }));
        coinOptions.value.sort((a, b) => {
          if (a.coin_type === 'base' && b.coin_type !== 'base') return -1;
          if (a.coin_type !== 'base' && b.coin_type === 'base') return 1;
          return a.label.localeCompare(b.label);
        });
        if (coinOptions.value.length > 0) { coinValue.value = coinOptions.value[0].key; currentCoin.value = coinOptions.value[0]; }
      }
    } catch (error) { console.error('加载代币列表失败:', error); coinOptions.value = []; coinValue.value = ''; currentCoin.value = null; }
  } else {
    currentChain.value = null;
    coinOptions.value = [];
    coinValue.value = '';
    currentCoin.value = null;
    stopGasTimer();
  }
}

async function coinChange(value) {
  currentCoin.value = coinOptions.value.filter((item) => item.key === value)[0];
}

function openBlockchainScan() {
  if (currentChain.value?.scan_url) {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      import('@tauri-apps/plugin-shell').then(({ open }) => open(currentChain.value.scan_url)).catch((error) => console.error('打开浏览器失败:', error));
    } else { window.open(currentChain.value.scan_url, '_blank'); }
  }
}

function handleAddCoinCancel() { addCoinVisible.value = false; }

async function addCoinFunc() {
  try {
    const tokenInfo = await invoke('get_token_info', { chain: chainValue.value, contractAddress: coinAddress.value });
    const json = { key: tokenInfo.symbol.toLowerCase(), coin: tokenInfo.symbol, type: 'token', contract_type: '', contract_address: coinAddress.value, abi: tokenInfo.abi };
    await invoke('add_coin', { chain: chainValue.value, objJson: JSON.stringify(json) });
    addCoinVisible.value = false;
    coinAddress.value = '';
    return Promise.resolve();
  } catch (err) { return Promise.reject(err.toString() || '添加代币失败！'); }
}

const handleAddCoinBeforeOk = async () => {
  coinAddress.value = coinAddress.value.trim();
  if (!coinAddress.value) { Notification.warning({ content: '请输入代币地址！', position: 'topLeft' }); return false; }
  let flag = false;
  await addCoinFunc().then(() => { Notification.success({ content: '添加代币成功！', position: 'topLeft' }); flag = true; }).catch((err) => Notification.error(err));
  chainChange();
  return flag;
};

function clearData() { clearDataFn({ startLoading, balanceLoading }); }

function handleManualImport() {
  if (walletImportRef.value) walletImportRef.value.show();
}

function handleFileUpload() { upload(); }

async function downloadTemplateAction() { downloadTemplateFn(); }

function exportAllData() {
  exportPrivateKeyAddress(data.value, { isSelected: false });
}

function exportSelectedData() {
  if (selectedKeys.value.length === 0) {
    Notification.warning({ content: '请先选择要导出的数据！', position: 'topLeft' });
    return;
  }
  const selectedData = data.value.filter((item) => selectedKeys.value.includes(item.key));
  exportPrivateKeyAddress(selectedData, { isSelected: true });
}

function deleteItem(item) {
  if (startLoading.value) { Notification.warning({ content: '请停止或等待执行完成后再删除数据！', position: 'topLeft' }); return; }
  const result = deleteItemFn(item, { startLoading });
  Object.assign(deleteItemVisible, result.deleteItemVisible);
  Object.assign(currentItemKey, result.currentItemKey);
  Object.assign(currentItemPrivateKey, result.currentItemPrivateKey);
}

function deleteItemCancel() { deleteItemVisible.value = false; }

async function deleteItemConfirm() {
  deleteItemVisible.value = false;
  data.value = data.value.filter((obj) => currentItemKey.value !== obj.key);
  Notification.success({ content: '删除成功！', position: 'topLeft' });
}

async function queryFromAddressBalance(item) {
  try {
    const address = item.address || item.from_addr;
    if (!address) { Notification.warning({ content: '无法获取出账账号地址', position: 'topLeft' }); return; }
    let balance = 0;
    if (currentCoin.value?.coin_type === 'base') {
      const result = await invoke('query_balance', { chain: chainValue.value, address });
      balance = typeof result === 'string' ? parseFloat(result || 0) : typeof result === 'number' ? result : 0;
    } else if (currentCoin.value?.coin_type === 'token') {
      const result = await invoke('query_balances_simple', { params: { chain: chainValue.value, coin_config: { coin_type: currentCoin.value.coin_type, contract_address: currentCoin.value.contract_address || null, abi: currentCoin.value.abi || null }, items: [{ key: address, address, private_key: null, plat_balance: null, coin_balance: null, nonce: null, exec_status: '0', error_msg: null, retry_flag: false }], only_coin_config: true, thread_count: 1 } });
      if (result?.success && result.items?.length > 0) {
        const firstItem = result.items[0];
        if (firstItem.exec_status === '2') balance = parseFloat(firstItem.coin_balance || 0);
        else throw new Error(firstItem.error_msg || '代币余额查询失败');
      } else throw new Error('代币余额查询失败');
    }
    const coinSymbol = currentCoin.value?.coin_symbol || (currentCoin.value?.coin_type === 'base' ? 'ETH' : '代币');
    const walletShort = item.private_key ? item.private_key.substring(0, 8) + '...' : address.substring(0, 8) + '...';
    Notification.success({ title: '出账账号余额', content: `钱包: ${walletShort}\n余额: ${balance} ${coinSymbol}`, duration: 4000, position: 'topLeft' });
  } catch (error) { Notification.error('查询出账账号余额失败: ' + error); }
}

async function queryToAddressBalanceRow(item) {
  try {
    const address = item.to_addr;
    if (!address) { Notification.warning({ content: '无法获取到账账号地址', position: 'topLeft' }); return; }
    let balance = 0;
    if (currentCoin.value?.coin_type === 'base') {
      const result = await invoke('query_balance', { chain: chainValue.value, address });
      balance = typeof result === 'string' ? parseFloat(result || 0) : typeof result === 'number' ? result : 0;
    } else if (currentCoin.value?.coin_type === 'token') {
      const result = await invoke('query_balances_simple', { params: { chain: chainValue.value, coin_config: { coin_type: currentCoin.value.coin_type, contract_address: currentCoin.value.contract_address || null, abi: currentCoin.value.abi || null }, items: [{ key: address, address, private_key: null, plat_balance: null, coin_balance: null, nonce: null, exec_status: '0', error_msg: null, retry_flag: false }], only_coin_config: true, thread_count: 1 } });
      if (result?.success && result.items?.length > 0) {
        const firstItem = result.items[0];
        if (firstItem.exec_status === '2') balance = parseFloat(firstItem.coin_balance || 0);
        else throw new Error(firstItem.error_msg || '代币余额查询失败');
      } else throw new Error('代币余额查询失败');
    }
    const coinSymbol = currentCoin.value?.coin_symbol || (currentCoin.value?.coin_type === 'base' ? 'ETH' : '代币');
    const walletShort = address.substring(0, 8) + '...';
    Notification.success({ title: '到账账号余额', content: `钱包: ${walletShort}\n余额: ${balance} ${coinSymbol}`, duration: 4000, position: 'topLeft' });
  } catch (error) { Notification.error('查询到账账号余额失败: ' + error); }
}

async function resendTransaction(item) {
  try {
    if (!item.address || !item.to_addr || !item.amount) { Notification.warning({ content: '缺少必要的转账信息', position: 'topLeft' }); return; }
    if (startLoading.value) { Notification.warning({ content: '请停止或等待执行完成后再操作！', position: 'topLeft' }); return; }
    const index = data.value.findIndex((d) => d.key === item.key);
    if (index === -1) { Notification.warning({ content: '未找到对应数据', position: 'topLeft' }); return; }
    data.value[index].exec_status = '0';
    data.value[index].error_msg = '';
    data.value[index].retry_flag = true;
    Notification.success({ content: '已加入重试队列', position: 'topLeft' });
    if (!startLoading.value) await debouncedStartTransfer();
  } catch (error) { Notification.error('重新发送失败: ' + error); }
}

function deleteTokenCancel() { deleteTokenVisible.value = false; }

async function deleteTokenConfirm() {
  deleteTokenVisible.value = false;
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
    await invoke('remove_coin', { chain: chainValue.value, key: currentCoin.value.key })
      .then(() => { Notification.success({ content: '删除成功！', position: 'topLeft' }); chainChange(); })
      .catch(() => Notification.error({ content: '删除失败！', position: 'topLeft' }));
  } else { Notification.success({ content: '删除成功！', position: 'topLeft' }); chainChange(); }
}

function quickValidateData() {
  const currentDataLength = data.value.length;
  const currentFormState = `${form.send_type}_${form.amount_from}`;
  const currentTime = Date.now();
  if (dataValidationCache.value.lastDataLength === currentDataLength && dataValidationCache.value.lastFormState === currentFormState && currentTime - dataValidationCache.value.cacheTime < 5000) {
    return { isValid: dataValidationCache.value.isValid, reason: dataValidationCache.value.invalidReason };
  }
  let isValid = true; let reason = '';
  const sampleSize = Math.min(100, currentDataLength);
  const step = Math.max(1, Math.floor(currentDataLength / sampleSize));
  for (let i = 0; i < currentDataLength; i += step) {
    const item = data.value[i];
    if (!item.private_key || !item.to_addr) { isValid = false; reason = '存在私钥或地址为空的数据'; break; }
    if (form.send_type === '2' && form.amount_from === '1' && !item.amount) { isValid = false; reason = '存在转账金额为空的数据'; break; }
  }
  dataValidationCache.value = { lastDataLength: currentDataLength, lastFormState: currentFormState, isValid, invalidReason: reason, cacheTime: currentTime };
  return { isValid, reason };
}

function clearValidationCache() {
  dataValidationCache.value = { lastDataLength: 0, lastFormState: '', isValid: false, invalidReason: '', cacheTime: 0 };
}

async function resetDataStatusAsync() {
  const totalItems = data.value.length;
  if (totalItems <= 500) {
    for (let i = 0; i < totalItems; i++) { const item = data.value[i]; item.exec_status = '0'; item.error_msg = ''; item.retry_flag = false; item.error_count = 0; }
    return;
  }
  const batchSize = Math.max(50, Math.min(200, Math.floor(totalItems / 20)));
  for (let i = 0; i < totalItems; i += batchSize) {
    const endIndex = Math.min(i + batchSize, totalItems);
    for (let j = i; j < endIndex; j++) { const item = data.value[j]; item.exec_status = '0'; item.error_msg = ''; item.retry_flag = false; item.error_count = 0; }
    if (i > 0 && i % (batchSize * 5) === 0) await new Promise((resolve) => setTimeout(resolve, 0));
  }
}

function executeTransfer(transferData, resetStatus = true) {
  validateFormFn()
    .then(async () => {
      stopFlag.value = false; stopStatus.value = false;
      hasExecutedTransfer.value = true; transferSessionCompleted.value = false;
      if (resetStatus) { transferStartTime.value = Date.now(); transferTotal.value = data.value.length; transferCompleted.value = 0; transferProgress.value = 0; await resetDataStatusAsync(); }
      showProgress.value = true;
      await transferFnc(transferData);
    })
    .catch(() => { startLoading.value = false; });
}

function startTransfer() {
  if (balanceLoading.value) { startLoading.value = false; Notification.warning({ content: '请等待余额查询完成后再执行！', position: 'topLeft' }); return; }
  if (data.value.length === 0) { startLoading.value = false; Notification.warning({ content: '请先导入钱包信息！', position: 'topLeft' }); return; }
  startLoading.value = true;
  const performValidationAndStart = () => {
    try {
      const quickValidation = quickValidateData();
      if (!quickValidation.isValid) { startLoading.value = false; Notification.warning(quickValidation.reason === '存在私钥或地址为空的数据' ? '请检查是否所有私钥都有对应的转账地址！' : '包含转账金额为空的错误数据请核实！'); return; }
      let hasIncompleteTransfers = hasExecutedTransfer.value && !transferSessionCompleted.value;
      if (hasIncompleteTransfers && stopStatus.value) { startLoading.value = false; transferConfirmVisible.value = true; }
      else { executeTransfer(data.value, true); }
    } catch (error) { console.error('数据验证过程中发生错误:', error); startLoading.value = false; Notification.error({ content: '数据验证失败，请重试', position: 'topLeft' }); }
  };
  if (window.requestIdleCallback) window.requestIdleCallback(performValidationAndStart, { timeout: 100 });
  else setTimeout(performValidationAndStart, 0);
}

function handleTransferConfirmOk() {
  isTransferConfirmAction.value = true;
  transferConfirmLoading.value = true;
  setTimeout(() => {
    const incompleteData = data.value.filter((item) => item.exec_status === '0');
    if (incompleteData.length === 0) { transferConfirmVisible.value = false; transferConfirmLoading.value = false; startLoading.value = false; Notification.info({ content: '所有转账已完成！', position: 'topLeft' }); return; }
    transferConfirmVisible.value = false; transferConfirmLoading.value = false; startLoading.value = true; executeTransfer(incompleteData, false);
  }, 100);
}

function handleTransferConfirmCancel() {
  isTransferConfirmAction.value = true;
  transferConfirmLoading.value = true;
  transferConfirmVisible.value = false;
  transferConfirmLoading.value = false;
  startLoading.value = true;
  setTimeout(() => { executeTransfer(data.value, true); }, 100);
}

function handleTransferConfirmClose() {
  transferConfirmVisible.value = false;
  transferConfirmLoading.value = false;
  if (isTransferConfirmAction.value) {
    isTransferConfirmAction.value = false;
    return;
  }
  startLoading.value = false;
}

const { transferFnc, stopTransfer: stopTransferFn, performIntelligentRetry, iterTransfer, iterTransferFuryMode, retryInProgress, retryResults } = useTransfer({
   data, form, chainValue, currentChain, currentCoin, threadCount, enableMultiThread, transferConfig, transferProgress, transferTotal, transferCompleted, showProgress, startLoading, stopFlag, stopStatus, transferStartTime, hasExecutedTransfer, transferSessionCompleted, updateTransferProgress, checkGasPriceForTransfer, startGasPriceMonitoring, stopGasPriceMonitoring, transferPaused, pausedTransferData,
   validateForm: validateFormFn, quickValidateData, resetDataStatusAsync, clearValidationCache, executeTransfer,
});

const { queryBalance, queryToAddressBalance, stopBalanceQuery, currentQueryId } = useBalanceQuery({
  data, chainValue, currentCoin, threadCount, balanceLoading, balanceStopFlag, balanceStopStatus, balanceTotal, balanceCompleted, balanceProgress, showBalanceProgress, toAddressBalanceTotal, toAddressBalanceCompleted, toAddressBalanceProgress, showToAddressBalanceProgress, updateBalanceProgress, updateToAddressBalanceProgress, windowId: currentWindowId,
});

const debouncedQueryBalance = customDebounce(queryBalance, 500);
const debouncedQueryToAddressBalance = customDebounce(queryToAddressBalance, 500);
const debouncedStartTransfer = customDebounce(startTransfer, 800);
const debouncedStopBalanceQuery = customDebounce(stopBalanceQuery, 300);
const debouncedStopTransfer = customDebounce(stopTransferFn, 300);
const debouncedHandleClick = customDebounce(handleManualImport, 500);
const debouncedClearData = customDebounce(clearData, 600);
const debouncedDeleteItemConfirm = customDebounce(deleteItemConfirm, 400);
const debouncedOpenMultipleWindow = customDebounce(openMultipleWindow, 600);

function selectSucceeded() {
  if (data.value.length === 0) { Notification.warning({ content: '请先导入数据后再进行选择！', position: 'topLeft' }); return; }
  selectedKeys.value = data.value.filter((item) => item.exec_status === '2').map((item) => item.key);
}

function selectFailed() {
  if (data.value.length === 0) { Notification.warning({ content: '请先导入数据后再进行选择！', position: 'topLeft' }); return; }
  selectedKeys.value = data.value.filter((item) => item.exec_status === '3').map((item) => item.key);
}

function InvertSelection() {
  if (data.value.length === 0) { Notification.warning({ content: '请先导入数据后再进行选择！', position: 'topLeft' }); return; }
  selectedKeys.value = data.value.filter((item) => selectedKeys.value.indexOf(item.key) < 0).map((item) => item.key);
}

function showAdvancedFilter() { advancedFilterVisible.value = true; }

function applyAdvancedFilter() {
  let filteredItems = [];
  data.value.forEach((item) => {
    let shouldSelect = true;
    if (filterForm.platBalanceValue && filterForm.platBalanceValue.trim() !== '') {
      const platBalanceValue = parseFloat(filterForm.platBalanceValue);
      const itemPlatBalance = parseFloat(item.plat_balance || 0);
      if (filterForm.platBalanceOperator === 'gt' && itemPlatBalance <= platBalanceValue) shouldSelect = false;
      else if (filterForm.platBalanceOperator === 'eq' && itemPlatBalance !== platBalanceValue) shouldSelect = false;
      else if (filterForm.platBalanceOperator === 'lt' && itemPlatBalance >= platBalanceValue) shouldSelect = false;
    }
    if (shouldSelect && filterForm.coinBalanceValue && filterForm.coinBalanceValue.trim() !== '') {
      const coinBalanceValue = parseFloat(filterForm.coinBalanceValue);
      const itemCoinBalance = parseFloat(item.coin_balance || 0);
      if (filterForm.coinBalanceOperator === 'gt' && itemCoinBalance <= coinBalanceValue) shouldSelect = false;
      else if (filterForm.coinBalanceOperator === 'eq' && itemCoinBalance !== coinBalanceValue) shouldSelect = false;
      else if (filterForm.coinBalanceOperator === 'lt' && itemCoinBalance >= coinBalanceValue) shouldSelect = false;
    }
    if (shouldSelect && filterForm.errorMsg && filterForm.errorMsg.trim()) {
      const errorMsg = item.error_msg || '';
      if (!errorMsg.toLowerCase().includes(filterForm.errorMsg.toLowerCase())) shouldSelect = false;
    }
    if (shouldSelect) filteredItems.push(item.key);
  });
  selectedKeys.value = filteredItems;
  advancedFilterVisible.value = false;
  Notification.success({ content: `筛选完成，共选中 ${filteredItems.length} 条数据`, position: 'topLeft' });
}

function deleteSelected() {
  if (startLoading.value) { Notification.warning({ content: '请停止或等待执行完成后再删除数据！', position: 'topLeft' }); return; }
  if (selectedKeys.value.length === 0) { Notification.warning({ content: '请先选择要删除的项目！', position: 'topLeft' }); return; }
  Modal.confirm({
    title: '确认删除',
    content: `确定要删除选中的 ${selectedKeys.value.length} 个项目吗？此操作不可撤销。`,
    okText: '确认删除',
    cancelText: '取消',
    okButtonProps: { status: 'danger' },
    onOk: () => { data.value = data.value.filter((item) => !selectedKeys.value.includes(item.key)); selectedKeys.value = []; Notification.success({ content: '删除成功', position: 'topLeft' }); },
  });
}

function goHome() { router.push({ name: 'home' }); }

function showTokenManage() {
  if (!chainValue.value) { Notification.warning({ content: '请先选择区块链！', position: 'topLeft' }); return; }
  tokenManageRef.value?.show();
}

async function loadTokenManageData() {
  tokenTableLoading.value = true;
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    const tokenList = isTauri ? await invoke('get_coin_list', { chainKey: chainValue.value }) : [{ key: 'eth', coin: 'ETH', type: 'base', decimals: 18 }, { key: 'usdt', coin: 'USDT', type: 'token', contract_address: '0x...', decimals: 6 }];
    tokenManageData.value = tokenList.map((token) => ({ key: token.key, coin: token.symbol || token.coin || token.label, name: token.label || token.coin, symbol: token.symbol || token.coin || token.label, type: token.type || token.coin_type, contract_type: token.contract_type || '', contract_address: token.contract_address || '', abi: token.abi || '', decimals: token.decimals || 18, label: token.label || token.coin }));
  } catch (error) { console.error('加载代币数据失败:', error); Notification.error('加载代币数据失败：' + error); }
  finally { tokenTableLoading.value = false; }
}

function showAddToken() {
  isTokenEditMode.value = false;
  currentEditToken.value = null;
  Object.assign(tokenForm, { key: '', name: '', symbol: '', decimals: 18, type: 'token', contract_type: '', contract_address: '', abi: '' });
  tokenFormVisible.value = true;
}

function showEditToken(record) {
  isTokenEditMode.value = true;
  currentEditToken.value = record;
  Object.assign(tokenForm, { key: record.key || '', name: record.name || record.coin || record.label || '', symbol: record.symbol || record.coin || record.label || '', decimals: record.decimals || 18, type: record.type || 'token', contract_type: record.contract_type || '', contract_address: record.contract_address || '', abi: record.abi || '' });
  tokenFormVisible.value = true;
}

async function submitTokenForm() {
  try {
    if (!tokenForm.name || !tokenForm.name.trim()) { Notification.warning({ content: '请输入代币名称', position: 'topLeft' }); return false; }
    if (!tokenForm.symbol || !tokenForm.symbol.trim()) { Notification.warning({ content: '请输入代币符号', position: 'topLeft' }); return false; }
    if (!tokenForm.key || !tokenForm.key.trim()) { Notification.warning({ content: '请输入代币标识', position: 'topLeft' }); return false; }
    if (tokenForm.type === 'token' && (!tokenForm.contract_address || !tokenForm.contract_address.trim())) { Notification.warning({ content: '代币类型为token时，合约地址不能为空', position: 'topLeft' }); return false; }
    if (tokenForm.type === 'token' && (!tokenForm.abi || !tokenForm.abi.trim())) { Notification.warning({ content: '代币类型为合约代币时，ABI不能为空', position: 'topLeft' }); return false; }
    if (!tokenForm.decimals || tokenForm.decimals < 0) { Notification.warning({ content: '请输入有效的小数位数', position: 'topLeft' }); return false; }
    if (!isTokenEditMode.value && !tokenForm.key.trim()) tokenForm.key = tokenForm.symbol.toLowerCase();
    const requestData = { key: tokenForm.key, name: tokenForm.name, symbol: tokenForm.symbol, coin_type: tokenForm.type, contract_address: tokenForm.contract_address, decimals: tokenForm.decimals, abi: tokenForm.abi };
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      if (isTokenEditMode.value) { await invoke('update_coin', { chain: chainValue.value, key: tokenForm.key, objJson: JSON.stringify(requestData) }); Notification.success({ content: '编辑代币成功！', position: 'topLeft' }); }
      else { await invoke('add_coin', { chain: chainValue.value, objJson: JSON.stringify(requestData) }); Notification.success({ content: '添加代币成功！', position: 'topLeft' }); }
    } else { Notification.success({ content: isTokenEditMode.value ? '编辑代币成功！' : '添加代币成功！', position: 'topLeft' }); }
    loadTokenManageData();
    await chainChange();
    tokenFormVisible.value = false;
    return true;
  } catch (error) { console.error('代币操作失败:', error); Notification.error('代币操作失败：' + error); return false; }
}

async function deleteTokenFromManage(tokenKey) {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) await invoke('remove_coin', { chain: chainValue.value, key: tokenKey });
    Notification.success({ content: '删除代币成功！', position: 'topLeft' });
    loadTokenManageData();
    await chainChange();
  } catch (error) { console.error('删除代币失败:', error); Notification.error('删除代币失败：' + error); }
}

async function handleChainUpdated() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      const result = await invoke('get_chain_list');
      chainOptions.value = result || [];
      const currentChainExists = chainOptions.value.find((chain) => chain.key === chainValue.value);
      if (!currentChainExists && chainOptions.value.length > 0) { chainValue.value = chainOptions.value[0].key; await chainChange(); }
      else if (currentChainExists) currentChain.value = currentChainExists;
      else { chainValue.value = ''; currentChain.value = null; coinOptions.value = []; coinValue.value = ''; currentCoin.value = null; }
    }
  } catch (error) { console.error('更新链列表失败:', error); Notification.error({ content: '更新链列表失败', position: 'topLeft' }); }
}

function handleRpcUpdated() { console.log('RPC已更新'); }

function handleTokenUpdated() { chainChange(); }

function showRpcManage() {
  if (!chainValue.value) { Notification.warning({ content: '请先选择区块链！', position: 'topLeft' }); return; }
  rpcManageRef.value?.show();
}

function showChainManage() { if (chainManageRef.value) chainManageRef.value.show(); }

function openProxyConfig() { proxyConfigVisible.value = true; }

function handleProxyConfigChange(config) {
  proxyEnabled.value = config.enabled;
  proxyCount.value = config.proxies ? config.proxies.length : 0;
  proxyStatus.value = config.enabled && proxyCount.value > 0 ? '已配置' : '未配置';
  
  // 保存到 localStorage
  const currentWindow = getCurrentWindow();
  const storageKey = `proxy_config_${currentWindow.label}`;
  localStorage.setItem(storageKey, JSON.stringify({
    enabled: config.enabled,
    proxies: config.proxies || []
  }));
}

const proxyStatusColor = computed(() => {
  switch (proxyStatus.value) {
    case '已配置': return '#00b42a';
    case '连接中': return '#ff7d00';
    case '已连接': return '#00b42a';
    case '连接失败': return '#f53f3f';
    default: return '#86909c';
  }
});

async function initProxyStatus() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      // 获取或生成窗口ID
      let windowId = currentWindowId.value;
      if (!windowId || windowId.trim() === '') {
        windowId = generateWindowId();
        currentWindowId.value = windowId;
      }
      
      // 检查是否有持久化的窗口ID
      const currentWindow = await getCurrentWindow();
      const storageKey = `proxy_window_id_${currentWindow.label}`;
      const storedWindowId = localStorage.getItem(storageKey);
      if (storedWindowId) {
        windowId = storedWindowId;
        currentWindowId.value = windowId;
      } else {
        localStorage.setItem(storageKey, windowId);
      }
      
      // 设置窗口ID到后端
      await invoke('set_proxy_window_id', { windowId });
      
      // 尝试从 localStorage 读取配置
      const proxyStorageKey = `proxy_config_${currentWindow.label}`;
      const storedConfig = localStorage.getItem(proxyStorageKey);
      
      let config;
      if (storedConfig) {
        try {
          config = JSON.parse(storedConfig);
          console.log('从 localStorage 加载代理配置:', config);
        } catch (e) {
          console.error('解析代理配置失败:', e);
          config = await invoke('get_proxy_config_for_window', { windowId });
        }
      } else {
        config = await invoke('get_proxy_config_for_window', { windowId });
      }
      
      handleProxyConfigChange(config);
      
      console.log('initProxyStatus 完成:', {
        windowId,
        currentWindowId: currentWindowId.value,
        enabled: config.enabled,
        proxyCount: config.proxies?.length || 0
      });
    }
  } catch (error) { console.error('初始化代理状态失败:', error); }
}

// 复制代理配置到新窗口
async function copyProxyConfigToWindow(newWindowId, newWindowLabel, sourceWindowLabel) {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (!isTauri) return;

    console.log('copyProxyConfigToWindow 开始:', {
      newWindowId,
      newWindowLabel,
      sourceWindowLabel,
      currentWindowId: currentWindowId.value
    });

    // 从源窗口的 localStorage 读取代理配置
    const storageKey = `proxy_config_${sourceWindowLabel}`;
    const storedConfig = localStorage.getItem(storageKey);
    
    let configToCopy = { enabled: false, proxies: [] };
    
    if (storedConfig) {
      try {
        configToCopy = JSON.parse(storedConfig);
        console.log('从源窗口 localStorage 读取到的配置:', configToCopy);
      } catch (e) {
        console.error('解析代理配置失败:', e);
      }
    }
    
    if (configToCopy.proxies && configToCopy.proxies.length > 0) {
      console.log('代理配置有效，开始保存到新窗口');
    } else {
      console.log('代理配置为空或无效，配置将使用默认值');
    }
    
    // 设置新窗口的窗口ID
    await invoke('set_proxy_window_id', { windowId: newWindowId });
    
    // 复制配置到新窗口
    await invoke('save_proxy_config_for_window', {
      windowId: newWindowId,
      proxies: configToCopy.proxies || [],
      enabled: configToCopy.enabled
    });
    
    // 同时保存到新窗口的 localStorage
    const newStorageKey = `proxy_config_${newWindowLabel}`;
    localStorage.setItem(newStorageKey, JSON.stringify({
      enabled: configToCopy.enabled,
      proxies: configToCopy.proxies || []
    }));
    
    // 保存新窗口的窗口ID映射
    const newWindowIdKey = `proxy_window_id_${newWindowLabel}`;
    localStorage.setItem(newWindowIdKey, newWindowId);
    
    console.log(`已复制代理配置到新窗口 ${newWindowLabel}:`, {
      windowId: newWindowId,
      enabled: configToCopy.enabled,
      proxyCount: configToCopy.proxies?.length || 0
    });
  } catch (error) {
    console.error('复制代理配置到新窗口失败:', error);
  }
}

// 生成唯一的窗口ID
function generateWindowId() {
  const timestamp = Date.now().toString(36);
  const randomPart = Math.random().toString(36).substring(2, 9);
  return `window_${timestamp}_${randomPart}`;
}

async function handleBeforeClose() {
  console.log('TitleBar触发关闭事件，正在停止后台操作...');
  if (balanceLoading.value) await stopBalanceQuery();
  if (startLoading.value) await stopTransferFn();
  if (gasPriceMonitoring.value && gasPriceTimer.value) { clearInterval(gasPriceTimer.value); gasPriceTimer.value = null; gasPriceMonitoring.value = false; }
  transferPaused.value = false;
  pausedTransferData.value = null;
  gasPriceCountdown.value = 0;
  currentGasPrice.value = 0;

  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
    try {
      const currentWindow = await getCurrentWindow();
      const windowLabel = currentWindow.label;
      
      // 清除前端localStorage
      localStorage.removeItem(`proxy_config_${windowLabel}`);
      localStorage.removeItem(`proxy_window_id_${windowLabel}`);
      
      // 清除后端文件缓存和内存缓存
      await invoke('clear_proxy_config_for_window', { windowId: windowLabel });
      
      console.log(`已完全清除窗口 ${windowLabel} 的代理配置`);
      
      await currentWindow.destroy();
    } catch (error) {
      console.error('清除代理配置失败:', error);
      const currentWindow = getCurrentWindow();
      await currentWindow.destroy();
    }
  }
}

onBeforeMount(async () => {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  let sharedConfig = null;
  const configKey = route.query.configKey;
  if (configKey) {
    try {
      const configData = localStorage.getItem(configKey);
      if (configData) { sharedConfig = JSON.parse(configData); localStorage.removeItem(configKey); }
    } catch (error) { console.error('读取共享配置失败:', error); }
  }
  if (isTauri) {
    try {
      const result = await invoke('get_chain_list');
      chainOptions.value = result || [];
      chainOptions.value.sort((a, b) => (a.name || '').localeCompare(b.name || ''));
      if (sharedConfig) {
        applySharedConfig(sharedConfig);
      } else {
        const ethChain = chainOptions.value.find((c) => c.key === 'eth');
        if (ethChain) { chainValue.value = 'eth'; currentChain.value = ethChain; }
        else { chainValue.value = chainOptions.value[0]?.key; currentChain.value = chainOptions.value[0]; }
        await chainChange();
      }
    } catch (error) { console.error('初始化链列表失败:', error); }
  } else {
    chainOptions.value = [{ key: 'eth', name: 'Ethereum', scan_url: 'etherscan.io', pic_url: 'eth.png' }, { key: 'bnb', name: 'BNB Chain', scan_url: 'bscscan.com', pic_url: 'bnb.png' }];
    chainOptions.value.sort((a, b) => (a.name || '').localeCompare(b.name || ''));
    if (sharedConfig) applySharedConfig(sharedConfig);
    else { const ethChain = chainOptions.value.find((c) => c.key === 'eth'); if (ethChain) { chainValue.value = 'eth'; currentChain.value = ethChain; } else { chainValue.value = chainOptions.value[0]?.key; currentChain.value = chainOptions.value[0]; } await chainChange(); }
  }
});

onMounted(async () => {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow();
      windowTitle.value = (await currentWindow.title()) || '批量转账';
      currentWindowId.value = currentWindow.label;
      await initProxyStatus();
    } catch (error) { console.error('获取窗口信息失败:', error); }
  } else { windowTitle.value = '批量转账'; currentWindowId.value = 'browser_transfer_window'; }
  document.addEventListener('click', handleClickOutside);
  setTimeout(() => {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) { const currentWindow = getCurrentWindow(); currentWindow.emit('page-loaded'); }
  }, 100);
  
  const guideCompleted = localStorage.getItem('transfer_guide_completed');
  if (!guideCompleted) {
    setTimeout(() => {
      guideVisible.value = true;
    }, 500);
  }
  
   if (isTauri) {
     await listen('balance_item_update', (event) => {
       const { item, window_id, query_id } = event.payload;
       if (window_id && window_id !== currentWindowId.value) return;
       if (balanceStopFlag.value) return;
       if (query_id && String(query_id) !== String(currentQueryId.value)) return;
       const targetIndex = data.value.findIndex((dataItem) => dataItem.key === item.key);
       if (targetIndex !== -1) { data.value[targetIndex].plat_balance = item.plat_balance; data.value[targetIndex].coin_balance = item.coin_balance; data.value[targetIndex].exec_status = item.exec_status; data.value[targetIndex].error_msg = item.error_msg; updateBalanceProgress(); updateToAddressBalanceProgress(); }
     });
     await listen('transfer_status_update', (event) => {
       const { index, error_msg, exec_status, item } = event.payload;
       if (index === 999999) return;
       let targetIndex = -1;
       if (item && item.private_key) targetIndex = data.value.findIndex((dataItem) => dataItem.private_key === item.private_key);
       else targetIndex = index;
       if (targetIndex !== -1 && data.value[targetIndex]) { data.value[targetIndex].error_msg = error_msg; data.value[targetIndex].exec_status = exec_status; updateTransferProgress(); }
     });
   }
});

onBeforeUnmount(async () => {
  document.removeEventListener('click', handleClickOutside);
  if (startLoading.value) { stopFlag.value = true; startLoading.value = false; stopStatus.value = true; Notification.warning({ content: '窗口关闭，已自动停止转账操作', position: 'topLeft' }); }
  stopGasPriceMonitoring();
  stopGasTimer();
  if (gasPriceTimer.value) { clearInterval(gasPriceTimer.value); gasPriceTimer.value = null; }
  transferPaused.value = false; pausedTransferData.value = null; gasPriceMonitoring.value = false; gasPriceCountdown.value = 0; currentGasPrice.value = 0;
});

function handleClickOutside(event) {
  const chainSelector = document.querySelector('.chain-selector-container');
  const tokenSelector = document.querySelector('.token-selector-container');
  const isInChainSelector = chainSelector && chainSelector.contains(event.target);
  const isInTokenSelector = tokenSelector && tokenSelector.contains(event.target);
  if (!isInChainSelector) chainSelectorExpanded.value = false;
  if (!isInTokenSelector) tokenSelectorExpanded.value = false;
}
</script>

<template>
  <TitleBar :title="windowTitle" @before-close="handleBeforeClose" />
  <div class="container transfer" style="height: 100vh; display: flex; flex-direction: column; overflow: hidden" @paste="handleGlobalPaste">
    <div class="toolBar" style="flex-shrink: 0; height: 0; overflow: visible; margin-top: 0">
      <input type="file" ref="uploadInputRef" @change="UploadFile" id="btn_file" style="display: none" />
    </div>
    <div class="main-content" :style="{ gap: isSidePanelExpanded ? '10px' : '0px' }">
      <div class="left-panel" style="flex: 1; display: flex; flex-direction: column; overflow: visible;">
        <div class="table-section" id="table-section" style="flex: 1; display: flex; flex-direction: column; min-height: 0; position: relative">
          <TableSkeleton v-if="(tableLoading || balanceLoading) && data.length === 0" :rows="8" />
            <VirtualScrollerTable :columns="columns" :data="data" :row-selection="rowSelection" :loading="tableLoading" :selected-keys="selectedKeys" @row-click="rowClick" @update:selected-keys="selectedKeys = $event" @open-manual-import="handleManualImport" @open-file-upload="handleFileUpload" @download-template="downloadTemplateAction" row-key="key" height="100%" :empty-data="data.length === 0" class="table-with-side-actions" :class="{ 'expanded': !isSidePanelExpanded }" :hover-keys="Object.keys(rowHoverStates).filter((key) => rowHoverStates[key])">
            <template #exec_status="{ record }">
              <div class="exec-status-wrapper" @mouseenter="rowHoverStates[record.key] = true" @mouseleave="rowHoverStates[record.key] = false">
                <a-tooltip content="" trigger="hover" :mouseEnterDelay="300" :mouseLeaveDelay="100" :popup-style="{ padding: 0, pointerEvents: 'auto' }">
                  <template #content>
                    <div class="exec-actions" @mouseenter="rowHoverStates[record.key] = true" @mouseleave="rowHoverStates[record.key] = false">
                      <div class="action-btn" :class="{ 'action-btn-clicked': actionClickStates[record.key]?.queryFrom }" @click="queryFromAddressBalance(record); setActionClickState(record, 'queryFrom');">
                        <Icon :icon="actionClickStates[record.key]?.queryFrom ? 'mdi:check' : 'mdi:arrow-up'" /> 查出账余额
                      </div>
                      <div class="action-btn" :class="{ 'action-btn-clicked': actionClickStates[record.key]?.queryTo }" @click="queryToAddressBalanceRow(record); setActionClickState(record, 'queryTo');">
                        <Icon :icon="actionClickStates[record.key]?.queryTo ? 'mdi:check' : 'mdi:arrow-down'" /> 查到账余额
                      </div>
                      <div class="action-btn danger" :class="{ 'action-btn-clicked': actionClickStates[record.key]?.resend }" @click="resendTransaction(record); setActionClickState(record, 'resend');">
                        <Icon :icon="actionClickStates[record.key]?.resend ? 'mdi:check' : 'mdi:refresh'" /> 重新转账
                      </div>
                    </div>
                  </template>
                  <a-tag v-if="record.exec_status === '0'" color="#86909c">等待执行</a-tag>
                  <a-tag v-if="record.exec_status === '1'" color="#ff7d00">执行中</a-tag>
                  <a-tag v-if="record.exec_status === '2'" color="#00b42a">执行成功</a-tag>
                  <a-tag v-if="record.exec_status === '3'" color="#f53f3f">执行失败</a-tag>
                </a-tooltip>
              </div>
            </template>
            <template #optional="{ record }">
              <a-button type="text" size="small" @click.stop="deleteItem(record)" status="danger"><template #icon><IconDelete /></template></a-button>
            </template>
          </VirtualScrollerTable>
          <Transition name="progress-slide" appear>
            <div v-if="showImportProgress" class="floating-progress-bar">
              <div class="progress-content">
                <div class="progress-header"><span class="progress-title">{{ importProgressText }}</span><span class="progress-count">{{ importCompleted }} / {{ importTotal }}</span></div>
                <a-progress :percent="importProgress" :show-text="true" :stroke-width="6" :color="{ '0%': '#722ed1', '100%': '#722ed1' }" class="progress-bar" />
              </div>
            </div>
          </Transition>
          <Transition name="progress-slide" appear>
            <div v-if="showProgress" class="floating-progress-bar" :style="{ top: showImportProgress ? '120px' : '45px' }">
              <div class="progress-content">
                <div class="progress-header"><span class="progress-title">转账进度</span><span class="progress-count">{{ transferCompleted }} / {{ transferTotal }}</span></div>
                <a-progress :percent="transferProgress" :show-text="true" :stroke-width="6" :color="{ '0%': '#00b42a', '100%': '#00b42a' }" class="progress-bar" />
              </div>
            </div>
          </Transition>
          <Transition name="progress-slide" appear>
            <div v-if="showBalanceProgress" class="floating-progress-bar" :style="{ top: (showImportProgress && showProgress) ? '220px' : (showImportProgress || showProgress) ? '120px' : '45px' }">
              <div class="progress-content">
                <div class="progress-header"><span class="progress-title">查出账地址进度</span><span class="progress-count">{{ balanceCompleted }} / {{ balanceTotal }}</span></div>
                <a-progress :percent="balanceProgress" :show-text="true" :stroke-width="6" :color="{ '0%': '#1890ff', '100%': '#1890ff' }" class="progress-bar" />
              </div>
            </div>
          </Transition>
          <Transition name="progress-slide" appear>
            <div v-if="showToAddressBalanceProgress" class="floating-progress-bar" :style="{ top: (showImportProgress && showProgress && showBalanceProgress) ? '320px' : ((showImportProgress && showProgress) || (showImportProgress && showBalanceProgress) || (showProgress && showBalanceProgress)) ? '220px' : (showImportProgress || showProgress || showBalanceProgress) ? '120px' : '45px' }">
              <div class="progress-content">
                <div class="progress-header"><span class="progress-title">查到账地址进度</span><span class="progress-count">{{ toAddressBalanceCompleted }} / {{ toAddressBalanceTotal }}</span></div>
                <a-progress :percent="toAddressBalanceProgress" :show-text="true" :stroke-width="6" :color="{ '0%': '#52c41a', '100%': '#52c41a' }" class="progress-bar" />
              </div>
            </div>
          </Transition>
          <div v-if="retryInProgress" style="margin-top: 10px; padding: 10px; background: #f8f9fa; border-radius: 6px; border-left: 4px solid #1890ff; flex-shrink: 0">
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 8px"><a-spin size="small" /><span style="font-size: 14px; color: #1d2129; font-weight: 500">智能重试检查中...</span></div>
            <div style="font-size: 12px; color: #86909c">正在检查失败交易的链上状态，判断是否需要重试</div>
          </div>
          <div v-if="retryResults.length > 0 && !retryInProgress" style="margin-top: 10px; padding: 10px; background: #f6ffed; border-radius: 6px; border-left: 4px solid #52c41a; flex-shrink: 0">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px"><span style="font-size: 14px; color: #1d2129; font-weight: 500">智能重试检查完成</span><a-button size="mini" type="text" @click="retryResults = []"><template #icon><Icon icon="mdi:close" /></template></a-button></div>
            <div style="font-size: 12px; color: #52c41a; margin-bottom: 4px">跳过重试: {{ retryResults.filter((r) => r.action === '跳过重试').length }} 笔 | 加入重试: {{ retryResults.filter((r) => r.action === '加入重试').length }} 笔</div>
          </div>
        </div>
        <div class="action-buttons-section">
          <div class="floating-action-bar">
            <div class="floating-action-content">
              <div v-if="!balanceLoading" class="btn-wrapper">
                <a-dropdown>
                  <a-button type="primary" class="floating-btn primary-btn" :disabled="startLoading">
                    <template #icon><Icon icon="mdi:magnify" class="btn-icon" /></template>
                    <span class="btn-text">查询余额</span>
                  </a-button>
                  <template #content>
                    <a-doption @click="debouncedQueryBalance" :disabled="startLoading" class="dropdown-option">
                      <Icon icon="mdi:account-arrow-right" style="margin-right: 8px; margin-bottom: -2px" />查出账地址
                    </a-doption>
                    <a-doption @click="debouncedQueryToAddressBalance" :disabled="startLoading" class="dropdown-option">
                      <Icon icon="mdi:account-arrow-left" style="margin-right: 8px; margin-bottom: -2px" />查到账地址
                    </a-doption>
                  </template>
                </a-dropdown>
              </div>
              <div v-else class="btn-wrapper btn-stop-wrapper">
                <a-button type="primary" class="floating-btn primary-btn stopping" @click="debouncedStopBalanceQuery">
                  <template #icon><Icon icon="mdi:magnify" /></template>
                  <span class="btn-text btn-text-stop">
                    <span class="btn-text-normal">查询中...</span>
                    <span class="btn-text-hover">停止查询</span>
                  </span>
                </a-button>
              </div>
              <div v-if="!startLoading" class="btn-wrapper">
                <a-button type="success" class="floating-btn success-btn" :disabled="balanceLoading" @click="debouncedStartTransfer">
                  <template #icon><Icon icon="mdi:rocket-launch" /></template>
                  <span class="btn-text">执行转账</span>
                </a-button>
              </div>
              <div v-else class="btn-wrapper btn-stop-wrapper">
                <a-button type="success" class="floating-btn success-btn stopping" @click="debouncedStopTransfer">
                  <template #icon><Icon icon="mdi:rocket-launch" /></template>
                  <span class="btn-text btn-text-stop">
                    <span class="btn-text-normal">转账中...</span>
                    <span class="btn-text-hover">停止转账</span>
                  </span>
                </a-button>
              </div>
            </div>
          </div>
        </div>
        <div class="config-section" id="config-section" style="flex-shrink: 0; padding-top: 35px">
          <a-form ref="formRef" :model="form" :style="{ width: '100%' }" layout="horizontal" label-align="left">
            <a-row class="config-row">
              <div class="config-column column-first">
                <a-form-item field="send_type" label="发送模式" :label-col-props="{ span: 6 }">
                  <a-radio-group v-model="form.send_type" type="button"><a-radio value="1">全部</a-radio><a-radio value="2">指定数值</a-radio><a-radio value="3">范围随机</a-radio><a-radio value="4">剩余随机</a-radio></a-radio-group>
                </a-form-item>
                <a-form-item v-if="form.send_type === '2'" field="amount_from" label="数量来源" :label-col-props="{ span: 6 }">
                  <a-radio-group v-model="form.amount_from" type="button"><a-radio value="1">表格数据</a-radio><a-radio value="2">自定义</a-radio></a-radio-group>
                </a-form-item>
                <a-form-item v-if="form.send_type === '2' && form.amount_from === '2'" field="send_count" label="发送数量" :label-col-props="{ span: 6 }">
                  <a-input v-model="form.send_count" />
                </a-form-item>
                <a-form-item v-if="form.send_type === '3' || form.send_type === '4'" field="send_count_scope" :label="form.send_type === '3' ? '发送数量从' : '剩余数量从'" :label-col-props="{ span: 6 }">
                  <a-space><a-input v-model="form.send_min_count" placeholder="最小" style="width: 66px" /><span style="margin: 0 8px">至</span><a-input v-model="form.send_max_count" placeholder="最大" style="width: 85px" /><span style="margin-left: 10px">范围内随机生成</span></a-space>
                </a-form-item>
                <a-form-item v-if="form.send_type === '3' || form.send_type === '4'" field="amount_precision" label="金额精度" :label-col-props="{ span: 6 }">
                  <a-input v-model="form.amount_precision" />
                </a-form-item>
              </div>
              <div class="config-divider"></div>
              <div class="config-column column-second">
                <a-form-item field="limit_type" label="Gas Limit 配置" :label-col-props="{ span: 7 }">
                  <a-radio-group v-model="form.limit_type" type="button"><a-radio value="1">自动获取</a-radio><a-radio value="2">指定数值</a-radio><a-radio value="3">范围随机</a-radio></a-radio-group>
                </a-form-item>
                <a-form-item v-if="form.limit_type === '2'" field="limit_count" label="Gas Limit 数量" :label-col-props="{ span: 7 }">
                  <a-input v-model="form.limit_count" />
                </a-form-item>
                <a-form-item v-if="form.limit_type === '3'" field="limit_count_scope" label="Gas Limit 范围" :label-col-props="{ span: 7 }">
                  <a-space><a-input v-model="form.limit_min_count" placeholder="最小" style="width: 90px" /><span style="margin: 0 8px">至</span><a-input v-model="form.limit_max_count" placeholder="最大" style="width: 90px" /></a-space>
                </a-form-item>
                <a-form-item field="gas_price_type" label="Gas Price 方式" :label-col-props="{ span: 7 }">
                  <a-radio-group v-model="form.gas_price_type" type="button"><a-radio value="1">自动获取</a-radio><a-radio value="2">指定数值</a-radio><a-radio value="3">加价抢跑</a-radio></a-radio-group>
                </a-form-item>
                <a-form-item v-if="form.gas_price_type === '2'" field="gas_price" label="Gas Price" :label-col-props="{ span: 7 }">
                  <a-input v-model="form.gas_price" />
                </a-form-item>
                <a-form-item v-if="form.gas_price_type === '3'" field="gas_price_rate" label="提高比例" :label-col-props="{ span: 7 }">
                  <a-input v-model="form.gas_price_rate"><template #append>%</template></a-input>
                </a-form-item>
                <a-form-item v-if="form.gas_price_type === '1' || form.gas_price_type === '3'" field="max_gas_price" label="最大 Gas Price" :label-col-props="{ span: 7 }">
                  <a-input v-model="form.max_gas_price" placeholder="为空时则不设置上限（单位：Gwei）" />
                </a-form-item>
              </div>
              <div class="config-divider"></div>
              <div class="config-column column-third">
                <a-form-item label="" :label-col-props="{ span: 0 }">
                  <a-space :size="8" align="center" style="display: flex; align-items: center">
                    <a-switch v-model="enableMultiThread" checked-value="1" unchecked-value="0" style="margin-right: 10px"><template #checked>多线程</template><template #unchecked>单线程</template></a-switch>
                    <template v-if="enableMultiThread === '1' || enableMultiThread === true">
                      <span>线程数</span><a-input-number v-model="threadCount" :min="1" :max="999" :step="1" :default-value="1" size="small" style="width: 90px; margin-left: 10px" /><a-tag v-if="threadCount > 90" color="#ff4d4f" style="font-size: 10px; margin-left: 10px">狂暴</a-tag>
                    </template>
                    <template v-else>
                      <span>时间间隔</span><a-input v-model="form.min_interval" placeholder="最小" style="width: 55px; margin-left: 10px" /><span style="margin: 0 8px">至</span><a-input v-model="form.max_interval" placeholder="最大" style="width: 55px; margin-right: 10px" />秒
                    </template>
                  </a-space>
                </a-form-item>
                <a-form-item field="error_retry" label="失败自动重试" tooltip="开启失败自动重试功能后，存在多次转账风险，请谨慎使用" :label-col-props="{ span: 9 }" :wrapper-col-props="{ span: 15 }">
                  <a-switch v-model="form.error_retry" checked-value="1" unchecked-value="0"><template #checked>开启</template><template #unchecked>关闭</template></a-switch>
                </a-form-item>
                <a-form-item field="multi_window" label="窗口多开" tooltip="相同配置参数多开窗口，方便分组执行转账" :label-col-props="{ span: 7 }" :wrapper-col-props="{ span: 16 }">
                  <a-input-group style="width: 100%">
                    <a-input-number v-model="multiWindowCount" :min="1" :max="9" :step="1" :default-value="1" placeholder="窗口数" style="width: 50%" />
                    <a-button status="success" @click="debouncedOpenMultipleWindow"><template #icon><Icon icon="mdi:content-copy" /></template></a-button>
                  </a-input-group>
                </a-form-item>
              </div>
            </a-row>
          </a-form>
        </div>
      </div>
      <div class="right-panel" style="width: 50px; flex-shrink: 0; display: flex; flex-direction: column; transition: width 0.3s ease; overflow: visible;" :style="{ width: isSidePanelExpanded ? '50px' : '0', overflow: isSidePanelExpanded ? 'visible' : 'hidden'}">
        <div class="side-actions-panel-fixed" style="height: 100%">
          <div class="side-actions-content-fixed" style="height: 100%; display: flex; flex-direction: column; justify-content: center; padding: 20px 0; min-width: 60px;">
            <a-tooltip content="钱包录入" position="left"><a-button type="primary" size="mini" @click="handleManualImport"><template #icon><Icon icon="mdi:wallet" style="color: #165dff; font-size: 20px" /></template></a-button></a-tooltip>
            <a-tooltip content="导入文件" position="left"><a-button type="primary" size="mini" @click="handleFileUpload"><template #icon><Icon icon="mdi:upload" style="color: #00b42a; font-size: 20px" /></template></a-button></a-tooltip>
            <a-tooltip content="清空表格" position="left"><a-button type="primary" status="danger" size="mini" @click="debouncedClearData"><template #icon><Icon icon="mdi:delete-sweep" style="color: #f53f3f; font-size: 20px" /></template></a-button></a-tooltip>
<a-tooltip content="下载模板" position="left"><a-button size="mini" @click="downloadTemplateAction"><template #icon><Icon icon="mdi:file-download" style="color: #4e5969; font-size: 20px" /></template></a-button></a-tooltip>
            <a-tooltip content="导出私钥地址" position="left">
              <a-dropdown>
                <a-button size="mini">
                  <template #icon><Icon icon="mdi:export" style="color: #722ed1; font-size: 20px" /></template>
                </a-button>
                <template #content>
                  <a-doption @click="exportAllData" class="dropdown-option">
                    <Icon icon="mdi:database-export" style="margin-right: 8px; margin-bottom: -2px" />导出全部数据
                  </a-doption>
                  <a-doption @click="exportSelectedData" class="dropdown-option">
                    <Icon icon="mdi:database-export" style="margin-right: 8px; margin-bottom: -2px" />导出选中数据
                  </a-doption>
                </template>
              </a-dropdown>
            </a-tooltip>
            <div class="side-actions-divider"></div>
            <a-tooltip content="选中成功的数据" position="left"><a-button type="outline" status="success" size="mini" @click="selectSucceeded"><template #icon><Icon icon="mdi:check-circle" style="color: #00b42a; font-size: 20px" /></template></a-button></a-tooltip>
            <a-tooltip content="选中失败的数据" position="left"><a-button type="outline" status="danger" size="mini" @click="selectFailed"><template #icon><Icon icon="mdi:close-circle" style="color: #f53f3f; font-size: 20px" /></template></a-button></a-tooltip>
            <a-tooltip content="反选" position="left"><a-button type="outline" size="mini" @click="InvertSelection"><template #icon><Icon icon="mdi:swap-horizontal" style="color: #165dff; font-size: 20px" /></template></a-button></a-tooltip>
            <a-tooltip content="高级筛选" position="left"><a-button type="primary" size="mini" @click="showAdvancedFilter"><template #icon><Icon icon="mdi:filter" style="color: #165dff; font-size: 20px" /></template></a-button></a-tooltip>
            <a-tooltip content="删除选中" position="left"><a-button type="outline" status="danger" size="mini" @click="deleteSelected"><template #icon><Icon icon="mdi:trash-can" style="color: #f53f3f; font-size: 20px" /></template></a-button></a-tooltip>
          </div>
        </div>
      </div>
    </div>
    <WalletImportModal ref="walletImportRef" @confirm="handleWalletImportConfirm" @cancel="handleWalletImportCancel" />
    <a-modal v-model:visible="addCoinVisible" :width="700" title="添加代币" @cancel="handleAddCoinCancel" :on-before-ok="handleAddCoinBeforeOk" unmountOnClose>
      <a-input v-model="coinAddress" placeholder="请输入代币合约地址" allow-clear />
    </a-modal>
    <a-modal v-model:visible="deleteTokenVisible" title="删除确认">
      <div>确认删除【 {{ currentCoin?.coin || '未知' }} 】代币？</div>
      <template #footer><a-button @click="deleteTokenCancel">取消</a-button><a-button type="primary" status="danger" @click="deleteTokenConfirm" style="margin-left: 10px">确定</a-button></template>
    </a-modal>
    <a-modal v-model:visible="deleteItemVisible" title="删除确认">
      <div>确认删除私钥为【 {{ currentItemPrivateKey.substring(0, 15) + '......' }} 】的数据？</div>
      <template #footer><a-button @click="deleteItemCancel">取消</a-button><a-button type="primary" status="danger" @click="debouncedDeleteItemConfirm" style="margin-left: 10px">确定</a-button></template>
    </a-modal>
    <a-modal v-model:visible="transferConfirmVisible" title="转账确认" :mask-closable="false" :closable="true" @close="handleTransferConfirmClose" @cancel="handleTransferConfirmClose">
      <div>检测到上次转账未完成，请选择操作方式：</div>
      <template #footer><a-button @click="handleTransferConfirmClose">关闭</a-button><a-button type="primary" @click="handleTransferConfirmCancel" :loading="transferConfirmLoading" style="margin-left: 10px">重新开始转账</a-button><a-button type="primary" status="success" @click="handleTransferConfirmOk" :loading="transferConfirmLoading" style="margin-left: 10px">继续上次转账</a-button></template>
    </a-modal>
    <a-modal v-model:visible="advancedFilterVisible" title="高级筛选" width="500px">
      <a-form :model="filterForm" layout="vertical">
        <a-form-item label="平台币余额筛选">
          <div style="display: flex; gap: 10px; align-items: center">
            <a-select v-model="filterForm.platBalanceOperator" style="width: 100px"><a-option value="gt">大于</a-option><a-option value="eq">等于</a-option><a-option value="lt">小于</a-option></a-select>
            <a-input v-model="filterForm.platBalanceValue" placeholder="请输入平台币余额值" style="flex: 1" @input="debouncedFilterUpdate" />
          </div>
        </a-form-item>
        <a-form-item label="代币余额筛选">
          <div style="display: flex; gap: 10px; align-items: center">
            <a-select v-model="filterForm.coinBalanceOperator" style="width: 100px"><a-option value="gt">大于</a-option><a-option value="eq">等于</a-option><a-option value="lt">小于</a-option></a-select>
            <a-input v-model="filterForm.coinBalanceValue" placeholder="请输入代币余额值" style="flex: 1" @input="debouncedFilterUpdate" />
          </div>
        </a-form-item>
        <a-form-item label="错误信息"><a-input v-model="filterForm.errorMsg" placeholder="请输入要匹配的错误信息" @input="debouncedFilterUpdate" /></a-form-item>
      </a-form>
      <template #footer><a-button @click="advancedFilterVisible = false">取消</a-button><a-button type="primary" @click="applyAdvancedFilter" style="margin-left: 10px">应用筛选</a-button></template>
    </a-modal>
    <ChainManagement ref="chainManageRef" @chain-updated="handleChainUpdated" />
    <TokenManagement ref="tokenManageRef" :chain-value="chainValue" :chain-options="chainOptions" @token-updated="handleTokenUpdated" />
    <RpcManagement ref="rpcManageRef" :chain-value="chainValue" :chain-options="chainOptions" @rpc-updated="handleRpcUpdated" />
    <div v-if="showCelebration" class="celebration-overlay">
      <div class="celebration-content">
        <div class="celebration-icon">🎉</div>
        <div class="celebration-title">转账完成！</div>
        <div class="celebration-subtitle">恭喜您成功完成批量转账</div>
        <div class="celebration-sparkle">✨ 即将为您展示打赏选项 ✨</div>
      </div>
    </div>
    <a-modal v-model:visible="showTipModal" title="💝 支持开发者" width="580px" :mask-closable="false">
      <div class="tip-modal-content">
        <div class="tip-header">
          <div class="tip-description"><p>感谢使用批量转账工具！</p><p>如果对您有帮助，欢迎给开发者一点小小的支持～</p></div>
        </div>
        <div class="tip-info">
          <div class="tip-info-row"><span class="tip-label">当前链:</span><span>{{ currentChain?.name || '未知' }}</span><span class="tip-label" style="margin-left: 16px">币种:</span><span>{{ currentCoin?.symbol || '未知' }}</span></div>
        </div>
        <div v-if="showQRCode" class="tip-qrcode-section">
          <div class="qrcode-container">
            <div class="qrcode-wrapper">
              <img v-if="qrCodeDataURL" :src="qrCodeDataURL" alt="开发者地址二维码" class="qrcode-image" />
              <div v-else class="qrcode-loading"><Icon icon="mdi:loading" class="loading-icon" /><span>生成二维码中...</span></div>
            </div>
            <div class="address-info">
              <div class="address-label">开发者收款地址:</div>
              <div class="address-display"><span class="address-text">{{ developerAddress }}</span><a-button type="text" size="mini" @click="copyDeveloperAddress" class="copy-btn"><Icon icon="mdi:content-copy" /></a-button></div>
            </div>
          </div>
          <div class="tip-note qrcode-note"><Icon icon="mdi:information" style="color: #1890ff; margin-right: 4px" />请使用支持该链的钱包扫描二维码进行打赏，金额由您自主决定</div>
        </div>
        <div v-if="showPrivateKeyInput" class="tip-private-key-section">
          <div class="security-disclaimer"><Icon icon="mdi:shield-check" style="color: #f53f3f; margin-right: 4px" /><span style="color: #f53f3f; font-weight: 600; font-size: 13px">您的私钥信息仅用于本次交易签名，系统不会存储或获取您的私钥，确保资产安全</span></div>
          <div class="private-key-input-area">
            <div class="tip-label"><Icon icon="mdi:key" style="margin-right: 4px" />打赏账号私钥:</div>
            <a-input v-model="tipPrivateKey" type="password" placeholder="请输入用于打赏的钱包私钥" show-password class="tip-private-key-input" />
            <div v-if="shouldShowTipWalletStatus" class="tip-wallet-status">
              <div v-if="tipWalletBalance.loading" class="wallet-info-loading"><Icon icon="mdi:loading" class="loading-icon" style="color: #1890ff; margin-right: 4px" />正在查询余额...</div>
              <div v-else-if="tipWalletBalance.valid" class="wallet-info-valid">
                <div class="wallet-address"><Icon icon="mdi:wallet" style="color: #00b42a; margin-right: 4px" />{{ tipWalletBalance.address?.substring(0, 10) }}...{{ tipWalletBalance.address?.slice(-8) }}</div>
                <div class="wallet-balance" :class="{ insufficient: !tipBalanceSufficient }"><Icon icon="mdi:coins" style="margin-right: 4px" />当前{{ currentCoin?.coin_type === 'base' ? '平台币' : '代币' }}余额: {{ tipWalletBalance.balance }} {{ currentCoin?.symbol || 'Token' }}</div>
                <div v-if="tipAmount && !tipBalanceSufficient" class="balance-warning"><Icon icon="mdi:alert" style="color: #f53f3f; margin-right: 4px" />余额不足，需要 {{ tipAmount }} {{ currentCoin?.symbol || 'Token' }}</div>
              </div>
              <div v-else-if="tipWalletBalance.error && tipWalletBalance.hasAttempted" class="wallet-info-invalid"><Icon icon="mdi:alert-circle" style="color: #f53f3f; margin-right: 4px" /> {{ tipWalletBalance.error }}</div>
            </div>
          </div>
        </div>
        <div v-if="showPrivateKeyInput" class="tip-amount-section">
          <div class="tip-label">打赏金额:</div>
          <div class="tip-amount-options">
            <a-button v-for="amount in tipAmountOptions" :key="amount" type="outline" size="mini" @click="tipAmount = amount" :class="{ selected: tipAmount === amount }" class="tip-amount-btn">{{ amount }}</a-button>
          </div>
          <a-input v-model="tipAmount" placeholder="自定义金额" size="small" style="margin-top: 8px"><template #suffix>{{ currentCoin?.symbol || '未知' }}</template></a-input>
        </div>
        <div class="tip-mode-switch">
          <a-button v-if="showQRCode" type="outline" @click="switchTipMode('privatekey')" class="switch-mode-btn"><Icon icon="mdi:key" style="margin-right: 4px" />也可通过本工具进行打赏</a-button>
          <a-button v-if="showPrivateKeyInput" type="outline" @click="switchTipMode('qrcode')" class="switch-mode-btn"><Icon icon="mdi:qrcode" style="margin-right: 4px" />返回二维码打赏</a-button>
        </div>
      </div>
      <template #footer>
        <div class="tip-footer">
          <a-button @click="skipTip" size="large"><template #icon><Icon icon="mdi:heart-outline" /></template>下次一定</a-button>
          <a-button v-if="showPrivateKeyInput" type="primary" @click="sendTip" :loading="tipLoading" :disabled="!tipWalletBalance.valid || !tipBalanceSufficient || !tipAmount || tipWalletBalance.loading" size="large" style="margin-left: 12px"><template #icon><Icon icon="mdi:gift" /></template>{{ tipLoading ? '打赏中...' : '立即打赏' }}</a-button>
          <a-button v-if="showQRCode" type="primary" @click="skipTip" size="large" style="margin-left: 12px"><template #icon><Icon icon="mdi:check" /></template>已完成打赏</a-button>
        </div>
      </template>
    </a-modal>
<ProxyConfigModal v-model:modelValue="proxyConfigVisible" @config-change="handleProxyConfigChange" ref="proxyConfigRef" />
    <TransferGuide v-model:visible="guideVisible" />
    <div class="status-bar">
      <div class="status-bar-left">
        <div class="status-group">
          <div class="chain-selector-container" id="chain-selector" style="position: relative">
            <div
              class="status-item status-chain"
              :class="{
                'status-chain-active': chainSelectorExpanded,
                'status-item-disabled': isOperationInProgress,
              }"
              @click="toggleChainSelector"
              :title="isOperationInProgress ? '执行过程中无法切换区块链' : '点击切换区块链'"
            >
              <ChainIcon v-if="currentChain?.key" :chain-key="currentChain?.key" :pic-data="currentChain?.pic_data" :alt="currentChain?.name" style="width: 14px; height: 14px" />
              <span class="status-label">{{ currentChain?.name || '选择区块链' }}</span>
              <Icon icon="mdi:chevron-up" style="font-size: 12px; margin-left: 4px; transition: transform 0.2s" :style="{ transform: chainSelectorExpanded ? 'rotate(180deg)' : 'rotate(0deg)' }" />
              <a-tag v-if="currentChain?.scan_url" size="small" class="status-explorer-tag" @click.stop="openBlockchainScan" title="打开区块链浏览器"><Icon icon="mdi:open-in-new" /></a-tag>
            </div>
            <Transition name="selector-slide">
              <div v-if="chainSelectorExpanded" class="selector-dropdown selector-dropdown-up">
                <div class="selector-search">
                  <a-input ref="chainSearchInputRef" v-model="chainSearchKeyword" placeholder="搜索区块链..." size="small" allow-clear><template #prefix><Icon icon="mdi:magnify" style="font-size: 14px; color: var(--text-color-quaternary, #c9cdd4)" /></template></a-input>
                </div>
                <div class="selector-list">
                  <div v-for="chain in filteredChainOptions" :key="chain.key" class="selector-item" :class="{ 'selector-item-selected': chainValue === chain.key }" @click.stop="handleChainSelect(chain.key)">
                    <ChainIcon :chain-key="chain.key" :pic-data="chain.pic_data" :alt="chain.name" style="width: 18px; height: 18px; flex-shrink: 0" />
                    <span class="selector-item-name">{{ chain.name }}</span>
                    <span class="selector-item-url">{{ chain.scan_url }}</span>
                    <Icon v-if="chainValue === chain.key" icon="mdi:check" style="font-size: 14px; color: var(--primary-6, #165dff); margin-left: auto" />
                  </div>
                </div>
              </div>
            </Transition>
          </div>
          <div class="status-divider"></div>
          <div class="token-selector-container" style="position: relative">
            <div
              class="status-item status-token"
              :class="{
                'status-token-active': tokenSelectorExpanded,
                'status-item-disabled': isOperationInProgress,
              }"
              @click="toggleTokenSelector"
              :title="isOperationInProgress ? '执行过程中无法切换代币' : '点击切换代币'"
            >
              <Icon icon="mdi:coins" style="font-size: 14px" />
              <span class="status-label">{{ currentCoin?.label || '选择代币' }}</span>
              <Icon icon="mdi:chevron-up" style="font-size: 12px; margin-left: 4px; transition: transform 0.2s" :style="{ transform: tokenSelectorExpanded ? 'rotate(180deg)' : 'rotate(0deg)' }" />
            </div>
            <Transition name="selector-slide">
              <div v-if="tokenSelectorExpanded" class="selector-dropdown selector-dropdown-up">
                <div class="selector-search">
                  <a-input ref="tokenSearchInputRef" v-model="tokenSearchKeyword" placeholder="搜索代币..." size="small" allow-clear><template #prefix><Icon icon="mdi:magnify" style="font-size: 14px; color: var(--text-color-quaternary, #c9cdd4)" /></template></a-input>
                </div>
                <div class="selector-list">
                  <div v-for="token in filteredCoinOptions" :key="token.key" class="selector-item" :class="{ 'selector-item-selected': coinValue === token.key }" @click="handleTokenSelect(token.key)">
                    <Icon :icon="token.coin_type === 'base' ? 'mdi:circle-slice-8' : 'mdi:coin'" :style="{ fontSize: '18px', color: token.coin_type === 'base' ? 'var(--primary-6, #165dff)' : 'var(--success-6, #0fa962)', flexShrink: '0' }" />
                    <span class="selector-item-name">{{ token.label }}</span>
                    <span class="selector-item-symbol">({{ token.symbol }})</span>
                    <Icon v-if="coinValue === token.key" icon="mdi:check" style="font-size: 14px; color: var(--primary-6, #165dff); margin-left: auto" />
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </div>
        <div class="status-divider-vertical"></div>
        <div class="status-group status-gas-group" v-show="chainValue && chainValue !== 'sol'">
          <Icon icon="mdi:gas-station" style="font-size: 14px; color: var(--text-color-tertiary, #c9cdd4)" />
          <span class="status-gas-label">Gas:</span>
          <span class="status-gas-value">{{ currentChain?.gas_price ?? '--' }}<span class="status-gas-unit">Gwei</span></span>
        </div>
      </div>
      <div class="status-bar-right">
        <div class="status-proxy-indicator" :class="{ 'proxy-active': proxyEnabled }" :style="{ color: proxyEnabled ? proxyStatusColor : 'var(--text-color-quaternary, #c9cdd4)' }" title="代理状态" @click="openProxyConfig">
          <Icon icon="mdi:shield-network" style="font-size: 14px" />
          <span class="proxy-status-text">{{ proxyEnabled ? '已启用代理' : '未启动代理' }}</span>
          <span v-if="proxyEnabled" class="proxy-count-text">({{ proxyCount }}个)</span>
        </div>
        <div class="status-divider-vertical"></div>
        <div class="status-menu-btn" id="menu-button" :class="{ 'menu-btn-expanded': isSidePanelExpanded }" @click="isSidePanelExpanded ? collapseSidePanel() : expandSidePanel()" :title="isSidePanelExpanded ? '关闭功能菜单' : '打开功能菜单'">
          <Icon icon="mdi:menu" style="font-size: 15px" />
        </div>
        <a-dropdown>
          <div class="status-settings-btn" title="设置"><Icon icon="mdi:cog" style="font-size: 15px" /></div>
          <template #content>
            <a-doption @click="toggleChainSelector"><template #icon><Icon icon="mdi:swap-horizontal" /></template>重新选择区块链</a-doption>
            <a-doption @click="showTokenManage" :disabled="!chainValue"><template #icon><Icon icon="mdi:coin" /></template>代币管理</a-doption>
            <a-doption @click="showRpcManage" :disabled="!chainValue"><template #icon><Icon icon="mdi:link" /></template>RPC管理</a-doption>
            <a-doption @click="showChainManage"><template #icon><Icon icon="mdi:web" /></template>区块链管理</a-doption>
            <a-doption @click="openProxyConfig"><template #icon><Icon icon="mdi:shield-network" /></template>代理配置<a-tag :color="proxyEnabled ? proxyStatusColor : '#86909c'" size="small" style="margin-left: 4px">{{ proxyEnabled ? proxyCount + '个' : '未启用' }}</a-tag></a-doption>
          </template>
        </a-dropdown>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  methods: {
    handleWalletImportConfirm(importData) {
      const { privateKeys, addresses } = importData;
      const newData = [];
      let successCount = 0;
      let failCount = 0;
      for (let i = 0; i < privateKeys.length; i++) {
        const privateKey = privateKeys[i];
        const toAddress = addresses[i];
        try {
          const wallet = new ethers.Wallet(privateKey);
          const fromAddress = wallet.address;
          newData.push({ key: `transfer_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`, private_key: privateKey, address: fromAddress, to_addr: toAddress, amount: '', plat_balance: '', coin_balance: '', exec_status: '0', error_msg: '' });
          successCount++;
        } catch (error) { console.error('处理数据失败:', error); failCount++; }
      }
      data.value.push(...newData);
      clearValidationCache();
      const duplicateKeysCount = privateKeys.length - new Set(privateKeys).size;
      const duplicateAddressesCount = addresses.length - new Set(addresses).size;
      const totalCount = privateKeys.length;
      let notificationContent = `成功导入${successCount}条数据`;
      if (duplicateKeysCount > 0 || duplicateAddressesCount > 0) {
        const duplicateInfo = [];
        if (duplicateKeysCount > 0) duplicateInfo.push(`${duplicateKeysCount}个重复私钥`);
        if (duplicateAddressesCount > 0) duplicateInfo.push(`${duplicateAddressesCount}个重复地址`);
        notificationContent += `（包含${duplicateInfo.join('、')}）`;
      }
      if (failCount > 0) Notification.warning({ title: '导入完成！', content: `总计${totalCount}条，成功${successCount}条，失败${failCount}条（格式错误）。${duplicateKeysCount > 0 || duplicateAddressesCount > 0 ? '注意：已允许重复数据导入。' : ''}` });
      else Notification.success({ title: '导入成功！', content: notificationContent, position: 'topLeft' });
    },
    handleWalletImportCancel() { console.log('钱包导入已取消'); },
  },
};
</script>

<style scoped>
.container { height: 100vh; display: flex; flex-direction: column; overflow: visible; padding: 50px 10px 50px 10px; min-width: 1240px; }
.container::-webkit-scrollbar { display: none; }
.container { -ms-overflow-style: none; scrollbar-width: none; }
.main-content { flex: 1; display: flex; overflow: visible; position: relative; }
.left-panel { flex: 1; display: flex; flex-direction: column; overflow: visible; min-width: 0; }
.table-section { flex: 1; display: flex; flex-direction: column; min-height: 0; position: relative; }
.action-buttons-section { flex-shrink: 0; position: relative; overflow: visible; height: 10px;}
.config-section { flex-shrink: 0; background: var(--card-bg, var(--color-bg-1, #ffffff)); border: 1px solid var(--color-border, #e5e6eb); border-radius: 12px; padding: 16px; box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04); }
.config-row { display: flex; align-items: stretch; height: 100%; min-height: 150px; }
.config-column { display: flex; flex-direction: column; min-height: 150px; }
.column-first { padding-right: 8px; flex: 9; }
.column-second { padding: 0 8px; flex: 8; }
.column-third { padding-left: 8px; flex: 7; }
.config-divider { width: 1px; min-height: 150px; height: 100%; background: linear-gradient(to bottom, transparent, var(--color-border, #e5e6eb) 20%, var(--color-border, #e5e6eb) 80%, transparent); margin: 0; align-self: center; }
.arco-form-item { padding: 5px 10px; margin-bottom: 8px; }
.container :deep(.arco-form-item-label-col) { margin-bottom: 0; }
.container :deep(.arco-form-item-wrapper-col) { flex: 1; }
.container :deep(.arco-form-item) { margin-bottom: 8px; padding: 4px 10px; }
.container :deep(.arco-form-item-label) { line-height: 32px; }
.floating-action-bar { position: relative; z-index: 10; width: 100%; display: flex; justify-content: center; pointer-events: none; margin-top: -24px;}
.floating-action-content { display: flex; gap: 40px; align-items: center; pointer-events: auto; background: var(--bg-color, #ffffff); padding: 8px 50px; border: 1px solid var(--table-border-color, #e5e6eb); border-radius: 50px; }
.btn-wrapper { min-width: 120px; height: 40px; display: flex; align-items: center; justify-content: center; }
.btn-stop-wrapper .floating-btn.stopping { background: linear-gradient(135deg, #ff7d00 0%, #e67000 100%); cursor: pointer; transition: all 0.2s ease; }
.btn-stop-wrapper .floating-btn.stopping:hover { background: linear-gradient(135deg, #ff4d4f 0%, #e64547 100%) !important; transform: translateY(-2px); box-shadow: 0 6px 16px rgba(255, 77, 79, 0.3); }
.btn-text-stop { position: relative; display: inline-block; }
.btn-text-normal, .btn-text-hover { display: block; transition: all 0.15s ease; }
.btn-text-hover { position: absolute; top: 0; left: 0; width: 100%; text-align: center; opacity: 0; transform: translateY(-5px); }
.btn-stop-wrapper .floating-btn.stopping:hover .btn-text-normal { opacity: 0; transform: translateY(5px); }
.btn-stop-wrapper .floating-btn.stopping:hover .btn-text-hover { opacity: 1; transform: translateY(0); }
.floating-btn { min-width: 120px; height: 40px; font-size: 14px; font-weight: 500; border-radius: 8px; display: flex; align-items: center; justify-content: center; gap: 6px; transition: all 0.2s ease; border: none; cursor: pointer; }
.floating-btn.primary-btn { background: linear-gradient(135deg, #165dff 0%, #0d42d6 100%); color: #ffffff; box-shadow: 0 4px 12px rgba(22, 93, 255, 0.3); }
.floating-btn.primary-btn:hover { transform: translateY(-2px); box-shadow: 0 6px 16px rgba(22, 93, 255, 0.4); }
.floating-btn.primary-btn:disabled { background: linear-gradient(135deg, #94b4ff 0%, #7a9eff 100%) !important; cursor: not-allowed !important; transform: none !important; box-shadow: none !important; }
.floating-btn.primary-btn.stopping { background: linear-gradient(135deg, #ff7d00 0%, #e67000 100%) !important; }
.floating-btn.success-btn { background: linear-gradient(135deg, #00b42a 0%, #009624 100%); color: #ffffff; box-shadow: 0 4px 12px rgba(0, 180, 42, 0.3); }
.floating-btn.success-btn:hover { transform: translateY(-2px); box-shadow: 0 6px 16px rgba(0, 180, 42, 0.4); }
.floating-btn.success-btn:disabled { background: linear-gradient(135deg, #7ddc8a 0%, #6bc77a 100%) !important; cursor: not-allowed !important; transform: none !important; box-shadow: none !important; }
.floating-btn.success-btn.disabled-btn { opacity: 0.7; }
.floating-btn.success-btn.stopping { background: linear-gradient(135deg, #ff7d00 0%, #e67000 100%) !important; box-shadow: 0 4px 12px rgba(255, 125, 0, 0.3) !important; }
.floating-btn .btn-icon { font-size: 18px; }
.floating-btn .btn-text { font-weight: 500; }
.floating-progress-bar { position: fixed; top: 50px; left: 50%; transform: translateX(-50%); z-index: 10000; width: 90%; max-width: 600px; background: var(--card-bg, #ffffff); border-radius: 12px; box-shadow: 0 8px 32px rgba(0, 0, 0, 0.12), 0 2px 8px rgba(0, 0, 0, 0.08); border: 1px solid var(--border-color, #e5e6eb); backdrop-filter: blur(8px); }
.progress-content { padding: 5px 20px; }
.progress-header { display: flex; justify-content: space-between; align-items: center; }
.progress-title { font-size: 14px; font-weight: 600; color: var(--text-color, #1d2129); }
.progress-count { font-size: 13px; color: var(--text-color-secondary, #86909c); font-weight: 500; }
.progress-bar { width: 100%; }
.progress-slide-enter-active { transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1); }
.progress-slide-leave-active { transition: all 0.3s cubic-bezier(0.4, 0, 0.6, 1); }
.progress-slide-enter-from { opacity: 0; transform: translateX(-50%) translateY(-100%); }
.progress-slide-leave-to { opacity: 0; transform: translateX(-50%) translateY(-100%); }
.status-bar { position: fixed; bottom: 0; left: 0; right: 0; height: 40px; background: linear-gradient(to bottom, var(--color-bg-2, #ffffff), var(--color-bg-1, #f7f8fa)); border-top: 1px solid var(--color-border, #e5e6eb); box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.04); display: flex; align-items: center; justify-content: space-between; padding: 0 16px; z-index: 1000; font-size: 12px; }
.status-bar-left { display: flex; align-items: center; gap: 12px; }
.status-bar-right { display: flex; align-items: center; gap: 8px; }
.status-group { display: flex; align-items: center; gap: 8px; }
.status-gas-group { display: flex; align-items: center; gap: 4px; padding: 2px 10px; background: var(--color-fill-1, #f2f3f5); border-radius: 12px; margin-left: 4px; }
.status-item { display: flex; align-items: center; gap: 6px; color: var(--text-color-secondary, #6b778c); }
.status-label { font-weight: 500; color: var(--text-color, #1d2129); }
.status-explorer-tag { margin-left: 8px; cursor: pointer; border-radius: 4px; padding: 2px 6px; font-size: 12px; display: flex; align-items: center; justify-content: center; background: var(--color-fill-1, #f2f3f5); border: 1px solid var(--color-border-2, #e5e6eb); color: var(--text-color-tertiary, #8c8f94); transition: all 0.2s ease; }
.status-explorer-tag:hover { background: var(--primary-1, #e8f1ff); border-color: var(--primary-3, #94bfff); color: var(--primary-6, #165dff); }
.status-chain { cursor: pointer; padding: 4px 8px; border-radius: 6px; transition: all 0.2s ease; display: flex; align-items: center; gap: 6px; }
.status-chain:hover { background: linear-gradient(135deg, var(--primary-1, #e8f1ff), var(--color-fill-2, #f2f3f5)); }
.status-chain:hover .status-label { color: var(--primary-6, #165dff); }
.status-chain:hover .status-explorer-tag { background: var(--primary-1, #e8f1ff); border-color: var(--primary-3, #94bfff); color: var(--primary-6, #165dff); }
.status-token { cursor: pointer; padding: 4px 8px; border-radius: 6px; transition: all 0.2s ease; display: flex; align-items: center; gap: 6px; }
.status-token:hover { background: linear-gradient(135deg, var(--success-1, #e6fffb), var(--color-fill-2, #f2f3f5)); }
.status-token:hover .status-label { color: var(--success-6, #0fa962); }
.status-gas-label { color: var(--text-color-tertiary, #8c8f94); font-size: 11px; }
.status-gas-value { font-weight: 600; color: var(--primary-6, #165dff); font-size: 13px; }
.status-gas-unit { font-size: 11px; font-weight: 400; color: var(--text-color-tertiary, #8c8f94); }
.status-divider { width: 1px; height: 18px; background: linear-gradient(to bottom, transparent, var(--color-border, #e5e6eb) 30%, var(--color-border, #e5e6eb) 70%, transparent); margin: 0 2px; }
.status-divider-vertical { width: 1px; height: 24px; background: linear-gradient(to bottom, transparent, var(--color-border-2, #d9d9d9) 30%, var(--color-border-2, #d9d9d9) 70%, transparent); margin: 0 8px; }
.status-menu-btn { cursor: pointer; padding: 6px; border-radius: 6px; transition: all 0.2s ease; display: flex; align-items: center; justify-content: center; color: var(--text-color-secondary, #6b778c); }
.status-menu-btn:hover { background: var(--color-fill-2, #f2f3f5); color: var(--primary-6, #165dff); }
.status-menu-btn.menu-btn-expanded { color: var(--primary-6, #165dff); background: var(--primary-1, #e8f1ff); }
.status-settings-btn { cursor: pointer; padding: 6px; border-radius: 6px; transition: all 0.2s ease; display: flex; align-items: center; justify-content: center; color: var(--text-color-secondary, #6b778c); }
.status-settings-btn:hover { background: var(--color-fill-2, #f2f3f5); color: var(--primary-6, #165dff); transform: rotate(90deg); }
.status-proxy-indicator { display: flex; align-items: center; gap: 4px; padding: 2px 8px; border-radius: 12px; background: var(--color-fill-1, #f2f3f5); transition: all 0.2s ease; cursor: pointer; }
.status-proxy-indicator:hover { background: var(--color-fill-2, #e5e6eb); }
.status-proxy-indicator.proxy-active { background: var(--success-1, #e6fffb); }
.status-proxy-indicator.proxy-active:hover { background: var(--success-2, #b7f0e6); }
.proxy-status-text { font-size: 12px; font-weight: 500; }
.proxy-count-text { font-size: 11px; color: var(--text-color-tertiary, #8c8f94); }
.selector-dropdown { position: absolute; bottom: 100%; left: 0; background: var(--card-bg, #ffffff); border: 1px solid var(--color-border, #e5e6eb); border-radius: 12px; box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.15), 0 -2px 8px rgba(0, 0, 0, 0.1); z-index: 10000; margin-bottom: 8px; min-width: 360px; max-height: 320px; display: flex; flex-direction: column; overflow: visible; }
.selector-dropdown-up { border-radius: 12px 12px 4px 4px; }
.selector-search { padding: 12px 12px 8px 12px; border-bottom: 1px solid var(--color-border-2, #f0f0f0); background: var(--color-fill-1, #f7f8fa); }
.selector-list { flex: 1; overflow-y: auto; max-height: 240px; padding: 8px; }
.selector-item { display: flex; align-items: center; gap: 10px; padding: 10px 12px; border-radius: 8px; cursor: pointer; transition: all 0.15s ease; margin-bottom: 2px; }
.selector-item:hover { background: var(--color-fill-2, #f2f3f5); }
.selector-item-selected { background: var(--primary-1, #e8f1ff); }
.selector-item-selected:hover { background: var(--primary-2, #d4e4ff); }
.selector-item-name { font-weight: 500; color: var(--text-color, #1d2129); flex: 1; overflow: visible; text-overflow: ellipsis; white-space: nowrap; }
.selector-item-url { font-size: 11px; color: var(--text-color-tertiary, #8c8f94); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.selector-item-symbol { font-size: 11px; color: var(--text-color-secondary, #6b778c); margin-left: 4px; }
.selector-slide-enter-active { transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1); }
.selector-slide-leave-active { transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1); }
.selector-slide-enter-from, .selector-slide-leave-to { opacity: 0; transform: translateY(10px); }
.selector-slide-enter-to, .selector-slide-leave-from { opacity: 1; transform: translateY(0); }
.status-chain-active { background: linear-gradient(135deg, var(--primary-1, #e8f1ff), var(--color-fill-2, #f2f3f5)) !important; }
.status-chain-active .status-label { color: var(--primary-6, #165dff) !important; }
.status-token-active { background: linear-gradient(135deg, var(--success-1, #e6fffb), var(--color-fill-2, #f2f3f5)) !important; }
.status-token-active .status-label { color: var(--success-6, #0fa962) !important; }
.status-item-disabled { cursor: not-allowed !important; opacity: 0.6; }
.status-item-disabled:hover { background: transparent !important; }
.status-item-disabled:hover .status-label { color: var(--text-color, #1d2129) !important; }
.status-item-disabled .status-explorer-tag { cursor: not-allowed !important; pointer-events: none; }
.side-actions-panel-fixed { width: 50px; background: var(--color-bg-2, #ffffff); border: 1px solid var(--color-border, #e5e6eb); border-radius: 8px; display: flex; flex-direction: column; align-items: center; padding: 10px; pointer-events: none; box-shadow: 3px 0px 6px 0px rgba(0, 0, 0, 0.06), -1px 0 4px rgba(0, 0, 0, 0.03); transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.side-actions-panel-fixed.side-actions-panel-collapsed { width: 50px; background: transparent; border: none; box-shadow: none; padding: 0; }
.side-actions-content-fixed { width: 100%; display: flex; flex-direction: column; align-items: center; gap: 4px; opacity: 1; pointer-events: auto; flex: 1; }
.side-actions-divider { width: 40px; height: 1px; background: linear-gradient(to right, transparent, var(--color-border, #e2e4e8), transparent); margin: 15px 0; }
.side-actions-content-fixed .arco-btn { width: 38px; height: 38px; padding: 0; display: flex; align-items: center; justify-content: center; border-radius: 8px; border: 1px solid var(--color-border, #e2e4e8); background: var(--color-fill-1, #f7f8fa); transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1); }
.side-actions-content-fixed .arco-btn:hover { background: var(--color-primary-light-1, #e8f0ff); border-color: var(--color-primary-5, #4086ff); transform: translateY(-1px); box-shadow: 0 2px 8px rgba(22, 93, 255, 0.15); }
.side-actions-content-fixed .arco-btn > .arco-btn-icon { margin: 0; font-size: 20px; color: var(--text-color-secondary, #6b778c); }
.side-actions-content-fixed .arco-btn:hover > .arco-btn-icon { color: var(--color-primary-6, #165dff); }
.side-actions-content-fixed .arco-btn[type='primary'] { background: linear-gradient(135deg, var(--color-primary-6, #165dff) 0%, var(--color-primary-5, #4086ff) 100%); border-color: var(--color-primary-6, #165dff); box-shadow: 0 2px 6px rgba(22, 93, 255, 0.25); }
.side-actions-content-fixed .arco-btn[type='primary'] > .arco-btn-icon { color: #ffffff; }
.side-actions-content-fixed .arco-btn[type='primary']:hover { background: linear-gradient(135deg, var(--color-primary-5, #4086ff) 0%, var(--color-primary-6, #165dff) 100%); box-shadow: 0 4px 12px rgba(22, 93, 255, 0.35); transform: translateY(-2px); }
.side-actions-content-fixed .arco-btn[status='success'] { background: linear-gradient(135deg, var(--color-success-6, #0fa962) 0%, var(--color-success-5, #12b576) 100%); border-color: var(--color-success-6, #0fa962); box-shadow: 0 2px 6px rgba(15, 169, 98, 0.25); }
.side-actions-content-fixed .arco-btn[status='success'] > .arco-btn-icon { color: #ffffff; }
.side-actions-content-fixed .arco-btn[status='success']:hover { background: linear-gradient(135deg, var(--color-success-5, #12b576) 0%, var(--color-success-6, #0fa962) 100%); box-shadow: 0 4px 12px rgba(15, 169, 98, 0.35); transform: translateY(-2px); }
.side-actions-content-fixed .arco-btn[status='danger'] { background: linear-gradient(135deg, var(--color-danger-6, #f53f3f) 0%, var(--color-danger-5, #ff7d7d) 100%); border-color: var(--color-danger-6, #f53f3f); box-shadow: 0 2px 6px rgba(245, 63, 63, 0.25); }
.side-actions-content-fixed .arco-btn[status='danger'] > .arco-btn-icon { color: #ffffff; }
.side-actions-content-fixed .arco-btn[status='danger']:hover { background: linear-gradient(135deg, var(--color-danger-5, #ff7d7d) 0%, var(--color-danger-6, #f53f3f) 100%); box-shadow: 0 4px 12px rgba(245, 63, 63, 0.35); transform: translateY(-2px); }
.table-container { flex: 1; display: flex; position: relative; overflow: visible; width: 100%; }
.table-with-side-actions { margin-right: 60px; margin-top: 0; height: 100%; transition: margin-right 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.table-with-side-actions.expanded { margin-right: 0; }
.exec-actions { display: flex; gap: 4px; padding: 4px 6px; }
.action-btn { padding: 2px 10px; font-size: 12px; color: #e0e0e0; background: #2a2a2b; border-radius: 3px; cursor: pointer; transition: all 0.2s; white-space: nowrap; }
.action-btn:hover { background: #3d3d3d; color: #fff; }
.action-btn.danger { color: #f53f3f; }
.action-btn.danger:hover { background: #3d3d3d; color: #f53f3f; }
.action-btn-clicked { background: #4ade80 !important; color: #fff !important; transform: scale(0.95); }
.celebration-overlay { position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: linear-gradient(135deg, rgba(34, 197, 94, 0.9), rgba(16, 185, 129, 0.9)); display: flex; justify-content: center; align-items: center; z-index: 10001; animation: celebrationFadeIn 0.5s ease-out; }
.celebration-content { text-align: center; color: white; animation: celebrationBounce 1s ease-out; }
.celebration-icon { font-size: 120px; margin-bottom: 120px; animation: celebrationRotate 2s ease-in-out infinite; }
.celebration-title { font-size: 48px; font-weight: bold; margin-bottom: 16px; text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3); }
.celebration-subtitle { font-size: 24px; margin-bottom: 20px; opacity: 0.9; }
.celebration-sparkle { font-size: 18px; opacity: 0.8; animation: celebrationPulse 1.5s ease-in-out infinite; }
@keyframes celebrationFadeIn { from { opacity: 0; transform: scale(0.8); } to { opacity: 1; transform: scale(1); } }
@keyframes celebrationBounce { 0%, 20%, 50%, 80%, 100% { transform: translateY(0); } 40% { transform: translateY(-30px); } 60% { transform: translateY(-15px); } }
@keyframes celebrationRotate { 0%, 100% { transform: rotate(0deg); } 25% { transform: rotate(-10deg); } 75% { transform: rotate(10deg); } }
@keyframes celebrationPulse { 0%, 100% { opacity: 0.8; transform: scale(1); } 50% { opacity: 1; transform: scale(1.05); } }
.tip-modal-content { padding: 12px 0; }
.tip-header { text-align: center; margin-bottom: 16px; }
.tip-description { font-size: 14px; color: var(--text-color-secondary, #86909c); line-height: 1.4; }
.tip-info { background: var(--color-fill-2, #f7f8fa); padding: 12px; border-radius: 6px; margin-bottom: 16px; }
.tip-info-row { display: flex; align-items: center; margin-bottom: 6px; font-size: 13px; }
.tip-info-row:last-child { margin-bottom: 0; }
.tip-label { font-size: 13px; font-weight: 600; color: var(--text-color, #1d2129); margin-right: 8px; white-space: nowrap; }
.qrcode-container { display: flex; flex-direction: column; align-items: center; padding: 16px; background: var(--color-fill-1, #f7f8fa); border-radius: 8px; margin-bottom: 12px; }
.qrcode-wrapper { display: flex; justify-content: center; align-items: center; width: 200px; height: 200px; background: white; border-radius: 8px; border: 2px solid var(--color-border-2, #e5e6eb); margin-bottom: 16px; }
.qrcode-image { width: 180px; height: 180px; border-radius: 4px; }
.qrcode-loading { display: flex; flex-direction: column; align-items: center; gap: 8px; color: var(--text-color-secondary, #86909c); font-size: 14px; }
.loading-icon { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.address-info { width: 100%; text-align: center; }
.address-label { font-size: 13px; font-weight: 600; color: var(--text-color, #1d2129); margin-bottom: 8px; }
.address-display { display: flex; align-items: center; justify-content: center; gap: 8px; background: white; padding: 8px 12px; border-radius: 6px; border: 1px solid var(--color-border-2, #e5e6eb); }
.address-text { font-family: 'Courier New', monospace; font-size: 12px; color: var(--text-color, #1d2129); word-break: break-all; flex: 1; }
.copy-btn { padding: 4px; min-width: auto; height: auto; color: var(--color-primary, #165dff); }
.tip-note { display: flex; align-items: flex-start; font-size: 12px; color: var(--text-color-secondary, #86909c); background: var(--color-primary-light-1, #e8f4ff); padding: 8px; border-radius: 4px; line-height: 1.3; margin-top: 0; }
.security-disclaimer { display: flex; align-items: flex-start; margin-top: 8px; padding: 10px 12px; background: #fff2f0; border: 1px solid #ffccc7; border-radius: 6px; line-height: 1.4; }
.private-key-input-area { margin-top: 12px; }
.tip-wallet-status { padding: 8px; border-radius: 4px; font-size: 12px; }
.wallet-info-loading { background: var(--color-primary-light-1, #e8f4ff); border: 1px solid var(--color-primary-light-3, #7bc7ff); color: var(--color-primary, #165dff); display: flex; align-items: center; }
.wallet-info-valid { padding: 8px 15px; background: var(--color-success-light-1, #e8f5e8); border: 1px solid var(--color-success-light-3, #7bc77b); }
.wallet-info-invalid { background: var(--color-danger-light-1, #ffece8); border: 1px solid var(--color-danger-light-3, #f7a9a9); color: var(--color-danger, #f53f3f); display: flex; align-items: center; }
.wallet-address { display: flex; align-items: center; margin-bottom: 4px; font-family: 'Courier New', monospace; color: var(--color-success, #00b42a); font-size: 12px; }
.wallet-balance { display: flex; align-items: center; margin-bottom: 4px; color: var(--color-success, #00b42a); font-size: 12px; }
.wallet-balance.insufficient { color: var(--color-warning, #ff7d00); }
.balance-warning { display: flex; align-items: center; color: var(--color-danger, #f53f3f); font-weight: 500; font-size: 12px; }
.tip-amount-section { margin-bottom: 16px; }
.tip-amount-options { display: flex; gap: 6px; margin: 8px 0; flex-wrap: wrap; }
.tip-amount-btn { flex: 1; min-width: 60px; font-size: 12px; }
.tip-amount-btn.selected { background-color: var(--color-primary-light-1, #e8f4ff); border-color: var(--color-primary, #165dff); color: var(--color-primary, #165dff); }
.tip-mode-switch { display: flex; justify-content: center; margin-top: 16px; margin-bottom: 8px; }
.switch-mode-btn { font-size: 13px; padding: 8px 16px; border-color: var(--color-primary, #165dff); color: var(--color-primary, #165dff); }
.switch-mode-btn:hover { background-color: var(--color-primary-light-1, #e8f4ff); border-color: var(--color-primary, #165dff); color: var(--color-primary, #165dff); }
.tip-footer { display: flex; justify-content: center; gap: 12px; }
</style>
