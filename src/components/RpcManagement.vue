<template>
  <!-- RPC管理弹窗 -->
  <a-modal v-model:visible="visible" title="RPC管理" :width="1200" @cancel="closeManage">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <h3 style="margin: 0;">{{ chainName }} - RPC节点管理</h3>
      <div style="display: flex; gap: 8px;">
        <a-button type="outline" @click="batchTestAllRpc" :loading="batchTesting">
          <icon-thunderbolt />
          {{ batchTesting ? '批量测试中...' : '批量测试' }}
        </a-button>
        <a-button type="primary" @click="showAddRpc">
          <icon-plus />
          添加RPC
        </a-button>
      </div>
    </div>

    <a-table :data="rpcManageData" :loading="rpcTableLoading" :pagination="false" :scroll="{ y: 400 }">
      <template #columns>
        <a-table-column title="ID" data-index="id" :width="70" />
        <a-table-column title="RPC地址" data-index="rpc_url" :width="300" :ellipsis="true" :tooltip="true" />
        <a-table-column title="优先级" data-index="priority" :width="65" align="center" />
        <a-table-column title="状态" :width="60" align="center">
          <template #cell="{ record }">
            <a-tag :color="record.is_active ? 'green' : 'red'">
              {{ record.is_active ? '启用' : '禁用' }}
            </a-tag>
          </template>
        </a-table-column>
        <a-table-column title="响应时间(ms)" data-index="response_time" :width="100" align="center">
          <template #cell="{ record }">
            <span
              :style="{ color: record.response_time > 200 ? '#f53f3f' : record.response_time > 100 ? '#ff7d00' : '#00b42a' }">
              {{ record.response_time || '-' }}
            </span>
          </template>
        </a-table-column>
        <a-table-column title="成功率(%)" data-index="success_rate" :width="90" align="center">
          <template #cell="{ record }">
            <span
              :style="{ color: record.success_rate < 95 ? '#f53f3f' : record.success_rate < 98 ? '#ff7d00' : '#00b42a' }">
              {{ record.success_rate ? record.success_rate.toFixed(1) : '-' }}
            </span>
          </template>
        </a-table-column>
        <a-table-column title="操作" :width="280">
          <template #cell="{ record }">
            <a-button type="text" @click="testRpcConnection(record)" size="small"
              :loading="record.testing" status="normal">
              {{ record.testing ? '测试中...' : '测试' }}
            </a-button>
            <a-button type="text" @click="showEditRpc(record)" size="small">
              编辑
            </a-button>
            <a-button type="text" @click="toggleRpcStatus(record)" size="small"
              :status="record.is_active ? 'warning' : 'success'">
              {{ record.is_active ? '禁用' : '启用' }}
            </a-button>
            <a-popconfirm content="确定要删除这个RPC节点吗？" @ok="deleteRpcFromManage(record.id)">
              <a-button type="text" status="danger" size="small">
                <icon-delete />
                删除
              </a-button>
            </a-popconfirm>
          </template>
        </a-table-column>
      </template>
    </a-table>

    <template #footer>
      <a-button @click="closeManage">关闭</a-button>
    </template>
  </a-modal>

  <!-- 添加/编辑RPC弹窗 -->
  <a-modal v-model:visible="rpcFormVisible" :title="isRpcEditMode ? '编辑RPC' : '添加RPC'" :width="700"
    @cancel="() => rpcFormVisible = false" :on-before-ok="submitRpcForm">
    <a-form :model="rpcForm" layout="vertical">
      <!-- 批量模式切换 -->
      <div v-if="!isRpcEditMode" style="margin-bottom: 16px;">
        <a-radio-group v-model="rpcInputMode" type="button">
          <a-radio value="single">单个添加</a-radio>
          <a-radio value="batch">批量添加</a-radio>
        </a-radio-group>
      </div>
      
      <!-- 单个添加模式 -->
      <div v-if="rpcInputMode === 'single'">
        <a-form-item label="RPC地址" required>
          <a-input v-model="rpcForm.rpc_url" placeholder="例如：https://mainnet.infura.io/v3/your-key" />
          <template #help>
            请输入完整的RPC地址，必须以http://或https://开头
          </template>
        </a-form-item>

        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="优先级" required>
              <a-input-number v-model="rpcForm.priority" :min="1" :max="999" placeholder="100" />
              <template #help>
                数值越大优先级越高（1-999）
              </template>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="状态">
              <a-switch v-model="rpcForm.is_active" checked-text="启用" unchecked-text="禁用" />
            </a-form-item>
          </a-col>
        </a-row>
      </div>
      
      <!-- 批量添加模式 -->
      <div v-if="rpcInputMode === 'batch'">
        <a-form-item label="批量RPC地址" required>
          <div style="margin-bottom: 8px; color: #666; font-size: 14px;">
            请每行输入一个RPC地址，系统将自动校验和去重：
          </div>
          <a-textarea 
            v-model="batchRpcText" 
            placeholder="https://mainnet.infura.io/v3/your-key&#10;https://eth-mainnet.alchemyapi.io/v2/your-key&#10;https://rpc.ankr.com/eth"
            :rows="8"
            style="width: 100%;"
          />
        </a-form-item>
        
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="默认优先级">
              <a-input-number v-model="batchDefaultPriority" :min="1" :max="999" placeholder="100" />
              <template #help>
                批量添加的RPC将使用此优先级
              </template>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="默认状态">
              <a-switch v-model="batchDefaultActive" checked-text="启用" unchecked-text="禁用" />
            </a-form-item>
          </a-col>
        </a-row>
        
        <!-- 校验结果显示 -->
        <div v-if="batchRpcValidation.length > 0" style="margin-bottom: 16px;">
          <div style="margin-bottom: 8px; font-weight: 500;">校验结果：</div>
          <div style="max-height: 200px; overflow-y: auto; border: 1px solid #e5e5e5; border-radius: 4px; padding: 8px;">
            <div v-for="(item, index) in batchRpcValidation" :key="index" 
                 :style="{color: item.valid ? '#52c41a' : '#ff4d4f', fontSize: '12px', marginBottom: '4px'}">
              {{ item.url }} - {{ item.message }}
            </div>
          </div>
        </div>
        
        <!-- 统计信息 -->
        <div v-if="batchRpcStats.total > 0" style="background: #f6f8fa; padding: 12px; border-radius: 4px; font-size: 14px;">
          <div>总计：{{ batchRpcStats.total }} 个地址</div>
          <div style="color: #52c41a;">有效：{{ batchRpcStats.valid }} 个</div>
          <div style="color: #ff4d4f;">无效：{{ batchRpcStats.invalid }} 个</div>
          <div style="color: #faad14;">重复：{{ batchRpcStats.duplicate }} 个</div>
        </div>
      </div>
    </a-form>
  </a-modal>
