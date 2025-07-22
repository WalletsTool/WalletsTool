-- Web3 Tools 数据库初始化脚本
-- 创建链配置表
CREATE TABLE IF NOT EXISTS chains (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chain_key TEXT NOT NULL UNIQUE,
    chain_name TEXT NOT NULL,
    chain_id INTEGER NOT NULL,
    native_currency_symbol TEXT NOT NULL,
    native_currency_name TEXT NOT NULL,
    native_currency_decimals INTEGER NOT NULL DEFAULT 18,
    pic_url TEXT,
    scan_url TEXT,
    scan_api TEXT,
    verify_api TEXT,
    check_verify_api TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建RPC提供商表
CREATE TABLE IF NOT EXISTS rpc_providers (
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

-- 创建RPC提供商索引
CREATE INDEX IF NOT EXISTS idx_rpc_providers_chain_id ON rpc_providers(chain_id);
CREATE INDEX IF NOT EXISTS idx_rpc_providers_priority ON rpc_providers(priority);

-- 创建代币表
CREATE TABLE IF NOT EXISTS tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chain_id INTEGER NOT NULL,
    token_key TEXT NOT NULL,
    token_name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    contract_address TEXT,
    decimals INTEGER NOT NULL DEFAULT 18,
    token_type TEXT NOT NULL CHECK(token_type IN ('base', 'token')),
    abi TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chain_id) REFERENCES chains(id) ON DELETE CASCADE,
    UNIQUE(chain_id, token_key)
);

-- 创建代币表索引
CREATE INDEX IF NOT EXISTS idx_tokens_chain_id ON tokens(chain_id);
CREATE INDEX IF NOT EXISTS idx_tokens_symbol ON tokens(symbol);

-- 创建余额历史表
CREATE TABLE IF NOT EXISTS balance_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    address TEXT NOT NULL,
    chain_id INTEGER NOT NULL,
    token_id INTEGER,
    balance TEXT NOT NULL,
    nonce INTEGER,
    query_status TEXT NOT NULL CHECK(query_status IN ('success', 'failed', 'timeout')),
    error_message TEXT,
    rpc_url TEXT NOT NULL,
    response_time_ms INTEGER,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chain_id) REFERENCES chains(id) ON DELETE CASCADE,
    FOREIGN KEY (token_id) REFERENCES tokens(id) ON DELETE CASCADE
);

-- 创建余额历史表索引
CREATE INDEX IF NOT EXISTS idx_balance_history_address ON balance_history(address);
CREATE INDEX IF NOT EXISTS idx_balance_history_chain_id ON balance_history(chain_id);
CREATE INDEX IF NOT EXISTS idx_balance_history_created_at ON balance_history(created_at);

