# Ethereum Transfer Feature

**Parent:** [AGENTS.md](../../AGENTS.md)

## OVERVIEW

Batch transfer composables for Ethereum (native + ERC-20). Largest frontend feature (848 lines in `useTransfer.ts`).

## STRUCTURE

```
./
├── pages/Transfer.vue    # Route target (~116KB)
├── components/
│   ├── TransferTable.vue
│   ├── TransferConfigForm.vue
│   ├── TransferStatusBar.vue
│   └── index.ts          # Barrel exports
├── composables/
│   ├── useTransfer.ts    # Core logic (848 lines)
│   ├── useBalanceQuery.ts
│   ├── useValidation.ts
│   ├── useDataOperations.ts
│   ├── useTip.ts
│   └── index.ts
└── styles/index.ts       # Barrel export
```

## WHERE TO LOOK

| Task | File | Notes |
|------|------|-------|
| Transfer loop | `composables/useTransfer.ts` | `iterTransfer()` function |
| Multi-thread | `iterTransfer()` lines 431-609 | Thread pool with `Promise.all` |
| Fury mode | `iterTransferFuryMode()` lines 612-828 | >90 threads, fast submit + batch confirm |
| Validation | `useValidation.ts` | Form rules |
| Data I/O | `useDataOperations.ts` | Excel import/export |
| UI | `TransferTable.vue` | Virtual scrolling for large lists |

## CONVENTIONS

- **Import alias:** Use `@/` for imports from `src/`
- **UI library:** PrimeVue for core components
- **State:** Props passed from parent, not Pinia
- **Notifications:** `@arco-design/web-vue` Notification component
- **Tauri invoke:** All backend calls via `@tauri-apps/api/core`

## COMPOSABLES

**useTransfer.ts (main):**
```typescript
export function useTransfer(options = {}) {
  // Options: data, form, chainValue, currentChain, currentCoin,
  //          threadCount, enableMultiThread, transferConfig,
  //          transferProgress, validateForm, executeTransfer, ...

  // Returns: transferFnc, stopTransfer, performIntelligentRetry,
  //          iterTransfer, iterTransferFuryMode
}
```

**Key functions:**
- `transferFnc(inputData)` - Main entry, handles async flow
- `iterTransfer(accountData)` - Single/multi-thread mode
- `iterTransferFuryMode(accountData)` - High-concurrency batch
- `performIntelligentRetry(failedData)` - On-chain tx detection

## BACKEND COMMANDS

```javascript
invoke('base_coin_transfer', { index, item, config })
invoke('token_transfer', { index, item, config })
invoke('base_coin_transfer_fast', { ... }) // Fury mode
invoke('token_transfer_fast', { ... })
invoke('check_transaction_status', { chain, txHash })
invoke('check_wallet_recent_transfers', { ... })
```

## MODES

| Mode | Threshold | Behavior |
|------|-----------|----------|
| Single-thread | `enableMultiThread === '0'` | Sequential execution |
| Multi-thread | 1-90 threads | Wallet-grouped concurrency |
| Fury mode | >90 threads | Fast submit → batch confirm |

## ANTI-PATTERNS

- Never pass `stopFlag` directly to backend
- Never skip `updateTransferProgress()` calls
- Never use `form` properties without default checks
