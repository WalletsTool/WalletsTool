<template>
  <a-modal
    v-model:visible="modalVisible"
    title="代理配置"
    width="580px"
    :mask-closable="true"
    :closable="true"
    :keyboard="true"
    ok-text="保存"
    cancel-text="取消"
    @ok="handleSave"
    @cancel="handleCancel"
  >
    <div class="proxy-config-modal">
      <!-- 头部：开关 -->
      <div class="config-header">
        <span class="label">启用代理服务</span>
        <a-switch v-model="proxyConfig.enabled" checked-text="开" unchecked-text="关" />
      </div>

      <!-- 代理列表输入 -->
      <div class="config-content">
        <div class="section-title">
          <span>代理列表</span>
          <span class="proxy-count" v-if="validProxiesCount > 0">
            (共 {{ validProxiesCount }} 个)
          </span>
        </div>
        <a-textarea
          v-model="proxyListText"
          placeholder="请输入代理地址，每行一个
格式示例：
http://127.0.0.1:7890
socks5://user:pass@127.0.0.1:1080"
          :auto-size="{ minRows: 8, maxRows: 12 }"
          class="proxy-textarea"
          @input="updateProxyCount"
          @change="updateProxyCount"
          @blur="updateProxyCount"
        />
        <div class="input-tip" v-if="totalLines > 0">
          已识别 {{ totalLines }} 行 <span v-if="filteredLinesCount > 0"> (过滤 {{ filteredLinesCount }} 行空白/注释)</span>
        </div>
      </div>

      <!-- 代理测试区域 -->
      <div class="test-section" v-if="proxyConfig.enabled && proxyConfig.proxies.length > 0">
        <div class="test-header">
          <div class="test-controls">
             <a-button 
              :type="testing ? 'secondary' : 'primary'" 
              status="success"
              size="small"
              @click="testing ? cancelTesting() : testAllProxies()"
            >
              <template #icon>
                <icon-stop v-if="testing" />
                <icon-play-arrow v-else />
              </template>
              {{ testing ? '停止测试' : '测试连接' }}
            </a-button>
            <div v-if="testing || testResults.length > 0" class="test-summary">
              <span class="summary-item success"><icon-check-circle /> {{ successCount }}</span>
              <span class="summary-item failure"><icon-close-circle /> {{ failureCount }}</span>
              <span class="summary-item time" v-if="!testing && testDuration > 0">耗时: {{ formatDuration(testDuration) }}</span>
            </div>
          </div>
          <div class="test-progress-bar" v-if="testing">
             <a-progress 
              :percent="testProgress" 
              size="small" 
              :show-text="false"
              :color="{ '0%': 'rgb(var(--primary-6))', '100%': 'rgb(var(--success-6))' }"
            />
          </div>
        </div>

        <!-- 测试结果列表 -->
        <div class="test-results-list" v-if="testResults.length > 0">
          <div 
            v-for="result in testResults" 
            :key="result.proxy"
            class="result-row"
          >
            <div class="result-proxy" :title="result.proxy">{{ result.proxy }}</div>
            <div class="result-status">
              <span v-if="result.success" class="status-success">{{ result.latency }} ms</span>
              <span v-else class="status-failure" :title="result.error">连接失败</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </a-modal>
</template>

<script setup>
import { ref, reactive, watch, computed, onMounted } from 'vue'
import { Message } from '@arco-design/web-vue'
import { IconPlayArrow, IconStop, IconCheckCircle, IconCloseCircle } from '@arco-design/web-vue/es/icon'

// Props
const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false
  }
})

// Emits
const emit = defineEmits(['update:modelValue', 'saved', 'config-change'])
const proxyConfig = reactive({
  enabled: false,
  proxies: []
})
const proxyListText = ref('')
const testing = ref(false)
const testResults = ref([])
const proxyStats = ref({})

// 多线程测试相关变量
const threadCount = ref(50) // 默认50个线程
const testProgress = ref(0) // 测试进度百分比
const testCompletedCount = ref(0) // 已完成测试数量
const testTotalCount = ref(0) // 总测试数量
const testSpeed = ref(0) // 测试速度（个/秒）
const estimatedTime = ref(0) // 预计剩余时间（秒）
const testDuration = ref(0) // 测试总耗时（毫秒）
const testStartTime = ref(0) // 测试开始时间
const testCancelled = ref(false) // 测试是否被取消

// 计算属性
const successCount = computed(() => testResults.value.filter(r => r.success).length)
const failureCount = computed(() => testResults.value.filter(r => !r.success).length)