-- 插入默认链配置（仅在表为空时插入）
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api)
VALUES
    ('eth', 'Ethereum', 1, 'ETH', 'Ethereum', 18, 'https://assets.coingecko.com/coins/images/279/small/ethereum.png', 'https://etherscan.io', 'https://api.etherscan.io/api', 'https://api.etherscan.io/api', 'https://api.etherscan.io/api'),
    ('bsc', 'BNB Smart Chain', 56, 'BNB', 'BNB', 18, 'https://assets.coingecko.com/coins/images/825/small/bnb-icon2_2x.png', 'https://bscscan.com', 'https://api.bscscan.com/api', 'https://api.bscscan.com/api', 'https://api.bscscan.com/api'),
    ('avax', 'Avalanche', 43114, 'AVAX', 'Avalanche', 18, 'https://assets.coingecko.com/coins/images/12559/small/Avalanche_Circle_RedWhite_Trans.png', 'https://snowscan.xyz', 'https://api.routescan.io/v2/network/mainnet/evm/43114/etherscan/api', 'https://api.routescan.io/v2/network/mainnet/evm/43114/etherscan/api', 'https://api.routescan.io/v2/network/mainnet/evm/43114/etherscan/api'),
    ('zgs', '0G Network', 16600, '0G', '0G Network', 18, 'https://0g.ai/favicon.ico', 'https://chainscan.0g.ai', 'https://chainscan.0g.ai/api', 'https://chainscan.0g.ai/api', 'https://chainscan.0g.ai/api'),
    ('bevm', 'BEVM', 11501, 'BTC', 'Bitcoin', 8, 'https://assets.coingecko.com/coins/images/1/small/bitcoin.png', 'https://scan-mainnet.bevm.io', 'https://scan-mainnet.bevm.io/api', 'https://scan-mainnet.bevm.io/api', 'https://scan-mainnet.bevm.io/api'),
    ('sahara', 'Sahara AI', 0, 'SAHARA', 'Sahara', 18, 'https://www.saharalabs.ai/favicon.ico', 'https://explorer.saharalabs.ai', 'https://explorer.saharalabs.ai/api', 'https://explorer.saharalabs.ai/api', 'https://explorer.saharalabs.ai/api'),
    ('story', 'Story Protocol', 0, 'IP', 'Story', 18, 'https://story.foundation/favicon.ico', 'https://testnet.storyscan.xyz', 'https://testnet.storyscan.xyz/api', 'https://testnet.storyscan.xyz/api', 'https://testnet.storyscan.xyz/api'),
    ('manta', 'Manta Pacific', 169, 'ETH', 'Ethereum', 18, 'https://assets.coingecko.com/coins/images/279/small/ethereum.png', 'https://pacific-explorer.manta.network', 'https://pacific-explorer.manta.network/api', 'https://pacific-explorer.manta.network/api', 'https://pacific-explorer.manta.network/api'),
    ('linea', 'Linea', 59144, 'ETH', 'Ethereum', 18, 'https://assets.coingecko.com/coins/images/279/small/ethereum.png', 'https://lineascan.build', 'https://api.lineascan.build/api', 'https://api.lineascan.build/api', 'https://api.lineascan.build/api'),
    ('base', 'Base', 8453, 'ETH', 'Ethereum', 18, 'https://assets.coingecko.com/coins/images/279/small/ethereum.png', 'https://basescan.org', 'https://api.basescan.org/api', 'https://api.basescan.org/api', 'https://api.basescan.org/api'),
    ('sol', 'Solana', 0, 'SOL', 'Solana', 9, 'https://assets.coingecko.com/coins/images/4128/small/solana.png', 'https://solscan.io', 'https://api.solscan.io', 'https://api.solscan.io', 'https://api.solscan.io'),
    ('opbnb', 'opBNB', 204, 'BNB', 'BNB', 18, 'https://assets.coingecko.com/coins/images/825/small/bnb-icon2_2x.png', 'https://opbnbscan.com', 'https://api-opbnb.bscscan.com/api', 'https://api-opbnb.bscscan.com/api', 'https://api-opbnb.bscscan.com/api'),
    ('geth', 'Goerli Testnet', 5, 'ETH', 'Ethereum', 18, 'https://assets.coingecko.com/coins/images/279/small/ethereum.png', 'https://goerli.etherscan.io', 'https://api-goerli.etherscan.io/api', 'https://api-goerli.etherscan.io/api', 'https://api-goerli.etherscan.io/api'),
    ('sepolia', 'Sepolia Testnet', 11155111, 'ETH', 'Ethereum', 18, 'https://assets.coingecko.com/coins/images/279/small/ethereum.png', 'https://sepolia.etherscan.io', 'https://api-sepolia.etherscan.io/api', 'https://api-sepolia.etherscan.io/api', 'https://api-sepolia.etherscan.io/api');

-- 插入默认RPC提供商（仅在表为空时插入）
-- Ethereum RPC
INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://rpc.ankr.com/eth/7b0305a9ff9721e1f27753ef99e285fdecf8b8b90c11cda831e7d54718c70a9f', 0 
FROM chains WHERE chain_key = 'eth';

INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://eth-mainnet.nodereal.io/v1/0f6a7df001924b749c9466dc0bdb99c5', 1 
FROM chains WHERE chain_key = 'eth';

INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://1rpc.io/eth', 2 
FROM chains WHERE chain_key = 'eth';

-- BSC RPC
INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://bsc-dataseed1.bnbchain.org', 0 
FROM chains WHERE chain_key = 'bsc';

INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://bsc.publicnode.com', 1 
FROM chains WHERE chain_key = 'bsc';

INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://bsc.drpc.org', 2 
FROM chains WHERE chain_key = 'bsc';

-- Avalanche RPC
INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://ava-mainnet.public.blastapi.io/ext/bc/C/rpc', 0 
FROM chains WHERE chain_key = 'avax';

INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://api.avax.network/ext/bc/C/rpc', 1 
FROM chains WHERE chain_key = 'avax';

INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://avalanche.drpc.org', 2 
FROM chains WHERE chain_key = 'avax';

-- 插入默认基础代币（仅在表为空时插入）
INSERT OR IGNORE INTO tokens (chain_id, token_key, token_name, symbol, decimals, token_type, abi)
SELECT id, chain_key || '_base', native_currency_name, native_currency_symbol, native_currency_decimals, 'base', NULL
FROM chains;
