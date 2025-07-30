<template>
  <!-- 自定义标题栏 -->
  <div class="title-bar">
    <div class="title-bar-text">{{ title }}</div>
    <div class="title-bar-controls">
      <!-- 主题切换按钮 -->
      <button class="title-bar-control theme-toggle" @click="toggleTheme" :title="currentTheme === 'dark' ? '切换到明亮主题' : '切换到暗黑主题'">
        <span class="theme-icon">{{ currentTheme === 'dark' ? '☀️' : '🌙' }}</span>
      </button>
      <button class="title-bar-control" @click="minimizeWindow" title="最小化">
        <span class="minimize-icon">―</span>
      </button>
      <button class="title-bar-control" @click="maximizeWindow" title="最大化">
        <span class="maximize-icon">▢</span>
      </button>
      <button class="title-bar-control close" @click="closeWindow" title="关闭">
        <span class="close-icon">✕</span>
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useThemeStore } from '@/stores'

// Props
const props = defineProps({
  title: {
    type: String,
    default: 'Web3 Tools'
  }
})

// 主题管理
const themeStore = useThemeStore()
const currentTheme = computed(() => themeStore.currentTheme)

// 主题切换方法
function toggleTheme() {
  themeStore.toggleTheme()
}

// 初始化主题
onMounted(() => {
  themeStore.initTheme()
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
      const isMaximized = await currentWindow.isMaximized()
      if (isMaximized) {
        await currentWindow.unmaximize()
      } else {
        await currentWindow.maximize()
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
  height: 30px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  font-size: 14px;
  -webkit-app-region: drag;
  user-select: none;
}

.title-bar-text {
  margin-left: 10px;
  font-weight: 500;
}

.title-bar-controls {
  display: flex;
  height: 100%;
}

.title-bar-control {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 30px;
  background: transparent;
  border: none;
  color: white;
  cursor: pointer;
  font-size: 16px;
  transition: background-color 0.2s;
  -webkit-app-region: no-drag;
}

.title-bar-control:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.title-bar-control.close:hover {
  background-color: #e81123;
}

.theme-toggle {
  font-size: 14px;
}

.theme-icon {
  font-size: 14px;
}

.minimize-icon {
  font-size: 14px;
  font-weight: bold;
}

.maximize-icon {
  font-size: 12px;
}

.close-icon {
  font-size: 14px;
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
</style>