// 计算总行数和有效代理数
const totalLines = computed(() => {
  if (!proxyListText.value) return 0
  return proxyListText.value.split('\n').length
})

const validProxiesCount = computed(() => proxyConfig.proxies.length)

// 计算过滤掉的行数
const filteredLinesCount = computed(() => {
  if (!proxyListText.value) return 0
  const lines = proxyListText.value.split('\n')
  const emptyOrCommentLines = lines.filter(line => {
    const trimmed = line.trim()
    return !trimmed || trimmed.startsWith('#')
  }).length
  return emptyOrCommentLines
})

// 弹窗可见性计算属性 - 用于双向绑定
const modalVisible = computed({
  get() {
    return Boolean(props.modelValue)
  },
  set(value) {
    emit('update:modelValue', value)
  }
})

// 监听 modelValue 变化
watch(() => props.modelValue, (newVal) => {
  if (newVal) {
    loadProxyConfig()
    loadProxyStats()
  }
})

// 监听代理列表文本变化，确保实时更新
watch(proxyListText, () => {
  updateProxyCount()
})

// 生成唯一的窗口ID
const generateWindowId = () => {
  const timestamp = Date.now().toString(36);
  const randomPart = Math.random().toString(36).substring(2, 9);
  return `window_${timestamp}_${randomPart}`;
}

// 加载代理配置
const loadProxyConfig = async () => {
  try {
    // 检查是否在Tauri环境中
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (!isTauri) {
      Message.warning('当前环境不支持代理配置功能')
      return
    }

    const { invoke } = await import('@tauri-apps/api/core')
    
    // 获取当前窗口的唯一ID
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const currentWindow = await getCurrentWindow()
    
    // 优先使用窗口标签，如果没有则生成唯一ID
    let windowId = currentWindow.label
    if (!windowId || windowId.trim() === '') {
      windowId = generateWindowId()
    }
    
    // 检查是否有持久化的窗口ID
    const storageKey = `proxy_window_id_${currentWindow.label}`
    const storedWindowId = localStorage.getItem(storageKey)
    if (storedWindowId) {
      windowId = storedWindowId
    } else {
      // 存储窗口ID供后续使用
      localStorage.setItem(storageKey, windowId)
    }
    
    console.log('ProxyConfigModal - 当前窗口ID:', windowId, 'label:', currentWindow.label)
    
    // 先设置窗口ID，确保后端使用正确的配置
    await invoke('set_proxy_window_id', { windowId })
    
    const config = await invoke('get_proxy_config')
    proxyConfig.enabled = config.enabled
    proxyConfig.proxies = config.proxies || []
    proxyListText.value = proxyConfig.proxies.join('\n')
    
    // 输出当前加载的代理配置到日志
    console.log('代理配置已加载:', {
      windowId,
      label: currentWindow.label,
      enabled: config.enabled,
      proxyCount: config.proxies ? config.proxies.length : 0,
      proxies: config.enabled && config.proxies ? config.proxies : '代理已禁用或无代理'
    })
  } catch (error) {
    console.error('加载代理配置失败:', error)
    Message.error('加载代理配置失败: ' + error)
  }
}

// 加载代理统计
const loadProxyStats = async () => {
  try {
    // 检查是否在Tauri环境中
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (!isTauri) {
      return
    }

    const { invoke } = await import('@tauri-apps/api/core')
    
    // 获取当前窗口ID
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const currentWindow = await getCurrentWindow()
    
    // 优先使用窗口标签，如果没有则生成唯一ID
    let windowId = currentWindow.label
    if (!windowId || windowId.trim() === '') {
      windowId = generateWindowId()
    }
    
    // 检查是否有持久化的窗口ID
    const storageKey = `proxy_window_id_${currentWindow.label}`
    const storedWindowId = localStorage.getItem(storageKey)
    if (storedWindowId) {
      windowId = storedWindowId
    }
    
    // 使用窗口ID特定的统计命令
    const stats = await invoke('get_proxy_stats_for_window', { windowId })
    proxyStats.value = stats
  } catch (error) {
    console.error('加载代理统计失败:', error)
  }
}

