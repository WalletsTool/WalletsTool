<script setup name="wallet-import-modal">
import { ref, reactive, computed, nextTick } from 'vue';
import { ethers } from 'ethers';
import CodeEditor from './CodeEditor.vue';

// Props
const props = defineProps({
  title: {
    type: String,
    default: '钱包信息录入'
  }
});

// 内部管理的visible状态
const visible = ref(false);

// Emits
const emit = defineEmits(['confirm', 'cancel']);

// 响应式数据
const privateKeyText = ref('');
const addressText = ref('');
const validationErrors = ref([]);
const errorsExpanded = ref(false);
const showUsageInstructions = ref(true);
const privateKeyErrorLines = ref([]);
const addressErrorLines = ref([]);
const importLoading = ref(false);

// 编辑器引用
const privateKeyEditorRef = ref(null);
const addressEditorRef = ref(null);
const isScrollSyncing = ref(false);
const isLineSelectionSyncing = ref(false);

// 计算属性：显示的错误信息
const displayedErrors = computed(() => {
  if (errorsExpanded.value || validationErrors.value.length <= 3) {
    return validationErrors.value;
  }
  return validationErrors.value.slice(0, 3);
});

// 验证私钥格式
function validatePrivateKey(privateKey) {
  try {
    // 移除前缀0x（如果有）
    const cleanKey = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey;
    // 检查是否为64位十六进制字符串
    if (!/^[0-9a-fA-F]{64}$/.test(cleanKey)) {
      return false;
    }
    // 尝试创建钱包实例验证私钥有效性
    new ethers.Wallet(privateKey);
    return true;
  } catch (error) {
    return false;
  }
}

// 验证地址格式
function validateAddress(address) {
  try {
    // 检查地址是否为空或undefined
    if (!address || typeof address !== 'string') {
      return false;
    }

    // 去除首尾空格
    const trimmedAddress = address.trim();

    // 检查是否以0x开头且长度为42
    if (!trimmedAddress.startsWith('0x') || trimmedAddress.length !== 42) {
      return false;
    }

    // 检查除0x外的部分是否为有效的十六进制字符
    const hexPart = trimmedAddress.slice(2);
    if (!/^[0-9a-fA-F]{40}$/.test(hexPart)) {
      return false;
    }

    // 使用ethers.js进行最终验证
    return ethers.utils.isAddress(trimmedAddress);
  } catch (error) {
    return false;
  }
}

// 验证导入数据
function validateImportData() {
  const privateKeys = privateKeyText.value.split('\n').filter(line => line.trim() !== '');
  const addresses = addressText.value.split('\n').filter(line => line.trim() !== '');
  
  validationErrors.value = [];
  const errorLines = new Set();
  
  // 检查行数是否匹配（只有当两个字段都有内容时才检查）
  if (privateKeys.length > 0 && addresses.length > 0 && privateKeys.length !== addresses.length) {
    validationErrors.value.push(`私钥行数(${privateKeys.length})与地址行数(${addresses.length})不匹配`);
  }
  
  // 验证私钥格式
  privateKeys.forEach((key, index) => {
    const trimmedKey = key.trim();
    if (trimmedKey && !validatePrivateKey(trimmedKey)) {
      validationErrors.value.push(`第${index + 1}行私钥格式错误`);
      errorLines.add(index + 1);
    }
  });
  
  // 验证地址格式
  addresses.forEach((addr, index) => {
    const trimmedAddr = addr.trim();
    if (trimmedAddr && !validateAddress(trimmedAddr)) {
      validationErrors.value.push(`第${index + 1}行地址格式错误`);
      errorLines.add(index + 1);
    }
  });
  
  // 同步错误行号到两个编辑器
  privateKeyErrorLines.value = Array.from(errorLines);
  addressErrorLines.value = Array.from(errorLines);
}

// 切换错误信息展开状态
function toggleErrorsExpanded() {
  errorsExpanded.value = !errorsExpanded.value;
}

