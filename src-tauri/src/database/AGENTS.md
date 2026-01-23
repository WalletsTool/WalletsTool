# Database Module

**Parent:** [AGENTS.md](../../../../AGENTS.md)

## OVERVIEW

SQLite operations with SQLx ORM. Service pattern for chains, RPCs, tokens. Hot-reload capability.

## STRUCTURE

```
database/
├── mod.rs           # Init, reload, export, schema check
├── models.rs        # Data models (Chain, Token, RpcProvider)
├── chain_service.rs # Chain CRUD operations
└── rpc_service.rs   # RPC CRUD operations
```

## WHERE TO LOOK

| Task | File | Notes |
|------|------|-------|
| Init/Reload | `mod.rs` | `init_database()`, `reload_database()` |
| Schema export | `mod.rs` | `export_database_to_init_sql()` |
| Chain CRUD | `chain_service.rs` | `ChainService` struct |
| RPC CRUD | `rpc_service.rs` | `RpcService` struct |
| Models | `models.rs` | SQLx query macros |

## COMMANDS

| Command | Description |
|---------|-------------|
| `reload_database` | Drop all tables, re-init from `data/init.sql` |
| `check_database_schema` | Verify tables, check migration needs |
| `export_database_to_init_sql` | Dump DB to `data/init.sql` |

## CONFIG

Loaded from `package.json`:
```json
{
  "config": {
    "database": {
      "forceInit": false,
      "enableDebugLog": false,
      "initSqlPath": "data/init.sql"
    }
  }
}
```

## SERVICE PATTERN

**ChainService:**
```rust
pub struct ChainService<'a> {
    pool: &'a SqlitePool,
}

impl<'a> ChainService<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self;
    pub async fn get_all_chains(&self) -> Result<Vec<ChainInfo>>;
    pub async fn get_chain_by_key(&self, chain_key: &str) -> Result<Option<Chain>>;
    pub async fn add_chain(&self, request: CreateChainRequest) -> Result<i64>;
    // ... more methods
}
```

## TABLES

| Table | Purpose |
|-------|---------|
| `chains` | Blockchain configs (name, chain_id, symbol, decimals, URLs) |
| `rpc_providers` | RPC endpoints with priority, health status |
| `tokens` | Token configs (contract_address, ABI, decimals) |
| `monitor_configs` | Future: address monitoring |
| `monitor_history` | Future: monitoring history |

## ANTI-PATTERNS

- Never expose raw SQL errors to frontend
- Never skip `PRAGMA foreign_keys = OFF` before `DROP TABLE`
- Never forget to re-enable foreign keys after schema changes
- Never log connection strings or credentials