// 测试所有代理 - 多线程并发版本
const testAllProxies = async () => {
  if (proxyConfig.proxies.length === 0) {
    Message.warning('请先添加代理地址')
    return
  }

  // 检查是否在Tauri环境中
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (!isTauri) {
    Message.warning('当前环境不支持代理测试功能')
    return
  }

  // 初始化测试状态
  testing.value = true
  testCancelled.value = false
  testResults.value = []
  testProgress.value = 0
  testCompletedCount.value = 0
  testTotalCount.value = proxyConfig.proxies.length
  testSpeed.value = 0
  estimatedTime.value = 0
  testStartTime.value = Date.now()

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    console.log('开始多线程测试代理连接:', {
      proxyCount: proxyConfig.proxies.length,
      threadCount: threadCount.value,
      proxies: proxyConfig.proxies
    })

    // 分批处理代理列表，每批最多 threadCount 个
    const batchSize = threadCount.value
    const proxies = [...proxyConfig.proxies]
    const results = []

    // 创建进度更新定时器
    const progressTimer = setInterval(() => {
      updateTestProgress()
    }, 100)

    try {
      // 分批并发测试
      for (let i = 0; i < proxies.length; i += batchSize) {
        if (testCancelled.value) {
          console.log('测试被用户取消')
          break
        }

        const batch = proxies.slice(i, i + batchSize)
        console.log(`开始测试第 ${Math.floor(i / batchSize) + 1} 批，共 ${batch.length} 个代理`)

        // 并发测试当前批次的代理
        const batchPromises = batch.map(async (proxy) => {
          if (testCancelled.value) return null

          try {
            console.log('正在测试代理:', proxy)
            const result = await invoke('test_proxy_connection', { proxyUrl: proxy })
            
            const testResult = {
              proxy,
              success: result.success,
              latency: result.latency,
              error: result.error
            }

            // 实时更新结果
            testResults.value.push(testResult)
            testCompletedCount.value++

            console.log('代理测试结果:', {
              proxy,
              success: result.success,
              latency: result.latency
            })

            return testResult
          } catch (error) {
            const testResult = {
              proxy,
              success: false,
              error: error.toString()
            }

            // 实时更新结果
            testResults.value.push(testResult)
            testCompletedCount.value++

            console.log('代理测试失败:', { proxy, error: error.toString() })
            return testResult
          }
        })

        // 等待当前批次完成
        const batchResults = await Promise.all(batchPromises)
        results.push(...batchResults.filter(r => r !== null))

        // 如果被取消，跳出循环
        if (testCancelled.value) {
          break
        }
      }
    } finally {
      clearInterval(progressTimer)
    }

    // 计算最终测试时间
    testDuration.value = Date.now() - testStartTime.value

    if (testCancelled.value) {
      Message.warning(`测试已取消，已完成 ${testCompletedCount.value}/${testTotalCount.value} 个代理的测试`)
    } else {
      Message.success(`测试完成: ${successCount.value} 成功, ${failureCount.value} 失败`)
    }

  } catch (error) {
    console.error('代理测试失败:', error)
    Message.error('代理测试失败: ' + error)
  } finally {
    testing.value = false
    testCancelled.value = false
    // 最终更新进度
    updateTestProgress()
  }
}

// 取消测试
const cancelTesting = () => {
  testCancelled.value = true
  console.log('用户取消了代理测试')
}

// 更新测试进度
const updateTestProgress = () => {
  if (testTotalCount.value > 0) {
    testProgress.value = Math.round((testCompletedCount.value / testTotalCount.value) * 100)
  }

  // 计算测试速度和预计剩余时间
  const elapsedTime = (Date.now() - testStartTime.value) / 1000 // 秒
  if (elapsedTime > 0 && testCompletedCount.value > 0) {
    testSpeed.value = testCompletedCount.value / elapsedTime
    
    const remainingCount = testTotalCount.value - testCompletedCount.value
    if (testSpeed.value > 0 && remainingCount > 0) {
      estimatedTime.value = remainingCount / testSpeed.value
    } else {
      estimatedTime.value = 0
    }
  }
}

// 保存配置
const handleSave = async () => {
  try {
    // 检查是否在Tauri环境中
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (!isTauri) {
      Message.warning('当前环境不支持代理配置功能')
      return
    }

    const { invoke } = await import('@tauri-apps/api/core')
    
    // 获取当前窗口ID
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const currentWindow = await getCurrentWindow()
    
    // 优先使用窗口标签，如果没有则生成唯一ID
    let windowId = currentWindow.label
    if (!windowId || windowId.trim() === '') {
      windowId = generateWindowId()
    }
    
    // 检查是否有持久化的窗口ID
    const storageKey = `proxy_window_id_${currentWindow.label}`
    const storedWindowId = localStorage.getItem(storageKey)
    if (storedWindowId) {
      windowId = storedWindowId
    } else {
      localStorage.setItem(storageKey, windowId)
    }
    
    // 使用窗口ID特定的保存命令
    await invoke('save_proxy_config_for_window', {
      windowId,
      proxies: proxyConfig.proxies,
      enabled: proxyConfig.enabled
    })
    
    // 输出保存的代理配置到日志
    console.log('代理配置已保存:', {
      windowId,
      enabled: proxyConfig.enabled,
      proxyCount: proxyConfig.proxies.length,
      proxies: proxyConfig.enabled ? proxyConfig.proxies : '代理已禁用'
    })
    
    Message.success('代理配置保存成功')
    
    // 触发 saved 事件（保持向后兼容）
    emit('saved', proxyConfig)
    
    // 触发 config-change 事件，通知父组件更新状态
    emit('config-change', {
      enabled: proxyConfig.enabled,
      proxies: proxyConfig.proxies
    })
    
    // 关闭弹窗
    emit('update:modelValue', false)
  } catch (error) {
    console.error('保存代理配置失败:', error)
    Message.error('保存代理配置失败: ' + error)
  }
}

