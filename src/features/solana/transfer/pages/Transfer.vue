<script setup name="transfer-solana">
import { Icon } from '@iconify/vue';
import { useRouter, useRoute } from "vue-router";
import { IconDelete } from '@arco-design/web-vue/es/icon';
import { computed, defineAsyncComponent, onBeforeMount, onBeforeUnmount, reactive, ref, watch } from "vue";
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
import * as party from 'party-js'

// 懒加载组件
const ChainManagement = defineAsyncComponent(() => import('@/components/ChainManagement.vue'))
const RpcManagement = defineAsyncComponent(() => import('@/components/RpcManagement.vue'))
const TokenManagement = defineAsyncComponent(() => import('@/components/TokenManagement.vue'))
const WalletImportModal = defineAsyncComponent(() => import('@/components/WalletImportModal.vue'))

const router = useRouter();
const route = useRoute();

const windowTitle = ref('Solana 批量转账');

// 表格列定义
const columns = [
  { title: "序号", width: 55, slotName: "index" },
  { title: "发送方私钥", width: 250, dataIndex: "private_key", ellipsis: true, tooltip: true },
  { title: "接收地址", width: 250, dataIndex: "to_addr", ellipsis: true, tooltip: true },
  { title: "转账数量", width: 95, dataIndex: "amount", ellipsis: true, tooltip: true },
  { title: "SOL余额", width: 95, dataIndex: "plat_balance", ellipsis: true, tooltip: true },
  { title: "代币余额", width: 85, dataIndex: "coin_balance", ellipsis: true, tooltip: true },
  { title: "状态", width: 90, slotName: "exec_status", ellipsis: true, tooltip: true },
  { title: "返回信息", dataIndex: "error_msg", ellipsis: true, tooltip: true },
  { title: "操作", width: 55, slotName: "optional", ellipsis: true, tooltip: true }
];

// 基础状态
let tableLoading = ref(false);
let pageLoading = ref(false);
const data = ref([]);
const selectedKeys = ref([]);
const rowSelection = reactive({
  type: "checkbox",
  showCheckedAll: true,
  onlyCurrent: false,
});

// 链和代币配置
const chainValue = ref("sol");
const currentChain = ref({});
const chainFieldNames = { value: "key", label: "scan_url" };
let chainOptions = ref([]);
let coinValue = ref("");
const coinFieldNames = { value: "key", label: "label" };
const coinOptions = ref([]);
let currentCoin = ref({});

// 余额查询状态
let balanceLoading = ref(false);
let balanceStopFlag = ref(false);
let balanceStopStatus = ref(true);

// 转账状态
let startLoading = ref(false);
let stopFlag = ref(false);
let stopStatus = ref(true);
let threadCount = ref(1);

// 表单配置 - 移除Gas相关配置
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

// 弹窗状态
let addCoinVisible = ref(false);
let coinAddress = ref("");
let deleteTokenVisible = ref(false);
let deleteItemVisible = ref(false);
let currentItemKey = ref("");
let currentItemPrivateKey = ref("");

// 组件引用
const chainManageRef = ref(null);
const rpcManageRef = ref(null);
const tokenManageRef = ref(null);
const walletImportRef = ref(null);

// 进度相关
const transferProgress = ref(0);
const transferTotal = ref(0);
const transferCompleted = ref(0);
const showProgress = ref(false);
const balanceProgress = ref(0);
const balanceTotal = ref(0);
const balanceCompleted = ref(0);
const showBalanceProgress = ref(false);

// 转账配置计算属性
const transferConfig = computed(() => {
  return {
    error_count_limit: 3,
    error_retry: form.error_retry,
    chain: chainValue.value,
    delay: [
      Number(form.min_interval) || 1,
      Number(form.max_interval) || 3
    ],
    transfer_type: form.send_type,
    transfer_amount_list: [
      Number(form.send_min_count) || 0,
      Number(form.send_max_count) || 0
    ],
    amount_precision: Number(form.amount_precision) || 6
  };
});

// 统计数据计算属性
const transferStatistics = computed(() => {
  const total = data.value.length;
  const pending = data.value.filter(item => item.exec_status === '0').length;
  const processing = data.value.filter(item => item.exec_status === '1').length;
  const succeeded = data.value.filter(item => item.exec_status === '2').length;
  const failed = data.value.filter(item => item.exec_status === '3').length;

  return { total, pending, processing, succeeded, failed };
});

