# Repository Guidelines

## 项目结构与模块组织
- 前端：`src/`（Vue 3 + Vite）。页面：`src/features/<ecosystem>/<feature>/pages/`；公用组件：`src/components/`；状态：`src/stores/`；路由：`src/router/`。
- 桌面/Rust：`src-tauri/`。核心逻辑：`src-tauri/src/wallets_tool/`（含 `ecosystems/ethereum`、`solana`）；数据库层：`src-tauri/src/database/`；插件：`src-tauri/src/plugins/`；配置：`src-tauri/tauri.conf.json`、`Cargo.toml`。
- 资源与脚本：静态资源 `public/`；图标 `src-tauri/icons/`；脚本 `scripts/`；文档 `docs/`；数据库文件 `src-tauri/data/wallets_tool.db`（初始化 SQL：`src-tauri/data/init.sql`）。

## 构建、测试与本地开发
- `yarn start`：安装依赖并运行 `tauri dev`（桌面调试）。
- `yarn dev` / `yarn build` / `yarn preview`：前端开发/打包/本地预览。
- `yarn tauri-build`：桌面打包生成安装包。
- 实用脚本：`yarn setup`（安装依赖）、`yarn icon`（生成应用图标）、`yarn version:update`（同步版本）。
- Rust 子项目：在 `src-tauri/` 下执行 `cargo build`、`cargo test`（通常由 Tauri CLI 间接触发）。

## 代码风格与命名约定
- Vue 组件文件使用 PascalCase（如 `TitleBar.vue`）；Pinia store 用名词复数（如 `stores/ecosystems.ts`）。
- JS/Vue 使用 2 空格缩进，优先组合式 API；避免未使用的变量/样式。
- Rust 遵循 `rustfmt`；模块按领域分层（如 `ecosystems/ethereum/*`）。

## 测试指南
- E2E：Playwright，运行 `npx playwright test`。 
- 前端单元：可选 Vitest，命名 `*.spec.ts|js` 或 `__tests__/` 目录。
- Rust 单元：在模块中使用 `#[cfg(test)]` 内联测试。
- 覆盖：涉及链上交互与数据库改动必须补充基本用例。

## Commit 与 Pull Request
- 提交信息：`scope: 简述变更`，使用祈使句（示例：`ethereum: 修复余额查询超时`）。
- PR 要求：说明目的与影响范围，关联 Issue（如 `#123`）；列出关键改动与风险；UI 变更附截图；性能/稳定性改动附基准或日志；涉及配置/数据库时注明迁移与回滚步骤。

## 安全与配置提示
- 禁止提交私钥、RPC 密钥或导入的钱包数据。
- RPC/链配置在 `wallets_tool/*/rpc_management.rs` 与 `chain_config.rs`；修改需评估兼容性与回放方案。
