# 浏览器自动化功能设计文档

## 1. 概述

### 1.1 功能目标
完善浏览器自动化页面功能，实现以下核心能力：
- 钱包管理：导入钱包（名称、私钥、地址、备注），支持编辑和删除
- 环境配置：批量生成/创建指纹信息，配置抗审查技术选项
- 脚本编辑：编辑和管理脚本程序
- 执行面板：单账号或批量测试执行
- 任务管理：配置定时任务，执行时随机使用所有环境配置
- 任务监控：查看任务执行情况

### 1.2 使用流程

```
┌─────────────────────────────────────────────────────────────────────┐
│                        浏览器自动化使用流程                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Step 1: 钱包管理                                                    │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • 导入钱包（名称、私钥、地址、备注）                        │     │
│  │ • 支持 Excel 导入                                          │     │
│  │ • 编辑、删除钱包                                           │     │
│  │ • 数据保存到独立数据表（SQLite）                           │     │
│  └────────────────────────────────────────────────────────────┘     │
│                              ↓                                       │
│  Step 2: 环境配置                                                    │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • 批量生成指纹配置                                         │     │
│  │ • 配置抗审查技术选项：                                     │     │
│  │   - Canvas 指纹混淆                                        │     │
│  │   - WebGL 渲染伪装                                         │     │
│  │   - Audio Context 噪音                                     │     │
│  │   - 时区伪装                                               │     │
│  │   - 地理位置伪装                                           │     │
│  │   - 字体伪装                                               │     │
│  │   - WebRTC 防泄漏                                          │     │
│  │ • 持久化保存到数据库                                       │     │
│  └────────────────────────────────────────────────────────────┘     │
│                              ↓                                       │
│  Step 3: 脚本编辑                                                    │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • 创建/编辑脚本                                            │     │
│  │ • 使用自定义 API（连接钱包、签名、交易等）                 │     │
│  │ • 导入/导出脚本                                            │     │
│  └────────────────────────────────────────────────────────────┘     │
│                              ↓                                       │
│  Step 4: 执行面板（可选测试）                                        │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • 选择脚本、环境、钱包                                     │     │
│  │ • 单账号测试                                               │     │
│  │ • 批量账号测试                                             │     │
│  │ • 验证脚本正确性                                           │     │
│  └────────────────────────────────────────────────────────────┘     │
│                              ↓                                       │
│  Step 5: 任务管理                                                    │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • 配置定时任务（Cron 表达式）                              │     │
│  │ • 选择要执行的钱包列表                                     │     │
│  │ • 选择执行的脚本                                           │     │
│  │ • 环境分配策略：随机使用所有环境配置                       │     │
│  │ • 启用/禁用任务                                            │     │
│  └────────────────────────────────────────────────────────────┘     │
│                              ↓                                       │
│  Step 6: 任务监控                                                    │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • 查看任务执行状态                                         │     │
│  │ • 查看执行日志                                             │     │
│  │ • 统计成功率                                               │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.3 整体架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Browser Automation System                      │
├─────────────────────────────────────────────────────────────────────┤
│  Frontend (Vue 3)                                                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ 钱包管理     │  │ 环境配置     │  │ 脚本编辑器  │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
│                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ 执行面板     │  │ 任务管理     │  │ 任务监控    │              │
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
│  │ Tables: airdrop_wallets, browser_profiles, automation_tasks │     │
│  │         automation_scripts, task_executions, task_logs     │     │
│  └────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. 数据库设计

### 2.1 表结构

#### 2.1.1 空投钱包表 (`airdrop_wallets`)

```sql
CREATE TABLE airdrop_wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,           -- 钱包名称
    address VARCHAR(255) NOT NULL,        -- 钱包地址
    private_key VARCHAR(512) NOT NULL,    -- 加密存储的私钥
    label VARCHAR(255),                   -- 备注
    group_name VARCHAR(128) DEFAULT 'Default', -- 分组
    proxy VARCHAR(255) DEFAULT 'Direct',  -- 代理配置
    chain_type VARCHAR(32) DEFAULT 'evm', -- 链类型 (evm/solana)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_airdrop_wallets_address ON airdrop_wallets(address);
CREATE INDEX idx_airdrop_wallets_group ON airdrop_wallets(group_name);
```

#### 2.1.2 浏览器环境配置表 (`browser_profiles`)

```sql
CREATE TABLE browser_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- 浏览器基础配置
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

    -- 指纹保护配置 (抗审查技术)
    canvas_spoof BOOLEAN DEFAULT TRUE,           -- Canvas 指纹混淆
    webgl_spoof BOOLEAN DEFAULT TRUE,            -- WebGL 渲染伪装
    audio_spoof BOOLEAN DEFAULT TRUE,            -- Audio Context 噪音
    timezone_spoof BOOLEAN DEFAULT TRUE,         -- 时区伪装
    geolocation_spoof BOOLEAN DEFAULT TRUE,      -- 地理位置伪装
    font_spoof BOOLEAN DEFAULT TRUE,             -- 字体伪装
    webrtc_spoof BOOLEAN DEFAULT TRUE,           -- WebRTC 防泄漏
    navigator_override BOOLEAN DEFAULT TRUE,     -- Navigator 属性覆盖
    webdriver_override BOOLEAN DEFAULT TRUE,     -- WebDriver 属性覆盖

    -- 高级配置
    custom_headers TEXT,  -- JSON 格式自定义请求头
    headless BOOLEAN DEFAULT FALSE,
    extensions TEXT,  -- JSON: ["metamask", "okxwallet"]

    is_default BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_browser_profiles_default ON browser_profiles(is_default);
