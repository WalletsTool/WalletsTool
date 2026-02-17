<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Message } from '@arco-design/web-vue';
import {
  IconRecord,
  IconStop,
  IconPlayArrow,
  IconCode,
  IconDelete,
  IconDownload,
  IconPlus,
  IconRefresh,
  IconApps
} from '@arco-design/web-vue/es/icon';
import { recorderService } from '../services/recorderService';
import { extensionService } from '../services/extensionService';

const emit = defineEmits(['insert-code', 'close']);

const recordUrl = ref('');
const isRecording = ref(false);
const isBrowserOpen = ref(false);
const generatedCode = ref('');
const browserType = ref('chromium');
const headless = ref(false);

const availableExtensions = ref([]);
const selectedExtensions = ref([]);
const loadingExtensions = ref(false);

const loadExtensions = async () => {
  loadingExtensions.value = true;
  try {
    availableExtensions.value = await extensionService.getExtensions();
  } catch (error) {
    console.error('Failed to load extensions:', error);
  } finally {
    loadingExtensions.value = false;
  }
};

onMounted(() => {
  loadExtensions();
});

const currentSession = computed(() => recorderService.getCurrentSession());
const actionCount = computed(() => currentSession.value?.actions.length || 0);
const recordedActions = computed(() => currentSession.value?.actions || []);

const actionTypeMap = {
  click: '点击',
  fill: '输入',
  navigate: '导航',
  select: '选择',
  hover: '悬停',
  screenshot: '截图',
  upload: '上传文件',
  evaluate: '执行脚本'
};

const getActionIcon = (type) => {
  const icons = {
    click: '👆',
    fill: '⌨️',
    navigate: '🔗',
    select: '📋',
    hover: '🖱️',
    screenshot: '📷',
    upload: '📎',
    evaluate: '⚙️'
  };
  return icons[type] || '📝';
};

const startRecording = async () => {
  if (!recordUrl.value.trim()) {
    Message.warning('请输入要录制的网址');
    return;
  }

  try {
    isRecording.value = true;
    generatedCode.value = '';
    
    const extensionPaths = selectedExtensions.value.map(id => {
      const ext = availableExtensions.value.find(e => e.id === id);
      return ext?.path;
    }).filter(Boolean);
    
    await recorderService.startRecording(recordUrl.value, {
      browserType: browserType.value,
      headless: headless.value,
      viewportWidth: 1280,
      viewportHeight: 720,
      includeComments: true,
      extensions: extensionPaths
    });
    
    isBrowserOpen.value = true;
    Message.success('浏览器已启动，请开始操作');
  } catch (error) {
    Message.error('启动浏览器失败: ' + (error.message || error));
    isRecording.value = false;
    isBrowserOpen.value = false;
  }
};

const stopRecording = async () => {
  try {
    const code = await recorderService.stopRecording();
    
    if (code) {
      generatedCode.value = code;
    }
    
    isBrowserOpen.value = false;
    isRecording.value = false;
    Message.success('录制已停止');
  } catch (error) {
    Message.error('停止录制失败: ' + (error.message || error));
    isRecording.value = false;
    isBrowserOpen.value = false;
  }
};

const clearActions = () => {
  recorderService.clearRecording();
  generatedCode.value = '';
  Message.success('已清空录制记录');
};

const insertCode = () => {
  if (!generatedCode.value) {
    Message.warning('没有可插入的代码，请先录制操作');
    return;
  }
  emit('insert-code', generatedCode.value);
  Message.success('代码已插入到编辑器');
};

const copyCode = async () => {
  if (!generatedCode.value) {
    Message.warning('没有可复制的代码');
    return;
  }
  try {
    await navigator.clipboard.writeText(generatedCode.value);
    Message.success('代码已复制到剪贴板');
  } catch {
    Message.error('复制失败');
  }
};

