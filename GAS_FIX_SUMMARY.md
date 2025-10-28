# Gas费用计算问题修复说明

## 问题描述

在BSC链转USDT时，出现余额不足的错误提示：
- **BNB余额**: 0.000985 (985,488,132,723,422 wei)
- **预估Gas费用**: 0.00391 (3,907,296,800,000,000 wei)
- **Gas Price**: 0.05 gwei (正常)
- **Gas Limit**: 78,145,936 (异常！正常应该是 60,000 左右)

## 根本原因

### 问题1：BSC链误用EIP-1559 baseFee机制
BSC虽然不是标准的EIP-1559链，但RPC可能返回`base_fee_per_gas`值。原代码会将这个baseFee强制应用到Gas Price计算中，导致Gas Price被放大。

**修复**: 明确标记BSC(56)和BSC Testnet(97)为非EIP-1559链，这些链不使用baseFee机制。

### 问题2：Gas Limit使用区块gas limit的百分比计算
当`estimate_gas`失败时，代码尝试使用`block_gas_limit * 5%`作为备用值。BSC的区块gas limit是140,000,000，5%就是7,000,000，远超正常的60,000！

**修复**: 移除基于区块gas limit的百分比计算，直接使用固定的合理默认值。

### 问题3：平均Gas Limit计算错误
`get_average_gas_limit_from_recent_blocks`函数使用`tx.gas`（交易的gas limit）而不是实际使用的gas，导致平均值被严重高估。

**修复**: 禁用这个不可靠的函数，直接使用固定默认值。

## 修复内容

### 1. 优化Gas Price计算 (transfer.rs)

**位置**: `get_gas_price`函数

**修改**:
- 识别真正的EIP-1559链（以太坊、Polygon、Arbitrum等）
- BSC链明确标记为非EIP-1559链
- 只对EIP-1559链应用baseFee逻辑

```rust
// 判断是否为真正的EIP-1559链
let is_eip1559_chain = match chain_id_u64 {
    1 => true,      // Ethereum Mainnet
    137 => true,    // Polygon
    42161 => true,  // Arbitrum One
    56 => false,    // BSC - 非EIP-1559链
    97 => false,    // BSC Testnet - 非EIP-1559链
    _ => base_fee > U256::zero(),
};

// 只对EIP-1559链应用baseFee检查
if is_eip1559_chain && base_fee > U256::zero() {
    // baseFee逻辑...
} else {
    // 非EIP-1559网络直接使用计算的Gas Price
}
```

### 2. 修复Gas Limit计算 (transfer.rs)

**位置**: `get_gas_limit_with_token_type`函数

**修改**:
- 移除基于区块gas limit百分比的计算
- 使用固定的合理默认值
- 禁用不可靠的平均值计算

```rust
// 修改前（错误）：
std::cmp::max(
    block_gas_limit * U256::from(5) / U256::from(100),  // BSC: 140M * 5% = 7M
    min_token_gas
)

// 修改后（正确）：
let default_token_gas = match chain_id {
    42161 => U256::from(150_000),  // Arbitrum One
    1 => U256::from(65_000),       // Ethereum Mainnet
    137 => U256::from(65_000),     // Polygon
    56 => U256::from(60_000),      // BSC
    _ => U256::from(80_000),
};
```

### 3. 优化错误提示 (token_transfer.rs)

**位置**: 平台币余额检查

**修改**:
- 在错误信息中显示可读的单位（BNB/ETH）
- 显示详细的Gas Price (gwei) 和 Gas Limit
- 方便用户和开发者调试

```rust
return Err(format!(
    "平台币余额不足支付Gas费用！\n当前余额: {} ({} wei)\n预估Gas费用: {} ({} wei)\nGas Price: {} gwei, Gas Limit: {}",
    balance_formatted, wallet_balance,
    gas_fee_formatted, estimated_gas_fee,
    gas_price_gwei, gas_limit
).into());
```

## 修复效果

### 修复前
- Gas Limit: 78,145,936 (异常)
- Gas费用: 78,145,936 × 0.05 gwei ≈ 0.00391 BNB
- 结果: 余额不足 (0.000985 < 0.00391)

### 修复后
- Gas Limit: 60,000 (正常)
- Gas费用: 60,000 × 0.05 gwei = 0.000003 BNB
- 结果: 余额充足 (0.000985 > 0.000003)

## 各链Gas Limit默认值

| 链 | 链ID | ETH转账 | 代币转账 |
|---|---|---|---|
| Ethereum Mainnet | 1 | 21,000 | 65,000 |
| BSC | 56 | 21,000 | 60,000 |
| BSC Testnet | 97 | 21,000 | 60,000 |
| Polygon | 137 | 21,000 | 65,000 |
| Arbitrum One | 42161 | 21,000 | 150,000 |
| 其他链 | - | 21,000 | 80,000 |

## 测试建议

1. 测试BSC链USDT转账，确认Gas费用正常
2. 测试其他链（ETH、Polygon等），确保没有回归问题
3. 检查日志中的Gas Price和Gas Limit是否合理
4. 验证错误提示信息是否清晰

## 相关文件

- `src-tauri/src/wallets_tool/ecosystems/ethereum/transfer.rs`
- `src-tauri/src/wallets_tool/ecosystems/ethereum/token_transfer.rs`