```

#### 2.1.3 自动化任务表 (`automation_tasks`)

```sql
CREATE TABLE automation_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- 执行配置
    script_id INTEGER NOT NULL,              -- 关联的脚本ID
    wallet_ids TEXT NOT NULL,                -- JSON 数组: [1, 2, 3, ...]

    -- 环境分配策略
    profile_strategy VARCHAR(32) DEFAULT 'random', -- random, sequential, specific
    specific_profile_id INTEGER,             -- 当 strategy 为 specific 时使用

    -- 调度配置
    schedule_type VARCHAR(32) DEFAULT 'once',  -- once, interval, cron
    schedule_config TEXT NOT NULL,           -- JSON: {"cron": "0 9 * * *"}

    -- 执行参数
    concurrency INTEGER DEFAULT 1,           -- 并发数
    timeout_seconds INTEGER DEFAULT 300,     -- 超时时间
    retry_times INTEGER DEFAULT 3,           -- 重试次数
    retry_interval_seconds INTEGER DEFAULT 60, -- 重试间隔

    -- 状态
    status VARCHAR(32) DEFAULT 'draft',      -- draft, enabled, paused, running
    last_run_time DATETIME,
    next_run_time DATETIME,
    total_runs INTEGER DEFAULT 0,
    success_runs INTEGER DEFAULT 0,
    failed_runs INTEGER DEFAULT 0,

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (script_id) REFERENCES automation_scripts(id),
    FOREIGN KEY (specific_profile_id) REFERENCES browser_profiles(id)
);

CREATE INDEX idx_tasks_status ON automation_tasks(status);
CREATE INDEX idx_tasks_next_run ON automation_tasks(next_run_time) WHERE status = 'enabled';
```

#### 2.1.4 自动化脚本表 (`automation_scripts`)

```sql
CREATE TABLE automation_scripts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    content TEXT NOT NULL,                   -- 原始脚本内容（含自定义API）
    compiled_content TEXT,                   -- 预处理后的完整脚本
    version INTEGER DEFAULT 1,
    is_system BOOLEAN DEFAULT FALSE,         -- 系统内置脚本

    -- API依赖
    required_apis TEXT,                      -- JSON: ["connectWallet", "signMessage"]

    -- 元数据
    author VARCHAR(255),
    tags TEXT,                               -- JSON: ["uniswap", "okx", "airdrop"]

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_scripts_tags ON automation_scripts(tags);
```

#### 2.1.5 任务执行记录表 (`task_executions`)

```sql
CREATE TABLE task_executions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    wallet_id INTEGER NOT NULL,              -- 关联的钱包ID
    profile_id INTEGER,                      -- 使用的环境配置ID

    -- 执行状态
    status VARCHAR(32) NOT NULL,             -- pending, running, success, failed, stopped
    start_time DATETIME,
    end_time DATETIME,
    duration_ms INTEGER,

    -- 结果
    error_message TEXT,
    result_data TEXT,                        -- JSON: {"txHash": "...", "data": {...}}
    logs TEXT,                               -- 执行日志（JSON数组）

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (task_id) REFERENCES automation_tasks(id),
    FOREIGN KEY (wallet_id) REFERENCES airdrop_wallets(id),
    FOREIGN KEY (profile_id) REFERENCES browser_profiles(id)
);

CREATE INDEX idx_executions_task ON task_executions(task_id);
CREATE INDEX idx_executions_status ON task_executions(status);
CREATE INDEX idx_executions_time ON task_executions(start_time DESC);
```

---

## 3. 前端设计

### 3.1 钱包管理页面

#### 3.1.1 布局结构

```
┌─────────────────────────────────────────────────────────────────────┐
│  钱包管理                                                            │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ [添加钱包] [从系统同步] [导入(Excel)] [导出] [清空]         │     │
│  │ 搜索: [________________________________]                   │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ ID │ 名称      │ 地址              │ 备注    │ 分组   │ 操作 │     │
│  ├────────────────────────────────────────────────────────────┤     │
│  │ 1  │ Main      │ 0x742d...f44e     │ 主钱包  │ Default│ 编辑 │     │
│  │ 2  │ Airdrop 1 │ 0x123d...abc      │ 空投1   │ Group A│ 删除 │     │
│  │ 3  │ Airdrop 2 │ 0x456d...def      │ 空投2   │ Group A│      │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  总计: 3 个钱包                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