</template>

<script setup>
import { ref, reactive, watch, computed } from 'vue'
import { IconPlus, IconDelete, IconThunderbolt } from '@arco-design/web-vue/es/icon'
import { Notification } from '@arco-design/web-vue'
import { invoke } from '@tauri-apps/api/core'

// Props
const props = defineProps({
  chainValue: {
    type: String,
    default: ''
  },
  chainOptions: {
    type: Array,
    default: () => []
  }
})

// Emits
const emit = defineEmits(['rpc-updated'])

// 响应式数据
const visible = ref(false)
const rpcFormVisible = ref(false)
const isRpcEditMode = ref(false)
const rpcManageData = ref([])
const currentEditRpc = ref(null)
const rpcTableLoading = ref(false)
const batchTesting = ref(false)

// 批量添加RPC相关
const rpcInputMode = ref('single')
const batchRpcText = ref('')
const batchDefaultPriority = ref(100)
const batchDefaultActive = ref(true)
const batchRpcValidation = ref([])
const batchRpcStats = reactive({
  total: 0,
  valid: 0,
  invalid: 0,
  duplicate: 0
})

// RPC信息表单
const rpcForm = reactive({
  rpc_url: '',
  priority: 100,
  is_active: true
})

// 计算属性
const chainName = computed(() => {
  const chain = props.chainOptions.find(c => c.key === props.chainValue)
  return chain?.name || '当前区块链'
})

// 监听 visible 变化
watch(visible, (newVal) => {
  if (newVal && props.chainValue) {
    loadRpcData()
  }
})

// 监听链变化
watch(() => props.chainValue, (newVal) => {
  if (newVal && visible.value) {
    loadRpcData()
  }
})

// 加载RPC数据
async function loadRpcData() {
  if (!props.chainValue) return
  
  try {
    rpcTableLoading.value = true
    const result = await invoke('get_rpc_providers', { chainKey: props.chainValue })
    // 排序：先按启用状态排序（启用在前），再按RPC地址排序
    rpcManageData.value = sortRpcData(result || [])
  } catch (error) {
    console.error('加载RPC数据失败:', error)
    Notification.error('加载RPC数据失败')
  } finally {
    rpcTableLoading.value = false
  }
}

