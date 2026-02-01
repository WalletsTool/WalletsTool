<script setup name="app">
import { onMounted, ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'

const CHECK_INTERVAL_MS = 12 * 60 * 60 * 1000
const STORAGE_KEYS = {
  lastCheckAt: 'walletstool:update:lastCheckAt',
  ignoreVersion: 'walletstool:update:ignoreVersion'
}

const updateVisible = ref(false)
const updateInfo = ref(null)

const shouldCheckUpdate = () => {
  const lastCheckAtRaw = localStorage.getItem(STORAGE_KEYS.lastCheckAt)
  const lastCheckAt = lastCheckAtRaw ? Number(lastCheckAtRaw) : 0
  if (!Number.isFinite(lastCheckAt)) return true
  return Date.now() - lastCheckAt >= CHECK_INTERVAL_MS
}

const setLastCheckNow = () => {
  localStorage.setItem(STORAGE_KEYS.lastCheckAt, String(Date.now()))
}

const getIgnoredVersion = () => localStorage.getItem(STORAGE_KEYS.ignoreVersion) || ''

const ignoreThisVersion = () => {
  if (!updateInfo.value?.latest_version) return
  localStorage.setItem(STORAGE_KEYS.ignoreVersion, updateInfo.value.latest_version)
  updateVisible.value = false
}

const openReleasePage = async () => {
  const url = updateInfo.value?.html_url
  if (!url) return

  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (isTauri) {
    await import('@tauri-apps/plugin-shell').then(({ open }) => open(url))
    return
  }

  window.open(url, '_blank')
}

const checkForUpdate = async (force = false) => {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (!isTauri) return

  if (!force && !shouldCheckUpdate()) return
  setLastCheckNow()

  try {
    const currentVersion = await getVersion()
    const result = await invoke('check_github_release_update', {
      owner: 'WalletsTool',
      repo: 'WalletsTool',
      currentVersion
    })

    if (!result) return

    const ignored = getIgnoredVersion()
    if (ignored && result.latest_version === ignored) return

    updateInfo.value = result
    updateVisible.value = true
  } catch (e) {
    console.error('检查更新失败:', e)
  }
}

onMounted(() => {
  checkForUpdate(false)
})
</script>

<template>
  <Suspense>
    <router-view></router-view>
  </Suspense>

  <a-modal v-model:visible="updateVisible" :width="720" :mask-closable="false" title="发现新版本" unmountOnClose>
    <div v-if="updateInfo" style="display: flex; flex-direction: column; gap: 10px;">
      <div>
        <div style="font-weight: 600;">当前版本：{{ updateInfo.current_version }}</div>
        <div style="font-weight: 600;">最新版本：{{ updateInfo.latest_version }}</div>
      </div>
      <div v-if="updateInfo.name" style="opacity: 0.9;">{{ updateInfo.name }}</div>
      <div v-if="updateInfo.published_at" style="opacity: 0.7;">发布时间：{{ updateInfo.published_at }}</div>
      <div v-if="updateInfo.body" style="max-height: 320px; overflow: auto; white-space: pre-wrap; background: rgba(0,0,0,0.04); padding: 10px; border-radius: 6px;">
        {{ updateInfo.body }}
      </div>
    </div>

    <template #footer>
      <a-button @click="updateVisible = false">稍后提醒</a-button>
      <a-button @click="ignoreThisVersion">忽略此版本</a-button>
      <a-button type="primary" @click="openReleasePage">前往下载</a-button>
    </template>
  </a-modal>
</template>

<style scoped>
</style>
