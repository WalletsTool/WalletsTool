<template>
  <div class="virtual-scroller-table" :style="{ height: height, width: '100%' }">
    <!-- 表头 -->
    <div class="table-header">
      <div class="header-row">
        <!-- 选择列 -->
        <div v-if="rowSelection" class="header-cell checkbox-cell">
          <input
            type="checkbox"
            :checked="isAllSelected"
            :indeterminate="isIndeterminate"
            @change="toggleSelectAll"
            class="checkbox"
          />
        </div>
        <!-- 普通列 -->
        <div
          v-for="column in sortedColumns"
          :key="column.dataIndex || column.slotName"
          class="header-cell"
          :style="{ ...getHeaderColumnStyle(column), textAlign: column.align || 'left' }"
        >
          {{ column.title }}
        </div>
      </div>
    </div>

    <!-- 虚拟滚动内容 -->
    <div
      class="table-body"
      :style="{ height: `calc(${height} - 40px)` }"
      @wheel="handleWheel"
    >
      <!-- 空数据提示 -->
      <div v-if="showEmptyData" class="empty-data">
        <template v-if="pageType === 'balance'">
          <div class="empty-icon">
            <Icon icon="icon-park-outline:wallet" style="width: 64px; height: 64px;"/>
          </div>
          <div class="empty-text">还没有钱包数据</div>
          <div class="empty-text-second">请先导入钱包开始查询余额</div>
          <div>
            <a-button
              type="primary"
              style="margin-top: 12px"
              @click="$emit('open-manual-import')"
            >
              <icon icon="mdi:upload" :size="16" style="margin-right: 4px" />
              手动录入钱包
            </a-button>
          </div>
        </template>
        <template v-else-if="pageType === 'monitor'">
          <div class="empty-icon">
            <Icon icon="icon-park-outline:wallet" style="width: 64px; height: 64px;"/>
          </div>
          <div class="empty-text">还没有监控数据</div>
        </template>
        <template v-else-if="pageType === 'transfer'">
          <div class="empty-icon">
            <Icon icon="icon-park-outline:wallet" :style="{ width: '64px', height: '64px' }" />
          </div>
          <div class="empty-text">还没有转账数据</div>
          <div class="empty-text-second">请先录入钱包或者上传文件开始批量转账</div>
          <div>
            <a-button
              type="primary"
              style="margin-top: 12px"
              @click="handleEmptyAction('manual')"
            >
              <icon icon="mdi:upload" :size="16" style="margin-right: 4px" />
              手动录入钱包
            </a-button>
            <a-button
              type="primary"
              style="margin-top: 12px;margin-left: 20px"
              status="success"
              @click="handleEmptyAction('upload')"
            >
              <icon icon="mdi:upload" :size="16" style="margin-right: 4px" />
              上传文件导入
            </a-button>
          </div>
          <div style="margin-top: 15px; display: flex; align-items: center; justify-content: center;">
            <Icon
              icon="mdi:info"
              :size="12"
              style="margin-right: 4px;"/>
            <span class="empty-text-second">
              支持CSV、XLSX格式文件，
              <a
                href="#"
                @click.prevent="handleEmptyAction('template')"
                style="color: #a2beff;"
              >
                下载
              </a>
              导入模板
            </span>
          </div>
        </template>
      </div>

      <VirtualScroller
        v-else
        :items="data"
        :itemSize="35"
        class="virtual-scroller"
      >
        <template #item="{ item, options }">
          <div
            class="table-row"
            :class="{
              selected: isRowSelected(item),
              'row-hovered': isRowHovered(item),
              clickable: true,
              'zebra-stripe': getItemIndex(item) % 2 === 1,
            }"
            v-memo="[getRowKey(item), isRowSelected(item), isRowHovered(item), item.exec_status, item.private_key, item.to_addr, item.amount, item.plat_balance, item.coin_balance, item.error_msg]"
            @click="handleRowClick(item, getItemIndex(item))"
          >
            <!-- 选择列 -->
            <div v-if="rowSelection" class="table-cell checkbox-cell">
              <input
                type="checkbox"
                :checked="isRowSelected(item)"
                @change="toggleRowSelection(item)"
                @click.stop
                class="checkbox"
              />
            </div>
            <!-- 普通列 -->
            <div
              v-for="column in sortedColumns"
              :key="column.dataIndex || column.slotName"
              class="table-cell"
              :class="{ 'copyable-cell': isCopyableColumn(column) }"
              :style="{
                ...getContentColumnStyle(column),
                textAlign: column.align || 'left',
              }"
              :title="getTooltipText(column, item)"
              @dblclick="handleCellDoubleClick($event, column, item)"
            >
              <!-- 可复制列使用Tooltip包装 -->
              <Tooltip
                v-if="isCopyableColumn(column)"
                content="双击可复制"
                position="top"
                :mouseEnterDelay="0.3"
                :mouseLeaveDelay="0.1"
              >
                <!-- 插槽内容 -->
                <template v-if="column.slotName">
                  <!-- 自动处理序号列 -->
                  <span v-if="column.slotName === 'index'">
                    {{ getItemIndex(item) + 1 }}
                  </span>
                  <!-- 其他插槽内容 -->
                  <slot
                    v-else
                    :name="column.slotName"
                    :record="item"
                    :rowIndex="getItemIndex(item)"
                  >
                    {{ item[column.dataIndex] }}
                  </slot>
                </template>
                <!-- 普通内容 -->
                <span v-else :class="{ ellipsis: column.ellipsis }">
                  {{ getDisplayText(column, item) }}
                </span>
              </Tooltip>
              <!-- 非可复制列正常显示 -->
              <template v-else>
                <!-- 插槽内容 -->
                <template v-if="column.slotName">
                  <!-- 自动处理序号列 -->
                  <span v-if="column.slotName === 'index'">
                    {{ getItemIndex(item) + 1 }}
                  </span>
                  <!-- 其他插槽内容 -->
                  <slot
                    v-else
                    :name="column.slotName"
                    :record="item"
                    :rowIndex="getItemIndex(item)"
                  >
                    {{ item[column.dataIndex] }}
                  </slot>
                </template>
                <!-- 普通内容 -->
                <span v-else :class="{ ellipsis: column.ellipsis }">
                  {{ getDisplayText(column, item) }}
                </span>
              </template>
            </div>
          </div>
        </template>
      </VirtualScroller>
    </div>

    <!-- Loading 遮罩 -->
    <div v-if="loading" class="loading-overlay">
      <div class="loading-spinner"></div>
      <div class="loading-text">加载中...</div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, shallowRef, watch, onMounted, onUnmounted } from "vue";
