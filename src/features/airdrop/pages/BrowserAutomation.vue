<script setup>
import { ref, shallowRef, onMounted, nextTick, defineAsyncComponent } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import WalletManager from '../components/WalletManager.vue';
import {
  IconSafe,
  IconComputer,
  IconCode,
  IconPlayCircle,
  IconPoweroff,
  IconFolder,
  IconSchedule
} from '@arco-design/web-vue/es/icon';

// 异步加载非首屏组件，加快窗口打开速度
const BrowserFarm = defineAsyncComponent(() => import('../components/BrowserFarm.vue'));
const ScriptEditor = defineAsyncComponent(() => import('../components/ScriptEditor.vue'));
const ExecutionPanel = defineAsyncComponent(() => import('../components/ExecutionPanel.vue'));
const TaskManager = defineAsyncComponent(() => import('../components/TaskManager.vue'));
const TaskMonitor = defineAsyncComponent(() => import('../components/TaskMonitor.vue'));

const appWindow = getCurrentWindow();

// Navigation
const menuItems = [
  { id: 'wallets', label: '钱包管理', icon: IconSafe, component: WalletManager },
  { id: 'envs', label: '环境配置', icon: IconComputer, component: BrowserFarm },
  { id: 'scripts', label: '脚本编辑', icon: IconCode, component: ScriptEditor },
  { id: 'tasks', label: '任务管理', icon: IconFolder, component: TaskManager },
  { id: 'monitor', label: '任务监控', icon: IconSchedule, component: TaskMonitor },
  { id: 'execution', label: '执行面板', icon: IconPlayCircle, component: ExecutionPanel },
];

const activeTab = ref('wallets');
const currentComponent = shallowRef(WalletManager);

// 立即发送 page-loaded 事件，不需要等待 onMounted
// 因为窗口只需要显示基础布局，不需要等待所有数据加载
const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
if (isTauri) {
  // 使用微任务确保在组件初始化后立即发送事件
  Promise.resolve().then(() => {
    appWindow.emit('page-loaded');
  });
}

onMounted(async () => {
  // 组件挂载后的其他初始化逻辑
});

const handleNavClick = (item) => {
  activeTab.value = item.id;
  currentComponent.value = item.component;
};

const closeWindow = async () => {
    await appWindow.close();
};

</script>

<template>
  <div class="browser-automation-layout">
    <div class="layout-body">
      <!-- Sidebar -->
      <div class="sidebar">
        <!-- Logo Area is removed as it's now in TitleBar -->
        <div class="nav-menu">
          <div 
            v-for="item in menuItems" 
            :key="item.id" 
            class="nav-item"
            :class="{ active: activeTab === item.id }"
            @click="handleNavClick(item)"
            :title="item.label"
          >
            <component :is="item.icon" class="nav-icon" />
            <span class="nav-label">{{ item.label }}</span>
          </div>
        </div>

        <div class="sidebar-footer">
          <div class="nav-item close-btn" @click="closeWindow" title="关闭窗口">
            <IconPoweroff class="nav-icon" />
            <span class="nav-label">退出</span>
          </div>
        </div>
      </div>

      <!-- Main Content -->
      <div class="main-content">
        <header class="content-header">
          <h2>{{ menuItems.find(i => i.id === activeTab)?.label }}</h2>
        </header>
        
        <div class="content-body">
          <component :is="currentComponent" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.browser-automation-layout {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  background: var(--color-bg-1);
  color: var(--color-text-1);
  overflow: hidden;
}

.layout-body {
  display: flex;
  flex: 1;
  height: 100vh;
  overflow: hidden;
}

/* Sidebar */
.sidebar {
  width: 60px;
  background: var(--color-bg-2);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 0;
  z-index: 10;
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.sidebar:hover {
  width: 180px;
}

.nav-menu {
  flex: 1;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  height: 44px;
  width: 90%;
  margin: 0 auto;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-text-3);
  padding: 0 12px;
  white-space: nowrap;
  overflow: hidden;
  box-sizing: border-box;
}

.nav-icon {
  font-size: 20px;
  min-width: 20px;
  /* Ensure icon stays centered if text is hidden */
  display: flex;
  justify-content: center;
  align-items: center;
}

.nav-label {
  margin-left: 12px;
  font-size: 14px;
  opacity: 0;
  transition: opacity 0.2s;
  font-weight: 500;
}

.sidebar:hover .nav-label {
  opacity: 1;
}

.nav-item:hover {
  background: var(--color-fill-2);
  color: var(--color-text-1);
}

.nav-item.active {
  background: rgba(var(--primary-6), 0.1);
  color: rgb(var(--primary-6));
}

.sidebar-footer {
  width: 100%;
  padding-bottom: 10px;
}

.close-btn:hover {
  color: rgb(var(--danger-6));
  background: rgba(var(--danger-6), 0.1);
}

/* Main Content */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-1);
  overflow: hidden;
}

.content-header {
  height: 60px;
  padding: 0 24px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  background: var(--color-bg-2);
}

.content-header h2 {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text-1);
}

.content-body {
  flex: 1;
  padding: 20px 24px;
  overflow: hidden;
}
</style>
