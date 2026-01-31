# 浏览器自动化功能设计文档

## 1. 概述

### 1.1 功能目标
完善浏览器自动化页面功能，实现以下核心能力：
- 指定执行钱包
- 选择执行脚本
- 制定执行频率（一次性/周期执行）
- 监控任务执行情况
- 脚本自定义API（连接钱包、签名等）
- 预处理器将自定义方法转换为完整Playwright脚本

### 1.2 整体架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Browser Automation System                      │
├─────────────────────────────────────────────────────────────────────┤
│  Frontend (Vue 3)                                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ 任务创建页面 │  │ 任务监控面板 │  │ 脚本编辑器  │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
│                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ 钱包选择器   │  │ 定时调度配置 │  │ 日志查看器  │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
├─────────────────────────────────────────────────────────────────────┤
│  Backend (Tauri/Rust + Node.js Bridge)                               │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │                   Task Scheduler                            │     │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │     │
│  │  │ Cron Parser │ │ Queue Manager│ │ Execution Engine   │   │     │
│  │  └─────────────┘ └─────────────┘ └─────────────────────┘   │     │
│  └────────────────────────────────────────────────────────────┘     │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │                Script Preprocessor                          │     │
│  │  - API Method Resolution                                    │     │
│  │  - Template Injection                                       │     │
│  │  - Validation                                               │     │
│  └────────────────────────────────────────────────────────────┘     │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │              Playwright Bridge (Node.js)                    │     │
│  │  - Browser Farm Management                                  │     │
│  │  - Wallet Injection (MetaMask/OKX)                          │     │
│  │  - Fingerprint Protection                                   │     │
│  └────────────────────────────────────────────────────────────┘     │
├─────────────────────────────────────────────────────────────────────┤
│  Database (SQLite)                                                   │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ Tables: automation_tasks, automation_scripts, task_logs,   │     │
│  │         task_executions, browser_profiles, wallet_groups   │     │
│  └────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. 数据库设计

### 2.1 表结构

#### 2.1.1 自动化任务表 (`automation_tasks`)

```sql
CREATE TABLE automation_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    wallet_group_id INTEGER,
    script_id INTEGER NOT NULL,
    browser_profile_id INTEGER,

    -- 调度配置
    schedule_type VARCHAR(32) NOT NULL DEFAULT 'once',  -- once, interval, cron
    schedule_config TEXT NOT NULL,  -- JSON: {"interval": 3600} or {"cron": "0 * * * *"}

    -- 执行配置
    concurrency INTEGER DEFAULT 1,  -- 并发执行数量
    timeout_seconds INTEGER DEFAULT 300,
    retry_times INTEGER DEFAULT 3,
    retry_interval_seconds INTEGER DEFAULT 60,

    -- 状态
    status VARCHAR(32) NOT NULL DEFAULT 'draft',  -- draft, enabled, paused, running
    last_run_time DATETIME,
    next_run_time DATETIME,
    total_runs INTEGER DEFAULT 0,
    success_runs INTEGER DEFAULT 0,
    failed_runs INTEGER DEFAULT 0,

    -- 通知配置
    notify_on_success BOOLEAN DEFAULT FALSE,
    notify_on_failure BOOLEAN DEFAULT TRUE,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (wallet_group_id) REFERENCES wallet_groups(id),
    FOREIGN KEY (script_id) REFERENCES automation_scripts(id),
    FOREIGN KEY (browser_profile_id) REFERENCES browser_profiles(id)
);

CREATE INDEX idx_tasks_status ON automation_tasks(status);
CREATE INDEX idx_tasks_next_run ON automation_tasks(next_run_time) WHERE status = 'enabled';
```

#### 2.1.2 自动化脚本表 (`automation_scripts`)

```sql
CREATE TABLE automation_scripts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    content TEXT NOT NULL,  -- 原始脚本内容（含自定义API）
    compiled_content TEXT,  -- 预处理后的完整脚本
    version INTEGER DEFAULT 1,
    is_system BOOLEAN DEFAULT FALSE,  -- 系统内置脚本

    -- API依赖
    required_apis TEXT,  -- JSON: ["connectWallet", "signMessage", "approveToken"]

    -- 元数据
    author VARCHAR(255),
    tags TEXT,  -- JSON: ["uniswap", "okx", "airdrop"]

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_scripts_tags ON automation_scripts(tags);
```

#### 2.1.3 任务执行记录表 (`task_executions`)

```sql
CREATE TABLE task_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    wallet_address VARCHAR(255) NOT NULL,
    wallet_group_name VARCHAR(255),

    -- 执行状态
    status VARCHAR(32) NOT NULL,  -- pending, running, success, failed, stopped
    start_time DATETIME,
    end_time DATETIME,
    duration_ms INTEGER,

    -- 结果
    error_message TEXT,
    result_data TEXT,  -- JSON: {"txHash": "...", "data": {...}}

    -- 日志引用
    log_file_path VARCHAR(512),

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (task_id) REFERENCES automation_tasks(id)
);

CREATE INDEX idx_executions_task ON task_executions(task_id);
CREATE INDEX idx_executions_status ON task_executions(status);
CREATE INDEX idx_executions_time ON task_executions(start_time DESC);
```

#### 2.1.4 浏览器环境配置表 (`browser_profiles`)

