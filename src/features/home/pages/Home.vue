<script setup name="home">
import { useRouter } from 'vue-router'
import { useEcosystemStore } from '@/stores/ecosystem'
import {Notification, Modal, Message} from "@arco-design/web-vue";
import { onMounted, onBeforeUnmount, ref, h, computed } from "vue";
import party from "party-js";
import { confettiStore, useThemeStore } from '@/stores'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { WINDOW_CONFIG } from '@/utils/windowNames'
import {message} from "@tauri-apps/plugin-dialog";

const router = useRouter()
const ecoStore = useEcosystemStore()
const store = confettiStore()
const themeStore = useThemeStore()
let windowCount = ref({})
let windowListObj = ref({})

// 当前激活的 Tab
const activeTab = ref('evm')

// 事件监听器引用，用于清理
let unlistenCloseEvent = null
let unlistenTrayQuitEvent = null

// 关闭确认标记位
let closeConfirmed = ref(false)

// 确认弹窗状态跟踪
let isConfirmModalVisible = ref(false)

// 调试模式相关状态
let debugMode = ref(false)

// 数据库管理相关状态
let databaseStatus = ref(null)
let databaseLoading = ref(false)

// 主题切换相关状态 - 使用computed从themeStore获取
const isDarkTheme = computed(() => themeStore.currentTheme === 'dark')

onMounted(async () => {
  // 初始化主题状态
  themeStore.initTheme()

  const newFlag = mergedFuncList.filter(item => item.isNew).length > 0
  if (newFlag && store.status) {
    // 动画效果
    party.confetti(document.getElementById('app'), {
      count: party.variation.range(100, 150),
      spread: party.variation.range(30, 80),
      size: party.variation.range(0.6, 1.3),
      colors: ['#9dbd4d', '#5a91d9', '#e8c261'],
      origin: {
        x: 0.5,
        y: 0.3
      }
    })
    // 关闭动画
    store.changeStatus(false)
  }

  // 监听主窗口关闭请求事件
  try {
    // 检查是否在Tauri环境中
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      unlistenCloseEvent = await listen('main-window-close-requested', async () => {
        await handleMainWindowCloseRequest()
      })

      // 监听托盘退出请求事件
      unlistenTrayQuitEvent = await listen('tray-quit-requested', async () => {
        await handleMainWindowCloseRequest()
      })
    }
  } catch (error) {
    console.error('Failed to listen for close event:', error)
  }

  // 注意：主窗口的显示由 SplashScreen 组件控制，这里不需要再次调用 show()
  // 启动窗口会在加载完成后自动显示主窗口并关闭自己
})

// 组件卸载时清理事件监听器
onBeforeUnmount(() => {
  if (unlistenCloseEvent) {
    unlistenCloseEvent()
  }
  if (unlistenTrayQuitEvent) {
    unlistenTrayQuitEvent()
  }
})

// 功能菜单列表 - 合并版
const mergedFuncList = [
  {
    title: "多对多转账",
    desc: "支持 EVM 和 Solana 生态的批量转账，模拟真实用户行为，降低风控风险",
    picture: "avatar/optimized/transfer.webp",
    pageName: "transfer"
  },
  {
    title: "余额查询",
    desc: "支持多链资产查询，包括原生代币和 SPL/ERC20 代币，支持导出 Excel 报表",
    picture: "avatar/optimized/balance.webp",
    pageName: "balance"
  },
  {
    title: "极速分发",
    desc: "专注单钱包分发，追求极致的分发速度与极低的gas消耗",
    isBuilding: true,
    picture: "avatar/optimized/distribution.webp",
    pageName: "distribution"
  },
  {
    title: "链上地址分析监控",
    desc: "监控钱包余额与Nonce变化（目前仅支持 EVM）",
    isBuilding: true,
    picture: "avatar/optimized/monitor.webp",
    pageName: "monitor"
  },
]

const airdropFuncList = [
  {
    title: "浏览器自动化",
    desc: "使用 Playwright + 钱包并发执行交互任务 (抗检测环境)",
    picture: "avatar/optimized/monitor.webp",
    pageName: "airdrop-browser",
    isNew: true
  }
]

