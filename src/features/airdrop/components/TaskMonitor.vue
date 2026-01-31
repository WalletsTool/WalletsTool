<script setup>
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue';
import {
  IconPlayArrow,
  IconStop,
  IconRefresh,
  IconDelete,
  IconCheckCircle,
  IconCloseCircle,
  IconLoading,
  IconSchedule,
  IconFile,
  IconSettings
} from '@arco-design/web-vue/es/icon';

// 任务监控状态
const isRunning = ref(false);
const currentTask = ref(null);
const progress = ref(0);
const elapsedTime = ref(0);
const logs = ref([]);
const executionResults = ref([]);
let timer = null;

// 模拟执行数据
const mockWalletResults = [
  { id: 1, address: '0x742d35Cc6634C0532925a3b844Bc454e4438f44e', label: 'Main Wallet', status: 'success', duration: '12.5s', txHash: '0xabc123...' },
  { id: 2, address: '0x1234567890abcdef1234567890abcdef12345678', label: 'Airdrop 1', status: 'success', duration: '11.8s', txHash: '0xdef456...' },
  { id: 3, address: '0xabcdef1234567890abcdef1234567890abcdef12', label: 'Airdrop 2', status: 'failed', duration: '8.2s', error: 'Transaction reverted' },
  { id: 4, address: '0x567890abcdef1234567890abcdef1234567890ab', label: 'Swap Wallet', status: 'running', duration: '-', txHash: '' },
  { id: 5, address: '0xabcdefabcdefabcdefabcdefabcdefabcdefabcd', label: 'Airdrop 3', status: 'pending', duration: '-', txHash: '' },
];

const stats = computed(() => ({
  total: executionResults.value.length,
  success: executionResults.value.filter(r => r.status === 'success').length,
  failed: executionResults.value.filter(r => r.status === 'failed').length,
  running: executionResults.value.filter(r => r.status === 'running').length,
  pending: executionResults.value.filter(r => r.status === 'pending').length
}));

// 日志颜色映射
const getLogColor = (type) => {
  const colors = {
    info: 'var(--color-text-2)',
    success: 'rgb(var(--success-6))',
    warning: 'rgb(var(--warning-6))',
    error: 'rgb(var(--danger-6))'
  };
  return colors[type] || colors.info;
};

// 添加日志
const addLog = (type, message) => {
  const time = new Date().toLocaleTimeString();
  logs.value.unshift({ time, type, message });
};

// 格式化时间
const formatTime = (seconds) => {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
};

// 选择要监控的任务
const selectTask = (task) => {
  currentTask.value = task;
  logs.value = [];
  executionResults.value = [];
  progress.value = 0;
  elapsedTime.value = 0;
};

// 启动执行
const handleStart = () => {
  if (!currentTask.value) {
    return;
  }

  isRunning.value = true;
  addLog('info', `开始执行任务: ${currentTask.value.name}`);
  addLog('info', `初始化浏览器环境...`);

  // 模拟执行
  executionResults.value = mockWalletResults.map(w => ({ ...w }));
  timer = setInterval(() => {
    elapsedTime.value++;

    // 模拟进度更新
    const pendingCount = executionResults.value.filter(r => r.status === 'pending').length;
    const runningCount = executionResults.value.filter(r => r.status === 'running').length;

    if (pendingCount > 0 && runningCount < 3) {
      const nextPending = executionResults.value.find(r => r.status === 'pending');
      if (nextPending) {
        nextPending.status = 'running';
        addLog('info', `开始执行钱包: ${nextPending.address.slice(0, 8)}...${nextPending.address.slice(-4)}`);
      }
    }

    // 模拟完成
    executionResults.value.forEach(r => {
      if (r.status === 'running' && Math.random() > 0.7) {
        r.status = Math.random() > 0.2 ? 'success' : 'failed';
        r.duration = `${(Math.random() * 10 + 5).toFixed(1)}s`;
        if (r.status === 'success') {
          r.txHash = '0x' + Math.random().toString(16).slice(2, 10) + '...';
          addLog('success', `钱包 ${r.address.slice(0, 8)}...${r.address.slice(-4)} 执行成功`);
        } else {
          r.error = 'Transaction reverted';
          addLog('error', `钱包 ${r.address.slice(0, 8)}...${r.address.slice(-4)} 执行失败: ${r.error}`);
        }
      }
    });

    // 更新进度
    const completed = executionResults.value.filter(r => r.status === 'success' || r.status === 'failed').length;
    progress.value = Math.round((completed / executionResults.value.length) * 100);

    // 检查是否全部完成
    if (completed === executionResults.value.length) {
      clearInterval(timer);
      isRunning.value = false;
      addLog('success', `任务执行完成 - 成功: ${stats.value.success}, 失败: ${stats.value.failed}`);
    }
  }, 1500);
};