// Solana地址验证
function validateSolanaAddress(address) {
  if (!address || typeof address !== 'string') return false;
  return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address);
}

// Solana私钥验证
function validateSolanaPrivateKey(privateKey) {
  if (!privateKey || typeof privateKey !== 'string') return false;
  return privateKey.length >= 80 && privateKey.length <= 90;
}

// 点击行选中
function rowClick(record, event) {
  const index = selectedKeys.value.indexOf(record.key);
  index >= 0
    ? selectedKeys.value.splice(index, 1)
    : selectedKeys.value.push(record.key);
}

// 链变化事件
async function chainChange() {
  const chainResult = chainOptions.value.filter(
    (item) => item.key === chainValue.value
  );

  if (chainResult.length > 0) {
    currentChain.value = chainResult[0];
    
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

        coinOptions.value.sort((a, b) => {
          if (a.coin_type === 'base' && b.coin_type !== 'base') {
            return -1;
          }
          if (a.coin_type !== 'base' && b.coin_type === 'base') {
            return 1;
          }
          return a.label.localeCompare(b.label);
        });

        if (coinOptions.value.length > 0) {
          coinValue.value = coinOptions.value[0].key;
          currentCoin.value = coinOptions.value[0];
        }
      }
    } catch (error) {
      console.error('获取代币列表失败:', error);
      Notification.error({ content: '获取代币列表失败', position: 'topLeft' });
    }
  }
}

// 代币变化事件
function coinChange() {
  const coinResult = coinOptions.value.filter(
    (item) => item.key === coinValue.value
  );
  if (coinResult.length > 0) {
    currentCoin.value = coinResult[0];
  }
}

// 查询余额
async function queryBalance() {
  if (data.value.length === 0) {
    Notification.error({ content: '请先导入转账数据', position: 'topLeft' });
    return;
  }

  balanceLoading.value = true;
  balanceStopFlag.value = false;
  balanceStopStatus.value = false;
  
  try {
    const params = {
      chain: chainValue.value,
      coin_config: {
        coin_type: currentCoin.value.coin_type,
        contract_address: currentCoin.value.contract_address || null,
        abi: currentCoin.value.abi || null
      },
      items: data.value.map(item => ({
        key: item.key,
        private_key: item.private_key,
        plat_balance: null,
        coin_balance: null,
        exec_status: '0',
        error_msg: null
      })),
      thread_count: Math.min(threadCount.value, 10)
    };

    const result = await invoke('query_balances', { params });
    
    if (result && result.success) {
      Notification.success({ content: '余额查询完成', position: 'topLeft' });
    } else {
      Notification.error('余额查询失败: ' + (result?.error || '未知错误'));
    }
  } catch (error) {
    console.error('查询余额失败:', error);
    Notification.error('查询余额失败: ' + error.message);
  } finally {
    balanceLoading.value = false;
    balanceStopStatus.value = true;
  }
}

// 开始转账
async function startTransfer() {
  if (data.value.length === 0) {
    Notification.error({ content: '请先导入转账数据', position: 'topLeft' });
    return;
  }

  const validData = data.value.filter(item => 
    item.exec_status === '0' || item.exec_status === '3'
  );

  if (validData.length === 0) {
    Notification.warning({ content: '没有可执行的转账数据', position: 'topLeft' });
    return;
  }

  startLoading.value = true;
  stopFlag.value = false;
  stopStatus.value = false;

  try {
    const result = await invoke('execute_batch_transfer', {
      chain: chainValue.value,
      items: validData.map(item => ({
        key: item.key,
        private_key: item.private_key,
        to_addr: item.to_addr,
        amount: item.amount || '0',
        error_msg: item.error_msg || '',
        error_count: item.error_count || 0
      })),
      config: {
        ...transferConfig.value,
        coin_config: {
          coin_type: currentCoin.value.coin_type,
          contract_address: currentCoin.value.contract_address || null,
          abi: currentCoin.value.abi || null
        }
      },
      thread_count: threadCount.value
    });

    if (result && result.success) {
      const stats = transferStatistics.value;
      Notification.success({ content: `转账执行完成！成功: ${stats.succeeded}, 失败: ${stats.failed}`, position: 'topLeft' });
    } else {
      Notification.error('转账执行失败: ' + (result?.error || '未知错误'));
    }
  } catch (error) {
    console.error('执行转账失败:', error);
    Notification.error('执行转账失败: ' + error.message);
  } finally {
    startLoading.value = false;
    stopStatus.value = true;
  }
}