// 跳转逻辑
function goPage(pageName) {

  const targetModule = [...mergedFuncList, ...airdropFuncList].find(item => item.pageName === pageName);
  if (targetModule?.isBuilding) {
    Message.warning('功能建设中，敬请期待')
    return
  }

  // 检查是否在Tauri环境中
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (!isTauri) {
    // 浏览器环境：跳转到入口页
    router.push(`/entry?target=${pageName}`)
    return
  }

  try {
    // 正确实现多窗口
    const count = windowCount.value[pageName] ?? 0
    windowCount.value[pageName] = count + 1
    const newCount = windowCount.value[pageName]
    if (!windowListObj.value[pageName]) {
      windowListObj.value[pageName] = new Map()
    }
    const windowLabel = WINDOW_CONFIG.generateLabel(pageName, newCount)
    
    // 修改：指向 entry 页面，而不是具体的 eth/sol 页面
    const windowUrl = pageName === 'airdrop-browser'
      ? `/#/airdrop/browser?count=${newCount}`
      : `/#/entry?target=${pageName}&count=${newCount}`
    
    // 生成窗口标题：统一格式 "WalletsTool - {图标} {功能名} [{序号}]"
    const moduleIcons = { transfer: '💸', balance: '💰', monitor: '👁️', 'airdrop-browser': '🤖' }
    const moduleNames = { transfer: '批量转账', balance: '余额查询', monitor: '链上监控', 'airdrop-browser': '浏览器自动化' }
    const title = newCount > 1
      ? `WalletsTool - ${moduleIcons[pageName] || ''} ${moduleNames[pageName] || pageName} [${newCount}]`
      : `WalletsTool - ${moduleIcons[pageName] || ''} ${moduleNames[pageName] || pageName}`

    const webview = new WebviewWindow(windowLabel, {
      url: windowUrl,
      width: 1350,
      height: 900,
      title: title,
      resizable: true,
      center: true,
      decorations: false,
      backgroundColor: document.documentElement.getAttribute('data-theme') === 'light' ? '#FFFFFF' : '#2A2A2B',
    })

    windowListObj.value[pageName].set(windowLabel, webview)

    webview.once('tauri://created', function () {
      // Window created successfully
      // 延迟显示窗口，等待页面加载
      setTimeout(() => {
        webview.show()
      }, 100)
    })

    webview.once('tauri://close-requested', function (event) {
      // 在 Tauri 2.x 中，需要手动关闭窗口
      webview.close()
    })

    webview.once('tauri://destroyed', function (event) {
      windowListObj.value[pageName].delete(event.windowLabel)
      if (windowListObj.value[pageName].size === 0) {
        windowCount.value[pageName] = 0
      }
    })

    webview.once('tauri://error', function (e) {
      console.error('Window creation error:', e)
    })

    // 监听页面加载完成事件
    webview.listen('page-loaded', () => {
      webview.show()
    })

  } catch (error) {
    console.error('Error in goPage:', error)
  }
}

// 切换调试模式
function toggleDebugMode() {
  debugMode.value = !debugMode.value
  if (debugMode.value) {
    Notification.success({ content: '调试模式开启', position: 'topLeft' })
  } else {
    Notification.error({ content: '调试模式关闭', position: 'topLeft' })
  }
}

// 切换主题
function toggleTheme() {
  themeStore.toggleTheme()
  // if (isDarkTheme.value) {
  //   Notification.success({ content: '已切换到暗黑主题', position: 'topLeft' })
  // } else {
  //   Notification.success({ content: '已切换到明亮主题', position: 'topLeft' })
  // }
}

// 检查数据库状态
async function checkDatabaseStatus() {
  try {
    databaseLoading.value = true
    let status
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
    if (isTauri) {
      status = await invoke('check_database_schema')
    } else {
      // 浏览器环境下模拟正常状态
      status = {
        db_exists: true,
        chains_table_exists: true,
        tokens_table_exists: true,
        rpc_table_exists: true,
        abi_column_exists: true,
        contract_type_column_exists: true,
        needs_migration: false
      }
    }

    // 将状态对象转换为友好的中文描述
    let statusText = ''
    let notificationType = 'success'

    if (typeof status === 'object' && status !== null) {
      // 根据新的检查逻辑生成状态文本
      if (!status.db_exists) {
        statusText = '❌ 数据库文件不存在'
        notificationType = 'error'
      } else if (!(status.chains_table_exists && status.tokens_table_exists && status.rpc_table_exists)) {
        const missingTables = []
        if (!status.chains_table_exists) missingTables.push('链表(chains)')
        if (!status.tokens_table_exists) missingTables.push('代币表(tokens)')
        if (!status.rpc_table_exists) missingTables.push('RPC表(rpc_providers)')
        statusText = `❌ 数据库缺少必要表：\n${missingTables.join('\n')}`
        notificationType = 'error'
      } else if (status.needs_migration) {
        statusText = '⚠️ 数据库需要迁移更新'
        notificationType = 'warning'
      } else {
        statusText = '✅ 数据库结构完整，运行正常'
      }
    } else {
      statusText = typeof status === 'string' ? status : JSON.stringify(status)
    }

    databaseStatus.value = statusText

    if (notificationType === 'success') {
      Notification.success({ 
        title: '数据库状态检查完成',
        content: statusText
      , position: 'topLeft' })
    } else if (notificationType === 'warning') {
      Notification.warning({ 
        title: '数据库状态检查完成',
        content: statusText
      , position: 'topLeft' })
    } else {
      Notification.error({ 
        title: '数据库状态检查完成',
        content: statusText
      , position: 'topLeft' })
    }

    // 数据库状态检查完成
  } catch (error) {
    console.error('检查数据库状态失败:', error)
    const errorText = typeof error === 'string' ? error : error.message || '未知错误'
    databaseStatus.value = '检查失败: ' + errorText
    Notification.error({ 
      title: '检查数据库状态失败',
      content: errorText
    , position: 'topLeft' })
  } finally {
    databaseLoading.value = false
  }
}

