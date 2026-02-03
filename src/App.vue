<script setup name="app">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { useDatabaseStore } from '@/stores/database'

const databaseStore = useDatabaseStore()

const CHECK_INTERVAL_MS = 12 * 60 * 60 * 1000
const STORAGE_KEYS = {
  lastCheckAt: 'walletstool:update:lastCheckAt',
  ignoreVersion: 'walletstool:update:ignoreVersion',
  lastAttemptVersion: 'walletstool:update:lastAttemptVersion',
  lastAttemptAt: 'walletstool:update:lastAttemptAt',
  forceUpdatePopup: 'walletstool:update:forcePopup'
}

const updateVisible = ref(false)
const updateInfo = ref(null)
const pendingTauriUpdate = ref(null)
const updateLoading = ref(false)
const updateError = ref('')
const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 720)
let resizeHandler = null

const GH_PROXY_BASE_URL = 'https://gh-proxy.org/'

const updateModalWidth = computed(() => {
  const width = Math.floor(windowWidth.value * 0.92)
  return Math.min(520, Math.max(320, width))
})

const normalizeVersion = (value) => String(value || '').trim().replace(/^v/i, '')

const toGhProxyUrl = (value) => {
  const url = String(value || '').trim()
  if (!url) return ''
  if (url.startsWith(GH_PROXY_BASE_URL)) return url
  if (
    url.startsWith('https://github.com/') ||
    url.startsWith('https://api.github.com/') ||
    url.startsWith('https://raw.githubusercontent.com/')
  ) {
    return `${GH_PROXY_BASE_URL}${url}`
  }
  return url
}

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
const shouldForceUpdatePopup = () => {
  if (!import.meta.env.DEV) return false
  if (import.meta.env.VITE_FORCE_UPDATE_POPUP === '1') return true
  return localStorage.getItem(STORAGE_KEYS.forceUpdatePopup) === '1'
}

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
  return toGhProxyUrl(`https://github.com/WalletsTool/WalletsTool/releases/tag/v${normalized}`)
}

const getUpdatePublishedAt = (update) => {
  const value = update?.date || update?.pub_date || update?.published_at
  return value ? String(value) : ''
}

const openReleasePage = async () => {
  const url = toGhProxyUrl(updateInfo.value?.html_url)
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
  updateError.value = ''
  updateVisible.value = true
}

const startInAppUpdate = async () => {
  if (updateLoading.value) return
  updateError.value = ''

  try {
    updateLoading.value = true
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
    if (!isTauri) {
      throw new Error('当前环境不支持应用内更新')
    }

    const update = pendingTauriUpdate.value
    const latestVersion = normalizeVersion(update?.version || updateInfo.value?.latest_version || '')
    if (latestVersion) {
      localStorage.setItem(STORAGE_KEYS.lastAttemptVersion, latestVersion)
      localStorage.setItem(STORAGE_KEYS.lastAttemptAt, String(Date.now()))
    }

    if (!update) {
      await invoke('download_and_install_update')
      await relaunch()
      return
    }

    await update.downloadAndInstall()
    await relaunch()
  } catch (e) {
    console.error('应用内更新失败:', e)
    pendingTauriUpdate.value = null
    if (updateInfo.value?.latest_version) {
      updateInfo.value = { ...updateInfo.value, html_url: buildReleasePageUrl(updateInfo.value.latest_version) }
    }
    updateError.value = typeof e === 'string' ? e : e?.message || '应用内更新失败'
  } finally {
    updateLoading.value = false
  }
}

