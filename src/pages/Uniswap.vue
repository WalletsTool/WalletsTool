<script setup name="uniswap">
import {IconDelete, IconDoubleLeft, IconDownload, IconPlus} from '@arco-design/web-vue/es/icon';
import {useRouter} from "vue-router";
import {nextTick, onMounted, reactive, ref, watch} from "vue";
import {Notification} from "@arco-design/web-vue";
import {utils as xlUtils, writeFile} from "xlsx";

const router = useRouter()
// table列名
const columns = [
    {
        title: '序号',
        align: 'center',
        width: '60',
        slotName: 'index'
    },
    {
        title: '钱包地址',
        align: 'center',
        dataIndex: 'address',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '平台币余额',
        align: 'center',
        dataIndex: 'plat_balance',
        width: '120',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '代币余额',
        align: 'center',
        dataIndex: 'coin_balance',
        width: '120',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '状态',
        align: 'center',
        slotName: 'exec_status',
        width: '100',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '错误信息',
        align: 'center',
        dataIndex: 'error_msg',
        ellipsis: "true",
        tooltip: 'true'
    },
    {
        title: '操作',
        align: 'center',
        slotName: 'optional',
        width: '60',
        ellipsis: "true",
        tooltip: 'true'
    }
]
let tableLoading = ref(false)
const data = ref([])
// 选中的数据key
const selectedKeys = ref([])
// 选择配置
const rowSelection = reactive({
    type: 'checkbox',
    showCheckedAll: true,
    onlyCurrent: false,
})
// 分页
const pagination = ref(false)
const scrollbar = ref(true)
// 滚动条设置
let scroll = {
    y: document.documentElement.clientHeight >= 625 ? document.documentElement.clientHeight - 445 : 180
}
let tableBool = ref(true)

// 点击行实现选中和取消
function rowClick(record, event) {
    const index = selectedKeys.value.indexOf(record.address)
    index >= 0 ? selectedKeys.value.splice(index, 1) : selectedKeys.value.push(record.address)
}
// 录入 钱包地址 弹窗
let visible = ref(false)
let importText = ref('')
// 当前数据的key
let currentItemKey = ref('')
// 删除数据弹窗
let deleteItemVisible = ref(false)

onMounted(() => {
    // 监听页面高度
    window.onresize = () => {
        return (() => {
            window.screenHeight = document.documentElement.clientHeight
            setTimeout(() => {
                if (window.screenHeight >= 700) {
                    tableBool.value = false
                    scroll = {
                        y: window.screenHeight - 445
                    }
                    nextTick(() => {
                        tableBool.value = true
                    })
                } else {
                    tableBool.value = false
                    scroll = {
                        y: 180
                    }
                    nextTick(() => {
                        tableBool.value = true
                    })
                }
            }, 200)
        })()
    }
})

// 清空列表
function clearData() {
    data.value = []
    Notification.success('清空列表成功！');
}

// 导入事件触发
function handleClick() {
    visible.value = true
}

// 导入弹窗关闭事件
function handleCancel() {
    // TODO 判断是否正在进行数据处理 如果进行数据处理则提示
    visible.value = false
    importText.value = ''
}

// 导入弹窗保存事件
const handleBeforeOk = () => {
    let importList = importText.value.split('\n').filter(item => item !== '')
    const total_count = importList.length
    importList = importList.filter(item => data.value.length === 0 || !data.value.find(obj => obj.address === item))
    const success_count = importList.length
    const fail_count = total_count - success_count
    data.value.push(...importList.map(item => {
        return {
            address: item,
            plat_balance: '',
            coin_balance: '',
            exec_status: '0',
            error_msg: ''
        }
    }))
    if (fail_count > 0) {
        Notification.warning({
            title: '导入完成！',
            content: `执行${total_count}条，成功${success_count}条，失败${fail_count}条！`,
        })
    } else {
        Notification.success({
            title: '导入成功！',
            content: `成功导入${total_count}条`,
        })
    }
    importText.value = ''
    return true
}

// 删除数据
function deleteItem(item) {
    // 删除确认
    deleteItemModalShow(item.address)
}

// 删除数据弹窗显示
function deleteItemModalShow(address) {
    deleteItemVisible.value = true
    currentItemKey.value = address
}

// 删除item取消
function deleteItemCancel() {
    deleteItemVisible.value = false
}

// 删除item确认
async function deleteItemConfirm() {
    deleteItemVisible.value = false
    data.value = data.value.filter(obj => currentItemKey.value !== obj.address)
    Notification.success('删除成功！');
}

// 选中成功
function selectSucceeded() {
    selectedKeys.value = data.value.filter(item => item.exec_status === '2').map(item => item.address)
}

// 选中失败
function selectFailed() {
    selectedKeys.value = data.value.filter(item => item.exec_status === '3').map(item => item.address)
}

// 反选
function InvertSelection() {
    selectedKeys.value = data.value.filter(item => selectedKeys.value.indexOf(item.address) < 0).map(item => item.address)
}

function deleteSelected() {
    data.value = data.value.filter(item => !selectedKeys.value.includes(item.address))
    Notification.success('删除成功')
}