// 重载数据库
async function reloadDatabase() {
  try {
    databaseLoading.value = true
    let result
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
    if (isTauri) {
      result = await invoke('reload_database')
    } else {
      // 浏览器环境下模拟成功
      result = '数据库重载成功'
    }

    // 确保result是字符串格式
    const resultText = typeof result === 'string' ? result : JSON.stringify(result)

    Notification.success({ 
      title: '数据库重载完成',
      content: resultText
    , position: 'topLeft' })

    // 重新检查数据库状态
    setTimeout(async () => {
      await checkDatabaseStatus()
    }, 500)

  } catch (error) {
    console.error('重载数据库失败:', error)
    const errorText = typeof error === 'string' ? error : error.message || '未知错误'
    Notification.error({ 
      title: '重载数据库失败',
      content: errorText
    , position: 'topLeft' })
  } finally {
    databaseLoading.value = false
  }
}

// 刷新页面数据
async function refreshPageData() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
    if (isTauri) {
      // 在Home页面，主要是刷新一些基础数据
      // 可以根据需要添加更多刷新逻辑
    }

    // 重置数据库状态
    databaseStatus.value = null

    Notification.success({ 
      title: '页面数据已刷新',
      content: '所有状态已重置'
    , position: 'topLeft' })

    // 自动重新检查数据库状态
    setTimeout(async () => {
      await checkDatabaseStatus()
    }, 300)

  } catch (error) {
    console.error('刷新页面数据失败:', error)
    const errorText = typeof error === 'string' ? error : error.message || '未知错误'
    Notification.error({ 
      title: '刷新页面数据失败',
      content: errorText
    , position: 'topLeft' })
  }
}

// 导出数据库数据到init.sql
async function exportDatabaseToInitSql() {
  try {
    databaseLoading.value = true
    let result
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
    if (isTauri) {
      result = await invoke('export_database_to_init_sql')
    } else {
      // 浏览器环境下模拟成功
      result = '数据库导出成功（浏览器环境模拟）'
    }

    // 确保result是字符串格式
    const resultText = typeof result === 'string' ? result : JSON.stringify(result)

    Notification.success({ 
      title: '数据库导出完成',
      content: resultText
    , position: 'topLeft' })

    // 数据库导出完成

  } catch (error) {
    console.error('导出数据库失败:', error)
    const errorText = typeof error === 'string' ? error : error.message || '未知错误'
    Notification.error({ 
      title: '导出数据库失败',
      content: errorText
    , position: 'topLeft' })
  } finally {
    databaseLoading.value = false
  }
}

// 标题栏控制方法
async function minimizeWindow() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      const currentWindow = getCurrentWindow()
      await currentWindow.minimize()
    }
  } catch (error) {
    console.error('Error minimizing window:', error)
  }
}

// 最小化到托盘
async function minimizeToTray() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      const currentWindow = getCurrentWindow()
      await currentWindow.hide()
    }
  } catch (error) {
    console.error('Error minimizing to tray:', error)
  }
}

async function closeWindow() {
  try {
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri) {
      // 调用确认关闭函数而不是直接关闭窗口
      await handleMainWindowCloseRequest()
    }
  } catch (error) {
    console.error('Error closing window:', error)
  }
}

// 清除所有代理配置缓存
async function clearAllProxyConfigs() {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  
  // 清除前端localStorage
  const keysToRemove = [];
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key && (key.startsWith('proxy_config_') || key.startsWith('proxy_window_id_'))) {
      keysToRemove.push(key);
    }
  }
  keysToRemove.forEach(key => {
    localStorage.removeItem(key);
    console.log(`已清除缓存: ${key}`);
  });
  console.log(`已清除 ${keysToRemove.length} 个代理配置缓存`);
  
  // 清除后端文件缓存和内存缓存
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow();
      await invoke('clear_proxy_config_for_window', { windowId: currentWindow.label });
      console.log(`已清除窗口 ${currentWindow.label} 的后端代理配置`);
    } catch (error) {
      console.error('清除后端代理配置失败:', error);
    }
  }
}

