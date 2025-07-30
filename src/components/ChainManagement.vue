<template>
  <!-- 链管理弹窗 -->
  <a-modal v-model:visible="visible" title="区块链管理" :width="1100" @cancel="closeManage">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <h3 style="margin: 0;">链配置管理</h3>
      <a-button type="primary" @click="showAddChain">
        <icon-plus />
        添加新链
      </a-button>
    </div>

    <a-table :data="chainManageData" :loading="chainTableLoading" :pagination="false" :scroll="{ y: 500 }">
      <template #columns>
        <a-table-column title="链标识" data-index="key" :width="80" />
        <a-table-column title="链名称" data-index="name" :width="100" />
        <a-table-column title="链ID" data-index="chain_id" :width="80" />
        <a-table-column title="原生代币" data-index="symbol" :width="80" />
        <a-table-column title="图标" :width="50">
          <template #cell="{ record }">
            <ChainIcon v-if="record.pic_data" :chain-key="record.key" :pic-data="record.pic_data"
              :alt="record.name" />
            <span v-else>-</span>
          </template>
        </a-table-column>
        <a-table-column title="浏览器" data-index="scan_url" :width="150" :ellipsis="true" :tooltip="true" />
        <a-table-column title="RPC数量" :width="70">
          <template #cell="{ record }">
            {{ record.rpc_urls ? record.rpc_urls.length : 0 }} 个
          </template>
        </a-table-column>
        <a-table-column title="操作" :width="150">
          <template #cell="{ record }">
            <a-button type="text" @click="showEditChain(record)" size="small">
              编辑
            </a-button>
            <a-popconfirm content="确定要删除这个链吗？这将同时删除该链下的所有代币！" @ok="deleteChain(record.key)">
              <a-button type="text" status="danger" size="small">
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

  <!-- 添加/编辑链弹窗 -->
  <a-modal v-model:visible="chainFormVisible" :title="isEditMode ? '编辑链' : '添加新链'" :width="800"
    :body-style="{ maxHeight: '400px', overflowY: 'auto' }" @cancel="closeAddChain" :on-before-ok="submitChainForm">
    <a-form :model="chainForm" layout="vertical">
      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="链标识符" required>
            <a-input v-model="chainForm.chain_key" placeholder="例如：eth, bsc, polygon" :disabled="isEditMode"
              @input="formatChainKey" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="链名称" required>
            <a-input v-model="chainForm.chain_name" placeholder="例如：Ethereum, BSC" />
          </a-form-item>
        </a-col>
      </a-row>

      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="链ID" required>
            <a-input v-model="chainForm.chain_id" placeholder="例如：1, 56, 137" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="原生代币符号" required>
            <a-input v-model="chainForm.native_currency_symbol" placeholder="例如：ETH, BNB, MATIC" />
          </a-form-item>
        </a-col>
      </a-row>

      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="原生代币名称" required>
            <a-input v-model="chainForm.native_currency_name" placeholder="例如：Ethereum, Binance Coin" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="小数位数">
            <a-input-number v-model="chainForm.native_currency_decimals" :min="0" :max="18" />
          </a-form-item>
        </a-col>
      </a-row>

      <a-form-item label="链图标">
        <div style="display: flex; gap: 8px; align-items: center;">
          <a-upload :custom-request="uploadChainIcon" :show-file-list="false" accept="image/*">
            <template #upload-button>
              <a-button type="outline">
                <icon-upload />
                上传图标
              </a-button>
            </template>
          </a-upload>
          <!-- 图标预览 -->
          <div v-if="uploadedIconData || (isEditMode && currentEditChain && currentEditChain.pic_data)"
            style="width: 32px; height: 32px; border: 1px solid #e5e5e5; border-radius: 4px; display: flex; align-items: center; justify-content: center; background: #fafafa;">
            <img v-if="uploadedIconData" :src="uploadedIconData"
              style="width: 30px; height: 30px; border-radius: 3px; object-fit: cover;" alt="图标预览" />
            <ChainIcon v-else-if="isEditMode && currentEditChain && currentEditChain.pic_data"
              :chain-key="chainForm.chain_key" :pic-data="currentEditChain.pic_data" :alt="chainForm.chain_name" />
          </div>
        </div>
      </a-form-item>

      <a-form-item label="区块链浏览器URL">
        <a-input v-model="chainForm.scan_url" placeholder="例如：https://etherscan.io" />
      </a-form-item>

      <a-form-item label="浏览器API">
        <a-input v-model="chainForm.scan_api" placeholder="例如：https://api.etherscan.io/api" />
      </a-form-item>

      <a-form-item label="验证API">
        <a-input v-model="chainForm.verify_api" placeholder="验证合约的API地址" />
      </a-form-item>

      <a-form-item label="检查验证API">
        <a-input v-model="chainForm.check_verify_api" placeholder="检查合约验证状态的API地址" />
      </a-form-item>

      <a-form-item label="RPC URLs">
        <div class="rpc-urls">
          <div v-for="(url, index) in chainForm.rpc_urls" :key="index"
            style="margin-bottom: 8px; display: flex; align-items: center; gap: 8px;">
            <a-input v-model="chainForm.rpc_urls[index]" placeholder="RPC节点地址" style="flex: 1;" />
            <a-button v-if="chainForm.rpc_urls.length > 1" @click="removeRpcUrl(index)" type="outline" status="danger"
              size="small">
              <icon-delete />
            </a-button>
          </div>
          <div style="margin-top: 8px;">
            <a-button @click="addRpcUrl" type="outline" size="small">
              <icon-plus />
              添加RPC
            </a-button>
          </div>
        </div>
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup>
import { ref, reactive, watch } from 'vue'
import { IconPlus, IconUpload, IconDelete } from '@arco-design/web-vue/es/icon'
import { Notification } from '@arco-design/web-vue'
import { invoke } from '@tauri-apps/api/core'
import ChainIcon from './ChainIcon.vue'