// 取消 - 修复版本，确保弹窗能正常关闭
const handleCancel = () => {
  console.log('代理配置弹窗取消操作')
  
  // 重置测试结果
  testResults.value = []
  testing.value = false
  
  // 关闭弹窗 - 使用正确的方式
  emit('update:modelValue', false)
  
  console.log('弹窗关闭操作完成')
}

// 更新代理数量（处理输入事件）
const updateProxyCount = () => {
  if (proxyListText.value) {
    const lines = proxyListText.value.split('\n')
    const proxies = lines
      .map(line => line.trim())
      .filter(line => line && !line.startsWith('#'))
    proxyConfig.proxies = proxies
    
    // 详细的调试信息
    console.log('代理列表已更新:', {
      原始内容: proxyListText.value,
      总行数: lines.length,
      每行内容: lines.map((line, idx) => `[${idx}]: "${line}"`),
      trim后: lines.map((line, idx) => `[${idx}]: "${line.trim()}"`),
      有效代理数: proxies.length,
      有效代理: proxies
    })
  } else {
    proxyConfig.proxies = []
    console.log('代理列表已清空')
  }
}

// 格式化时间
const formatTime = (timeStr) => {
  if (!timeStr) return '-'
  try {
    const date = new Date(timeStr)
    return date.toLocaleString('zh-CN')
  } catch {
    return '-'
  }
}

// 格式化持续时间
const formatDuration = (milliseconds) => {
  if (!milliseconds || milliseconds < 0) return '0秒'
  
  const seconds = Math.floor(milliseconds / 1000)
  const minutes = Math.floor(seconds / 60)
  
  if (minutes > 0) {
    return `${minutes}分${seconds % 60}秒`
  } else {
    return `${seconds}秒`
  }
}

// 格式化预计剩余时间
const formatEstimatedTime = (seconds) => {
  if (!seconds || seconds < 0) return '0秒'
  
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  
  if (mins > 0) {
    return `${mins}分${secs}秒`
  } else {
    return `${secs}秒`
  }
}

// 组件挂载时加载配置
onMounted(() => {
  if (props.modelValue) {
    loadProxyConfig()
    loadProxyStats()
  }
})
</script>

<style scoped>
.proxy-config-modal {
  padding: 8px 0;
}

.config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 4px 16px;
  border-bottom: 1px solid var(--color-border);
  margin-bottom: 16px;
}

.label {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-1);
}

.section-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--color-text-1);
}

.proxy-count {
  font-weight: normal;
  font-size: 12px;
  color: var(--color-text-3);
}

.proxy-textarea {
  font-family: 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.5;
  background-color: var(--color-fill-1);
}

.input-tip {
  margin-top: 6px;
  font-size: 12px;
  color: var(--color-text-4);
  text-align: right;
}

.test-section {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--color-border);
}

.test-header {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 12px;
}

.test-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.test-summary {
  display: flex;
  gap: 12px;
  font-size: 13px;
  align-items: center;
}

.summary-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.summary-item.success { color: rgb(var(--green-6)); }
.summary-item.failure { color: rgb(var(--red-6)); }
.summary-item.time { color: var(--color-text-3); font-size: 12px; }

.test-results-list {
  max-height: 180px;
  overflow-y: auto;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  padding: 4px 0;
}

.result-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 12px;
  font-size: 12px;
  border-bottom: 1px solid var(--color-fill-2);
}

.result-row:last-child {
  border-bottom: none;
}

.result-proxy {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--color-text-2);
  margin-right: 12px;
  font-family: 'Consolas', monospace;
}

.status-success {
  color: rgb(var(--green-6));
  font-weight: 500;
}

.status-failure {
  color: rgb(var(--red-6));
}
</style>