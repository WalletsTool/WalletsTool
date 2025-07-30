import {createRouter, createWebHashHistory} from "vue-router";
const routes = [
    {path: "/", redirect: "/home"},
    {
        path: "/home",
        name: "home",
        component:() => import('../pages/Home.vue')
    },
    {
        path: "/transfer",
        name: "transfer",
        component:() => import('../pages/Transfer.vue')
    },
    {
        path: "/balance",
        name: "balance",
        component:() => import('../pages/Balance.vue')
    }
]

const router = createRouter({
    history: createWebHashHistory(),
    routes,
})

export {router}
