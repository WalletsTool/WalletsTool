<template>
  <div class="chain-icon-container">
    <img v-if="iconSrc && !loading" :src="iconSrc" :alt="alt" class="chain-icon" />
    <div v-else-if="loading" class="chain-icon-loading"></div>
    <span v-else>-</span>
  </div>
</template>

<script setup>
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Props定义
const props = defineProps({
  chainKey: {
    type: String,
    default: ''
  },
  picData: {
    type: String,
    default: ''
  },
  alt: {
    type: String,
    default: 'Chain Icon'
  }
})

// 响应式数据
const iconSrc = ref(null)
const loading = ref(true)

// 链图标缓存
const chainIconCache = new Map()

// 获取链图标数据
const getChainIconSrc = async (chainKey) => {
  if (!chainKey) return null

  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (isTauri) {
    try {
      const iconData = await invoke('get_chain_icon', { chainKey })
      if (iconData) {
        return `${iconData}`
      }
    } catch (error) {
      console.error('获取链图标失败:', error)
    }
  }

  // 回退到文件路径（向后兼容）
  return null
}

// 获取缓存的链图标
const getCachedChainIcon = async (chainKey) => {
  const cacheKey = `${chainKey}`

  if (chainIconCache.has(cacheKey)) {
    return chainIconCache.get(cacheKey)
  }

  const iconSrc = await getChainIconSrc(chainKey)
  chainIconCache.set(cacheKey, iconSrc)
  return iconSrc
}

// 加载图标
const loadIcon = async () => {
  try {
    loading.value = true
    // 优先使用pic_data，如果没有则使用原有逻辑
    if (props.picData) {
      iconSrc.value = `${props.picData}`
    } else {
      iconSrc.value = await getCachedChainIcon(props.chainKey)
    }
  } catch (error) {
    console.error('加载图标失败:', error)
    iconSrc.value = null
  } finally {
    loading.value = false
  }
}

// 监听props变化
watch(() => [props.chainKey, props.picData], loadIcon, { immediate: true })

// 组件挂载时加载图标
onMounted(() => {
  loadIcon()
})
</script>

<style scoped>
.chain-icon-container {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.chain-icon {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  object-fit: cover;
}

.chain-icon-loading {
  width: 20px;
  height: 20px;
  background: #f0f0f0;
  border-radius: 50%;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0% {
    opacity: 1;
  }

  50% {
    opacity: 0.5;
  }

  100% {
    opacity: 1;
  }
}
</style>