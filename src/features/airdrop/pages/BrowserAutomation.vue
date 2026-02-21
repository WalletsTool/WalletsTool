<script setup>
import { ref, shallowRef, onMounted, nextTick, defineAsyncComponent, computed } from 'vue';
import { useThemeStore } from '../../../stores';
import WalletManager from '../components/WalletManager.vue';
import {
  IconSafe,
  IconComputer,
  IconCode,
  IconPlayCircle,
  IconPoweroff,
  IconFolder,
  IconSchedule,
  IconPushpin,
  IconApps,
  IconMinus
} from '@arco-design/web-vue/es/icon';

const themeStore = useThemeStore();
const isDarkTheme = computed(() => themeStore.currentTheme === 'dark');

const BrowserFarm = defineAsyncComponent(() => import('../components/BrowserFarm.vue'));
const ScriptEditor = defineAsyncComponent(() => import('../components/ScriptEditor.vue'));
const ExecutionPanel = defineAsyncComponent(() => import('../components/ExecutionPanel.vue'));
const TaskManager = defineAsyncComponent(() => import('../components/TaskManager.vue'));
const TaskMonitor = defineAsyncComponent(() => import('../components/TaskMonitor.vue'));
const ExtensionManager = defineAsyncComponent(() => import('../components/ExtensionManager.vue'));

const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
let appWindow = null;

if (isTauri) {
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
    appWindow = getCurrentWindow();
    Promise.resolve().then(() => {
      appWindow.emit('page-loaded');
    });
  });
}

const menuItems = [
  { id: 'wallets', label: '钱包管理', icon: IconSafe, component: WalletManager },
  { id: 'envs', label: '环境配置', icon: IconComputer, component: BrowserFarm },
  { id: 'extensions', label: '插件管理', icon: IconApps, component: ExtensionManager },
  { id: 'scripts', label: '脚本编辑', icon: IconCode, component: ScriptEditor },
  { id: 'tasks', label: '任务管理', icon: IconFolder, component: TaskManager },
  { id: 'monitor', label: '任务监控', icon: IconSchedule, component: TaskMonitor },
  { id: 'execution', label: '执行面板', icon: IconPlayCircle, component: ExecutionPanel },
];

const activeTab = ref('wallets');
const currentComponent = shallowRef(WalletManager);
const isExpanded = ref(true);
const isPinned = ref(true);

onMounted(async () => {
});

const handleNavClick = (item) => {
  activeTab.value = item.id;
  currentComponent.value = item.component;
};

const handleMouseEnter = () => {
  if (!isPinned.value) {
    isExpanded.value = true;
  }
};

const handleMouseLeave = () => {
  if (!isPinned.value) {
    isExpanded.value = false;
  }
};

const togglePin = () => {
  isPinned.value = !isPinned.value;
  if (isPinned.value) {
    isExpanded.value = true;
  }
};

const closeWindow = async () => {
  if (appWindow) {
    await appWindow.destroy();
  } else if (typeof window !== 'undefined') {
    window.close();
  }
};

const minimizeWindow = async () => {
  if (appWindow) {
    await appWindow.minimize();
  }
};

</script>

<template>
  <div class="browser-automation-layout" :class="{ 'light-theme': !isDarkTheme }">
    <div class="layout-body">
      <!-- Sidebar Container -->
      <div class="sidebar-container" :class="{ expanded: isExpanded }">
        <!-- Collapsed Sidebar -->
        <div class="sidebar-collapsed" v-show="!isExpanded" @mouseenter="handleMouseEnter">
          <div class="nav-menu-collapsed">
            <div 
              v-for="item in menuItems" 
              :key="item.id" 
              class="nav-item-collapsed"
              :class="{ active: activeTab === item.id }"
              @click="handleNavClick(item)"
              :title="item.label"
            >
              <component :is="item.icon" class="nav-icon-collapsed" />
            </div>
          </div>

          <div class="sidebar-footer-collapsed">
            <div class="nav-item-collapsed minimize-btn" @click="minimizeWindow" title="最小化">
              <IconMinus class="nav-icon-collapsed" />
            </div>
            <div class="nav-item-collapsed close-btn" @click="closeWindow" title="关闭窗口">
              <IconPoweroff class="nav-icon-collapsed" />
            </div>
          </div>
        </div>

        <!-- Expanded Sidebar -->
        <div class="sidebar-expanded" v-show="isExpanded" @mouseleave="handleMouseLeave">
          <div class="sidebar-header">
            <div 
              class="pin-btn"
              :class="{ pinned: isPinned }"
              @click="togglePin"
              :title="isPinned ? '取消固定' : '固定侧边栏'"
            >
              <IconPushpin class="pin-icon" :class="{ 'is-pinned': isPinned }" />
            </div>
          </div>

          <div class="nav-menu-expanded">
            <div 
              v-for="item in menuItems" 
              :key="item.id" 
              class="nav-item-expanded"
              :class="{ active: activeTab === item.id }"
              @click="handleNavClick(item)"
            >
              <component :is="item.icon" class="nav-icon-expanded" />
              <span class="nav-label-expanded">{{ item.label }}</span>
            </div>
          </div>

          <div class="sidebar-footer-expanded">
            <div class="nav-item-expanded minimize-btn" @click="minimizeWindow">
              <IconMinus class="nav-icon-expanded" />
              <span class="nav-label-expanded">最小化</span>
            </div>
            <div class="nav-item-expanded close-btn" @click="closeWindow">
              <IconPoweroff class="nav-icon-expanded" />
              <span class="nav-label-expanded">退出</span>
            </div>
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

