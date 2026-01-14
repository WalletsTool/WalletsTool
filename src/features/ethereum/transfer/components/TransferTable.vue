<script setup name="TransferTable">
import { Icon } from '@iconify/vue';
import { ref, computed } from 'vue';

const props = defineProps({
  columns: {
    type: Array,
    required: true,
  },
  data: {
    type: Array,
    required: true,
  },
  rowSelection: {
    type: Object,
    default: () => ({}),
  },
  loading: {
    type: Boolean,
    default: false,
  },
  selectedKeys: {
    type: Array,
    default: () => [],
  },
  emptyData: {
    type: Boolean,
    default: false,
  },
  hoverKeys: {
    type: Array,
    default: () => [],
  },
  isSidePanelExpanded: {
    type: Boolean,
    default: true,
  },
  actionClickStates: {
    type: Object,
    default: () => ({}),
  },
  rowHoverStates: {
    type: Object,
    default: () => ({}),
  },
  queryFromAddressBalance: {
    type: Function,
    default: () => {},
  },
  queryToAddressBalanceRow: {
    type: Function,
    default: () => {},
  },
  resendTransaction: {
    type: Function,
    default: () => {},
  },
  deleteItem: {
    type: Function,
    default: () => {},
  },
  selectSucceeded: {
    type: Function,
    default: () => {},
  },
  selectFailed: {
    type: Function,
    default: () => {},
  },
  invertSelection: {
    type: Function,
    default: () => {},
  },
  showAdvancedFilter: {
    type: Function,
    default: () => {},
  },
  deleteSelected: {
    type: Function,
    default: () => {},
  },
  debouncedClearData: {
    type: Function,
    default: () => {},
  },
  setActionClickState: {
    type: Function,
    default: () => {},
  },
});

const emit = defineEmits([
  'row-click',
  'update:selected-keys',
  'open-manual-import',
  'open-file-upload',
  'download-template',
]);

const tableBool = ref(true);

function rowClick(record, event) {
  emit('row-click', record, event);
}

function updateSelectedKeys(keys) {
  emit('update:selected-keys', keys);
}

function handleManualImport() {
  emit('open-manual-import');
}

function handleFileUpload() {
  emit('open-file-upload');
}

function handleDownloadTemplate() {
  emit('download-template');
}

function handleSelectSucceeded() {
  selectSucceeded();
}

function handleSelectFailed() {
  selectFailed();
}

function handleInvertSelection() {
  invertSelection();
}

function handleShowAdvancedFilter() {
  showAdvancedFilter();
}

function handleDeleteSelected() {
  deleteSelected();
}

const uploadInputRef = ref(null);
</script>

