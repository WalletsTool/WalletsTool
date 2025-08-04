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

// 添加路由守卫进行调试
router.beforeEach((to, from, next) => {
    console.log('[DEBUG] 路由跳转 - 从:', from.path, '到:', to.path)
    next()
})

router.afterEach((to, from) => {
    console.log('[DEBUG] 路由跳转完成 - 当前路由:', to.path)
})

router.onError((error) => {
    console.error('[DEBUG] 路由错误:', error)
})

export {router}