const checkUpdateOnLaunch = async (force = false) => {
  // 只在 main 窗口显示更新弹窗
  const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__
  if (isTauri) {
    try {
      const currentWindow = getCurrentWindow()
      const windowLabel = await currentWindow.label
      if (windowLabel !== 'main') return
    } catch {
      return
    }
  }

  if (shouldForceUpdatePopup()) {
    showUpdateModal({
      current_version: '0.0.0-dev',
      latest_version: '9.9.9',
      html_url: buildReleasePageUrl('9.9.9'),
      name: '本地测试更新弹窗',
      body: '用于本地预览"发现新版本"弹窗样式。\n关闭该开关后恢复正常检查流程。',
      published_at: new Date().toISOString(),
      prerelease: false
    })
    return
  }

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

const initDatabase = async () => {
  await databaseStore.checkStatus()

  if (!databaseStore.publicReady) {
    await databaseStore.initPublicDatabase()
  }

  // 启动时不再自动提示设置安全数据库密码
  // 仅在访问钱包管理页面时才按需提示
}

onMounted(async () => {
  await initDatabase()
  checkUpdateOnLaunch(false)
  if (typeof window === 'undefined') return
  resizeHandler = () => {
    windowWidth.value = window.innerWidth
  }
  window.addEventListener('resize', resizeHandler, { passive: true })
})

onBeforeUnmount(() => {
  if (typeof window === 'undefined') return
  if (!resizeHandler) return
  window.removeEventListener('resize', resizeHandler)
  resizeHandler = null
})
</script>

<template>
  <Suspense>
    <router-view></router-view>
  </Suspense>

  <a-modal v-model:visible="updateVisible" :width="updateModalWidth" :mask-closable="false" title="发现新版本" unmountOnClose>
    <div v-if="updateInfo" class="update-modal">
      <div class="update-modal__versions">
        <div class="update-modal__version-row">
          <span class="update-modal__label">当前</span>
          <span class="update-modal__value">{{ updateInfo.current_version }}</span>
        </div>
        <div class="update-modal__version-row update-modal__version-row--latest">
          <span class="update-modal__label">最新</span>
          <span class="update-modal__value">{{ updateInfo.latest_version }}</span>
        </div>
      </div>
      <div v-if="updateInfo.name" class="update-modal__name">{{ updateInfo.name }}</div>
      <div v-if="updateInfo.published_at" class="update-modal__meta">发布时间：{{ updateInfo.published_at }}</div>
      <div v-if="updateInfo.body" class="update-modal__body">
        {{ updateInfo.body }}
      </div>
      <div v-if="updateError" class="update-modal__error">{{ updateError }}</div>
      <a-link v-if="updateInfo.html_url" class="update-modal__link" @click="openReleasePage">查看更新详情</a-link>
    </div>

    <template #footer>
      <div class="update-modal__footer">
        <div class="update-modal__footer-left">
          <a-button :disabled="updateLoading" type="secondary" @click="ignoreThisVersion">忽略此版本</a-button>
        </div>
        <div class="update-modal__footer-right">
          <a-button :disabled="updateLoading" @click="remindLater">稍后提醒</a-button>
          <a-button :loading="updateLoading" type="primary" @click="startInAppUpdate">立即更新</a-button>
        </div>
      </div>
    </template>
  </a-modal>
</template>

<style scoped>
.update-modal {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.update-modal__versions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.update-modal__version-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.update-modal__version-row--latest .update-modal__label {
  color: rgb(var(--primary-6));
}

.update-modal__label {
  font-weight: 600;
  opacity: 0.75;
  white-space: nowrap;
}

.update-modal__value {
  font-weight: 600;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.update-modal__name {
  opacity: 0.9;
  line-height: 1.4;
}

.update-modal__meta {
  opacity: 0.7;
  font-size: 12px;
}

.update-modal__body {
  max-height: 240px;
  overflow: auto;
  white-space: pre-wrap;
  background: rgba(0, 0, 0, 0.04);
  padding: 10px;
  border-radius: 6px;
  line-height: 1.5;
  font-size: 13px;
}

.update-modal__link {
  align-self: flex-start;
}

.update-modal__error {
  color: rgb(var(--danger-6));
  font-size: 12px;
  line-height: 1.4;
  word-break: break-word;
}

.update-modal__footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.update-modal__footer-left {
  display: flex;
  gap: 8px;
}

.update-modal__footer-right {
  display: flex;
  gap: 8px;
}

@media (max-width: 480px) {
  .update-modal__footer {
    flex-direction: column-reverse;
    align-items: stretch;
  }

  .update-modal__footer-left,
  .update-modal__footer-right {
    justify-content: center;
  }
}

@media (max-width: 420px) {
  .update-modal__versions {
    flex-direction: column;
    align-items: flex-start;
  }

  .update-modal__body {
    max-height: 200px;
  }
}
</style>