<template>
  <div class="table-container">
    <VirtualScrollerTable
      :columns="columns"
      :data="data"
      :row-selection="rowSelection"
      :loading="loading"
      :selected-keys="selectedKeys"
      @row-click="rowClick"
      @update:selected-keys="updateSelectedKeys"
      @open-manual-import="handleManualImport"
      @open-file-upload="handleFileUpload"
      @download-template="handleDownloadTemplate"
      row-key="key"
      height="100%"
      :empty-data="emptyData"
      class="table-with-side-actions"
      :hover-keys="hoverKeys"
    >
      <template #exec_status="{ record }">
        <div
          class="exec-status-wrapper"
          @mouseenter="rowHoverStates[record.key] = true"
          @mouseleave="rowHoverStates[record.key] = false"
        >
          <a-tooltip
            content=""
            trigger="hover"
            :mouseEnterDelay="300"
            :mouseLeaveDelay="100"
            :popup-style="{ padding: 0, pointerEvents: 'auto' }"
          >
            <template #content>
              <div
                class="exec-actions"
                @mouseenter="rowHoverStates[record.key] = true"
                @mouseleave="rowHoverStates[record.key] = false"
              >
                <div
                  class="action-btn"
                  :class="{
                    'action-btn-clicked':
                      actionClickStates[record.key]?.queryFrom,
                  }"
                  @click="
                    queryFromAddressBalance(record);
                    setActionClickState(record, 'queryFrom');
                  "
                >
                  <Icon
                    :icon="
                      actionClickStates[record.key]?.queryFrom
                        ? 'mdi:check'
                        : 'mdi:arrow-up'
                    "
                  />
                  查出账余额
                </div>
                <div
                  class="action-btn"
                  :class="{
                    'action-btn-clicked':
                      actionClickStates[record.key]?.queryTo,
                  }"
                  @click="
                    queryToAddressBalanceRow(record);
                    setActionClickState(record, 'queryTo');
                  "
                >
                  <Icon
                    :icon="
                      actionClickStates[record.key]?.queryTo
                        ? 'mdi:check'
                        : 'mdi:arrow-down'
                    "
                  />
                  查到账余额
                </div>
                <div
                  class="action-btn danger"
                  :class="{
                    'action-btn-clicked': actionClickStates[record.key]?.resend,
                  }"
                  @click="
                    resendTransaction(record);
                    setActionClickState(record, 'resend');
                  "
                >
                  <Icon
                    :icon="
                      actionClickStates[record.key]?.resend
                        ? 'mdi:check'
                        : 'mdi:refresh'
                    "
                  />
                  重新转账
                </div>
              </div>
            </template>
            <a-tag v-if="record.exec_status === '0'" color="#86909c"
              >等待执行</a-tag
            >
            <a-tag v-if="record.exec_status === '1'" color="#ff7d00"
              >执行中</a-tag
            >
            <a-tag v-if="record.exec_status === '2'" color="#00b42a"
              >执行成功</a-tag
            >
            <a-tag v-if="record.exec_status === '3'" color="#f53f3f"
              >执行失败</a-tag
            >
          </a-tooltip>
        </div>
      </template>
      <template #optional="{ record }">
        <a-button
          type="text"
          size="small"
          @click.stop="deleteItem(record)"
          status="danger"
        >
          <template #icon>
            <IconDelete />
          </template>
        </a-button>
      </template>
    </VirtualScrollerTable>

    <div
      class="side-actions-panel-fixed"
      :class="{ 'side-actions-panel-collapsed': !isSidePanelExpanded }"
    >
      <Transition name="panel-slide">
        <div v-if="isSidePanelExpanded" class="side-actions-content-fixed">
          <a-tooltip content="钱包录入" position="left">
            <a-button type="primary" size="mini" @click="handleManualImport">
              <template #icon>
                <Icon icon="mdi:wallet" style="color: #165dff; font-size: 20px" />
              </template>
            </a-button>
          </a-tooltip>
          <a-tooltip content="导入文件" position="left">
            <a-button type="primary" size="mini" @click="handleFileUpload">
              <template #icon>
                <Icon icon="mdi:upload" style="color: #00b42a; font-size: 20px" />
              </template>
            </a-button>
          </a-tooltip>
          <a-tooltip content="清空表格" position="left">
            <a-button
              type="primary"
              status="danger"
              size="mini"
              @click="debouncedClearData"
            >
              <template #icon>
                <Icon
                  icon="mdi:delete-sweep"
                  style="color: #f53f3f; font-size: 20px"
                />
              </template>
            </a-button>
          </a-tooltip>
          <a-tooltip content="下载模板" position="left">
            <a-button size="mini" @click="handleDownloadTemplate">
              <template #icon>
                <Icon
                  icon="mdi:file-download"
                  style="color: #4e5969; font-size: 20px"
                />
              </template>
            </a-button>
          </a-tooltip>
          <div class="side-actions-divider"></div>
          <a-tooltip content="选中成功的数据" position="left">
            <a-button
              type="outline"
              status="success"
              size="mini"
              @click="handleSelectSucceeded"
            >
              <template #icon>
                <Icon
                  icon="mdi:check-circle"
                  style="color: #00b42a; font-size: 20px"
                />
              </template>
            </a-button>
          </a-tooltip>
          <a-tooltip content="选中失败的数据" position="left">
            <a-button type="outline" status="danger" size="mini" @click="handleSelectFailed">
              <template #icon>
                <Icon icon="mdi:close-circle" style="color: #f53f3f; font-size: 20px" />
              </template>
            </a-button>
          </a-tooltip>
          <a-tooltip content="反选" position="left">
            <a-button type="outline" size="mini" @click="handleInvertSelection">
              <template #icon>
                <Icon icon="mdi:swap-horizontal" style="color: #165dff; font-size: 20px" />
              </template>
            </a-button>
          </a-tooltip>
          <a-tooltip content="高级筛选" position="left">
            <a-button type="primary" size="mini" @click="handleShowAdvancedFilter">
              <template #icon>
                <Icon icon="mdi:filter" style="color: #165dff; font-size: 20px" />
              </template>
            </a-button>
          </a-tooltip>
          <a-tooltip content="删除选中" position="left">
            <a-button type="outline" status="danger" size="mini" @click="handleDeleteSelected">
              <template #icon>
                <Icon icon="mdi:trash-can" style="color: #f53f3f; font-size: 20px" />
              </template>
            </a-button>
          </a-tooltip>
          <div class="side-actions-divider"></div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.table-container {
  flex: 1;
  display: flex;
  position: relative;
  overflow: hidden;
  width: 100%;
}