// 显示添加RPC弹窗
function showAddRpc() {
  isRpcEditMode.value = false
  currentEditRpc.value = null
  resetRpcForm()
  rpcFormVisible.value = true
}

// 显示编辑RPC弹窗
function showEditRpc(record) {
  isRpcEditMode.value = true
  currentEditRpc.value = record
  
  // 填充表单数据
  Object.assign(rpcForm, {
    rpc_url: record.rpc_url || '',
    priority: record.priority || 100,
    is_active: record.is_active !== undefined ? record.is_active : true
  })
  
  rpcFormVisible.value = true
}

// 重置RPC表单
function resetRpcForm() {
  Object.assign(rpcForm, {
    rpc_url: '',
    priority: 100,
    is_active: true
  })
  
  // 重置批量模式相关数据
  rpcInputMode.value = 'single'
  batchRpcText.value = ''
  batchDefaultPriority.value = 100
  batchDefaultActive.value = true
  batchRpcValidation.value = []
  resetBatchRpcStats()
}

// 重置批量RPC统计
function resetBatchRpcStats() {
  Object.assign(batchRpcStats, {
    total: 0,
    valid: 0,
    invalid: 0,
    duplicate: 0
  })
}

// URL标准化函数
function normalizeUrl(url) {
  try {
    const urlObj = new URL(url.trim())
    // 移除末尾斜杠，转换为小写
    return urlObj.href.toLowerCase().replace(/\/$/, '')
  } catch {
    return url.trim().toLowerCase().replace(/\/$/, '')
  }
}

// 实时校验批量RPC输入
function validateBatchRpcInput() {
  if (!batchRpcText.value.trim()) {
    batchRpcValidation.value = []
    resetBatchRpcStats()
    return
  }
  
  // 解析输入的RPC地址
  const inputUrls = batchRpcText.value
    .split('\n')
    .map(url => url.trim())
    .filter(url => url.length > 0)
  
  // 获取现有RPC地址并标准化用于去重比较
  const existingNormalizedUrls = new Set(
    rpcManageData.value.map(rpc => normalizeUrl(rpc.rpc_url))
  )
  
  // 校验和统计
  const validation = []
  const seenNormalizedUrls = new Set()
  
  resetBatchRpcStats()
  batchRpcStats.total = inputUrls.length
  
  inputUrls.forEach(url => {
    const result = validateRpcUrl(url)
    const normalizedUrl = normalizeUrl(url)
    
    if (!result.valid) {
      validation.push({ url, valid: false, message: result.message })
      batchRpcStats.invalid++
    } else if (existingNormalizedUrls.has(normalizedUrl)) {
      validation.push({ url, valid: false, message: '重复地址（已存在）' })
      batchRpcStats.duplicate++
    } else if (seenNormalizedUrls.has(normalizedUrl)) {
      validation.push({ url, valid: false, message: '重复地址（输入重复）' })
      batchRpcStats.duplicate++
    } else {
      validation.push({ url, valid: true, message: result.message })
      seenNormalizedUrls.add(normalizedUrl)
      batchRpcStats.valid++
    }
  })
  
  batchRpcValidation.value = validation
}

// 监听批量RPC文本变化
watch(batchRpcText, () => {
  validateBatchRpcInput()
}, { immediate: true })

// RPC数据排序函数
function sortRpcData(data) {
  return data.sort((a, b) => {
    // 首先按is_active降序排序（true在前，false在后）
    if (a.is_active !== b.is_active) {
      return b.is_active - a.is_active
    }
    // 在相同启用状态内，按rpc_url升序排序
    return a.rpc_url.localeCompare(b.rpc_url)
  })
}

// 提交RPC表单
async function submitRpcForm() {
  try {
    if (rpcInputMode.value === 'batch' && !isRpcEditMode.value) {
      return await submitBatchRpcs()
    } else {
      return await submitSingleRpc()
    }
  } catch (error) {
    console.error('保存RPC信息失败:', error)
    Notification.error('保存RPC信息失败: ' + error.message)
    return false
  }
}