#### 3.1.2 Excel 导入格式

| 名称 | 私钥 | 地址 | 备注 | 分组 | 代理 |
|------|------|------|------|------|------|
| Main | 0x... | 0x742d... | 主钱包 | Default | Direct |
| Airdrop 1 | 0x... | 0x123d... | 空投1 | Group A | http://proxy:8080 |

### 3.2 环境配置页面

#### 3.2.1 布局结构

```
┌─────────────────────────────────────────────────────────────────────┐
│  环境配置                                                            │
│  ┌──────────────────────────────┐  ┌──────────────────────────────┐ │
│  │ [批量生成] [导入] [导出]      │  │ 编辑配置: Profile-001        │ │
│  │                              │  │                              │ │
│  │ 环境列表                      │  │ 配置名称: [____________]     │ │
│  │ ┌──────────────────────────┐ │  │                              │ │
│  │ │ Profile-001              │ │  │ User Agent:                  │ │
│  │ │ 1920x1080 | Direct       │ │  │ [________________________]   │ │
│  │ │                          │ │  │                              │ │
│  │ │ Profile-002              │ │  │ 分辨率: [1920x1080    v]     │ │
│  │ │ 1366x768 | HTTP Proxy    │ │  │ 代理:   [Direct       v]     │ │
│  │ │                          │ │  │                              │ │
│  │ │ Profile-003              │ │  │ ┌──────────────────────────┐ │ │
│  │ │ ...                      │ │  │ │ 指纹保护 (Anti-Detect)   │ │ │
│  │ └──────────────────────────┘ │  │ ├──────────────────────────┤ │ │
│  │                              │  │ │ [✓] Canvas 指纹混淆      │ │ │
│  └──────────────────────────────┘  │ │ [✓] WebGL 渲染伪装       │ │ │
│                                     │ │ [✓] Audio Context 噪音   │ │ │
│                                     │ │ [✓] 时区伪装             │ │ │
│                                     │ │ [✓] 地理位置伪装         │ │ │
│                                     │ │ [✓] 字体伪装             │ │ │
│                                     │ │ [✓] WebRTC 防泄漏        │ │ │
│                                     │ └──────────────────────────┘ │ │
│                                     │                              │ │
│                                     │ [删除] [取消] [保存]         │ │
│                                     └──────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.3 脚本编辑页面

#### 3.3.1 布局结构

```
┌─────────────────────────────────────────────────────────────────────┐
│  脚本编辑                                                            │
│  ┌──────────────────┐  ┌──────────────────────────────────────────┐ │
│  │ [新建] [导入]     │  │ 当前编辑: OKX Daily Claim                │ │
│  │                  │  │                                          │ │
│  │ 脚本列表          │  │ ┌──────────────────────────────────────┐ │ │
│  │ ┌──────────────┐ │  │ │ [API文档] [全屏] [导出] [复制] [测试]│ │ │
│  │ │ OKX Daily    │ │  │ └──────────────────────────────────────┘ │ │
│  │ │ Uniswap Swap │ │  │                                          │ │
│  │ │ LayerZero    │ │  │ ┌──────────────────────────────────────┐ │ │
│  │ │ ...          │ │  │ │ // OKX Daily Claim Script            │ │ │
│  │ └──────────────┘ │  │ │ async function run({ page, wallet,   │ │ │
│  │                  │  │ │   api }) {                             │ │ │
│  │                  │  │ │   api.log('info', '开始执行');         │ │ │
│  │                  │  │ │   await page.goto('https://okx.com');  │ │ │
│  │                  │  │ │   await api.connectOKXWallet();        │ │ │
│  │                  │  │ │   // ...                               │ │ │
│  │                  │  │ │ }                                      │ │ │
│  │                  │  │ └──────────────────────────────────────┘ │ │
│  │                  │  │                                          │ │
│  └──────────────────┘  └──────────────────────────────────────────┘ │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ API 参考文档                                                    │ │
│  │ [钱包连接] [签名操作] [交易操作] [浏览器操作] [工具函数]        │ │
│  │                                                                 │ │
│  │ connectMetaMask(options) - 连接 MetaMask 钱包                  │ │
│  │ signMessage(message) - 签名消息                                │ │
│  │ clickElement(selector) - 点击元素                              │ │
│  │ ...                                                            │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.4 执行面板页面

#### 3.4.1 布局结构

