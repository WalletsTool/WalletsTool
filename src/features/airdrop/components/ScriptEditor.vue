<script setup>
import { ref, onMounted, nextTick } from 'vue';
import { Message, Modal } from '@arco-design/web-vue';
import { 
  IconPlus, 
  IconFile, 
  IconPlayArrow, 
  IconSave, 
  IconCode, 
  IconDelete 
} from '@arco-design/web-vue/es/icon';

const STORAGE_KEY = 'browser_scripts';

const loadScripts = () => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      return JSON.parse(saved);
    }
  } catch (e) {
    console.error('Failed to load scripts:', e);
  }
  return [
    { id: 1, name: 'OKX Daily Claim', content: '// Playwright script for OKX Daily Claim\nasync function run(page, wallet) {\n  await page.goto("https://www.okx.com");\n  // ...\n}' },
    { id: 2, name: 'Uniswap Swap', content: '// Swap ETH for USDC\nasync function run(page, wallet) {\n  await page.goto("https://app.uniswap.org");\n  // ...\n}' },
  ];
};

const saveScripts = () => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(scripts.value));
  } catch (e) {
    console.error('Failed to save scripts:', e);
    Message.error('保存失败');
  }
};

const scripts = ref(loadScripts());

const activeScript = ref(null);
const scriptContent = ref('');
const isNewModalVisible = ref(false);
const newScriptName = ref('');

const editingScriptId = ref(null);
const editNameInput = ref(null);
const editNameValue = ref('');

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

const saveEditName = () => {
  const trimmedName = editNameValue.value.trim();
  if (!trimmedName) {
    editingScriptId.value = null;
    return;
  }
  const script = scripts.value.find(s => s.id === editingScriptId.value);
  if (script) {
    script.name = trimmedName;
    saveScripts();
    Message.success('名称已更新');
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

const handleSave = () => {
  if (activeScript.value) {
    activeScript.value.content = scriptContent.value;
    saveScripts();
    Message.success('脚本已保存');
  } else {
    Message.warning('请先选择或创建一个脚本');
  }
};

const handleRun = () => {
  if (!activeScript.value) return;
  Message.success('脚本已添加到执行队列');
  // TODO: Emit event to start execution
};

const handleNewScript = () => {
  isNewModalVisible.value = true;
  newScriptName.value = '';
};

const confirmNewScript = () => {
  if (!newScriptName.value) {
    Message.error('请输入脚本名称');
    return;
  }
  
  const newScript = {
    id: Date.now(),
    name: newScriptName.value,
    content: '// New Playwright Script\nasync function run(page, wallet) {\n  \n}'
  };
  
  scripts.value.push(newScript);
  saveScripts();
  handleSelectScript(newScript);
  isNewModalVisible.value = false;
  Message.success('创建成功');
};

const handleDeleteScript = (e, scriptId) => {
  e.stopPropagation();
  Modal.warning({
    title: '确认删除',
    content: '确定要删除这个脚本吗？此操作不可恢复。',
    onOk: () => {
      scripts.value = scripts.value.filter(s => s.id !== scriptId);
      saveScripts();
      if (activeScript.value && activeScript.value.id === scriptId) {
        activeScript.value = null;
        scriptContent.value = '';
      }
      Message.success('删除成功');
    }
  });
};
</script>

<template>
  <div class="script-editor">
    <div class="script-list">
      <div class="list-header">
        <h3>脚本列表</h3>
        <a-button type="primary" size="small" @click="handleNewScript">
          <template #icon><icon-plus /></template>
        </a-button>
      </div>
      
      <div class="list-content">
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
      </div>
    </div>

    <div class="editor-area" v-if="activeScript">
      <div class="editor-toolbar">
        <div class="file-info">
          <icon-file />
          <span>{{ activeScript.name }}</span>
        </div>
        <div class="actions">
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
    </div>

    <div class="empty-state" v-else>
      <icon-code style="font-size: 48px; color: var(--color-text-4)" />
      <p>请选择左侧脚本进行编辑，或创建新脚本</p>
      <a-button type="primary" @click="handleNewScript">创建新脚本</a-button>
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

.editor-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-2); /* Changed to theme bg */
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
  gap: 10px;
}

.code-container {
  flex: 1;
  position: relative;
}

.code-input {
  width: 100%;
  height: 100%;
  background: var(--color-bg-1); /* Theme bg */
  color: var(--color-text-1);
  border: none;
  padding: 15px;
  font-family: 'Fira Code', 'Consolas', monospace;
  font-size: 14px;
  line-height: 1.5;
  resize: none;
  outline: none;
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
