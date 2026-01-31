<script setup>
import { ref, reactive, computed, watch } from 'vue';
import { Message, Modal } from '@arco-design/web-vue';
import {
  IconPlus,
  IconEdit,
  IconDelete,
  IconPlayArrow,
  IconPause,
  IconSettings,
  IconSchedule,
  IconSafe,
  IconUser,
  IconCheck
} from '@arco-design/web-vue/es/icon';

// 任务表单数据
const formData = reactive({
  name: '',
  description: '',
  walletSelectionType: 'all', // all, group, specific
  walletGroupId: null,
  selectedWallets: [],
  scriptId: null,
  browserProfileId: null,
  scheduleType: 'once', // once, interval, cron
  intervalSeconds: 3600,
  maxExecutions: 1,
  unlimitedExecutions: false,
  cronExpression: '0 0 * * *',
  timeoutSeconds: 300,
  retryTimes: 3,
  retryIntervalSeconds: 60,
  concurrency: 1,
  notifyOnSuccess: false,
  notifyOnFailure: true,
  enableProxy: true,
  enableFingerprintProtection: true,
  injectExtensions: ['metamask', 'okxwallet']
});

// 模拟数据
const walletGroups = ref([
  { id: 1, name: 'Default Group', walletCount: 5 },
  { id: 2, name: 'Group A - Airdrop', walletCount: 10 },
  { id: 3, name: 'Group B - Swap', walletCount: 3 },
]);

const scripts = ref([
  { id: 1, name: 'OKX Daily Claim', version: '1.2', description: 'OKX daily rewards claim' },
  { id: 2, name: 'Uniswap V3 Swap', version: '2.0', description: 'Swap ETH for tokens' },
  { id: 3, name: 'LayerZero Bridge', version: '1.0', description: 'Cross-chain bridge' },
  { id: 4, name: 'Custom Script', version: '1.0', description: 'User defined script' },
]);

const browserProfiles = ref([
  { id: 1, name: 'Default Profile', description: 'Standard browser profile' },
  { id: 2, name: 'Mobile Profile', description: 'iPhone simulation' },
  { id: 3, name: 'Incognito Profile', description: 'No tracking' },
]);

const allWallets = ref([
  { id: 1, address: '0x742d35Cc6634C0532925a3b844Bc454e4438f44e', label: 'Main Wallet', group: 'Default' },
  { id: 2, address: '0x1234567890abcdef1234567890abcdef12345678', label: 'Airdrop 1', group: 'Group A' },
  { id: 3, address: '0xabcdef1234567890abcdef1234567890abcdef12', label: 'Airdrop 2', group: 'Group A' },
  { id: 4, address: '0x567890abcdef1234567890abcdef1234567890ab', label: 'Swap Wallet', group: 'Group B' },
]);

// 任务列表
const tasks = ref([
  {
    id: 1,
    name: 'OKX Daily Claim Task',
    description: 'Automatically claim OKX daily rewards',
    status: 'enabled',
    scheduleType: 'interval',
    nextRunTime: '2024-01-15 08:00:00',
    totalRuns: 45,
    successRuns: 43,
    failedRuns: 2,
    scriptName: 'OKX Daily Claim',
    walletCount: 5
  },
  {
    id: 2,
    name: 'Weekly Portfolio Check',
    description: 'Check portfolio balance weekly',
    status: 'paused',
    scheduleType: 'cron',
    nextRunTime: '2024-01-20 00:00:00',
    totalRuns: 12,
    successRuns: 12,
    failedRuns: 0,
    scriptName: 'Portfolio Check',
    walletCount: 10
  }
]);

const isModalVisible = ref(false);
const isEditing = ref(false);
const currentTaskId = ref(null);
const activeTab = ref('basic');

// 钱包选择相关
const walletSearch = ref('');
const walletSelectMode = ref('all'); // all, group, specific

const filteredWallets = computed(() => {
  if (!walletSearch.value) return allWallets.value;
  const search = walletSearch.value.toLowerCase();
  return allWallets.value.filter(w =>
    w.address.toLowerCase().includes(search) ||
    w.label.toLowerCase().includes(search)
  );
});