```
┌─────────────────────────────────────────────────────────────────────┐
│  执行面板                                                            │
│  ┌──────────────────────────────┐  ┌──────────────────────────────┐ │
│  │ 配置                          │  │ 执行状态                     │ │
│  │                              │  │                              │ │
│  │ 选择脚本:                    │  │ 执行时间: 00:05:32           │ │
│  │ [OKX Daily Claim      v]     │  │ 总任务: 10                   │ │
│  │                              │  │ 成功: 8    失败: 0   等待: 2 │ │
│  │ 浏览器配置:                  │  │                              │ │
│  │ [Profile-001          v]     │  │ 进度: 80%                    │ │
│  │                              │  │ ████████████████████░░       │ │
│  │ 选择钱包:                    │  │                              │ │
│  │ [✓] 全部 (10)               │  │ [开始执行] [停止] [重置]     │ │
│  │ [✓] Main (0x742d...)        │  │                              │ │
│  │ [✓] Airdrop 1 (0x123d...)   │  │                              │ │
│  │ [✓] Airdrop 2 (0x456d...)   │  │                              │ │
│  │ ...                         │  │                              │ │
│  │                              │  │                              │ │
│  │ 执行配置:                    │  │                              │ │
│  │ [✓] 无头模式                │  │                              │ │
│  │ [✓] 使用代理                │  │                              │ │
│  │ [✓] 指纹保护                │  │                              │ │
│  │                              │  │                              │ │
│  └──────────────────────────────┘  └──────────────────────────────┘ │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ [执行队列] [执行日志]                                           │ │
│  │                                                                 │ │
│  │ 钱包              状态      耗时      交易哈希                  │ │
│  │ Main              [✓]成功   12.5s     0xabc...                  │ │
│  │ Airdrop 1         [✓]成功   11.8s     0xdef...                  │ │
│  │ Airdrop 2         [运行中]  --        --                        │ │
│  │ Airdrop 3         [等待中]  --        --                        │ │
│  │ ...                                                             │ │
│  │                                                                 │ │
│  │ [14:25:32] [INFO] 任务开始执行: OKX Daily Claim                 │ │
│  │ [14:25:33] [INFO] 初始化浏览器环境...                           │ │
│  │ [14:25:34] [SUCCESS] Main 执行成功                              │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.5 任务管理页面

#### 3.5.1 布局结构

```
┌─────────────────────────────────────────────────────────────────────┐
│  任务管理                                                            │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ [新建任务] [导入] [导出] [清空]                            │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ 任务名称        执行时间      状态      执行次数    操作   │     │
│  ├────────────────────────────────────────────────────────────┤     │
│  │ OKX Daily       每天 9:00     [运行中]  45/∞       [停止] │     │
│  │ Uniswap Swap    每周一 10:00  [已暂停]  12/12      [启动] │     │
│  │ LayerZero       2024-01-15    [已启用]  0/100      [编辑] │     │
│  │ Airdrop Batch   立即执行      [已完成]  100/100    [删除] │     │
│  └────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
```

#### 3.5.2 新建/编辑任务弹窗

```
┌─────────────────────────────────────────────────────────────────────┐
│  新建任务                                                            │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ 基本信息                                                    │     │
│  │ 任务名称: [________________________] *                      │     │
│  │ 任务描述: [________________________]                        │     │
│  ├────────────────────────────────────────────────────────────┤     │
│  │ 脚本选择                                                    │     │
│  │ 选择脚本: [OKX Daily Claim          v] *                    │     │
│  ├────────────────────────────────────────────────────────────┤     │
│  │ 钱包选择                                                    │     │
│  │ 选择方式: (○) 全部钱包  ( ) 指定分组  ( ) 指定钱包         │     │
│  │                                                            │     │
│  │ 钱包列表:                                                  │     │
│  │ [✓] Main (0x742d...)                                       │     │
│  │ [✓] Airdrop 1 (0x123d...)                                  │     │
│  │ [✓] Airdrop 2 (0x456d...)                                  │     │
│  ├────────────────────────────────────────────────────────────┤     │
│  │ 环境配置                                                    │     │
│  │ 分配策略: (○) 随机使用所有环境  ( ) 顺序使用  ( ) 指定环境 │     │
│  │                                                            │     │
│  │ 说明: 执行时将随机从所有环境配置中选择一个使用             │     │
│  ├────────────────────────────────────────────────────────────┤     │
│  │ 执行计划                                                    │     │
│  │ 执行方式: (○) 立即执行  ( ) 定时执行(Cron)                 │     │
│  │                                                            │     │
│  │ Cron 表达式: [0 9 * * *            ]                       │     │
│  │ 示例: 0 9 * * * (每天9点), 0 */6 * * * (每6小时)           │     │
│  ├────────────────────────────────────────────────────────────┤     │
│  │ 高级选项                                                    │     │
│  │ 超时时间: [300    ] 秒   重试次数: [3    ]                 │     │
│  │ 并发数:   [1      ]                                      │     │
│  │ [✓] 执行失败时通知                                        │     │
│  ├────────────────────────────────────────────────────────────┤     │
│  │ [取消]  [保存为草稿]  [保存并启用]                         │     │
│  └────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.6 任务监控页面

#### 3.6.1 布局结构

