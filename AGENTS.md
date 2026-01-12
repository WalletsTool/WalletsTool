# AGENTS.md - WalletsTool Development Guide

## Project Overview

WalletsTool is a Web3 multi-chain wallet management desktop application built with Vue 3 + Tauri (Rust). It supports Ethereum and Solana ecosystems with features like batch wallet import, balance queries, batch transfers, RPC/token configuration, and Excel import/export with local SQLite storage.

## Development Commands

### Frontend (Vue 3 + Vite)
- `yarn dev` - Start Vite development server only (port 1422)
- `yarn build` - Build frontend for production
- `yarn preview` - Preview production build

### Tauri Desktop App
- `yarn tauri-dev` - Start Tauri development server (combines frontend + backend)
- `yarn tauri-build` - Build production desktop application

### Utilities
- `yarn start` or `yarn setup` - Auto-install dependencies and start dev environment
- `node scripts/install-deps.cjs` - Manually run dependency installation
- `yarn version:update <version>` - Update version across package.json and Cargo.toml

### Running Tests
This project uses Playwright for E2E tests:
- `yarn playwright test` - Run all tests
- `yarn playwright test --project=chromium` - Run tests in specific browser
- `yarn playwright test <filename>` - Run specific test file

## Code Style Guidelines

### Vue 3 / JavaScript

**Imports:**
- Use `@/` alias for imports from `src/` directory: `import { useStore } from '@/stores/ecosystem'`
- Third-party imports first, then local imports
- Group imports by source (vue, @tauri-apps, @arco-design, local)

**Naming Conventions:**
- Components: PascalCase (`WalletImportModal.vue`)
- Variables/functions: camelCase (`windowCount`, `checkDatabaseStatus()`)
- Props: camelCase (consistent with JavaScript)
- Constants: SCREAMING_SNAKE_CASE or camelCase for local consts
- CSS classes: kebab-case (`.custom-titlebar`, `.func-card`)

**Component Structure (script setup):**
```vue
<script setup name="ComponentName">
// Imports
// Props/Emits
// Composables/Stores
// Refs/Computed
// Lifecycle hooks
// Methods
</script>

<template>
<!-- Template content -->
</template>

<style scoped>
/* Component styles */
</style>
```

**Template Style:**
- Use `v-for` with `:key` (prefer index only if no unique id)
- Prefer composition API helpers (computed, ref, watch)
- Use `async` for methods calling backend commands
- Check Tauri environment with: `window.__TAURI_INTERNALS__`

### Rust / Tauri

**Modules:**
- Files: snake_case (`rpc_management.rs`, `simple_balance_query.rs`)
- Modules: snake_case (defined in `mod.rs`)

**Naming Conventions:**
- Structs/Enums/Traits: PascalCase (`TransactionRequest`, `ProviderState`)
- Functions/Variables: snake_case (`get_balance`, `rpc_providers`)
- Constants: SCREAMING_SNAKE_CASE
- Macros: snake_case

**Error Handling Pattern:**
```rust
#[tauri::command]
async fn function_name(param: Type) -> Result<ReturnType, String> {
    // Implementation
}
```
- Return `Result<T, String>` for all public commands
- Use `?` operator for early returns on errors
- Convert errors to descriptive strings for frontend

**Module Organization:**
- Each ecosystem has its own module: `ecosystems/ethereum/`, `ecosystems/solana/`
- Shared utilities in `wallets_tool/utils.rs`
- Database operations in `database/` module

**Tauri Commands:**
- Mark with `#[tauri::command]` attribute
- Must be async
- Register in `main.rs` via `invoke_handler!`
- Bridge frontend via Tauri's `invoke()` API

### General Guidelines

**Comments:**
- DO NOT ADD COMMENTS unless explicitly required by task
- Self-documenting code preferred
- Complex logic may warrant brief inline comments

**Formatting:**
- 2-space indentation for JavaScript/Vue
- Standard Rust formatting (run `cargo fmt` before committing)
- No trailing whitespace
- Use semicolons in JavaScript (consistent with codebase)

**TypeScript/JavaScript:**
- Use TypeScript types when adding new code
- Prefer `const` over `let`
- Use arrow functions for callbacks
- Handle async/await with try-catch

**Security:**
- Never log or expose private keys
- Sensitive operations only in memory, never persist
- Validate all user inputs from frontend

**Database (SQLx):**
- Use migrations in `data/init.sql`
- Models in `database/models.rs`
- Services handle business logic

**State Management (Pinia):**
- Store files in `src/stores/`
- Use composition API stores (`defineStore(id, () => {...})`)
- Access stores via composable functions

**UI Components:**
- Primary: PrimeVue for core components
- Secondary: Arco Design for tabs, modals, tooltips
- Custom components in `src/components/`
- Use `<script setup>` with `name` attribute

**File Organization:**
- Feature-based structure: `src/features/{ecosystem}/{feature}/`
- Shared components: `src/components/`
- Ecosystem-specific components colocated with features

**Git Commits:**
- Use conventional commits format
- Keep changes focused and minimal
- Run linters before committing
