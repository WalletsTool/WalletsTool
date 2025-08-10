import { createRouter, createWebHashHistory } from "vue-router";
const routes = [
    // default redirects (keep backward compatibility for tray/open_function_window)
    { path: "/", redirect: "/eth/home" },
    { path: "/home", redirect: "/eth/home" },
    { path: "/transfer", redirect: "/eth/transfer" },
    { path: "/balance", redirect: "/eth/balance" },

    // Home shared
    { path: "/eth/home", name: "home-eth", component: () => import('@/features/home/pages/Home.vue') },
    { path: "/sol/home", name: "home-sol", component: () => import('@/features/home/pages/Home.vue') },

    // Ethereum
    { path: "/eth/transfer", name: "eth-transfer", component: () => import('@/features/ethereum/transfer/pages/Transfer.vue') },
    { path: "/eth/balance", name: "eth-balance", component: () => import('@/features/ethereum/balance/pages/Balance.vue') },

    // Solana
    { path: "/sol/transfer", name: "sol-transfer", component: () => import('@/features/solana/transfer/pages/Transfer.vue') },
    { path: "/sol/balance", name: "sol-balance", component: () => import('@/features/solana/balance/pages/Balance.vue') },
];

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

router.onError((error) => {
    console.error('路由错误:', error)
})

export {router}
