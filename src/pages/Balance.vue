<script setup name="balance">
import { IconDelete, IconDoubleLeft, IconDownload, IconPlus } from '@arco-design/web-vue/es/icon';
import { useRouter } from "vue-router";
import { nextTick, onBeforeMount, onMounted, reactive, ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Notification } from "@arco-design/web-vue";
// import token_utils from "@/scripts/token/token_utils.js"; // 已迁移到Rust后端
import { utils as xlUtils, writeFile } from "xlsx";
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useVirtualizer } from '@tanstack/vue-virtual'
 import ChainIcon from '@/components/ChainIcon.vue';

const router = useRouter()
// table列名
const columns = [
  {
    title: '序号',
    align: 'center',
    width: '60',
    slotName: 'index'
  },
  {
    title: '钱包地址',
    align: 'center',
    dataIndex: 'address',
    ellipsis: "true",
    tooltip: 'true'
  },
  {
    title: 'Nonce',
    align: 'center',
    width: '80',
    dataIndex: 'nonce',
    ellipsis: "true",
    tooltip: 'true'
  },
  {
    title: '平台币余额',
    align: 'center',
    dataIndex: 'plat_balance',
    width: '120',
    ellipsis: "true",
    tooltip: 'true'
  },
  {
    title: '代币余额',
    align: 'center',
    dataIndex: 'coin_balance',
    width: '120',
    ellipsis: "true",
    tooltip: 'true'
  },
  {
    title: '状态',
    align: 'center',
    slotName: 'exec_status',
    width: '100',
    ellipsis: "true",
    tooltip: 'true'
  },
  {
    title: '错误信息',
    align: 'center',
    dataIndex: 'error_msg',
    ellipsis: "true",
    tooltip: 'true'
  },
  {
    title: '操作',
    align: 'center',
    slotName: 'optional',
    width: '60',
    ellipsis: "true",
    tooltip: 'true'
  }
]
let tableLoading = ref(false)
const data = ref([])
// 选中的数据key
const selectedKeys = ref([]);
// 选择配置
const rowSelection = reactive({
  type: 'checkbox',
  showCheckedAll: true,
  onlyCurrent: false,
});

// 点击行实现选中和取消
function rowClick(record, event) {
  const index = selectedKeys.value.indexOf(record.address)
  index >= 0 ? selectedKeys.value.splice(index, 1) : selectedKeys.value.push(record.address)
}

// 仅查询目标代币
const onlyCoin = ref(true);
// 进度
const progress = ref(0);
// 分页
const pagination = ref(false);
const scrollbar = ref(true);
// rpc默认值
const rpcValue = ref('');
// 当前rpc
const currentRpc = ref({});
// rpc自定义字段名
const rpcFieldNames = { value: 'key', label: 'scan_url' }
// 主网选择器
let rpcOptions = ref([])
// coin默认值
let coinValue = ref('');
// coin自定义字段名
const coinFieldNames = { value: 'key', label: 'name' }
// 币种选择器
const coinOptions = ref([]);
// 查询余额按钮loading
let balanceLoading = ref(false)
// 详细配置
const form = reactive({
  thread_count: 3
})
// 录入 钱包地址 弹窗
let visible = ref(false)
let importText = ref('')
// 添加代币弹窗
let addCoinVisible = ref(false)
let coinAddress = ref('')
// 删除代币弹窗
let deleteTokenVisible = ref(false)
// 删除信息弹窗
let deleteItemVisible = ref(false)
// 当前币种名称
let currentCoin = ref({})
// 当前数据的key
let currentItemKey = ref('')
// 当前窗口ID
let currentWindowId = ref('')

// 虚拟滚动相关配置
const tableContainer = ref(null)

// 虚拟滚动器配置
const virtualizer = computed(() => {
  if (!tableContainer.value || data.value.length === 0) return null
  return useVirtualizer({
    count: data.value.length,
    getScrollElement: () => tableContainer.value,
    estimateSize: () => 45, // 每行的估计高度
    overscan: 10 // 预渲染的项目数
  })
})

// 复选框状态管理
const isItemSelected = (address) => selectedKeys.value.includes(address)
const toggleRowSelection = (address) => {
  const index = selectedKeys.value.indexOf(address)
  if (index >= 0) {
    selectedKeys.value.splice(index, 1)
  } else {
    selectedKeys.value.push(address)
  }
}