```sql
CREATE TABLE browser_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- 浏览器配置
    user_agent TEXT,
    viewport_width INTEGER DEFAULT 1920,
    viewport_height INTEGER DEFAULT 1080,
    device_scale_factor DECIMAL(3,1) DEFAULT 1.0,
    locale VARCHAR(16) DEFAULT 'en-US',
    timezone_id VARCHAR(64) DEFAULT 'America/New_York',

    -- 代理配置
    proxy_type VARCHAR(32) DEFAULT 'direct',  -- direct, http, socks5
    proxy_host VARCHAR(255),
    proxy_port INTEGER,
    proxy_username VARCHAR(255),
    proxy_password VARCHAR(255),

    -- 指纹保护
    canvas_fingerprinting_protection BOOLEAN DEFAULT TRUE,
    webgl_rendering_spoofing BOOLEAN DEFAULT TRUE,
    audio_context_noise BOOLEAN DEFAULT TRUE,
    navigator_properties_override BOOLEAN DEFAULT TRUE,
    webdriver_property_override BOOLEAN DEFAULT TRUE,

    -- 其他配置
    headless BOOLEAN DEFAULT FALSE,
    extensions TEXT,  -- JSON: ["metamask", "okxwallet"]

    is_default BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 3. 前端设计

### 3.1 任务创建页面

#### 3.1.1 布局结构

```
┌─────────────────────────────────────────────────────────────────────┐
│  任务管理                                                            │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ [任务列表]  [+ 新建任务]                                    │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  基本信息                                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │ 任务名称 *    [________________________________]    │    │    │
│  │  │ 任务描述    [________________________________]       │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │  钱包配置                                                    │    │
│ ┌─────────────────────────────────────────────────────┐    │  │     │
│  │  │ 选择方式: (○) 全部钱包  ( ) 指定分组  ( ) 指定钱包  │    │    │
│  │  │                                                    │    │    │
│  │  │  分组选择: [Default Group v]                        │    │    │
│  │  │                                                    │    │    │
│  │  │  钱包列表:                                          │    │    │
│  │  │  [✓] 0x742d...f44e (Main Wallet) - Group A        │    │    │
│  │  │  [✓] 0x123d...abc (Airdrop 1) - Group A           │    │    │
│  │  │  [ ] 0x456d...def (Airdrop 2) - Group B           │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │  脚本选择                                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │ 脚本库: [Search...                          ] [v]   │    │    │
│  │  │                                                    │    │    │
│  │  │  ┌──────────────────────────────────────────────┐  │    │    │
│  │  │  │ OKX Daily Claim (v1.2)                       │  │    │    │
│  │  │  │ Uniswap V3 Swap (v2.0)                       │  │    │    │
│  │  │  │ LayerZero Bridge (v1.0)                      │  │    │    │
│  │  │  └──────────────────────────────────────────────┘  │    │    │
│  │  │                                                    │    │    │
│  │  │  [预览脚本]  [编辑脚本]                            │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │  执行计划                                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │ 执行方式:                                            │    │    │
│  │  │  (○) 立即执行  ( ) 定时执行  ( ) 循环执行          │    │    │
│  │  │                                                    │    │    │
│  │  │  循环配置:                                          │    │    │
│  │  │  间隔 [____] 秒  最大执行 [____] 次  无限循环 [ ]  │    │    │
│  │  │                                                    │    │    │
│  │  │  或者 Cron表达式: [____]                           │    │    │
│  │  │  示例: 0 0 * * * (每天凌晨)                        │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │  环境配置                                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │ 浏览器配置: [Default Profile v]  [+] 新建配置       │    │    │
│  │  │                                                    │    │    │
│  │  │  [✓] 使用代理  [✓] 指纹保护  [✓] 扩展注入          │    │    │
│  │  │                                                    │    │    │
│  │  │  注入扩展: [✓] MetaMask  [✓] OKX Wallet           │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │  高级选项                                                    │    │
│  │  ┌─────────────────────────────────────────────────────┐    │    │
│  │  │ 超时时间: [____] 秒  重试次数: [____]               │    │    │
│  │  │ 重试间隔: [____] 秒  并发数: [____]                 │    │    │
│  │  │                                                    │    │    │
│  │  │ 通知: [✓] 执行失败时通知                           │    │    │
│  │  └─────────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │  [取消]  [保存为草稿]  [保存并启用]                        │     │
│  └────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 任务监控页面

#### 3.2.1 布局结构

