# WalletsTool Development Guide

**Generated:** 2026-01-13
**Commit:** d1f52e9 (dev branch)

## OVERVIEW

Web3 multi-chain wallet desktop app (Vue 3 + Tauri/Rust). Ethereum/Solana, batch import/transfer/balance, RPC/token config, Excel I/O, SQLite local storage (private keys in memory only).

## STRUCTURE

```
./
├── src/                      # Vue 3 frontend
│   ├── features/{eth,sol}/   # Feature-based architecture
│   ├── components/           # Shared components
│   ├── stores/               # Pinia state
│   └── router/               # Hash routing
├── src-tauri/                # Tauri backend
│   ├── src/wallets_tool/     # Business logic
│   │   └── ecosystems/       # Chain implementations
│   └── data/                 # SQLite DB + init.sql
├── scripts/                  # Build utilities
└── .github/workflows/        # CI (multi-platform)
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Frontend entry | `src/main.js` | PrimeVue + Arco dual UI |
| Backend entry | `src-tauri/src/main.rs` | 37+ Tauri commands |
| Transfer logic | `src/features/ethereum/transfer/` | Composables-heavy |
| Ethereum backend | `src-tauri/src/wallets_tool/ecosystems/ethereum/` | 9 modules (Alloy migration) |
| Database ops | `src-tauri/src/database/` | Service pattern (ChainService, RpcService) |
| Configs | `src-tauri/Cargo.toml` | Rust deps + profiles |
| Build | `vite.config.js` | 213 lines, manual chunk splitting |

## CONVENTIONS (DEVIATIONS FROM STANDARD)

**JavaScript (not TypeScript):** Frontend uses `.js` despite 2025 project

**Dual UI libraries:** PrimeVue (primary) + Arco Design (tabs/modals/tooltips)

**Deep nesting:** `src/features/{ecosystem}/{feature}/{pages,components,composables,styles}/`

**2-space indentation:** JavaScript/Vue; Rust uses standard

**No comments:** "DO NOT ADD COMMENTS unless explicitly required"

**Semicolons:** Required in JavaScript

## ANTI-PATTERNS (THIS PROJECT)

- Never log or expose private keys
- Never persist private keys (memory-only)
- Never remove `custom-protocol` feature from Cargo.toml

## UNIQUE STYLES

**Feature structure:**
```
features/{ecosystem}/{feature}/
├── pages/          # Route targets
├── components/     # Feature-local UI
├── composables/    # Business logic (barrel exports)
└── styles/         # Feature CSS (barrel exports)
```

**Backend module organization:**
```
ecosystems/{chain}/
├── mod.rs          # Module declarations
├── chain_config.rs # Chain metadata
├── provider.rs     # RPC provider mgmt
├── transfer.rs     # Native coin transfer
├── token_transfer.rs # ERC-20 transfers
├── rpc_management.rs # RPC load balancing
├── simple_balance_query.rs
└── proxy_manager/proxy_commands # HTTP proxy
```

**Tauri commands:** All async, return `Result<T, String>`

**Database:** SQLite at `data/wallets_tool.db`, hot-reload via `reload_database()` command

## COMMANDS

```bash
yarn start          # Auto-install deps + tauri-dev
yarn tauri-dev      # Full dev (frontend + backend)
yarn tauri-build    # Production desktop build
yarn version:update <version> # Bump all versions
```

## TESTING

Playwright installed (`@playwright/test` v1.57.0).
Rust unit tests added in `src-tauri/src/wallets_tool/ecosystems/ethereum/alloy_utils.rs`.
Run with `cargo test`.

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `useTransfer` | composable | `features/ethereum/transfer/composables/` | Batch transfer logic |
| `ChainService` | struct | `database/chain_service.rs` | Chain CRUD |
| `RpcService` | struct | `database/rpc_service.rs` | RPC CRUD |
| `init_database` | fn | `database/mod.rs` | DB initialization |

## NOTES

- Database config in `package.json` (`config.database`) - unusual coupling
- Multi-window support (main + child windows)
- System tray integration
- "Fury mode" for high-concurrency transfers (>90 threads)
- Intelligent retry with on-chain tx detection
- Proxy manager with HTTP/SOCKS5 support
- Migrated to Alloy framework (Jan 2026)