const walletCount = computed(() => {
  if (walletSelectMode.value === 'all') return allWallets.value.length;
  if (walletSelectMode.value === 'group') {
    const group = walletGroups.value.find(g => g.id === formData.walletGroupId);
    return group ? group.walletCount : 0;
  }
  return formData.selectedWallets.length;
});

const isWalletSelected = (walletId) => {
  return formData.selectedWallets.includes(walletId);
};

const toggleWalletSelection = (walletId) => {
  const index = formData.selectedWallets.indexOf(walletId);
  if (index === -1) {
    formData.selectedWallets.push(walletId);
  } else {
    formData.selectedWallets.splice(index, 1);
  }
};

const selectAllWallets = () => {
  formData.selectedWallets = filteredWallets.value.map(w => w.id);
};

const deselectAllWallets = () => {
  formData.selectedWallets = [];
};

// 任务操作
const handleCreateTask = () => {
  resetForm();
  isEditing.value = false;
  isModalVisible.value = true;
};

const handleEditTask = (task) => {
  currentTaskId.value = task.id;
  isEditing.value = true;
  // 加载任务数据到表单
  formData.name = task.name;
  formData.description = task.description;
  isModalVisible.value = true;
};

const handleDeleteTask = (task) => {
  Modal.warning({
    title: '确认删除',
    content: `确定要删除任务 "${task.name}" 吗？此操作不可恢复。`,
    onOk: () => {
      tasks.value = tasks.value.filter(t => t.id !== task.id);
      Message.success('删除成功');
    }
  });
};

const handleToggleStatus = (task) => {
  task.status = task.status === 'enabled' ? 'paused' : 'enabled';
  Message.success(task.status === 'enabled' ? '任务已启用' : '任务已暂停');
};

const handleRunNow = (task) => {
  Message.success(`任务 "${task.name}" 已添加到执行队列`);
};

// 表单操作
const resetForm = () => {
  formData.name = '';
  formData.description = '';
  formData.walletSelectionType = 'all';
  formData.walletGroupId = null;
  formData.selectedWallets = [];
  formData.scriptId = null;
  formData.browserProfileId = null;
  formData.scheduleType = 'once';
  formData.intervalSeconds = 3600;
  formData.maxExecutions = 1;
  formData.unlimitedExecutions = false;
  formData.cronExpression = '0 0 * * *';
  formData.timeoutSeconds = 300;
  formData.retryTimes = 3;
  formData.retryIntervalSeconds = 60;
  formData.concurrency = 1;
  formData.notifyOnSuccess = false;
  formData.notifyOnFailure = true;
  formData.enableProxy = true;
  formData.enableFingerprintProtection = true;
  formData.injectExtensions = ['metamask', 'okxwallet'];
  activeTab.value = 'basic';
};

