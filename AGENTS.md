# WalletsTool Development Guide

**Generated:** 2026-01-30
**Commit:** d1f52e9 (dev branch)

## OVERVIEW

Web3 multi-chain wallet desktop app (Vue 3 + Tauri/Rust). Ethereum/Solana, batch import/transfer/balance, RPC/token config, Excel I/O, SQLite local storage. **Security-first architecture**: 私钥/助记词仅以加密形式落库，绝不明文持久化。

## BUILD COMMANDS

```bash
# Frontend-only
yarn dev                  # Vite dev server (port 1422)
yarn build                # Vite production build

# Full stack
yarn tauri-dev            # Full dev stack (recommended)
yarn tauri-build          # Production build (.msi/.dmg/.deb)

# Backend (Rust)
cargo test                # Run all tests
cargo test -- --nocapture # Run tests with output
cargo test <test_name>    # Run single test by name

# Setup
yarn start                # Install deps + dev server (first-time setup)
yarn setup                # Install dependencies only
yarn version:update       # Update version in package.json
```

## CODE STYLE GUIDELINES

### General Principles

- **Self-documenting code**: Avoid comments unless explaining complex algorithms
- **Consistency**: Match existing patterns in the file/module
- **Security-first**: Never log or persist sensitive data (keys, mnemonics)

### Imports and Dependencies

**Frontend (JavaScript):**
```javascript
// Absolute imports for @ alias (src directory)
import { useTransfer } from '@/features/ethereum/transfer/composables/useTransfer'

// Relative imports for local modules
import { formatAddress } from '@/utils/format'

// Vue imports
import { ref, computed, onMounted } from 'vue'

// Component imports (named exports)
import { Button, Modal, Table } from '@arco-design/web-vue'
```

**Backend (Rust):**
```rust
// Standard module imports
use tauri::{Manager, Runtime};

// Internal imports
use crate::wallets_tool::security::SecureMemory;
```

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Vue composables | `camelCase` prefixed with `use` | `useTransfer`, `useWallet` |
| Vue components | `PascalCase` | `WalletList.vue`, `TransferModal.vue` |
| Variables | `camelCase` | `walletAddress`, `gasPrice` |
| Constants | `UPPER_SNAKE_CASE` | `MAX_RETRIES`, `DEFAULT_RPC` |
| Database tables | `snake_case` | `wallet_groups`, `chain_configs` |
| Rust functions | `snake_case` | `get_chain_list`, `transfer_tokens` |
| Rust structs/enums | `PascalCase` | `ChainConfig`, `TransferStatus` |
| Chain types | lowercase string | `'evm'`, `'solana'` |

### Component Structure (Vue SFC)

```vue
<template>
  <!-- Template here -->
</template>

<script setup>
// Imports - alphabetical order by package then name
import { ref, computed, onMounted } from 'vue'
import { useWalletStore } from '@/stores/wallet'

// Props with defaults
const props = defineProps({
  modelValue: { type: String, default: '' },
  disabled: { type: Boolean, default: false }
})

// Emits
const emit = defineEmits(['update:modelValue', 'confirm'])

// Composables (use stores first)
const walletStore = useWalletStore()

// Refs and computed
const loading = ref(false)
const items = computed(() => walletStore.items)

// Methods (grouped logically, alphabetical within groups)

// Lifecycle
onMounted(() => {
  loadData()
})
</script>

<style scoped>
/* Local styles only */
</style>
```

### Error Handling

**Frontend:**
```javascript
try {
  await transferFunds(wallet)
} catch (error) {
  console.error('Transfer failed:', error.message)
  notify.error(`Transfer failed: ${error.message}`)
}
```

**Backend (Rust):**
```rust
// All Tauri commands return Result<T, String>
#[tauri::command]
async fn transfer<R: Runtime>(app: AppHandle<R>, request: TransferRequest) -> Result<TransferResult, String> {
    // Use ? operator or map_err for conversion
    let provider = get_provider(&request.chain_id).await.map_err(|e| e.to_string())?;
    
    // Explicit error handling for complex cases
    match execute_transfer(&provider, &request).await {
        Ok(tx) => Ok(tx),
        Err(e) => Err(format!("Transfer failed: {}", e)),
    }
}
```