```
┌─────────────────────────────────────────────────────────────────────┐
│  任务监控                                                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐               │
│  │ 总执行   │ │ 成功     │ │ 失败     │ │ 成功率   │               │
│  │ 156      │ │ 150      │ │ 6        │ │ 96%      │               │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘               │
│                                                                      │
│  ┌──────────────────────────────┐  ┌──────────────────────────────┐ │
│  │ 执行历史                      │  │ 执行详情                     │ │
│  │ [搜索...] [刷新] [清空]      │  │                              │ │
│  │                              │  │ 任务: OKX Daily Claim        │ │
│  │ 钱包        脚本      时间   │  │ 钱包: Main (0x742d...)       │ │
│  │ Main        OKX       14:25  │  │ 环境: Profile-003            │ │
│  │ Airdrop 1   OKX       14:26  │  │ 状态: [✓] 成功               │ │
│  │ Airdrop 2   OKX       14:27  │  │ 耗时: 12.5s                  │ │
│  │ ...                          │  │ 交易: 0xabc...               │ │
│  │                              │  │                              │ │
│  │                              │  │ 执行日志:                    │ │
│  │                              │  │ [14:25:32] 开始执行          │ │
│  │                              │  │ [14:25:33] 连接钱包          │ │
│  │                              │  │ [14:25:35] 签到成功          │ │
│  │                              │  │                              │ │
│  └──────────────────────────────┘  └──────────────────────────────┘ │
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
 * @param {string} options.expectedChainId - 期望的网络ID (如 '0x1')
 * @returns {Promise<string>} 连接后的钱包地址
 */
async function connectMetaMask(options = {}) {}

/**
 * 连接 OKX Wallet
 * @param {Object} options - 连接选项
 * @param {string} options.chainId - 链ID
 * @returns {Promise<string>} 连接后的钱包地址
 */
async function connectOKXWallet(options = {}) {}

/**
 * 切换钱包网络
 * @param {string} chainId - 目标网络ID (十六进制字符串)
 * @returns {Promise<boolean>} 是否切换成功
 */
async function switchNetwork(chainId) {}

/**
 * 获取当前连接的钱包地址
 * @returns {Promise<string>} 当前地址
 */
async function getCurrentAddress() {}
```

#### 4.1.2 签名类

```javascript
/**
 * 签名消息
 * @param {string} message - 要签名的消息
 * @param {Object} options - 选项
 * @returns {Promise<string>} 签名结果 (hex格式)
 */
async function signMessage(message, options = {}) {}

/**
 * 签名交易
 * @param {Object} tx - 交易对象
 * @param {string} tx.to - 目标地址
 * @param {string} tx.value - 转账金额 (wei/lamports)
 * @param {string} tx.data - 可选数据
 * @returns {Promise<string>} 签名后的交易
 */
async function signTransaction(tx) {}

/**
 * 签名类型数据 (EIP-712)
 * @param {Object} domain - EIP-712 Domain
 * @param {Object} types - 类型定义
 * @param {Object} value - 要签名的值
 * @returns {Promise<string>} 签名结果
 */
async function signTypedData(domain, types, value) {}
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
async function sendNativeTransfer(to, amount, options = {}) {}

/**
 * ERC-20 Token 授权
 * @param {string} tokenAddress - Token合约地址
 * @param {string} spender - 授权给谁
 * @param {string} amount - 授权数量 (可读格式，如 '1000 USDC')
 * @returns {Promise<Object>} 交易结果
 */
async function approveToken(tokenAddress, spender, amount) {}

/**
 * ERC-20 Token 转账
 * @param {string} tokenAddress - Token合约地址
 * @param {string} to - 接收地址
 * @param {string} amount - 转账数量
 * @returns {Promise<Object>} 交易结果
 */
async function transferToken(tokenAddress, to, amount) {}

/**
 * 等待交易确认
 * @param {string} txHash - 交易哈希
 * @param {number} confirmations - 确认数 (默认1)
 * @returns {Promise<Object>} 交易收据
 */
async function waitForTransaction(txHash, confirmations = 1) {}
```

#### 4.1.4 浏览器操作类

```javascript
/**
 * 等待元素出现
 * @param {string} selector - CSS选择器
 * @param {number} timeout - 超时时间 (毫秒)
 * @returns {Promise<Element>} 元素
 */
async function waitForSelector(selector, timeout = 30000) {}

/**
 * 等待元素消失
 * @param {string} selector - CSS选择器
 * @param {number} timeout - 超时时间 (毫秒)
 * @returns {Promise<boolean>} 是否成功消失
 */
async function waitForSelectorHidden(selector, timeout = 30000) {}

/**
 * 点击元素
 * @param {string} selector - CSS选择器
 * @param {Object} options - 选项
 * @returns {Promise<void>}
 */
async function clickElement(selector, options = {}) {}

/**
 * 输入文本
 * @param {string} selector - CSS选择器
 * @param {string} text - 输入文本
 * @param {Object} options - 选项
 * @returns {Promise<void>}
 */
async function inputText(selector, text, options = {}) {}

/**
 * 获取元素文本
 * @param {string} selector - CSS选择器
 * @returns {Promise<string>} 元素文本
 */
async function getElementText(selector) {}
```

#### 4.1.5 工具类