```
┌─────────────────────────────────────────────────────────────────────┐
│  任务监控                                                            │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ 总任务: 12  运行中: 2  成功: 156  失败: 3                  │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ 任务列表                                    [刷新] [批量操作]│     │
│  │ ┌────────────────────────────────────────────────────────┐│     │
│  │ │ 名称              状态     下次执行    执行次数  操作  ││     │
│  │ ├────────────────────────────────────────────────────────┤│     │
│  │ │ OKX Daily Claim   运行中   --          45/45     [详情]││     │
│  │ │ Uniswap Swap      已暂停   14:30:00    12/12     [启动]││     │
│  │ │ LayerZero Bridge  已启用   2024-01-15  0/∞       [停止]││     │
│  │ │ Airdrop Batch     已完成   --          100/100   [重置]││     │
│  │ └────────────────────────────────────────────────────────┘│     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ 执行详情 - OKX Daily Claim                                  │     │
│  │ [所有钱包 v]  总进度: 45/45 (100%)  ████████████████████   │     │
│  │                                                              │     │
│  │ ┌────────────────────────────────────────────────────────┐  │     │
│  │ │ 0x742d...f44e  [✓] 成功  耗时: 12.5s  交易: 0xabc...  │  │     │
│  │ │ 0x123d...abc   [✓] 成功  耗时: 11.8s  交易: 0xdef...  │  │     │
│  │ │ 0x456d...def   [✓] 成功  耗时: 13.2s  交易: 0xghi...  │  │     │
│  │ │ 0x789d...jkl   [✓] 成功  耗时: 10.5s  交易: 0xmno...  │  │     │
│  │ │ 0x321d...pqr   [运行中] ...                           │  │     │
│  │ │ 0x654d...stu   [等待中] 等待执行...                   │  │     │
│  │ └────────────────────────────────────────────────────────┘  │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ 实时日志                                                    │     │
│  │ [清空]                                                       │     │
│  │ [14:25:32] [INFO] 任务开始执行: OKX Daily Claim           │     │
│  │ [14:25:33] [INFO] 初始化浏览器环境...                     │     │
│  │ [14:25:34] [INFO] 注入 MetaMask 扩展 v10.33.1             │     │
│  │ [14:25:35] [INFO] 加载脚本: OKX Daily Claim v1.2          │     │
│  │ [14:25:36] [INFO] 开始执行钱包: 0x742d...f44e             │     │
│  │ [14:25:38] [SUCCESS] 0x742d...f44e 执行成功               │     │
│  │ [14:25:39] [INFO] 开始执行钱包: 0x123d...abc              │     │
│  │ [14:25:41] [SUCCESS] 0x123d...abc 执行成功               │     │
│  │ ...                                                         │     │
│  └────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. 脚本自定义API设计

### 4.1 API方法列表

#### 4.1.1 钱包连接类

```javascript
/**
 * 连接 MetaMask 钱包
 * @param {Object} options - 连接选项
 * @param {string} options.expectedChainId - 期望的网络ID (如 '0x1' for Ethereum Mainnet)
 * @returns {Promise<string>} 连接后的钱包地址
 */
async function connectMetaMask(options = {}) {
    // 实现逻辑
}

/**
 * 连接 OKX Wallet
 * @param {Object} options - 连接选项
 * @param {string} options.chainId - 链ID
 * @returns {Promise<string>} 连接后的钱包地址
 */
async function connectOKXWallet(options = {}) {
    // 实现逻辑
}

/**
 * 切换钱包网络
 * @param {string} chainId - 目标网络ID (十六进制字符串)
 * @returns {Promise<boolean>} 是否切换成功
 */
async function switchNetwork(chainId) {
    // 实现逻辑
}

/**
 * 获取当前连接的钱包地址
 * @returns {Promise<string>} 当前地址
 */
async function getCurrentAddress() {
    // 实现逻辑
}
```

#### 4.1.2 签名类

```javascript
/**
 * 签名消息
 * @param {string} message - 要签名的消息
 * @param {Object} options - 选项
 * @returns {Promise<string>} 签名结果 (hex格式)
 */
async function signMessage(message, options = {}) {
    // 实现逻辑
}

/**
 * 签名交易
 * @param {Object} tx - 交易对象
 * @param {string} tx.to - 目标地址
 * @param {string} tx.value - 转账金额 (wei/lamports)
 * @param {string} tx.data - 可选数据
 * @returns {Promise<string>} 签名后的交易
 */
async function signTransaction(tx) {
    // 实现逻辑
}

/**
 * 签名类型数据 (EIP-712)
 * @param {Object} domain - EIP-712 Domain
 * @param {Object} types - 类型定义
 * @param {Object} value - 要签名的值
 * @returns {Promise<string>} 签名结果
 */
async function signTypedData(domain, types, value) {
    // 实现逻辑
}
```

#### 4.1.3 交易类

```javascript
/**
 * 发送原生币转账
 * @param {string} to - 接收地址
 * @param {string} amount - 金额 (可读格式，如 '0.1 ETH')
 * @param {Object} options - 选项
 * @returns {Promise<Object>} 交易结果 {hash, status}
 */
async function sendNativeTransfer(to, amount, options = {}) {
    // 实现逻辑
}

/**
 * ERC-20 Token 授权
 * @param {string} tokenAddress - Token合约地址
 * @param {string} spender - 授权给谁
 * @param {string} amount - 授权数量 (可读格式，如 '1000 USDC')
 * @returns {Promise<Object>} 交易结果
 */
async function approveToken(tokenAddress, spender, amount) {
    // 实现逻辑
}

/**
 * ERC-20 Token 转账
 * @param {string} tokenAddress - Token合约地址
 * @param {string} to - 接收地址
 * @param {string} amount - 转账数量
 * @returns {Promise<Object>} 交易结果
 */
async function transferToken(tokenAddress, to, amount) {
    // 实现逻辑
}

/**
 * 等待交易确认
 * @param {string} txHash - 交易哈希
 * @param {number} confirmations - 确认数 (默认1)
 * @returns {Promise<Object>} 交易收据
 */
async function waitForTransaction(txHash, confirmations = 1) {
    // 实现逻辑
}
```

#### 4.1.4 浏览器操作类

```javascript
/**
 * 等待元素出现
 * @param {string} selector - CSS选择器
 * @param {number} timeout - 超时时间 (毫秒)
 * @returns {Promise<Element>} 元素
 */