const takeScreenshot = async () => {
  if (!isBrowserOpen.value) {
    Message.warning('请先启动浏览器');
    return;
  }
  try {
    const mcpPlaywright = window.__MCP_PLAYWRIGHT__;
    if (typeof window !== 'undefined' && mcpPlaywright) {
      await mcpPlaywright.playwright_screenshot({
        name: `recording-${Date.now()}`,
        fullPage: false
      });
    }
    recorderService.addAction('screenshot', '截图', {});
    Message.success('截图已保存');
  } catch (error) {
    Message.error('截图失败: ' + (error.message || error));
  }
};

const refreshPage = async () => {
  if (!isBrowserOpen.value) {
    Message.warning('请先启动浏览器');
    return;
  }
  try {
    const mcpPlaywright = window.__MCP_PLAYWRIGHT__;
    if (typeof window !== 'undefined' && mcpPlaywright) {
      await mcpPlaywright.playwright_navigate({
        url: recordUrl.value,
        browserType: browserType.value
      });
    }
    recorderService.addAction('navigate', '刷新页面', { value: recordUrl.value });
    Message.success('页面已刷新');
  } catch (error) {
    Message.error('刷新失败: ' + (error.message || error));
  }
};

onUnmounted(() => {
  if (isBrowserOpen.value) {
    recorderService.stopRecording();
  }
});
</script>

<template>
  <div class="script-recorder">
    <div class="recorder-header">
      <h3>
        <icon-record :style="{ color: isRecording ? 'rgb(var(--danger-6))' : 'inherit' }" />
        脚本录制
      </h3>
      <a-button type="text" size="small" @click="emit('close')">
        <template #icon><icon-delete /></template>
      </a-button>
    </div>

    <div class="recorder-config">
      <div class="config-row">
        <label>目标网址:</label>
        <a-input 
          v-model="recordUrl" 
          placeholder="https://example.com"
          :disabled="isRecording"
          @press-enter="startRecording"
        >
          <template #prefix>🔗</template>
        </a-input>
      </div>
      
      <div class="config-row inline">
        <label>浏览器:</label>
        <a-select v-model="browserType" :disabled="isRecording" style="width: 120px">
          <a-option value="chromium">Chromium</a-option>
          <a-option value="firefox">Firefox</a-option>
          <a-option value="webkit">WebKit</a-option>
        </a-select>
        
        <a-checkbox v-model="headless" :disabled="isRecording">无头模式</a-checkbox>
      </div>
      
      <div class="config-row" v-if="availableExtensions.length > 0">
        <label>
          <icon-apps style="margin-right: 4px;" />
          加载插件:
        </label>
        <a-select 
          v-model="selectedExtensions" 
          :disabled="isRecording"
          multiple
          placeholder="选择要加载的浏览器插件"
          style="width: 100%"
        >
          <a-option 
            v-for="ext in availableExtensions" 
            :key="ext.id" 
            :value="ext.id"
            :disabled="!ext.enabled"
          >
            <span>{{ ext.name }}</span>
            <span v-if="ext.version" style="color: var(--color-text-3); margin-left: 8px;">
              v{{ ext.version }}
            </span>
            <a-tag v-if="!ext.enabled" size="small" color="gray" style="margin-left: 8px;">未启用</a-tag>
          </a-option>
        </a-select>
      </div>
    </div>

    <div class="recorder-controls">
      <a-space>
        <a-button 
          v-if="!isRecording"
          type="primary"
          @click="startRecording"
        >
          <template #icon><icon-play-arrow /></template>
          开始录制
        </a-button>
        
        <a-button 
          v-else
          type="primary"
          status="danger"
          @click="stopRecording"
        >
          <template #icon><icon-stop /></template>
          停止录制
        </a-button>
        
        <a-button 
          v-if="isBrowserOpen"
          @click="refreshPage"
        >
          <template #icon><icon-refresh /></template>
          刷新
        </a-button>
        
        <a-button 
          v-if="isBrowserOpen"
          @click="takeScreenshot"
        >
          <template #icon>📷</template>
          截图
        </a-button>
      </a-space>
    </div>

    <div class="recorder-status" v-if="isRecording">
      <a-badge status="processing" text="正在录制中..." />
      <span class="action-count">已记录 {{ actionCount }} 个操作</span>
    </div>

    <div class="recorder-actions" v-if="recordedActions.length > 0">
      <div class="actions-header">
        <span>录制操作 ({{ actionCount }})</span>
        <a-button type="text" size="small" @click="clearActions">
          <template #icon><icon-delete /></template>
          清空
        </a-button>
      </div>
      
      <div class="actions-list">
        <div 
          v-for="(action, index) in recordedActions" 
          :key="index"
          class="action-item"
        >
          <span class="action-icon">{{ getActionIcon(action.type) }}</span>
          <span class="action-desc">{{ action.description }}</span>
          <span class="action-time">{{ new Date(action.timestamp).toLocaleTimeString() }}</span>
        </div>
      </div>
    </div>

    <div class="generated-code" v-if="generatedCode">
      <div class="code-header">
        <span><icon-code /> 生成的代码</span>
        <a-space>
          <a-button type="text" size="small" @click="copyCode">
            复制
          </a-button>
          <a-button type="primary" size="small" @click="insertCode">
            <template #icon><icon-plus /></template>
            插入到编辑器
          </a-button>
        </a-space>
      </div>
      
      <div class="code-preview">
        <pre><code>{{ generatedCode }}</code></pre>
      </div>
    </div>

    <div class="recorder-tips" v-if="!isRecording && recordedActions.length === 0">
      <div class="tip-item">
        <span class="tip-icon">💡</span>
        <span>输入目标网址，点击"开始录制"启动浏览器</span>
      </div>
      <div class="tip-item">
        <span class="tip-icon">🖱️</span>
        <span>在浏览器中的操作将被自动记录</span>
      </div>
      <div class="tip-item">
        <span class="tip-icon">📝</span>
        <span>停止录制后可生成 Playwright 脚本代码</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.script-recorder {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-2);
  border-radius: 8px;
  overflow: hidden;
}