// 初始化
async function initializeData() {
  pageLoading.value = true;
  
  try {
    const chainList = await invoke("get_chain_list");
    
    chainOptions.value = chainList
      .filter(chain => chain.key === 'sol' || chain.key.startsWith('sol-'))
      .sort((a, b) => a.name.localeCompare(b.name));

    if (chainOptions.value.length > 0) {
      chainValue.value = chainOptions.value[0].key;
      currentChain.value = chainOptions.value[0];
      await chainChange();
    }
  } catch (error) {
    console.error('初始化失败:', error);
    Notification.error('初始化失败: ' + error.message);
  } finally {
    pageLoading.value = false;
  }
}

// 生命周期
onBeforeMount(async () => {
  await initializeData();
});

// 防抖函数
const debouncedQueryBalance = customDebounce(queryBalance, 500);
const debouncedStartTransfer = customDebounce(startTransfer, 800);
</script>

<template>
  <div class="page-container">
    <TitleBar :title="windowTitle" />
    
    <div v-if="pageLoading" class="page-loading">
      <a-spin size="large" tip="初始化中..." />
    </div>
    
    <div v-else class="main-content">
      <!-- 配置区域 -->
      <div class="config-section">
        <a-form layout="inline" style="padding: 15px; background: var(--color-fill-1); border-radius: 6px; margin-bottom: 10px;">
          <a-row style="width: 100%; margin-bottom: 10px;">
            <a-form-item label="Solana网络" style="width: 200px;">
              <a-select v-model="chainValue" @change="chainChange" style="width: 180px;">
                <a-option v-for="chain in chainOptions" :key="chain.key" :value="chain.key">
                  <div style="display: flex; align-items: center; gap: 8px;">
                    <ChainIcon :chain="chain.key" :size="16" />
                    <span>{{ chain.name }}</span>
                  </div>
                </a-option>
              </a-select>
            </a-form-item>
            
            <a-form-item label="代币" style="width: 160px;">
              <a-select v-model="coinValue" @change="coinChange" style="width: 140px;">
                <a-option v-for="coin in coinOptions" :key="coin.key" :value="coin.key">
                  {{ coin.label }}
                </a-option>
              </a-select>
            </a-form-item>
            
            <a-form-item label="线程数" style="width: 120px;">
              <a-input-number v-model="threadCount" :min="1" :max="10" style="width: 80px;" />
            </a-form-item>
          </a-row>
          
          <a-row style="width: 100%;">
            <a-form-item label="转账类型">
              <a-radio-group v-model="form.send_type" type="button">
                <a-radio value="2">指定数量</a-radio>
                <a-radio value="3">随机数量</a-radio>
              </a-radio-group>
            </a-form-item>
            
            <a-form-item v-if="form.send_type === '3'" label="数量范围">
              <a-input-group>
                <a-input v-model="form.send_min_count" style="width: 80px;" placeholder="最小" />
                <span style="padding: 0 5px;">至</span>
                <a-input v-model="form.send_max_count" style="width: 80px;" placeholder="最大" />
              </a-input-group>
            </a-form-item>
            
            <a-form-item label="时间间隔">
              <a-input-group>
                <a-input v-model="form.min_interval" style="width: 60px;" />
                <span style="padding: 0 5px;">至</span>
                <a-input v-model="form.max_interval" style="width: 60px;" />
              </a-input-group>
            </a-form-item>
          </a-row>
        </a-form>
      </div>

      <!-- 统计信息 -->
      <div class="stats-section" style="padding: 10px 20px; background: var(--color-fill-1); border-radius: 6px; margin-bottom: 10px;">
        <a-row>
          <a-col :span="6">
            <a-statistic title="总计" :value="transferStatistics.total" />
          </a-col>
          <a-col :span="6">
            <a-statistic title="等待中" :value="transferStatistics.pending" value-style="color: #909399" />
          </a-col>
          <a-col :span="6">
            <a-statistic title="成功" :value="transferStatistics.succeeded" value-style="color: #67C23A" />
          </a-col>
          <a-col :span="6">
            <a-statistic title="失败" :value="transferStatistics.failed" value-style="color: #F56C6C" />
          </a-col>
        </a-row>
      </div>
      
      <!-- 数据表格 -->
      <div style="flex: 1; min-height: 0;">
        <VirtualScrollerTable
          :columns="columns"
          :data="data"
          :row-selection="rowSelection"
          :selected-keys="selectedKeys"
          :loading="tableLoading"
          :scroll="{ y: 'calc(100vh - 350px)' }"
          @row-click="rowClick"
          row-key="key"
        >
          <template #index="{ record, rowIndex }">
            {{ rowIndex + 1 }}
          </template>
          
          <template #exec_status="{ record }">
            <a-tag v-if="record.exec_status === '0'" color="gray">等待中</a-tag>
            <a-tag v-else-if="record.exec_status === '1'" color="blue">执行中</a-tag>
            <a-tag v-else-if="record.exec_status === '2'" color="green">成功</a-tag>
            <a-tag v-else-if="record.exec_status === '3'" color="red">失败</a-tag>
            <a-tag v-else color="gray">未知</a-tag>
          </template>
          
          <template #optional="{ record }">
            <a-button size="mini" status="danger">
              <template #icon>
                <IconDelete />
              </template>
            </a-button>
          </template>
        </VirtualScrollerTable>
      </div>
      
      <!-- 功能配置区 -->
      <div style="display: flex; padding-top: 15px; flex-shrink: 0;">
        <a-form :model="form" :style="{ width: '100%' }" layout="horizontal" :label-col-props="{ span: 8 }" :wrapper-col-props="{ span: 16 }">
          <a-row style="display: flex; gap: 20px;">
            <!-- 第一列 -->
            <div style="flex: 1;">
              <a-form-item label="Solana网络">
                <a-select v-model="chainValue" @change="chainChange" style="width: 100%;">
                  <a-option v-for="chain in chainOptions" :key="chain.key" :value="chain.key">
                    <div style="display: flex; align-items: center; gap: 8px;">
                      <ChainIcon :chain="chain.key" :size="16" />
                      <span>{{ chain.name }}</span>
                    </div>
                  </a-option>
                </a-select>
              </a-form-item>
              <a-form-item label="代币">
                <a-select v-model="coinValue" @change="coinChange" style="width: 100%;">
                  <a-option v-for="coin in coinOptions" :key="coin.key" :value="coin.key">
                    {{ coin.label }}
                  </a-option>
                </a-select>
              </a-form-item>
            </div>
            <a-divider direction="vertical" style="height: 100%; margin: 0;" />
            <!-- 第二列 -->
            <div style="flex: 1;">
              <a-form-item label="线程数">
                <a-input-number v-model="threadCount" :min="1" :max="10" style="width: 100%;" />
              </a-form-item>
              <a-form-item label="转账类型">
                <a-radio-group v-model="form.send_type" type="button">
                  <a-radio value="2">指定数量</a-radio>
                  <a-radio value="3">随机数量</a-radio>
                </a-radio-group>
              </a-form-item>
              <a-form-item v-if="form.send_type === '3'" label="数量范围">
                <a-space>
                  <a-input v-model="form.send_min_count" placeholder="最小" style="width: 80px;" />
                  <span>至</span>
                  <a-input v-model="form.send_max_count" placeholder="最大" style="width: 80px;" />
                </a-space>
              </a-form-item>
            </div>
            <a-divider direction="vertical" style="height: 100%; margin: 0;" />
            <!-- 第三列 -->
            <div style="flex: 1;">
              <a-form-item label="时间间隔">
                <a-space>
                  <a-input v-model="form.min_interval" placeholder="最小" style="width: 80px;" />
                  <span>至</span>
                  <a-input v-model="form.max_interval" placeholder="最大" style="width: 80px;" />
                </a-space>
              </a-form-item>
              <a-form-item :label-col-props="{ span: 0 }" :wrapper-col-props="{ span: 24 }">
                <div style="display: flex; gap: 20px; justify-content: center; align-items: center;">
                  <a-button
                      v-if="!balanceLoading"
                      type="primary"
                      class="core-action-btn primary-btn"
                      @click="debouncedQueryBalance"
                  >
                    <template #icon>
                      <Icon icon="mdi:magnify"/>
                    </template>
                    查询余额
                  </a-button>
                  <a-button
                      v-else
                      class="core-action-btn primary-btn executing"
                      loading
                  >
                    <template #icon>
                      <Icon icon="mdi:pause-circle"/>
                    </template>
                    查询中...
                  </a-button>
                  <a-button
                      v-if="!startLoading"
                      type="success"
                      class="core-action-btn success-btn"
                      @click="debouncedStartTransfer"
                  >
                    <template #icon>
                      <Icon icon="mdi:rocket-launch"/>
                    </template>
                    执行转账
                  </a-button>
                  <a-button
                      v-else
                      class="core-action-btn success-btn executing"
                      loading
                  >
                    <template #icon>
                      <Icon icon="mdi:rocket-launch"/>
                    </template>
                    执行中...
                  </a-button>
                </div>
              </a-form-item>
            </div>
          </a-row>
        </a-form>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--color-bg-1);
}

