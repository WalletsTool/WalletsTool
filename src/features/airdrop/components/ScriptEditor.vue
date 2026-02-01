<script setup>
import { ref, onMounted, nextTick, watch } from 'vue';
import { Message, Modal } from '@arco-design/web-vue';
import {
  IconPlus,
  IconFile,
  IconPlayArrow,
  IconSave,
  IconCode,
  IconDelete,
  IconBook,
  IconFullscreen,
  IconFullscreenExit,
  IconImport,
  IconDownload,
  IconCopy,
  IconCheck
} from '@arco-design/web-vue/es/icon';
import ApiHelper from './ApiHelper.vue';
import { scriptService, executionService } from '../services/browserAutomationService';

const scripts = ref([]);

const activeScript = ref(null);
const scriptContent = ref('');
const isNewModalVisible = ref(false);
const newScriptName = ref('');
const showApiHelper = ref(true);
const isFullscreen = ref(false);
const copiedCode = ref(false);
const loading = ref(false);

const editingScriptId = ref(null);
const editNameInput = ref(null);
const editNameValue = ref('');

// 加载脚本
const loadScripts = async () => {
  loading.value = true;
  try {
    scripts.value = await scriptService.getScripts();
  } catch (error) {
    Message.error('加载脚本失败: ' + error.message);
  } finally {
    loading.value = false;
  }
};

const startEditName = async (script, event) => {
  event?.stopPropagation();
  editingScriptId.value = script.id;
  editNameValue.value = script.name;
  await nextTick();
  if (editNameInput.value) {
    editNameInput.value.focus();
    editNameInput.value.select();
  }
};

const saveEditName = async () => {
  const trimmedName = editNameValue.value.trim();
  if (!trimmedName) {
    editingScriptId.value = null;
    return;
  }
  const script = scripts.value.find(s => s.id === editingScriptId.value);
  if (script) {
    try {
      await scriptService.updateScript(script.id, { name: trimmedName });
      script.name = trimmedName;
      Message.success('名称已更新');
    } catch (error) {
      Message.error('更新失败: ' + error.message);
    }
  }
  editingScriptId.value = null;
};

const cancelEditName = () => {
  editingScriptId.value = null;
};

const handleNameKeydown = (event) => {
  if (event.key === 'Enter') {
    event.preventDefault();
    saveEditName();
  } else if (event.key === 'Escape') {
    event.preventDefault();
    cancelEditName();
  }
};

const handleSelectScript = (script) => {
  activeScript.value = script;
  scriptContent.value = script.content;
};

const handleSave = async () => {
  if (!activeScript.value) {
    Message.warning('请先选择或创建一个脚本');
    return;
  }
  
  try {
    await scriptService.updateScript(activeScript.value.id, {
      content: scriptContent.value
    });
    activeScript.value.content = scriptContent.value;
    Message.success('脚本已保存');
  } catch (error) {
    Message.error('保存失败: ' + error.message);
  }
};

const handleRun = async () => {
  if (!activeScript.value) return;
  
  try {
    // 创建临时执行任务
    const execution = await executionService.createExecution({
      script_id: activeScript.value.id,
      wallet_ids: [],
      profile_ids: [],
      parallel_mode: false
    });
    
    // 启动执行
    await executionService.startExecution(execution.id);
    
    Message.success('脚本已添加到执行队列，请切换到"执行面板"查看');
  } catch (error) {
    Message.error('启动执行失败: ' + error.message);
  }
};

const handleNewScript = () => {
  isNewModalVisible.value = true;
  newScriptName.value = '';
};

const confirmNewScript = async () => {
  if (!newScriptName.value.trim()) {
    Message.error('请输入脚本名称');
    return;
  }

  const defaultContent = `// ${newScriptName.value.trim()}
// 依赖API: waitForSelector, clickElement, randomDelay, log

async function run({ page, wallet, api }) {
    api.log('info', '开始执行脚本');
    
    // 在此编写你的脚本逻辑
    // 示例:
    // await page.goto('https://example.com');
    // await api.waitForSelector('.button');
    // await api.clickElement('.button');
    
    api.log('success', '脚本执行完成');
    return { success: true };
}`;

  try {
    const newScript = await scriptService.createScript({
      name: newScriptName.value.trim(),
      content: defaultContent,
      description: ''
    });
    
    scripts.value.push(newScript);
    handleSelectScript(newScript);
    isNewModalVisible.value = false;
    Message.success('创建成功');
  } catch (error) {
    Message.error('创建失败: ' + error.message);
  }
};