const handleSubmit = () => {
  if (!formData.name) {
    Message.error('请输入任务名称');
    return;
  }
  if (!formData.scriptId) {
    Message.error('请选择执行脚本');
    return;
  }
  if (formData.walletSelectionType === 'group' && !formData.walletGroupId) {
    Message.error('请选择钱包分组');
    return;
  }
  if (formData.walletSelectionType === 'specific' && formData.selectedWallets.length === 0) {
    Message.error('请选择要执行的钱包');
    return;
  }

  const taskData = {
    id: isEditing.value ? currentTaskId.value : Date.now(),
    name: formData.name,
    description: formData.description,
    scriptId: formData.scriptId,
    walletSelectionType: formData.walletSelectionType,
    walletGroupId: formData.walletGroupId,
    selectedWallets: [...formData.selectedWallets],
    browserProfileId: formData.browserProfileId,
    scheduleType: formData.scheduleType,
    intervalSeconds: formData.intervalSeconds,
    maxExecutions: formData.maxExecutions,
    unlimitedExecutions: formData.unlimitedExecutions,
    cronExpression: formData.cronExpression,
    timeoutSeconds: formData.timeoutSeconds,
    retryTimes: formData.retryTimes,
    retryIntervalSeconds: formData.retryIntervalSeconds,
    concurrency: formData.concurrency,
    notifyOnSuccess: formData.notifyOnSuccess,
    notifyOnFailure: formData.notifyOnFailure,
    enableProxy: formData.enableProxy,
    enableFingerprintProtection: formData.enableFingerprintProtection,
    injectExtensions: [...formData.injectExtensions]
  };

  if (isEditing.value) {
    const index = tasks.value.findIndex(t => t.id === currentTaskId.value);
    if (index !== -1) {
      tasks.value[index] = { ...tasks.value[index], ...taskData };
    }
    Message.success('任务已更新');
  } else {
    tasks.value.push({
      ...taskData,
      status: 'draft',
      nextRunTime: null,
      totalRuns: 0,
      successRuns: 0,
      failedRuns: 0,
      scriptName: scripts.value.find(s => s.id === formData.scriptId)?.name || '',
      walletCount: walletCount.value
    });
    Message.success('任务已创建');
  }

  isModalVisible.value = false;
  resetForm();
};

const handleCancel = () => {
  isModalVisible.value = false;
  resetForm();
};

// 获取脚本名称
const getScriptName = (scriptId) => {
  return scripts.value.find(s => s.id === scriptId)?.name || '-';
};

// 格式化地址
const formatAddress = (address) => {
  if (!address) return '-';
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
};
</script>