// 全选/取消全选
const selectAll = ref(false)
const indeterminate = computed(() => {
  const selectedCount = selectedKeys.value.length
  return selectedCount > 0 && selectedCount < data.value.length
})
const handleSelectAll = (checked) => {
  if (checked) {
    selectedKeys.value = data.value.map(item => item.address)
  } else {
    selectedKeys.value = []
  }
  selectAll.value = checked
}

// 初始化RPC列表
onBeforeMount(async () => {
  rpcOptions.value = await invoke('get_chain_list')
  if (rpcOptions.value && rpcOptions.value.length > 0) {
    rpcValue.value = rpcOptions.value[0].key
    currentRpc.value = rpcOptions.value[0]
    // 获取rpc对应的代币列表
    await rpcChange()
  }
})

onMounted(async () => {
  // 获取窗口标题和ID
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow();
      // 获取当前窗口ID
      currentWindowId.value = currentWindow.label;
      console.log('当前窗口ID:', currentWindowId.value);
      
      // 添加Tauri窗口关闭事件监听器
      await currentWindow.onCloseRequested(async (event) => {
        console.log('窗口关闭事件触发，正在停止后台操作...');
        
        // 停止余额查询操作
        if (balanceLoading.value) {
          await stopBalanceQuery();
          console.log('已停止余额查询操作');
        }
        
        console.log('窗口关闭清理完成，所有后台操作已停止');
      });
    } catch (error) {
      console.error('Error getting window info:', error);
    }
  } else {
    // 浏览器环境下设置默认ID
    currentWindowId.value = 'browser_window';
  }

  // 页面高度现在通过 CSS 自动调整，无需监听器

  // 监听余额查询更新事件
  await listen('balance_item_update', (event) => {
    const { index, item, window_id } = event.payload
    // 检查是否是本窗口的事件
    if (window_id && window_id !== currentWindowId.value) {
      return; // 不是本窗口的事件，直接返回
    }
    if (data.value[index]) {
      // 更新对应索引的数据
      Object.assign(data.value[index], item)

      // 更新进度
      const completedItems = data.value.filter(item => item.exec_status === '2' || item.exec_status === '3').length
      const totalItems = data.value.length
      progress.value = totalItems > 0 ? Number((completedItems / totalItems).toFixed(2)) : 0
    }
  })
  
  // 页面加载完成后发送事件
  nextTick(() => {
    setTimeout(() => {
      const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
      if (isTauri) {
        const currentWindow = getCurrentWindow();
        currentWindow.emit('page-loaded');
      }
    }, 50);
  });
})


// RPC变化事件
async function rpcChange() {
  coinOptions.value = await invoke("get_coin_list", { chainKey: rpcValue.value })
  if (coinOptions.value && coinOptions.value.length > 0) {
    coinValue.value = coinOptions.value[0].key
    currentCoin.value = coinOptions.value[0]
  }
  currentRpc.value = rpcOptions.value.find(item => item.key === rpcValue.value) || {}
}

// coin变化事件
async function coinChange(value) {
  currentCoin.value = coinOptions.value.filter(item => item.key === value)[0]
}

// 删除代币方法
function deleteToken() {
  if (rpcValue.value === 'starknet') {
    Notification.warning(' StarkNet 暂不支持删除代币！');
    return
  }
  if (rpcValue.value === 'okt') {
    Notification.warning(' OKT Chain 暂不支持删除代币！');
    return
  }
  if (rpcValue.value === "evmos") {
    Notification.warning(" EVMOS Chain 暂不支持删除代币！");
    return;
  }
  if (rpcValue.value === 'geth') {
    Notification.warning(' Goerli Ethereum 暂不支持删除代币！');
    return
  }
  if (rpcValue.value === 'sepolia') {
    Notification.warning(' Sepolia Ethereum 暂不支持删除代币！');
    return
  }
  if (rpcValue.value === 'scroll') {
    Notification.warning(' Scroll Alpha TestNet 暂不支持删除代币！');
    return
  }
  if (rpcValue.value === 'linea') {
    Notification.warning(' Linea MainNet 暂不支持删除代币！');
    return
  }
  if (rpcValue.value === 'base') {
    Notification.warning(' Base MainNet 暂不支持删除代币！');
    return
  }
  deleteTokenVisible.value = true
}

// 删除代币取消
function deleteTokenCancel() {
  deleteTokenVisible.value = false
}