const handleDeleteScript = async (e, scriptId) => {
  e.stopPropagation();
  const script = scripts.value.find(s => s.id === scriptId);
  Modal.warning({
    title: '确认删除',
    content: `确定要删除脚本 "${script?.name || ''}" 吗？此操作不可恢复。`,
    onOk: async () => {
      try {
        await scriptService.deleteScript(scriptId);
        scripts.value = scripts.value.filter(s => s.id !== scriptId);
        if (activeScript.value && activeScript.value.id === scriptId) {
          activeScript.value = null;
          scriptContent.value = '';
        }
        Message.success('删除成功');
      } catch (error) {
        Message.error('删除失败: ' + error.message);
      }
    }
  });
};

// 导入脚本
const handleImportScript = async () => {
  try {
    const imported = await scriptService.importScript();
    if (imported) {
      // 保存到数据库
      const newScript = await scriptService.createScript({
        name: imported.name,
        content: imported.content,
        description: imported.description || ''
      });
      scripts.value.push(newScript);
      handleSelectScript(newScript);
      Message.success('导入成功');
    }
  } catch (error) {
    Message.error('导入失败: ' + error.message);
  }
};

// 导出脚本
const handleExportScript = async () => {
  if (!activeScript.value) {
    Message.warning('请先选择要导出的脚本');
    return;
  }
  try {
    await scriptService.exportScript(activeScript.value);
    Message.success('导出成功');
  } catch (error) {
    Message.error('导出失败: ' + error.message);
  }
};

// 复制脚本内容
const handleCopyScript = async () => {
  if (!activeScript.value) return;
  try {
    await navigator.clipboard.writeText(scriptContent.value);
    copiedCode.value = true;
    Message.success('已复制到剪贴板');
    setTimeout(() => {
      copiedCode.value = false;
    }, 2000);
  } catch (e) {
    Message.error('复制失败');
  }
};

const handleInsertCode = (code) => {
  if (activeScript.value) {
    scriptContent.value += '\n' + code;
    Message.success('代码已插入');
  }
};

const toggleApiHelper = () => {
  showApiHelper.value = !showApiHelper.value;
};

const toggleFullscreen = () => {
  isFullscreen.value = !isFullscreen.value;
};

// 键盘快捷键
const handleKeydown = (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault();
    handleSave();
  }
};

onMounted(() => {
  loadScripts();
  window.addEventListener('keydown', handleKeydown);
});
</script>

<template>
  <div class="script-editor" :class="{ fullscreen: isFullscreen }">
    <div class="script-list" v-if="!isFullscreen">
      <div class="list-header">
        <h3>脚本列表</h3>
        <a-space>
          <a-button type="text" size="small" @click="handleImportScript" title="导入脚本">
            <template #icon><icon-import /></template>
          </a-button>
          <a-button type="primary" size="small" @click="handleNewScript">
            <template #icon><icon-plus /></template>
          </a-button>
        </a-space>
      </div>

      <div class="list-content" v-loading="loading">
        <div
          v-for="script in scripts"
          :key="script.id"
          class="script-item"
          :class="{ active: activeScript && activeScript.id === script.id }"
          @click="handleSelectScript(script)"
        >
          <div class="item-main">
            <icon-code />
            <template v-if="editingScriptId === script.id">
              <input
                ref="editNameInput"
                v-model="editNameValue"
                class="name-edit-input"
                @blur="saveEditName"
                @keydown="handleNameKeydown"
                @click.stop
              />
            </template>
            <template v-else>
              <span class="script-name editable" @click="(e) => startEditName(script, e)" title="点击编辑名称">
                {{ script.name }}
              </span>
            </template>
          </div>
          <div class="item-actions">
            <icon-delete class="delete-icon" @click="(e) => handleDeleteScript(e, script.id)" />
          </div>
        </div>
        <div v-if="scripts.length === 0" class="empty-scripts">
          暂无脚本，点击 + 创建新脚本
        </div>
      </div>
    </div>

    <div class="editor-main" v-if="activeScript" :style="{ width: isFullscreen ? '100%' : (showApiHelper ? 'calc(50% - 10px)' : 'calc(100% - 20px)') }">
      <div class="editor-area">
        <div class="editor-toolbar">
          <div class="file-info">
            <icon-file />
            <span>{{ activeScript.name }}</span>
          </div>
          <div class="actions">
            <a-tooltip content="API 参考文档">
              <a-button type="text" size="small" @click="toggleApiHelper" :status="showApiHelper ? 'primary' : 'normal'" v-if="!isFullscreen">
                <template #icon><icon-book /></template>
              </a-button>
            </a-tooltip>
            <a-tooltip content="全屏编辑">
              <a-button type="text" size="small" @click="toggleFullscreen">
                <template #icon><icon-fullscreen v-if="!isFullscreen" /><icon-fullscreen-exit v-else /></template>
              </a-button>
            </a-tooltip>
            <a-tooltip content="导出脚本">
              <a-button type="text" size="small" @click="handleExportScript">
                <template #icon><icon-download /></template>
              </a-button>
            </a-tooltip>
            <a-tooltip content="复制代码">
              <a-button type="text" size="small" @click="handleCopyScript">
                <template #icon><icon-check v-if="copiedCode" /><icon-copy v-else /></template>
              </a-button>
            </a-tooltip>
            <a-button type="secondary" size="small" @click="handleRun">
              <template #icon><icon-play-arrow /></template>
              测试运行
            </a-button>
            <a-button type="primary" size="small" @click="handleSave">
              <template #icon><icon-save /></template>
              保存
            </a-button>
          </div>
        </div>

        <div class="code-container">
          <textarea
            v-model="scriptContent"
            class="code-input"
            spellcheck="false"
            placeholder="在此编写 Playwright 脚本..."
          ></textarea>
        </div>

        <div class="editor-footer">
          <div class="script-tips">
            <span>提示: 使用 api. 调用自定义方法，如 api.connectMetaMask() | 按 Ctrl+S 保存</span>
          </div>
        </div>
      </div>
    </div>

    <div class="api-helper-panel" v-if="showApiHelper && !isFullscreen && activeScript">
      <ApiHelper @insert-code="handleInsertCode" />
    </div>

    <div class="empty-state" v-if="!activeScript">
      <icon-code style="font-size: 48px; color: var(--color-text-4)" />
      <p>请选择左侧脚本进行编辑，或创建新脚本</p>
      <a-space>
        <a-button type="primary" @click="handleNewScript">创建新脚本</a-button>
        <a-button type="outline" @click="handleImportScript">导入脚本</a-button>
      </a-space>
    </div>

    <!-- New Script Modal -->
    <a-modal v-model:visible="isNewModalVisible" title="新建脚本" @ok="confirmNewScript">
      <a-form-item label="脚本名称">
        <a-input v-model="newScriptName" placeholder="e.g., My Airdrop Task" @press-enter="confirmNewScript" />
      </a-form-item>
    </a-modal>
  </div>
