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
  IconFullscreenExit
} from '@arco-design/web-vue/es/icon';
import ApiHelper from './ApiHelper.vue';

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
    {
      id: 1,
      name: 'OKX Daily Claim',
      content: `// OKX Daily Claim Script
// 依赖API: connectOKXWallet, clickElement, waitForSelector, randomDelay, log

async function run(page, wallet, api) {
    log('info', '开始执行 OKX Daily Claim');

    // 1. 打开OKX官网
    await page.goto('https://www.okx.com');
    await api.waitForSelector('body');
    await api.randomDelay(2000, 4000);

    // 2. 连接钱包
    log('info', '连接 OKX Wallet...');
    await api.connectOKXWallet({ chainId: '0x1' });
    await api.randomDelay(1000, 2000);

    // 3. 导航到签到页面
    log('info', '导航到 Rewards 页面...');
    await page.goto('https://www.okx.com/rewards');
    await api.waitForSelector('button.claim-button', 10000);
    await api.randomDelay(1000, 2000);

    // 4. 点击签到
    log('info', '执行签到操作...');
    const claimButton = await page.$('button.claim-button');
    if (claimButton) {
        await claimButton.click();
        await api.randomDelay(3000, 5000);
        log('success', '签到完成');
    } else {
        log('warn', '未找到签到按钮，可能今日已签到');
    }

    return { success: true };
}`
    },
    {
      id: 2,
      name: 'Uniswap Swap',
      content: `// Uniswap V3 Swap Script
// 依赖API: connectMetaMask, switchNetwork, waitForSelector, log

async function run(page, wallet, api) {
    const ETH_AMOUNT = '0.1';

    log('info', '开始执行 Uniswap Swap');

    // 1. 连接钱包
    await api.connectMetaMask({ expectedChainId: '0x1' });
    await api.switchNetwork('0x1');

    // 2. 打开Uniswap
    await page.goto('https://app.uniswap.org');
    await api.waitForSelector('body');
    await api.randomDelay(2000, 3000);

    // 3. 确认交易
    log('info', '确认交易...');
    await api.clickElement('button[data-testid="swap-button"]');
    await api.randomDelay(1000, 2000);

    // 4. 等待MetaMask确认
    log('info', '等待钱包签名...');
    await api.waitForSelector('div.swap-review', 30000);

    return { success: true };
}`
    },
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
const showApiHelper = ref(true);
const isFullscreen = ref(false);

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
    content: `// ${newScriptName.value}
// 依赖API: waitForSelector, clickElement, randomDelay, log

async function run(page, wallet, api) {
    log('info', '开始执行脚本');

    // 在此编写你的脚本逻辑

    return { success: true };
}`
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

// 监听内容变化自动保存
watch(scriptContent, (newVal) => {
  if (activeScript.value) {
    activeScript.value.content = newVal;
  }
});
</script>

<template>
  <div class="script-editor" :class="{ fullscreen: isFullscreen }">
    <div class="script-list" v-if="!isFullscreen">
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

    <div class="editor-main" v-if="activeScript" :style="{ width: isFullscreen ? '100%' : (showApiHelper ? 'calc(50% - 10px)' : 'calc(100% - 20px)') }">
      <div class="editor-area">
        <div class="editor-toolbar">
          <div class="file-info">
            <icon-file />
            <span>{{ activeScript.name }}</span>
          </div>
          <div class="actions">
            <a-tooltip content="API 参考文档">
              <a-button type="text" size="small" @click="toggleApiHelper" :type="showApiHelper ? 'primary' : 'secondary'">
                <template #icon><icon-book /></template>
              </a-button>
            </a-tooltip>
            <a-tooltip content="全屏编辑">
              <a-button type="text" size="small" @click="toggleFullscreen">
                <template #icon><icon-fullscreen v-if="!isFullscreen" /><icon-fullscreen-exit v-else /></template>
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
            <span>提示: 使用 api. 调用自定义方法，如 api.connectMetaMask()</span>
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
