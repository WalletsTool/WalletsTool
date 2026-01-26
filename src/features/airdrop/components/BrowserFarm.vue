<script setup>
import { ref } from 'vue';
import { Message } from '@arco-design/web-vue';
import { 
  IconPlus, 
  IconPublic, 
  IconRight, 
  IconSettings, 
  IconEdit 
} from '@arco-design/web-vue/es/icon';

const profiles = ref([
  { id: 1, name: 'Default Profile', userAgent: 'Mozilla/5.0...', viewport: '1920x1080', proxy: 'Direct', canvasSpoof: true },
  { id: 2, name: 'Mobile Profile', userAgent: 'Mozilla/5.0 (iPhone...)', viewport: '390x844', proxy: 'Type: HTTP', canvasSpoof: true },
]);

const activeProfile = ref(null);
const isEditing = ref(false);

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
    isEditing.value = false;
    Message.success('配置已保存');
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
        <a-button type="primary" size="small" @click="Message.info('新建功能开发中')">
          <template #icon><icon-plus /></template>
          新建配置
        </a-button>
      </div>
      
      <div class="list-content">
        <div 
          v-for="profile in profiles" 
          :key="profile.id"
          class="profile-item"
          @click="handleEdit(profile)"
        >
          <div class="profile-icon">
            <icon-public />
          </div>
          <div class="profile-info">
            <div class="profile-name">{{ profile.name }}</div>
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
          <a-button @click="handleCancel">取消</a-button>
          <a-button type="primary" @click="handleSave">保存</a-button>
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
          
          <a-space direction="vertical" size="large">
            <a-space>
               <a-switch v-model="activeProfile.canvasSpoof" />
               <span>Canvas 指纹混淆</span>
            </a-space>
             <a-space>
               <a-switch default-checked />
               <span>WebGL 渲染伪装</span>
            </a-space>
             <a-space>
               <a-switch default-checked />
               <span>Audio Context 噪音</span>
            </a-space>
          </a-space>
        </a-form>
      </div>
    </div>
    
    <div class="empty-state" v-else>
      <icon-settings style="font-size: 48px; color: var(--color-text-4)" />
      <p>请选择左侧配置进行编辑</p>
    </div>
  </div>
</template>

<style scoped>
.browser-farm {
  height: 100%;
  display: flex;
  gap: 20px;
}

.profile-list {
  width: 300px;
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
