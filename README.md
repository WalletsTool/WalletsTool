# Wallet Manager

一个功能强大的多链钱包管理工具，基于 Vue 3 + Tauri 构建，支持钱包导入、余额查询、批量转账等功能。提供直观的用户界面，帮助用户安全便捷地管理多个区块链网络上的数字资产。

<div align="center">
  <img src="app-icon.png" width="80" height="80" alt="应用截图"/>
</div>

## ✨ 功能特性

### 🔗 多链支持

* **Ethereum** - 以太坊主网及测试网

* **Solana** - Solana 主网及测试网

* **自定义链** - 支持添加自定义区块链网络

* **链配置管理** - RPC 节点管理、代币配置

### 💰 钱包管理

* **批量导入** - 支持私钥文本粘贴和 Excel 文件导入

* **多对多转账** - 支持批量转账操作，模拟正常用户行为

* **余额查询** - 多链余额实时查询，支持原生代币和 ERC-20 代币

* **数据导出** - 支持查询结果导出为 Excel 报表

### ⚡ 高级功能

* **智能 Gas 管理** - 自动 Gas 估算和优化

* **交易监控** - 实时交易状态跟踪

* **失败重试** - 智能重试机制，避免重复转账

* **多窗口支持** - 支持同时打开多个功能窗口

### 🛡️ 安全特性

* **本地存储** - 所有数据本地 SQLite 数据库存储

* **私钥保护** - 私钥仅在内存中处理，不明文存储

* **交易验证** - 完整的交易验证和确认机制

## 🏗️ 技术架构

### 前端技术栈

* **Vue 3** - 现代化前端框架

* **Vite** - 快速构建工具

* **PrimeVue** - UI 组件库

* **Arco Design** - 补充 UI 组件

* **ethers.js** - 以太坊交互库

* **party-js** - 动画效果库

* **xlsx** - Excel 文件处理

### 后端技术栈

* **Tauri** - 跨平台桌面应用框架 (Rust)

* **SQLite** - 轻量级本地数据库

* **tokio** - 异步运行时

### 项目结构

```
wallet_manager/
├── src/                    # 前端源码
│   ├── components/         # 公共组件
│   ├── features/          # 功能模块
│   │   ├── ethereum/      # 以太坊相关功能
│   │   ├── solana/        # Solana 相关功能
│   │   └── home/          # 主页
│   ├── router/            # 路由配置
│   └── stores/            # 状态管理
├── src-tauri/             # Tauri 后端
│   ├── src/               # Rust 源码
│   ├── data/              # 数据库文件
│   └── icons/             # 应用图标
├── public/                # 静态资源
└── tests/                 # 测试文件
```

## 🚀 快速开始

### 环境要求

* **Node.js** >= 18.0.0

* **Yarn** 包管理器

* **Rust** >= 1.70.0

* **Tauri CLI** >= 2.0.0

### 安装依赖

```bash
# 安装前端依赖
yarn install

# 安装 Tauri CLI (如果未安装)
cargo install tauri-cli --version "^2.0.0"
```

### 开发模式

```bash
# 启动开发服务器
yarn tauri-dev
```

### 构建应用

```bash
# 构建生产版本
yarn tauri-build
```

## 📖 使用指南

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
3. 执行批量余额查询
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

## 🗂️ 数据库结构

### 主要数据表

* **chains** - 区块链网络配置

* **rpc\_providers** - RPC 节点配置

* **tokens** - 代币配置信息

* **monitor\_configs** - 监控配置（未来功能）

* **monitor\_history** - 监控历史（未来功能）

## 🛣️ 开发路线图

### 已完成功能 ✅

* [x] 基础钱包管理

* [x] 多链余额查询

* [x] 批量转账功能

* [x] 链配置管理

* [x] 数据导入导出

### 开发中功能 🚧

* [ ] 极速分发功能

* [ ] 链上地址监控

* [ ] 私钥加密存储

* [ ] 多线程转账优化

### 计划功能 📋

* [ ] CEX 工具集成 (Binance, OKX, Bybit)

* [ ] DEX 工具集成 (Uniswap, SushiSwap)

* [ ] 合约监控功能

* [ ] 自动抢购机制

* [ ] 消息通知系统

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

* [ethers.js 文档](https://docs.ethers.org/)

## ⚠️ 免责声明

本工具仅供学习和研究使用，使用者需要自行承担使用风险。请确保在使用前充分了解相关区块链网络的规则和风险，谨慎操作您的数字资产。

***

如有问题或建议，欢迎提交 Issue 或联系开发团队。