<template>
  <div class="task-manager">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="left-tools">
        <a-button type="primary" @click="handleCreateTask">
          <template #icon><icon-plus /></template>
          新建任务
        </a-button>
      </div>
      <div class="right-tools">
        <a-input-search placeholder="搜索任务..." style="width: 250px" allow-clear />
      </div>
    </div>

    <!-- Task List -->
    <div class="task-list">
      <a-table :data="tasks" :pagination="{ pageSize: 10 }" :bordered="false" hoverable>
        <template #columns>
          <a-table-column title="任务名称" data-index="name" :width="180">
            <template #cell="{ record }">
              <div class="task-name">{{ record.name }}</div>
              <div class="task-desc">{{ record.description || '暂无描述' }}</div>
            </template>
          </a-table-column>
          <a-table-column title="脚本" data-index="scriptName" :width="140" />
          <a-table-column title="钱包数量" data-index="walletCount" :width="100" align="center" />
          <a-table-column title="状态" data-index="status" :width="100" align="center">
            <template #cell="{ record }">
              <a-tag :color="record.status === 'enabled' ? 'green' : record.status === 'paused' ? 'orange' : 'gray'">
                {{ record.status === 'enabled' ? '已启用' : record.status === 'paused' ? '已暂停' : '草稿' }}
              </a-tag>
            </template>
          </a-table-column>
          <a-table-column title="下次执行" data-index="nextRunTime" :width="160">
            <template #cell="{ record }">
              {{ record.nextRunTime || '-' }}
            </template>
          </a-table-column>
          <a-table-column title="执行统计" :width="150">
            <template #cell="{ record }">
              <div class="stats">
                <span class="success">{{ record.successRuns }} 成功</span>
                <span class="failed">{{ record.failedRuns }} 失败</span>
              </div>
            </template>
          </a-table-column>
          <a-table-column title="操作" :width="200" align="center">
            <template #cell="{ record }">
              <a-space>
                <a-button type="text" size="mini" @click="handleRunNow(record)" :disabled="record.status !== 'enabled'">
                  <template #icon><icon-play-arrow /></template>
                </a-button>
                <a-button type="text" size="mini" @click="handleToggleStatus(record)">
                  <template #icon><icon-pause v-if="record.status === 'enabled'" /><icon-play-arrow v-else /></template>
                </a-button>
                <a-button type="text" size="mini" @click="handleEditTask(record)">
                  <template #icon><icon-edit /></template>
                </a-button>
                <a-button type="text" size="mini" status="danger" @click="handleDeleteTask(record)">
                  <template #icon><icon-delete /></template>
                </a-button>
              </a-space>
            </template>
          </a-table-column>
        </template>
      </a-table>
    </div>

    <!-- Create/Edit Task Modal -->
    <a-modal
      v-model:visible="isModalVisible"
      :title="isEditing ? '编辑任务' : '新建任务'"
      :footer="false"
      width="900px"
      unmount-on-close
    >
      <div class="task-form">
        <!-- Tabs -->
        <a-tabs v-model:active-key="activeTab" type="card-gutter">
          <a-tab-pane key="basic" title="基本信息">
            <a-form :model="formData" layout="vertical">
              <a-form-item label="任务名称" required>
                <a-input v-model="formData.name" placeholder="输入任务名称" />
              </a-form-item>
              <a-form-item label="任务描述">
                <a-textarea v-model="formData.description" placeholder="输入任务描述" :max-length="200" show-word-limit />
              </a-form-item>
            </a-form>
          </a-tab-pane>

          <a-tab-pane key="wallets" title="钱包配置">
            <div class="wallet-config">
              <div class="config-section">
                <div class="section-title">选择方式</div>
                <a-radio-group v-model="formData.walletSelectionType">
                  <a-radio value="all">全部钱包</a-radio>
                  <a-radio value="group">指定分组</a-radio>
                  <a-radio value="specific">指定钱包</a-radio>
                </a-radio-group>
              </div>

              <!-- 分组选择 -->
              <div class="config-section" v-if="formData.walletSelectionType === 'group'">
                <div class="section-title">选择分组</div>
                <a-select v-model="formData.walletGroupId" placeholder="选择钱包分组" style="width: 300px">
                  <a-option v-for="group in walletGroups" :key="group.id" :value="group.id">
                    {{ group.name }} ({{ group.walletCount }} 个钱包)
                  </a-option>
                </a-select>
              </div>

              <!-- 指定钱包选择 -->
              <div class="config-section" v-if="formData.walletSelectionType === 'specific'">
                <div class="section-title">
                  <span>选择钱包 (已选择 {{ formData.selectedWallets.length }} 个)</span>
                  <a-space>
                    <a-button size="mini" @click="selectAllWallets">全选</a-button>
                    <a-button size="mini" @click="deselectAllWallets">清空</a-button>
                  </a-space>
                </div>
                <a-input-search v-model="walletSearch" placeholder="搜索钱包地址或备注" style="width: 300px; margin-bottom: 10px;" allow-clear />
                <div class="wallet-selection-list">
                  <div
                    v-for="wallet in filteredWallets"
                    :key="wallet.id"
                    class="wallet-item"
                    :class="{ selected: isWalletSelected(wallet.id) }"
                    @click="toggleWalletSelection(wallet.id)"
                  >
                    <div class="wallet-check">
                      <icon-check v-if="isWalletSelected(wallet.id)" />
                    </div>
                    <div class="wallet-info">
                      <div class="wallet-address">{{ formatAddress(wallet.address) }}</div>
                      <div class="wallet-label">{{ wallet.label }} - {{ wallet.group }}</div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 统计信息 -->
              <div class="wallet-stats">
                <icon-user />
                <span>将执行 <strong>{{ walletCount }}</strong> 个钱包</span>
              </div>
            </div>
          </a-tab-pane>

          <a-tab-pane key="script" title="脚本选择">
            <div class="script-config">
              <div class="section-title">选择执行脚本</div>
              <a-radio-group v-model="formData.scriptId" direction="vertical">
                <div class="script-grid">
                  <div
                    v-for="script in scripts"
                    :key="script.id"
                    class="script-card"
                    :class="{ selected: formData.scriptId === script.id }"
                    @click="formData.scriptId = script.id"
                  >
                    <div class="script-radio">
                      <a-radio :value="script.id" />
                    </div>
                    <div class="script-info">
                      <div class="script-name">{{ script.name }}</div>
                      <div class="script-desc">{{ script.description }}</div>
                      <div class="script-version">v{{ script.version }}</div>
                    </div>
                  </div>
                </div>
              </a-radio-group>
            </div>
          </a-tab-pane>

          <a-tab-pane key="schedule" title="执行计划">
            <div class="schedule-config">
              <a-form :model="formData" layout="vertical">
                <a-form-item label="执行方式">
                  <a-radio-group v-model="formData.scheduleType">
                    <a-radio value="once">立即执行</a-radio>
                    <a-radio value="interval">循环执行</a-radio>
                    <a-radio value="cron">Cron表达式</a-radio>
                  </a-radio-group>
                </a-form-item>

                <!-- 循环执行配置 -->
                <div v-if="formData.scheduleType === 'interval'" class="sub-config">
                  <a-form-item label="执行间隔">
                    <a-space>
                      <a-input-number v-model="formData.intervalSeconds" :min="60" :step="60" />
                      <span>秒</span>
                    </a-space>
                  </a-form-item>
                  <a-form-item label="最大执行次数">
                    <a-space>
                      <a-switch v-model="formData.unlimitedExecutions" />
                      <span>无限循环</span>
                    </a-space>
                  </a-form-item>
                  <a-form-item v-if="!formData.unlimitedExecutions" label="执行次数">
                    <a-input-number v-model="formData.maxExecutions" :min="1" :max="1000" />
                  </a-form-item>
                </div>

                <!-- Cron表达式配置 -->
                <div v-if="formData.scheduleType === 'cron'" class="sub-config">
                  <a-form-item label="Cron表达式">
                    <a-input v-model="formData.cronExpression" style="width: 200px;" />
                  </a-form-item>
                  <div class="cron-examples">
                    <span>示例:</span>
                    <a-tag @click="formData.cronExpression = '0 0 * * *'">每天凌晨</a-tag>
                    <a-tag @click="formData.cronExpression = '0 */1 * * *'">每小时</a-tag>
                    <a-tag @click="formData.cronExpression = '0 0 * * 0'">每周日</a-tag>
                    <a-tag @click="formData.cronExpression = '0 0 1 * *'">每月1号</a-tag>
                  </div>
                </div>
              </a-form>
            </div>
          </a-tab-pane>

          <a-tab-pane key="environment" title="环境配置">
            <div class="environment-config">
              <a-form :model="formData" layout="vertical">
                <a-form-item label="浏览器配置">
                  <a-select v-model="formData.browserProfileId" placeholder="选择浏览器配置" style="width: 300px">
                    <a-option v-for="profile in browserProfiles" :key="profile.id" :value="profile.id">
                      {{ profile.name }}
                    </a-option>
                  </a-select>
                </a-form-item>

                <a-divider orientation="left">功能选项</a-divider>

                <a-space direction="vertical" size="large">
                  <a-space>
                    <a-switch v-model="formData.enableProxy" />
                    <span>使用代理</span>
                  </a-space>
                  <a-space>
                    <a-switch v-model="formData.enableFingerprintProtection" />
                    <span>指纹保护</span>
                  </a-space>
                </a-space>

                <a-divider orientation="left">注入扩展</a-divider>

                <a-space>
                  <a-checkbox v-model="formData.injectExtensions" value="metamask">MetaMask</a-checkbox>
                  <a-checkbox v-model="formData.injectExtensions" value="okxwallet">OKX Wallet</a-checkbox>
                </a-space>
              </a-form>
            </div>
          </a-tab-pane>

          <a-tab-pane key="advanced" title="高级选项">
            <div class="advanced-config">
              <a-form :model="formData" layout="vertical">
                <a-row :gutter="16">
                  <a-col :span="8">
                    <a-form-item label="超时时间 (秒)">
                      <a-input-number v-model="formData.timeoutSeconds" :min="30" :max="3600" />
                    </a-form-item>
                  </a-col>
                  <a-col :span="8">
                    <a-form-item label="重试次数">
                      <a-input-number v-model="formData.retryTimes" :min="0" :max="10" />
                    </a-form-item>
                  </a-col>
                  <a-col :span="8">
                    <a-form-item label="重试间隔 (秒)">
                      <a-input-number v-model="formData.retryIntervalSeconds" :min="10" :max="600" />
                    </a-form-item>
                  </a-col>
                </a-row>

                <a-form-item label="并发数">
                  <a-input-number v-model="formData.concurrency" :min="1" :max="10" />
                  <span class="form-hint">同时执行的钱包数量</span>
                </a-form-item>

                <a-divider orientation="left">通知设置</a-divider>

                <a-space>
                  <a-checkbox v-model="formData.notifyOnSuccess">执行成功时通知</a-checkbox>
                  <a-checkbox v-model="formData.notifyOnFailure">执行失败时通知</a-checkbox>
                </a-space>
              </a-form>
            </div>
          </a-tab-pane>
        </a-tabs>

        <!-- Form Actions -->
        <div class="form-actions">
          <a-button @click="handleCancel">取消</a-button>
          <a-space>
            <a-button @click="handleSubmit" type="outline">保存为草稿</a-button>
            <a-button type="primary" @click="handleSubmit">保存并启用</a-button>
          </a-space>
        </div>
      </div>
    </a-modal>
  </div>
