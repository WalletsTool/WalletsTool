<script setup>
import { ref, onMounted, nextTick } from 'vue';
import { Message } from '@arco-design/web-vue';
import { 
  IconPlus, 
  IconPublic, 
  IconRight, 
  IconSettings, 
  IconEdit,
  IconRobot 
} from '@arco-design/web-vue/es/icon';

const STORAGE_KEY = 'browser_profiles';

const loadProfiles = () => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      return JSON.parse(saved);
    }
  } catch (e) {
    console.error('Failed to load profiles:', e);
  }
  return [
    { id: 1, name: 'Default Profile', userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36', viewport: '1920x1080', proxy: 'Direct', canvasSpoof: true },
    { id: 2, name: 'Mobile Profile', userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1', viewport: '390x844', proxy: 'Type: HTTP', canvasSpoof: true },
  ];
};

const saveProfiles = () => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(profiles.value));
  } catch (e) {
    console.error('Failed to save profiles:', e);
    Message.error('保存失败');
  }
};

const profiles = ref(loadProfiles());

const activeProfile = ref(null);
const isEditing = ref(false);
const showBatchModal = ref(false);
const batchCount = ref(100);

const editingProfileId = ref(null);
const editNameInput = ref(null);
const editNameValue = ref('');

const startEditName = async (profile, event) => {
  event?.stopPropagation();
  editingProfileId.value = profile.id;
  editNameValue.value = profile.name;
  await nextTick();
  if (editNameInput.value) {
    editNameInput.value.focus();
    editNameInput.value.select();
  }
};

const saveEditName = () => {
  const trimmedName = editNameValue.value.trim();
  if (!trimmedName) {
    editingProfileId.value = null;
    return;
  }
  const profile = profiles.value.find(p => p.id === editingProfileId.value);
  if (profile) {
    profile.name = trimmedName;
    saveProfiles();
    Message.success('名称已更新');
  }
  editingProfileId.value = null;
};

const cancelEditName = () => {
  editingProfileId.value = null;
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

const USER_AGENTS = [
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:121.0) Gecko/20100101 Firefox/121.0",
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15"
];

const VIEWPORTS = ["1920x1080", "1366x768", "1440x900", "1536x864", "2560x1440", "1280x720"];

const handleBatchGenerate = () => {
  const newProfiles = [];
  const startId = profiles.value.length > 0 ? Math.max(...profiles.value.map(p => p.id)) + 1 : 1;
  
  for (let i = 0; i < batchCount.value; i++) {
    const ua = USER_AGENTS[Math.floor(Math.random() * USER_AGENTS.length)];
    const vp = VIEWPORTS[Math.floor(Math.random() * VIEWPORTS.length)];
    
    newProfiles.push({
      id: startId + i,
      name: `Auto-Profile-${String(startId + i).padStart(3, '0')}`,
      userAgent: ua,
      viewport: vp,
      proxy: 'Direct',
      canvasSpoof: true,
    });
  }
  
  profiles.value.push(...newProfiles);
  saveProfiles();
  Message.success(`成功生成 ${batchCount.value} 个配置`);
  showBatchModal.value = false;
};

const handleEdit = (profile) => {
  activeProfile.value = { ...profile };
  isEditing.value = true;
};

const handleSave = () => {
  if (activeProfile.value) {
    const index = profiles.value.findIndex(p => p.id === activeProfile.value.id);
    if (index !== -1) {
      profiles.value[index] = { ...activeProfile.value };
    }
    saveProfiles();
    isEditing.value = false;
    Message.success('配置已保存');
  }
};

const handleDelete = () => {
  if (activeProfile.value) {
    const index = profiles.value.findIndex(p => p.id === activeProfile.value.id);
    if (index !== -1) {
      profiles.value.splice(index, 1);
      saveProfiles();
      Message.success('配置已删除');
    }
    isEditing.value = false;
    activeProfile.value = null;
  }
};

const handleCancel = () => {
  isEditing.value = false;
  activeProfile.value = null;
};
</script>

