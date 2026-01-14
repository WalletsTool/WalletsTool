# 自定义窗口名称功能

本文档描述了 WalletsTool 中自定义窗口名称功能的使用方法和技术细节。

## 功能概述

从版本 2.0 开始，WalletsTool 支持用户自定义窗口名称，便于在多开窗口时区分和管理。

## 使用方法

### 1. 重命名窗口

**操作步骤：**
1. 将鼠标移动到标题栏的窗口名称上
2. 鼠标指针会变成光标，显示可编辑状态
3. 点击窗口名称进入编辑模式
4. 输入新的窗口名称
5. 按 `Enter` 键保存，或按 `Escape` 键取消
6. 失去焦点也会自动保存

**示例：**
```
原始名称: WalletsTool - 💸 批量转账
重命名后: 热钱包转账 - A组
```

### 2. 恢复默认名称

**操作步骤：**
1. 将鼠标移动到窗口名称上
2. 看到黄色星标（★）表示已自定义名称
3. 鼠标悬停时会出现"恢复默认"按钮（↺）
4. 点击按钮恢复为默认名称

### 3. 多窗口场景

当打开多个相同功能的窗口时，自定义名称可以快速区分：

```
窗口 1: 热钱包转账 - A组
窗口 2: 热钱包转账 - B组  
窗口 3: 冷钱包转账
```

## 技术实现

### 存储机制

自定义窗口名称存储在浏览器的 `localStorage` 中：

```javascript
// 存储键名
'wallets_tool_window_titles'

// 存储格式
{
  "transfer1": "热钱包转账 - A组",
  "transfer2": "冷钱包转账",
  "balance1": "VIP地址检查"
}
```

### 窗口标签

每个窗口通过唯一的标签（label）来识别：

- 主窗口: `main`
- 功能窗口: `{pageName}{count}` (如 `transfer1`, `balance2`)
- 多开窗口: `{parentLabel}_multi_{timestamp}`

### API 函数

`src/utils/windowNames.js` 提供了以下函数：

```javascript
// 保存自定义标题
WINDOW_CONFIG.saveCustomTitle(windowLabel, title)

// 获取自定义标题
WINDOW_CONFIG.getCustomTitle(windowLabel)

// 删除自定义标题
WINDOW_CONFIG.removeCustomTitle(windowLabel)

// 获取所有自定义标题
WINDOW_CONFIG.getAllCustomTitles()

// 清除所有自定义标题
WINDOW_CONFIG.clearAllCustomTitles()

// 获取标题统计信息
WINDOW_CONFIG.getTitleStats()
```

## 限制

1. **名称长度**: 最多 50 个字符
2. **特殊字符**: 支持所有字符，但建议使用常规字符
3. **持久化**: 关闭应用后自定义名称仍然保留
4. **多设备**: 不支持跨设备同步

## 最佳实践

### 推荐的命名格式

**转账窗口:**
```
热钱包转账 - A组
冷钱包归集
交易所分发
批量转账 - ETH主网
```

**余额查询窗口:**
```
VIP地址检查
代币持仓分布
大额地址监控
```

**监控窗口:**
```
新地址监控 - 24h
大额交易监控
异常地址预警
```

## 常见问题

### Q: 为什么有些窗口没有重命名功能？
A: 所有功能页面（转账、余额查询、链上监控）都支持自定义名称。

### Q: 自定义名称会丢失吗？
A: 不会。自定义名称保存在 `localStorage` 中，关闭应用后仍然保留。

### Q: 可以批量重命名吗？
A: 目前不支持批量操作，需要逐个窗口重命名。

### Q: 如何查看所有自定义名称？
A: 可以在浏览器控制台执行：
```javascript
console.log(JSON.parse(localStorage.getItem('wallets_tool_window_titles')))
```

## 与 Tauri 的集成

使用 Tauri 2.9.5 的窗口 API 实现：

```javascript
// 获取当前窗口实例
const currentWindow = getCurrentWindow()

// 动态设置窗口标题
await currentWindow.setTitle('新的窗口名称')

// 获取当前窗口标签
const label = currentWindow.label
```

## 文件修改记录

| 文件 | 修改内容 |
|------|---------|
| `src/utils/windowNames.js` | 添加自定义标题存储和管理函数 |
| `src/components/TitleBar.vue` | 添加原地编辑、恢复默认功能 |
| `src/features/ethereum/transfer/pages/Transfer.vue` | 集成自定义标题 |
| `src/features/ethereum/balance/pages/Balance.vue` | 集成自定义标题 |
| `src/features/ethereum/monitor/pages/Monitor.vue` | 集成自定义标题 |
| `src/features/solana/transfer/pages/Transfer.vue` | 集成自定义标题 |
| `src/features/solana/balance/pages/Balance.vue` | 集成自定义标题 |

## 未来改进

1. 快捷键支持：双击快速重命名
2. 名称模板：保存和重用常用名称
3. 批量管理：多窗口名称批量设置
4. 导入导出：跨设备同步名称配置
5. 名称建议：基于窗口内容智能建议名称
