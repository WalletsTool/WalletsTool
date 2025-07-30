<script setup name="home">
import {useRouter} from 'vue-router'
import {Notification, Modal} from "@arco-design/web-vue";
import {onMounted, onBeforeUnmount, ref, h, computed} from "vue";
import party from "party-js";
import {confettiStore, useThemeStore} from '@/stores'
import {WebviewWindow} from '@tauri-apps/api/webviewWindow'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {invoke} from '@tauri-apps/api/core'
import {listen} from '@tauri-apps/api/event'

const router = useRouter()
const store = confettiStore()
const themeStore = useThemeStore()
let windowCount = ref({})
let windowListObj = ref({})

// 事件监听器引用，用于清理
let unlistenCloseEvent = null

// 关闭确认标记位
let closeConfirmed = ref(false)

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
  
  const newFlag = funcList.filter(item => item.isNew).length > 0
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
  }
  } catch (error) {
    console.error('Failed to listen for close event:', error)
  }
})

// 组件卸载时清理事件监听器
onBeforeUnmount(() => {
  if (unlistenCloseEvent) {
    unlistenCloseEvent()
  }
})

// 功能菜单列表
const funcList = [
  {
    title: "批量转账",
    desc: "支持多条链上的钱包转账，支持多对多转账（持续更新中）",
    picture: "avatar/transfer.png",
    pageName: "transfer"
  },
  {
    title: "余额查询",
    desc: "支持多条链上的余额查询（持续更新中）",
    picture: "avatar/balance.png",
    pageName: "balance"
  },
  {
    title: "链上地址监控",
    isBuilding: true,
    desc: "支持多条链上的地址监测（建设中）",
    picture: "avatar/monitor.png",
    pageName: "monitor"
  },
  // {
  //   title: "Uniswap批量交易",
  //   isBuilding: true,
  //   desc: "支持 Uniswap  V3 交易（建设中）",
  //   picture: "avatar/uniswap.png",
  //   pageName: "uniswap"
  // }
]