.recorder-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-3);
}

.recorder-header h3 {
  margin: 0;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--color-text-1);
}

.recorder-config {
  padding: 16px;
  border-bottom: 1px solid var(--color-border);
}

.config-row {
  margin-bottom: 12px;
}

.config-row:last-child {
  margin-bottom: 0;
}

.config-row label {
  display: block;
  font-size: 12px;
  color: var(--color-text-3);
  margin-bottom: 6px;
}

.config-row.inline {
  display: flex;
  align-items: center;
  gap: 16px;
}

.config-row.inline label {
  margin-bottom: 0;
}

.recorder-controls {
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-1);
}

.recorder-status {
  padding: 12px 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  background: rgba(var(--danger-1), 0.3);
  border-bottom: 1px solid var(--color-border);
}

.action-count {
  font-size: 12px;
  color: var(--color-text-2);
}

.recorder-actions {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 150px;
  max-height: 250px;
  border-bottom: 1px solid var(--color-border);
}

.actions-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: var(--color-bg-3);
  font-size: 12px;
  color: var(--color-text-2);
}

.actions-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.action-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 6px;
  background: var(--color-bg-1);
  margin-bottom: 6px;
  font-size: 13px;
}

.action-item:last-child {
  margin-bottom: 0;
}

.action-icon {
  font-size: 14px;
}

.action-desc {
  flex: 1;
  color: var(--color-text-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.action-time {
  font-size: 11px;
  color: var(--color-text-4);
}

.generated-code {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 200px;
}

.code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 16px;
  background: var(--color-bg-3);
  font-size: 12px;
  color: var(--color-text-2);
}

.code-preview {
  flex: 1;
  overflow: auto;
  background: var(--color-bg-1);
  padding: 12px;
}

.code-preview pre {
  margin: 0;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-text-1);
  white-space: pre-wrap;
  word-break: break-all;
}

.recorder-tips {
  padding: 20px;
  background: var(--color-bg-1);
}

.tip-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
  font-size: 13px;
  color: var(--color-text-3);
}

.tip-icon {
  font-size: 16px;
}
</style>