async function waitForSelector(selector, timeout = 30000) {
    // 实现逻辑
}

/**
 * 等待元素消失
 * @param {string} selector - CSS选择器
 * @param {number} timeout - 超时时间 (毫秒)
 * @returns {Promise<boolean>} 是否成功消失
 */
async function waitForSelectorHidden(selector, timeout = 30000) {
    // 实现逻辑
}

/**
 * 等待页面加载
 * @param {string} url - 期望的URL
 * @param {number} timeout - 超时时间
 * @returns {Promise<boolean>}
 */
async function waitForPageLoad(url, timeout = 60000) {
    // 实现逻辑
}

/**
 * 点击元素
 * @param {string} selector - CSS选择器
 * @param {Object} options - 选项
 * @returns {Promise<void>}
 */
async function clickElement(selector, options = {}) {
    // 实现逻辑
}

/**
 * 输入文本
 * @param {string} selector - CSS选择器
 * @param {string} text - 输入文本
 * @param {Object} options - 选项
 * @returns {Promise<void>}
 */
async function inputText(selector, text, options = {}) {
    // 实现逻辑
}

/**
 * 获取元素文本
 * @param {string} selector - CSS选择器
 * @returns {Promise<string>} 元素文本
 */
async function getElementText(selector) {
    // 实现逻辑
}

/**
 * 执行任意 JavaScript
 * @param {string|Function} fn - JavaScript代码或函数
 * @param {...any} args - 参数
 * @returns {Promise<any>} 执行结果
 */
async function executeScript(fn, ...args) {
    // 实现逻辑
}
```

#### 4.1.5 工具类

```javascript
/**
 * 随机延迟
 * @param {number} minMs - 最小延迟 (毫秒)
 * @param {number} maxMs - 最大延迟 (毫秒)
 * @returns {Promise<void>}
 */
async function randomDelay(minMs = 1000, maxMs = 3000) {
    // 实现逻辑
}

/**
 * 随机点击 (带鼠标移动)
 * @param {string} selector - CSS选择器
 * @param {Object} options - 选项
 * @returns {Promise<void>}
 */
async function humanLikeClick(selector, options = {}) {
    // 实现逻辑
}

/**
 * 获取当前钱包余额
 * @param {string} tokenAddress - Token地址 (空则为主币)
 * @returns {Promise<string>} 余额 (可读格式)
 */
async function getBalance(tokenAddress = null) {
    // 实现逻辑
}

/**
 * 获取Gas价格
 * @returns {Promise<Object>} {slow, standard, fast}
 */
async function getGasPrices() {
    // 实现逻辑
}

/**
 * 日志输出
 * @param {string} level - 日志级别 (info, warn, error, success)
 * @param {string} message - 消息
 */
function log(level, message) {
    // 实现逻辑
}
```

### 4.2 脚本示例

#### 4.2.1 OKX 每日签到示例

```javascript
// OKX Daily Claim Script
// 依赖API: connectOKXWallet, clickElement, waitForSelector, randomDelay, log

async function run(page, wallet, { connectOKXWallet, clickElement, waitForSelector, randomDelay, log }) {
    log('info', '开始执行 OKX Daily Claim');

    // 1. 打开OKX官网
    await page.goto('https://www.okx.com');
    await waitForSelector('body');
    await randomDelay(2000, 4000);

    // 2. 连接钱包
    log('info', '连接 OKX Wallet...');
    await connectOKXWallet({ chainId: '0x1' });
    await randomDelay(1000, 2000);

    // 3. 导航到签到页面
    log('info', '导航到 Rewards 页面...');
    await page.goto('https://www.okx.com/rewards');
    await waitForSelector('button.claim-button', 10000);
    await randomDelay(1000, 2000);

    // 4. 点击签到
    log('info', '执行签到操作...');
    const claimButton = await page.$('button.claim-button');
    if (claimButton) {
        await claimButton.click();
        await randomDelay(3000, 5000);
        log('success', '签到完成');
    } else {
        log('warn', '未找到签到按钮，可能今日已签到');
    }

    return { success: true };
}
```

#### 4.2.2 Uniswap Swap 示例

```javascript
// Uniswap V3 Swap Script
// 依赖API: connectMetaMask, switchNetwork, approveToken, transferToken, waitForSelector, log

