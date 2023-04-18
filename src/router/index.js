import {createRouter, createWebHashHistory} from "vue-router";

const Home = () => import("@/pages/Home.vue")
const Transfer = () => import("@/pages/Transfer.vue")
const Balance = () => import("@/pages/Balance.vue")
const Monitor = () => import("@/pages/Monitor.vue")
const Uniswap = () => import("@/pages/Uniswap.vue")
const routes = [
    {path: "/", redirect: "/home"},
    {
        path: "/home",
        name: "home",
        component: Home
    },
    {
        path: "/transfer",
        name: "transfer",
        component: Transfer
    },
    {
        path: "/balance",
        name: "balance",
        component: Balance
    },
    {
        path: "/monitor",
        name: "monitor",
        component: Monitor
    },
    {
        path: "/uniswap",
        name: "uniswap",
        component: Uniswap
    }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

export {router}
