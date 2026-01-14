// 窗口命名使用示例
import { WINDOW_CONFIG } from '@/utils/windowNames';

// 示例1: 从Home页面打开功能窗口时
function openFunctionWindow(pageName) {
  const count = windowCount.value[pageName] ?? 0;
  const title = WINDOW_CONFIG.generateTitle(pageName, null, count + 1);
  
  new WebviewWindow(windowLabel, {
    title: title,
    // ...
  });
}

// 示例2: 批量转账多开窗口
async function openMultipleTransferWindows() {
  const walletGroups = ['热钱包', '冷钱包', '交易所'];
  
  for (let i = 0; i < walletGroups.length; i++) {
    const group = walletGroups[i];
    const windowId = Date.now() + i;
    const title = WINDOW_CONFIG.generateTitle('transfer', group, i + 1);
    
    new WebviewWindow(`transfer_multi_${windowId}`, {
      title: title,
      // ...
    });
  }
}

// 示例3: 根据实际业务场景生成窗口标题
function getWindowTitleForScenario(scenario) {
  const scenarios = {
    largeTransfer: '批量转账 | 大额地址',
    airdropCheck: '余额查询 | 空投检查',
    monitorNew: '链上监控 | 新地址',
    hotWallet: '批量转账 | 热钱包',
    coldWallet: '批量转账 | 冷钱包'
  };
  
  const label = scenarios[scenario] || '未分类';
  return WINDOW_CONFIG.generateTitle('transfer', label);
}

// 示例4: 动态更新窗口标题
async function updateWindowTitleWithStatus(windowId, status) {
  const baseTitle = WINDOW_CONFIG.generateTitle('transfer', '热钱包', 1);
  const statusText = {
    pending: '(待执行)',
    running: '(执行中)',
    completed: '(已完成)',
    paused: '(已暂停)'
  };
  
  const finalTitle = baseTitle + (statusText[status] || '');
  await getCurrentWindow().setTitle(finalTitle);
}

// 示例5: 托盘菜单创建窗口
async function createWindowFromTray(pageName) {
  const existingLabels = getExistingWindowLabels(pageName);
  const count = existingLabels.length + 1;
  
  const title = WINDOW_CONFIG.generateTrayTitle(pageName, count);
  
  new WebviewWindow(WINDOW_CONFIG.generateLabel(pageName, count), {
    title: title,
    // ...
  });
}

// 示例6: 批量操作时自动分配业务标识
function autoAssignBusinessLabels(pageName, windowCount) {
  const module = WINDOW_CONFIG.modules[pageName];
  const existingLabels = [];
  const titles = [];
  
  for (let i = 0; i < windowCount; i++) {
    const label = WINDOW_CONFIG.suggestBusinessLabel(pageName, existingLabels);
    existingLabels.push(label);
    titles.push(WINDOW_CONFIG.generateTitle(pageName, label, i + 1));
  }
  
  return titles;
}