// 停止执行
const handleStop = () => {
  clearInterval(timer);
  isRunning.value = false;
  addLog('warning', '任务已手动停止');
};

// 清空日志
const clearLogs = () => {
  logs.value = [];
};

// 清空结果
const clearResults = () => {
  executionResults.value = [];
  progress.value = 0;
  elapsedTime.value = 0;
  logs.value = [];
};

// 格式化地址
const formatAddress = (address) => {
  if (!address) return '-';
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
};

onUnmounted(() => {
  if (timer) {
    clearInterval(timer);
  }
});
</script>

<template>
  <div class="task-monitor">
    <!-- 任务选择 -->
    <div class="task-selector">
      <div class="selector-header">
        <span class="header-title">选择监控任务</span>
        <a-button type="text" size="mini" @click="selectTask({ name: 'OKX Daily Claim', id: 1 })">
          <template #icon><icon-refresh /></template>
        </a-button>
      </div>
      <div class="task-dropdown">
        <div
          v-for="task in [{ name: 'OKX Daily Claim', id: 1, status: 'enabled' }, { name: 'Weekly Swap', id: 2, status: 'paused' }]"
          :key="task.id"
          class="task-option"
          :class="{ active: currentTask && currentTask.id === task.id }"
          @click="selectTask(task)"
        >
          <div class="task-info">
            <span class="task-name">{{ task.name }}</span>
            <a-tag :color="task.status === 'enabled' ? 'green' : 'orange'" size="mini">
              {{ task.status === 'enabled' ? '运行中' : '已暂停' }}
            </a-tag>
          </div>
        </div>
      </div>
    </div>

    <!-- 监控面板 -->
    <div class="monitor-panel" v-if="currentTask">
      <!-- 状态概览 -->
      <div class="status-overview">
        <div class="stat-card">
          <div class="stat-icon">
            <icon-schedule />
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ formatTime(elapsedTime) }}</div>
            <div class="stat-label">执行时间</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-icon total">
            <icon-file />
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.total }}</div>
            <div class="stat-label">总任务</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-icon success">
            <icon-check-circle />
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.success }}</div>
            <div class="stat-label">成功</div>
          </div>
        </div>
        <div class="stat-card">
          <div class="stat-icon error">
            <icon-close-circle />
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ stats.failed }}</div>
            <div class="stat-label">失败</div>
          </div>
        </div>
      </div>

      <!-- 控制栏 -->
      <div class="control-bar">
        <div class="task-info">
          <span class="task-title">{{ currentTask.name }}</span>
          <span class="task-progress">{{ progress }}%</span>
        </div>
        <div class="control-actions">
          <a-button type="primary" status="success" @click="handleStart" :disabled="isRunning">
            <template #icon><icon-play-arrow /></template>
            开始执行
          </a-button>
          <a-button type="primary" status="danger" @click="handleStop" :disabled="!isRunning">
            <template #icon><icon-stop /></template>
            停止
          </a-button>
        </div>
      </div>

      <!-- 进度条 -->
      <div class="progress-section">
        <a-progress :percent="progress" :show-text="false" :status="progress === 100 ? 'success' : 'active'" />
      </div>

      <!-- 分割视图 -->
      <div class="split-view">
        <!-- 执行队列 -->
        <div class="queue-panel">
          <div class="panel-header">
            <span>执行队列</span>
            <a-space>
              <span class="queue-count">{{ stats.running }} 运行中 / {{ stats.pending }} 等待</span>
            </a-space>
          </div>
          <div class="queue-list">
            <div
              v-for="result in executionResults"
              :key="result.id"
              class="queue-item"
              :class="result.status"
            >
              <div class="item-status">
                <icon-loading v-if="result.status === 'running'" spin />
                <icon-schedule v-else-if="result.status === 'pending'" />
                <icon-check-circle v-else-if="result.status === 'success'" />
                <icon-close-circle v-else-if="result.status === 'failed'" />
              </div>
              <div class="item-info">
                <div class="item-address">{{ formatAddress(result.address) }}</div>
                <div class="item-label">{{ result.label }}</div>
              </div>
              <div class="item-meta">
                <div class="item-duration">{{ result.duration }}</div>
                <div class="item-tx" v-if="result.txHash">{{ result.txHash }}</div>
                <div class="item-error" v-if="result.error">{{ result.error }}</div>
              </div>
            </div>
            <div v-if="executionResults.length === 0" class="empty-queue">
              <icon-file style="font-size: 32px; color: var(--color-text-4)" />
              <p>暂无执行任务</p>
            </div>
          </div>
        </div>

        <!-- 实时日志 -->
        <div class="log-panel">
          <div class="panel-header">
            <span>实时日志</span>
            <a-space>
              <a-button type="text" size="mini" @click="clearLogs" :disabled="logs.length === 0">清空</a-button>
              <a-button type="text" size="mini" @click="clearResults" :disabled="executionResults.length > 0 && !isRunning">重置</a-button>
            </a-space>
          </div>
          <div class="log-container">
            <div v-for="(log, index) in logs" :key="index" class="log-line" :style="{ color: getLogColor(log.type) }">
              <span class="log-time">[{{ log.time }}]</span>
              <span class="log-message">{{ log.message }}</span>
            </div>
            <div v-if="logs.length === 0" class="empty-logs">
              <p>暂无日志</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div class="empty-state" v-else>
      <icon-settings style="font-size: 48px; color: var(--color-text-4)" />
      <p>请从左侧选择要监控的任务</p>
    </div>
  </div>