</template>

<style scoped>
.task-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  background: var(--color-bg-2);
  border-radius: 8px;
  border: 1px solid var(--color-border);
}

.task-list {
  flex: 1;
  background: var(--color-bg-2);
  border-radius: 8px;
  padding: 10px;
  overflow: hidden;
  border: 1px solid var(--color-border);
}

.task-name {
  font-weight: 500;
  color: var(--color-text-1);
}

.task-desc {
  font-size: 12px;
  color: var(--color-text-3);
}

.stats {
  font-size: 12px;
}

.stats .success {
  color: rgb(var(--success-6));
  margin-right: 8px;
}

.stats .failed {
  color: rgb(var(--danger-6));
}

/* Modal Styles */
.task-form {
  padding: 10px 0;
}

.config-section {
  margin-bottom: 20px;
}

.section-title {
  font-weight: 500;
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.wallet-selection-list {
  max-height: 250px;
  overflow-y: auto;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 8px;
}

.wallet-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.wallet-item:hover {
  background: var(--color-fill-2);
}

.wallet-item.selected {
  background: rgba(var(--primary-6), 0.1);
}

.wallet-check {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  border: 2px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}

.wallet-item.selected .wallet-check {
  background: rgb(var(--primary-6));
  border-color: rgb(var(--primary-6));
}

.wallet-address {
  font-family: monospace;
  color: rgb(var(--primary-6));
}

.wallet-label {
  font-size: 12px;
  color: var(--color-text-3);
}

.wallet-stats {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: var(--color-fill-2);
  border-radius: 6px;
  margin-top: 15px;
}

.wallet-stats strong {
  color: rgb(var(--primary-6));
  font-size: 16px;
}

.script-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-top: 10px;
}

.script-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 15px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.script-card:hover {
  background: var(--color-fill-2);
}

.script-card.selected {
  border-color: rgb(var(--primary-6));
  background: rgba(var(--primary-6), 0.05);
}

.script-name {
  font-weight: 500;
  color: var(--color-text-1);
}

.script-desc {
  font-size: 12px;
  color: var(--color-text-3);
  margin: 4px 0;
}

.script-version {
  font-size: 11px;
  color: var(--color-text-4);
}

.cron-examples {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}

.sub-config {
  padding: 15px;
  background: var(--color-fill-2);
  border-radius: 6px;
  margin-top: 10px;
}

.form-hint {
  margin-left: 10px;
  font-size: 12px;
  color: var(--color-text-3);
}

.form-actions {
  display: flex;
  justify-content: space-between;
  margin-top: 20px;
  padding-top: 15px;
  border-top: 1px solid var(--color-border);
}

:deep(.arco-table-cell) {
  background: transparent !important;
}
</style>