// 处理主窗口关闭请求
async function handleMainWindowCloseRequest() {
  try {
    // 检查是否在Tauri环境中
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (!isTauri) {
      return true
    }

    // 检查是否已有确认弹窗显示，避免重复弹窗
    if (isConfirmModalVisible.value) {
      return false
    }

    // 检查关闭确认标记位
    if (closeConfirmed.value) {
      // 如果已经确认过，直接关闭
      // 关闭确认已存在，直接关闭主窗口
      // await invoke('force_close_main_window')
      return true
    }

    // 先获取所有子窗口
    const childWindows = await invoke('get_all_child_windows', {
      mainWindowLabel: 'main'
    })

    // 获取子窗口列表

    let confirmMessage = '确定要关闭应用程序吗？'
    if (childWindows && childWindows.length > 0) {
      confirmMessage = `当前还有 ${childWindows.length} 个子窗口正在运行，关闭主窗口将关闭所有窗口。确定要继续吗？`
    }



    // 设置弹窗状态为显示中
    isConfirmModalVisible.value = true

    // 显示确认对话框
    Modal.confirm({
      title: '确认关闭',
      content: confirmMessage,
      okText: '确定',
      cancelText: '取消',
      width: 250, // 设置较小的宽度
      okButtonProps: {
        status: 'danger'
      },
      onOk: () => {
        return new Promise(async (resolve, reject) => {
          try {
            // 开始关闭应用程序

            // 设置关闭确认标记位
            closeConfirmed.value = true

            // 清除所有代理配置缓存
            await clearAllProxyConfigs()

            // 先关闭所有子窗口
            if (childWindows && childWindows.length > 0) {
              // 正在关闭子窗口
              await invoke('close_all_child_windows', {
                mainWindowLabel: 'main'
              })
              // 已关闭子窗口

              // 给子窗口一些时间完全关闭
              await new Promise(resolveTimeout => setTimeout(resolveTimeout, 500))
            }

            // 最后强制关闭主窗口避免循环
            await invoke('force_close_main_window')

            resolve(true) // 操作成功

          } catch (error) {
            console.error('关闭窗口时发生错误:', error)
            // 发生错误时重置标记位
            closeConfirmed.value = false
            isConfirmModalVisible.value = false
            Notification.error({ 
              title: '错误',
              content: '关闭窗口时发生错误，请重试'
            , position: 'topLeft' })
            reject(false) // 操作失败
          } finally {
            // 无论成功还是失败，都重置弹窗状态
            isConfirmModalVisible.value = false
          }
        })
      },
      onCancel: () => {
        // 用户取消关闭操作
        // 取消时重置标记位
        closeConfirmed.value = false
        isConfirmModalVisible.value = false
      }
    })

  } catch (error) {
    console.error('处理窗口关闭请求时发生错误:', error)

    // 设置弹窗状态为显示中
    isConfirmModalVisible.value = true

    // 如果出现错误，显示简单的确认对话框
    Modal.confirm({
      title: '确认关闭',
      content: '确定要关闭应用程序吗？',
      okText: '确定',
      cancelText: '取消',
      width: 420, // 设置较小的宽度
      okButtonProps: {
        status: 'danger'
      },
      onOk: () => {
        return new Promise(async (resolve, reject) => {
          try {
            // 使用强制关闭命令避免循环
            // 强制关闭主窗口
            await invoke('force_close_main_window')
            resolve(true) // 操作成功
          } catch (closeError) {
            console.error('强制关闭窗口时发生错误:', closeError)
            Notification.error({ 
              title: '错误',
              content: '强制关闭窗口时发生错误，请重试'
            , position: 'topLeft' })
            reject(false) // 操作失败
          } finally {
            // 无论成功还是失败，都重置弹窗状态
            isConfirmModalVisible.value = false
          }
        })
      },
      onCancel: () => {
        // 用户取消关闭操作时重置弹窗状态
        isConfirmModalVisible.value = false
      }
    })
  }
}
</script>