async function run(page, wallet, { connectMetaMask, switchNetwork, approveToken, waitForSelector, log }) {
    const ETH_AMOUNT = '0.1';
    const USDC_ADDRESS = '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48';
    const UNISWAP_ROUTER = '0xE592427A0AEce92De3Edee1F18E0157C05861564';

    log('info', '开始执行 Uniswap Swap');

    // 1. 连接钱包
    await connectMetaMask({ expectedChainId: '0x1' });
    await switchNetwork('0x1');

    // 2. 打开Uniswap
    await page.goto('https://app.uniswap.org');
    await waitForSelector('body');
    await randomDelay(2000, 3000);

    // 3. 选择Token
    // ... Token选择逻辑

    // 4. 输入金额
    // ... 输入金额逻辑

    // 5. 确认交易
    log('info', '确认交易...');
    await clickElement('button[data-testid="swap-button"]');
    await randomDelay(1000, 2000);

    // 6. 等待MetaMask确认
    log('info', '等待钱包签名...');
    await waitForSelector('div.swap-review', 30000);

    return { success: true };
}
```

---

## 5. 脚本预处理器设计

### 5.1 预处理流程

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Script Preprocessor                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   原始脚本                                                                 │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ async function run(page, wallet, api) {                         │   │
│   │   await api.connectMetaMask();                                  │   │
│   │   await api.signMessage('Hello');                               │   │
│   │   await page.goto('https://example.com');                       │   │
│   │   await api.clickElement('.submit-btn');                        │   │
│   │ }                                                               │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                 │                                        │
│                                 ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ Step 1: 解析依赖API                                             │   │
│   │   - 扫描脚本中的 api.XXX 调用                                   │   │
│   │   - 提取所需的API列表: [connectMetaMask, signMessage,           │   │
│   │     clickElement]                                               │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                 │                                        │
│                                 ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ Step 2: 生成API实现代码                                         │   │
│   │   - 根据依赖列表生成对应的实现代码                              │   │
│   │   - 包含DOM操作、钱包交互、Web3调用等                           │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                 │                                        │
│                                 ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ Step 3: 注入模板代码                                            │   │
│   │   - 添加Playwright初始化代码                                    │   │
│   │   - 添加防检测代码                                              │   │
│   │   - 添加钱包注入代码                                            │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                 │                                        │
│                                 ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ Step 4: 代码验证                                                │   │
│   │   - 语法检查                                                    │   │
│   │   - 依赖完整性检查                                              │   │
│   │   - 安全检查 (禁止的危险操作)                                   │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                 │                                        │
│                                 ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────┐   │
│   │ Step 5: 输出完整脚本                                           │   │
│   │   - 生成可直接执行的JavaScript代码                              │   │
│   │   - 保存到临时文件                                              │   │
│   └─────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### 5.2 预处理器核心代码

```javascript
// src-tauri/src/automation/script_preprocessor.rs

use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;

pub struct ScriptPreprocessor {
    // API定义
    api_definitions: HashMap<String, ApiDefinition>,
}

struct ApiDefinition {
    name: String,
    params: Vec<String>,
    return_type: String,
    implementation: String,
}

impl ScriptPreprocessor {
    /// 预处理脚本
    pub fn preprocess(&self, source: &str, required_apis: &[String]) -> Result<String, String> {
        // 1. 解析依赖
        let dependencies = self.parse_dependencies(source)?;

        // 2. 验证依赖
        self.validate_dependencies(&dependencies)?;

        // 3. 生成API对象
        let api_code = self.generate_api_code(&dependencies)?;

        // 4. 注入模板
        let full_script = self.inject_template(source, &api_code)?;

        // 5. 语法验证
        self.validate_syntax(&full_script)?;

        Ok(full_script)
    }

    /// 解析脚本中的API依赖
    fn parse_dependencies(&self, source: &str) -> Result<HashSet<String>, String> {
        let re = Regex::new(r"api\.(\w+)").unwrap();
        let mut deps = HashSet::new();

        for cap in re.captures_iter(source) {
            if let Some(method_name) = cap.get(1) {
                deps.insert(method_name.as_str().to_string());
            }
        }

        Ok(deps)
    }

    /// 生成API实现代码
    fn generate_api_code(&self, deps: &HashSet<String>) -> Result<String, String> {
        let mut implementations = Vec::new();

        for dep in deps {
            if let Some(api_def) = self.api_definitions.get(dep) {
                implementations.push(api_def.implementation.clone());
            }
        }

        Ok(implementations.join("\n\n"))
    }

