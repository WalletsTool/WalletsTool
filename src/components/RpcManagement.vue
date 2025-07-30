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
        <a-table-column title="ID" data-index="id" :width="40" />
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
  <a-modal v-model:visible="rpcFormVisible" :title="isRpcEditMode ? '编辑RPC' : '添加RPC'" :width="600"
    @cancel="() => rpcFormVisible = false" :on-before-ok="submitRpcForm">
    <a-form :model="rpcForm" layout="vertical">
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
    rpcManageData.value = result || []
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
}

// 提交RPC表单
async function submitRpcForm() {
  try {
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
  } catch (error) {
    console.error('保存RPC信息失败:', error)
    Notification.error('保存RPC信息失败: ' + error.message)
    return false
  }
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
    const result = await invoke('test_rpc_connection', { rpc_url: record.rpc_url })
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
        const result = await invoke('test_rpc_connection', { rpc_url: record.rpc_url })
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