<template>
  <div class="container home" :class="{ 'light-theme': !isDarkTheme }">
    <!-- 自定义标题栏 -->
    <div class="custom-titlebar">
      <div class="titlebar-content">
        <div class="titlebar-left" data-tauri-drag-region>
          <img src="/app-icon.png" alt="App Logo" class="app-icon" />
          <!-- <span class="app-title">钱包管理工具</span> -->
        </div>
        <div class="titlebar-drag-area" data-tauri-drag-region></div>

        <!-- 偏左侧的主题切换区域 -->
        <div class="titlebar-center">
          <div class="theme-toggle-container">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
              class="theme-icon">
              <circle cx="12" cy="12" r="5" />
              <path
                d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
            </svg>
            <a-switch v-model="isDarkTheme" @change="toggleTheme" size="small" class="theme-switch" />
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
              class="theme-icon">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
            </svg>
          </div>
        </div>

        <div class="titlebar-right">
          <button class="titlebar-btn minimize-tray-btn" @click="minimizeToTray" title="最小化到托盘">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="2" y="18" width="20" height="3" rx="1" />
              <path d="M8 14l4 4 4-4" stroke-linecap="round" stroke-linejoin="round" />
              <path d="M12 3v11" stroke-linecap="round" />
            </svg>
          </button>
          <button class="titlebar-btn minimize-btn" @click="minimizeWindow" title="最小化">
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path d="M2 6h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
          <button class="titlebar-btn close-btn" @click="closeWindow" title="关闭">
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- 背景装饰 -->
    <div class="bg-decoration">
      <div class="bg-circle bg-circle-1"></div>
      <div class="bg-circle bg-circle-2"></div>
      <div class="bg-circle bg-circle-3"></div>
      <div class="bg-gradient"></div>
    </div>

    <!-- 标题区域 -->
    <div class="header-section">
      <div class="funcListTitle">
        <span class="title-text">功能列表</span>
        <div class="title-underline"></div>
      </div>
      <div class="subtitle">探索最强大的web3转账工具</div>
    </div>
    
    <a-tabs default-active-key="assets" class="custom-tabs" animation>
      <a-tab-pane key="assets">
        <template #title>
          <span style="display: flex; align-items: center; gap: 6px; font-size: 16px;">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="7" width="20" height="14" rx="2" ry="2"></rect>
              <line x1="12" y1="12" x2="12" y2="12"></line>
              <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16"></path>
            </svg>
            资产管理
          </span>
        </template>
        
        <!-- 统一功能列表 -->
        <div class="func-grid">
          <div class="func-card" :class="{
            'func-card--disabled': item.isBuilding,
            'func-card--new': item.isNew
          }" @click="goPage(item.pageName)" v-for="(item, idx) in mergedFuncList" :key="idx"
            :style="{ '--delay': idx * 0.1 + 's' }">
            <!-- 新功能标识 -->
            <div v-if="item.isNew" class="new-badge">
              <span>NEW</span>
            </div>

            <!-- 建设中标识 -->
            <div v-if="item.isBuilding" class="building-badge">
              <span>建设中</span>
            </div>

            <!-- 卡片内容 -->
            <div class="card-content">
              <div class="card-icon">
                <img :src="item.picture" alt="功能图标" />
              </div>

              <div class="card-info">
                <h3 class="card-title">{{ item.title }}</h3>
                <p class="card-desc">{{ item.desc }}</p>
              </div>
            </div>

            <!-- 卡片底部装饰 -->
            <div class="card-footer">
              <div class="card-arrow">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M5 12h14M12 5l7 7-7 7" />
                </svg>
              </div>
            </div>

            <!-- 悬浮效果 -->
            <div class="card-hover-effect"></div>
          </div>
        </div>
      </a-tab-pane>

      <a-tab-pane key="airdrop">
        <template #title>
          <span style="display: flex; align-items: center; gap: 6px; font-size: 16px;">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
              <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
              <line x1="12" y1="22.08" x2="12" y2="12"></line>
            </svg>
            空投交互
          </span>
        </template>
        
        <div class="func-grid">
          <div class="func-card" :class="{
            'func-card--disabled': item.isBuilding,
            'func-card--new': item.isNew
          }" @click="goPage(item.pageName)" v-for="(item, idx) in airdropFuncList" :key="idx"
            :style="{ '--delay': idx * 0.1 + 's' }">
            
            <div v-if="item.isNew" class="new-badge"><span>NEW</span></div>
            <div v-if="item.isBuilding" class="building-badge"><span>建设中</span></div>

            <div class="card-content">
              <div class="card-icon">
                <img :src="item.picture" alt="功能图标" />
              </div>

              <div class="card-info">
                <h3 class="card-title">{{ item.title }}</h3>
                <p class="card-desc">{{ item.desc }}</p>
              </div>
            </div>

            <div class="card-footer">
              <div class="card-arrow">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M5 12h14M12 5l7 7-7 7" />
                </svg>
              </div>
            </div>
            <div class="card-hover-effect"></div>
          </div>
        </div>
      </a-tab-pane>
    </a-tabs>

    <!-- 调试模式区域 -->
    <div class="debug-area">
      <!-- 调试模式切换按钮 -->
      <div class="debug-toggle" @click="toggleDebugMode" title="调试">
        <span class="debug-icon">🔧</span>
      </div>

      <!-- 数据库管理面板 -->
      <div v-if="debugMode" class="database-panel">
        <div class="panel-header">
          <span class="panel-title">数据库管理</span>
          <span v-if="databaseStatus" class="status-indicator"
            :class="{ 'status-ok': databaseStatus.includes('valid') }">
            {{ databaseStatus.includes('valid') ? '✓' : '⚠' }}
          </span>
        </div>
        <div class="panel-actions">
          <a-button size="small" type="outline" @click="checkDatabaseStatus" :loading="databaseLoading"
            class="action-btn">
            检查状态
          </a-button>
          <a-button size="small" type="outline" @click="reloadDatabase" :loading="databaseLoading" class="action-btn">
            重载数据库
          </a-button>
          <a-button size="small" type="outline" @click="refreshPageData" class="action-btn">
            刷新页面
          </a-button>
          <a-button size="small" type="outline" @click="exportDatabaseToInitSql" :loading="databaseLoading"
            class="action-btn">
            导出数据库
          </a-button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.coming-soon-container {
  display: flex;
  flex-direction: column;
  align-items: center;
}

/* 自定义标题栏 */
.custom-titlebar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 40px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  z-index: 1000;
  user-select: none;
}

.titlebar-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 16px;
  position: relative;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-icon {
  width: 24px;
  height: 24px;
  margin-right: 8px;
  border-radius: 4px;
  object-fit: contain;
}

.app-title {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
  letter-spacing: 0.5px;
}

.titlebar-drag-area {
  flex: 1;
  height: 100%;
  min-width: 100px;
}

.titlebar-center {
  position: absolute;
  left: 46%;
  top: 50%;
  transform: translate(-50%, -50%);
  z-index: 1002;
}

.theme-toggle-container {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  padding: 6px 12px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.theme-icon {
  color: rgba(255, 255, 255, 0.7);
  transition: color 0.2s ease;
}

.theme-toggle-container:hover .theme-icon {
  color: rgba(255, 255, 255, 0.9);
}

.theme-switch {
  margin: 0 4px;
}

.minimize-tray-btn {
  background: rgba(255, 255, 255, 0.08) !important;
}

.minimize-tray-btn:hover {
  background: rgba(255, 255, 255, 0.15) !important;
  color: rgba(255, 255, 255, 0.9);
}

.titlebar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.titlebar-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  color: rgba(255, 255, 255, 0.7);
  /* 确保按钮不阻止拖拽 */
  position: relative;
  z-index: 1001;
}