</template>

<style scoped>
.script-editor {
  height: 100%;
  display: flex;
  gap: 20px;
}

.script-editor.fullscreen {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
  background: var(--color-bg-1);
  padding: 20px;
}

.script-list {
  width: 250px;
  background: var(--color-bg-2);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--color-border);
}

.list-header {
  padding: 10px 15px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.list-header h3 {
  margin: 0;
  font-size: 14px;
  color: var(--color-text-2);
}

.list-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.script-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--color-text-2);
  transition: all 0.2s;
}

.item-main {
  display: flex;
  align-items: center;
  gap: 10px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.script-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.script-name.editable {
  cursor: text;
  padding: 2px 4px;
  margin: -2px -4px;
  border-radius: 4px;
  transition: all 0.2s;
}

.script-name.editable:hover {
  background: var(--color-fill-2);
}

.name-edit-input {
  background: var(--color-bg-1);
  border: 1px solid rgb(var(--primary-6));
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 14px;
  color: var(--color-text-1);
  outline: none;
  line-height: 1.2;
  max-width: 150px;
}

.name-edit-input:focus {
  border-color: rgb(var(--primary-6));
  box-shadow: 0 0 0 2px rgba(var(--primary-6), 0.2);
}

.item-actions {
  opacity: 0;
  transition: opacity 0.2s;
}

.script-item:hover {
  background: var(--color-fill-2);
  color: var(--color-text-1);
}

.script-item:hover .item-actions {
  opacity: 1;
}

.delete-icon {
  cursor: pointer;
}

.delete-icon:hover {
  color: rgb(var(--danger-6));
}

.script-item.active {
  background: rgba(var(--primary-6), 0.1);
  color: rgb(var(--primary-6));
}

.empty-scripts {
  text-align: center;
  padding: 30px 20px;
  color: var(--color-text-3);
  font-size: 12px;
}

.editor-main {
  display: flex;
  flex-direction: column;
  transition: width 0.3s ease;
}

.editor-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-2);
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid var(--color-border);
}

.editor-toolbar {
  padding: 8px 15px;
  background: var(--color-bg-3);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--color-text-3);
}

.actions {
  display: flex;
  gap: 8px;
}

.code-container {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.code-input {
  width: 100%;
  height: 100%;
  background: var(--color-bg-1);
  color: var(--color-text-1);
  border: none;
  padding: 15px;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 14px;
  line-height: 1.6;
  resize: none;
  outline: none;
  tab-size: 2;
}

.editor-footer {
  padding: 8px 15px;
  background: var(--color-bg-3);
  border-top: 1px solid var(--color-border);
}

.script-tips {
  font-size: 12px;
  color: var(--color-text-4);
}

.api-helper-panel {
  width: 400px;
  flex-shrink: 0;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid var(--color-border);
}

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