</template>

<style scoped>
.task-monitor {
  height: 100%;
  display: flex;
  gap: 20px;
}

/* 任务选择器 */
.task-selector {
  width: 250px;
  background: var(--color-bg-2);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--color-border);
  overflow: hidden;
}

.selector-header {
  padding: 12px 15px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-title {
  font-weight: 500;
  font-size: 14px;
}

.task-dropdown {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.task-option {
  padding: 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 8px;
  border: 1px solid transparent;
}

.task-option:hover {
  background: var(--color-fill-2);
}

.task-option.active {
  background: rgba(var(--primary-6), 0.1);
  border-color: rgb(var(--primary-6));
}

.task-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.task-name {
  font-size: 14px;
  font-weight: 500;
}

/* 监控面板 */
.monitor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 15px;
}

/* 状态概览 */
.status-overview {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 15px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 15px;
  background: var(--color-bg-2);
  border-radius: 8px;
  border: 1px solid var(--color-border);
}

.stat-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: var(--color-fill-2);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  color: var(--color-text-3);
}

.stat-icon.total {
  background: rgba(var(--primary-6), 0.1);
  color: rgb(var(--primary-6));
}

.stat-icon.success {
  background: rgba(var(--success-6), 0.1);
  color: rgb(var(--success-6));
}

.stat-icon.error {
  background: rgba(var(--danger-6), 0.1);
  color: rgb(var(--danger-6));
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text-1);
}

.stat-label {
  font-size: 12px;
  color: var(--color-text-3);
}

/* 控制栏 */
.control-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: var(--color-bg-2);
  border-radius: 8px;
  border: 1px solid var(--color-border);
}

.task-info {
  display: flex;
  align-items: center;
  gap: 15px;
}

.task-title {
  font-size: 16px;
  font-weight: 500;
}

.task-progress {
  font-size: 24px;
  font-weight: 600;
  color: rgb(var(--primary-6));
}

.control-actions {
  display: flex;
  gap: 10px;
}

/* 进度条 */
.progress-section {
  padding: 0 5px;
}

/* 分割视图 */
.split-view {
  flex: 1;
  display: flex;
  gap: 20px;
  min-height: 0;
}

.queue-panel, .log-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-2);
  border-radius: 8px;
  border: 1px solid var(--color-border);
  overflow: hidden;
}

.panel-header {
  padding: 12px 15px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: 500;
}

.queue-count {
  font-size: 12px;
  color: var(--color-text-3);
}

.queue-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.queue-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--color-border);
  transition: all 0.2s;
}

.queue-item:hover {
  background: var(--color-fill-2);
}

.queue-item.pending .item-status {
  color: var(--color-text-4);
}

.queue-item.running .item-status {
  color: rgb(var(--primary-6));
}

.queue-item.success .item-status {
  color: rgb(var(--success-6));
}

.queue-item.failed .item-status {
  color: rgb(var(--danger-6));
}

.item-status {
  width: 24px;
  display: flex;
  justify-content: center;
}

.item-info {
  flex: 1;
}

.item-address {
  font-family: monospace;
  font-size: 13px;
  color: var(--color-text-1);
}

.item-label {
  font-size: 12px;
  color: var(--color-text-3);
}

.item-meta {
  text-align: right;
}

.item-duration {
  font-size: 12px;
  color: var(--color-text-3);
}

.item-tx {
  font-family: monospace;
  font-size: 11px;
  color: rgb(var(--success-6));
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-error {
  font-size: 11px;
  color: rgb(var(--danger-6));
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.log-container {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
  background: var(--color-bg-1);
  font-family: 'Fira Code', monospace;
  font-size: 12px;
}

.log-line {
  margin-bottom: 4px;
  line-height: 1.5;
}

.log-time {
  color: var(--color-text-4);
  margin-right: 8px;
}

.empty-queue, .empty-logs {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-4);
  gap: 10px;
}

.empty-logs p, .empty-state p {
  margin: 0;
}

/* 空状态 */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-4);
  background: var(--color-bg-2);
  border-radius: 8px;
  border: 2px dashed var(--color-border);
  gap: 15px;
}
</style>
