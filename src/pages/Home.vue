<script setup name="home">
import {useRouter} from 'vue-router'
import {Notification} from "@arco-design/web-vue";
import {onMounted} from "vue";
import party from "party-js";
import {confettiStore} from '@/stores'

const router = useRouter()
const store = confettiStore()

onMounted(() => {
    const newFlag = funcList.filter(item => item.isNew).length > 0
    if (newFlag && store.status) {
        // 动画效果
        party.confetti(document.getElementById('app'), {
            count: party.variation.range(100, 150),
            spread: party.variation.range(30, 80),
            size: party.variation.range(0.6, 1.3),
            colors: ['#9dbd4d', '#5a91d9', '#e8c261'],
            origin: {
                x: 0.5,
                y: 0.3
            }
        })
        // 关闭动画
        store.changeStatus(false)
    }
})

// 功能菜单列表
const funcList = [
    {
        title: "钱包多对多转账",
        desc: "目前支持链：ETH（主网、Goerli）、OP、ARB、BSC、Polygon、OKT_Chain（持续更新中）",
        picture: "avatar/transfer.png",
        pageName: "transfer"
    },
    {
        title: "链上余额查询",
        desc: "目前支持链：ETH（主网、Goerli）、OP、ARB、BSC、Polygon、StarkNet、OKT_Chain（持续更新中）",
        picture: "avatar/balance.png",
        pageName: "balance"
    },
    {
        title: "链上地址监控",
        isBuilding: true,
        desc: "目前支持链：ETH、OP、ARB、BSC、Polygon（建设中）",
        picture: "avatar/monitor.png",
        pageName: "monitor"
    },
    {
        title: "Uniswap批量交易",
        isBuilding: true,
        desc: "支持 Uniswap  V3 交易（即将上线，敬请期待）",
        picture: "avatar/uniswap.png",
        pageName: "uniswap"
    }
]

// 跳转到钱包多对多转账
function goPage(pageName) {
    if (pageName === 'monitor' || pageName === 'uniswap') {
        Notification.success('功能建设中，敬请期待')
        return
    }
    router.push({
        name: pageName
    })
}
</script>

<template>
    <div class="container home">
        <div class="funcListTitle">功能列表</div>
        <a-list class="funcList" size="large" :hoverable="true">
            <a-list-item class="funcItem" @click="goPage(item.pageName)" v-for="(item,idx) in funcList" :key="idx">
                <a-list-item-meta>
                    <template #title>
                        <span :class="[item.isBuilding?'grayTitle':'']">{{ item.title }}</span>
                        <a-badge v-if="item.isNew" style="margin-left: 10px" text="NEW"/>
                    </template>
                    <template #description>
                        <span :class="[item.isBuilding?'grayTitle':'']">{{ item.desc }}</span>
                    </template>
                    <template #avatar>
                        <a-avatar shape="square">
                            <img
                                    alt="avatar"
                                    :src="item.picture"
                                    style="background-color: transparent;"
                            />
                        </a-avatar>
                    </template>
                </a-list-item-meta>
            </a-list-item>
        </a-list>
    </div>
</template>

<style scoped>
.container {
    padding: 10px;
}

.funcListTitle {
    font-size: 20px;
    font-weight: 600;
    display: block;
    height: 30px;
    line-height: 30px;
}

.funcList {
    margin-top: 10px;
    background-color: #ffffff;
}

.funcItem {
    cursor: pointer;
    user-select: none;
}

</style>
<style>
.home .arco-list-item-meta-title {
    font-size: 18px;
    font-weight: 600;
}

.home .arco-avatar {
    background-color: transparent;
}

.grayTitle {
    color: #e0e0e0;
}

</style>