    /// 注入完整模板
    fn inject_template(&self, user_code: &str, api_code: &str) -> Result<String, String> {
        format!(r#"
/**
 * Auto-generated Automation Script
 * Generated at: {}
 */

const API = {{}};

// Inject API implementations
(async function() {{
    {api_code}
}})();

// User Script
{user_code}
        "#, chrono::Utc::now().to_rfc3339(), api_code = api_code, user_code = user_code)
            .parse()
            .map_err(|e| e.to_string())
    }

    /// 验证依赖是否都已知
    fn validate_dependencies(&self, deps: &HashSet<String>) -> Result<(), String> {
        let unknown: Vec<_> = deps.iter()
            .filter(|d| !self.api_definitions.contains_key(*d))
            .collect();

        if !unknown.is_empty() {
            return Err(format!("Unknown API methods: {}", unknown.join(", ")));
        }

        Ok(())
    }

    /// 验证JavaScript语法
    fn validate_syntax(&self, code: &str) -> Result<(), String> {
        // 使用 quickjs 或其他JS引擎验证语法
        // 或者使用 swc 解析
        Ok(())
    }
}
```

### 5.3 API实现生成器

```javascript
// API实现模板 - 实际会转换为Playwright可执行代码

const API_METHODS = {
    // MetaMask连接
    connectMetaMask: `
async function connectMetaMask(options = {}) {
    const { expectedChainId } = options;

    // 点击 MetaMask 图标或连接按钮
    const connectBtn = await page.$('button:has-text("Connect Wallet")');
    if (connectBtn) {
        await connectBtn.click();
        await page.waitForTimeout(1000);

        // 选择 MetaMask
        const metamaskBtn = await page.$('text=MetaMask');
        if (metamaskBtn) {
            await metamaskBtn.click();
            await page.waitForTimeout(2000);
        }
    }

    // 处理弹窗
    // 点击"签名"或"连接"按钮
    const signBtn = await page.$('button:has-text("Sign")');
    if (signBtn) {
        await signBtn.click();
    }

    // 等待连接成功
    await page.waitForTimeout(3000);

    // 获取连接后的地址
    const address = await page.evaluate(() => {
        // 尝试从页面获取地址
        const els = document.querySelectorAll('[data-testid="account-button"]');
        if (els.length > 0) {
            return els[0].textContent.trim();
        }
        return '';
    });

    return address;
}`,

    // OKX Wallet连接
    connectOKXWallet: `
async function connectOKXWallet(options = {}) {
    const { chainId } = options;

    // 点击连接钱包按钮
    const connectBtn = await page.$('button:has-text("Connect Wallet")');
    if (connectBtn) {
        await connectBtn.click();
        await page.waitForTimeout(1000);
    }

    // 选择 OKX Wallet
    const okxBtn = await page.$('text=OKX Wallet');
    if (okxBtn) {
        await okxBtn.click();
        await page.waitForTimeout(2000);
    }

    // 等待连接
    await page.waitForTimeout(3000);

    return await page.evaluate(() => {
        // 获取地址逻辑
        return '';
    });
}`,

    // 点击元素
    clickElement: `
async function clickElement(selector, options = {}) {
    const element = await page.$(selector);
    if (element) {
        await element.click();
        await page.waitForTimeout(1000);
    } else {
        throw new Error(\`Element not found: \${selector}\`);
    }
}`,

    // 等待选择器
    waitForSelector: `
async function waitForSelector(selector, timeout = 30000) {
    return await page.waitForSelector(selector, { timeout, state: 'visible' });
}`,

    // 随机延迟
    randomDelay: `
async function randomDelay(minMs = 1000, maxMs = 3000) {
    const delay = Math.floor(Math.random() * (maxMs - minMs + 1)) + minMs;
    await page.waitForTimeout(delay);
}`,

    // 日志
    log: `
function log(level, message) {
    const timestamp = new Date().toISOString();
    console.log(\`[\${timestamp}] [\${level.toUpperCase()}] \${message}\`);
}`,
};
```

---

## 6. 后端任务调度设计

### 6.1 任务调度器架构

```rust
// src-tauri/src/automation/mod.rs

use tokio::sync::mpsc;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use cron::Schedule;

pub struct TaskScheduler {
    // 任务仓库
    task_repo: Arc<TaskRepository>,
    // 执行器
    executor: Arc<TaskExecutor>,
    // 调度器
    cron_scheduler: Arc<CronScheduler>,
    // 任务状态
    state: Arc<RwLock<SchedulerState>>,
}

struct SchedulerState {
    running_tasks: HashMap<u64, RunningTask>,
    last_check: DateTime<Utc>,
}

struct RunningTask {
    task_id: u64,
    status: TaskStatus,
    started_at: DateTime<Utc>,
    wallet_count: usize,
    completed_count: usize,
    failed_count: usize,
    tx: mpsc::Sender<TaskEvent>,
}

impl TaskScheduler {
    /// 启动调度器
    pub async fn start(&self) -> Result<(), String> {
        // 启动cron调度循环
        self.start_cron_loop().await?;

        // 启动任务执行循环
        self.start_execution_loop().await?;

        Ok(())
    }

    /// 添加任务到执行队列
    pub async fn schedule_task(&self, task: Task) -> Result<(), String> {
        // 解析调度配置
        let schedule = self.parse_schedule(&task.schedule_config)?;

        // 计算下次执行时间
        let next_run = schedule.after(&Utc::now()).next();

        // 保存任务
        self.task_repo.save(&task, next_run).await?;

        Ok(())
    }

    /// 手动触发任务执行
    pub async fn trigger_task(&self, task_id: u64) -> Result<(), String> {
        let task = self.task_repo.get(task_id).await?
            .ok_or_else(|| "Task not found".to_string())?;

        // 立即执行
        self.execute_task(task).await?;

        Ok(())
    }

    /// 停止任务
    pub async fn stop_task(&self, task_id: u64) -> Result<(), String> {
        // 发送停止信号
        if let Some(running) = self.state.write().await.running_tasks.get(&task_id) {
            running.tx.send(TaskEvent::Stop).await?;
        }

        Ok(())
    }
}
```

### 6.2 任务执行器

```rust
// src-tauri/src/automation/executor.rs

pub struct TaskExecutor {
    // Playwright Bridge
    playwright_bridge: Arc<PlaywrightBridge>,
    // 浏览器实例池
    browser_pool: BrowserPool,
    // 数据库
    db: Arc<Database>,
}

impl TaskExecutor {
    /// 执行任务
    pub async fn execute(&self, task: &Task) -> Result<TaskResult, String> {
        let wallets = self.get_wallets(&task.wallet_group_id).await?;
        let script = self.get_script(&task.script_id).await?;

        // 预处理脚本
        let compiled_script = self.preprocessor.preprocess(
            &script.content,
            &script.required_apis
        )?;

        let total_wallets = wallets.len();
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failed_count = 0;

        // 并发执行
        let (tx, mut rx) = mpsc::channel(100);
        let semaphore = Arc::new(Semaphore::new(task.concurrency));

        for wallet in wallets {
            let permit = semaphore.acquire().await;
            let tx = tx.clone();
            let compiled_script = compiled_script.clone();

            tokio::spawn(async move {
                let result = Self::execute_for_wallet(
                    &wallet,
                    &compiled_script,
                    &task.browser_profile,
                ).await;

                permit.forget();
                tx.send((wallet.address.clone(), result)).await.ok();
            });
        }

        // 收集结果
        for _ in 0..total_wallets {
            if let Some((address, result)) = rx.recv().await {
                results.push((address.clone(), result.clone()));

                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => failed_count += 1,
                }

                // 更新进度
                self.update_progress(task.id, success_count, failed_count).await;
            }
        }

