<script setup name="monitor">
import {IconDelete, IconDoubleLeft, IconDownload, IconPlus} from '@arco-design/web-vue/es/icon';
import {useRouter} from "vue-router";
import {nextTick, onBeforeMount, onMounted, reactive, ref} from "vue";
import {invoke} from "@tauri-apps/api/tauri";
import {Notification} from "@arco-design/web-vue";
import balance_utils from "@/scripts/balance/balance_utils.js";
import token_utils from "@/scripts/token/token_utils.js";
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
const selectedKeys = ref([]);
// 选择配置
const rowSelection = reactive({
  type: 'checkbox',
  showCheckedAll: true,
  onlyCurrent: false,
});

// 点击行实现选中和取消
function rowClick(record, event) {
  const index = selectedKeys.value.indexOf(record.address)
  index >= 0 ? selectedKeys.value.splice(index, 1) : selectedKeys.value.push(record.address)
}

// 分页
const pagination = ref(false);
const scrollbar = ref(true);
// 滚动条设置
let scroll = {
  y: document.documentElement.clientHeight >= 625 ? document.documentElement.clientHeight - 445 : 180
}
let tableBool = ref(true)
// rpc默认值
const rpcValue = ref('');
// 当前rpc
const currentRpc = ref({});
// rpc自定义字段名
const rpcFieldNames = {value: 'key', label: 'scan_url'}
// 主网选择器
let rpcOptions = ref([])
// coin默认值
let coinValue = ref('');
// coin自定义字段名
const coinFieldNames = {value: 'key', label: 'coin'}
// 币种选择器
const coinOptions = ref([]);
// 查询余额按钮loading
let balanceLoading = ref(false)
// 详细配置
const form = reactive({
  interval: 0,
  thread_count: 3
})
// 录入 钱包地址 弹窗
let visible = ref(false)
let importText = ref('')
// 添加代币弹窗
let addCoinVisible = ref(false)
let coinAddress = ref('')
// 删除代币弹窗
let deleteTokenVisible = ref(false)
// 删除信息弹窗
let deleteItemVisible = ref(false)
// 当前币种名称
let currentCoin = ref({})
// 当前数据的key
let currentItemKey = ref('')

// 初始化RPC列表
onBeforeMount(async () => {
  rpcOptions.value = await invoke('get_chain_list')
  rpcValue.value = rpcOptions.value[0].key
  currentRpc.value = rpcOptions.value[0]
  // 获取rpc对应的代币列表
  await rpcChange()
})

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

// RPC变化事件
async function rpcChange() {
  coinOptions.value = await invoke("get_coin_list", {chain: rpcValue.value, page: 'balance'})
  coinValue.value = coinOptions.value[0].key
  currentCoin.value = coinOptions.value[0]
  currentRpc.value = rpcOptions.value.filter(item => item.key === rpcValue.value)[0]
}

// coin变化事件
async function coinChange(value) {
  currentCoin.value = coinOptions.value.filter(item => item.key === value)[0]
  console.log(currentCoin.value)
}

// 删除代币方法
function deleteToken() {
  if (rpcValue.value === 'starknet') {
    Notification.warning(' StarkNet 暂不支持删除代币！');
    return
  }
  if (rpcValue.value === 'okt') {
    Notification.warning(' OKT Chain 暂不支持删除代币！');
    return
  }
  if (rpcValue.value === 'geth') {
    Notification.warning(' Goerli Ethereum 暂不支持删除代币！');
    return
  }
  if (rpcValue.value === 'scroll') {
    Notification.warning(' Scroll Alpha TestNet 暂不支持删除代币！');
    return
  }
  deleteTokenVisible.value = true
}

// 删除代币取消
function deleteTokenCancel() {
  deleteTokenVisible.value = false
}

