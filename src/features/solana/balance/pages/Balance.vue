<script setup name="balance-solana">
import TitleBar from '@/components/TitleBar.vue'
import { ref } from 'vue'
import { WINDOW_CONFIG } from '@/utils/windowNames'

// 窗口标题定义
const windowTitle = ref('Solana 余额查询')

// 窗口标题初始化
function initSolanaBalanceWindowTitle() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
    if (isTauri) {
      const windowLabel = getCurrentWindow().label
      const saved = WINDOW_CONFIG.getCustomTitle(windowLabel)
      if (saved) {
        windowTitle.value = saved
        return
      }
    }
  } catch (e) {
    console.error('初始化Solana余额查询窗口标题失败:', e)
  }
  
  // 不再设置默认标题，由后端或调用方设置
}

initSolanaBalanceWindowTitle()
</script>

<template>
  <div>
    <TitleBar :title="windowTitle" />
    <div style="padding:16px;">
      即将支持 Solana 生态的余额查询，这里是占位页面。
    </div>
  </div>
</template>

<style scoped>
</style>

