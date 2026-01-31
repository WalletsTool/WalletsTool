<template>
  <a-modal
    :visible="visible"
    :title="title"
    :width="1000"
    :mask-closable="false"
    :footer="false"
    unmountOnClose
    @update:visible="handleVisibleUpdate"
  >
    <div class="system-import-layout">
      <div class="left">
        <a-spin :loading="groupsLoading" style="width: 100%">
          <a-tree
            :data="groups"
            :field-names="{ key: 'id', title: 'name', children: 'children' }"
            block-node
            v-model:selected-keys="selectedGroupKeys"
            v-model:expanded-keys="expandedKeys"
            @select="onGroupSelect"
            @node-click="onNodeClick"
          />
        </a-spin>
      </div>
      <div class="right">
        <div class="filters">
          <a-select v-model="filterForm.chainType" style="width: 120px" :disabled="chainLocked">
            <a-option value="all">全部链</a-option>
            <a-option value="evm">EVM</a-option>
            <a-option value="solana">Solana</a-option>
          </a-select>
          <a-select v-model="filterForm.walletType" style="width: 140px">
            <a-option value="all">全部类型</a-option>
            <a-option value="full_wallet">完整钱包</a-option>
            <a-option value="address_only">仅地址</a-option>
          </a-select>
          <a-select v-model="filterForm.sourceType" style="width: 140px">
            <a-option value="all">全部来源</a-option>
            <a-option value="wallet">系统钱包</a-option>
            <a-option value="watch_address">观察地址</a-option>
          </a-select>
          <a-input-search
            v-model="filterForm.searchKeyword"
            allow-clear
            placeholder="搜索名称/地址/备注"
            style="flex: 1"
          />
        </div>

        <div class="list">
          <VirtualScrollerTable
            :columns="columns"
            :data="filteredItems"
            :row-selection="rowSelection"
            :selected-keys="selectedItemKeys"
            row-key="key"
            height="100%"
            page-type="system_import"
            :loading="walletsLoading || watchAddressesLoading"
            :empty-data="filteredItems.length === 0"
            @update:selected-keys="handleSelectionChange"
          >
            <template #index="{ rowIndex }">
              {{ rowIndex + 1 }}
            </template>
            <template #chain_type="{ record }">
              <span>{{ record.chain_type === 'evm' ? 'EVM' : 'SOL' }}</span>
            </template>
            <template #wallet_type="{ record }">
              <span>{{ record.has_private_key ? '完整' : '仅地址' }}</span>
            </template>
            <template #source_type="{ record }">
              <span>{{ record.source_type === 'wallet' ? '系统钱包' : '观察地址' }}</span>
            </template>
          </VirtualScrollerTable>
        </div>

        <div class="footer-bar">
          <div class="left-info">
            已选择: {{ selectedItemKeys.length }} 个
            <span v-if="maxSelection">（最多 {{ maxSelection }} 个）</span>
          </div>
          <div class="right-actions">
            <a-button @click="handleCancel" :disabled="confirmLoading">取消</a-button>
            <a-button @click="handleClearSelection" :disabled="selectedItemKeys.length === 0">清空</a-button>
            <a-button type="primary" @click="handleConfirm" :loading="confirmLoading">确认</a-button>
          </div>
        </div>
      </div>
    </div>
  </a-modal>

  <a-modal
    v-model:visible="passwordModalVisible"
    title="输入钱包管理主密码"
    :mask-closable="false"
    :closable="false"
    :confirm-loading="passwordVerifying"
    unmountOnClose
    @cancel="handlePasswordCancel"
    @before-ok="handlePasswordOk"
  >
    <a-form layout="vertical">
      <a-form-item label="密码">
        <a-input-password ref="passwordInputRef" v-model="passwordInput" placeholder="仅用于本次解密，不会保存" />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup>
import { computed, nextTick, reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Message, Modal, Notification } from '@arco-design/web-vue'
import VirtualScrollerTable from '@/components/VirtualScrollerTable.vue'
import { openSealedSecret } from '@/utils/secretCrypto'

const props = defineProps({
  visible: { type: Boolean, default: false },
  ecosystem: { type: String, default: 'evm' },
  importMode: { type: String, default: 'address_only' },
  title: { type: String, default: '从系统钱包导入' },
  multiple: { type: Boolean, default: true },
  maxSelection: { type: Number, default: undefined },
})

const emit = defineEmits(['confirm', 'cancel', 'update:visible'])

const groupsLoading = ref(false)
const walletsLoading = ref(false)
const watchAddressesLoading = ref(false)
const confirmLoading = ref(false)

