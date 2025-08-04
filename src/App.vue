<script setup name="app">
import { onMounted, onBeforeMount } from 'vue'

onBeforeMount(() => {
  console.log('[DEBUG] App.vue - onBeforeMount 开始')
})

onMounted(() => {
  console.log('[DEBUG] App.vue - onMounted 完成')
  console.log('[DEBUG] App.vue - DOM元素:', document.getElementById('app'))
})
</script>

<template>
  <Suspense>
    <router-view></router-view>
  </Suspense>
</template>

<style scoped>
</style>