<template>
  <div class="browser-farm">
    <div class="profile-list">
      <div class="list-header">
        <h3>环境配置列表</h3>
        <a-space>
          <a-button type="outline" size="small" @click="showBatchModal = true">
             <template #icon><icon-robot /></template>
             批量生成
          </a-button>
          <a-button style="margin-left: 10px;" type="primary" size="small" @click="Message.info('新建功能开发中')">
            <template #icon><icon-plus /></template>
            新建
          </a-button>
        </a-space>
      </div>
      
      <div class="list-content">
        <div 
          v-for="profile in profiles" 
          :key="profile.id"
          class="profile-item"
          :class="{ active: activeProfile && activeProfile.id === profile.id }"
          @click="handleEdit(profile)"
        >
          <div class="profile-icon">
            <icon-public />
          </div>
          <div class="profile-info">
            <template v-if="editingProfileId === profile.id">
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
              <div class="profile-name editable" @click="(e) => startEditName(profile, e)" title="点击编辑名称">
                {{ profile.name }}
              </div>
            </template>
            <div class="profile-desc">{{ profile.viewport }} | {{ profile.proxy }}</div>
          </div>
          <icon-right class="arrow" />
        </div>
      </div>
    </div>

    <div class="profile-editor" v-if="isEditing && activeProfile">
      <div class="editor-header">
        <h3>编辑配置: {{ activeProfile.name }}</h3>
        <div class="actions">
          <a-button status="danger" @click="handleDelete">删除</a-button>
          <a-button style="margin-left: 10px;" @click="handleCancel">取消</a-button>
          <a-button style="margin-left: 10px;" type="primary" @click="handleSave">保存</a-button>
        </div>
      </div>

      <div class="editor-form">
        <a-form :model="activeProfile" layout="vertical">
          <a-form-item label="配置名称">
            <a-input v-model="activeProfile.name" />
          </a-form-item>
          
          <a-form-item label="User Agent">
            <a-textarea v-model="activeProfile.userAgent" :auto-size="{ minRows: 2, maxRows: 4 }" />
          </a-form-item>

          <a-row :gutter="16">
            <a-col :span="12">
              <a-form-item label="分辨率 (Viewport)">
                <a-input v-model="activeProfile.viewport" />
              </a-form-item>
            </a-col>
            <a-col :span="12">
              <a-form-item label="代理模式">
                <a-select v-model="activeProfile.proxy">
                   <a-option>Direct</a-option>
                   <a-option>Type: HTTP</a-option>
                   <a-option>Type: SOCKS5</a-option>
                </a-select>
              </a-form-item>
            </a-col>
          </a-row>

          <a-divider orientation="left">指纹保护 (Anti-Detect)</a-divider>
          
          <a-space style="display: flex;" direction="vertical" size="large">
            <a-space>
               <a-switch v-model="activeProfile.canvasSpoof" />
               <span style="margin-left: 5px;">Canvas 指纹混淆</span>
            </a-space>
             <a-space style="margin-left: 20px;">
               <a-switch default-checked />
               <span style="margin-left: 5px;">WebGL 渲染伪装</span>
            </a-space>
             <a-space style="margin-left: 20px;"> 
               <a-switch default-checked />
               <span style="margin-left: 5px;">Audio Context 噪音</span>
            </a-space>
          </a-space>
        </a-form>
      </div>
    </div>
    
    <div class="empty-state" v-else>
      <icon-settings style="font-size: 48px; color: var(--color-text-4)" />
      <p>请选择左侧配置进行编辑</p>
    </div>

    <a-modal v-model:visible="showBatchModal" title="批量生成环境配置" @ok="handleBatchGenerate">
      <a-form :model="{ batchCount }">
        <a-form-item label="生成数量">
          <a-input-number v-model="batchCount" :min="1" :max="1000" />
        </a-form-item>
        <div style="color: var(--color-text-3); font-size: 12px;">
          <p>将随机生成以下配置项：</p>
          <ul>
            <li>User Agent (Chrome/Firefox/Safari)</li>
            <li>分辨率 (1920x1080 等常用分辨率)</li>
            <li>指纹保护 (默认开启)</li>
          </ul>
        </div>
      </a-form>
    </a-modal>
  </div>
</template>

<style scoped>
.browser-farm {
  height: 100%;
  display: flex;
  gap: 20px;
}

.profile-list {
  width: 350px;
  background: var(--color-bg-2);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--color-border);
}

.list-header {
  padding: 15px;
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

.profile-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-text-2);
}

.profile-item:hover {
  background: var(--color-fill-2);
}

.profile-item.active {
  background: rgba(var(--primary-6), 0.1);
}

.profile-icon {
  width: 32px;
  height: 32px;
  background: rgba(var(--primary-6), 0.1);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgb(var(--primary-6));
}

.profile-info {
  flex: 1;
}

.profile-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-1);
}

.profile-name.editable {
  cursor: text;
  padding: 2px 4px;
  margin: -2px -4px;
  border-radius: 4px;
  transition: all 0.2s;
}

.profile-name.editable:hover {
  background: var(--color-fill-2);
}

.name-edit-input {
  width: 100%;
  background: var(--color-bg-1);
  border: 1px solid rgb(var(--primary-6));
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-1);
  outline: none;
  line-height: 1.2;
}

.name-edit-input:focus {
  border-color: rgb(var(--primary-6));
  box-shadow: 0 0 0 2px rgba(var(--primary-6), 0.2);
}

.profile-desc {
  font-size: 12px;
  color: var(--color-text-3);
}

.arrow {
  color: var(--color-text-4);
  font-size: 12px;
}

.profile-editor {
  flex: 1;
  background: var(--color-bg-2);
  border-radius: 8px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--color-border);
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid var(--color-border);
}

.editor-header h3 {
  margin: 0;
  color: var(--color-text-1);
}

.editor-form {
  flex: 1;
  overflow-y: auto;
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
}
</style>