// 删除代币确认
async function deleteTokenConfirm() {
  console.log('确认删除代币')
  deleteTokenVisible.value = false
  await invoke("remove_coin", {chain: rpcValue.value, page: 'balance', key: currentCoin.value.key}).then(() => {
    Notification.success('删除成功！');
    // 删除成功后重新获取代币列表
    rpcChange()
  }).catch(() => {
    Notification.error('删除失败！');
  })
}

// 导入事件触发
function handleAddCoinClick() {
  if (rpcValue.value === 'starknet') {
    Notification.warning(' StarkNet 暂不支持添加代币！');
    return
  }
  if (rpcValue.value === 'okt') {
    Notification.warning(' OKT Chain 暂不支持添加代币！');
    return
  }
  if (rpcValue.value === 'geth') {
    Notification.warning(' Goerli Ethereum 暂不支持添加代币！');
    return
  }
  if (rpcValue.value === 'scroll') {
    Notification.warning(' Scroll Alpha TestNet 暂不支持添加代币！');
    return
  }
  addCoinVisible.value = true
}

// 添加代币弹窗取消
function handleAddCoinCancel() {
  addCoinVisible.value = false
}

// 添加代币核心方法
function addCoinFunc() {
  return new Promise((resolve, reject) => {
    const scan_api = currentRpc.value.scan_api
    const verify_api = currentRpc.value.verify_api
    const check_verify_api = currentRpc.value.check_verify_api

    console.log('校验是否存在代理合约')
    // 校验是否存在代理合约
    token_utils.getProxyAddress(coinAddress.value, verify_api, check_verify_api).then(proxy_address => {
      let address = coinAddress.value
      if (proxy_address) {
        address = proxy_address
      }
      console.log('获取合约ABI')
      // 获取合约ABI
      token_utils.getAbi(address, scan_api).then(abi => {
        console.log('获取代币名称')
        token_utils.getTokenSymbol(rpcValue.value, coinAddress.value, abi).then(symbol => {
          let json = {
            "key": symbol.toLowerCase(),
            "coin": symbol,
            "type": "token",
            "contract_type": "",
            "contract_address": coinAddress.value,
            "abi": abi
          }
          console.log('添加代币')
          // 添加代币
          invoke('add_coin', {
            chain: rpcValue.value,
            page: 'balance',
            objJson: JSON.stringify(json)
          }).then(() => {
            addCoinVisible.value = false
            coinAddress.value = ''
            resolve()
          }).catch(err => {
            console.log(err)
            reject('添加代币失败！')
          })
        }).catch(err => {
          console.log(err)
          reject('获取代币名称异常，添加代币失败！')
        })
      }).catch(err => {
        reject(err)
      })
    }).catch(() => {
      reject('校验合约地址异常，添加代币失败！')
    })
  })
}

// 添加代币弹窗确认
const handleAddCoinBeforeOk = async () => {
  coinAddress.value = coinAddress.value.trim()
  if (!coinAddress.value) {
    Notification.warning('请输入代币地址！');
    return false
  }
  let flag = false
  await addCoinFunc().then(() => {
    Notification.success('添加代币成功！');
    flag = true
  }).catch(err => {
    Notification.error(err);
  })
  // 删除成功后重新获取代币列表
  rpcChange()
  return flag
}

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
  if (balanceLoading.value) {
    Notification.warning('请停止或等待执行完成后再删除数据！');
    return
  }
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