### Type Handling

- **Frontend**: No TypeScript (`.js` files only). Use JSDoc for complex types if needed.
- **Backend**: Full Rust type safety required. No `unwrap()` on `Result` or `Option` in production code.
- Never suppress type errors: no `as any`, `@ts-ignore`, `.unwrap()` without context.

## STRUCTURE

```
./
├── src/                      # Vue 3 frontend (Feature-based)
│   ├── features/             # Business domains (Deep nesting)
│   │   ├── ethereum/         # EVM logic
│   │   ├── solana/           # SVM logic
│   │   └── airdrop/          # Browser Automation & Scripting
│   ├── pages/                # Route targets
│   ├── components/           # Global UI components
│   ├── composables/          # Global composables
│   ├── stores/               # Pinia stores
│   ├── utils/                # Utility functions
│   ├── main.js               # Entry (Plugins + Config)
│   └── App.vue               # Root + Global Events
├── src-tauri/                # Tauri backend (Library-style)
│   ├── src/wallets_tool/     # Core Business Logic
│   │   ├── ecosystems/       # Chain implementations
│   │   ├── security/         # AES-256 Memory Guard
│   │   ├── transfer/         # Transfer logic
│   │   └── wallet_manager/   # Wallet CRUD + encryption
│   ├── src/database/         # SQLite Service Layer
│   └── data/                 # DB file + init.sql
├── vcpkg/                    # C++ Dependencies
└── scripts/                  # Build/Version utilities
```

## WHERE TO LOOK

| Task | Location |
|------|----------|
| **Frontend Entry** | `src/main.js` |
| **Backend Entry** | `src-tauri/src/main.rs` |
| **Transfer Logic** | `src/features/{chain}/transfer/` |
| **Balance Logic** | `src/features/{chain}/balance/` |
| **Chain Backend** | `src-tauri/src/wallets_tool/ecosystems/` |
| **Security Core** | `src-tauri/src/wallets_tool/security/` |
| **Database Ops** | `src-tauri/src/database/` |
| **Wallet Manager** | `src-tauri/src/wallets_tool/wallet_manager/` |

## CONVENTIONS

- **Feature-First**: Frontend code lives in `src/features/{chain}/{domain}/`
- **Dual UI**: PrimeVue (Data/Lists) + Arco Design (Interactions/Modals)
- **JS over TS**: Frontend is strictly `.js`
- **Async Backend**: All Tauri commands return `Result<T, String>`
- **Chain Filtering**: Filter `get_chain_list` results by `ecosystem` ('evm' | 'solana')
- **Encrypted Secrets**: 私钥/助记词传输使用加密封装（`p1:` 或 `t1:`），不走明文

## ANTI-PATTERNS

- **NEVER** persist private keys to disk or DB in plain text
- **NEVER** log sensitive data (keys, mnemonics)
- **NEVER** mix Pinia state with local feature state unnecessarily
- **NEVER** block the main thread; use `tokio` for heavy lifting
- **NEVER** remove `custom-protocol` from `tauri.conf.json`
- **NEVER** use `as any`, `@ts-ignore`, or `.unwrap()` to suppress errors

## SECURITY RULES

1. **Memory-Only Keys**: Private keys never persisted, only in RAM
2. **Dynamic Encryption**: AES-256-CBC encryption in memory
3. **Zeroize**: Immediate memory wipe after signing
4. **Encrypted Persistence**: If needed, use Master Data Key (PBKDF2 derived from user password)
5. **IPC 传输加密**: 密码可用 RSA-OAEP 加密传输；批量导入/预览 secrets 使用 AES-256-GCM(`t1:`) 传输

## TOOLS AVAILABLE

| Tool | Purpose |
|------|---------|
| ESLint (via LSP) | JavaScript linting |
| cargo | Rust compilation and testing |
| vite | Frontend build |
| tauri | Desktop app packaging |

## VERIFICATION

Before submitting:
1. Run `lsp_diagnostics` on changed files
2. Run `cargo test` for Rust changes
3. Build succeeds with `yarn tauri-build`