// 提交单个RPC
async function submitSingleRpc() {
  // 验证必填字段
  if (!rpcForm.rpc_url) {
    Notification.warning('请输入RPC地址')
    return false
  }

  // 验证RPC地址格式
  if (!rpcForm.rpc_url.startsWith('http://') && !rpcForm.rpc_url.startsWith('https://')) {
    Notification.warning('RPC地址必须以http://或https://开头')
    return false
  }

  const rpcData = {
    chain_key: props.chainValue,
    rpc_url: rpcForm.rpc_url,
    priority: rpcForm.priority,
    is_active: rpcForm.is_active
  }

  if (isRpcEditMode.value) {
    await invoke('update_rpc_provider', {
      id: currentEditRpc.value.id,
      request: {
        rpc_url: rpcData.rpc_url,
        priority: rpcData.priority,
        is_active: rpcData.is_active
      }
    })
    Notification.success('RPC信息更新成功')
  } else {
    await invoke('add_rpc_provider', {
      chainKey: rpcData.chain_key,
      rpcUrl: rpcData.rpc_url,
      priority: rpcData.priority
    })
    Notification.success('RPC添加成功')
  }

  rpcFormVisible.value = false
  await loadRpcData()
  return true
}

// 校验RPC地址
function validateRpcUrl(url) {
  const trimmedUrl = url.trim()
  if (!trimmedUrl) {
    return { valid: false, message: '地址为空' }
  }
  
  // 检查是否为有效的URL格式
  try {
    new URL(trimmedUrl)
  } catch {
    return { valid: false, message: '无效的URL格式' }
  }
  
  // 检查是否以http或https开头
  if (!trimmedUrl.startsWith('http://') && !trimmedUrl.startsWith('https://')) {
    return { valid: false, message: '必须以http://或https://开头' }
  }
  
  // 建议使用https
  if (trimmedUrl.startsWith('http://')) {
    return { valid: true, message: '有效（建议使用https）' }
  }
  
  return { valid: true, message: '有效' }
}

// 提交批量RPC
async function submitBatchRpcs() {
  if (!batchRpcText.value.trim()) {
    Notification.warning('请输入RPC地址')
    return false
  }
  
  // 解析输入的RPC地址
  const inputUrls = batchRpcText.value
    .split('\n')
    .map(url => url.trim())
    .filter(url => url.length > 0)
  
  if (inputUrls.length === 0) {
    Notification.warning('请输入有效的RPC地址')
    return false
  }
  
  // 获取现有RPC地址并标准化用于去重比较
  const existingNormalizedUrls = new Set(
    rpcManageData.value.map(rpc => normalizeUrl(rpc.rpc_url))
  )
  
  // 校验和统计
  const validation = []
  const validUrls = []
  const seenNormalizedUrls = new Set()
  
  resetBatchRpcStats()
  batchRpcStats.total = inputUrls.length
  
  inputUrls.forEach(url => {
    const result = validateRpcUrl(url)
    const normalizedUrl = normalizeUrl(url)
    
    if (!result.valid) {
      validation.push({ url, valid: false, message: result.message })
      batchRpcStats.invalid++
    } else if (existingNormalizedUrls.has(normalizedUrl)) {
      validation.push({ url, valid: false, message: '重复地址（已存在）' })
      batchRpcStats.duplicate++
    } else if (seenNormalizedUrls.has(normalizedUrl)) {
      validation.push({ url, valid: false, message: '重复地址（输入重复）' })
      batchRpcStats.duplicate++
    } else {
      validation.push({ url, valid: true, message: result.message })
      validUrls.push(url)
      seenNormalizedUrls.add(normalizedUrl)
      existingNormalizedUrls.add(normalizedUrl) // 防止后续重复
      batchRpcStats.valid++
    }
  })
  
  batchRpcValidation.value = validation
  
  // 如果没有有效的URL，不关闭弹窗，让用户修改
  if (validUrls.length === 0) {
    Notification.warning('没有有效的RPC地址可以添加')
    return false
  }
  
  // 批量添加有效的RPC
  let successCount = 0
  let failCount = 0
  
  for (const url of validUrls) {
    try {
      await invoke('add_rpc_provider', {
        chainKey: props.chainValue,
        rpcUrl: url,
        priority: batchDefaultPriority.value
      })
      successCount++
    } catch (error) {
      console.error(`添加RPC失败: ${url}`, error)
      failCount++
    }
  }
  
  // 显示结果
  if (successCount > 0) {
    Notification.success(`成功添加 ${successCount} 个RPC地址${failCount > 0 ? `，失败 ${failCount} 个` : ''}`)
  }
  
  if (failCount > 0 && successCount === 0) {
    Notification.error(`添加失败，请检查网络连接或RPC地址`)
    return false
  }
  
  // 刷新数据并关闭弹窗
  await loadRpcData()
  rpcFormVisible.value = false
  return true
}