.titlebar-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.9);
}

.theme-btn {
  transition: all 0.3s ease;
}

.theme-btn:hover {
  transform: rotate(180deg);
}

.close-btn:hover {
  background: rgba(255, 96, 96, 0.8);
  color: white;
}

.minimize-btn:hover {
  background: rgba(255, 206, 84, 0.8);
  color: white;
}

.minimize-tray-btn:hover {
  background: rgba(34, 197, 94, 0.8) !important;
  color: white !important;
}

/* 主容器 */
.container {
  position: relative;
  min-height: 100vh;
  height: 100vh;
  padding: 140px 0 0;
  background: var(--bg-gradient);
  overflow: hidden;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
}

/* 背景装饰 */
.bg-decoration {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  pointer-events: none;
}

.bg-circle {
  position: absolute;
  border-radius: 50%;
  background: rgba(103, 126, 234, 0.08);
  animation: float 6s ease-in-out infinite;
}

.bg-circle-1 {
  width: 200px;
  height: 200px;
  top: 10%;
  left: 10%;
  animation-delay: 0s;
}

.bg-circle-2 {
  width: 150px;
  height: 150px;
  top: 60%;
  right: 15%;
  animation-delay: 2s;
  background: rgba(118, 75, 162, 0.08);
}

.bg-circle-3 {
  width: 100px;
  height: 100px;
  bottom: 10%;
  left: 20%;
  animation-delay: 4s;
  background: rgba(52, 152, 219, 0.08);
}

.bg-gradient {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(45deg,
      rgba(103, 126, 234, 0.05) 0%,
      rgba(118, 75, 162, 0.03) 50%,
      rgba(52, 152, 219, 0.05) 100%);
  opacity: 0.6;
}

/* 标题区域 */
.header-section {
  position: relative;
  z-index: 1;
  text-align: center;
  margin-bottom: 20px;
  animation: slideInDown 0.8s ease-out;
}

.funcListTitle {
  position: relative;
  display: inline-block;
  margin-bottom: 12px;
}

.title-text {
  font-size: 32px;
  font-weight: 700;
  color: #fff;
  text-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
  letter-spacing: 2px;
}

.title-underline {
  position: absolute;
  bottom: -8px;
  left: 50%;
  transform: translateX(-50%);
  width: 60px;
  height: 4px;
  background: linear-gradient(90deg, #586cc7, #764ba2, #f093fb);
  border-radius: 2px;
  animation: expandWidth 0.8s ease-out 0.3s both;
}

.subtitle {
  font-size: 16px;
  color: rgba(255, 255, 255, 0.9);
  font-weight: 400;
  margin-top: 5px;
}

/* 功能网格 */
.func-grid {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
  max-width: 1200px;
  margin: 0 auto;
  /* padding: 0 15px; */
}

/* 功能卡片 */
.func-card {
  position: relative;
  background: rgb(53 56 61);
  border-radius: 16px;
  padding: 15px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(102, 126, 234, 0.2);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  animation: slideInUp 0.6s ease-out both;
  animation-delay: var(--delay, 0s);
  overflow: hidden;
}

.func-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(90deg, #586cc7, #764ba2, #f093fb);
  transform: scaleX(0);
  transition: transform 0.3s ease;
}

.func-card:hover::before {
  transform: scaleX(1);
}

.func-card:hover {
  transform: translateY(-8px);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.25);
}