function exportExcel() {
    if (data.value.length === 0) {
        Notification.warning('无法导出空列表！');
        return
    }
    let export_data = [['地址', '平台余额', '代币余额', '执行状态', '错误信息']]
    data.value.forEach(item => {
        export_data.push([item.address, item.plat_balance, item.coin_balance, item.exec_status, item.error_msg])
    })
    // 创建工作簿
    const workbook = xlUtils.book_new();
    // 创建工作表
    const worksheet = xlUtils.aoa_to_sheet(export_data);
    // 将工作表添加到工作簿
    xlUtils.book_append_sheet(workbook, worksheet, 'Sheet1');
    // 导出文件
    writeFile(workbook, 'balance_data.xlsx');
}

// 返回首页
function goHome() {
    router.push({
        name: 'home'
    })
}
</script>

<template>
    <div class="container uniswap">
        <span class="pageTitle">Uniswap批量交易</span>
        <!-- 工具栏 -->
        <div class="toolBar">
            <a-button type="primary" @click="handleClick()">录入钱包地址</a-button>
            <a-divider direction="vertical"/>
            <!-- 选择操作区按钮 -->
            <a-button type="outline" status="success" @click="selectSucceeded">选中成功</a-button>
            <a-button type="outline" status="danger" style="margin-left: 10px" @click="selectFailed">选中失败</a-button>
            <a-button type="outline" status="normal" style="margin-left: 10px" @click="InvertSelection">反选</a-button>
            <a-button type="primary" status="danger" style="margin-left: 10px" @click="deleteSelected">删除选中
            </a-button>
            <a-divider direction="vertical"/>
            <a-button type="primary" status="success" @click="exportExcel">
                <template #icon>
                    <icon-download/>
                </template>
                导出表格
            </a-button>
            <a-button class="goHome" type="outline" status="success" @click="goHome">
                <template #icon>
                    <icon-double-left/>
                </template>
                返回首页
            </a-button>
            <a-button type="outline" status="normal" style="float: right;margin-right: 10px"
                      @click="clearData">清空列表
            </a-button>
        </div>
        <!-- 操作账号表格 -->
        <div class="mainTable">
            <a-table v-if="tableBool" row-key="address" :columns="columns" :column-resizable="true" :data="data"
                     :row-selection="rowSelection" :loading="tableLoading"
                     :scroll="scroll"
                     :scrollbar="scrollbar" @row-click="rowClick"
                     v-model:selectedKeys="selectedKeys" :pagination="pagination">
                <template #index="{ rowIndex }">
                    {{ rowIndex + 1 }}
                </template>
                <template #exec_status="{ rowIndex }">
                    <a-tag v-if="data[rowIndex].exec_status === '0'" color="#86909c">等待执行</a-tag>
                    <a-tag v-if="data[rowIndex].exec_status === '1'" color="#ff7d00">执行中</a-tag>
                    <a-tag v-if="data[rowIndex].exec_status === '2'" color="#00b42a">执行成功</a-tag>
                    <a-tag v-if="data[rowIndex].exec_status === '3'" color="#f53f3f">执行失败</a-tag>
                </template>
                <template #optional="{ record }">
                    <icon-delete style="font-size: 16px" @click.prevent="deleteItem(record)"/>
                </template>
            </a-table>
        </div>
    </div>
  <!-- 录入弹窗 -->
    <a-modal v-model:visible="visible" :width="700" title="录入钱包地址" @cancel="handleCancel"
             :on-before-ok="handleBeforeOk">
        <a-textarea v-model="importText" style="margin-top: 10px" placeholder="格式：一行一个" allow-clear :auto-size="{
            minRows:15,
            maxRows:20
          }"/>
    </a-modal>
  <!-- 删除数据确认框 -->
    <a-modal v-model:visible="deleteItemVisible" title="删除确认">
        <div>确认删除地址为【
            {{ currentItemKey.substring(0, 15) + '......' + currentItemKey.substring(currentItemKey.length - 15) }}
            】的数据？
        </div>
        <template #footer>
            <a-button @click="deleteItemCancel">取消</a-button>
            <a-button type="primary" status="danger" @click="deleteItemConfirm" style="margin-left: 10px">确定
            </a-button>
        </template>
    </a-modal>
</template>

<style scoped lang="less">
.container {
  background-color: white;
  padding: 10px;
  min-width: 1240px;
}

.pageTitle {
  position: fixed;
  padding: 0 30px;
  user-select: none;
  text-align: start;
  line-height: 100px;
  font-size: 100px;
  background-image: linear-gradient(to bottom, #f2f3f5, #ffffff);
  -webkit-background-clip: text;
  font-weight: 600;
  height: 120px;
  right: -10px;
  border-radius: 30px;
  color: transparent;
  top: 15px;
  z-index: 0;
}

.toolBar {
  margin-top: 10px;
}

.goHome {
  float: right;
  background-color: white;
}

.goHome:hover {
  background-color: #ffffffa6;
}

.mainTable {
  margin-top: 20px;
}

.arco-form-item {
  padding: 20px 10px 0 10px;
  margin-bottom: 10px;
}

.arco-btn-secondary.arco-btn-loading {
  color: #ffffff;
  background-color: #11c06f;
}

.arco-btn-secondary.arco-btn-loading:hover {
  color: #ffffff;
  background-color: #fc0934;
}

.arco-btn-outline.arco-btn-loading:hover {
  color: #ffffff;
  background-color: #fc0934;
  border: none;
}

.arco-radio-button.arco-radio-checked {
  color: #ffffff;
  background-color: #165dff;
}
</style>
<style lang="less">
.uniswap {
  .arco-table-body {
    min-height: 355px;

    .arco-table-element .arco-empty {
      min-height: 330px;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
    }
  }
}
</style>
