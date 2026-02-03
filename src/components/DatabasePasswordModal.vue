<script setup>
import { ref, watch, nextTick } from 'vue'
import { Message } from '@arco-design/web-vue'
import { useDatabaseStore } from '@/stores/database'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false,
  },
  mode: {
    type: String,
    default: 'setup',
    validator: (value) => ['setup', 'unlock'].includes(value),
  },
})

const emit = defineEmits(['update:visible', 'success'])

const databaseStore = useDatabaseStore()
const password = ref('')
const confirmPassword = ref('')
const loading = ref(false)
const error = ref('')
const passwordInputRef = ref(null)

watch(() => props.visible, (visible) => {
  if (visible) {
    error.value = ''
    password.value = ''
    confirmPassword.value = ''
    nextTick(() => {
      passwordInputRef.value?.focus()
    })
  }
})

const isSetupMode = () => props.mode === 'setup'

const validatePassword = () => {
  if (!password.value) {
    error.value = '请输入密码'
    return false
  }

  if (isSetupMode()) {
    if (password.value.length < 8) {
      error.value = '密码长度至少为8位'
      return false
    }

    if (password.value !== confirmPassword.value) {
      error.value = '两次输入的密码不一致'
      return false
    }
  }

  return true
}

const handleSubmit = async () => {
  if (!validatePassword()) return

  loading.value = true
  error.value = ''

  try {
    if (isSetupMode()) {
      await databaseStore.initSecureDatabase(password.value)
      Message.success('安全数据库初始化成功')
    } else {
      await databaseStore.unlockSecureDatabase(password.value)
      Message.success('解锁成功')
    }

    password.value = ''
    confirmPassword.value = ''
    emit('success')
    emit('update:visible', false)
  } catch (err) {
    error.value = err.message || '操作失败'
    Message.error(error.value)
  } finally {
    loading.value = false
  }
}

const handleCancel = () => {
  emit('update:visible', false)
}

const handlePasswordInput = () => {
  error.value = ''
}
</script>

<template>
  <a-modal
    :visible="visible"
    :mask-closable="false"
    :esc-to-close="false"
    unmount-on-close
    @cancel="handleCancel"
  >
    <template #title>
      <div class="password-modal-title">
        <icon-lock-fill class="title-icon" />
        <span>{{ mode === 'setup' ? '设置主密码' : '解锁安全数据库' }}</span>
      </div>
    </template>

    <div class="password-modal-content">
      <div class="info-text">
        <template v-if="mode === 'setup'">
          <p>首次使用需要设置主密码，用于加密您的钱包私钥和助记词。</p>
          <p>请务必牢记此密码，丢失后无法找回。</p>
        </template>
        <template v-else>
          <p>请输入主密码以解锁安全数据库。</p>
        </template>
      </div>

      <a-form :model="{ password }" layout="vertical">
        <a-form-item label="主密码">
          <a-input-password
            ref="passwordInputRef"
            v-model="password"
            :placeholder="mode === 'setup' ? '请输入主密码（至少8位）' : '请输入主密码'"
            :disabled="loading"
            @input="handlePasswordInput"
          />
        </a-form-item>

        <a-form-item v-if="mode === 'setup'" label="确认密码">
          <a-input-password
            v-model="confirmPassword"
            placeholder="请再次输入主密码"
            :disabled="loading"
            @input="handlePasswordInput"
          />
        </a-form-item>

        <div v-if="error" class="error-message">{{ error }}</div>
      </a-form>
    </div>

    <template #footer>
      <a-button @click="handleCancel" :disabled="loading">
        取消
      </a-button>
      <a-button type="primary" :loading="loading" @click="handleSubmit">
        {{ mode === 'setup' ? '设置密码' : '解锁' }}
      </a-button>
    </template>
  </a-modal>
</template>

<style scoped>
.password-modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
}

.title-icon {
  font-size: 20px;
  color: rgb(var(--primary-6));
}

.password-modal-content {
  padding: 8px 0;
}

.info-text {
  margin-bottom: 24px;
  padding: 12px;
  background: rgba(var(--primary-6), 0.08);
  border-radius: 6px;
  border-left: 3px solid rgb(var(--primary-6));
}

.info-text p {
  margin: 4px 0;
  line-height: 1.5;
  color: rgb(var(--text-2));
  font-size: 13px;
}

.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(var(--danger-6), 0.1);
  border-radius: 4px;
  color: rgb(var(--danger-6));
  font-size: 13px;
}
</style>