// 处理私钥编辑器滚动事件
function handlePrivateKeyScroll(scrollInfo) {
  if (isScrollSyncing.value) return;
  
  isScrollSyncing.value = true;
  if (addressEditorRef.value && addressEditorRef.value.syncScroll) {
    addressEditorRef.value.syncScroll(scrollInfo);
  }
  nextTick(() => {
    isScrollSyncing.value = false;
  });
}

// 处理地址编辑器滚动事件
function handleAddressScroll(scrollInfo) {
  if (isScrollSyncing.value) return;
  
  isScrollSyncing.value = true;
  if (privateKeyEditorRef.value && privateKeyEditorRef.value.syncScroll) {
    privateKeyEditorRef.value.syncScroll(scrollInfo);
  }
  nextTick(() => {
    isScrollSyncing.value = false;
  });
}

// 处理私钥编辑器行号选择事件
function handlePrivateKeyLineSelect(lineSelectionInfo) {
  if (isLineSelectionSyncing.value) return;
  
  isLineSelectionSyncing.value = true;
  if (addressEditorRef.value && addressEditorRef.value.syncLineSelection) {
    addressEditorRef.value.syncLineSelection(lineSelectionInfo);
  }
  nextTick(() => {
    isLineSelectionSyncing.value = false;
  });
}

// 处理地址编辑器行号选择事件
function handleAddressLineSelect(lineSelectionInfo) {
  if (isLineSelectionSyncing.value) return;
  
  isLineSelectionSyncing.value = true;
  if (privateKeyEditorRef.value && privateKeyEditorRef.value.syncLineSelection) {
    privateKeyEditorRef.value.syncLineSelection(lineSelectionInfo);
  }
  nextTick(() => {
    isLineSelectionSyncing.value = false;
  });
}

// 处理弹窗取消
function handleCancel() {
  visible.value = false;
  emit('cancel');
  // 重置数据
  privateKeyText.value = '';
  addressText.value = '';
  validationErrors.value = [];
  errorsExpanded.value = false;
}

// 处理弹窗确认前的验证
const handleBeforeOk = async () => {
  // 验证数据
  validateImportData();
  
  if (validationErrors.value.length > 0) {
    return false;
  }
  
  importLoading.value = true;
  
  try {
    const privateKeys = privateKeyText.value.split('\n').filter(line => line.trim() !== '');
    const addresses = addressText.value.split('\n').filter(line => line.trim() !== '');
    
    // 发送确认事件，传递数据
    emit('confirm', {
      privateKeys: privateKeys.map(key => key.trim()),
      addresses: addresses.map(addr => addr.trim())
    });
    
    // 关闭弹窗
    visible.value = false;
    
    // 重置数据
    privateKeyText.value = '';
    addressText.value = '';
    validationErrors.value = [];
    errorsExpanded.value = false;
    
    return true;
  } catch (error) {
    console.error('处理导入数据失败:', error);
    return false;
  } finally {
    importLoading.value = false;
  }
};

// 显示弹窗的方法
function show() {
  // 重置数据
  privateKeyText.value = '';
  addressText.value = '';
  validationErrors.value = [];
  errorsExpanded.value = false;
  showUsageInstructions.value = true;
  
  // 显示弹窗
  visible.value = true;
}

// 暴露方法给父组件
defineExpose({
  show
});
</script>