.table-with-side-actions {
  margin-right: 58px;
  margin-top: 0;
  height: 100%;
  transition: margin-right 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.side-actions-panel-fixed {
  position: fixed;
  right: 10px;
  top: 50px;
  bottom: 45px;
  width: 50px;
  background: var(--color-bg-2, #ffffff);
  border: 1px solid var(--color-border, #e5e6eb);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 10px;
  pointer-events: none;
  box-shadow: 3px 0px 6px 0px rgba(0, 0, 0, 0.06),
    -1px 0 4px rgba(0, 0, 0, 0.03);
  z-index: 10;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.side-actions-panel-fixed.side-actions-panel-collapsed {
  width: 50px;
  background: transparent;
  border: none;
  box-shadow: none;
  padding: 0;
}

.side-actions-content-fixed {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  opacity: 1;
  pointer-events: auto;
  flex: 1;
}

.side-actions-divider {
  width: 24px;
  height: 1px;
  background: linear-gradient(
    to right,
    transparent,
    var(--color-border, #e2e4e8),
    transparent
  );
  margin: 13px 0;
}

.side-actions-content-fixed .arco-btn {
  width: 38px;
  height: 38px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  border: 1px solid var(--color-border, #e2e4e8);
  background: var(--color-fill-1, #f7f8fa);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.side-actions-content-fixed .arco-btn:hover {
  background: var(--color-primary-light-1, #e8f0ff);
  border-color: var(--color-primary-5, #4086ff);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(22, 93, 255, 0.15);
}

.side-actions-content-fixed .arco-btn > .arco-btn-icon {
  margin: 0;
  font-size: 20px;
  color: var(--text-color-secondary, #6b778c);
}

.side-actions-content-fixed .arco-btn:hover > .arco-btn-icon {
  color: var(--color-primary-6, #165dff);
}

.side-actions-content-fixed .arco-btn[type='primary'] {
  background: linear-gradient(
    135deg,
    var(--color-primary-6, #165dff) 0%,
    var(--color-primary-5, #4086ff) 100%
  );
  border-color: var(--color-primary-6, #165dff);
  box-shadow: 0 2px 6px rgba(22, 93, 255, 0.25);
}

.side-actions-content-fixed .arco-btn[type='primary'] > .arco-btn-icon {
  color: #ffffff;
}

.side-actions-content-fixed .arco-btn[type='primary']:hover {
  background: linear-gradient(
    135deg,
    var(--color-primary-5, #4086ff) 0%,
    var(--color-primary-6, #165dff) 100%
  );
  box-shadow: 0 4px 12px rgba(22, 93, 255, 0.35);
  transform: translateY(-2px);
}

.side-actions-content-fixed .arco-btn[status='success'] {
  background: linear-gradient(
    135deg,
    var(--color-success-6, #0fa962) 0%,
    var(--color-success-5, #12b576) 100%
  );
  border-color: var(--color-success-6, #0fa962);
  box-shadow: 0 2px 6px rgba(15, 169, 98, 0.25);
}

.side-actions-content-fixed .arco-btn[status='success'] > .arco-btn-icon {
  color: #ffffff;
}

.side-actions-content-fixed .arco-btn[status='success']:hover {
  background: linear-gradient(
    135deg,
    var(--color-success-5, #12b576) 0%,
    var(--color-success-6, #0fa962) 100%
  );
  box-shadow: 0 4px 12px rgba(15, 169, 98, 0.35);
  transform: translateY(-2px);
}

.side-actions-content-fixed .arco-btn[status='danger'] {
  background: linear-gradient(
    135deg,
    var(--color-danger-6, #f53f3f) 0%,
    var(--color-danger-5, #ff7d7d) 100%
  );
  border-color: var(--color-danger-6, #f53f3f);
  box-shadow: 0 2px 6px rgba(245, 63, 63, 0.25);
}

.side-actions-content-fixed .arco-btn[status='danger'] > .arco-btn-icon {
  color: #ffffff;
}

.side-actions-content-fixed .arco-btn[status='danger']:hover {
  background: linear-gradient(
    135deg,
    var(--color-danger-5, #ff7d7d) 0%,
    var(--color-danger-6, #f53f3f) 100%
  );
  box-shadow: 0 4px 12px rgba(245, 63, 63, 0.35);
  transform: translateY(-2px);
}

.panel-slide-enter-active,
.panel-slide-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.panel-slide-enter-from,
.panel-slide-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.mainTable:has(.side-actions-panel-fixed.side-actions-panel-collapsed)
  .table-with-side-actions {
  margin-right: 0;
}
</style>
