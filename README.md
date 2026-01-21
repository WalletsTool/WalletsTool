# Wallets Tool
[![GitHub release](https://img.shields.io/github/v/release/WalletsTool/WalletsTool)](https://github.com/WalletsTool/WalletsTool/releases)
[![Release](https://github.com/WalletsTool/WalletsTool/actions/workflows/release.yml/badge.svg)](https://github.com/WalletsTool/WalletsTool/actions/workflows/release.yml)
![Tauri](https://img.shields.io/badge/Tauri-Rust-orange) ![Vue 3](https://img.shields.io/badge/Vue-3-42b883) ![Desktop](https://img.shields.io/badge/Platform-Desktop-blue)
<p align="center">
  <a href="https://github.com/WalletsTool/WalletsTool/releases">快速开始 | Download</a>
</p>

> [!WARNING]
> 本项目仅供区块链工作室交流学习使用。使用本工具需自行承担风险，请确保在使用前充分了解相关区块链网络的规则和风险，谨慎操作您的数字资产。

<div align="center"  style="margin-bottom: 20px;">
    <img src="app-icon.png" width="80" height="80" alt="应用logo"/>
</div>

Web3 多链钱包管理工具（Vue 3 + Tauri/Rust），支持 Ethereum/Solana，批量导入、余额批量查询、批量转账、RPC/代币配置、Excel 导入导出，SQLite 本地存储（私钥不落盘）。

Web3 multi-chain wallet manager (Vue 3 + Tauri/Rust). Supports Ethereum & Solana, batch import/transfer/balance check, RPC/Token config, Excel I/O, local SQLite with no private key persistence.

## 🏆 为什么选择本地桌面应用？

### 🔒 **极致安全性**
| 本地桌面应用 | 浏览器在线工具 |
|-------------|---------------|
| ✅ **私钥永不离开设备** - 所有敏感数据仅在本地内存中处理，不做任何传输和存储 | ❌ 私钥可能通过网络传输，存在泄露风险 |
| ✅ **无第三方依赖** - 完全离线运行，不依赖外部服务 | ❌ 依赖第三方服务器，存在服务商风险 |
| ✅ **代码开源透明** - 完全开源，可审计安全性 | ❌ 后端代码不透明，无法验证安全性 |
| ✅ **永久可用** - 一次安装，永久使用 | ❌ 服务商可能随时停止服务 |
| ✅ **数据完全自主** - 所有数据存储在本地，完全掌控 | ❌ 数据依赖云端服务，可能面临服务中断 |
| ✅ **无审查风险** - 不受网络封锁或服务商限制影响 | ❌ 可能面临服务商封禁或地区限制 |

### ⚡ **卓越性能**
| 本地桌面应用 | 浏览器在线工具 |
|-------------|---------------|
| ✅ **原生性能** - Rust + Tauri 提供接近原生的执行效率 | ❌ 受浏览器性能限制，执行效率较低 |
| ✅ **批量操作优化** - 多线程并发处理，速度更快 | ❌ 受浏览器并发限制，批量操作缓慢 |
| ✅ **内存管理优化** - 精确的内存控制，避免内存泄露 | ❌ 受浏览器内存管理限制 |
| ✅ **无网络延迟** - 本地计算，响应即时 | ❌ 依赖网络请求，存在延迟和超时风险 |

### 🎯 **专业功能**
| 本地桌面应用 | 浏览器在线工具 |
|-------------|---------------|
| ✅ **系统级集成** - 深度集成操作系统，提供更好体验 | ❌ 受浏览器沙盒限制，功能受限 |
| ✅ **文件系统访问** - 直接读写本地文件，支持 Excel 导入导出 | ❌ 文件操作受限，用户体验较差 |
| ✅ **多窗口支持** - 支持同时打开多个功能窗口 | ❌ 受浏览器标签页限制 |
| ✅ **系统通知** - 原生系统通知，及时提醒交易状态 | ❌ 通知功能受限且不稳定 |

### 💡 **用户体验优势**
- **🚀 一键启动** - 使用 `yarn start` 命令自动安装依赖并启动开发环境
- **🔧 零配置** - 自动检测和安装所需依赖（Node.js、Yarn、Rust、Cargo）
- **📱 原生界面** - 完全原生的桌面应用体验，不是网页套壳
- **🎨 自定义主题** - 支持深度定制界面主题和布局
- **⌨️ 快捷键支持** - 完整的键盘快捷键支持，提高操作效率

## ✨ 功能特性

### 🔗 多链支持

* **Ethereum** - 以太坊主网及测试网

* **Solana** - Solana 主网及测试网（进行中）

* **自定义链** - 支持添加自定义区块链网络

* **链配置管理** - RPC 节点管理、代币配置

### 💰 钱包管理

* **批量导入** - 支持私钥文本粘贴和 Excel 文件导入

* **多对多转账** - 支持批量转账操作，模拟正常用户行为

* **余额查询** - 多链余额实时查询，支持原生代币和 ERC-20 代币

* **数据导出** - 支持查询结果导出为 Excel 报表

### ⚡ 高级功能

* **极速转账 (Fury Mode)** - 90+ 线程并发处理，极速批量转账
* **智能 Gas 管理** - 自动 Gas 估算和优化
* **交易监控** - 实时交易状态跟踪，链上交易检测
* **智能重试** - 智能重试机制，避免重复转账
* **多窗口支持** - 支持主窗口 + 子窗口同时操作
* **系统托盘** - 最小化到托盘运行
* **代理支持** - HTTP/SOCKS5 代理配置
* **数据热重载** - 数据库配置热更新 (`reload_database` 命令)

### 🛡️ 安全特性

* **本地存储** - 所有数据本地 SQLite 数据库存储（数据库仅保存链信息、币信息和rpc信息，钱包私钥仅在内存中处理，不会存储在数据库中）


* **私钥保护** - 私钥仅在内存中处理，不做任何存储

* **交易验证** - 完整的交易验证和确认机制

## 🏗️ 技术架构

### 前端技术栈

* **Vue 3** - 现代化前端框架 (Composition API)

* **Vite** - 极速构建工具 (手动分包优化)

* **PrimeVue** - 主要 UI 组件库

* **Arco Design** - 补充 UI 组件 (标签/弹窗/提示)

* **Pinia** - 状态管理

* **Vue Router** - 路由管理 (Hash 模式)

* **ethers.js** - 以太坊交互库

* **solana-web3.js** - Solana 交互库

* **xlsx** - Excel 文件导入导出

* **party-js** - 动画效果库

### 后端技术栈

* **Tauri 2.0** - 跨平台桌面应用框架 (Rust)

* **SQLite** - 轻量级本地数据库 (数据目录: `src-tauri/data/`)

* **tokio** - 异步运行时 (高性能并发)

* **reqwest** - HTTP 客户端 (代理支持)

* **SQLx** - 异步 SQL 框架

* **serde** - 序列化/反序列化

### 项目结构

```
WalletsTool/
├── src/                      # Vue 3 前端源码
│   ├── features/             # 功能模块目录
│   │   └── {ecosystem}/      # 生态系统模块 (ethereum/solana)
│   │       └── {feature}/    # 功能模块 (transfer/balance/import等)
│   │           ├── pages/    # 路由页面
│   │           ├── components/ # 功能组件
│   │           ├── composables/ # 业务逻辑 (桶式导出)
│   │           └── styles/   # 样式文件
│   ├── components/           # 公共组件
│   ├── stores/               # Pinia 状态管理
│   └── router/               # 路由配置
├── src-tauri/                # Tauri 后端
│   ├── src/
│   │   ├── main.rs           # 后端入口 (37+ Tauri 命令)
│   │   ├── wallets_tool/     # 业务逻辑
│   │   │   └── ecosystems/   # 链实现模块
│   │   │       └── {chain}/  # 区块链实现
│   │   │           ├── mod.rs
│   │   │           ├── chain_config.rs
│   │   │           ├── provider.rs
│   │   │           ├── transfer.rs
│   │   │           ├── token_transfer.rs
│   │   │           ├── rpc_management.rs
│   │   │           ├── simple_balance_query.rs
│   │   │           └── proxy_manager/  # HTTP/SOCKS5 代理
│   │   └── database/         # 数据库服务
│   │       ├── mod.rs        # init_database 初始化函数
│   │       ├── chain_service.rs  # ChainService 链CRUD
│   │       └── rpc_service.rs    # RpcService RPC CRUD
│   └── data/                 # SQLite 数据库 + init.sql
├── scripts/                  # 构建工具 (version:update 版本更新)
└── .github/workflows/        # CI (多平台)
```

### 代码规范

* **JavaScript (非 TypeScript):** 前端使用 `.js` 文件
* **双 UI 库:** PrimeVue (主要) + Arco Design (标签/弹窗/提示)
* **缩进:** JavaScript/Vue 使用 2 空格，Rust 使用标准缩进
* **无注释原则:** "DO NOT ADD COMMENTS unless explicitly required"
* **分号:** JavaScript 代码必须使用分号
* **Tauri 命令:** 所有命令异步执行，返回 `Result<T, String>`

### 重要文件索引

| 符号 | 类型 | 位置 | 职责 |
|------|------|------|------|
| `useTransfer` | composable | `features/ethereum/transfer/composables/` | 批量转账逻辑 |
| `useBalance` | composable | `features/ethereum/balance/composables/` | 余额查询逻辑 |
| `ChainService` | struct | `database/chain_service.rs` | 链配置 CRUD |
| `RpcService` | struct | `database/rpc_service.rs` | RPC 配置 CRUD |
| `init_database` | fn | `database/mod.rs` | 数据库初始化 |
| 前端入口 | - | `src/main.js` | PrimeVue + Arco 双 UI |
| 后端入口 | - | `src-tauri/src/main.rs` | 37+ Tauri 命令 |
| 构建配置 | - | `vite.config.js` | 手动分包 (213行) |

## 🚀 快速开始

### 🎯 一键启动（推荐）

```bash
# 自动安装所有依赖并启动开发环境
yarn start
```

> **💡 智能依赖管理**：`yarn start` 命令会自动检测并安装所需的所有依赖项，包括 Node.js、Yarn、Rust 和 Cargo，然后启动开发环境。无需手动配置！

### 📋 环境要求

以下依赖项将通过 `yarn start` 自动安装和配置：

* **Node.js** >= 18.0.0
* **Yarn** 包管理器
* **Rust** >= 1.70.0
* **Tauri CLI** >= 2.0.0

### 🔧 手动安装（可选）

如果您希望手动安装依赖项：

```bash
# 安装前端依赖
yarn install

# 安装 Tauri CLI (如果未安装)
cargo install tauri-cli --version "^2.0.0"
```

### 🚀 开发模式

```bash
# 启动开发服务器
yarn tauri-dev
```

### 📦 构建应用

```bash
# 构建生产版本
yarn tauri-build
```

### 🔖 版本更新

```bash
# 自动更新版本号并创建 Git 标签
yarn version:update <version>

# 示例
yarn version:update 1.2.3
yarn version:update v1.0.0-beta.1
```

版本更新脚本会自动更新 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock` 中的版本号，创建 Git 标签并推送到远程仓库。

### 🛠️ 故障排除

如果遇到依赖安装问题：

1. **清理缓存**：
   ```bash
   yarn cache clean
   rm -rf node_modules
   yarn install
   ```

2. **重新安装 Rust**：
   ```bash
   rustup update
   cargo install tauri-cli --version "^2.0.0" --force
   ```

3. **检查系统环境**：
   - 确保已安装 Visual Studio Build Tools (Windows)
   - 确保已安装 Xcode Command Line Tools (macOS)
   - 确保已安装 build-essential (Linux)

4. **使用依赖安装脚本**：
   ```bash
   node scripts/install-deps.js
   ```

## 📖 使用指南
<div align="center">
  <br/>
    <span>主页截图</span>
  <br/>
    <br/>
    <img src="docs/screenshot/app-screenshot_1.png" alt="应用截图"/>
    <br/>
      <br/>
    <span>批量转账页面截图</span>
    <br/>
      <br/>
    <img src="docs/screenshot/app-screenshot_2.png" alt="应用截图"/>
</div>

### 1. 链配置管理

1. 打开应用后，点击「链管理」按钮
2. 添加或编辑区块链网络配置
3. 配置 RPC 节点和代币信息

### 2. 批量转账

1. 选择目标区块链网络
2. 导入发送方私钥和接收方地址
3. 配置转账参数（数量、Gas 设置等）
4. 开始执行批量转账

### 3. 余额查询

1. 导入要查询的钱包地址
2. 选择要查询的代币类型
3. 可选：开启“查询最后交易时间”开关以同时查询最后交易时间（默认关闭）
4. 执行批量余额查询

#### 配置参数：查询最后交易时间

- `queryLastTransactionTime`（前端组件 props / 表单项）
  - 类型：`boolean`
  - 默认值：`false`
  - 作用：开启后端对每个地址的“最后交易时间”查询；关闭时跳过该步骤，提高查询速度。
- 传参与行为：
  - 前端会在查询参数中传递 `query_last_transaction_time` 字段到后端。
  - 后端按该布尔值决定是否执行最后交易时间查询。

示例（在 Balance 页面中使用）：

```vue
<Balance :queryLastTransactionTime="false" />
```

在页面右上方的配置区域也提供了开关项“查询最后交易时间”，用于动态控制此行为。
4. 导出查询结果为 Excel 文件

## 🔧 开发指南

### 添加新的区块链支持

1. 在 `src/features/` 下创建新的链目录
2. 实现转账和余额查询功能
3. 在数据库中添加链配置
4. 更新路由配置

### 自定义组件开发

```vue
<template>
  <div class="custom-component">
    <!-- 组件内容 -->
  </div>
</template>

<script setup>
// 组件逻辑
</script>
```

### 数据库操作

```rust
// 在 Rust 后端中操作数据库
use crate::database::chain_service::ChainService;

#[tauri::command]
async fn get_chains() -> Result<Vec<Chain>, String> {
    // 数据库操作逻辑
}
```

## 🧪 测试

* **Playwright** - 已安装 (`@playwright/test` v1.57.0)
* **当前状态:** 暂无测试文件，欢迎贡献测试用例

```bash
# 安装 Playwright 浏览器
npx playwright install

# 运行测试 (未来)
npx playwright test
```

## 🛣️ 开发路线图

### 已完成功能 ✅

* [x] 基础钱包管理

* [x] 多链余额查询

* [x] 批量转账功能

* [x] 链配置管理

* [x] 数据导入导出

### 开发中功能 🚧

* [ ] Solana 完整支持

* [ ] 极速分发功能 (Fury Mode 优化)

* [ ] 链上地址监控

* [ ] 私钥加密存储

* [ ] 多线程转账优化 (>90 线程)

### 计划功能 📋

* [ ] CEX 工具集成 (Binance, OKX, Bybit)

* [ ] DEX 工具集成 (Uniswap, SushiSwap, Raydium)

* [ ] 合约交互功能

* [ ] 批量空投工具

* [ ] 交易历史追踪

* [ ] 消息通知系统 (系统通知 + 邮件)

* [ ] 主题自定义

## 🤝 贡献指南

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 📄 许可证

Copyright © 2025 EzBan. All rights reserved.

## 🔗 相关链接

* [Tauri 官方文档](https://tauri.app/)

* [Vue 3 官方文档](https://vuejs.org/)

* [PrimeVue 组件库](https://primevue.org/)

* [Arco Design 组件库](https://arco.design/)

* [Pinia 状态管理](https://pinia.vuejs.org/)

* [ethers.js 文档](https://docs.ethers.org/)

* [Solana Web3.js](https://docs.solana.com/developing/clients/javascript-api)

* [SQLx (Rust DB)](https://github.com/launchbadge/sqlx)

## ⚠️ 免责声明

本工具仅供学习和研究使用，使用者需要自行承担使用风险。请确保在使用前充分了解相关区块链网络的规则和风险，谨慎操作您的数字资产。

***

如有问题或建议，欢迎提交 Issue 或联系开发团队。
