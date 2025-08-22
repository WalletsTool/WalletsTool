# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WalletsTool is a Web3 multi-chain wallet management desktop application built with Vue 3 + Tauri (Rust). It supports Ethereum and Solana ecosystems, providing features like batch wallet import, balance queries, batch transfers, RPC/token configuration, and Excel import/export with local SQLite storage.

## Development Commands

### Frontend (Vue 3 + Vite)
- `yarn dev` - Start Vite development server only
- `yarn build` - Build frontend for production
- `yarn preview` - Preview production build

### Tauri Desktop App
- `yarn tauri-dev` - Start Tauri development server (combines frontend + backend)
- `yarn tauri-build` - Build production desktop application
- `yarn icon` - Generate app icons from source

### Utilities
- `yarn version:update <version>` - Update version across package.json and Cargo.toml, create git tag
- `node scripts/update-version.js <version>` - Direct version update script

## Architecture Overview

### Frontend Structure (Vue 3)
- **Feature-based organization** under `src/features/`
  - `ethereum/` - Ethereum-specific functionality (balance, transfer)
  - `solana/` - Solana-specific functionality (balance, transfer)
  - `home/` - Main dashboard
- **Shared components** in `src/components/`
- **Vue Router** with hash-based routing (`createWebHashHistory`)
- **Pinia** for state management in `src/stores/`

### Backend Structure (Tauri/Rust)
- **Modular architecture** in `src-tauri/src/`
  - `wallets_tool/` - Core business logic
    - `ecosystems/` - Blockchain-specific implementations
      - `ethereum/` - Ethereum chain handling, RPC management, transfers
      - `solana/` - Solana implementation (in progress)
  - `database/` - SQLite database management and models
  - `plugins/` - Tauri plugins and custom filesystem operations
- **Multi-window support** with tray integration
- **SQLite database** with schema management and hot reload capabilities

### Key Integrations
- **ethers.js** (v5.7.0) for Ethereum blockchain interactions
- **SQLite** for local data persistence (chains, tokens, RPC providers)
- **Tauri commands** bridge frontend-backend communication
- **System tray** with context menu for quick function access

## Database Architecture

The application uses SQLite with these main tables:
- `chains` - Blockchain network configurations
- `rpc_providers` - RPC endpoint management
- `tokens` - Token contract configurations
- `monitor_configs` - Address monitoring settings (future feature)
- `monitor_history` - Monitoring history (future feature)

**Security Note**: Private keys are never stored in the database - they exist only in memory during operations.

## Development Patterns

### Adding New Blockchain Support
1. Create new ecosystem directory under `src-tauri/src/wallets_tool/ecosystems/`
2. Implement chain-specific provider, transfer, and balance query logic
3. Add frontend feature directory under `src/features/`
4. Update routing in `src/router/index.js`
5. Register Tauri commands in `main.rs`

### Tauri Command Pattern
Backend functions exposed to frontend via `#[tauri::command]` attribute:
```rust
#[tauri::command]
async fn function_name(param: Type) -> Result<ReturnType, String> {
    // Implementation
}
```

Register in `main.rs` invoke_handler and call from frontend via Tauri's `invoke()`.

### Window Management
- Main window: "WalletsTool" 
- Dynamic child windows for different functions (transfer, balance, etc.)
- Tray integration allows opening function windows independently
- Custom close handling prevents accidental app termination

## Technology Stack

### Frontend Dependencies
- Vue 3.5.17 with Composition API
- PrimeVue 4.3.7 + Arco Design for UI components
- ethers.js 5.7.0 for Ethereum interactions
- xlsx for Excel file handling
- party-js for animations

### Backend Dependencies
- Tauri 2.1 with custom protocol support
- SQLx 0.7 for database operations
- tokio for async runtime
- reqwest for HTTP client functionality
- ethers 1.0 (Rust) for blockchain operations

## Build Configuration

### Vite Configuration
- **Base path**: Relative paths for Tauri compatibility
- **Node.js polyfills**: Configured for ethers.js browser compatibility
- **Code splitting**: Optimized chunk strategy for vendor, pages, components
- **Terser optimization**: Production builds remove console logs and optimize bundle size

### Cargo Release Profile
- Maximum optimization level (opt-level = 3)
- Fat LTO enabled for best performance
- Debug symbols stripped for smaller binaries
- Panic strategy set to abort

## Security Considerations

- Private keys handled only in memory, never persisted
- Local SQLite storage for configuration data only
- Transaction validation and confirmation mechanisms
- No sensitive data in version control or logs