// 测试RPC连接
async function testRpcConnection(record) {
  try {
    // 设置测试状态
    const index = rpcManageData.value.findIndex(item => item.id === record.id)
    if (index !== -1) {
      rpcManageData.value[index].testing = true
    }
    
    console.log('开始测试RPC连接:', record.rpc_url)
    const result = await invoke('test_rpc_connection', { rpcUrl: record.rpc_url })
    console.log('RPC测试结果:', result)
    
    if (result.success) {
      Notification.success(`RPC测试成功，响应时间: ${result.response_time_ms}ms`)
      // 更新响应时间 - 使用响应式更新
      if (index !== -1) {
        rpcManageData.value[index].response_time = result.response_time_ms
      }
    } else {
      Notification.error(`RPC测试失败: ${result.error || '未知错误'}`)
      if (index !== -1) {
        rpcManageData.value[index].response_time = null
      }
    }
  } catch (error) {
    console.error('RPC测试失败:', error)
    Notification.error('RPC测试失败: ' + error.message)
    const index = rpcManageData.value.findIndex(item => item.id === record.id)
    if (index !== -1) {
      rpcManageData.value[index].response_time = null
    }
  } finally {
    const index = rpcManageData.value.findIndex(item => item.id === record.id)
    if (index !== -1) {
      rpcManageData.value[index].testing = false
    }
  }
}

// 批量测试所有RPC
async function batchTestAllRpc() {
  if (rpcManageData.value.length === 0) {
    Notification.warning('没有可测试的RPC节点')
    return
  }

  try {
    batchTesting.value = true
    console.log('开始批量测试RPC节点，总数:', rpcManageData.value.length)
    
    // 并发测试所有RPC
    const testPromises = rpcManageData.value.map(async (record, index) => {
      try {
        rpcManageData.value[index].testing = true
        console.log('测试RPC:', record.rpc_url)
        const result = await invoke('test_rpc_connection', { rpcUrl: record.rpc_url })
        console.log('RPC测试结果:', record.rpc_url, result)
        
        if (result.success) {
          rpcManageData.value[index].response_time = result.response_time_ms
          rpcManageData.value[index].success_rate = 100 // 简化处理，实际应该基于历史数据
        } else {
          rpcManageData.value[index].response_time = null
          rpcManageData.value[index].success_rate = 0
        }
      } catch (error) {
        console.error('RPC测试失败:', record.rpc_url, error)
        rpcManageData.value[index].response_time = null
        rpcManageData.value[index].success_rate = 0
      } finally {
        rpcManageData.value[index].testing = false
      }
    })

    await Promise.all(testPromises)
    Notification.success('批量测试完成')
  } catch (error) {
    console.error('批量测试失败:', error)
    Notification.error('批量测试失败: ' + error.message)
  } finally {
    batchTesting.value = false
  }
}

// 切换RPC状态
async function toggleRpcStatus(record) {
  try {
    await invoke('update_rpc_provider', {
      id: record.id,
      request: {
        rpc_url: record.rpc_url,
        priority: record.priority,
        is_active: !record.is_active
      }
    })
    
    // 使用响应式更新
    const index = rpcManageData.value.findIndex(item => item.id === record.id)
    if (index !== -1) {
      rpcManageData.value[index].is_active = !rpcManageData.value[index].is_active
    }
    
    // 重新排序数据
    rpcManageData.value = sortRpcData(rpcManageData.value)
    
    Notification.success(`RPC已${!record.is_active ? '启用' : '禁用'}`)
  } catch (error) {
    console.error('切换RPC状态失败:', error)
    Notification.error('切换RPC状态失败: ' + error.message)
  }
}

// 删除RPC
async function deleteRpcFromManage(id) {
  try {
    await invoke('delete_rpc_provider', { id })
    Notification.success('RPC删除成功')
    await loadRpcData()
  } catch (error) {
    console.error('删除RPC失败:', error)
    Notification.error('删除RPC失败: ' + error.message)
  }
}

// 关闭管理弹窗
function closeManage() {
  visible.value = false
}

// 显示弹窗
function show() {
  visible.value = true
}

// 隐藏弹窗
function hide() {
  visible.value = false
}

// 暴露方法给父组件
defineExpose({
  show,
  hide
})
</script>

<style scoped>
/* 组件特定样式 */
</style>