import VirtualScroller from "primevue/virtualscroller";
import { Icon } from "@iconify/vue";
import { Message, Tooltip } from "@arco-design/web-vue";

// Props
const props = defineProps({
  columns: {
    type: Array,
    required: true,
  },
  data: {
    type: Array,
    default: () => [],
  },
  rowKey: {
    type: String,
    default: "id",
  },
  rowSelection: {
    type: Object,
    default: null,
  },
  selectedKeys: {
    type: Array,
    default: () => [],
  },
  loading: {
    type: Boolean,
    default: false,
  },
  height: {
    type: String,
    default: "100%",
  },
  hoverKeys: {
    type: Array,
    default: () => [],
  },
  pageType: {
    type: String,
    default: "transfer", // transfer | balance | monitor
  },
  emptyData: {
    type: Boolean,
    default: null, // null 表示自动根据 data.length 判断
  },
});

// Emits
const emit = defineEmits(["row-click", "update:selectedKeys", "open-manual-import", "open-file-upload", "download-template"]);

// 处理空数据页面按钮点击
const handleEmptyAction = (type) => {
  switch (type) {
    case 'manual':
      emit('open-manual-import');
      break;
    case 'upload':
      emit('open-file-upload');
      break;
    case 'template':
      emit('download-template');
      break;
  }
};

// 计算属性
const isAllSelected = computed(() => {
  if (!props.rowSelection || props.data.length === 0) return false;
  return props.data.every((item) => props.selectedKeys.includes(getRowKey(item)));
});

const isIndeterminate = computed(() => {
  if (!props.rowSelection || props.data.length === 0) return false;
  const selectedCount = props.data.filter((item) =>
    props.selectedKeys.includes(getRowKey(item))
  ).length;
  return selectedCount > 0 && selectedCount < props.data.length;
});

// 缓存列排序结果，避免重复计算
const sortedColumns = shallowRef([]);