const groups = ref([])
const expandedKeys = ref([])
const selectedGroupKeys = ref(['ALL'])

const wallets = ref([])
const watchAddresses = ref([])

const rowSelection = { type: 'checkbox' }

const selectedItemKeys = ref([])

const filterForm = reactive({
  chainType: 'all',
  walletType: 'all',
  sourceType: 'all',
  searchKeyword: '',
})

const chainLocked = computed(() => props.ecosystem === 'evm' || props.ecosystem === 'solana')

watch(
  () => props.ecosystem,
  (v) => {
    if (v === 'evm' || v === 'solana') {
      filterForm.chainType = v
    }
  },
  { immediate: true }
)

const columns = [
  { title: '序号', align: 'center', width: 55, slotName: 'index' },
  { title: '名称', align: 'center', dataIndex: 'name', width: 160, ellipsis: true, tooltip: true },
  { title: '地址', align: 'center', dataIndex: 'address', width: 360, ellipsis: true, tooltip: true },
  { title: '链', align: 'center', width: 70, slotName: 'chain_type' },
  { title: '类型', align: 'center', width: 80, slotName: 'wallet_type' },
  { title: '来源', align: 'center', width: 90, slotName: 'source_type' },
  { title: '备注', align: 'center', dataIndex: 'remark', ellipsis: true, tooltip: true },
]

const parseDbId = (val) => {
  if (typeof val === 'number' && Number.isFinite(val)) return val
  if (typeof val === 'string' && /^\d+$/.test(val)) return Number(val)
  return null
}

const buildTree = (list) => {
  const map = {}
  const roots = []
  ;(list || []).forEach((item) => {
    map[item.id] = { ...item }
  })
  ;(list || []).forEach((item) => {
    if (item.parent_id && map[item.parent_id]) {
      if (!map[item.parent_id].children) {
        map[item.parent_id].children = []
      }
      map[item.parent_id].children.push(map[item.id])
    } else {
      roots.push(map[item.id])
    }
  })
  return roots
}

async function loadGroups() {
  groupsLoading.value = true
  try {
    const res = await invoke('get_groups')
    const userRoots = buildTree(res)
    const systemGroups = [
      { id: 'SYSTEM_EVM', name: 'EVM 生态', children: [], isSystem: true },
      { id: 'SYSTEM_SOLANA', name: 'Solana 生态', children: [], isSystem: true },
    ]
    const otherRoots = []
    userRoots.forEach((node) => {
      if (node.chain_type === 'evm') systemGroups[0].children.push(node)
      else if (node.chain_type === 'solana') systemGroups[1].children.push(node)
      else otherRoots.push(node)
    })
    groups.value = [{ id: 'ALL', name: '全部', children: [...systemGroups, ...otherRoots], isSystem: true }]
    expandedKeys.value = ['ALL', 'SYSTEM_EVM', 'SYSTEM_SOLANA']
  } catch (e) {
    Message.error('加载分组失败: ' + (e?.message || e))
    groups.value = [{ id: 'ALL', name: '全部', children: [], isSystem: true }]
    expandedKeys.value = ['ALL']
  } finally {
    groupsLoading.value = false
  }
}

async function loadWallets() {
  walletsLoading.value = true
  try {
    const key = selectedGroupKeys.value?.[0]
    if (!key) {
      wallets.value = []
      return
    }

    if (key === 'ALL') {
      const chainType = filterForm.chainType === 'all' ? null : filterForm.chainType
      wallets.value = await invoke('get_wallets', { group_id: null, chain_type: chainType })
      return
    }

    if (key === 'SYSTEM_EVM' || key === 'SYSTEM_SOLANA') {
      const chainType = key === 'SYSTEM_EVM' ? 'evm' : 'solana'
      wallets.value = await invoke('get_wallets', { group_id: null, chain_type: chainType })
      return
    }

    const gid = parseDbId(key)
    if (gid === null) {
      wallets.value = []
      return
    }
    const chainType = filterForm.chainType === 'all' ? null : filterForm.chainType
    wallets.value = await invoke('get_wallets', { group_id: gid, chain_type: chainType })
  } catch (e) {
    Message.error('加载钱包失败: ' + (e?.message || e))
    wallets.value = []
  } finally {
    walletsLoading.value = false
  }
}

