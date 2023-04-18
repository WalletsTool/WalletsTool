<script setup name="home">
import {useRouter} from 'vue-router'

const router = useRouter()

// 功能菜单列表
const funcList = [
    {
        title: "钱包多对多转账",
        desc: "目前支持链：ETH、OP、ARB、BSC、Polygon（持续更新中）",
        picture: "pic/transfer.png",
        pageName: "transfer"
    },
    {
        title: "链上余额查询",
        desc: "目前支持链：ETH、OP、ARB、BSC、Polygon、StarkNet（持续更新中）",
        picture: "pic/balance.png",
        pageName: "balance"
    },
    {
        title: "链上地址监控",
        desc: "目前支持链：ETH、OP、ARB、BSC、Polygon（持续更新中）",
        picture: "pic/monitor.png",
        pageName: "monitor"
    }
]

// 跳转到钱包多对多转账
function goPage(pageName) {
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
                <a-list-item-meta
                        :title="item.title"
                        :description="item.desc"
                >
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
</style>