```javascript
/**
 * 随机延迟
 * @param {number} minMs - 最小延迟 (毫秒)
 * @param {number} maxMs - 最大延迟 (毫秒)
 * @returns {Promise<void>}
 */
async function randomDelay(minMs = 1000, maxMs = 3000) {}

/**
 * 模拟人类点击 (带随机偏移)
 * @param {string} selector - CSS选择器
 * @param {Object} options - 选项
 * @returns {Promise<void>}
 */
async function humanLikeClick(selector, options = {}) {}

/**
 * 获取钱包余额
 * @param {string} tokenAddress - Token地址 (空则为主币)
 * @returns {Promise<string>} 余额
 */
async function getBalance(tokenAddress = null) {}

/**
 * 日志输出
 * @param {string} level - 日志级别 (info, warn, error, success)
 * @param {string} message - 消息
 */
function log(level, message) {}
```

### 4.2 脚本示例

```javascript
// OKX Daily Claim Script
async function run({ page, wallet, api }) {
    api.log('info', '开始执行 OKX Daily Claim');
    
    // 1. 打开OKX官网
    await page.goto('https://www.okx.com');
    await api.waitForSelector('body');
    await api.randomDelay(2000, 4000);
    
    // 2. 连接钱包
    api.log('info', '连接 OKX Wallet...');
    await api.connectOKXWallet({ chainId: '0x1' });
    await api.randomDelay(1000, 2000);
    
    // 3. 导航到签到页面
    api.log('info', '导航到 Rewards 页面...');
    await page.goto('https://www.okx.com/rewards');
    await api.waitForSelector('button.claim-button', 10000);
    
    // 4. 点击签到
    api.log('info', '执行签到操作...');
    await api.clickElement('button.claim-button');
    await api.randomDelay(3000, 5000);
    
    api.log('success', '签到完成');
    return { success: true };
}
```

---

## 5. 环境随机分配策略

### 5.1 分配策略实现

```typescript
// 环境分配策略类型
type ProfileStrategy = 'random' | 'sequential' | 'specific';

// 环境分配函数
function assignProfile(
    strategy: ProfileStrategy,
    profiles: BrowserProfile[],
    index: number,
    specificProfileId?: number
): BrowserProfile {
    switch (strategy) {
        case 'random':
            // 随机从所有环境中选择一个
            return profiles[Math.floor(Math.random() * profiles.length)];
        
        case 'sequential':
            // 按顺序循环使用
            return profiles[index % profiles.length];
        
        case 'specific':
            // 使用指定的环境
            const profile = profiles.find(p => p.id === specificProfileId);
            if (!profile) {
                throw new Error(`指定的环境配置不存在: ${specificProfileId}`);
            }
            return profile;
        
        default:
            return profiles[0];
    }
}

// 批量执行时的环境分配示例
async function executeBatch(
    wallets: Wallet[],
    profiles: BrowserProfile[],
    strategy: ProfileStrategy
) {
    const assignments = wallets.map((wallet, index) => ({
        wallet,
        profile: assignProfile(strategy, profiles, index)
    }));
    
    // 输出分配结果
    assignments.forEach(({ wallet, profile }) => {
        console.log(`钱包 ${wallet.name} -> 环境 ${profile.name}`);
    });
    
    return assignments;
}
```

---

## 6. 实施计划

### Phase 1: 数据层改造 (1周)
- [ ] 创建 SQLite 数据库表结构
- [ ] 实现数据库访问层 (Tauri Command)
- [ ] 数据迁移工具 (localStorage -> SQLite)
- [ ] 更新前端 Service 层

### Phase 2: 钱包管理增强 (3天)
- [ ] 添加私钥字段 (UI + 加密存储)
- [ ] 扩展 Excel 导入模板（支持私钥列）
- [ ] 私钥安全显示（掩码 + 复制）

### Phase 3: 环境配置增强 (3天)
- [ ] 扩展指纹选项 UI（WebGL、Audio、时区等）
- [ ] 实现指纹注入脚本
- [ ] 环境配置持久化到数据库

### Phase 4: 任务调度系统 (1周)
- [ ] 后端 Cron 调度器
- [ ] 任务执行队列
- [ ] 环境随机分配逻辑
- [ ] 执行结果持久化

### Phase 5: 执行引擎 (1周)
- [ ] Playwright 真实执行
- [ ] 指纹注入集成
- [ ] 日志和错误处理

---

## 7. 文件结构

```
src/
├── features/
│   └── airdrop/
│       ├── pages/
│       │   ├── BrowserAutomation.vue      # 主页面
│       │   └── Airdrop.vue                # 空投页面
│       ├── components/
│       │   ├── WalletManager.vue          # 钱包管理
│       │   ├── BrowserFarm.vue            # 环境配置
│       │   ├── ScriptEditor.vue           # 脚本编辑
│       │   ├── ExecutionPanel.vue         # 执行面板
│       │   ├── TaskManager.vue            # 任务管理
│       │   ├── TaskMonitor.vue            # 任务监控
│       │   └── ApiHelper.vue              # API帮助
│       └── services/
│           └── playwrightService.ts       # 服务层
│
src-tauri/src/
├── automation/
│   ├── mod.rs                            # 模块入口
│   ├── scheduler.rs                      # 任务调度器
│   ├── executor.rs                       # 任务执行器
│   ├── models.rs                         # 数据模型
│   └── commands.rs                       # Tauri命令
│
docs/
└── BROWSER_AUTOMATION_DESIGN.md          # 本设计文档
```

