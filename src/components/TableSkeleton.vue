<template>
  <div class="table-skeleton">
    <!-- 表格头部骨架 -->
    <div class="skeleton-header">
      <div class="skeleton-row">
        <div class="skeleton-cell skeleton-checkbox"></div>
        <div class="skeleton-cell skeleton-index"></div>
        <div class="skeleton-cell skeleton-address"></div>
        <div class="skeleton-cell skeleton-balance"></div>
        <div class="skeleton-cell skeleton-balance"></div>
        <div class="skeleton-cell skeleton-status"></div>
        <div class="skeleton-cell skeleton-action"></div>
      </div>
    </div>
    
    <!-- 表格内容骨架 -->
    <div class="skeleton-body">
      <div v-for="i in rows" :key="i" class="skeleton-row">
        <div class="skeleton-cell skeleton-checkbox">
          <div class="skeleton-item skeleton-round"></div>
        </div>
        <div class="skeleton-cell skeleton-index">
          <div class="skeleton-item skeleton-text-short"></div>
        </div>
        <div class="skeleton-cell skeleton-address">
          <div class="skeleton-item skeleton-text-long"></div>
        </div>
        <div class="skeleton-cell skeleton-balance">
          <div class="skeleton-item skeleton-text-medium"></div>
        </div>
        <div class="skeleton-cell skeleton-balance">
          <div class="skeleton-item skeleton-text-medium"></div>
        </div>
        <div class="skeleton-cell skeleton-status">
          <div class="skeleton-item skeleton-tag"></div>
        </div>
        <div class="skeleton-cell skeleton-action">
          <div class="skeleton-item skeleton-button"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  rows: {
    type: Number,
    default: 8
  }
})
</script>

<style scoped>
.table-skeleton {
  width: 100%;
  background: #fff;
  border-radius: 6px;
  overflow: hidden;
}

.skeleton-header {
  background: #f7f8fa;
  border-bottom: 1px solid #e5e6eb;
}

.skeleton-body {
  background: #fff;
}

.skeleton-row {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #f2f3f5;
}

.skeleton-row:last-child {
  border-bottom: none;
}

.skeleton-cell {
  display: flex;
  align-items: center;
  justify-content: center;
}

.skeleton-checkbox {
  width: 40px;
  min-width: 40px;
}

.skeleton-index {
  width: 60px;
  min-width: 60px;
}

.skeleton-address {
  flex: 1;
  min-width: 200px;
  justify-content: flex-start;
  padding-left: 8px;
}

.skeleton-balance {
  width: 120px;
  min-width: 120px;
}

.skeleton-status {
  width: 100px;
  min-width: 100px;
}

.skeleton-action {
  width: 80px;
  min-width: 80px;
}

.skeleton-item {
  background: linear-gradient(90deg, #f2f3f5 25%, #e5e6eb 50%, #f2f3f5 75%);
  background-size: 200% 100%;
  animation: skeleton-loading 1.5s infinite;
  border-radius: 4px;
}

.skeleton-text-short {
  width: 30px;
  height: 16px;
}

.skeleton-text-medium {
  width: 80px;
  height: 16px;
}

.skeleton-text-long {
  width: 180px;
  height: 16px;
}

.skeleton-round {
  width: 16px;
  height: 16px;
  border-radius: 50%;
}

.skeleton-tag {
  width: 60px;
  height: 24px;
  border-radius: 12px;
}

.skeleton-button {
  width: 32px;
  height: 32px;
  border-radius: 4px;
}

@keyframes skeleton-loading {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

/* 深色主题适配 */
:root[data-theme="dark"] .table-skeleton {
  background: #232324;
}

:root[data-theme="dark"] .skeleton-header {
  background: #2a2a2b;
  border-bottom-color: #3a3a3b;
}

:root[data-theme="dark"] .skeleton-body {
  background: #232324;
}

:root[data-theme="dark"] .skeleton-row {
  border-bottom-color: #3a3a3b;
}

:root[data-theme="dark"] .skeleton-item {
  background: linear-gradient(90deg, #3a3a3b 25%, #4a4a4b 50%, #3a3a3b 75%);
  background-size: 200% 100%;
}
</style>