// 查询余额
async function queryBalance() {
  if (data.value.length === 0) {
    Notification.warning('请先导入地址！');
    return
  }
  if (currentCoin.value.type === 'base' || currentCoin.value.type === 'token') {
    balanceLoading.value = true
    data.value.forEach(item => {
      item.plat_balance = ''
      item.coin_balance = ''
      item.error_msg = ''
    })
    balance_utils.exec_group_query(rpcValue.value, currentCoin.value, data.value, false, () => {
      Notification.success('查询成功！');
      balanceLoading.value = false
    }, form.thread_count)
  } else {
    Notification.warning('查询 coin 类型错误！');
  }
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
  if (balanceLoading.value) {
    Notification.warning('请停止或等待执行完成后再删除数据！');
    return
  }
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

const formRef = ref(null)

// 返回首页
function goHome() {
  router.push({
    name: 'home'
  })
}
</script>

<template>
  <div class="container monitor">
    <span class="pageTitle">链上地址监控</span>
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
    <!-- RPC 选择器 -->
    <a-select v-model="rpcValue" :options="rpcOptions" @change="rpcChange" :field-names="rpcFieldNames" size="large"
              :style="{'margin-top':'10px'}">
      <template #label="{ data }">
        <img alt="" :src="data?.pic_url" style="width: 18px;height: 18px">
        <span style="margin-left: 10px">{{ data?.chain }}</span>
        <span style="margin-left: 50px">{{ data?.scan_url }}</span>
      </template>
      <template #option="{ data }">
        <img alt="" :src="data?.pic_url" style="width: 16px;height: 16px">
        <span style="margin-left: 10px">{{ data?.chain }}</span>
        <span style="position: absolute;right: 20px;">{{ data?.scan_url }}</span>
      </template>
    </a-select>
    <!-- 管理代币 -->
    <div>
      <div class="subTitle">
        选择代币：
      </div>
      <div style="display: flex;flex-direction: row;align-items: center;margin-top: 10px">
        <a-button type="outline" status="normal" @click="handleAddCoinClick"
                  style="margin-left: 10px">
          <icon-plus/>
          <span style="margin-left: 5px">添加代币</span>
        </a-button>
        <!-- 代币 选择器 -->
        <a-select v-model="coinValue" :options="coinOptions" :field-names="coinFieldNames"
                  :style="{'margin-left':'10px',flex:'1'}" @change="coinChange">
          <template #label="{ data }">
            <span style="margin-left: 10px">{{ data?.coin }}</span>
          </template>
        </a-select>
        <a-button type="primary" status="danger" @click="deleteToken" style="margin-left: 10px">删除代币
        </a-button>
      </div>
    </div>
    <!-- 相关设置 -->
    <div style="display: flex;padding-top: 10px;flex-direction: column;">
      <div style="display: flex">
        <!-- 细节配置 -->
        <a-form ref="formRef" :model="form" layout="vertical">
          <a-row style="height: 100px">
            <a-form-item field="thread_count"
                         label="线程数" style="width: 240px;padding: 10px"
                         tooltip="同时执行查询的钱包数量（1-10）之间">
              <a-input-number :max="10" :min="1" mode="button" v-model="form.thread_count"/>
            </a-form-item>
          </a-row>
        </a-form>
        <div style="width: 300px;display: flex;align-items: center;justify-content: center;">
          <a-button type="outline" status="normal" style="margin-left: 10px;height: 50px;width: 200px;"
                    @click="queryBalance"
                    :loading="balanceLoading">查询余额
          </a-button>
        </div>
      </div>
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
  <!-- 添加代币弹窗 -->
  <a-modal v-model:visible="addCoinVisible" :width="700" title="添加代币" @cancel="handleAddCoinCancel"
           :on-before-ok="handleAddCoinBeforeOk" unmountOnClose>
    <a-input v-model="coinAddress" placeholder="请输入代币合约地址" allow-clear/>
  </a-modal>
  <!-- 删除代币确认框 -->
  <a-modal v-model:visible="deleteTokenVisible" title="删除确认">
    <div>确认删除【 {{ currentCoin.coin }} 】代币？</div>
    <template #footer>
      <a-button @click="deleteTokenCancel">取消</a-button>
      <a-button type="primary" status="danger" @click="deleteTokenConfirm" style="margin-left: 10px">确定
      </a-button>
    </template>
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

.subTitle {
  font-weight: 600;
  font-size: 16px;
  margin: 10px 0 0 10px;
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
.monitor {
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