        Ok(TaskResult {
            total: total_wallets,
            success: success_count,
            failed: failed_count,
            results,
        })
    }

    /// 为单个钱包执行脚本
    async fn execute_for_wallet(
        &self,
        wallet: &Wallet,
        script: &str,
        profile: &BrowserProfile,
    ) -> Result<WalletResult, String> {
        // 获取浏览器实例
        let browser = self.browser_pool.get(profile).await?;

        // 创建浏览器上下文
        let context = browser.new_context(&profile).await?;

        // 注入钱包
        self.inject_wallet(&context, wallet).await?;

        // 执行脚本
        let result = self.playwright_bridge.execute_script(
            &context,
            script,
            wallet,
        ).await?;

        // 保存执行记录
        self.save_execution_log(wallet, &result).await?;

        Ok(result)
    }
}
```

---

## 7. Playwright Bridge 设计

### 7.1 架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Playwright Bridge (Node.js)                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Main Process (Tauri/Rust)                                           │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │  Command Handler                                           │     │
│  │  - execute_script                                          │     │
│  │  - create_browser_context                                  │     │
│  │  - inject_wallet_extension                                 │     │
│  │  - get_browser_screenshot                                  │     │
│  └────────────────────────────────────────────────────────────┘     │
│           │                                                        │
│           │ IPC (stdin/stdout JSON)                                │
           ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Child Process (Node.js)                                             │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │  Message Dispatcher                                         │     │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │     │
│  │  │ ScriptExec │ │ ContextMgr │ │ ExtensionInjector   │   │     │
│  │  └─────────────┘ └─────────────┘ └─────────────────────┘   │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │  Browser Manager                                           │     │
│  │  - Browser launch/close                                    │     │
│  │  - Context management                                      │     │
│  │  - Page management                                         │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │  Anti-Detection Module                                     │     │
│  │  - Canvas fingerprint spoofing                             │     │
│  │  - WebGL rendering spoofing                                │     │
│  │  - Navigator property override                             │     │
│  │  - AudioContext noise injection                            │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │  Wallet Injection Module                                   │     │
│  │  - MetaMask extension injection                            │     │
│  │  - OKX Wallet extension injection                          │     │
│  │  - Wallet connection management                            │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 7.2 核心实现

```javascript
// playwright-bridge/src/index.js

const { chromium } = require('playwright');
const fs = require('fs-extra');
const path = require('path');

class PlaywrightBridge {
    constructor() {
        this.browser = null;
        this.context = null;
        this.antiDetection = new AntiDetectionModule();
        this.walletInjector = new WalletInjectorModule();
    }

    async initialize() {
        // 加载扩展
        await this.walletInjector.loadExtensions();

        // 启动浏览器
        this.browser = await chromium.launch({
            headless: false,
            args: [
                '--disable-blink-features=AutomationControlled',
                '--no-sandbox',
                '--disable-infobars',
                '--disable-dev-shm-usage',
                // 扩展加载参数
                `--disable-extensions-except=${this.walletInjector.extensionPath}`,
                `--load-extension=${this.walletInjector.extensionPath}`,
            ]
        });
    }

    async createContext(profile) {
        // 应用防检测
        const contextOptions = this.antiDetection.createContextOptions(profile);

        this.context = await this.browser.newContext(contextOptions);

        // 注入初始化脚本
        await this.context.addInitScript(this.antiDetection.getInitScript());

        return { contextId: 'default' };
    }

    async executeScript(contextId, scriptContent, walletData) {
        const page = await this.context.newPage();

        // 注入钱包数据到页面
        await page.evaluate((wallet) => {
            window.WALLET_DATA = wallet;
        }, walletData);

        // 编译脚本 (将API方法注入)
        const compiledScript = this.compileScript(scriptContent);

        // 执行
        try {
            const result = await page.evaluate(async (script) => {
                // 动态执行编译后的脚本
                const fn = new Function('page', 'wallet', 'api', script);
                return await fn(page, window.WALLET_DATA, window.API);
            }, compiledScript);

            return { success: true, data: result };
        } catch (error) {
            return { success: false, error: error.message };
        }
    }

    compileScript(source) {
        // 解析API调用
        const apiCalls = this.parseApiCalls(source);

        // 生成API实现
        const apiImpl = this.generateApiImplementation(apiCalls);

        // 组合
        return `
            const API = ${JSON.stringify(apiImpl)};
            ${source}
        `;
    }
}

// 防检测模块
class AntiDetectionModule {
    createContextOptions(profile) {
        return {
            userAgent: profile.userAgent,
            viewport: {
                width: profile.viewportWidth,
                height: profile.viewportHeight
            },
            deviceScaleFactor: profile.deviceScaleFactor,
            locale: profile.locale,
            timezoneId: profile.timezoneId,
            permissions: ['geolocation', 'notifications'],
            ...this.getProxyOptions(profile)
        };
    }