// 监听 columns 变化时更新 sortedColumns
watch(() => props.columns, (newCols) => {
  if (!newCols || newCols.length === 0) {
    sortedColumns.value = [];
    return;
  }
  const regular = newCols.filter((col) => col.slotName !== "optional");
  const optional = newCols.filter((col) => col.slotName === "optional");
  sortedColumns.value = [...regular, ...optional];
}, { immediate: true });

// 缓存行索引映射，O(1) 复杂度获取索引
const itemIndexMap = computed(() => {
  const map = new Map();
  const data = props.data;
  for (let i = 0; i < data.length; i++) {
    map.set(getRowKey(data[i]), i);
  }
  return map;
});

// 方法
const getRowKey = (item) => {
  return item[props.rowKey];
};

const getItemIndex = (item) => {
  return itemIndexMap.value.get(getRowKey(item)) ?? -1;
};

// 判断是否显示空数据提示
const showEmptyData = computed(() => {
  if (props.emptyData !== null) {
    return props.emptyData;
  }
  return !props.loading && props.data.length === 0;
});

// 检测是否会产生滚动条
const hasScrollbar = computed(() => {
  if (!props.data.length || !props.height) return false;

  // 计算容器高度（减去表头高度40px）
  const containerHeight = props.height === "100%" ? 400 : parseInt(props.height) - 40;

  // 计算内容总高度（数据项数量 * 行高35px）
  const contentHeight = props.data.length * 35;

  return contentHeight > containerHeight;
});

// 计算表头列宽度（保持原始设置不变）
const getHeaderColumnStyle = (column) => {
  if (column.width) {
    return { width: column.width + "px", flexShrink: 0 };
  }
  // 没有设置宽度的列使用flex: 1来占满剩余空间
  return { flex: 1, minWidth: "100px" };
};

// 计算虚拟滚动内容列宽度（只在optional列检测到滚动条时调整）
const getContentColumnStyle = (column) => {
  if (column.width) {
    let width = column.width;
    // 如果是optional列且有滚动条，减少10px宽度来补偿滚动条占用的空间
    if (column.slotName === "optional" && hasScrollbar.value) {
      width = Math.max(width - 15, 30); // 最小宽度30px
    }
    return { width: width + "px", flexShrink: 0 };
  }
  // 没有设置宽度的列使用flex: 1来占满剩余空间
  return { flex: 1, minWidth: "100px" };
};

// 方法
const isRowSelected = (item) => {
  if (!props.rowSelection) return false;
  return props.selectedKeys.includes(getRowKey(item));
};

const isRowHovered = (item) => {
  return props.hoverKeys.includes(getRowKey(item));
};

const handleRowClick = (item, index) => {
  emit("row-click", item, { index });
};

const toggleRowSelection = (item) => {
  if (!props.rowSelection) return;

  const key = getRowKey(item);
  const newSelectedKeys = [...props.selectedKeys];
  const index = newSelectedKeys.indexOf(key);

  if (index >= 0) {
    newSelectedKeys.splice(index, 1);
  } else {
    newSelectedKeys.push(key);
  }

  emit("update:selectedKeys", newSelectedKeys);
};

const toggleSelectAll = () => {
  if (!props.rowSelection) return;

  let newSelectedKeys;
  if (isAllSelected.value) {
    // 取消全选
    const currentPageKeys = props.data.map((item) => getRowKey(item));
    newSelectedKeys = props.selectedKeys.filter((key) => !currentPageKeys.includes(key));
  } else {
    // 全选
    const currentPageKeys = props.data.map((item) => getRowKey(item));
    newSelectedKeys = [...new Set([...props.selectedKeys, ...currentPageKeys])];
  }

  emit("update:selectedKeys", newSelectedKeys);
};

// 获取显示文本
const getDisplayText = (column, item) => {
  const value = item[column.dataIndex];
  if (!value) return "";

  // 特殊处理error_msg字段，只显示前20个字符
  if (column.dataIndex === "error_msg" && value.length > 25) {
    return value.substring(0, 25) + "...";
  }

  return value;
};

// 判断是否为最后一行
const isLastRow = (item) => {
  const index = getItemIndex(item);
  return index === props.data.length - 1;
};

