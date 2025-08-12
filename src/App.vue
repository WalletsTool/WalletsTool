<script setup name="app">
import { onMounted, onBeforeMount } from 'vue'

onBeforeMount(() => {
})

onMounted(() => {
})
</script>

<template>
  <Suspense>
    <router-view></router-view>
  </Suspense>
</template>

<style scoped>
</style>
