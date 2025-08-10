<script setup name="balance-solana">
import TitleBar from '@/components/TitleBar.vue'
import { ref } from 'vue'

const windowTitle = ref('Solana 余额查询（占位）')
</script>

<template>
  <div>
    <TitleBar :title="windowTitle" />
    <div style="padding:16px;">
      即将支持 Solana 生态的余额查询，这里是占位页面。
    </div>
  </div>
</template>

<style scoped>
</style>