// 获取tooltip文本
const getTooltipText = (column, item) => {
  // 最后一行总是显示tooltip
  if (isLastRow(item)) {
    const value = item[column.dataIndex];
    if (value) return value;
  }

  const value = item[column.dataIndex];
  if (!value) return "";

  // 如果设置了ellipsis或者是error_msg字段，显示完整内容作为tooltip
  if (column.ellipsis || column.dataIndex === "error_msg") {
    return value;
  }

  // 如果设置了tooltip属性且不是状态列，显示完整内容
  // 状态列使用自定义的a-tooltip，不需要原生tooltip
  if (column.tooltip && column.slotName !== "exec_status") {
    return value;
  }

  return "";
};

// 检查是否为可复制的列
const isCopyableColumn = (column) => {
  const copyableColumns = ["private_key", "address", "to_addr", "error_msg"];
  return copyableColumns.includes(column.dataIndex);
};

// 处理单元格双击事件
const handleCellDoubleClick = async (event, column, item) => {
  // 阻止事件冒泡和默认行为，避免触发行选择
  event.stopPropagation();
  event.preventDefault();

  if (!isCopyableColumn(column)) return;

  const value = item[column.dataIndex];
  if (!value) return;

  try {
    await navigator.clipboard.writeText(value);
    Message.success({
      content: "已复制",
      position: "top",
      offset: 500,
    });
  } catch (error) {
    console.error("复制失败:", error);
    Message.error({
      content: "复制失败",
      position: "top",
      offset: 500,
    });
  }
};

// 处理滚轮事件，滚动时隐藏所有tooltip
const handleWheel = () => {
  // 通过document.querySelector隐藏arco-tooltip-popup和arco-tooltip
  const tooltips = document.querySelectorAll('.arco-tooltip-popup, .arco-tooltip');
  tooltips.forEach((tooltip) => {
    tooltip.style.display = 'none';
  });
};
</script>

<style scoped>
.virtual-scroller-table {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--table-border-color, #e5e6eb);
  border-radius: 6px;
  background: var(--table-bg, #ffffff);
  overflow: hidden;
  width: 100%;
}

.table-header {
  background: var(--table-header-bg, #f7f8fa);
  border-bottom: 1px solid var(--table-border-color, #e5e6eb);
  flex-shrink: 0;
}

.header-row {
  display: flex;
  height: 40px;
  align-items: center;
  width: 100%;
}

.header-cell {
  padding: 0 12px;
  font-weight: 600;
  color: var(--table-header-text-color, #1d2129);
  border-right: 1px solid var(--table-border-color, #e5e6eb);
  display: flex;
  align-items: center;
}

.header-cell:last-child {
  border-right: none;
}

.table-body {
  flex: 1;
  overflow: hidden;
  background: var(--table-bg, #ffffff);
  width: 100%;
}

.virtual-scroller {
  width: 100%;
  height: calc(100% - 30px);
  border-bottom: 1px solid var(--table-border-color);
}

.table-row {
  display: flex;
  height: 35px;
  align-items: center;
  border-bottom: 1px solid var(--table-border-color, #f2f3f5);
  transition: background-color 0.2s;
  background: var(--table-bg, #ffffff);
  width: 100%;
}

.table-row.zebra-stripe {
  background-color: var(--table-zebra-bg, #fafafa);
}

.table-row:hover {
  background-color: var(--table-hover-bg, #fff9e6) !important;
}

.table-row.row-hovered {
  background-color: var(--table-hover-bg, #fff9e6) !important;
}

/* 暗黑主题 */
:root[data-theme="dark"] .table-row:hover,
:root[data-theme="dark"] .table-row.row-hovered {
  background-color: var(--table-hover-bg, #4a4520) !important;
}

.table-row.clickable {
  cursor: pointer;
}

.table-cell {
  padding: 0 12px;
  color: var(--table-text-color, #1d2129);
  border-right: 1px solid var(--table-border-color, #f2f3f5);
  display: flex;
  align-items: center;
  min-height: 35px;
}

.table-cell:last-child {
  border-right: none;
}

.copyable-cell {
  cursor: copy;
  position: relative;
}

.checkbox-cell {
  width: 50px;
  justify-content: center;
}

.checkbox {
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.ellipsis {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.8);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  z-index: 10;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f3f3f3;
  border-top: 3px solid #165dff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.loading-text {
  margin-top: 12px;
  color: #86909c;
  font-size: 14px;
}

.empty-data {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  min-height: 450px;
  color: #888888ff;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.6;
}

.empty-text {
  font-size: 14px;
}

.empty-text-second {
  font-size: 12px;
  color: rgba(134, 144, 156, 0.41);
}
</style>