// 删除代币确认
async function deleteTokenConfirm() {
  deleteTokenVisible.value = false
  await invoke("remove_coin", { chain: rpcValue.value, key: currentCoin.value.key }).then(() => {
    Notification.success('删除成功！');
    // 删除成功后重新获取代币列表
    rpcChange()
  }).catch(() => {
    Notification.error('删除失败！');
  })
}

// 导入事件触发
function handleAddCoinClick() {
  if (rpcValue.value === 'starknet') {
    Notification.warning(' StarkNet 暂不支持添加代币！');
    return
  }
  if (rpcValue.value === 'okt') {
    Notification.warning(' OKT Chain 暂不支持添加代币！');
    return
  }
  if (rpcValue.value === "evmos") {
    Notification.warning(" EVMOS Chain 暂不支持添加代币！");
    return;
  }
  if (rpcValue.value === 'geth') {
    Notification.warning(' Goerli Ethereum 暂不支持添加代币！');
    return
  }
  if (rpcValue.value === 'sepolia') {
    Notification.warning(' Sepolia Ethereum 暂不支持添加代币！');
    return
  }
  if (rpcValue.value === 'scroll') {
    Notification.warning(' Scroll Alpha TestNet 暂不支持添加代币！');
    return
  }
  if (rpcValue.value === 'linea') {
    Notification.warning(' Linea MainNet 暂不支持添加代币！');
    return
  }
  if (rpcValue.value === 'base') {
    Notification.warning(' Base MainNet 暂不支持添加代币！');
    return
  }
  addCoinVisible.value = true
}

// 添加代币弹窗取消
function handleAddCoinCancel() {
  addCoinVisible.value = false
}

// 添加代币核心方法 - 使用Rust后端
function addCoinFunc() {
  return new Promise(async (resolve, reject) => {
    try {
      // 直接使用Rust后端获取代币信息
      const tokenInfo = await invoke('get_token_info', {
        chain: rpcValue.value,
        contractAddress: coinAddress.value
      })
      
      let json = {
        "key": tokenInfo.symbol.toLowerCase(),
        "coin": tokenInfo.symbol,
        "type": "token",
        "contract_type": "",
        "contract_address": coinAddress.value,
        "abi": null // 使用标准ERC20 ABI
      }
      
      // 添加代币
      await invoke('add_coin', {
        chain: rpcValue.value,
        objJson: JSON.stringify(json)
      })
      
      addCoinVisible.value = false
      coinAddress.value = ''
      resolve()
    } catch (err) {
      reject('添加代币失败：' + err)
    }
  })
}

// 添加代币弹窗确认
const handleAddCoinBeforeOk = async () => {
  coinAddress.value = coinAddress.value.trim()
  if (!coinAddress.value) {
    Notification.warning('请输入代币地址！');
    return false
  }
  let flag = false
  await addCoinFunc().then(() => {
    Notification.success('添加代币成功！');
    flag = true
  }).catch(err => {
    Notification.error(err);
  })
  // 删除成功后重新获取代币列表
  rpcChange()
  return flag
}

// 清空列表
function clearData() {
  data.value = []
  Notification.success('清空列表成功！');
}

// 导入事件触发
function handleClick() {
  visible.value = true
}

// 导入弹窗关闭事件
function handleCancel() {
  // TODO 判断是否正在进行数据处理 如果进行数据处理则提示
  visible.value = false
  importText.value = ''
}

// 导入弹窗保存事件
const handleBeforeOk = () => {
  let importList = importText.value.split('\n').filter(item => item !== '')
  const total_count = importList.length
  importList = importList.filter(item => data.value.length === 0 || !data.value.find(obj => obj.address === item))
  const success_count = importList.length
  const fail_count = total_count - success_count
  data.value.push(...importList.map(item => {
    return {
      address: item,
      nonce: '',
      plat_balance: '',
      coin_balance: '',
      exec_status: '0',
      error_msg: ''
    }
  }))
  if (fail_count > 0) {
    Notification.warning({
      title: '导入完成！',
      content: `执行${total_count}条，成功${success_count}条，失败${fail_count}条！`,
    })
  } else {
    Notification.success({
      title: '导入成功！',
      content: `成功导入${total_count}条`,
    })
  }
  importText.value = ''
  return true
}

// 删除数据
function deleteItem(item) {
  if (balanceLoading.value) {
    Notification.warning('请停止或等待执行完成后再删除数据！');
    return
  }
  // 删除确认
  deleteItemModalShow(item.address)
}

// 删除数据弹窗显示
function deleteItemModalShow(address) {
  deleteItemVisible.value = true
  currentItemKey.value = address
}

