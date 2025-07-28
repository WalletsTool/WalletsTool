import {createRouter, createWebHashHistory} from "vue-router";
import Home from "@/pages/Home.vue"
import Transfer from "@/pages/Transfer.vue"
import Balance from "@/pages/Balance.vue"
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
    }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

export {router}