// Emits
const emit = defineEmits(['refresh', 'chain-updated'])

// 响应式数据
const visible = ref(false)
const chainFormVisible = ref(false)
const isEditMode = ref(false)
const chainManageData = ref([])
const currentEditChain = ref(null)
const chainTableLoading = ref(false)
const uploadedIconData = ref(null)

// 链信息表单
const chainForm = reactive({
  chain_key: '',
  chain_name: '',
  chain_id: '',
  native_currency_symbol: '',
  native_currency_name: '',
  native_currency_decimals: 18,
  scan_url: '',
  scan_api: '',
  verify_api: '',
  check_verify_api: '',
  rpc_urls: ['']
})

// 监听 visible 变化，当弹窗打开时加载数据
watch(visible, (newVal) => {
  if (newVal) {
    loadChainData()
  }
})

// 加载链数据
async function loadChainData() {
  try {
    chainTableLoading.value = true
    const result = await invoke('get_chain_list')
    chainManageData.value = result || []
  } catch (error) {
    console.error('加载链数据失败:', error)
    Notification.error('加载链数据失败')
  } finally {
    chainTableLoading.value = false
  }
}

// 显示添加链弹窗
function showAddChain() {
  isEditMode.value = false
  currentEditChain.value = null
  resetChainForm()
  chainFormVisible.value = true
}

// 显示编辑链弹窗
function showEditChain(record) {
  isEditMode.value = true
  currentEditChain.value = record
  
  // 填充表单数据 - 注意字段映射
  Object.assign(chainForm, {
    chain_key: record.key || record.chain_key || '',
    chain_name: record.name || record.chain_name || '',
    chain_id: String(record.chain_id || ''),
    native_currency_symbol: record.symbol || '',
    native_currency_name: record.currency_name || record.native_currency_name || '',
    native_currency_decimals: record.decimals || record.native_currency_decimals || 18,
    scan_url: record.scan_url || '',
    scan_api: record.scan_api || '',
    verify_api: record.verify_api || '',
    check_verify_api: record.check_verify_api || '',
    rpc_urls: record.rpc_urls && record.rpc_urls.length > 0 ? [...record.rpc_urls] : ['']
  })
  
  // 重置上传的图标数据，保留原有图标
  uploadedIconData.value = null
  
  chainFormVisible.value = true
}