// 删除item取消
function deleteItemCancel() {
  deleteItemVisible.value = false
}

// 删除item确认
async function deleteItemConfirm() {
  deleteItemVisible.value = false
  data.value = data.value.filter(obj => currentItemKey.value !== obj.address)
  Notification.success('删除成功！');
}

// 查询余额（改为使用Rust后端）
async function queryBalance() {
  if (data.value.length === 0) {
    Notification.warning('请先导入地址！');
    return
  }
  if (currentCoin.value.coin_type === 'base' || currentCoin.value.coin_type === 'token') {
    balanceLoading.value = true

    // 重置所有项目状态和进度
    data.value.forEach(item => {
      item.nonce = ''
      item.plat_balance = ''
      item.coin_balance = ''
      item.error_msg = ''
      item.exec_status = '0'
    })

    progress.value = 0

    try {
      // 使用Rust后端进行查询
      const params = {
        chain: rpcValue.value,
        coin_config: {
          coin_type: currentCoin.value.coin_type,
          contract_address: currentCoin.value.contract_address || null,
          abi: currentCoin.value.abi || null
        },
        items: data.value.map(item => ({
          address: item.address,
          private_key: item.private_key || null,
          plat_balance: null,
          coin_balance: null,
          nonce: null,
          exec_status: '0',
          error_msg: null
        })),
        only_coin_config: onlyCoin.value,
        thread_count: form.thread_count
      }

      const result = await invoke('query_balances_with_updates', { 
        params,
        windowId: currentWindowId.value 
      })

      if (result.success || result.items) {
        // 更新数据 - 无论总体是否成功，都要更新单条记录的状态
        result.items.forEach((resultItem, index) => {
          if (data.value[index]) {
            Object.assign(data.value[index], resultItem)
          }
        })

        // 确保进度条显示100%
        progress.value = 1

        // 统计成功和失败的数量
        const successCount = result.items.filter(item => item.exec_status === '2').length
        const failCount = result.items.filter(item => item.exec_status === '3').length
        const totalCount = result.items.length

        // 查询完成统计

        if (successCount === totalCount) {
          Notification.success('查询成功！')
        } else if (successCount > 0) {
          Notification.warning(`查询完成！成功 ${successCount} 条，失败 ${failCount} 条`)
        } else {
          Notification.error('查询失败：所有记录都查询失败')
        }
      } else {
        // 只有在没有返回任何结果时才设置所有项目为失败状态
        data.value.forEach(item => {
          item.exec_status = '3'
          item.error_msg = result.error_msg || '查询失败！'
        })
        progress.value = 1 // 即使失败也要显示100%完成
        Notification.error('查询失败：' + (result.error_msg || '未知错误'))
      }

    } catch (error) {
      console.error('查询失败:', error)

      // 设置所有项目为失败状态
      data.value.forEach(item => {
        item.exec_status = '3'
        item.error_msg = '查询失败！'
      })

      Notification.error('查询失败：' + error.message)
    }

    balanceLoading.value = false
  } else {
    Notification.warning('查询 coin 类型错误！');
  }
}

// 停止余额查询
async function stopBalanceQuery() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      await invoke('stop_balance_query', {
        windowId: currentWindowId.value
      });
      console.log('已发送停止查询请求到后端，窗口ID:', currentWindowId.value);
    }
  } catch (error) {
    console.error('停止查询请求失败:', error);
  }
  
  balanceLoading.value = false;
}

// 选中成功
function selectSucceeded() {
  selectedKeys.value = data.value.filter(item => item.exec_status === '2').map(item => item.address)
}

// 选中失败
function selectFailed() {
  selectedKeys.value = data.value.filter(item => item.exec_status === '3').map(item => item.address)
}

// 反选
function InvertSelection() {
  selectedKeys.value = data.value.filter(item => selectedKeys.value.indexOf(item.address) < 0).map(item => item.address)
}

function deleteSelected() {
  if (balanceLoading.value) {
    Notification.warning('请停止或等待执行完成后再删除数据！');
    return
  }
  data.value = data.value.filter(item => !selectedKeys.value.includes(item.address))
  Notification.success('删除成功')
}

function exportAllToExcel() {
  exportExcel(data.value)
}

function exportSelectToExcel() {
  const select_data = data.value.filter(item => selectedKeys.value.includes(item.address))
  exportExcel(select_data)
}