---

## 8. 风险和注意事项

1. **安全性**
   - 私钥必须加密存储
   - 脚本执行环境需要沙箱化
   - 敏感操作需要二次确认

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

---

## 9. 现有实现分析与改进建议

### 9.1 当前实现状态

通过源码分析，当前各模块实现状态如下：

| 模块 | 实现状态 | 完成度 | 说明 |
|------|----------|--------|------|
| 钱包管理 | 基本完成 | 70% | 缺少私钥字段、数据库持久化 |
| 环境配置 | 基本完成 | 60% | 指纹选项不完整、使用localStorage |
| 脚本编辑 | 基本完成 | 80% | 功能基本完整 |
| 执行面板 | 基本完成 | 70% | 模拟执行，非真实Playwright |
| 任务管理 | 基本完成 | 50% | 缺少真实Cron调度、环境分配 |
| 任务监控 | 基本完成 | 75% | 基本功能完整 |

### 9.2 功能差距分析

#### 9.2.1 钱包管理模块差距

**当前实现:**
- 仅存储 address、label、group、proxy
- 使用 localStorage 存储
- 支持 Excel 导入导出

**与设计文档差距:**
- ❌ 缺少 `name` 字段（钱包名称与备注应分开）
- ❌ 缺少 `private_key` 字段（关键功能缺失）
- ❌ 缺少 `chain_type` 字段
- ❌ 未使用 SQLite 数据库持久化
- ❌ 私钥加密存储未实现

**改进建议:**
```javascript
// 当前钱包数据结构
interface Wallet {
  id: number;
  address: string;
  label: string;      // 当前用作名称
  group: string;
  proxy: string;
}

// 应改为（与设计文档一致）
interface AirdropWallet {
  id: number;
  name: string;           // 钱包名称
  address: string;        // 钱包地址
  private_key: string;    // 加密存储的私钥
  label: string;          // 备注
  group_name: string;     // 分组
  proxy: string;          // 代理
  chain_type: 'evm' | 'solana';  // 链类型
}
```

#### 9.2.2 环境配置模块差距

**当前实现:**
- 存储 userAgent、viewport、proxy、canvasSpoof
- 使用 localStorage 存储
- 支持批量生成

**与设计文档差距:**
- ❌ 指纹保护选项不完整（当前仅 canvasSpoof 可配置）
- ❌ 缺少 WebGL、Audio、时区、地理位置、字体、WebRTC 配置
- ❌ 缺少代理认证配置（proxy_username, proxy_password）
- ❌ 缺少 locale、timezone_id、device_scale_factor
- ❌ 缺少 custom_headers、extensions 配置
- ❌ 未使用 SQLite 数据库持久化

**改进建议:**
```javascript
// 当前配置结构
interface BrowserProfile {
  id: number;
  name: string;
  userAgent: string;
  viewport: string;
  proxy: string;
  canvasSpoof: boolean;
}

// 应扩展为（UI已有部分但未绑定数据）
interface BrowserProfile {
  id: number;
  name: string;
  description?: string;
  
  // 浏览器基础配置
  user_agent: string;
  viewport_width: number;
  viewport_height: number;
  device_scale_factor: number;
  locale: string;
  timezone_id: string;
  
  // 代理配置
  proxy_type: 'direct' | 'http' | 'socks5';
  proxy_host?: string;
  proxy_port?: number;
  proxy_username?: string;
  proxy_password?: string;
  
  // 指纹保护配置
  canvas_spoof: boolean;
  webgl_spoof: boolean;
  audio_spoof: boolean;
  timezone_spoof: boolean;
  geolocation_spoof: boolean;
  font_spoof: boolean;
  webrtc_spoof: boolean;
  navigator_override: boolean;
  webdriver_override: boolean;
  
  // 高级配置
  custom_headers?: Record<string, string>;
  headless: boolean;
  extensions?: string[];  // ["metamask", "okxwallet"]
}
```

**UI 改进:**
当前 BrowserFarm.vue 编辑器中的 WebGL、Audio Context 开关是静态的 `default-checked`，需要绑定到实际数据模型。

#### 9.2.3 任务管理模块差距

**当前实现:**
- 简单的任务 CRUD
- 使用 localStorage 存储
- 无真实 Cron 调度