.page-loading {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 10px;
  overflow: hidden;
}

.config-section {
  flex-shrink: 0;
}

.stats-section {
  flex-shrink: 0;
}

.main-content :deep(.arco-form-item-label-col) {
  margin-bottom: 0;
}

.main-content :deep(.arco-form-item-wrapper-col) {
  flex: 1;
}

.main-content :deep(.arco-form-item) {
  margin-bottom: 8px;
  padding: 4px 10px;
}

.main-content :deep(.arco-form-item-label) {
  line-height: 32px;
}

/* 核心功能按钮样式 */
.core-action-btn {
  width: 130px;
  height: 48px;
  font-size: 15px;
  font-weight: 600;
  color: #ffffff;
  border: none;
  border-radius: 16px;
  will-change: transform, background-color, box-shadow;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.core-action-btn::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.25), transparent);
  transition: left 0.5s ease;
}

.core-action-btn:hover::before {
  left: 100%;
}

.core-action-btn:hover {
  color: #ffffff;
  transform: translateY(-3px) scale(1.02);
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.35);
}

.core-action-btn.primary-btn {
  background: linear-gradient(135deg, #9945ff 0%, #14f195 100%);
  box-shadow: 0 6px 20px rgba(153, 69, 255, 0.45), 0 2px 8px rgba(153, 69, 255, 0.2);
}

.core-action-btn.primary-btn:hover {
  background: linear-gradient(135deg, #14f195 0%, #9945ff 100%);
  box-shadow: 0 10px 30px rgba(153, 69, 255, 0.55), 0 4px 12px rgba(153, 69, 255, 0.3);
}

.core-action-btn.executing.primary-btn {
  background: linear-gradient(135deg, #14f195 0%, #9945ff 100%);
  box-shadow: 0 6px 20px rgba(153, 69, 255, 0.45), 0 2px 8px rgba(153, 69, 255, 0.2);
  animation: pulse-solana 2s ease-in-out infinite;
}

@keyframes pulse-solana {
  0%, 100% {
    box-shadow: 0 6px 20px rgba(153, 69, 255, 0.45), 0 2px 8px rgba(153, 69, 255, 0.2);
  }
  50% {
    box-shadow: 0 6px 35px rgba(153, 69, 255, 0.7), 0 3px 12px rgba(153, 69, 255, 0.35);
  }
}

.core-action-btn.success-btn {
  background: linear-gradient(135deg, #10a85c 0%, #12c47d 50%, #14e08e 100%);
  box-shadow: 0 6px 20px rgba(16, 168, 92, 0.45), 0 2px 8px rgba(16, 168, 92, 0.2);
}

.core-action-btn.success-btn:hover {
  background: linear-gradient(135deg, #12c47d 0%, #14e08e 50%, #16f89f 100%);
  box-shadow: 0 10px 30px rgba(16, 168, 92, 0.55), 0 4px 12px rgba(16, 168, 92, 0.3);
}

.core-action-btn.executing.success-btn {
  background: linear-gradient(135deg, #12c47d 0%, #14e08e 100%);
  box-shadow: 0 6px 20px rgba(16, 168, 92, 0.45), 0 2px 8px rgba(16, 168, 92, 0.2);
  animation: pulse-success-soft 2s ease-in-out infinite;
}

@keyframes pulse-success-soft {
  0%, 100% {
    box-shadow: 0 6px 20px rgba(16, 168, 92, 0.45), 0 2px 8px rgba(16, 168, 92, 0.2);
  }
  50% {
    box-shadow: 0 6px 35px rgba(16, 168, 92, 0.7), 0 3px 12px rgba(16, 168, 92, 0.35);
  }
}

/* Solana风格图标动画 */
.core-action-btn .arco-icon {
  transition: transform 0.3s ease, opacity 0.3s ease;
  filter: drop-shadow(0 1px 2px rgba(0,0,0,0.2));
}

.core-action-btn:hover .arco-icon {
  transform: scale(1.15);
}

.core-action-btn.executing .arco-icon {
  animation: icon-spin 1s linear infinite;
}

@keyframes icon-spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}
</style>