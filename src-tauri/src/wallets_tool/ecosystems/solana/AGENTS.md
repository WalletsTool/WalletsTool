# Solana Backend Ecosystem

**Parent:** [AGENTS.md](../../../../AGENTS.md)

## OVERVIEW

Solana implementation using `solana-sdk`. Handles native SOL and SPL Token transfers, RPC management, and high-concurrency batch operations.

## STRUCTURE

```
ecosystems/solana/
├── mod.rs          # Module declarations
├── provider.rs     # RpcClient initialization & commitment config
├── transfer.rs     # Native SOL + SPL Token transfers
└── chain_config.rs # Chain metadata
```

## WHERE TO LOOK

| Task | File | Notes |
|------|------|-------|
| Transfer Logic | `transfer.rs` | Handles both SOL and SPL tokens in one file |
| Connection | `provider.rs` | Uses `RpcClient` with `confirmed` commitment |
| Signatures | `transfer.rs` | Uses `Keypair` from memory (Zeroize) |

## COMMANDS EXPOSED

| Command | File | Description |
|---------|------|-------------|
| `sol_transfer` | `transfer.rs` | Native SOL transfer |
| `sol_token_transfer` | `transfer.rs` | SPL Token transfer (auto-ATA creation) |
| `sol_transfer_fast` | `transfer.rs` | Fury mode (high concurrency) |
| `sol_token_transfer_fast`| `transfer.rs` | Fury mode for tokens |

## ANTI-PATTERNS (SOLANA SPECIFIC)

- **Rent Exemption**: NEVER skip checks for rent-exempt minimums on new accounts.
- **Blockhash**: NEVER reuse old blockhashes; always fetch fresh before signing.
- **ATA**: ALWAYS check/create Associated Token Accounts (ATA) for token transfers.
- **Secrets**: NEVER clone `Keypair` or secret bytes; use references inside `SecureMemory` closure.
