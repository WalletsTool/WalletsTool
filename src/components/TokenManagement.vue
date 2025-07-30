<template>
  <!-- 代币管理弹窗 -->
  <a-modal v-model:visible="visible" title="代币管理" :width="1000" @cancel="closeManage">
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
      <h3 style="margin: 0;">{{ chainName }} - 代币配置管理</h3>
      <a-button type="primary" @click="showAddToken">
        <icon-plus />
        添加代币
      </a-button>
    </div>

    <a-table :data="tokenManageData" :loading="tokenTableLoading" :pagination="false" :scroll="{ y: 400 }">
      <template #columns>
        <a-table-column title="标识" data-index="key" :width="120" />
        <a-table-column title="代币符号" data-index="coin" :width="80" />
        <a-table-column title="类型" data-index="type" :width="60" />
        <a-table-column title="合约地址" data-index="contract_address" :width="200" :ellipsis="true" :tooltip="true" />
        <a-table-column title="合约类型" data-index="contract_type" :width="80" />
        <a-table-column title="小数位数" data-index="decimals" :width="80" />
        <a-table-column title="操作" :width="150">
          <template #cell="{ record }">
            <a-button type="text" @click="showEditToken(record)" size="small">
              编辑
            </a-button>
            <a-popconfirm content="确定要删除这个代币吗？" @ok="deleteTokenFromManage(record.key)">
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

  <!-- 添加/编辑代币弹窗 -->
  <a-modal v-model:visible="tokenFormVisible" :title="isTokenEditMode ? '编辑代币' : '添加代币'" :width="600"
    @cancel="closeAddToken" :on-before-ok="submitTokenForm">
    <a-form :model="tokenForm" layout="vertical">
      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="代币标识" :required="!isTokenEditMode">
            <a-input v-model="tokenForm.key" placeholder="例如：usdt, usdc" :disabled="isTokenEditMode" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="代币名称" required>
            <a-input v-model="tokenForm.name" placeholder="例如：Tether USD, USD Coin" />
          </a-form-item>
        </a-col>
      </a-row>

      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="代币符号" required>
            <a-input v-model="tokenForm.symbol" placeholder="例如：USDT, USDC" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="小数位数" required>
            <a-input-number v-model="tokenForm.decimals" :min="0" :max="18" placeholder="18" />
          </a-form-item>
        </a-col>
      </a-row>

      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item label="代币类型" required>
            <a-select v-model="tokenForm.type" placeholder="选择代币类型">
              <a-option value="base">原生代币</a-option>
              <a-option value="token">合约代币</a-option>
            </a-select>
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="合约类型">
            <a-input v-model="tokenForm.contract_type" placeholder="例如：ERC20, BEP20" />
          </a-form-item>
        </a-col>
      </a-row>

      <a-form-item label="合约地址" :required="tokenForm.type === 'token'">
        <a-input v-model="tokenForm.contract_address" placeholder="代币合约地址" />
      </a-form-item>

      <a-form-item :label="tokenForm.type === 'token' ? 'ABI' : 'ABI (可选)'" :required="tokenForm.type === 'token'">
        <a-textarea v-model="tokenForm.abi" placeholder="合约ABI JSON字符串" :auto-size="{ minRows: 3, maxRows: 6 }" />
      </a-form-item>
      
      <div style="margin-top: 8px; text-align: left;">
        <a-button size="small" @click="setDefaultAbi">默认值</a-button>
      </div>
    </a-form>
  </a-modal>
</template>

<script setup>
import { ref, reactive, watch, computed } from 'vue'
import { IconPlus } from '@arco-design/web-vue/es/icon'
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
const emit = defineEmits(['token-updated'])

// 响应式数据
const visible = ref(false)
const tokenFormVisible = ref(false)
const isTokenEditMode = ref(false)
const tokenManageData = ref([])
const currentEditToken = ref(null)
const tokenTableLoading = ref(false)

// 代币信息表单
const tokenForm = reactive({
  key: '',
  name: '',
  symbol: '',
  decimals: 18,
  type: 'token',
  contract_type: '',
  contract_address: '',
  abi: ''
})

// 计算属性
const chainName = computed(() => {
  const chain = props.chainOptions.find(c => c.key === props.chainValue)
  return chain?.name || '当前区块链'
})

// 监听 visible 变化
watch(visible, (newVal) => {
  if (newVal && props.chainValue) {
    loadTokenData()
  }
})

// 监听链变化
watch(() => props.chainValue, (newVal) => {
  if (newVal && visible.value) {
    loadTokenData()
  }
})

// 加载代币数据
async function loadTokenData() {
  if (!props.chainValue) return
  
  try {
    tokenTableLoading.value = true
    const result = await invoke('get_coin_list', { chainKey: props.chainValue })
    tokenManageData.value = result?.map(token => ({
      key: token.key,
      coin: token.symbol,
      name: token.label, // 后端返回的是 label 字段对应代币名称
      symbol: token.symbol,
      type: token.coin_type || 'token', // 后端返回的是 coin_type 字段
      contract_type: token.contract_type || '',
      contract_address: token.contract_address || '',
      abi: token.abi || '',
      decimals: token.decimals || 18,
      label: token.label
    })) || []
  } catch (error) {
    console.error('加载代币数据失败:', error)
    Notification.error('加载代币数据失败')
  } finally {
    tokenTableLoading.value = false
  }
}