// 重置链表单
function resetChainForm() {
  Object.assign(chainForm, {
    chain_key: '',
    chain_name: '',
    chain_id: '',
    native_currency_symbol: '',
    native_currency_name: '',
    native_currency_decimals: 18,
    scan_url: '',
    scan_api: '',
    verify_api: '',
    check_verify_api: '',
    rpc_urls: ['']
  })
  uploadedIconData.value = null
}

// 格式化链标识符
function formatChainKey() {
  chainForm.chain_key = chainForm.chain_key.toLowerCase().replace(/[^a-z0-9]/g, '')
}

// 添加RPC URL
function addRpcUrl() {
  chainForm.rpc_urls.push('')
}

// 删除RPC URL
function removeRpcUrl(index) {
  if (chainForm.rpc_urls.length > 1) {
    chainForm.rpc_urls.splice(index, 1)
  }
}

// 上传链图标
function uploadChainIcon(option) {
  const file = option.fileItem.file
  if (!file) return

  const reader = new FileReader()
  reader.onload = (e) => {
    uploadedIconData.value = e.target.result
  }
  reader.readAsDataURL(file)
}

// 提交链表单
async function submitChainForm() {
  try {
    // 验证必填字段
    if (!chainForm.chain_key || !chainForm.chain_name || !chainForm.chain_id || !chainForm.native_currency_symbol) {
      Notification.warning('请填写所有必填字段')
      return false
    }

    // 过滤空的RPC URLs
    const filteredRpcUrls = chainForm.rpc_urls.filter(url => url.trim() !== '')
    if (filteredRpcUrls.length === 0) {
      Notification.warning('至少需要一个有效的RPC地址')
      return false
    }

    const chainData = {
      ...chainForm,
      chain_id: parseInt(chainForm.chain_id),
      native_currency_decimals: parseInt(chainForm.native_currency_decimals),
      rpc_urls: filteredRpcUrls,
      // 在编辑模式下，如果没有上传新图标，保留原有图标数据
      pic_data: uploadedIconData.value || (isEditMode.value && currentEditChain.value ? currentEditChain.value.pic_data : null)
    }

    if (isEditMode.value) {
      await invoke('update_chain', { 
        chainKey: chainForm.chain_key,
        requestJson: JSON.stringify(chainData) 
      })
      Notification.success('链信息更新成功')
    } else {
      await invoke('add_chain', { requestJson: JSON.stringify(chainData) })
      Notification.success('链添加成功')
    }

    chainFormVisible.value = false
    await loadChainData()
    emit('refresh')
    emit('chain-updated')
    return true
  } catch (error) {
    console.error('保存链信息失败:', error)
    Notification.error('保存链信息失败: ' + error.message)
    return false
  }
}

// 删除链
async function deleteChain(chainKey) {
  try {
    await invoke('remove_chain', { chainKey })
    Notification.success('链删除成功')
    await loadChainData()
    emit('refresh')
    emit('chain-updated')
  } catch (error) {
    console.error('删除链失败:', error)
    Notification.error('删除链失败: ' + error.message)
  }
}

// 显示弹窗
function show() {
  visible.value = true
}

// 隐藏弹窗
function hide() {
  visible.value = false
}

// 关闭管理弹窗
function closeManage() {
  visible.value = false
}

// 关闭添加链弹窗
function closeAddChain() {
  chainFormVisible.value = false
  resetChainForm()
}

// 暴露方法给父组件
defineExpose({
  show,
  hide
})
</script>

<style scoped>
.rpc-urls {
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  padding: 12px;
  width: 100%;
}
</style>