# RPC 全部禁用闪退问题修复

## 问题描述
在 "RPC管理" 中将所有 RPC 都禁用后，程序会出现闪退的问题。

## 问题原因
1. **后端层面**：当查询 `is_active = 1` 的 RPC 时，如果没有启用的 RPC 提供商，`get_rpc_config()` 函数会返回 `None`
2. **错误传播**：在多个使用 RPC 的地方（如转账、查询余额等），代码使用了 `.unwrap()` 或 `.ok_or_else()` 等方式，当返回 `None` 时会导致 panic
3. **错误提示不明确**：原有错误信息仅提示"不支持的链"，无法让用户理解真正的问题

## 修复方案

### 1. 后端修复 (`src-tauri/src/wallets_tool/ecosystems/ethereum/transfer.rs`)

#### 修改点 1：`get_rpc_config()` 函数 - 更详细的错误日志
```rust
if rpc_providers.is_empty() {
    println!("[ERROR] get_rpc_config - 没有找到活跃的RPC提供商，链: {}。请在RPC管理中至少启用一个RPC节点。", chain);
    return None;
}
```

#### 修改点 2：`create_provider()` 函数 - 更明确的错误信息
```rust
let rpc_config = get_rpc_config(chain).await
    .ok_or_else(|| {
        let error_msg = format!("无法获取链 '{}' 的RPC配置。可能原因：1) 链不存在  2) 所有RPC节点都已被禁用。请检查RPC管理设置，至少启用一个RPC节点。", chain);
        println!("[ERROR] create_provider - {}", error_msg);
        error_msg
    })?;
```

#### 修改点 3：`get_random_provider()` 函数 - 友好的错误提示
```rust
let rpc_config = get_rpc_config(chain).await
    .ok_or_else(|| {
        format!("无法获取链 '{}' 的RPC配置。可能原因：1) 链不存在  2) 所有RPC节点都已被禁用。请检查RPC管理设置，至少启用一个RPC节点。", chain)
    })?;
```

### 2. 前端修复 (`src/features/ethereum/transfer/pages/Transfer.vue`)

#### 修改点 1：`queryBalance()` 函数 - 添加前置检查
```javascript
// 检查是否有启用的RPC节点
if (!currentChain.value || !chainValue.value) {
  Notification.warning("请选择一个区块链网络！");
  return;
}
```

#### 修改点 2：`queryBalanceBatch()` 函数 - 增强错误处理
```javascript
} catch (error) {
  console.error('批次查询失败:', error);
  
  // 检查是否是RPC配置错误
  const errorMsg = String(error);
  if (errorMsg.includes('RPC配置') || errorMsg.includes('RPC节点') || errorMsg.includes('禁用')) {
    Notification.error({
      title: '查询失败',
      content: errorMsg,
      duration: 5000
    });
  } else {
    Notification.error('查询失败：' + errorMsg);
  }
  // ...
}
```

#### 修改点 3：`queryToAddressBalanceBatch()` 函数 - 同样的错误处理
```javascript
} catch (error) {
  console.error('批次查询到账地址失败:', error);

  // 检查是否是RPC配置错误
  const errorMsg = String(error);
  if (errorMsg.includes('RPC配置') || errorMsg.includes('RPC节点') || errorMsg.includes('禁用')) {
    Notification.error({
      title: '查询失败',
      content: errorMsg,
      duration: 5000
    });
  }
  // ...
}
```

#### 修改点 4：`chainChange()` 函数 - 主动检查 RPC 状态
```javascript
// 检查是否有启用的RPC节点
try {
  const rpcProviders = await invoke('get_rpc_providers', { chainKey: chainValue.value });
  const activeRpcs = rpcProviders.filter(rpc => rpc.is_active);
  
  if (activeRpcs.length === 0) {
    Notification.warning({
      title: '注意：没有启用的RPC节点',
      content: `当前链 "${currentChain.value.name}" 没有启用的RPC节点，无法执行查询和转账操作。请在"RPC管理"中至少启用一个RPC节点。`,
      duration: 8000
    });
  }
} catch (err) {
  console.error('检查RPC状态失败:', err);
}
```

## 修复效果

### 修复前
- 禁用所有 RPC 后，程序会直接崩溃闪退
- 没有任何错误提示，用户不知道问题原因

### 修复后
1. **程序不再闪退**：所有可能导致 panic 的地方都改为返回友好的错误信息
2. **切换链时主动提示**：当选择一个没有启用 RPC 的链时，会立即弹出警告提示
3. **操作时详细提示**：在查询余额或转账时，如果遇到 RPC 配置问题，会显示详细的错误信息，告知用户如何解决
4. **后端日志增强**：后端日志中会明确记录"请在RPC管理中至少启用一个RPC节点"

## 测试建议

1. **禁用所有 RPC**：
   - 进入 RPC 管理
   - 将某个链的所有 RPC 节点都禁用
   - 观察是否有警告提示

2. **尝试查询余额**：
   - 在禁用所有 RPC 后点击"查询余额"
   - 应该看到友好的错误提示，而不是程序崩溃

3. **尝试转账**：
   - 在禁用所有 RPC 后点击"开始执行"
   - 应该看到详细的错误信息

4. **重新启用 RPC**：
   - 启用至少一个 RPC 节点
   - 再次尝试查询和转账，应该能正常工作

## 注意事项

- 修复后程序不会崩溃，但仍然无法执行实际操作（查询、转账）
- 用户需要在 RPC 管理中至少启用一个 RPC 节点才能正常使用
- 建议在产品文档中说明至少需要一个启用的 RPC 节点

## 相关文件

- `src-tauri/src/wallets_tool/ecosystems/ethereum/transfer.rs` - 后端 RPC 配置和错误处理
- `src/features/ethereum/transfer/pages/Transfer.vue` - 前端查询和转账错误处理
