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
    contract_type TEXT,
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

-- 插入链配置数据
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('eth', 'Ethereum', 1, 'ETH', 'Ethereum', 18, 'eth.png', 'https://cn.etherscan.com/', 'https://api.etherscan.io/api?module=contract&action=getabi&apikey=5QKYWV24Q7E4NHXV928Q62BDIPHTJJSYH3&address=', 'https://api.etherscan.io/api?module=contract&action=verifyproxycontract&apikey=5QKYWV24Q7E4NHXV928Q62BDIPHTJJSYH3', 'https://api.etherscan.io/api?module=contract&action=checkproxyverification&apikey=5QKYWV24Q7E4NHXV928Q62BDIPHTJJSYH3&guid=');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('avax', 'Avalanche C-Chain', 43114, 'AVAX', 'Avalanche', 18, 'avax.png', 'https://odyssey.storyscan.xyz/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('sahara_test', 'SAHARA_TEST', 0, 'SAHARA', 'Sahara', 18, 'sahara.png', 'https://testnet-explorer.saharalabs.ai/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('story_test', 'Story_TEST', 0, 'IP', 'Story', 18, 'story.png', 'https://odyssey.storyscan.xyz/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('linea', 'Linea', 59144, 'ETH', 'Ethereum', 18, 'linea.png', 'https://lineascan.build/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('base', 'Base', 8453, 'ETH', 'Ethereum', 18, 'base.png', 'https://basescan.org/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('opbnb', 'opBNB', 204, 'BNB', 'BNB', 18, 'opbnb.png', 'https://opbnbscan.com', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('op', 'Optimism', 10, 'ETH', 'Ethereum', 18, 'optimism.png', 'https://www.optimism.io/', 'https://api-optimistic.etherscan.io/api?module=contract&action=getabi&apikey=E3TST2HW8GNGBKDTTB6WXFWMIJGBTIZ2EM&address=', 'https://api-optimistic.etherscan.io/api?module=contract&action=verifyproxycontract&apikey=E3TST2HW8GNGBKDTTB6WXFWMIJGBTIZ2EM', 'https://api-optimistic.etherscan.io/api?module=contract&action=checkproxyverification&apikey=E3TST2HW8GNGBKDTTB6WXFWMIJGBTIZ2EM&guid=');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('polygon', 'Polygon', 137, 'MATIC', 'Polygon', 18, 'polygon.png', 'https://polygonscan.com/', 'https://api.polygonscan.com/api?module=contract&action=getabi&apikey=P5FRCI1WTR6ZZP4HTZQPPIUN8E9QY7BM99&address=', 'https://api.polygonscan.com/api?module=contract&action=verifyproxycontract&apikey=P5FRCI1WTR6ZZP4HTZQPPIUN8E9QY7BM99', 'https://api.polygonscan.com/api?module=contract&action=checkproxyverification&apikey=P5FRCI1WTR6ZZP4HTZQPPIUN8E9QY7BM99&guid=');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('arb', 'Arbitrum', 42161, 'ETH', 'Ethereum', 18, 'arb.png', 'https://arbiscan.io/', 'https://api.arbiscan.io/api?module=contract&action=getabi&apikey=F8NKW7EDV4DBA1CP1YHT3G8UKKTNYTC3F8&address=', 'https://api.arbiscan.io/api?module=contract&action=verifyproxycontract&apikey=F8NKW7EDV4DBA1CP1YHT3G8UKKTNYTC3F8', 'https://api.arbiscan.io/api?module=contract&action=checkproxyverification&apikey=F8NKW7EDV4DBA1CP1YHT3G8UKKTNYTC3F8&guid=');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('binance', 'Binance Smart Chain', 56, 'BNB', 'BNB', 18, 'bnb.png', 'https://bscscan.com/', 'https://api.bscscan.com/api?module=contract&action=getabi&apikey=5BUY5BGWVP8MQJRQQXQW4VQXCS4KDEK7HC&address=', 'https://api.bscscan.com/api?module=contract&action=verifyproxycontract&apikey=5BUY5BGWVP8MQJRQQXQW4VQXCS4KDEK7HC', 'https://api.bscscan.com/api?module=contract&action=checkproxyverification&apikey=5BUY5BGWVP8MQJRQQXQW4VQXCS4KDEK7HC&guid=');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('starknet', 'StarkNet', 0, 'ETH', 'Ethereum', 18, 'starknet.png', 'https://starkscan.co/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('okt', 'OKT Chain', 66, 'OKT', 'OKExChain', 18, 'okt.png', 'https://www.oklink.com/cn/oktc', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('geth', 'Goerli Ethereum TestNet', 5, 'ETH', 'Ethereum', 18, 'eth.png', 'https://cn.etherscan.com/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('sepolia', 'Sepolia Ethereum TestNet', 11155111, 'ETH', 'Ethereum', 18, 'eth.png', 'https://cn.etherscan.com/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('holesky', 'Holesky Ethereum TestNet', 17000, 'ETH', 'Ethereum', 18, 'eth.png', 'https://cn.etherscan.com/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('scroll', 'Scroll', 534352, 'ETH', 'Ethereum', 18, 'scroll.png', 'https://blockscout.scroll.io/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('sol', 'Solana', 0, 'SOL', 'Solana', 9, 'solana.png', 'https://explorer.solana.com/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('bevm', 'Bevm', 11501, 'BTC', 'Bitcoin', 8, 'bevm.png', 'https://scan-mainnet.bevm.io/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('dym', 'Dymension', 1100, 'DYM', 'Dymension', 18, 'dym.png', 'https://portal.dymension.xyz/dymension/metrics', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('evmos', 'EVMOS', 9001, 'EVMOS', 'Evmos', 18, 'evmos.png', 'https://escan.live/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('0g', '0G-TEST', 16600, '0G', '0G Network', 18, '0g.png', 'https://storagescan-newton.0g.ai/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('zksync', 'zkSync', 324, 'ETH', 'Ethereum', 18, 'zksync.png', 'https://explorer.zksync.io/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('manta', 'Manta', 169, 'ETH', 'Ethereum', 18, 'manta.png', 'https://manta.socialscan.io/', '', '', '');
INSERT OR IGNORE INTO chains (chain_key, chain_name, chain_id, native_currency_symbol, native_currency_name, native_currency_decimals, pic_url, scan_url, scan_api, verify_api, check_verify_api) VALUES ('nexus_dev', 'NEXUS_DEV', 0, 'NEXUS', 'Nexus', 18, 'nexus.png', 'https://explorer.nexus.xyz/', '', '', '');

-- 插入默认RPC提供商
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
FROM chains WHERE chain_key = 'binance';

INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://bsc.publicnode.com', 1 
FROM chains WHERE chain_key = 'binance';

INSERT OR IGNORE INTO rpc_providers (chain_id, rpc_url, priority) 
SELECT id, 'https://bsc.drpc.org', 2 
FROM chains WHERE chain_key = 'binance';

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

-- 插入代币配置数据
-- 插入0g链的代币配置
INSERT OR IGNORE INTO tokens (chain_id, token_key, token_name, symbol, contract_address, decimals, token_type, abi) SELECT id, 'a0gi', 'A0GI', 'A0GI', '', 18, 'base', '' FROM chains WHERE chain_key = '0g';

-- 插入arb链的代币配置
INSERT OR IGNORE INTO tokens (chain_id, token_key, token_name, symbol, contract_address, decimals, token_type, abi) SELECT id, 'eth', 'ETH', 'ETH', '', 18, 'base', '' FROM chains WHERE chain_key = 'arb';
INSERT OR IGNORE INTO tokens (chain_id, token_key, token_name, symbol, contract_address, decimals, token_type, abi) SELECT id, 'arb', 'Arbitrum', 'ARB', '0x912CE59144191C1204E64559FE8253a0e49E6548', 18, 'token', '[{"inputs":[],"name":"name","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"symbol","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"decimals","outputs":[{"internalType":"uint8","name":"","type":"uint8"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"totalSupply","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"account","type":"address"}],"name":"balanceOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"transfer","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"owner","type":"address"},{"internalType":"address","name":"spender","type":"address"}],"name":"allowance","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"spender","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"approve","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"from","type":"address"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"transferFrom","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"}]' FROM chains WHERE chain_key = 'arb';
INSERT OR IGNORE INTO tokens (chain_id, token_key, token_name, symbol, contract_address, decimals, token_type, abi) SELECT id, 'usdt', 'Tether USD', 'USDT', '0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9', 6, 'token', '[{"inputs":[],"name":"name","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"symbol","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"decimals","outputs":[{"internalType":"uint8","name":"","type":"uint8"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"totalSupply","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"account","type":"address"}],"name":"balanceOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"transfer","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"owner","type":"address"},{"internalType":"address","name":"spender","type":"address"}],"name":"allowance","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"spender","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"approve","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"from","type":"address"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"name":"transferFrom","outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"}]' FROM chains WHERE chain_key = 'arb';

-- 插入默认基础代币（为其他链创建基础代币）
INSERT OR IGNORE INTO tokens (chain_id, token_key, token_name, symbol, decimals, token_type, abi)
SELECT id, chain_key || '_base', native_currency_name, native_currency_symbol, native_currency_decimals, 'base', NULL
FROM chains WHERE chain_key NOT IN ('0g', 'arb');