// 显示添加代币弹窗
function showAddToken() {
  isTokenEditMode.value = false
  currentEditToken.value = null
  resetTokenForm()
  tokenFormVisible.value = true
}

// 显示编辑代币弹窗
function showEditToken(record) {
  isTokenEditMode.value = true
  currentEditToken.value = record
  
  // 填充表单数据
  Object.assign(tokenForm, {
    key: record.key || '',
    name: record.name || '', // 使用正确的 name 字段
    symbol: record.symbol || '',
    decimals: record.decimals || 18,
    type: record.type || 'token', // 使用正确的 type 字段
    contract_type: record.contract_type || '',
    contract_address: record.contract_address || '',
    abi: record.abi || ''
  })
  
  tokenFormVisible.value = true
}

// 重置代币表单
function resetTokenForm() {
  Object.assign(tokenForm, {
    key: '',
    name: '',
    symbol: '',
    decimals: 18,
    type: 'token',
    contract_type: '',
    contract_address: '',
    abi: ''
  })
}

// 提交代币表单
async function submitTokenForm() {
  try {
    // 验证必填字段
    if (!tokenForm.name || !tokenForm.symbol) {
      Notification.warning('请填写代币名称和符号')
      return false
    }

    if (!isTokenEditMode.value && !tokenForm.key) {
      Notification.warning('请填写代币标识')
      return false
    }

    if (tokenForm.type === 'token' && !tokenForm.contract_address) {
      Notification.warning('合约代币必须填写合约地址')
      return false
    }

    if (tokenForm.type === 'token' && !tokenForm.abi) {
      Notification.warning('合约代币必须填写ABI')
      return false
    }

    // 验证ABI格式
    if (tokenForm.abi) {
      try {
        JSON.parse(tokenForm.abi)
      } catch (error) {
        Notification.warning('ABI格式不正确，请输入有效的JSON字符串')
        return false
      }
    }

    const tokenData = {
      chain_key: props.chainValue,
      key: tokenForm.key,
      name: tokenForm.name,
      symbol: tokenForm.symbol,
      coin: tokenForm.symbol, // 兼容性字段
      decimals: tokenForm.decimals,
      coin_type: tokenForm.type, // 后端期望的字段名是 coin_type
      contract_type: tokenForm.contract_type,
      contract_address: tokenForm.contract_address,
      abi: tokenForm.abi,
      label: `${tokenForm.symbol} - ${tokenForm.name}`
    }

    if (isTokenEditMode.value) {
      await invoke('update_coin', { 
        chain: props.chainValue,
        key: tokenForm.key,
        objJson: JSON.stringify(tokenData)
      })
      Notification.success('代币信息更新成功')
    } else {
      await invoke('add_coin', { 
        chain: props.chainValue,
        objJson: JSON.stringify(tokenData)
      })
      Notification.success('代币添加成功')
    }

    tokenFormVisible.value = false
    await loadTokenData()
    emit('token-updated')
    return true
  } catch (error) {
    console.error('保存代币信息失败:', error)
    Notification.error('保存代币信息失败: ' + error.message)
    return false
  }
}

// 删除代币
async function deleteTokenFromManage(tokenKey) {
  try {
    await invoke('remove_coin', { 
      chain: props.chainValue, 
      key: tokenKey 
    })
    Notification.success('代币删除成功')
    await loadTokenData()
    emit('token-updated')
  } catch (error) {
    console.error('删除代币失败:', error)
    Notification.error('删除代币失败: ' + error.message)
  }
}

// 关闭管理弹窗
function closeManage() {
  visible.value = false
}

// 关闭添加代币弹窗
function closeAddToken() {
  tokenFormVisible.value = false
  resetTokenForm()
}

// 显示弹窗
function show() {
  visible.value = true
}

// 隐藏弹窗
function hide() {
  visible.value = false
}

// 设置默认ABI
function setDefaultAbi() {
  const defaultAbi = '[{"constant":false,"inputs":[{"name":"newImplementation","type":"address"}],"name":"upgradeTo","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"newImplementation","type":"address"},{"name":"data","type":"bytes"}],"name":"upgradeToAndCall","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"constant":true,"inputs":[],"name":"implementation","outputs":[{"name":"","type":"address"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"newAdmin","type":"address"}],"name":"changeAdmin","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"admin","outputs":[{"name":"","type":"address"}],"payable":false,"stateMutability":"view","type":"function"},{"inputs":[{"name":"_implementation","type":"address"}],"payable":false,"stateMutability":"nonpayable","type":"constructor"},{"payable":true,"stateMutability":"payable","type":"fallback"},{"anonymous":false,"inputs":[{"indexed":false,"name":"previousAdmin","type":"address"},{"indexed":false,"name":"newAdmin","type":"address"}],"name":"AdminChanged","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"name":"implementation","type":"address"}],"name":"Upgraded","type":"event"}]'
  tokenForm.abi = defaultAbi
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