function exportExcel(target_data) {
  if (target_data.length === 0) {
    Notification.warning('无法导出空列表！');
    return
  }
  let export_data = [['地址', 'Nonce', '平台余额', '代币余额', '执行状态', '错误信息']]
  target_data.forEach(item => {
    export_data.push([item.address, item.nonce, item.plat_balance, item.coin_balance, item.exec_status, item.error_msg])
  })
  // 创建工作簿
  const workbook = xlUtils.book_new();
  // 创建工作表
  const worksheet = xlUtils.aoa_to_sheet(export_data);
  // 将工作表添加到工作簿
  xlUtils.book_append_sheet(workbook, worksheet, 'Sheet1');
  // 导出文件
  writeFile(workbook, 'balance_data.xlsx');
}

const formRef = ref(null)

// 返回首页
function goHome() {
  router.push({
    name: 'home'
  })
}

// 标题栏控制方法
async function minimizeWindow() {
  try {
    const currentWindow = getCurrentWindow()
    await currentWindow.minimize()
  } catch (error) {
    console.error('Error minimizing window:', error)
  }
}

async function maximizeWindow() {
  try {
    const currentWindow = getCurrentWindow()
    await currentWindow.toggleMaximize()
  } catch (error) {
    console.error('Error maximizing window:', error)
  }
}

async function closeWindow() {
  try {
    console.log('窗口关闭事件触发，正在停止后台操作...');
    
    // 停止余额查询操作
    if (balanceLoading.value) {
      await stopBalanceQuery();
      console.log('已停止余额查询操作');
    }
    
    const currentWindow = getCurrentWindow()
    await currentWindow.destroy()
  } catch (error) {
    console.error('Error closing window:', error)
  }
}
</script>

