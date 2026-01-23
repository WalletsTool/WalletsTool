import { createRouter, createWebHashHistory } from "vue-router";
const routes = [
    // 主窗口页面
    { path: "/", name: "main", component: () => import('@/features/home/pages/Home.vue') },
    // 其他功能页面
    { path: "/transfer", redirect: "/eth/transfer" },
    { path: "/balance", redirect: "/eth/balance" },
    { path: "/monitor", redirect: "/eth/monitor" },

    // Ethereum
    { path: "/eth/transfer", name: "eth-transfer", component: () => import('@/features/ethereum/transfer/pages/Transfer.vue') },
    { path: "/eth/balance", name: "eth-balance", component: () => import('@/features/ethereum/balance/pages/Balance.vue') },

    // Solana
    { path: "/sol/transfer", name: "sol-transfer", component: () => import('@/features/solana/transfer/pages/Transfer.vue') },
    { path: "/sol/balance", name: "sol-balance", component: () => import('@/features/solana/balance/pages/Balance.vue') },

    // EVM Monitor
    { path: "/eth/monitor", name: "eth-monitor", component: () => import('@/features/ethereum/monitor/pages/Monitor.vue') },
];

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

router.onError((error) => {
    console.error('路由错误:', error)
})

export {router}
