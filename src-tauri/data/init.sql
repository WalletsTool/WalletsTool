-- Wallet Manager 数据库初始化脚本
-- 此文件由系统自动生成，包含当前数据库的所有数据

-- 创建chains表
CREATE TABLE chains (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chain_key TEXT NOT NULL UNIQUE,
    chain_name TEXT NOT NULL,
    chain_id INTEGER NOT NULL,
    native_currency_symbol TEXT NOT NULL,
    native_currency_name TEXT NOT NULL,
    native_currency_decimals INTEGER NOT NULL DEFAULT 18,
    pic_data TEXT,
    scan_url TEXT,
    scan_api TEXT,
    verify_api TEXT,
    check_verify_api TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建rpc_providers表
CREATE TABLE rpc_providers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chain_id INTEGER NOT NULL,
    rpc_url TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    priority INTEGER NOT NULL DEFAULT 100,
    last_success_at DATETIME,
    failure_count INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chain_id) REFERENCES chains(id) ON DELETE CASCADE
);

-- 创建tokens表
CREATE TABLE tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chain_id INTEGER NOT NULL,
    token_key TEXT NOT NULL,
    token_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    contract_address TEXT,
    decimals INTEGER NOT NULL DEFAULT 18,
    token_type TEXT NOT NULL CHECK(token_type IN ('base', 'token')),
    contract_type TEXT,
    abi TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chain_id) REFERENCES chains(id) ON DELETE CASCADE,
    UNIQUE(chain_id, token_key)
);

-- 创建索引
CREATE INDEX idx_rpc_providers_chain_id ON rpc_providers(chain_id);
CREATE INDEX idx_rpc_providers_priority ON rpc_providers(priority);
CREATE INDEX idx_tokens_chain_id ON tokens(chain_id);
CREATE INDEX idx_tokens_symbol ON tokens(symbol);
