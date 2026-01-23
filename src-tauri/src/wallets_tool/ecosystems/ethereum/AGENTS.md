# Ethereum Ecosystem (Backend)

**Parent:** [AGENTS.md](../../../../AGENTS.md)

## OVERVIEW

Complete Ethereum implementation: RPC management, transfers, balance queries, proxy support. 8 specialized Rust modules.

## STRUCTURE

```
ecosystems/ethereum/
├── mod.rs              # Module declarations (8 pub mods)
├── chain_config.rs     # Chain metadata (chain_id, symbol, decimals)
├── provider.rs         # Provider initialization, RPC selection
├── rpc_management.rs   # Multi-RPC load balancing, health checks
├── simple_balance_query.rs # Native + ERC-20 balance
├── transfer.rs         # Native coin transfer
├── token_transfer.rs   # ERC-20 transfer with ABI
├── proxy_manager.rs    # HTTP/SOCKS5 proxy lifecycle
└── proxy_commands.rs   # Tauri commands for proxy ops
```

## WHERE TO LOOK

| Task | File | Notes |
|------|------|-------|
| Transfer logic | `transfer.rs`, `token_transfer.rs` | Core tx execution |
| RPC failover | `rpc_management.rs` | Load balancing, health tracking |
| Provider setup | `provider.rs` | ethers::Provider creation |
| Balance query | `simple_balance_query.rs` | Native + token balances |
| Proxy | `proxy_manager.rs`, `proxy_commands.rs` | HTTP/SOCKS5 |

## COMMANDS EXPOSED

| Command | File | Description |
|---------|------|-------------|
| `base_coin_transfer` | `transfer.rs` | Native ETH transfer |
| `token_transfer` | `token_transfer.rs` | ERC-20 transfer |
| `base_coin_transfer_fast` | `transfer.rs` | Fury mode submit |
| `token_transfer_fast` | `token_transfer.rs` | Fury mode submit |
| `query_balances_simple` | `simple_balance_query.rs` | Batch balance |
| `add_rpc_provider` | `rpc_management.rs` | RPC CRUD |
| `test_rpc_connection` | `rpc_management.rs` | Health check |

## PATTERNS

**Tauri command signature:**
```rust
#[tauri::command]
async fn function_name(param: Type) -> Result<ReturnType, String> {
    // Implementation
}
```

**Error handling:** Return `Err(format!("..."))` for user-facing errors

**Async:** All commands are `async`, use `tokio` runtime

## PROXY SUPPORT

- HTTP and SOCKS5 protocols
- Per-RPC-provider proxy assignment
- Commands: `add_proxy`, `test_proxy`, `get_proxies`

## ANTI-PATTERNS

- Never block in command handlers
- Never log private keys or sensitive data
- Never return raw ethers errors to frontend (wrap in descriptive string)
