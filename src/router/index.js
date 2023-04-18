import {createRouter, createWebHashHistory} from "vue-router";

const Home = () => import("@/pages/Home.vue")
const Transfer  = () => import("@/pages/Transfer.vue")
const Balance  = () => import("@/pages/Balance.vue")
const Monitor  = () => import("@/pages/Monitor.vue")
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
    }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

export {router}
