<script setup>
import { ref } from 'vue';
import { 
  IconPlayArrow, 
  IconStop, 
  IconLoading, 
  IconRecord,
  IconCheckCircle,
  IconCloseCircle
} from '@arco-design/web-vue/es/icon';

const isRunning = ref(false);
const progress = ref(0);
const logs = ref([]);
const activeTasks = ref([
  { id: 1, wallet: '0x123...abc', status: 'pending', log: '等待执行...' },
  { id: 2, wallet: '0x456...def', status: 'pending', log: '等待执行...' },
]);

const handleStart = () => {
  isRunning.value = true;
  addLog('info', '任务已启动，正在初始化执行环境...');
  
  // Simulation
  let p = 0;
  const timer = setInterval(() => {
    p += 10;
    progress.value = p;
    if (p >= 100) {
      clearInterval(timer);
      isRunning.value = false;
      addLog('success', '所有任务执行完成');
    } else {
       addLog('info', `正在执行批次任务... 进度 ${p}%`);
    }
  }, 1000);
};

const handleStop = () => {
  isRunning.value = false;
  addLog('warning', '任务已手动停止');
};

const addLog = (type, content) => {
  const time = new Date().toLocaleTimeString();
  logs.value.unshift({ time, type, content });
};

const clearLogs = () => {
  logs.value = [];
};
</script>

<template>
  <div class="execution-panel">
    <!-- Control Bar -->
    <div class="control-bar">
      <div class="status-overview">
        <div class="stat-item">
          <span class="label">总任务</span>
          <span class="value">2</span>
        </div>
        <div class="stat-item">
          <span class="label">成功</span>
          <span class="value success">0</span>
        </div>
        <div class="stat-item">
          <span class="label">失败</span>
          <span class="value error">0</span>
        </div>
      </div>

      <div class="control-actions">
        <a-button type="primary" status="success" size="large" @click="handleStart" :loading="isRunning">
          <template #icon><icon-play-arrow /></template>
          开始执行
        </a-button>
        <a-button type="primary" status="danger" size="large" @click="handleStop" :disabled="!isRunning">
          <template #icon><icon-stop /></template>
          停止
        </a-button>
      </div>
    </div>

    <!-- Progress -->
    <div class="progress-section">
      <div class="progress-info">
        <span>总进度</span>
        <span>{{ progress }}%</span>
      </div>
      <a-progress :percent="progress / 100" :show-text="false" status="active" />
    </div>

    <!-- Split View: Tasks & Logs -->
    <div class="split-view">
      <!-- Task List -->
      <div class="task-list-panel">
        <div class="panel-title">执行队列</div>
        <div class="task-list">
          <div v-for="task in activeTasks" :key="task.id" class="task-item">
            <div class="task-status">
               <icon-loading spin v-if="isRunning" />
               <icon-record v-else />
            </div>
            <div class="task-wallet">{{ task.wallet }}</div>
            <div class="task-log">{{ task.log }}</div>
          </div>
        </div>
      </div>

      <!-- Live Logs -->
      <div class="log-panel">
        <div class="panel-header">
          <span>实时日志</span>
          <a-button type="text" size="mini" @click="clearLogs">清空</a-button>
        </div>
        <div class="log-container">
          <div v-for="(log, index) in logs" :key="index" class="log-line" :class="log.type">
            <span class="log-time">[{{ log.time }}]</span>
            <span class="log-content">{{ log.content }}</span>
          </div>
          <div v-if="logs.length === 0" class="empty-logs">暂无日志</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.execution-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.control-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  background: var(--color-bg-2);
  border-radius: 12px;
  border: 1px solid var(--color-border);
}

.status-overview {
  display: flex;
  gap: 30px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.stat-item .label {
  font-size: 12px;
  color: var(--color-text-3);
}

.stat-item .value {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text-1);
}

.stat-item .value.success { color: rgb(var(--success-6)); }
.stat-item .value.error { color: rgb(var(--danger-6)); }

.control-actions {
  display: flex;
  gap: 15px;
}

.progress-section {
  padding: 0 5px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 13px;
  color: var(--color-text-3);
}

.split-view {
  flex: 1;
  display: flex;
  gap: 20px;
  overflow: hidden;
}

.task-list-panel, .log-panel {
  flex: 1;
  background: var(--color-bg-2);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--color-border);
}

.panel-title, .panel-header {
  padding: 12px 15px;
  border-bottom: 1px solid var(--color-border);
  font-weight: 500;
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: var(--color-text-1);
}

.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.task-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px;
  border-bottom: 1px solid var(--color-border);
}

.task-status {
  width: 20px;
  display: flex;
  justify-content: center;
  color: var(--color-text-3);
}

.task-wallet {
  font-family: monospace;
  color: rgb(var(--primary-6));
}

.task-log {
  color: var(--color-text-3);
  font-size: 13px;
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
  line-height: 1.4;
}

.log-time {
  color: var(--color-text-4);
  margin-right: 8px;
}

.log-line.info .log-content { color: var(--color-text-2); }
.log-line.success .log-content { color: rgb(var(--success-6)); }
.log-line.warning .log-content { color: rgb(var(--warning-6)); }
.log-line.error .log-content { color: rgb(var(--danger-6)); }

.empty-logs {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-4);
}
</style>
