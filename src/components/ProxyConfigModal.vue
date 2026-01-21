<template>
  <a-modal
    v-model:visible="modalVisible"
    title="代理配置"
    width="600px"
    :mask-closable="true"
    :closable="true"
    :keyboard="true"
    ok-text="保存"
    cancel-text="取消"
    @ok="handleSave"
    @cancel="handleCancel"
  >
    <div class="proxy-config-modal">
      <!-- 代理开关 -->
      <div class="config-section">
        <a-row align="middle" :gutter="16">
          <a-col :span="4">
            <label class="config-label">启用代理:</label>
          </a-col>
          <a-col :span="20">
            <a-switch v-model="proxyConfig.enabled" />
            <span class="switch-text">{{ proxyConfig.enabled ? '已启用' : '已禁用' }}</span>
          </a-col>
        </a-row>
      </div>

      <!-- 代理列表输入 -->
      <div class="config-section">
        <a-row :gutter="16">
          <a-col :span="4">
            <label class="config-label">代理列表:</label>
          </a-col>
          <a-col :span="20">
            <a-textarea
              v-model="proxyListText"
              placeholder="请输入代理地址，每行一个，支持格式：&#10;http://proxy.example.com:8080&#10;https://proxy.example.com:8080&#10;socks5://proxy.example.com:1080&#10;&#10;支持用户名密码认证：&#10;http://username:password@proxy.example.com:8080"
              :rows="8"
              :auto-size="false"
              class="proxy-textarea"
              @input="updateProxyCount"
              @change="updateProxyCount"
              @blur="updateProxyCount"
            />
            <div class="proxy-count">
              当前代理数量: {{ validProxiesCount }}
              <span v-if="totalLines > 0" class="proxy-count-detail">
                (总行数: {{ totalLines }}<span v-if="filteredLinesCount > 0">, 过滤: {{ filteredLinesCount }} 行空白/注释</span>)
              </span>
            </div>
          </a-col>
        </a-row>
      </div>

      <!-- 代理测试 -->
      <div class="config-section" v-if="proxyConfig.enabled && proxyConfig.proxies.length > 0">
        <a-row :gutter="16">
          <a-col :span="4">
            <label class="config-label">连接测试:</label>
          </a-col>
          <a-col :span="8">
            <div class="test-controls">
              <div class="test-buttons">
                <a-button 
                  type="primary" 
                  @click="testAllProxies"
                  :loading="testing"
                  :disabled="proxyConfig.proxies.length === 0"
                >
                  {{ testing ? '测试中...' : '测试所有代理' }}
                </a-button>
                <a-button 
                  v-if="testing"
                  type="default"
                  @click="cancelTesting"
                  :disabled="!testing"
                >
                  取消测试
                </a-button>
              </div>
              
              <!-- <div class="thread-config">
                <label class="thread-label">并发线程数:</label>
                <a-input-number
                  v-model:value="threadCount"
                  :min="1"
                  :max="100"
                  :disabled="testing"
                  :style="{ width: '80px' }"
                />
                <span class="thread-hint">（1-100）</span>
              </div> -->
            </div>
            
            <!-- 测试进度 -->
            <div v-if="testing || testResults.length > 0" class="test-progress">
              <div v-if="testing" class="progress-info">
                <a-progress 
                  :percent="testProgress" 
                  :show-info="true"
                  :format="(percent) => `${testCompletedCount}/${testTotalCount}`"
                />
                <div class="progress-details">
                  <span>进度: {{ testCompletedCount }}/{{ testTotalCount }}</span>
                  <span v-if="testSpeed > 0">速度: {{ testSpeed.toFixed(1) }} 个/秒</span>
                  <span v-if="estimatedTime > 0">预计剩余: {{ formatEstimatedTime(estimatedTime) }}</span>
                </div>
              </div>
              
              <div v-if="testResults.length > 0" class="test-summary">
                <span class="test-info">
                  成功: <span class="success-count">{{ successCount }}</span> / 
                  失败: <span class="failure-count">{{ failureCount }}</span>
                </span>
                <span v-if="!testing && testResults.length > 0" class="test-duration">
                  耗时: {{ formatDuration(testDuration) }}
                </span>
              </div>
            </div>
          </a-col>
        </a-row>
      </div>

      <!-- 测试结果 -->
      <div class="config-section" v-if="testResults.length > 0">
        <div class="test-results">
          <h4>测试结果</h4>
          <div class="result-list">
            <div 
              v-for="result in testResults" 
              :key="result.proxy"
              class="result-item"
              :class="{ 'success': result.success, 'failure': !result.success }"
            >
              <div class="proxy-url">{{ result.proxy }}</div>
              <div class="result-status">
                <a-tag :color="result.success ? 'green' : 'red'">
                  {{ result.success ? '成功' : '失败' }}
                </a-tag>
                <span v-if="result.success" class="latency">
                  {{ result.latency }}ms
                </span>
                <span v-else class="error-msg">
                  {{ result.error }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 代理统计 -->
      <div class="config-section" v-if="proxyStats && Object.keys(proxyStats).length > 0">
        <div class="proxy-stats">
          <h4>代理统计</h4>
          <div class="stats-list">
            <div 
              v-for="(stats, proxyUrl) in proxyStats" 
              :key="proxyUrl"
              class="stats-item"
            >
              <div class="proxy-url">{{ proxyUrl }}</div>
              <div class="stats-info">
                <span class="stat">成功: {{ stats.success_count }}</span>
                <span class="stat">失败: {{ stats.failure_count }}</span>
                <span class="stat">平均延迟: {{ stats.avg_latency.toFixed(0) }}ms</span>
                <span class="stat">最后使用: {{ formatTime(stats.last_used) }}</span>
              </div>
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
      Message.success(`代理测试完成，成功: ${successCount.value}，失败: ${failureCount.value}，耗时: ${formatDuration(testDuration.value)}`)
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
  const hours = Math.floor(minutes / 60)
  
  if (hours > 0) {
    return `${hours}小时${minutes % 60}分${seconds % 60}秒`
  } else if (minutes > 0) {
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
  padding: 16px 0;
}

.config-section {
  margin-bottom: 24px;
}

.config-label {
  font-weight: 500;
  color: #1d2129;
  display: inline-block;
  text-align: right;
  padding-right: 8px;
}

:root[data-theme="dark"] .config-label {
  color: #e5e6eb;
}

.switch-text {
  margin-left: 8px;
  color: #86909c;
  font-size: 14px;
}

:root[data-theme="dark"] .switch-text {
  color: #c9cdd4;
}

.proxy-textarea {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.4;
  resize: vertical;
  box-sizing: border-box;
}

.proxy-textarea :deep(.arco-textarea) {
  height: calc(8 * 1.4em + 16px) !important;
  min-height: calc(8 * 1.4em + 16px) !important;
  max-height: none !important;
}

.proxy-count {
  margin-top: 8px;
  color: #86909c;
  font-size: 12px;
}

.proxy-count-detail {
  margin-left: 8px;
  color: #c9cdd4;
  font-size: 11px;
}

:root[data-theme="dark"] .proxy-count {
  color: #a3a6ad;
}

:root[data-theme="dark"] .proxy-count-detail {
  color: #8c8f93;
}

/* 测试控制区域样式 */
.test-controls {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.test-buttons {
  display: flex;
  gap: 8px;
  align-items: center;
}

.thread-config {
  display: flex;
  align-items: center;
  gap: 8px;
}

.thread-label {
  font-size: 14px;
  color: #4e5969;
  white-space: nowrap;
}

.thread-hint {
  font-size: 12px;
  color: #86909c;
}

:root[data-theme="dark"] .thread-label {
  color: #c9cdd4;
}

:root[data-theme="dark"] .thread-hint {
  color: #a3a6ad;
}

/* 测试进度区域样式 */
.test-progress {
  margin-top: 16px;
}

.progress-info {
  margin-bottom: 12px;
}

.progress-details {
  display: flex;
  gap: 16px;
  margin-top: 8px;
  font-size: 12px;
  color: #86909c;
}

:root[data-theme="dark"] .progress-details {
  color: #a3a6ad;
}

.test-summary {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background-color: #f7f8fa;
  border-radius: 4px;
  font-size: 14px;
}

.test-duration {
  color: #4e5969;
  font-size: 12px;
}

.test-info {
  margin-left: 12px;
  color: #86909c;
  font-size: 14px;
}

:root[data-theme="dark"] .test-summary {
  background-color: #2a2d32;
}

:root[data-theme="dark"] .test-duration {
  color: #c9cdd4;
}

:root[data-theme="dark"] .test-info {
  color: #a3a6ad;
}

.success-count {
  color: #00b42a;
  font-weight: 500;
}

.failure-count {
  color: #f53f3f;
  font-weight: 500;
}

.test-duration {
  color: #4e5969;
  font-size: 12px;
}

.proxy-count-detail {
  color: #c9cdd4;
  font-size: 11px;
  margin-left: 4px;
}

.test-info {
  margin-left: 12px;
  color: #86909c;
  font-size: 14px;
}

.test-results {
  margin-top: 16px;
}

.test-results h4 {
  margin: 0 0 12px 0;
  color: #1d2129;
  font-size: 14px;
  font-weight: 500;
}

:root[data-theme="dark"] .test-results h4 {
  color: #e5e6eb;
}

.result-list {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid #e5e6eb;
  border-radius: 4px;
}

.result-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #f2f3f5;
}

.result-item:last-child {
  border-bottom: none;
}

.result-item.success {
  background-color: #f6ffed;
}

.result-item.failure {
  background-color: #fff2f0;
}

.proxy-url {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: #4e5969;
  flex: 1;
  margin-right: 12px;
  word-break: break-all;
}

.result-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.latency {
  color: #00b42a;
  font-size: 12px;
  font-weight: 500;
}

.error-msg {
  color: #f53f3f;
  font-size: 12px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

:root[data-theme="dark"] .result-item.success {
  background-color: #1d3a1d;
}

:root[data-theme="dark"] .result-item.failure {
  background-color: #3d1d1d;
}

:root[data-theme="dark"] .proxy-url {
  color: #c9cdd4;
}

:root[data-theme="dark"] .result-list {
  border-color: #3c3c3c;
}

:root[data-theme="dark"] .stats-list {
  border-color: #3c3c3c;
}

:root[data-theme="dark"] .stats-item {
  border-bottom-color: #3c3c3c;
}

.result-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid #f2f3f5;
}

.result-item:last-child {
  border-bottom: none;
}

.result-item.success {
  background-color: #f6ffed;
}

.result-item.failure {
  background-color: #fff2f0;
}

.proxy-url {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: #4e5969;
  flex: 1;
  margin-right: 12px;
  word-break: break-all;
}

:root[data-theme="dark"] .proxy-url {
  color: #c9cdd4;
}

.result-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.latency {
  color: #00b42a;
  font-size: 12px;
  font-weight: 500;
}

.error-msg {
  color: #f53f3f;
  font-size: 12px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.proxy-stats {
  margin-top: 16px;
}

.proxy-stats h4 {
  margin: 0 0 12px 0;
  color: #1d2129;
  font-size: 14px;
  font-weight: 500;
}

:root[data-theme="dark"] .proxy-stats h4 {
  color: #e5e6eb;
}

.stats-list {
  max-height: 150px;
  overflow-y: auto;
  border: 1px solid #e5e6eb;
  border-radius: 4px;
}

.stats-item {
  padding: 8px 12px;
  border-bottom: 1px solid #f2f3f5;
}

.stats-item:last-child {
  border-bottom: none;
}

.stats-info {
  margin-top: 4px;
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.stat {
  color: #86909c;
  font-size: 12px;
}

:root[data-theme="dark"] .stat {
  color: #a3a6ad;
}
</style>