async function loadWatchAddresses() {
  watchAddressesLoading.value = true
  try {
    const key = selectedGroupKeys.value?.[0]
    if (!key) {
      watchAddresses.value = []
      return
    }

    if (key === 'ALL') {
      const chainType = filterForm.chainType === 'all' ? null : filterForm.chainType
      watchAddresses.value = await invoke('get_watch_addresses', { group_id: null, chain_type: chainType })
      return
    }

    if (key === 'SYSTEM_EVM' || key === 'SYSTEM_SOLANA') {
      const chainType = key === 'SYSTEM_EVM' ? 'evm' : 'solana'
      watchAddresses.value = await invoke('get_watch_addresses', { group_id: null, chain_type: chainType })
      return
    }

    const gid = parseDbId(key)
    if (gid === null) {
      watchAddresses.value = []
      return
    }
    const chainType = filterForm.chainType === 'all' ? null : filterForm.chainType
    watchAddresses.value = await invoke('get_watch_addresses', { group_id: gid, chain_type: chainType })
  } catch (e) {
    Message.error('加载观察地址失败: ' + (e?.message || e))
    watchAddresses.value = []
  } finally {
    watchAddressesLoading.value = false
  }
}

const items = computed(() => {
  const wItems = (wallets.value || []).map((w) => ({
    key: `wallet:${w.id}`,
    id: w.id,
    source_type: 'wallet',
    name: w.name || '',
    address: w.address,
    chain_type: (w.chain_type || '').toLowerCase(),
    wallet_type: w.wallet_type || '',
    has_private_key: !!w.has_private_key,
    has_mnemonic: !!w.has_mnemonic,
    group_id: w.group_id ?? null,
    group_name: w.group_name || '',
    remark: w.remark || '',
  }))

  const aItems = (watchAddresses.value || []).map((a) => ({
    key: `watch_address:${a.id}`,
    id: a.id,
    source_type: 'watch_address',
    name: a.name || '',
    address: a.address,
    chain_type: (a.chain_type || '').toLowerCase(),
    wallet_type: 'address_only',
    has_private_key: false,
    has_mnemonic: false,
    group_id: a.group_id ?? null,
    group_name: a.group_name || '',
    remark: a.remark || '',
  }))

  return [...wItems, ...aItems]
})

const filteredItems = computed(() => {
  const keyword = (filterForm.searchKeyword || '').trim().toLowerCase()
  return items.value
    .filter((it) => {
      if (filterForm.chainType !== 'all' && it.chain_type !== filterForm.chainType) return false
      if (filterForm.sourceType !== 'all' && it.source_type !== filterForm.sourceType) return false
      if (filterForm.walletType === 'full_wallet' && !it.has_private_key) return false
      if (filterForm.walletType === 'address_only' && it.has_private_key) return false
      if (!keyword) return true
      const hay = `${it.name || ''} ${it.address || ''} ${it.remark || ''}`.toLowerCase()
      return hay.includes(keyword)
    })
})

function handleSelectionChange(keys) {
  if (!props.multiple && keys.length > 1) {
    selectedItemKeys.value = [keys[keys.length - 1]]
    return
  }

  if (props.maxSelection && keys.length > props.maxSelection) {
    Message.warning(`最多只能选择 ${props.maxSelection} 个`)
    selectedItemKeys.value = keys.slice(0, props.maxSelection)
    return
  }

  selectedItemKeys.value = keys
}

function handleClearSelection() {
  selectedItemKeys.value = []
}

function handleVisibleUpdate(v) {
  emit('update:visible', v)
  if (!v) emit('cancel')
}

function handleCancel() {
  emit('update:visible', false)
  emit('cancel')
}

function onGroupSelect(keys) {
  if (keys?.length) {
    selectedGroupKeys.value = keys
  }
  refreshLists()
}

function onNodeClick(node) {
  selectedGroupKeys.value = [node.id]
  refreshLists()
}

async function refreshLists() {
  selectedItemKeys.value = []
  await Promise.all([loadWallets(), loadWatchAddresses()])
}

watch(
  () => props.visible,
  async (v) => {
    if (!v) return
    if (!groups.value.length) await loadGroups()
    if (!selectedGroupKeys.value?.length) selectedGroupKeys.value = ['ALL']
    await refreshLists()
  }
)

watch(
  () => [filterForm.chainType, selectedGroupKeys.value?.[0]],
  async () => {
    if (!props.visible) return
    await refreshLists()
  }
)

const passwordModalVisible = ref(false)
const passwordInput = ref('')
const passwordVerifying = ref(false)
const passwordInputRef = ref(null)
let pendingPasswordResolve = null

async function requestPassword() {
  passwordInput.value = ''
  passwordModalVisible.value = true
  return new Promise((resolve) => {
    pendingPasswordResolve = resolve
  })
}

watch(passwordModalVisible, async (v) => {
  if (!v) return
  await nextTick()
  setTimeout(() => {
    const el = passwordInputRef.value
    if (el?.focus) {
      el.focus()
      return
    }
    const inputEl = el?.$el?.querySelector?.('input')
    if (inputEl?.focus) inputEl.focus()
  }, 0)
})