<template>
  <div class="title-bar">
    <div class="title-bar-text">钱包管理工具 - 余额查询</div>
    <div class="title-bar-controls">
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

  <div class="container balance"
    style="height: calc(100vh - 30px); display: flex; flex-direction: column; overflow: hidden;">
    <!-- <span class="pageTitle">余额查询</span> -->
    <!-- 工具栏 -->
    <div class="toolBar" style="flex-shrink: 0;">
      <a-button type="primary" @click="handleClick()">录入钱包地址</a-button>
      <a-divider direction="vertical" />
      <!-- 选择操作区按钮 -->
      <a-button type="outline" status="success" @click="selectSucceeded">选中成功</a-button>
      <a-button type="outline" status="danger" style="margin-left: 10px" @click="selectFailed">选中失败</a-button>
      <a-button type="outline" status="normal" style="margin-left: 10px" @click="InvertSelection">反选</a-button>
      <a-button type="primary" status="danger" style="margin-left: 10px" @click="deleteSelected">删除选中
      </a-button>
      <a-divider direction="vertical" />
      <a-button type="primary" status="success" @click="exportAllToExcel">
        <template #icon>
          <icon-download />
        </template>
        导出全表
      </a-button>
      <a-button type="outline" status="normal" style="margin-left: 10px" @click="exportSelectToExcel">
        <template #icon>
          <icon-download />
        </template>
        导出选中
      </a-button>
      <a-button v-show="false" class="goHome" type="outline" status="success" @click="goHome">
        <template #icon>
          <icon-double-left />
        </template>
        返回首页
      </a-button>
      <a-button type="outline" status="normal" style="float: right;margin-right: 10px" @click="clearData">清空列表
      </a-button>
    </div>
    <!-- 操作账号表格 -->
    <!-- 虚拟滚动表格 -->
    <div class="mainTable" style="flex: 1; overflow: hidden; display: flex; flex-direction: column; min-height: 0;">
      <!-- 表头 -->
      <div class="virtual-table-header" style="flex-shrink: 0;">
        <div class="virtual-header-cell" style="width: 40px">
          <a-checkbox v-model:checked="selectAll" :indeterminate="indeterminate"
            @change="handleSelectAll">全选</a-checkbox>
        </div>
        <div class="virtual-header-cell" style="width: 60px; text-align: center">序号</div>
        <div class="virtual-header-cell" style="flex: 2; text-align: center">钱包地址</div>
        <div class="virtual-header-cell" style="width: 80px; text-align: center">Nonce</div>
        <div class="virtual-header-cell" style="width: 120px; text-align: center">平台币余额</div>
        <div class="virtual-header-cell" style="width: 120px; text-align: center">代币余额</div>
        <div class="virtual-header-cell" style="width: 100px; text-align: center">状态</div>
        <div class="virtual-header-cell" style="flex: 1; text-align: center">错误信息</div>
        <div class="virtual-header-cell" style="width: 60px; text-align: center">操作</div>
      </div>

      <!-- 虚拟滚动容器 -->
      <div ref="tableContainer" class="virtual-table-container" style="flex: 1; overflow: auto; min-height: 400px; height: 100%;">
        <div v-if="virtualizer && data.length > 0" class="virtual-table-viewport"
          :style="{ height: `${virtualizer.value.getTotalSize()}px` }">
          <div v-for="virtualItem in virtualizer.value.getVirtualItems()" :key="virtualItem.index"
            class="virtual-table-row" :style="{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: `${virtualItem.size}px`,
              transform: `translateY(${virtualItem.start}px)`
            }" @click="rowClick(data[virtualItem.index])">
            <!-- 行内容 -->
            <div class="virtual-row-content">
              <div class="virtual-cell" style="width: 40px">
                <a-checkbox :checked="isItemSelected(data[virtualItem.index].address)"
                  @change="toggleRowSelection(data[virtualItem.index].address)"></a-checkbox>
              </div>
              <div class="virtual-cell" style="width: 60px; justify-content: center;">{{ virtualItem.index + 1 }}</div>
              <div class="virtual-cell" style="flex: 2; justify-content: center; overflow: hidden; text-overflow: ellipsis">
                {{
                  data[virtualItem.index].address }}</div>
              <div class="virtual-cell" style="width: 80px; justify-content: center;">{{ data[virtualItem.index].nonce }}
              </div>
              <div class="virtual-cell" style="width: 120px; justify-content: center;">{{ data[virtualItem.index].plat_balance
              }}
              </div>
              <div class="virtual-cell" style="width: 120px; justify-content: center;">{{ data[virtualItem.index].coin_balance
              }}
              </div>
              <div class="virtual-cell" style="width: 100px; justify-content: center;">
                <a-tag v-if="data[virtualItem.index].exec_status === '0'" color="gray">等待执行</a-tag>
                <a-tag v-if="data[virtualItem.index].exec_status === '1'" color="orange">执行中</a-tag>
                <a-tag v-if="data[virtualItem.index].exec_status === '2'" color="green">执行成功</a-tag>
                <a-tag v-if="data[virtualItem.index].exec_status === '3'" color="red">执行失败</a-tag>
              </div>
              <div class="virtual-cell" style="flex: 1; justify-content: center; overflow: hidden; text-overflow: ellipsis">
                {{
                  data[virtualItem.index].error_msg }}</div>
              <div class="virtual-cell" style="width: 60px; justify-content: center;">
                <icon-delete style="font-size: 16px; cursor: pointer;"
                  @click.stop="deleteItem(data[virtualItem.index])" />
              </div>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-if="data.length === 0" class="virtual-table-empty">
          <a-empty description="暂无数据" />
        </div>
      </div>
    </div>
    <a-progress v-if="balanceLoading" style="margin-top: 15px; flex-shrink: 0;" :percent="progress"
      :style="{ width: '100%' }" stroke-width="5" :animation="true" :color="{
        '0%': '#37ecba',
        '100%': '#009efd',
      }" />
    <!-- 链选择器和代币选择器的容器 -->
    <div style="display: flex; gap: 10px; align-items: center; margin-top: 10px; flex-shrink: 0;">
      <!-- 链选择器 -->
      <a-select v-model="rpcValue" :options="rpcOptions" @change="rpcChange" :field-names="rpcFieldNames" size="large"
        :style="{ width: '65%' }">
        <template #label="{ data }">
          <div style="
            display: flex;
            flex-direction: row;
            align-items: center;
            width: 100%;
          ">
            <span style="color: gray;">区块链：</span>
            <ChainIcon :chain-key="data?.key" :pic-data="data?.pic_data" :alt="data?.chain"
              style="width: 18px; height: 18px;" />
            <span style="margin-left: 10px">{{ data?.chain }}</span>
            <span style="margin-left: 20px;color: #c3c3c3;">{{ data?.scan_url }}</span>
          </div>
        </template>
        <template #option="{ data }">
          <div style="display: flex; flex-direction: row; align-items: center;height: 32px;">
            <ChainIcon :chain-key="data?.key" :pic-data="data?.pic_data" :alt="data?.chain"
              style="width: 18px; height: 18px;" />
            <span style="margin-left: 10px">{{ data?.chain }}</span>
            <span style="margin-left: 20px;color: #c3c3c3;">{{ data?.scan_url }}</span>
          </div>
        </template>
      </a-select>
      <!-- 代币 选择器 -->
      <a-select v-model="coinValue" :options="coinOptions" :field-names="coinFieldNames" :style="{ width: '30%' }"
        @change="coinChange">
        <template #label="{ data }">
          <span style="color: gray;">代币：</span>
          <span style="margin-left: 10px">{{ data?.label }}</span>
        </template>
        <template #option="{ data }">
          <span style="margin-left: 10px">{{ data?.label }}</span>
        </template>
      </a-select>
    </div>
    <!-- 管理代币按钮区域 -->
    <div style="display: flex; gap: 10px; align-items: center; margin-top: 10px; flex-shrink: 0;">
      <a-button type="outline" @click="handleAddCoinClick" style="white-space: nowrap;">
        添加代币
      </a-button>
      <a-button type="outline" @click="deleteToken" style="white-space: nowrap;">
        删除代币
      </a-button>
      <a-checkbox v-model="onlyCoin" style="margin-left: auto;">仅查询目标代币</a-checkbox>
    </div>
    <!-- 相关设置 -->
    <div style="display: flex;padding-top: 5px;flex-direction: column; flex-shrink: 0;">
      <div style="display: flex">
        <!-- 细节配置 -->
        <a-form ref="formRef" :model="form" layout="vertical">
          <a-row style="height: 70px">
            <a-form-item field="thread_count" label="线程数" style="width: 240px;padding: 5px 10px;"
              tooltip="同时执行查询的钱包数量（1-10）之间">
              <a-input-number :max="50" :min="1" mode="button" v-model="form.thread_count" />
            </a-form-item>
          </a-row>
        </a-form>
        <div style="width: 300px;display: flex;align-items: center;justify-content: center;">
          <a-button type="outline" status="normal" style="margin-left: 10px;height: 40px;width: 180px;font-size: 14px;"
            @click="queryBalance" :loading="balanceLoading">查询余额
          </a-button>
        </div>
      </div>
    </div>
  </div>
  <!-- 录入弹窗 -->
  <a-modal v-model:visible="visible" :width="700" title="录入钱包地址" @cancel="handleCancel" :on-before-ok="handleBeforeOk">
    <a-textarea v-model="importText" style="margin-top: 10px" placeholder="格式：一行一个" allow-clear :auto-size="{
      minRows: 15,
      maxRows: 20
    }" />
  </a-modal>
  <!-- 添加代币弹窗 -->
  <a-modal v-model:visible="addCoinVisible" :width="700" title="添加代币" @cancel="handleAddCoinCancel"
    :on-before-ok="handleAddCoinBeforeOk" unmountOnClose>
    <a-input v-model="coinAddress" placeholder="请输入代币合约地址" allow-clear />
  </a-modal>
  <!-- 删除代币确认框 -->
  <a-modal v-model:visible="deleteTokenVisible" title="删除确认">
    <div>确认删除【 {{ currentCoin.coin }} 】代币？</div>
    <template #footer>
      <a-button @click="deleteTokenCancel">取消</a-button>
      <a-button type="primary" status="danger" @click="deleteTokenConfirm" style="margin-left: 10px">确定
      </a-button>
    </template>
  </a-modal>
  <!-- 删除数据确认框 -->
  <a-modal v-model:visible="deleteItemVisible" title="删除确认">
    <div>确认删除地址为【
      {{ currentItemKey.substring(0, 15) + '......' + currentItemKey.substring(currentItemKey.length - 15) }}
      】的数据？
    </div>
    <template #footer>
      <a-button @click="deleteItemCancel">取消</a-button>
      <a-button type="primary" status="danger" @click="deleteItemConfirm" style="margin-left: 10px">确定
      </a-button>
    </template>
  </a-modal>