// 跳转到批量转账
function goPage(pageName) {
  if (pageName === 'monitor' || pageName === 'uniswap') {
    Notification.success('功能建设中，敬请期待')
    return
  }
  
  // 检查是否在Tauri环境中
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
  if (!isTauri) {
    // 在浏览器环境中，使用路由跳转
    router.push(`/${pageName}`)
    return
  }
  
  try {
    // 正确实现多窗口
    const count = windowCount.value[pageName] ?? 0
    windowCount.value[pageName] = count + 1
    if (!windowListObj.value[pageName]) {
      windowListObj.value[pageName] = new Map()
    }
    const title = funcList.filter(item => item.pageName === pageName)[0].title
    const windowLabel = pageName + windowCount.value[pageName]
    const windowUrl = `/#/${pageName}`
    
    const webview = new WebviewWindow(windowLabel, {
      url: windowUrl,
      width: 1275,
      height: 850,
      title: `▶ 窗口 ${windowCount.value[pageName]} 🧡 ${title}`,
      resizable: true,
      center: true,
      decorations: false,  // 移除Windows原生窗口边框
      backgroundColor: '#1a1a2e'  // 设置窗口背景色
    })
    
    windowListObj.value[pageName].set(windowLabel, webview)

    webview.once('tauri://created', function () {
      // Window created successfully
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
    
  } catch (error) {
    console.error('Error in goPage:', error)
  }
}

// 切换调试模式
function toggleDebugMode() {
  debugMode.value = !debugMode.value
  if (debugMode.value) {
    Notification.success('调试模式开启')
  } else {
    Notification.error('调试模式关闭')
  }
}

// 切换主题
function toggleTheme() {
  themeStore.toggleTheme()
  // if (isDarkTheme.value) {
  //   Notification.success('已切换到暗黑主题')
  // } else {
  //   Notification.success('已切换到明亮主题')
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
      })
    } else if (notificationType === 'warning') {
      Notification.warning({
        title: '数据库状态检查完成',
        content: statusText
      })
    } else {
      Notification.error({
        title: '数据库状态检查完成',
        content: statusText
      })
    }
    
    // 数据库状态检查完成
  } catch (error) {
    console.error('检查数据库状态失败:', error)
    const errorText = typeof error === 'string' ? error : error.message || '未知错误'
    databaseStatus.value = '检查失败: ' + errorText
    Notification.error({
      title: '检查数据库状态失败',
      content: errorText
    })
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
    })
    
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
    })
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
    })
    
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
    })
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
    })
    
    // 数据库导出完成
    
  } catch (error) {
    console.error('导出数据库失败:', error)
    const errorText = typeof error === 'string' ? error : error.message || '未知错误'
    Notification.error({
      title: '导出数据库失败',
      content: errorText
    })
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

// 处理主窗口关闭请求
async function handleMainWindowCloseRequest() {
  try {
    // 检查是否在Tauri环境中
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (!isTauri) {
      return true
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
      mainWindowLabel: 'wallet_manager'
    })
    
    // 获取子窗口列表
    
    let confirmMessage = '确定要关闭应用程序吗？'
    if (childWindows && childWindows.length > 0) {
      confirmMessage = `当前还有 ${childWindows.length} 个子窗口正在运行，关闭主窗口将关闭所有窗口。确定要继续吗？`
    }
    


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
            
            // 先关闭所有子窗口
            if (childWindows && childWindows.length > 0) {
              // 正在关闭子窗口
              const closedWindows = await invoke('close_all_child_windows', {
                mainWindowLabel: 'wallet_manager'
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
            Notification.error({
              title: '错误',
              content: '关闭窗口时发生错误，请重试'
            })
            reject(false) // 操作失败
          }
        })
      },
      onCancel: () => {
        // 用户取消关闭操作
        // 取消时重置标记位
        closeConfirmed.value = false
      }
    })
    
  } catch (error) {
    console.error('处理窗口关闭请求时发生错误:', error)
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
            })
            reject(false) // 操作失败
          }
        })
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
          <div class="app-icon"></div>
          <!-- <span class="app-title">钱包管理工具</span> -->
        </div>
        <div class="titlebar-drag-area" data-tauri-drag-region></div>
        <div class="titlebar-right">
          <button class="titlebar-btn theme-btn" @click="toggleTheme" :title="isDarkTheme ? '切换到明亮主题' : '切换到暗黑主题'">
            <svg v-if="isDarkTheme" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="5"/>
              <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
            </svg>
            <svg v-else width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
            </svg>
          </button>
          <button class="titlebar-btn minimize-btn" @click="minimizeWindow">
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path d="M2 6h8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
          <button class="titlebar-btn close-btn" @click="closeWindow">
            <svg width="12" height="12" viewBox="0 0 12 12">
              <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
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
      <div class="subtitle">探索强大的Web3工具集合</div>
    </div>
    
    <!-- 功能卡片网格 -->
    <div class="func-grid">
      <div 
        class="func-card" 
        :class="{
          'func-card--disabled': item.isBuilding,
          'func-card--new': item.isNew
        }"
        @click="goPage(item.pageName)" 
        v-for="(item, idx) in funcList" 
        :key="idx"
        :style="{ '--delay': idx * 0.1 + 's' }"
      >
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
              <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
          </div>
        </div>
        
        <!-- 悬浮效果 -->
        <div class="card-hover-effect"></div>
      </div>
    </div>
    
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
          <span v-if="databaseStatus" class="status-indicator" :class="{ 'status-ok': databaseStatus.includes('valid') }">
            {{ databaseStatus.includes('valid') ? '✓' : '⚠' }}
          </span>
        </div>
        <div class="panel-actions">
          <a-button 
            size="small" 
            type="outline" 
            @click="checkDatabaseStatus" 
            :loading="databaseLoading"
            class="action-btn"
          >
            检查状态
          </a-button>
          <a-button 
            size="small" 
            type="outline" 
            @click="reloadDatabase" 
            :loading="databaseLoading"
            class="action-btn"
          >
            重载数据库
          </a-button>
          <a-button 
            size="small" 
            type="outline" 
            @click="refreshPageData"
            class="action-btn"
          >
            刷新页面
          </a-button>
          <a-button 
            size="small" 
            type="outline" 
            @click="exportDatabaseToInitSql" 
            :loading="databaseLoading"
            class="action-btn"
          >
            导出数据库
          </a-button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
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
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-icon {
  width: 20px;
  height: 20px;
  background-image: url('/src-tauri/icons/32x32.png');
  background-size: contain;
  background-repeat: no-repeat;
  background-position: center;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
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

/* 主容器 */
.container {
  position: relative;
  min-height: 100vh;
  height: 100vh;
  padding: 140px 0 0;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
  overflow: hidden;
  box-sizing: border-box;
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
  background: linear-gradient(90deg, #667eea, #764ba2, #f093fb);
  border-radius: 2px;
  animation: expandWidth 0.8s ease-out 0.3s both;
}

.subtitle {
  font-size: 16px;
  color: rgba(255, 255, 255, 0.9);
  font-weight: 400;
  margin-top: 8px;
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
  padding: 0 15px;
}

/* 功能卡片 */
.func-card {
  position: relative;
  background: rgba(30, 42, 78, 0.85);
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
  background: linear-gradient(90deg, #667eea, #764ba2, #f093fb);
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
  align-items: flex-start;
  gap: 10px;
}

.card-icon {
  width: 60px;
  height: 60px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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
  width: 36px;
  height: 36px;
  object-fit: contain;
  filter: brightness(0) invert(1);
}

.card-info {
  flex: 1;
}

.card-title {
  font-size: 20px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  margin: 0 0 8px 0;
  line-height: 1.3;
}

.func-card--disabled .card-title {
  color: rgba(255, 255, 255, 0.4);
}

.card-desc {
  font-size: 14px;
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
  width: 24px;
  height: 24px;
  color: #667eea;
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
  0%, 100% {
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
  0%, 100% {
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
  background: rgba(255, 255, 255, 0.9) !important;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1) !important;
}

.light-theme .app-title {
  color: rgba(0, 0, 0, 0.8) !important;
}

.light-theme .titlebar-btn {
  background: rgba(0, 0, 0, 0.05) !important;
  color: rgba(0, 0, 0, 0.7) !important;
}

.light-theme .titlebar-btn:hover {
  background: rgba(0, 0, 0, 0.1) !important;
  color: rgba(0, 0, 0, 0.9) !important;
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
  color: #2c3e50 !important;
}

.light-theme .card-desc {
  color: rgba(0, 0, 0, 0.6) !important;
}

.light-theme .func-card--disabled .card-title {
  color: rgba(0, 0, 0, 0.4) !important;
}

.light-theme .func-card--disabled .card-desc {
  color: rgba(0, 0, 0, 0.3) !important;
}

.light-theme .card-arrow {
  color: #667eea !important;
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

/* 响应式设计 */
@media (max-width: 768px) {
  .container {
    padding: 60px 0 0 0;
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
}

/* 全局覆盖 */
.arco-notification {
  max-width: 320px !important;
  width: 320px !important;
}
</style>