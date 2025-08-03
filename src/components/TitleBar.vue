<template>
  <!-- 自定义标题栏 -->
  <div class="title-bar">
    <div class="title-bar-text">{{ title }}</div>
    <div class="title-bar-controls">
      <!-- 主题切换开关 -->
      <div class="titlebar-center">
        <div class="theme-toggle-container">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="theme-icon">
            <circle cx="12" cy="12" r="5" />
            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
          </svg>
          <a-switch v-model="isDarkTheme" @change="toggleTheme" size="small" class="theme-switch" />
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="theme-icon">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
          </svg>
        </div>
      </div>
      <button class="title-bar-control" @click="minimizeWindow" title="最小化">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path d="M2 6h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
      <button class="title-bar-control" @click="maximizeWindow" :title="isMaximized ? '还原' : '最大化'">
        <svg v-if="!isMaximized" width="12" height="12" viewBox="0 0 12 12">
          <rect x="2" y="2" width="8" height="8" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" />
        </svg>
        <svg v-else width="12" height="12" viewBox="0 0 12 12">
          <rect x="3" y="1" width="6" height="6" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" />
          <rect x="1" y="3" width="6" height="6" stroke="currentColor" stroke-width="1.5" fill="none" stroke-linecap="round" />
        </svg>
      </button>
      <button class="title-bar-control close" @click="closeWindow" title="关闭">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useThemeStore } from '@/stores'

// 窗口状态管理
const isMaximized = ref(false)

// Props
const props = defineProps({
  title: {
    type: String,
    default: 'Web3 Tools'
  }
})

// Emits
const emit = defineEmits(['before-close'])

// 主题管理
const themeStore = useThemeStore()
const currentTheme = computed(() => themeStore.currentTheme)
const isDarkTheme = computed({
  get: () => currentTheme.value === 'dark',
  set: (value) => {
    // 这里不需要处理，因为change事件会调用toggleTheme
  }
})

// 主题切换方法
function toggleTheme() {
  themeStore.toggleTheme()
}

// 初始化主题和窗口状态
onMounted(async () => {
  themeStore.initTheme()
  
  // 初始化窗口最大化状态
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow()
      isMaximized.value = await currentWindow.isMaximized()
    } catch (error) {
      console.error('Error getting window state:', error)
    }
  }
})

// 窗口控制方法
async function minimizeWindow() {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow()
      await currentWindow.minimize()
    } catch (error) {
      console.error('Error minimizing window:', error)
    }
  }
}

async function maximizeWindow() {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow()
      const currentMaximized = await currentWindow.isMaximized()
      if (currentMaximized) {
        await currentWindow.unmaximize()
        isMaximized.value = false
      } else {
        await currentWindow.maximize()
        isMaximized.value = true
      }
    } catch (error) {
      console.error('Error toggling maximize window:', error)
    }
  }
}

async function closeWindow() {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (isTauri) {
    try {
      console.log('TitleBar窗口关闭事件触发，正在通知父组件执行清理操作...');
      
      // 触发before-close事件，让父组件有机会执行清理操作
      await emit('before-close');
      
      const currentWindow = getCurrentWindow()
      await currentWindow.destroy()
    } catch (error) {
      console.error('Error closing window:', error)
    }
  }
}
</script>

<style scoped>
.title-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 40px;
  background: rgba(255, 255, 255, 0.1);
  color: white;
  font-size: 14px;
  -webkit-app-region: drag;
  user-select: none;
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(30, 58, 138, 0.3);
  padding: 0 10px;
  font-weight: 500;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
}

.title-bar-text {
  margin-left: 10px;
  font-weight: 500;
  font-size: 13px;
}

.title-bar-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 100%;
  -webkit-app-region: no-drag;
}

.title-bar-control {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  font-size: 16px;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.title-bar-control:hover {
  background: rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.9);
}

.title-bar-control.close:hover {
  background-color: rgba(255, 96, 96, 0.8) !important;
  color: white !important;
}

.title-bar-control.theme-toggle {
  width: 40px;
  margin-right: 5px;
}

.title-bar-control.theme-toggle:hover {
  background-color: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
}

.title-bar-control.theme-toggle:hover .theme-icon {
  transform: scale(1.1);
}

.theme-icon {
  font-size: 16px;
  transition: transform 0.3s ease;
  width: 14px;
  height: 14px;
  opacity: 0.8;
}

.title-bar-control.close:hover {
  background: rgba(255, 96, 96, 0.8);
  color: white;
}

.title-bar-control:first-of-type:hover {
  background: rgba(255, 206, 84, 0.8);
  color: white;
}

.title-bar-control:nth-of-type(2):hover {
  background: rgba(52, 152, 219, 0.8);
  color: white;
}

.titlebar-center {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  z-index: 1002;
}

.theme-toggle-container {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  padding: 6px 12px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  -webkit-app-region: no-drag;
}

.theme-icon {
  color: rgba(255, 255, 255, 0.7);
  transition: color 0.2s ease;
}

.theme-toggle-container:hover .theme-icon {
  color: rgba(255, 255, 255, 0.9);
}

.theme-switch {
  margin: 0 4px;
}

/* 图标样式统一处理 */
.title-bar-control .iconify {
  opacity: 0.9;
  transition: opacity 0.2s ease;
}

.title-bar-control:hover .iconify {
  opacity: 1;
}

/* 明亮主题样式 */
:root[data-theme="light"] .title-bar {
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
  color: #1a202c;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .title-bar-control {
  color: #4a5568;
}

:root[data-theme="light"] .title-bar-control:hover {
  background-color: rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .title-bar-control.close:hover {
  background-color: #e53e3e;
  color: white;
}

:root[data-theme="light"] .theme-switch :deep(.arco-switch) {
  background-color: rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(0, 0, 0, 0.2);
}

:root[data-theme="light"] .theme-switch :deep(.arco-switch-checked) {
  background-color: rgba(0, 0, 0, 0.2);
}

:root[data-theme="light"] .theme-switch :deep(.arco-switch-dot) {
  background-color: #4a5568;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

:root[data-theme="light"] .theme-toggle-container {
  background: rgba(0, 0, 0, 0.05);
  border: 1px solid rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .theme-icon {
  color: rgba(0, 0, 0, 0.7);
}

:root[data-theme="light"] .theme-toggle-container:hover .theme-icon {
  color: rgba(0, 0, 0, 0.9);
}
</style>