<script setup>
import { defineAsyncComponent, ref, reactive } from 'vue';
import { Message, Modal } from '@arco-design/web-vue';
import { 
  IconPlus, 
  IconImport, 
  IconDownload, 
  IconDelete,
  IconSync
} from '@arco-design/web-vue/es/icon';

const WalletSystemImportModal = defineAsyncComponent(() => import('@/components/WalletSystemImportModal.vue'))

// Mock data
const wallets = ref([
  { id: 1, address: '0x742d35Cc6634C0532925a3b844Bc454e4438f44e', label: 'Main Wallet', group: 'Default', proxy: '127.0.0.1:7890' },
  { id: 2, address: '0x123...abc', label: 'Airdrop 1', group: 'Group A', proxy: '192.168.1.10:1080' },
  { id: 3, address: '0x456...def', label: 'Airdrop 2', group: 'Group A', proxy: 'Direct' },
]);

const columns = [
  { title: 'ID', dataIndex: 'id', width: 60 },
  { title: '地址', dataIndex: 'address' },
  { title: '备注', dataIndex: 'label' },
  { title: '分组', dataIndex: 'group' },
  { title: '代理', dataIndex: 'proxy' },
  { title: '操作', slotName: 'action', width: 120 }
];

const isAddModalVisible = ref(false);
const systemImportVisible = ref(false)
const form = reactive({
  address: '',
  label: '',
  group: 'Default',
  proxy: 'Direct'
});

const handleAdd = () => {
  isAddModalVisible.value = true;
};

const handleImport = () => {
  Message.info('导入功能开发中...');
};

const openSystemSync = () => {
  systemImportVisible.value = true
}

const handleSystemSync = (imported) => {
  const existing = new Set(wallets.value.map((w) => String(w.address || '').toLowerCase()))
  const list = (imported || [])
    .map((w) => ({
      address: String(w?.address || '').trim(),
      label: (w?.name || '').trim(),
      group: (w?.group_name || '').trim(),
      chain_type: w?.chain_type,
    }))
    .filter((w) => w.address)

  const newOnes = []
  for (const w of list) {
    const key = w.address.toLowerCase()
    if (existing.has(key)) continue
    existing.add(key)
    newOnes.push(w)
  }

  if (!newOnes.length) {
    Message.info('没有新钱包需要同步')
    return
  }

  const startId = wallets.value.reduce((max, w) => Math.max(max, Number(w.id) || 0), 0)
  newOnes.forEach((w, idx) => {
    wallets.value.push({
      id: startId + idx + 1,
      address: w.address,
      label: w.label || `Wallet ${startId + idx + 1}`,
      group: w.group || 'Default',
      proxy: 'Direct',
      chain_type: w.chain_type,
    })
  })

  Message.success(`同步成功，新增 ${newOnes.length} 个钱包`)
}

const handleExport = () => {
  Message.info('导出功能开发中...');
};

const handleDelete = (record) => {
  wallets.value = wallets.value.filter(w => w.id !== record.id);
  Message.success('删除成功');
};

const handleSubmitAdd = () => {
  if (!form.address) {
    Message.error('请输入钱包地址');
    return;
  }
  
  wallets.value.push({
    id: wallets.value.length + 1,
    address: form.address,
    label: form.label || `Wallet ${wallets.value.length + 1}`,
    group: form.group,
    proxy: form.proxy
  });
  
  isAddModalVisible.value = false;
  Message.success('添加成功');
  
  // Reset form
  form.address = '';
  form.label = '';
  form.group = 'Default';
  form.proxy = 'Direct';
};
</script>

<template>
  <div class="wallet-manager">
    <!-- Toolbar -->
    <div class="toolbar">
      <div class="left-tools">
        <a-button type="primary" @click="handleAdd">
          <template #icon><icon-plus /></template>
          添加钱包
        </a-button>
        <a-button type="primary" status="warning" @click="openSystemSync">
          <template #icon><icon-sync /></template>
          从系统同步
        </a-button>
        <a-button type="outline" status="success" @click="handleImport">
          <template #icon><icon-import /></template>
          导入 (Excel)
        </a-button>
        <a-button type="outline" @click="handleExport">
          <template #icon><icon-download /></template>
          导出配置
        </a-button>
      </div>
      <div class="right-tools">
        <a-input-search placeholder="搜索地址/备注..." style="width: 250px" allow-clear />
      </div>
    </div>

    <!-- Table -->
    <div class="table-wrapper">
      <a-table 
        :data="wallets" 
        :columns="columns" 
        :pagination="{ pageSize: 15 }"
        :bordered="false"
        hoverable
      >
        <template #action="{ record }">
          <a-button type="text" status="danger" size="mini" @click="handleDelete(record)">
            <template #icon><icon-delete /></template>
            删除
          </a-button>
        </template>
      </a-table>
    </div>

    <!-- Add Wallet Modal -->
    <a-modal v-model:visible="isAddModalVisible" title="添加钱包" @ok="handleSubmitAdd">
      <a-form :model="form" layout="vertical">
        <a-form-item label="钱包地址" required>
          <a-input v-model="form.address" placeholder="0x..." />
        </a-form-item>
        <a-form-item label="备注">
          <a-input v-model="form.label" placeholder="我的钱包" />
        </a-form-item>
        <a-row :gutter="16">
          <a-col :span="12">
            <a-form-item label="分组">
              <a-input v-model="form.group" />
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="代理 (Proxy)">
              <a-input v-model="form.proxy" placeholder="Direct or IP:Port" />
            </a-form-item>
          </a-col>
        </a-row>
      </a-form>
    </a-modal>

    <WalletSystemImportModal
      v-model:visible="systemImportVisible"
      ecosystem="all"
      import-mode="address_only"
      :title="'从系统同步钱包'"
      @confirm="handleSystemSync"
      @cancel="systemImportVisible = false"
    />
  </div>
</template>

<style scoped>
.wallet-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  background: var(--color-bg-2);
  border-radius: 8px;
  border: 1px solid var(--color-border);
}

.left-tools {
  display: flex;
  gap: 10px;
}

.table-wrapper {
  flex: 1;
  background: var(--color-bg-2);
  border-radius: 8px;
  padding: 10px;
  overflow: hidden;
  border: 1px solid var(--color-border);
}

/* Custom Table Styles override */
:deep(.arco-table-cell) {
  background: transparent !important;
}
</style>
