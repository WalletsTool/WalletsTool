# Token ABI 功能说明

已成功为数据库的tokens表添加abi列，用于存储智能合约的ABI JSON字符串。

## 数据库更改

### 表结构更新
- 在 `tokens` 表中添加了 `abi` 列 (TEXT类型，可为空)
- 更新了相关的 Rust 结构体和模型

### 修改的文件
1. `src/database/migrations.rs` - 添加了数据库迁移逻辑
2. `src/database/models.rs` - 更新了 Token 和 CreateTokenRequest 结构体
3. `src/database/chain_service.rs` - 更新了数据库操作方法
4. `src/web3_tools/chain_config.rs` - 更新了 API 接口
5. `src/main.rs` - 注册了新的 API 命令

## API 接口更新

### 1. 获取代币列表 (`get_coin_list`)
现在返回的每个代币对象包含 `abi` 字段：

```json
{
  "key": "usdt",
  "name": "Tether USD",
  "symbol": "usdt",
  "contract_address": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
  "decimals": 6,
  "coin_type": "token",
  "abi": "[{\"inputs\":[],\"name\":\"name\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]"
}
```

### 2. 添加代币 (`add_coin`)
现在可以在添加代币时包含 ABI：

```json
{
  "key": "new_token",
  "name": "New Token",
  "symbol": "NT",
  "contract_address": "0x...",
  "decimals": 18,
  "coin_type": "token",
  "abi": "[{\"inputs\":[],\"name\":\"totalSupply\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]"
}
```

### 3. 更新代币 ABI (`update_token_abi`) - 新功能
新增的API接口，用于单独更新代币的ABI：

**调用方式：**
```javascript
await invoke('update_token_abi', {
  chain: 'eth',
  tokenKey: 'usdt',
  abi: '[{...}]'  // ABI JSON字符串，可为null清除ABI
});
```

**参数说明：**
- `chain`: 链的标识符 (如 'eth', 'bsc', 'avax' 等)
- `tokenKey`: 代币的key标识符
- `abi`: ABI JSON字符串，传入 null 可清除现有ABI

## 数据库迁移

系统会自动检查并执行数据库迁移：
- 如果 `tokens` 表已存在但没有 `abi` 列，会自动添加该列
- 现有数据不会受影响，新列默认为 NULL
- 迁移是幂等的，多次运行不会出现问题

## 使用场景

1. **智能合约交互**：存储代币合约的完整ABI，便于构建交易和查询
2. **动态调用**：根据存储的ABI动态调用合约方法
3. **UI展示**：根据ABI自动生成智能合约交互界面
4. **合约验证**：验证合约地址与ABI的匹配性

## 注意事项

1. ABI存储为JSON字符串格式，使用时需要解析
2. ABI字段是可选的，基础代币(type='base')通常不需要ABI
3. 存储的ABI应该是有效的JSON格式
4. 建议在存储前验证ABI的格式正确性

## 示例ABI

标准ERC-20代币的部分ABI示例：

```json
[
  {
    "inputs": [],
    "name": "name",
    "outputs": [{"internalType": "string", "name": "", "type": "string"}],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "symbol", 
    "outputs": [{"internalType": "string", "name": "", "type": "string"}],
    "stateMutability": "view",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "totalSupply",
    "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
    "stateMutability": "view", 
    "type": "function"
  },
  {
    "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
    "name": "balanceOf",
    "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
    "stateMutability": "view",
    "type": "function"
  }
]
```

## 技术实现

- 使用 SQLx 进行数据库操作，支持类型安全的查询
- 采用数据库迁移策略，确保升级的平滑性
- 所有文件编码都设置为UTF-8，保证中文兼容
- 支持网络代理配置(http://127.0.0.1:8595)用于中国大陆环境

该功能已通过编译测试，可以正常使用。
