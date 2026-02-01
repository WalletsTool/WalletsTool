<script setup name="app">
import { onMounted, ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'

const CHECK_INTERVAL_MS = 12 * 60 * 60 * 1000
const STORAGE_KEYS = {
  lastCheckAt: 'walletstool:update:lastCheckAt',
  ignoreVersion: 'walletstool:update:ignoreVersion',
  lastAttemptVersion: 'walletstool:update:lastAttemptVersion',
  lastAttemptAt: 'walletstool:update:lastAttemptAt'
}

const updateVisible = ref(false)
const updateInfo = ref(null)
const pendingTauriUpdate = ref(null)
const updateLoading = ref(false)

const normalizeVersion = (value) => String(value || '').trim().replace(/^v/i, '')

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

const remindLater = () => {
  setLastCheckNow()
  updateVisible.value = false
}

const ignoreThisVersion = () => {
  if (!updateInfo.value?.latest_version) return
  localStorage.setItem(STORAGE_KEYS.ignoreVersion, normalizeVersion(updateInfo.value.latest_version))
  setLastCheckNow()
  updateVisible.value = false
}

const buildReleasePageUrl = (version) => {
  const normalized = normalizeVersion(version)
  if (!normalized) return ''
  return `https://github.com/WalletsTool/WalletsTool/releases/tag/v${normalized}`
}

const getUpdatePublishedAt = (update) => {
  const value = update?.date || update?.pub_date || update?.published_at
  return value ? String(value) : ''
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

const fetchGithubReleaseUpdate = async () => {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (!isTauri) return null

  try {
    const currentVersion = await getVersion()
    const result = await invoke('check_github_release_update', {
      owner: 'WalletsTool',
      repo: 'WalletsTool',
      currentVersion
    })

    if (!result) return null
    return result
  } catch (e) {
    console.error('检查更新失败:', e)
    return null
  }
}

const showUpdateModal = (info, tauriUpdate = null) => {
  const ignored = normalizeVersion(getIgnoredVersion())
  if (ignored && normalizeVersion(info?.latest_version) === ignored) return

  pendingTauriUpdate.value = tauriUpdate
  updateInfo.value = info
  updateVisible.value = true
}

const startInAppUpdate = async () => {
  if (updateLoading.value) return
  const update = pendingTauriUpdate.value
  if (!update) {
    await openReleasePage()
    return
  }

  try {
    updateLoading.value = true
    const latestVersion = normalizeVersion(update.version)
    localStorage.setItem(STORAGE_KEYS.lastAttemptVersion, latestVersion)
    localStorage.setItem(STORAGE_KEYS.lastAttemptAt, String(Date.now()))
    await update.downloadAndInstall()
    await relaunch()
  } catch (e) {
    console.error('应用内更新失败:', e)
    pendingTauriUpdate.value = null
    if (updateInfo.value?.latest_version) {
      updateInfo.value = { ...updateInfo.value, html_url: buildReleasePageUrl(updateInfo.value.latest_version) }
    }
  } finally {
    updateLoading.value = false
  }
}

const checkUpdateOnLaunch = async (force = false) => {
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (!isTauri) return

  if (!force && !shouldCheckUpdate()) return
  let didFinishAnyCheck = false

  try {
    const currentVersion = await getVersion()
    const update = await check({ timeout: 8000 })
    didFinishAnyCheck = true

    if (!update) return
    showUpdateModal(
      {
        current_version: currentVersion,
        latest_version: normalizeVersion(update.version),
        html_url: buildReleasePageUrl(update.version),
        name: update?.title ? String(update.title) : '',
        body: update?.body ? String(update.body) : '',
        published_at: getUpdatePublishedAt(update),
        prerelease: false
      },
      update
    )
    return
  } catch (e) {
    console.error('Tauri Updater 检查失败，回退到 GitHub Release 检查:', e)
  }

  const release = await fetchGithubReleaseUpdate()
  didFinishAnyCheck = true
  if (release) {
    showUpdateModal(release, null)
  }

  if (didFinishAnyCheck) setLastCheckNow()
}

onMounted(() => {
  checkUpdateOnLaunch(false)
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
      <a-button :disabled="updateLoading" @click="remindLater">稍后提醒</a-button>
      <a-button :disabled="updateLoading" @click="ignoreThisVersion">忽略此版本</a-button>
      <a-button :loading="updateLoading" type="primary" @click="startInAppUpdate">立即更新</a-button>
      <a-button :disabled="updateLoading" @click="openReleasePage">前往下载</a-button>
    </template>
  </a-modal>
</template>

<style scoped>
</style>