.func-card--disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.func-card--disabled:hover {
  transform: none;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

/* 徽章 */
.new-badge,
.building-badge {
  position: absolute;
  top: 16px;
  right: 16px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 600;
  z-index: 2;
  animation: pulse 2s ease-in-out infinite;
}

.new-badge {
  background: linear-gradient(45deg, #ff6b6b, #ee5a24);
  color: white;
  box-shadow: 0 4px 12px rgba(255, 107, 107, 0.3);
}

.building-badge {
  background: linear-gradient(45deg, #ffa726, #ff9800);
  color: white;
  box-shadow: 0 4px 12px rgba(255, 167, 38, 0.3);
}

/* 卡片内容 */
.card-content {
  display: flex;
  align-items: center;
  gap: 10px;
}

.card-icon {
  width: 50px;
  height: 50px;
  background: linear-gradient(135deg, #586cc7 0%, #764ba2 100%);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 16px rgba(103, 126, 234, 0.3);
  transition: transform 0.3s ease;
}

.func-card:hover .card-icon {
  transform: scale(1.1) rotate(5deg);
}

.card-icon img {
  width: 24px;
  height: 24px;
  object-fit: contain;
  filter: brightness(0) invert(1);
}

.card-info {
  flex: 1;
}

.card-title {
  font-size: 18px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  margin: 0 0 8px 0;
  line-height: 1.3;
}

.func-card--disabled .card-title {
  color: rgba(255, 255, 255, 0.4);
}

.card-desc {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.7);
  line-height: 1.5;
  margin: 0;
}

.func-card--disabled .card-desc {
  color: rgba(255, 255, 255, 0.3);
}

/* 卡片底部 */
.card-footer {
  display: flex;
  justify-content: flex-end;
  align-items: center;
}

.card-arrow {
  width: 20px;
  height: 20px;
  color: #586cc7;
  transition: transform 0.3s ease;
}

.func-card:hover .card-arrow {
  transform: translateX(4px);
}

.func-card--disabled .card-arrow {
  color: #ccc;
}

/* 悬浮效果 */
.card-hover-effect {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(45deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.05));
  opacity: 0;
  transition: opacity 0.3s ease;
  pointer-events: none;
  border-radius: 16px;
}

.func-card:hover .card-hover-effect {
  opacity: 1;
}

/* 动画 */
@keyframes float {

  0%,
  100% {
    transform: translateY(0px);
  }

  50% {
    transform: translateY(-20px);
  }
}

@keyframes slideInDown {
  from {
    opacity: 0;
    transform: translateY(-30px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes slideInUp {
  from {
    opacity: 0;
    transform: translateY(30px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes expandWidth {
  from {
    width: 0;
  }

  to {
    width: 60px;
  }
}

@keyframes pulse {

  0%,
  100% {
    transform: scale(1);
  }

  50% {
    transform: scale(1.05);
  }
}

/* 调试区域样式 */
.debug-area {
  position: fixed;
  bottom: 15px;
  right: 15px;
  z-index: 1000;
}

.debug-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  padding: 0;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.3s ease;
  backdrop-filter: blur(10px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  font-size: 12px;
  color: #666;
}

.debug-toggle:hover {
  background: rgba(255, 255, 255, 0.95);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.debug-icon {
  font-size: 16px;
}

.database-panel {
  position: absolute;
  bottom: 50px;
  right: 0;
  min-width: 280px;
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 12px;
  padding: 16px;
  backdrop-filter: blur(10px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  animation: slideUp 0.3s ease;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.status-indicator {
  font-size: 16px;
  font-weight: bold;
}

.status-indicator.status-ok {
  color: #52c41a;
}

.status-indicator:not(.status-ok) {
  color: #ff4d4f;
}

.panel-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.action-btn {
  width: 100%;
  font-size: 12px;
  height: 28px;
  border-radius: 6px;
}

.action-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
}

/* 明亮主题样式 */
.light-theme {
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 50%, #e0eafc 100%) !important;
}

.light-theme .custom-titlebar {
  background: rgba(255, 255, 255, 0.7) !important;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1) !important;
}

.light-theme .app-title {
  color: rgba(0, 0, 0, 0.8) !important;
}

.light-theme .titlebar-btn {
  background: rgba(0, 0, 0, 0.05) !important;
  color: rgba(0, 0, 0, 0.7) !important;
}

.light-theme .theme-toggle-container {
  background: rgba(0, 0, 0, 0.05) !important;
  border: 1px solid rgba(0, 0, 0, 0.1) !important;
}

.light-theme .theme-icon {
  color: rgba(0, 0, 0, 0.7) !important;
}

.light-theme .theme-toggle-container:hover .theme-icon {
  color: rgba(0, 0, 0, 0.9) !important;
}

.light-theme .minimize-tray-btn {
  background: rgba(0, 0, 0, 0.08) !important;
}

.light-theme .minimize-tray-btn:hover {
  background: rgba(0, 0, 0, 0.15) !important;
  color: rgba(0, 0, 0, 0.9) !important;
}

.light-theme .titlebar-btn:hover {
  background: rgba(0, 0, 0, 0.1) !important;
  color: rgba(0, 0, 0, 0.9) !important;
}

.light-theme .close-btn:hover {
  background: rgba(255, 96, 96, 0.8) !important;
  color: white !important;
}

.light-theme .minimize-btn:hover {
  background: rgba(255, 206, 84, 0.8) !important;
  color: white !important;
}

.light-theme .minimize-tray-btn:hover {
  background: rgba(34, 197, 94, 0.8) !important;
  color: white !important;
}

.light-theme .bg-circle {
  background: rgba(103, 126, 234, 0.1) !important;
}

.light-theme .bg-circle-2 {
  background: rgba(118, 75, 162, 0.1) !important;
}

.light-theme .bg-circle-3 {
  background: rgba(52, 152, 219, 0.1) !important;
}

.light-theme .bg-gradient {
  background: linear-gradient(45deg,
      rgba(103, 126, 234, 0.08) 0%,
      rgba(118, 75, 162, 0.05) 50%,
      rgba(52, 152, 219, 0.08) 100%) !important;
}

.light-theme .title-text {
  color: #2c3e50 !important;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1) !important;
}

.light-theme .subtitle {
  color: rgba(0, 0, 0, 0.7) !important;
}

.light-theme .func-card {
  background: rgba(255, 255, 255, 0.9) !important;
  border: 1px solid rgba(0, 0, 0, 0.1) !important;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1) !important;
}

.light-theme .func-card:hover {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15) !important;
}

.light-theme .card-title {
  color: #586cc7 !important;
}

.light-theme .card-desc {
  color: rgb(115 116 119) !important;
}

.light-theme .func-card--disabled .card-title {
  color: rgba(0, 0, 0, 0.4) !important;
}

.light-theme .func-card--disabled .card-desc {
  color: rgba(0, 0, 0, 0.3) !important;
}

.light-theme .card-arrow {
  color: #586cc7 !important;
}

.light-theme .func-card--disabled .card-arrow {
  color: #999 !important;
}

.light-theme .debug-toggle {
  background: rgba(255, 255, 255, 0.95) !important;
  border: 1px solid rgba(0, 0, 0, 0.1) !important;
  color: #666 !important;
}

.light-theme .debug-toggle:hover {
  background: rgba(255, 255, 255, 1) !important;
}

.light-theme .database-panel {
  background: rgba(255, 255, 255, 0.98) !important;
  border: 1px solid rgba(0, 0, 0, 0.1) !important;
}

.light-theme .panel-title {
  color: #2c3e50 !important;
}

/* Tabs 样式美化 */
.custom-tabs {
  width: 95%;
  margin: 0 auto;
  background: transparent;
  backdrop-filter: blur(20px);
  border-radius: 16px;
  padding: 8px;
  transition: all 0.3s ease;
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* Tab 头部样式 */
.custom-tabs :deep(.arco-tabs-nav) {
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
  padding: 4px;
  margin-bottom: 5px;
}

.custom-tabs :deep(.arco-tabs-tab) {
  background: transparent;
  border: none;
  border-radius: 8px;
  margin: 0 4px;
  padding: 12px 20px;
  color: rgba(255, 255, 255, 0.7);
  font-weight: 500;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.custom-tabs :deep(.arco-tabs-tab::before) {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, rgba(103, 126, 234, 0.1), rgba(118, 75, 162, 0.1));
  opacity: 0;
  transition: opacity 0.3s ease;
  border-radius: 8px;
}

.custom-tabs :deep(.arco-tabs-tab:hover) {
  color: rgba(255, 255, 255, 0.9);
  transform: translateY(-1px);
  box-shadow: 0 4px 16px rgba(103, 126, 234, 0.2);
}

.custom-tabs :deep(.arco-tabs-tab:hover::before) {
  opacity: 1;
}

.custom-tabs :deep(.arco-tabs-tab-active) {
  background: linear-gradient(135deg, rgba(103, 126, 234, 0.2), rgba(118, 75, 162, 0.2));
  color: rgba(255, 255, 255, 1);
  box-shadow: 0 4px 20px rgba(103, 126, 234, 0.3);
  transform: translateY(-1px);
}

.custom-tabs :deep(.arco-tabs-tab-active::before) {
  opacity: 1;
}

/* Tab 内容区域 */
.custom-tabs :deep(.arco-tabs-content) {
  padding: 0;
  flex-grow: 1;
  overflow-y: auto;
  min-height: 0;
  /* 隐藏滚动条但保持滚动功能 */
  scrollbar-width: none;
  /* Firefox */
  -ms-overflow-style: none;
  /* IE/Edge */
}

/* 隐藏 Webkit 浏览器的滚动条 */
.custom-tabs :deep(.arco-tabs-content::-webkit-scrollbar) {
  display: none;
}

.custom-tabs :deep(.arco-tabs-pane) {
  animation: fadeInUp 0.4s ease-out;
}

/* Tab 指示器隐藏 */
.custom-tabs :deep(.arco-tabs-ink-bar) {
  display: none;
}

.light-theme .custom-tabs :deep(.arco-tabs-nav) {
  background: rgb(249 250 252 / 88%) !important;
}

.light-theme .custom-tabs :deep(.arco-tabs-tab) {
  color: rgba(0, 0, 0, 0.7) !important;
}

.light-theme .custom-tabs :deep(.arco-tabs-tab::before) {
  background: linear-gradient(135deg, rgba(103, 126, 234, 0.08), rgba(118, 75, 162, 0.08)) !important;
}

.light-theme .custom-tabs :deep(.arco-tabs-tab:hover) {
  color: rgba(0, 0, 0, 0.9) !important;
  box-shadow: 0 4px 16px rgba(103, 126, 234, 0.15) !important;
}

.light-theme .custom-tabs :deep(.arco-tabs-tab-active) {
  background: linear-gradient(135deg, rgba(103, 126, 234, 0.15), rgba(118, 75, 162, 0.15)) !important;
  color: #586cc7 !important;
  box-shadow: 0 4px 20px rgba(103, 126, 234, 0.2) !important;
}

/* 淡入动画 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .custom-tabs {
    width: 98%;
    padding: 6px;
  }

  .custom-tabs :deep(.arco-tabs-tab) {
    padding: 5px 15px;
    font-size: 15px;
  }
}

@media (max-width: 768px) {
  .container {
    padding: 65px 0 0 0;
  }

  .title-text {
    font-size: 24px;
  }

  .func-grid {
    grid-template-columns: 1fr;
    gap: 10px;
  }

  .func-card {
    padding: 15px 15px 10px 15px;
  }
}
</style>
<style>
/* 全局样式 - 确保主页无滚动条 */
body {
  overflow: hidden;
}

#app {
  overflow: hidden;
}

.home {
  overflow: hidden !important;

  .arco-tabs-nav-ink {
    display: none !important;
  }

  .arco-tabs-nav::before {
    display: none !important;
  }
}
</style>