</template>

<style scoped lang="less">
/* 自定义标题栏 */
.title-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 30px;
  background: linear-gradient(135deg, rgba(26, 26, 46, 0.95) 0%, rgba(22, 33, 62, 0.95) 50%, rgba(15, 52, 96, 0.95) 100%);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(103, 126, 234, 0.3);
  color: white;
  padding: 0 10px;
  -webkit-app-region: drag;
  user-select: none;
  font-size: 14px;
  font-weight: 500;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
}

.title-bar-text {
  font-size: 13px;
  font-weight: 500;
  margin-left: 8px;
}

.title-bar-controls {
  display: flex;
  -webkit-app-region: no-drag;
}

.title-bar-control {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 30px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.8);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 16px;
}

.title-bar-control:hover {
  background: rgba(255, 255, 255, 0.1);
}

.title-bar-control.close:hover {
  background: #ff5555;
}

.minimize-icon {
  font-size: 14px;
}

.close-icon {
  font-size: 14px;
}

.container {
  padding: 40px 20px 20px;
  height: 100vh;
  box-sizing: border-box;
}

.pageTitle {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 20px;
  background: linear-gradient(120deg, #11c06f 0%, #165dff 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  text-fill-color: transparent;
  position: relative;
  display: inline-block;
}

.pageTitle::after {
  content: "";
  position: absolute;
  bottom: -5px;
  left: 0;
  width: 100%;
  height: 3px;
  background: linear-gradient(120deg, #11c06f 0%, #165dff 100%);
  border-radius: 3px;
}

.pageTitle::before {
  content: "💰";
  margin-right: 10px;
  user-select: none;
  text-align: start;
  line-height: 100px;
  font-size: 100px;
  background-image: linear-gradient(to bottom, #f2f3f5, #ffffff);
  -webkit-background-clip: text;
  font-weight: 600;
  height: 120px;
  right: -10px;
  border-radius: 30px;
  color: transparent;
  top: 15px;
  z-index: 0;
}

.toolBar {
  margin-top: 35px;
}

.goHome {
  float: right;
  background-color: white;
}

.goHome:hover {
  background-color: #ffffffa6;
}

.mainTable {
  margin-top: 20px;
}

.subTitle {
  font-weight: 600;
  font-size: 16px;
  margin: 10px 0 0 10px;
}

.arco-form-item {
  padding: 20px 10px 0 10px;
  margin-bottom: 10px;
}

.arco-btn-secondary.arco-btn-loading {
  color: #ffffff;
  background-color: #11c06f;
}

.arco-btn-secondary.arco-btn-loading:hover {
  color: #ffffff;
  background-color: #fc0934;
}

.arco-btn-outline.arco-btn-loading:hover {
  color: #ffffff;
  background-color: #fc0934;
  border: none;
}

.arco-radio-button.arco-radio-checked {
  color: #ffffff;
  background-color: #165dff;
}

/* Arco Design风格的表格样式 */
.virtual-table-header {
  display: flex;
  align-items: center;
  background-color: var(--color-fill-2);
  border: 1px solid var(--color-neutral-3);
  border-bottom: 2px solid var(--color-neutral-4);
  height: 45px;
  padding: 0 10px;
  font-weight: 500;
  color: var(--color-text-2);
}

.virtual-header-cell {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px;
  white-space: nowrap;
  border-right: 1px solid var(--color-neutral-3);
  font-size: 14px;
}

.virtual-header-cell:last-child {
  border-right: none;
}

.virtual-table-container {
  border: 1px solid var(--color-neutral-3);
  border-top: none;
  overflow: auto;
  position: relative;
  background: var(--color-bg-2);
}

.virtual-table-viewport {
  position: relative;
  width: 100%;
}

.virtual-table-row {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  transform: translateY(0);
  border-bottom: 1px solid var(--color-neutral-3);
}

.virtual-table-row:last-child {
  border-bottom: none;
}

.virtual-table-row:hover {
  background-color: var(--color-fill-1);
}

.virtual-row-content {
  display: flex;
  align-items: center;
  height: 100%;
  padding: 8px 10px;
  font-size: 14px;
  color: var(--color-text-1);
}

.virtual-cell {
  display: flex;
  align-items: center;
  padding: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  border-right: 1px solid var(--color-neutral-3);
}

.virtual-cell:last-child {
  border-right: none;
}

.virtual-table-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 300px;
  color: var(--color-text-3);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .container {
    padding: 40px 10px 10px;
  }
  
  .toolBar {
    flex-wrap: wrap;
    gap: 10px;
  }
  
  .toolBar > * {
    margin: 5px;
  }
  
  .virtual-header-cell,
  .virtual-cell {
    padding: 4px 6px;
    font-size: 12px;
  }
  
  .virtual-row-content {
    padding: 4px 6px;
  }
}
</style>
<style lang="less">
.balance {
  .arco-table-body {
    min-height: 150px;

    .arco-table-element .arco-empty {
      min-height: 130px;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
    }
  }
}
</style>