<template>
  <a-modal 
    class="import-modal" 
    :visible="visible" 
    :width="1150" 
    :title="title" 
    @cancel="handleCancel"
    :on-before-ok="handleBeforeOk" 
    :confirm-loading="importLoading"
  >
    <div
      :style="{ display: 'flex', gap: '10px', marginTop: '10px', width: '1100px', height: showUsageInstructions ? '400px' : '500px' }">
      <!-- 左侧：私钥输入 -->
      <div style="width: 660px; height: 100%; display: flex; flex-direction: column;">
        <div class="input-label" style="margin-bottom: 8px; font-weight: 500;">发送方私钥</div>
        <CodeEditor 
          ref="privateKeyEditorRef"
          v-model="privateKeyText" 
          :error-lines="privateKeyErrorLines"
          placeholder="请输入私钥，一行一个&#10;格式：0x开头的64位十六进制字符串&#10;示例：0x1234567890abcdef..." 
          @input="validateImportData"
          @scroll="handlePrivateKeyScroll"
          @line-select="handlePrivateKeyLineSelect"
          style="flex: 1; height: 100%;" />
      </div>

      <!-- 右侧：接收地址输入 -->
      <div style="width: 430px; height: 100%; display: flex; flex-direction: column;">
        <div class="input-label" style="margin-bottom: 8px; font-weight: 500;">接收地址</div>
        <CodeEditor 
          ref="addressEditorRef"
          v-model="addressText" 
          :error-lines="addressErrorLines"
          placeholder="请输入接收地址，一行一个&#10;格式：0x开头的40位十六进制地址&#10;示例：0x742d35Cc6634C0532925a3b8D4..." 
          @input="validateImportData"
          @scroll="handleAddressScroll"
          @line-select="handleAddressLineSelect"
          style="flex: 1; height: 100%;" />
      </div>
    </div>

    <!-- 验证错误提示 -->
    <div v-if="validationErrors.length > 0" style="margin-top: 15px;">
      <a-alert style="padding: 5px 15px;" type="error" :title="`发现 ${validationErrors.length} 个问题`" :show-icon="true">
        <ul style="margin: 8px 0 0 0; padding-left: 20px;">
          <li v-for="(error, index) in displayedErrors" :key="error"
            style="margin-bottom: 4px; color: #f53f3f; font-size: 12px;">{{ error }}</li>
        </ul>
        <div v-if="validationErrors.length > 3" style="margin-top: 10px; text-align: center;">
          <a-button type="text" size="small" @click="toggleErrorsExpanded" style="color: #165dff;font-size: 12px;">
            {{ errorsExpanded ? '⬆️收起' : '⬇️展开全部' }}
          </a-button>
        </div>
      </a-alert>
    </div>

    <!-- 使用说明 -->
    <div v-if="showUsageInstructions" class="usage-instructions">
      <div class="usage-title" style="display: flex; justify-content: space-between; align-items: center;">
        <span>📋 使用说明：</span>
        <a-button type="text" size="small" @click="showUsageInstructions = false" style="color: #666; padding: 0;">
          ✕
        </a-button>
      </div>
      <div class="usage-list">
        <div class="usage-column">
          <ul>
            <li>私钥和接收地址必须一一对应，行数保持一致</li>
            <li>系统会自动验证格式，允许导入重复数据</li>
          </ul>
        </div>
        <div class="usage-column">
          <ul>
            <li>私钥格式：64位十六进制字符串（可选0x前缀）</li>
            <li>重复数据会在导入时给出提示信息</li>
          </ul>
        </div>
        <div class="usage-column">
          <ul>
            <li>地址格式：40位十六进制地址（必须0x前缀）</li>
          </ul>
        </div>
      </div>
    </div>
  </a-modal>
</template>

<style scoped>
.import-modal {
  /* 弹窗样式可以根据需要添加 */
}

.input-label {
  font-weight: 500;
  color: var(--text-color, #1d2129);
  margin-bottom: 8px;
}

.usage-instructions {
  margin-top: 15px;
  padding: 12px;
  background-color: var(--card-bg, #f7f8fa);
  border-radius: 6px;
  border: 1px solid var(--border-color, #e5e6eb);
}

.usage-title {
  font-weight: 500;
  color: var(--text-color, #1d2129);
  margin-bottom: 8px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.usage-list {
  display: flex;
  gap: 20px;
}

.usage-column {
  flex: 1;
}

.usage-column ul {
  margin: 0;
  padding-left: 16px;
  color: var(--text-color, #4e5969);
  font-size: 12px;
}

.usage-column li {
  margin-bottom: 4px;
  line-height: 1.4;
}
</style>