function handlePasswordCancel() {
  passwordModalVisible.value = false
  if (pendingPasswordResolve) pendingPasswordResolve(null)
  pendingPasswordResolve = null
}

async function handlePasswordOk() {
  const pwd = String(passwordInput.value || '').trim()
  if (!pwd) {
    Message.error('请输入密码')
    return false
  }

  passwordVerifying.value = true
  try {
    const ok = await invoke('verify_password', { request: { password: pwd } })
    if (!ok) {
      Message.error('主密码错误')
      return false
    }
    passwordModalVisible.value = false
    if (pendingPasswordResolve) pendingPasswordResolve(pwd)
    pendingPasswordResolve = null
    return true
  } catch (e) {
    Message.error('验证主密码失败: ' + (e?.message || e))
    return false
  } finally {
    passwordVerifying.value = false
  }
}

async function buildConfirmPayload(password) {
  const selected = new Map(items.value.map((it) => [it.key, it]))
  const picked = selectedItemKeys.value.map((k) => selected.get(k)).filter(Boolean)
  if (!picked.length) return []

  if (props.importMode !== 'full') {
    return picked.map((it) => ({
      id: it.id,
      source_type: it.source_type,
      name: it.name,
      address: it.address,
      chain_type: it.chain_type,
      wallet_type: it.has_private_key ? 'full_wallet' : 'address_only',
      group_id: it.group_id ?? undefined,
      group_name: it.group_name || undefined,
      remark: it.remark || undefined,
    }))
  }

  const walletsOnly = picked.filter((it) => it.source_type === 'wallet')
  const invalid = walletsOnly.filter((it) => !it.has_private_key)
  const valid = walletsOnly.filter((it) => it.has_private_key)

  if (!valid.length) return []

  const out = []
  for (const it of valid) {
    const secrets = await invoke('get_wallet_secrets', { id: it.id, password, transport_token: null })
    const sealedPk = secrets?.sealed_private_key
    if (!sealedPk) continue
    const privateKey = await openSealedSecret(sealedPk, password)
    out.push({
      id: it.id,
      source_type: it.source_type,
      name: it.name,
      address: it.address,
      private_key: privateKey,
      chain_type: it.chain_type,
      wallet_type: 'full_wallet',
      group_id: it.group_id ?? undefined,
      group_name: it.group_name || undefined,
      remark: it.remark || undefined,
    })
  }

  if (invalid.length) {
    Notification.warning({
      title: '部分钱包无私钥',
      content: `已自动忽略 ${invalid.length} 个仅地址钱包`,
      position: 'topLeft',
    })
  }

  return out
}

async function handleConfirm() {
  if (!selectedItemKeys.value.length) {
    Notification.warning({ title: '未选择任何钱包', position: 'topLeft' })
    return
  }

  if (props.importMode === 'full') {
    const selected = new Map(items.value.map((it) => [it.key, it]))
    const picked = selectedItemKeys.value.map((k) => selected.get(k)).filter(Boolean)
    const hasInvalid = picked.some((it) => it.source_type !== 'wallet' || !it.has_private_key)
    if (hasInvalid) {
      const ok = await new Promise((resolve) => {
        Modal.confirm({
          title: '部分选择无法用于转账',
          content: '包含观察地址或无私钥钱包，无法用于转账。是否仅导入可用的完整钱包？',
          onOk: () => resolve(true),
          onCancel: () => resolve(false),
        })
      })
      if (!ok) return
    }
  }

  confirmLoading.value = true
  let password = null
  try {
    if (props.importMode === 'full') {
      password = await requestPassword()
      if (!password) return
    }
    const payload = await buildConfirmPayload(password)
    if (!payload.length) {
      Notification.warning({ title: '无可导入数据', position: 'topLeft' })
      return
    }
    emit('confirm', payload)
    emit('update:visible', false)
  } catch (e) {
    Message.error('导入失败: ' + (e?.message || e))
  } finally {
    confirmLoading.value = false
    passwordInput.value = ''
    password = null
  }
}
</script>

<style scoped>
.system-import-layout {
  display: flex;
  gap: 12px;
  height: 560px;
}

.left {
  width: 260px;
  border-right: 1px solid var(--color-border);
  padding-right: 10px;
  overflow: auto;
}

.right {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.filters {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
}

.list {
  flex: 1;
  min-height: 0;
}

.footer-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 10px;
}

.left-info {
  color: var(--color-text-2);
}

.right-actions {
  display: flex;
  gap: 8px;
}
</style>