/* Sidebar Container */
.sidebar-container {
  width: 60px;
  flex-shrink: 0;
  transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.sidebar-container.expanded {
  width: 180px;
}

/* Collapsed Sidebar */
.sidebar-collapsed {
  width: 60px;
  height: 100%;
  background: var(--color-bg-2);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 0;
}

.nav-menu-collapsed {
  flex: 1;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.nav-item-collapsed {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-text-3);
  box-sizing: border-box;
}

.nav-icon-collapsed {
  font-size: 20px;
  width: 20px;
  height: 20px;
}

.nav-item-collapsed:hover {
  background: var(--color-fill-2);
  color: var(--color-text-1);
}

.nav-item-collapsed.active {
  background: rgba(var(--primary-6), 0.1);
  color: rgb(var(--primary-6));
}

.sidebar-footer-collapsed {
  padding-bottom: 10px;
}

/* Expanded Sidebar */
.sidebar-expanded {
  width: 180px;
  height: 100%;
  background: var(--color-bg-2);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  padding: 0 0 20px 0;
}

.sidebar-header {
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 10px;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 8px;
}

.pin-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-text-3);
}

.pin-btn:hover {
  background: var(--color-fill-2);
  color: var(--color-text-1);
}

.pin-btn.pinned {
  color: rgb(var(--primary-6));
}

.pin-btn.pinned:hover {
  background: rgba(var(--primary-6), 0.1);
}

.pin-icon {
  font-size: 16px;
  width: 16px;
  height: 16px;
  transition: all 0.2s;
}

.pin-icon.is-pinned {
  transform: rotate(-45deg);
  color: rgb(var(--primary-6));
}

.nav-menu-expanded {
  flex: 1;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 10px;
  box-sizing: border-box;
}

.nav-item-expanded {
  display: flex;
  align-items: center;
  height: 44px;
  padding: 0 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-text-3);
  white-space: nowrap;
  box-sizing: border-box;
}

.nav-icon-expanded {
  font-size: 20px;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.nav-label-expanded {
  margin-left: 12px;
  font-size: 14px;
  font-weight: 500;
}

.nav-item-expanded:hover {
  background: var(--color-fill-2);
  color: var(--color-text-1);
}

.nav-item-expanded.active {
  background: rgba(var(--primary-6), 0.1);
  color: rgb(var(--primary-6));
}

.sidebar-footer-expanded {
  width: 100%;
  padding: 0 10px 10px;
  box-sizing: border-box;
}

.close-btn:hover {
  color: rgb(var(--danger-6));
  background: rgba(var(--danger-6), 0.1);
}

.minimize-btn:hover {
  color: rgb(var(--primary-6));
  background: rgba(var(--primary-6), 0.1);
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

.light-theme {
  background: #f5f7fa;
}

.light-theme .sidebar-container {
  background: rgba(255, 255, 255, 0.95);
}

.light-theme .sidebar-collapsed,
.light-theme .sidebar-expanded {
  background: rgba(255, 255, 255, 0.95);
  border-color: rgba(0, 0, 0, 0.08);
}

.light-theme .sidebar-header {
  border-color: rgba(0, 0, 0, 0.08);
}

.light-theme .pin-btn {
  color: rgba(0, 0, 0, 0.5);
}

.light-theme .pin-btn:hover {
  background: rgba(0, 0, 0, 0.05);
  color: rgba(0, 0, 0, 0.85);
}

.light-theme .pin-btn.pinned {
  color: #586cc7;
}

.light-theme .pin-btn.pinned:hover {
  background: rgba(88, 108, 199, 0.1);
}

.light-theme .nav-item-collapsed,
.light-theme .nav-item-expanded {
  color: rgba(0, 0, 0, 0.6);
}

.light-theme .nav-item-collapsed:hover,
.light-theme .nav-item-expanded:hover {
  background: rgba(0, 0, 0, 0.05);
  color: rgba(0, 0, 0, 0.85);
}

.light-theme .nav-item-collapsed.active,
.light-theme .nav-item-expanded.active {
  background: rgba(88, 108, 199, 0.1);
  color: #586cc7;
}

.light-theme .close-btn:hover {
  background: rgba(255, 100, 100, 0.15);
  color: #e74c3c;
}

.light-theme .minimize-btn:hover {
  background: rgba(88, 108, 199, 0.1);
  color: #586cc7;
}

.light-theme .main-content {
  background: #f5f7fa;
}

.light-theme .content-header {
  background: rgba(255, 255, 255, 0.95);
  border-color: rgba(0, 0, 0, 0.08);
}

.light-theme .content-header h2 {
  color: #2c3e50;
}
</style>