    getInitScript() {
        return `
            // Navigator.webdriver
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined,
                configurable: true
            });

            // Navigator plugins
            Object.defineProperty(navigator, 'plugins', {
                get: () => [1, 2, 3, 4, 5],
                configurable: true
            });

            // Navigator languages
            Object.defineProperty(navigator, 'languages', {
                get: () => ['en-US', 'en'],
                configurable: true
            });

            // Canvas fingerprint spoofing
            const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
            HTMLCanvasElement.prototype.toDataURL = function(type) {
                const res = originalToDataURL.apply(this, arguments);
                // 添加轻微噪声
                return res;
            };

            // WebGL
            const getParameter = WebGLRenderingContext.prototype.getParameter;
            WebGLRenderingContext.prototype.getParameter = function(param) {
                // 返回虚拟参数
                if (param === 37445) return 'Google Inc. (NVIDIA)';
                if (param === 37446) return 'ANGLE (NVIDIA GeForce GTX 1080)';
                return getParameter.apply(this, arguments);
            };
        `;
    }
}

// 钱包注入模块
class WalletInjectorModule {
    constructor() {
        this.extensionPath = path.join(__dirname, '../extensions');
    }

    async loadExtensions() {
        // 从本地或远程加载MetaMask和OKX Wallet扩展
        await this.ensureExtension('metamask');
        await this.ensureExtension('okxwallet');
    }

    async ensureExtension(name) {
        const extPath = path.join(this.extensionPath, name);
        if (!await fs.pathExists(extPath)) {
            // 下载或解压扩展
            await this.downloadExtension(name, extPath);
        }
    }

    async injectWalletExtension(context, walletType) {
        // 将钱包扩展注入到上下文
        // 对于MetaMask，需要设置初始状态
        const extPath = path.join(this.extensionPath, walletType);

        // 配置钱包初始状态
        await context.addInitScript(`
            window.EXTENSION_PATH = '${extPath}';
            window.WALLET_TYPE = '${walletType}';
        `);
    }
}
```

---

## 8. 文件结构

```
src/
├── features/
│   └── airdrop/
│       ├── pages/
│       │   ├── BrowserAutomation.vue      # 主页面
│       │   ├── TaskManager.vue            # 任务管理页面 (新建/编辑)
│       │   └── TaskMonitor.vue            # 任务监控页面
│       ├── components/
│       │   ├── WalletSelector.vue         # 钱包选择器
│       │   ├── ScriptSelector.vue         # 脚本选择器
│       │   ├── ScheduleConfig.vue         # 调度配置
│       │   ├── BrowserConfig.vue          # 浏览器环境配置
│       │   ├── TaskProgress.vue           # 任务进度面板
│       │   ├── ExecutionLogs.vue          # 执行日志
│       │   └── ApiHelper.vue              # API帮助文档
│       └── store/
│           └── automation.ts              # Pinia状态管理
│
src-tauri/src/
├── automation/
│   ├── mod.rs                            # 模块入口
│   ├── scheduler.rs                      # 任务调度器
│   ├── executor.rs                       # 任务执行器
│   ├── preprocessor.rs                   # 脚本预处理器
│   ├── models.rs                         # 数据模型
│   └── commands.rs                       # Tauri命令
│
playwright-bridge/
├── src/
│   ├── index.js                          # Bridge主入口
│   ├── browser-manager.js                # 浏览器管理
│   ├── anti-detection.js                 # 防检测模块
│   ├── wallet-injector.js                # 钱包注入模块
│   └── api-runtime.js                    # API运行时
├── extensions/
│   ├── metamask/                         # MetaMask扩展
│   └── okxwallet/                        # OKX Wallet扩展
└── package.json
│
docs/
├── BROWSER_AUTOMATION_DESIGN.md          # 本设计文档
└── SCRIPT_API_REFERENCE.md               # API参考文档
```

---

## 9. 实施计划

### Phase 1: 基础框架 (1周)
- [ ] 数据库表结构创建
- [ ] 后端任务调度器基础实现
- [ ] 任务CRUD API
- [ ] 基础UI框架

### Phase 2: 脚本系统 (1周)
- [ ] 脚本预处理器实现
- [ ] 自定义API定义和生成
- [ ] 脚本编辑器增强
- [ ] 脚本版本管理

### Phase 3: 浏览器集成 (1周)
- [ ] Playwright Bridge完善
- [ ] 浏览器环境配置
- [ ] 防检测模块
- [ ] 钱包扩展注入

### Phase 4: 任务执行 (1周)
- [ ] 任务执行器实现
- [ ] 并发控制
- [ ] 进度追踪
- [ ] 日志系统

### Phase 5: 监控和通知 (3天)
- [ ] 实时监控面板
- [ ] 执行状态追踪
- [ ] 通知集成
- [ ] 错误处理和重试

---

## 10. 风险和注意事项

1. **安全性**
   - 脚本执行环境需要沙箱化
   - 敏感操作需要二次确认
   - 防止恶意脚本执行

2. **反检测**
   - 指纹保护需要持续更新
   - 扩展版本兼容性
   - 浏览器更新适配

3. **性能**
   - 大量钱包并发执行
   - 浏览器实例内存管理
   - 任务队列持久化

4. **兼容性**
   - 不同版本的MetaMask/OKX
   - 不同网站的变化
   - 网络条件变化