**与设计文档差距:**
- ❌ 无后端 Cron 调度器实现
- ❌ 缺少环境分配策略（random/sequential/specific）
- ❌ 缺少 wallet_ids 数组配置（当前只有简单选择）
- ❌ 缺少执行参数（concurrency、timeout_seconds、retry_times）
- ❌ 缺少任务状态跟踪（next_run_time、total_runs 等）
- ❌ 脚本和钱包选择是硬编码的静态选项

**改进建议:**
```javascript
// 当前任务结构
interface Task {
  id: number;
  name: string;
  schedule: string;
  script: string;      // 简单字符串
  wallets: string[];   // 简单数组
  enabled: boolean;
}

// 应改为
interface AutomationTask {
  id: number;
  name: string;
  description?: string;
  
  // 执行配置
  script_id: number;              // 关联脚本ID
  wallet_ids: number[];           // 关联钱包ID数组
  
  // 环境分配策略
  profile_strategy: 'random' | 'sequential' | 'specific';
  specific_profile_id?: number;
  
  // 调度配置
  schedule_type: 'once' | 'interval' | 'cron';
  schedule_config: {
    cron?: string;        // "0 9 * * *"
    interval?: number;    // 秒
    run_at?: string;      // ISO 时间字符串
  };
  
  // 执行参数
  concurrency: number;
  timeout_seconds: number;
  retry_times: number;
  retry_interval_seconds: number;
  
  // 状态
  status: 'draft' | 'enabled' | 'paused' | 'running';
  last_run_time?: string;
  next_run_time?: string;
  total_runs: number;
  success_runs: number;
  failed_runs: number;
}
```

#### 9.2.4 执行引擎差距

**当前实现:**
- 完全是模拟执行（simulateExecution）
- 未真正调用 Playwright

**与设计文档差距:**
- ❌ 无真实 Playwright 集成
- ❌ 无钱包扩展注入（MetaMask/OKX）
- ❌ 无指纹注入脚本
- ❌ 无 Node.js Bridge 实现

**改进建议:**
需要实现 `src-tauri/src/wallets_tool/playwright/mod.rs` 和对应的 Node.js 桥接层。

### 9.3 后端实现差距

**当前状态:**
- `src-tauri/src/wallets_tool/airdrop/mod.rs` 为空文件
- 无数据库表定义
- 无 Tauri Command 实现

**需要实现:**
1. SQLite 数据库表创建
2. 数据访问层（CRUD 操作）
3. 任务调度器（Cron）
4. Playwright Bridge
5. 私钥加密存储

### 9.4 优先级改进计划

#### Phase 1: 数据层迁移（高优先级）
1. 创建 SQLite 表结构（airdrop_wallets, browser_profiles, automation_tasks 等）
2. 实现 Tauri Command 进行数据库操作
3. 前端 Service 层改为调用 Tauri API
4. 数据迁移工具（localStorage -> SQLite）

#### Phase 2: 钱包管理增强（高优先级）
1. 添加 name、private_key、chain_type 字段
2. 实现私钥加密存储（使用系统 security 模块）
3. 更新 Excel 导入模板支持私钥列
4. 私钥安全显示 UI（掩码 + 复制）

#### Phase 3: 环境配置完善（中优先级）
1. 扩展 BrowserProfile 数据模型
2. 绑定 UI 中所有指纹保护开关到数据
3. 添加代理认证、高级配置 UI
4. 实现批量生成时的完整随机配置

#### Phase 4: 任务管理增强（中优先级）
1. 实现任务模型扩展
2. 脚本/钱包动态选择（从数据库加载）
3. 环境分配策略 UI 和逻辑
4. 执行参数配置 UI

#### Phase 5: 后端调度器（高优先级）
1. Rust 端 Cron 解析器
2. 任务队列管理
3. 定时执行触发
4. 执行状态持久化

#### Phase 6: Playwright 集成（高优先级）
1. Node.js Playwright 脚本执行器
2. Rust -> Node.js 桥接
3. 钱包扩展注入
4. 指纹保护脚本注入

### 9.5 代码修改清单

#### 前端需修改文件:
```
src/features/airdrop/
├── components/
│   ├── WalletManager.vue      # 添加私钥字段、数据库调用
│   ├── BrowserFarm.vue        # 完善指纹配置绑定、数据库调用
│   ├── TaskManager.vue        # 扩展任务配置、动态选择、数据库调用
│   ├── ExecutionPanel.vue     # 集成真实执行器
│   └── TaskMonitor.vue        # 优化（已基本完成）
└── services/
    └── playwrightService.ts   # 改为 Tauri API 调用
```

#### 后端需创建文件:
```
src-tauri/src/
├── wallets_tool/
│   ├── airdrop/
│   │   ├── mod.rs             # 模块入口
│   │   ├── models.rs          # 数据模型
│   │   ├── commands.rs        # Tauri Commands
│   │   ├── scheduler.rs       # 任务调度器
│   │   └── executor.rs        # 执行引擎
│   └── playwright/
│       ├── mod.rs             # Playwright 桥接
│       └── scripts/           # 注入脚本
└── database/
    └── airdrop_tables.sql     # 表结构定义
```
