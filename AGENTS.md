# WalletsTool Development Guide

**Generated:** 2026-01-27
**Commit:** d1f52e9 (dev branch)

## OVERVIEW

Web3 multi-chain wallet desktop app (Vue 3 + Tauri/Rust). Ethereum/Solana, batch import/transfer/balance, RPC/token config, Excel I/O, SQLite local storage. **Security-first architecture**: Private keys are memory-only, never persisted.

## STRUCTURE

```
./
├── src/                      # Vue 3 frontend (Feature-based)
│   ├── features/             # Business domains (Deep nesting)
│   │   ├── ethereum/         # EVM logic
│   │   ├── solana/           # SVM logic
│   │   └── airdrop/          # Browser Automation & Scripting
│   ├── main.js               # Entry (Plugins + Config)
│   └── App.vue               # Root + Global Events
├── src-tauri/                # Tauri backend (Library-style)
│   ├── src/wallets_tool/     # Core Business Logic
│   │   ├── ecosystems/       # Chain implementations
│   │   └── security/         # AES-256 Memory Guard
│   ├── src/database/         # SQLite Service Layer
│   └── data/                 # DB file + init.sql
├── vcpkg/                    # C++ Dependencies
└── scripts/                  # Build/Version utilities
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| **Frontend Entry** | `src/main.js` | PrimeVue + Arco + Router setup |
| **Backend Entry** | `src-tauri/src/main.rs` | Security init + Command registry |
| **Transfer Logic** | `src/features/{chain}/transfer/` | Feature-scoped pages/composables |
| **Balance Logic** | `src/features/{chain}/balance/` | Feature-scoped pages |
| **Chain Backend** | `src-tauri/src/wallets_tool/ecosystems/` | Modularized by chain (eth/sol) |
| **Security Core** | `src-tauri/src/wallets_tool/security/` | `SecureMemory` implementation |
| **Database Ops** | `src-tauri/src/database/` | `ChainService`, `RpcService` |
| **Build Config** | `vite.config.js` | Manual chunks + Polyfills |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `useTransfer` | composable | `*/transfer/composables/` | Core batch logic (Frontend) |
| `ChainService` | struct | `database/chain_service.rs` | Chain CRUD (Backend) |
| `SecureMemory` | struct | `security/mod.rs` | RAM-only key storage |
| `iterTransfer` | func | `useTransfer.ts` | Batch loop executor |
| `fury_mode` | logic | `*/transfer.rs` | High-concurrency executor |

## CONVENTIONS

- **Feature-First**: Frontend code lives in `src/features/{chain}/{domain}/`.
- **Dual UI**: **PrimeVue** (Data/Lists) + **Arco Design** (Interactions/Modals).
- **JS over TS**: Frontend is strictly `.js` (legacy decision).
- **No Comments**: Code must be self-documenting. Comments only for complex algos.
- **Async Backend**: All Tauri commands return `Result<T, String>`.
- **Chain Filtering**: Frontend components must filter `get_chain_list` results by `ecosystem` ('evm' or 'solana').
- **Wallet Manager Groups**: `create_group` request requires `chain_type` and returns group id (`i64`).
- **Wallet Manager Wallets**: 地址由后端根据私钥/助记词自动派生并落库；批量创建统一走 `create_wallets`，助记词派生钱包会记录 `mnemonic_index`；私钥/助记词入参使用 `sealed_*` 加密传输。
- **Wallet Manager get_wallets**: `get_wallets(group_id, chain_type, password)` 支持按生态（chain_type）过滤；系统生态分组查询应传 `group_id=null` 且带 `chain_type`；不会返回明文私钥/助记词，仅返回 `has_private_key/has_mnemonic` 用于状态展示。

## ANTI-PATTERNS (THIS PROJECT)

- **Security**: 
  - **General**: NEVER persist private keys to disk or DB in plain text.
  - **Wallet Manager**: Encrypted persistence is allowed using Master Data Key (AES-256-CBC) derived from user password (PBKDF2). Keys are only decrypted in memory when unlocked.
- **Logging**: NEVER log sensitive data (keys, mnemonics).
- **State**: NEVER mix Pinia state with local feature state unnecessarily.
- **Concurrency**: NEVER block the main thread; use `tokio` for heavy lifting.
- **Config**: NEVER remove `custom-protocol` from `tauri.conf.json`.

## UNIQUE STYLES

**Frontend Feature Structure**:
```
features/{chain}/{domain}/
├── pages/          # Route targets
├── components/     # Local UI
├── composables/    # Business logic
└── styles/         # Local CSS
```

**Backend Ecosystem Structure**:
```
ecosystems/{chain}/
├── mod.rs          # Exports
├── transfer.rs     # Native logic
├── token_transfer.rs # Token logic
└── provider.rs     # Connection mgmt
```

## COMMANDS

```bash
yarn start          # Install deps + Dev server
yarn tauri-dev      # Full dev stack
yarn tauri-build    # Production build
cargo test          # Backend unit tests
```

## NOTES

- **Fury Mode**: >90 threads for mass transfers.
- **Database**: `package.json` config triggers `init.sql` checks.
- **Alloy**: ETH backend migrated to Alloy framework (Jan 2026).
