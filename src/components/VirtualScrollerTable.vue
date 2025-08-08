<template>
  <div class="virtual-scroller-table" :style="{ height: height }">
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
    <div class="table-body" :style="{ height: `calc(${height} - 40px)` }">
      <!-- 空数据提示 -->
      <div v-if="!loading && data.length === 0" class="empty-data">
        <div class="empty-icon">📋</div>
        <div class="empty-text">暂无数据</div>
      </div>
      
      <VirtualScroller
        v-else
        :items="data"
        :itemSize="35"
        class="virtual-scroller"
        :style="{ height: '100%' }"
      >
        <template #item="{ item, options }">
          <div
            class="table-row"
            :class="{ 'selected': isRowSelected(item), 'clickable': true }"
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
              :style="{ ...getContentColumnStyle(column), textAlign: column.align || 'left' }"
              :title="getTooltipText(column, item)"
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
              <span v-else :class="{ 'ellipsis': column.ellipsis }">
                {{ getDisplayText(column, item) }}
              </span>
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
import { computed, ref, watch } from 'vue'
import VirtualScroller from 'primevue/virtualscroller'

// Props
const props = defineProps({
  columns: {
    type: Array,
    required: true
  },
  data: {
    type: Array,
    default: () => []
  },
  rowKey: {
    type: String,
    default: 'id'
  },
  rowSelection: {
    type: Object,
    default: null
  },
  selectedKeys: {
    type: Array,
    default: () => []
  },
  loading: {
    type: Boolean,
    default: false
  },
  height: {
    type: String,
    default: '100%'
  }
})

// Emits
const emit = defineEmits(['row-click', 'update:selectedKeys'])

// 计算属性
const isAllSelected = computed(() => {
  if (!props.rowSelection || props.data.length === 0) return false
  return props.data.every(item => props.selectedKeys.includes(getRowKey(item)))
})

const isIndeterminate = computed(() => {
  if (!props.rowSelection || props.data.length === 0) return false
  const selectedCount = props.data.filter(item => props.selectedKeys.includes(getRowKey(item))).length
  return selectedCount > 0 && selectedCount < props.data.length
})

// 重新排序列，将optional列放在最右端
const sortedColumns = computed(() => {
  const regularColumns = props.columns.filter(col => col.slotName !== 'optional')
  const optionalColumns = props.columns.filter(col => col.slotName === 'optional')
  return [...regularColumns, ...optionalColumns]
})

// 检测是否会产生滚动条
const hasScrollbar = computed(() => {
  if (!props.data.length || !props.height) return false
  
  // 计算容器高度（减去表头高度40px）
  const containerHeight = props.height === '100%' ? 400 : parseInt(props.height) - 40
  
  // 计算内容总高度（数据项数量 * 行高35px）
  const contentHeight = props.data.length * 35
  
  return contentHeight > containerHeight
})

// 计算表头列宽度（保持原始设置不变）
const getHeaderColumnStyle = (column) => {
  if (column.width) {
    return { width: column.width + 'px', flexShrink: 0 }
  }
  // 没有设置宽度的列使用flex: 1来占满剩余空间
  return { flex: 1, minWidth: '100px' }
}

// 计算虚拟滚动内容列宽度（只在optional列检测到滚动条时调整）
const getContentColumnStyle = (column) => {
  if (column.width) {
    let width = column.width
    // 如果是optional列且有滚动条，减少10px宽度来补偿滚动条占用的空间
    if (column.slotName === 'optional' && hasScrollbar.value) {
      width = Math.max(width - 15, 30) // 最小宽度30px
    }
    return { width: width + 'px', flexShrink: 0 }
  }
  // 没有设置宽度的列使用flex: 1来占满剩余空间
  return { flex: 1, minWidth: '100px' }
}

// 方法
const getRowKey = (item) => {
  return item[props.rowKey]
}

const getItemIndex = (item) => {
  return props.data.findIndex(dataItem => getRowKey(dataItem) === getRowKey(item))
}

const isRowSelected = (item) => {
  if (!props.rowSelection) return false
  return props.selectedKeys.includes(getRowKey(item))
}

const handleRowClick = (item, index) => {
  emit('row-click', item, { index })
}

const toggleRowSelection = (item) => {
  if (!props.rowSelection) return
  
  const key = getRowKey(item)
  const newSelectedKeys = [...props.selectedKeys]
  const index = newSelectedKeys.indexOf(key)
  
  if (index >= 0) {
    newSelectedKeys.splice(index, 1)
  } else {
    newSelectedKeys.push(key)
  }
  
  emit('update:selectedKeys', newSelectedKeys)
}

const toggleSelectAll = () => {
  if (!props.rowSelection) return
  
  let newSelectedKeys
  if (isAllSelected.value) {
    // 取消全选
    const currentPageKeys = props.data.map(item => getRowKey(item))
    newSelectedKeys = props.selectedKeys.filter(key => !currentPageKeys.includes(key))
  } else {
    // 全选
    const currentPageKeys = props.data.map(item => getRowKey(item))
    newSelectedKeys = [...new Set([...props.selectedKeys, ...currentPageKeys])]
  }
  
  emit('update:selectedKeys', newSelectedKeys)
}

// 获取显示文本
const getDisplayText = (column, item) => {
  const value = item[column.dataIndex]
  if (!value) return ''
  
  // 特殊处理error_msg字段，只显示前20个字符
  if (column.dataIndex === 'error_msg' && value.length > 20) {
    return value.substring(0, 20) + '...'
  }
  
  return value
}

// 获取tooltip文本
const getTooltipText = (column, item) => {
  const value = item[column.dataIndex]
  if (!value) return ''
  
  // 如果设置了ellipsis或者是error_msg字段，显示完整内容作为tooltip
  if (column.ellipsis || column.dataIndex === 'error_msg') {
    return value
  }
  
  // 如果设置了tooltip属性，显示完整内容
  if (column.tooltip) {
    return value
  }
  
  return ''
}
</script>

<style scoped>
.virtual-scroller-table {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--table-border-color, #e5e6eb);
  border-radius: 6px;
  background: var(--table-bg, #ffffff);
  overflow: hidden;
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
}

.virtual-scroller {
  width: 100%;
}

.table-row {
  display: flex;
  height: 35px;
  align-items: center;
  border-bottom: 1px solid var(--table-border-color, #f2f3f5);
  transition: background-color 0.2s;
  background: var(--table-bg, #ffffff);
}

.table-row:hover {
  background-color: var(--table-hover-bg, #f7f8fa);
}

.table-row.selected {
  background-color: var(--table-selected-bg, #e8f4ff);
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
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
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
  height: 200px;
  color: #86909c;
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.6;
}

.empty-text {
  font-size: 14px;
}

/* 滚动条样式 */
:deep(.p-virtualscroller-content) {
  scrollbar-width: thin;
  scrollbar-color: #c9cdd4 #f7f8fa;
}

:deep(.p-virtualscroller-content::-webkit-scrollbar) {
  width: 8px;
}

:deep(.p-virtualscroller-content::-webkit-scrollbar-track) {
  background: #f7f8fa;
}

:deep(.p-virtualscroller-content::-webkit-scrollbar-thumb) {
  background: #c9cdd4;
  border-radius: 4px;
}

:deep(.p-virtualscroller-content::-webkit-scrollbar-thumb:hover) {
  background: #a9aeb8;
}
</style>