<script setup>
import { ref, onMounted, reactive, watch, nextTick, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Message, Notification, Modal } from '@arco-design/web-vue';
import { IconPlus, IconDelete, IconEdit, IconDownload, IconLock, IconFolder, IconFile } from '@arco-design/web-vue/es/icon';
import { useRouter } from 'vue-router';
import * as XLSX from 'xlsx';
import TitleBar from '@/components/TitleBar.vue';
import VirtualScrollerTable from '@/components/VirtualScrollerTable.vue';
import SecretRevealModal from '@/components/SecretRevealModal.vue';
import { downloadWithDialog, openDirectory } from '@/utils/downloadWithDialog';
import { save } from '@tauri-apps/plugin-dialog';
import { openSealedSecret } from '@/utils/secretCrypto';

// 初始化加载遮罩
const isLoading = ref(true);

const showLoadingOverlay = () => {
  let overlay = document.getElementById('wallet-manager-loading-overlay');
  if (!overlay) {
    overlay = document.createElement('div');
    overlay.id = 'wallet-manager-loading-overlay';
    overlay.className = 'wallet-global-loading-overlay';
    overlay.innerHTML = `
      <div class="loading-spinner"></div>
      <p>正在加载...</p>
    `;
    document.body.appendChild(overlay);
  }
  overlay.style.display = 'flex';
};

const hideLoadingOverlay = () => {
  const overlay = document.getElementById('wallet-manager-loading-overlay');
  if (overlay) {
    overlay.style.display = 'none';
  }
};

onUnmounted(() => {
  hideLoadingOverlay();
});

const router = useRouter();
const windowTitle = ref('钱包管理');

const handleBeforeClose = async () => {
  // 关键操作进行中，不允许关闭
  if (isCriticalOperation.value || bulkGenerating.value || isImporting.value) {
    return;
  }
  try {
    const currentWindow = getCurrentWindow();
    await currentWindow.destroy();
  } catch (e) {
    console.error('关闭窗口失败:', e);
  }
};

const handleExit = async () => {
  // 关键操作进行中，不允许退出
  if (isCriticalOperation.value || bulkGenerating.value || isImporting.value) {
    return;
  }
  try {
    const currentWindow = getCurrentWindow();
    await currentWindow.destroy();
  } catch (e) {
    console.error('关闭窗口失败:', e);
  }
};

const isUnlocked = ref(false);
const showInitModal = ref(false);
const showUnlockModal = ref(false);
const passwordInput = ref('');
const passwordInputRef = ref(null);
const initPassword = ref('');
const initPasswordConfirm = ref('');
const initPasswordRef = ref(null);
const sessionPassword = ref('');
const transportToken = ref('');
const transportAesKey = ref(null);
let pageLoadedEmitted = false;

// Change Password Modal
const showChangePasswordModal = ref(false);
const changePasswordForm = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: ''
});
const changePasswordLoading = ref(false);
const changePasswordOldRef = ref(null);
const changePasswordNewRef = ref(null);
const changePasswordConfirmRef = ref(null);

// 是否正在进行关键操作（修改密码等）
const isCriticalOperation = ref(false);

// 监听修改密码弹窗显示，自动聚焦输入框
watch(showChangePasswordModal, (newVal) => {
  if (newVal) {
    nextTick(() => {
      if (changePasswordOldRef.value) {
        changePasswordOldRef.value.focus();
      }
    });
  } else {
    changePasswordForm.oldPassword = '';
    changePasswordForm.newPassword = '';
    changePasswordForm.confirmPassword = '';
  }
});

// 监听弹窗显示，自动聚焦输入框
watch(showUnlockModal, (newVal) => {
  if (newVal && passwordInputRef.value) {
    // 在下一个DOM更新周期聚焦，确保弹窗已渲染
    nextTick(() => {
      passwordInputRef.value.focus();
    });
  }
});

watch(showInitModal, (newVal) => {
  if (newVal && initPasswordRef.value) {
    nextTick(() => {
      initPasswordRef.value.focus();
    });
  }
});

// Data
const groups = ref([]);
const wallets = ref([]);
const selectedWalletIds = ref([]);
const selectedGroupKeys = ref([]);
const expandedKeys = ref([]);
const walletLoadSeq = ref(0);

// View type: 'full_wallet' or 'address_only'
const currentViewType = ref('full_wallet');

// Rename Group
const showRenameGroupModal = ref(false);
const renameGroupName = ref('');
const renamingGroupId = ref(null);
const renameGroupNameRef = ref(null);

// Export Modal
const showExportModal = ref(false);
const exportPassword = ref('');
const exportPasswordRef = ref(null);
const isExporting = ref(false);

// 监听导出弹窗显示，自动聚焦输入框，关闭时清空密码
watch(showExportModal, (newVal) => {
  if (newVal) {
    nextTick(() => {
      if (exportPasswordRef.value) {
        exportPasswordRef.value.focus();
      }
    });
  } else {
    exportPassword.value = '';
  }
});

// 监听重命名弹窗显示，自动聚焦并选中文字
watch(showRenameGroupModal, (newVal) => {
  if (newVal && renameGroupNameRef.value) {
    nextTick(() => {
      renameGroupNameRef.value.focus();
      renameGroupNameRef.value.select();
    });
  }
});

// Modals
const showAddGroupModal = ref(false);
const newGroupName = ref('');
const newGroupNameRef = ref(null);
const showAddWalletModal = ref(false);
const showEditWalletModal = ref(false);
const editingWallet = ref({ id: null, name: '', remark: '' });

// 监听新建分组弹窗显示，自动聚焦输入框
watch(showAddGroupModal, (newVal) => {
  if (newVal && newGroupNameRef.value) {
    // 在下一个DOM更新周期聚焦，确保弹窗已渲染
    nextTick(() => {
      newGroupNameRef.value.focus();
    });
  }
});
const newWallet = reactive({
  name: '',
  address: '',
  chain_type: 'evm',
  group_id: null,
  group_name: '',
  group_path: [],
  group_search: '',
  new_group_name: '',
  private_key: '',
  mnemonic: '',
  remark: ''
});

const addWalletActiveTab = ref('import');
const importKeyMode = ref('private_key');
const mnemonicWalletCount = ref(1);
const mnemonicStartIndex = ref(0);

const bulkMode = ref('same_mnemonic');
const bulkWordCount = ref(12);
const bulkWalletCount = ref(10);
const bulkStartIndex = ref(0);
const bulkGenerating = ref(false);
const bulkTotalCount = ref(0);
const bulkSealedMnemonic = ref('');
const bulkMnemonicMasked = ref('');

// ==================== Watch Address (Address Only) State ====================
const watchAddresses = ref([]);
const selectedWatchAddressIds = ref([]);
const watchAddressLoadSeq = ref(0);

// Watch address form state
const newWatchAddress = reactive({
  name_prefix: '',
  addresses_text: '',
  chain_type: 'evm',
  group_id: null,
  group_name: '',
  group_path: [],
  group_search: '',
  new_group_name: '',
  remark: ''
});

// Watch address table columns
const watchAddressColumns = [
  { title: '序号', align: 'center', width: 53, slotName: 'index' },
  { title: '名称', align: 'center', dataIndex: 'name', width: 120, ellipsis: true, tooltip: true },
  { title: '地址', align: 'center', dataIndex: 'address', width: 380, ellipsis: true, tooltip: true },
  { title: '链类型', align: 'center', slotName: 'chain_type', width: 70 },
  { title: '备注', align: 'center', dataIndex: 'remark', ellipsis: true, tooltip: true },
  { title: '操作', align: 'center', slotName: 'optional', width: 80, ellipsis: true, tooltip: true },
];

const watchAddressRowSelection = { type: 'checkbox' };

// Reset watch address form
const resetWatchAddressForm = () => {
  newWatchAddress.name_prefix = '';
  newWatchAddress.addresses_text = '';
  newWatchAddress.chain_type = 'evm';
  newWatchAddress.group_id = null;
  newWatchAddress.group_name = '';
  newWatchAddress.group_path = [];
  newWatchAddress.group_search = '';
  newWatchAddress.new_group_name = '';
  newWatchAddress.remark = '';
};

watch(showAddWalletModal, (newVal) => {
  if (!newVal) {
    resetAddWalletForm();
    resetWatchAddressForm();
  }
});

// Load watch addresses
const loadWatchAddresses = async () => {
  try {
    selectedWatchAddressIds.value = [];
    const seq = ++watchAddressLoadSeq.value;
    const selectedKey = selectedGroupKeys.value[0];

    if (!selectedKey) {
      watchAddresses.value = [];
      return;
    }

    // Handle System Groups
    if (selectedKey === 'SYSTEM_EVM' || selectedKey === 'SYSTEM_SOLANA') {
      const chainType = selectedKey === 'SYSTEM_EVM' ? 'evm' : 'solana';
      const res = await invoke('get_watch_addresses', { group_id: null, chain_type: chainType });
      if (seq === watchAddressLoadSeq.value) watchAddresses.value = res;
      return;
    }

    // Handle user groups
    const groupId = parseDbId(selectedKey);
    if (groupId === null) {
      watchAddresses.value = [];
      return;
    }
    const group = findGroupById(groupId);
    const chainType = group?.chain_type || undefined;
    const res = await invoke('get_watch_addresses', { group_id: groupId, chain_type: chainType });
    if (seq === watchAddressLoadSeq.value) watchAddresses.value = res;
  } catch (e) {
    Message.error('加载仅地址失败: ' + e);
  }
};

// Handle add watch address
const handleAddWatchAddresses = async () => {
  try {
    if (!newWatchAddress.chain_type || !Array.isArray(newWatchAddress.group_path) || newWatchAddress.group_path.length === 0) {
      Message.error('请选择链类型与分组');
      return false;
    }

    // Parse addresses from text
    const addresses = newWatchAddress.addresses_text
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0);

    if (addresses.length === 0) {
      Message.error('请输入地址');
      return false;
    }

    let groupId = newWatchAddress.group_id;
    if (newWatchAddress.new_group_name && newWatchAddress.new_group_name.trim()) {
      groupId = await createGroupUnderCurrentSelection(newWatchAddress.new_group_name);
    }

    const count = await invoke('create_watch_addresses', {
      request: {
        group_id: groupId,
        name_prefix: newWatchAddress.name_prefix || null,
        chain_type: newWatchAddress.chain_type,
        addresses: addresses,
        remark: newWatchAddress.remark || null
      }
    });

    await loadWatchAddresses();
    Message.success(`已添加 ${count} 个仅地址`);
    showAddWalletModal.value = false;
    return true;
  } catch (e) {
    Message.error('添加仅地址失败: ' + (e?.message || e));
    return false;
  }
};

// Delete watch address
const handleDeleteWatchAddress = (address) => {
  if (!address?.id) {
    Message.error('缺少地址ID，无法删除');
    return;
  }

  const titleText = address.name ? `确认删除仅地址「${address.name}」` : '确认删除仅地址';
  Modal.warning({
    title: titleText,
    content: '删除后无法恢复，确定继续吗？',
    onOk: async () => {
      try {
        await invoke('delete_watch_address', { id: address.id });
        selectedWatchAddressIds.value = selectedWatchAddressIds.value.filter((k) => k !== address.id);
        await loadWatchAddresses();
        Message.success('删除成功');
      } catch (e) {
        Message.error('删除失败: ' + e);
      }
    }
  });
};

// Batch delete watch addresses
const handleBatchDeleteWatchAddresses = () => {
  if (selectedWatchAddressIds.value.length === 0) {
    Message.warning('请先选择要删除的地址');
    return;
  }

  Modal.warning({
    title: '确认批量删除',
    content: `确定要删除选中的 ${selectedWatchAddressIds.value.length} 个仅地址吗？删除后无法恢复，确定继续吗？`,
    onOk: async () => {
      try {
        for (const id of selectedWatchAddressIds.value) {
          await invoke('delete_watch_address', { id });
        }
        selectedWatchAddressIds.value = [];
        await loadWatchAddresses();
        Message.success('批量删除成功');
      } catch (e) {
        Message.error('删除失败: ' + e);
      }
    }
  });
};

// Edit watch address
const showEditWatchAddressModal = ref(false);
const editingWatchAddress = ref({ id: null, name: '', remark: '', group_id: null });

const handleEditWatchAddress = (address) => {
  editingWatchAddress.value = {
    id: address.id,
    name: address.name || '',
    remark: address.remark || '',
    group_id: address.group_id
  };
  showEditWatchAddressModal.value = true;
};

const handleSaveWatchAddress = async () => {
  if (!editingWatchAddress.value.id) {
    Message.error('缺少地址ID');
    return;
  }
  try {
    await invoke('update_watch_address', {
      request: {
        id: editingWatchAddress.value.id,
        group_id: editingWatchAddress.value.group_id,
        name: editingWatchAddress.value.name || null,
        remark: editingWatchAddress.value.remark || null
      }
    });
    await loadWatchAddresses();
    showEditWatchAddressModal.value = false;
    Message.success('保存成功');
  } catch (e) {
    Message.error('保存失败: ' + e);
  }
};

// Export watch addresses
const handleExportWatchAddresses = async () => {
  if (selectedWatchAddressIds.value.length === 0) {
    Message.warning('请先选择要导出的地址');
    return;
  }
  try {
    const data = await invoke('export_watch_addresses', { ids: selectedWatchAddressIds.value });

    const exportData = data.map(item => ({
      名称: item.name || '',
      地址: item.address,
      类型: item.chain_type === 'evm' ? 'EVM' : 'SOL',
      备注: item.remark || '',
      ID: item.id
    }));

    const worksheet = XLSX.utils.json_to_sheet(exportData);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, '仅地址数据');

    const filePath = await save({
      defaultPath: '仅地址导出.xlsx',
      filters: [{ name: 'Excel Files', extensions: ['xlsx'] }]
    });

    if (filePath) {
      const excelBuffer = XLSX.write(workbook, { bookType: 'xlsx', type: 'array' });
      await invoke('save_file', {
        filePath,
        content: new Uint8Array(excelBuffer)
      });

      openDirectory(filePath);
      Notification.success({
        content: `成功导出 ${exportData.length} 个仅地址`,
        duration: 5000,
        position: 'topLeft',
      });
    }
  } catch (e) {
    Message.error('导出失败: ' + (e?.message || e));
  }
};

// Handle watch address row click
const handleWatchAddressRowClick = (record) => {
  const key = record.id;
  const index = selectedWatchAddressIds.value.indexOf(key);
  if (index >= 0) {
    selectedWatchAddressIds.value.splice(index, 1);
  } else {
    selectedWatchAddressIds.value.push(key);
  }
};

const maskSecret = (val) => {
  if (!val) return '';
  const s = String(val);
  if (s.length <= 12) return s;
  return `${s.slice(0, 6)}...${s.slice(-6)}`;
};

const bytesToHex = (bytes) => Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
const hexToBytes = (hex) => {
  const s = String(hex || '').trim();
  if (!s || s.length % 2 !== 0) throw new Error('hex 格式错误');
  const out = new Uint8Array(s.length / 2);
  for (let i = 0; i < out.length; i++) out[i] = parseInt(s.slice(i * 2, i * 2 + 2), 16);
  return out;
};
const bytesToBase64 = (bytes) => {
  let binary = '';
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunkSize));
  }
  return btoa(binary);
};
const base64ToBytes = (b64) => {
  const binary = atob(b64);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
  return out;
};
const pemToSpkiBytes = (pem) => {
  const clean = String(pem || '')
    .replace(/-----BEGIN PUBLIC KEY-----/g, '')
    .replace(/-----END PUBLIC KEY-----/g, '')
    .replace(/\s+/g, '');
  return base64ToBytes(clean);
};

const encryptWithWalletManagerRsa = async (plaintext) => {
  const pem = await invoke('get_wallet_transport_public_key');
  const spki = pemToSpkiBytes(pem);
  const rsaKey = await crypto.subtle.importKey(
    'spki',
    spki,
    { name: 'RSA-OAEP', hash: 'SHA-1' },
    false,
    ['encrypt']
  );
  const enc = new TextEncoder();
  const encrypted = new Uint8Array(await crypto.subtle.encrypt({ name: 'RSA-OAEP' }, rsaKey, enc.encode(String(plaintext ?? ''))));
  return bytesToBase64(encrypted);
};

const initTransport = async () => {
  if (transportToken.value && transportAesKey.value) return;
  const pem = await invoke('get_wallet_transport_public_key');
  const spki = pemToSpkiBytes(pem);
  const rsaKey = await crypto.subtle.importKey(
    'spki',
    spki,
    { name: 'RSA-OAEP', hash: 'SHA-1' },
    false,
    ['encrypt']
  );

  const aesKey = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, ['encrypt', 'decrypt']);
  const raw = new Uint8Array(await crypto.subtle.exportKey('raw', aesKey));
  const encryptedRaw = new Uint8Array(await crypto.subtle.encrypt({ name: 'RSA-OAEP' }, rsaKey, raw));
  const token = await invoke('register_wallet_transport_key', { encrypted_key_b64: bytesToBase64(encryptedRaw) });
  transportAesKey.value = aesKey;
  transportToken.value = token;
};

const sealTransportSecret = async (plaintext) => {
  if (!transportToken.value || !transportAesKey.value) {
    await initTransport();
  }
  const text = String(plaintext ?? '').trim();
  if (!text) throw new Error('密文内容为空');
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const enc = new TextEncoder();
  const cipher = new Uint8Array(
    await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, transportAesKey.value, enc.encode(text))
  );
  return `t1:${transportToken.value}:${bytesToHex(iv)}:${bytesToBase64(cipher)}`;
};

const openTransportSecret = async (sealed) => {
  const s = String(sealed ?? '').trim();
  if (!s.startsWith('t1:')) throw new Error('必须使用加密格式传输');
  const parts = s.slice(3).split(':');
  if (parts.length !== 3) throw new Error('密文格式错误');
  const [token, ivHex, cipherB64] = parts;
  if (transportToken.value && token !== transportToken.value) throw new Error('传输令牌不匹配');
  if (!transportAesKey.value) throw new Error('缺少传输密钥');
  const iv = hexToBytes(ivHex);
  const cipher = base64ToBytes(cipherB64);
  const plainBytes = new Uint8Array(
    await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, transportAesKey.value, cipher)
  );
  return new TextDecoder().decode(plainBytes);
};

const parseDbId = (val) => {
  if (typeof val === 'number' && Number.isFinite(val)) return val;
  if (typeof val === 'string' && /^\d+$/.test(val)) return Number(val);
  return null;
};

const findGroupById = (id) => {
  if (id === null || id === undefined) return null;
  const walk = (nodes) => {
    for (const node of nodes || []) {
      if (node?.id === id) return node;
      const found = walk(node?.children);
      if (found) return found;
    }
    return null;
  };
  return walk(groups.value);
};

const copyToClipboard = async (text) => {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    Message.success('已复制');
  } catch (e) {
    Message.error('复制失败: ' + e);
  }
};

const copySealedSecret = async (sealed) => {
  if (!sealed) return;
  try {
    const plain = sealed.startsWith('t1:') ? await openTransportSecret(sealed) : await openSealedSecret(sealed, sessionPassword.value);
    await copyToClipboard(plain);
  } catch (e) {
    Message.error('解密失败: ' + (e?.message || e));
  }
};

const handleRevealPrivateKey = async (wallet) => {
  if (!wallet?.has_private_key) {
    Message.warning('该钱包没有私钥');
    return;
  }
  try {
    const secrets = await invoke('get_wallet_secrets', { id: wallet.id, password: null, transport_token: transportToken.value });
    if (!secrets?.sealed_private_key) {
      Message.warning('该钱包没有私钥');
      return;
    }
    currentWalletName.value = wallet.name || wallet.address;
    currentSealedPrivateKey.value = secrets.sealed_private_key;
    showSecretRevealModal.value = true;
  } catch (e) {
    Message.error('解密失败: ' + (e?.message || e));
  }
};

const handleBatchDeleteWallets = () => {
  if (selectedWalletIds.value.length === 0) {
    Message.warning('请先选择要删除的钱包');
    return;
  }

  Modal.warning({
    title: '确认批量删除',
    content: `确定要删除选中的 ${selectedWalletIds.value.length} 个钱包吗？删除后无法恢复，确定继续吗？`,
    onOk: async () => {
      try {
        for (const id of selectedWalletIds.value) {
          await invoke('delete_wallet', { id });
        }
        selectedWalletIds.value = [];
        await loadWallets();
        Message.success('批量删除成功');
      } catch (e) {
        Message.error('删除失败: ' + e);
      }
    }
  });
};

const resetAddWalletForm = () => {
  addWalletActiveTab.value = 'import';
  importKeyMode.value = 'private_key';
  mnemonicWalletCount.value = 1;
  mnemonicStartIndex.value = 0;
  bulkMode.value = 'same_mnemonic';
  bulkWordCount.value = 12;
  bulkWalletCount.value = 10;
  bulkStartIndex.value = 0;
  bulkGenerating.value = false;

  newWallet.name = '';
  newWallet.address = '';
  newWallet.chain_type = 'evm';
  newWallet.group_id = null;
  newWallet.group_name = '';
  newWallet.group_path = [];
  newWallet.group_search = '';
  newWallet.new_group_name = '';
  newWallet.private_key = '';
  newWallet.mnemonic = '';
  newWallet.remark = '';
};

watch(showAddWalletModal, (newVal) => {
  if (!newVal) {
    resetAddWalletForm();
  }
});

watch(importKeyMode, (mode) => {
  if (mode === 'mnemonic') {
    newWallet.private_key = '';
  } else {
    newWallet.mnemonic = '';
  }
});

// Watch view type changes and load appropriate data
watch(currentViewType, (newVal) => {
  if (newVal === 'full_wallet') {
    loadWallets();
  } else {
    loadWatchAddresses();
  }
});

const walletColumns = [
  { title: '序号', align: 'center', width: 53, slotName: 'index' },
  { title: '名称', align: 'center', dataIndex: 'name', width: 100, ellipsis: true, tooltip: true },
  { title: '地址', align: 'center', dataIndex: 'address', width: 380, ellipsis: true, tooltip: true },
  { title: '类型', align: 'center', slotName: 'chain_type', width: 70 },
  { title: '私钥', align: 'center', slotName: 'private_key', width: 95, ellipsis: true, tooltip: true },
  { title: '备注', align: 'center', dataIndex: 'remark', ellipsis: true, tooltip: true },
  { title: '操作', align: 'center', slotName: 'optional', width: 80, ellipsis: true, tooltip: true },
];

const walletRowSelection = { type: 'checkbox' };

// Group Options for Cascader
const groupOptions = ref([]);
const cascaderFieldNames = ref({ value: 'value', label: 'label', children: 'children' });

// 构建级联选择框的选项
const buildGroupOptions = async () => {
  try {
    const groups = await invoke('get_groups');
    
    // 如果没有分组，创建默认的生态分组
    if (!groups || groups.length === 0) {
      groupOptions.value = [
        { value: 'SYSTEM_EVM', label: 'EVM 生态', chain_type: 'evm', children: [] },
        { value: 'SYSTEM_SOLANA', label: 'Solana 生态', chain_type: 'solana', children: [] }
      ];
      return;
    }
    
    const map = {};
    const roots = [];
    
    groups.forEach(group => {
      map[group.id] = { ...group, children: [] };
    });
    
    groups.forEach(group => {
      const node = map[group.id];
      if (group.parent_id && map[group.parent_id]) {
        map[group.parent_id].children.push(node);
      } else {
        roots.push(node);
      }
    });
    
    const toCascaderOption = (node) => {
      const option = {
        value: node.id,
        label: node.name,
        chain_type: node.chain_type,
        children: []
      };
      if (node.children && node.children.length > 0) {
        option.children = node.children.map(toCascaderOption);
      }
      return option;
    };
    
    const evmRoots = [];
    const solanaRoots = [];
    const otherRoots = [];
    
    roots.forEach(root => {
      const option = toCascaderOption(root);
      if (root.chain_type === 'evm') {
        evmRoots.push(option);
      } else if (root.chain_type === 'solana') {
        solanaRoots.push(option);
      } else {
        otherRoots.push(option);
      }
    });
    
    groupOptions.value = [
      { value: 'SYSTEM_EVM', label: 'EVM 生态', chain_type: 'evm', children: evmRoots },
      { value: 'SYSTEM_SOLANA', label: 'Solana 生态', chain_type: 'solana', children: solanaRoots },
      ...otherRoots
    ];
  } catch (e) {
    console.error('构建分组选项失败:', e);
    // 确保即使失败也有默认选项
    groupOptions.value = [
      { value: 'SYSTEM_EVM', label: 'EVM 生态', chain_type: 'evm', children: [] },
      { value: 'SYSTEM_SOLANA', label: 'Solana 生态', chain_type: 'solana', children: [] }
    ];
  }
};

// 处理分组选择变化
const handleGroupChange = (value, selectedOptions) => {
  if (!value) {
    newWallet.group_id = null;
    newWallet.group_name = '';
    return;
  }
  
  const path = Array.isArray(value) ? value : [value];
  const lastValue = path[path.length - 1];
  newWallet.group_id = parseDbId(lastValue);
  
  const firstValue = path[0];
  if (firstValue === 'SYSTEM_EVM') newWallet.chain_type = 'evm';
  if (firstValue === 'SYSTEM_SOLANA') newWallet.chain_type = 'solana';
  
  if (selectedOptions && selectedOptions.length > 0 && parseDbId(lastValue) !== null) {
    const lastOption = selectedOptions[selectedOptions.length - 1];
    newWallet.group_name = lastOption.label;
  } else {
    newWallet.group_name = '';
  }
};

const createGroupUnderCurrentSelection = async (groupName) => {
  const name = (groupName || '').trim();
  if (!name) {
    throw new Error('分组名称不能为空');
  }
  
  const path = Array.isArray(newWallet.group_path) ? newWallet.group_path : [];
  if (path.length === 0) {
    throw new Error('请先选择一个父节点');
  }
  
  const firstValue = path[0];
  const chainType = firstValue === 'SYSTEM_EVM' ? 'evm' : (firstValue === 'SYSTEM_SOLANA' ? 'solana' : newWallet.chain_type);
  const lastValue = path[path.length - 1];
  const parentId = parseDbId(lastValue);
  
  const newGroupId = await invoke('create_group', {
    request: { parent_id: parentId, name, chain_type: chainType }
  });
  
  await loadGroups();
  await buildGroupOptions();
  
  newWallet.group_path = [...path, newGroupId];
  newWallet.group_id = newGroupId;
  newWallet.group_name = name;
  newWallet.group_search = '';
  newWallet.new_group_name = '';
  
  return newGroupId;
};

const handleCreateGroupInAddWallet = async () => {
  try {
    await createGroupUnderCurrentSelection(newWallet.new_group_name);
    Message.success('创建分组成功');
  } catch (e) {
    Message.error('创建分组失败: ' + (e?.message || e));
  }
};

// Batch Import
const showBatchImportModal = ref(false);
const showSecretRevealModal = ref(false);
const currentSealedPrivateKey = ref('');
const currentWalletName = ref('');
const importedWallets = ref([]);
const importProgress = ref(0);
const isImporting = ref(false);
const importResult = ref({ success: 0, failed: 0, errors: [] });
const fileList = ref([]);
const importColumns = ref([
  { title: '名称', dataIndex: 'name' },
  { title: '地址', dataIndex: 'address' },
  { title: '链类型', dataIndex: 'chain_type' },
  { title: '组名称', dataIndex: 'group_name' },
  { title: '私钥', dataIndex: 'private_key', slots: { customRender: 'private_key' } },
  { title: '助记词', dataIndex: 'mnemonic', slots: { customRender: 'mnemonic' } },
  { title: '备注', dataIndex: 'remark' }
]);

onMounted(async () => {
  showLoadingOverlay();
  try {
    await invoke('init_wallet_manager_tables');
    const isSet = await invoke('is_password_set');
    hideLoadingOverlay();
    isLoading.value = false;
    if (!isSet) {
      showInitModal.value = true;
    } else {
      showUnlockModal.value = true;
    }
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri && !pageLoadedEmitted) {
      await nextTick();
      pageLoadedEmitted = true;
      getCurrentWindow().emit('page-loaded');
    }
  } catch (e) {
    hideLoadingOverlay();
    isLoading.value = false;
    Message.error('初始化失败: ' + e);
    const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__;
    if (isTauri && !pageLoadedEmitted) {
      await nextTick();
      pageLoadedEmitted = true;
      getCurrentWindow().emit('page-loaded');
    }
  }
});

  const handleUnlock = async () => {
    try {
      // 1. 先解锁加密数据库
      await invoke('unlock_encrypted_db', { password: passwordInput.value });
      
      // 2. 初始化传输加密
      await initTransport();
      
      // 3. 验证应用密码
      const encryptedPasswordB64 = await encryptWithWalletManagerRsa(passwordInput.value);
      const success = await invoke('verify_password', { request: { password: null, encrypted_password_b64: encryptedPasswordB64 } });
      if (success) {
        sessionPassword.value = passwordInput.value;
        isUnlocked.value = true;
        showUnlockModal.value = false;
        loadGroups();
        loadWallets();
      } else {
        Message.error('密码错误');
      }
    } catch (e) {
      Message.error('解锁失败: ' + (e?.toString() || '未知错误'));
    }
  };

  const handleInit = async () => {
    if (initPassword.value !== initPasswordConfirm.value) {
      Message.error('两次密码不一致');
      return;
    }
    if (!initPassword.value) {
      Message.error('密码不能为空');
      return;
    }
    try {
      // 1. 先初始化加密数据库
      await invoke('init_encrypted_db', { password: initPassword.value });
      
      // 2. 初始化传输加密
      await initTransport();
      
      // 3. 初始化应用密码
      const encryptedPasswordB64 = await encryptWithWalletManagerRsa(initPassword.value);
      await invoke('init_password', { request: { password: null, encrypted_password_b64: encryptedPasswordB64 } });
      sessionPassword.value = initPassword.value;
      isUnlocked.value = true;
      showInitModal.value = false;
      loadGroups();
      loadWallets();
    } catch (e) {
      Message.error('初始化失败: ' + (e?.toString() || '未知错误'));
    }
  };

  const handleChangePassword = async () => {
    if (!changePasswordForm.oldPassword) {
      Message.error('请输入当前密码');
      return;
    }
    if (!changePasswordForm.newPassword) {
      Message.error('请输入新密码');
      return;
    }
    if (changePasswordForm.newPassword.length < 6) {
      Message.error('新密码长度不能少于6位');
      return;
    }
    if (changePasswordForm.newPassword !== changePasswordForm.confirmPassword) {
      Message.error('两次密码不一致');
      return;
    }

    changePasswordLoading.value = true;
    isCriticalOperation.value = true;

    try {
      await invoke('change_password', {
        request: {
          old_password: changePasswordForm.oldPassword,
          new_password: changePasswordForm.newPassword
        }
      });

      // 更新会话密码
      sessionPassword.value = changePasswordForm.newPassword;

      showChangePasswordModal.value = false;
      Message.success('密码修改成功');
    } catch (e) {
      Message.error('密码修改失败: ' + (e?.message || e));
    } finally {
      changePasswordLoading.value = false;
      isCriticalOperation.value = false;
    }
  };

  const buildTree = (list) => {
    const map = {};
    const roots = [];
    
    // 第一次遍历：创建所有节点的映射
    list.forEach(item => {
      map[item.id] = { ...item };
    });
    
    // 第二次遍历：构建树结构
    list.forEach(item => {
      if (item.parent_id && map[item.parent_id]) {
        if (!map[item.parent_id].children) {
          map[item.parent_id].children = [];
        }
        map[item.parent_id].children.push(map[item.id]);
      } else {
        roots.push(map[item.id]);
      }
    });
    
    return roots;
  };

  const loadGroups = async () => {
    try {
      const res = await invoke('get_groups');
      const userRoots = buildTree(res);
      
      const systemGroups = [
        { id: 'SYSTEM_EVM', name: 'EVM 生态', children: [], isSystem: true },
        { id: 'SYSTEM_SOLANA', name: 'Solana 生态', children: [], isSystem: true }
      ];
      
      // Distribute user roots into system groups
      const otherRoots = [];
      userRoots.forEach(node => {
          if (node.chain_type === 'evm') {
              systemGroups[0].children.push(node);
          } else if (node.chain_type === 'solana') {
              systemGroups[1].children.push(node);
          } else {
              otherRoots.push(node);
          }
      });
      
      groups.value = [...systemGroups, ...otherRoots];
    } catch (e) {
      Message.error('加载分组失败: ' + e);
    }
  };



  const loadWallets = async () => {
    try {
      selectedWalletIds.value = [];
      const seq = ++walletLoadSeq.value;
      const selectedKey = selectedGroupKeys.value[0];
      
      if (!selectedKey) {
        wallets.value = [];
        return;
      }
      
      // Handle System Groups - show all wallets of that ecosystem
      if (selectedKey === 'SYSTEM_EVM' || selectedKey === 'SYSTEM_SOLANA') {
        const chainType = selectedKey === 'SYSTEM_EVM' ? 'evm' : 'solana';
        const res = await invoke('get_wallets', { group_id: null, chain_type: chainType });
        if (seq === walletLoadSeq.value) wallets.value = res;
        return;
      }
      
      // Handle user groups - show wallets in that group
      const groupId = parseDbId(selectedKey);
      if (groupId === null) {
        wallets.value = [];
        return;
      }
      const group = findGroupById(groupId);
      const chainType = group?.chain_type || undefined;
      const res = await invoke('get_wallets', { group_id: groupId, chain_type: chainType });
      if (seq === walletLoadSeq.value) wallets.value = res;
    } catch (e) {
      Message.error('加载钱包失败: ' + e);
    }
  };

  const onGroupSelect = (keys) => {
    if (keys.length > 0) {
      selectedGroupKeys.value = keys.map((k) => {
        const id = parseDbId(k);
        return id === null ? k : id;
      });
      loadWallets();
      loadWatchAddresses();
    }
  };

  const onNodeClick = (node) => {
    // 无论是否是叶子节点，都尝试展开/折叠
    // 只有当有子节点时，展开操作才有视觉效果，但这由 Tree 组件控制
    // 我们只需要维护 expandedKeys 即可
    if (node.children && node.children.length > 0) {
      const key = node.id;
      const index = expandedKeys.value.indexOf(key);
      if (index > -1) {
        expandedKeys.value.splice(index, 1);
      } else {
        expandedKeys.value.push(key);
      }
    }
    // 手动设置选中状态
    selectedGroupKeys.value = [node.id];
    loadWallets();
    loadWatchAddresses();
  };

    const getSelectedGroupName = () => {
    const key = selectedGroupKeys.value[0];
    if (!key) return '未选择 (无法创建)';
    
    if (key === 'SYSTEM_EVM') return 'EVM 生态 /';
    if (key === 'SYSTEM_SOLANA') return 'Solana 生态 /';
    
    // Find node and build path
    const findPath = (nodes, path = []) => {
        for (const node of nodes) {
            const currentPath = [...path, node.name];
            if (node.id === key) return currentPath;
            if (node.children) {
                const found = findPath(node.children, currentPath);
                if (found) return found;
            }
        }
        return null;
    };
    
    const path = findPath(groups.value);
    return path ? path.join(' / ') : '未知分组';
  };
  
  const handleRenameGroup = async () => {
    try {
        if (!renameGroupName.value) {
            Message.warning('请输入新的分组名称');
            return false;
        }

        const groupId = parseDbId(renamingGroupId.value);
        if (groupId === null) {
            Message.error('分组ID无效');
            return false;
        }
        
        await invoke('update_group', {
            request: {
                id: groupId,
                name: renameGroupName.value
            }
        });
        
        Message.success('重命名成功');
        showRenameGroupModal.value = false;
        loadGroups();
    } catch (e) {
        Message.error('重命名失败: ' + e);
        return false;
    }
  };

  const handleAddGroup = async () => {
    try {
      if (!newGroupName.value) {
        Message.warning('请输入分组名称');
        return;
      }
      
      const selectedKey = selectedGroupKeys.value[0];
      if (!selectedKey) {
          Message.warning('请选择一个生态分组或现有分组');
          return;
      }
      
      let parentId = null;
      let chainType = '';
      
      if (selectedKey === 'SYSTEM_EVM') {
          parentId = null;
          chainType = 'evm';
      } else if (selectedKey === 'SYSTEM_SOLANA') {
          parentId = null;
          chainType = 'solana';
      } else {
          // If selected a normal group, inherit its chain_type
          // We need to find the node to get its chain_type
          const findNode = (nodes) => {
              for (const node of nodes) {
                  if (node.id === selectedKey) return node;
                  if (node.children) {
                      const found = findNode(node.children);
                      if (found) return found;
                  }
              }
              return null;
          };
          
          const node = findNode(groups.value);
          if (node) {
              parentId = node.id;
              chainType = node.chain_type;
          } else {
              Message.error('无法确定父分组信息');
              return;
          }
      }
      
      await invoke('create_group', { 
          request: { 
              parent_id: parentId, 
              name: newGroupName.value,
              chain_type: chainType
          } 
      });
      showAddGroupModal.value = false;
      newGroupName.value = '';
      loadGroups();
      Message.success('创建分组成功');
    } catch (e) {
      Message.error('创建分组失败: ' + e);
    }
  };

  const handleDeleteGroup = async () => {
    // If called directly (not from context menu), use selectedGroupKeys
    let targetKey = selectedGroupKeys.value[0];
    
    // If context menu triggered this, we might want to prioritize that,
    // but current logic sets selectedGroupKeys on context menu delete.
    // Let's make it robust.
    
    if (!targetKey) {
        Message.warning('请选择要删除的分组');
        return;
    }
    
    if (targetKey === 'SYSTEM_EVM' || targetKey === 'SYSTEM_SOLANA') {
        Message.warning('系统生态分组无法删除');
        return;
    }

    const groupId = parseDbId(targetKey);
    if (groupId === null) {
        Message.error('分组ID无效');
        return;
    }
    
    Modal.warning({
        title: '确认删除',
        content: '确定要删除该分组吗？删除后该分组下的子分组将被一并删除，所有分组下的钱包信息将被删除。',
        onOk: async () => {
            try {
                await invoke('delete_group', { id: groupId });
                Message.success('删除成功');
                selectedGroupKeys.value = []; // 清除选中状态
                loadGroups();
                loadWallets(); // 刷新钱包列表（可能有些变成了未分组）
            } catch (e) {
                Message.error('删除失败: ' + e);
            }
        }
    });
  };

  const handleDeleteWallet = (wallet) => {
    if (!wallet?.id) {
      Message.error('缺少钱包ID，无法删除');
      return;
    }

    const titleText = wallet.name ? `确认删除钱包「${wallet.name}」` : '确认删除钱包';
    Modal.warning({
      title: titleText,
      content: '删除后无法恢复，确定继续吗？',
      onOk: async () => {
        try {
          await invoke('delete_wallet', { id: wallet.id });
          selectedWalletIds.value = selectedWalletIds.value.filter((k) => k !== wallet.id);
          await loadWallets();
          Message.success('删除成功');
        } catch (e) {
          Message.error('删除失败: ' + e);
        }
      }
    });
  };

  const handleEditWallet = (wallet) => {
    editingWallet.value = {
      id: wallet.id,
      name: wallet.name || '',
      remark: wallet.remark || '',
      group_id: wallet.group_id
    };
    showEditWalletModal.value = true;
  };

  const handleSaveWallet = async () => {
    if (!editingWallet.value.id) {
      Message.error('缺少钱包ID');
      return;
    }
    try {
      await invoke('update_wallet', {
        request: {
          id: editingWallet.value.id,
          group_id: editingWallet.value.group_id,
          name: editingWallet.value.name || null,
          remark: editingWallet.value.remark || null
        }
      });
      await loadWallets();
      showEditWalletModal.value = false;
      Message.success('保存成功');
    } catch (e) {
      Message.error('保存失败: ' + e);
    }
  };

  const handleRowClick = (record) => {
    const key = record.id;
    const index = selectedWalletIds.value.indexOf(key);
    if (index >= 0) {
      selectedWalletIds.value.splice(index, 1);
    } else {
      selectedWalletIds.value.push(key);
    }
  };

  const handleAddWallet = async () => {
    try {
      // Handle watch address tab
      if (addWalletActiveTab.value === 'watch') {
        return handleAddWatchAddresses();
      }

      if (!newWallet.chain_type || !Array.isArray(newWallet.group_path) || newWallet.group_path.length === 0) {
        Message.error('请选择链类型与分组');
        return false;
      }

      let groupId = newWallet.group_id;
      if (newWallet.new_group_name && newWallet.new_group_name.trim()) {
        groupId = await createGroupUnderCurrentSelection(newWallet.new_group_name);
      }

      if (addWalletActiveTab.value === 'import') {
        if (importKeyMode.value === 'mnemonic') {
          const mnemonic = (newWallet.mnemonic || '').trim();
          if (!mnemonic) {
            Message.error('请输入助记词');
            return false;
          }
          const sealedMnemonic = await sealTransportSecret(mnemonic);
          const count = Number(mnemonicWalletCount.value) || 1;
          const startIndex = Number(mnemonicStartIndex.value) || 0;
          await invoke('create_wallets', {
            request: {
              group_id: groupId,
              name: newWallet.name || null,
              chain_type: newWallet.chain_type,
              mode: 'mnemonic_import',
              sealed_mnemonic: sealedMnemonic,
              sealed_private_key: null,
              count,
              start_index: startIndex,
              word_count: null,
              remark: newWallet.remark || null,
              preview_limit: 0,
              include_secrets: false,
              transport_token: transportToken.value
            }
          });
        } else {
          const privateKey = (newWallet.private_key || '').trim();
          if (!privateKey) {
            Message.error('请输入私钥');
            return false;
          }
          const sealedPrivateKey = await sealTransportSecret(privateKey);
          await invoke('create_wallets', {
            request: {
              group_id: groupId,
              name: newWallet.name || null,
              chain_type: newWallet.chain_type,
              mode: 'private_key_import',
              sealed_mnemonic: null,
              sealed_private_key: sealedPrivateKey,
              count: 1,
              start_index: null,
              word_count: null,
              remark: newWallet.remark || null,
              preview_limit: 0,
              include_secrets: false,
              transport_token: transportToken.value
            }
          });
        }

        await loadWallets();
        Message.success('添加成功');
        showAddWalletModal.value = false;
        return true;
      }

      bulkGenerating.value = true;
      bulkTotalCount.value = 0;
      bulkSealedMnemonic.value = '';
      bulkMnemonicMasked.value = '';
      const mode = bulkMode.value === 'same_mnemonic' ? 'generate_same_mnemonic' : 'generate_different_mnemonic';
      const count = Number(bulkWalletCount.value) || 1;
      const startIndex = Number(bulkStartIndex.value) || 0;
      const wordCount = Number(bulkWordCount.value) || 12;

      const previewLimit = count > 1000 ? 200 : count;
      const created = await invoke('create_wallets', {
        request: {
          group_id: groupId,
          name: newWallet.name || null,
          chain_type: newWallet.chain_type,
          mode,
          sealed_mnemonic: null,
          sealed_private_key: null,
          count,
          start_index: startIndex,
          word_count: wordCount,
          remark: newWallet.remark || null,
          preview_limit: previewLimit,
          include_secrets: true,
          transport_token: transportToken.value
        }
      });

      const total = Number(created?.total || 0);
      bulkTotalCount.value = total;
      bulkSealedMnemonic.value = created?.sealed_mnemonic || '';
      if (bulkSealedMnemonic.value) {
        try {
          bulkMnemonicMasked.value = maskSecret(
            bulkSealedMnemonic.value.startsWith('t1:')
              ? await openTransportSecret(bulkSealedMnemonic.value)
              : await openSealedSecret(bulkSealedMnemonic.value, sessionPassword.value)
          );
        } catch (_) {}
      }

      bulkGenerating.value = false;
      await loadWallets();
      Message.success(`已生成并保存 ${bulkTotalCount.value} 个钱包`);
      return false;
    } catch (e) {
      bulkGenerating.value = false;
      Message.error('添加钱包失败: ' + e);
      return false;
    }
  };

// Batch Import Methods
const handleFileUpload = async (info) => {
  if (info.file.status === 'done') {
    const file = info.file.originFileObj;
    try {
      if (file.name.endsWith('.csv')) {
        await parseCSVFile(file);
      } else if (file.name.endsWith('.xlsx') || file.name.endsWith('.xls')) {
        await parseExcelFile(file);
      } else {
        Message.error('不支持的文件格式，请选择Excel或CSV文件');
      }
    } catch (e) {
      Message.error('文件解析失败: ' + e.message);
    }
  } else if (info.file.status === 'error') {
    Message.error('文件上传失败');
  }
};

const parseCSVFile = (file) => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const content = e.target.result;
        const lines = content.split('\n').filter(line => line.trim());
        const headers = lines[0].split(',').map(header => header.trim());
        const wallets = [];
        
        for (let i = 1; i < lines.length; i++) {
          const values = lines[i].split(',');
          const wallet = {};
          headers.forEach((header, index) => {
            wallet[header] = values[index]?.trim() || '';
          });
          wallets.push(wallet);
        }
        
        importedWallets.value = wallets;
        resolve();
      } catch (e) {
        reject(e);
      }
    };
    reader.onerror = () => reject(new Error('文件读取失败'));
    reader.readAsText(file);
  });
};

const parseExcelFile = (file) => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const data = new Uint8Array(e.target.result);
        const workbook = XLSX.read(data, { type: 'array' });
        const firstSheetName = workbook.SheetNames[0];
        const worksheet = workbook.Sheets[firstSheetName];
        const jsonData = XLSX.utils.sheet_to_json(worksheet);
        
        importedWallets.value = jsonData;
        resolve();
      } catch (e) {
        reject(e);
      }
    };
    reader.onerror = () => reject(new Error('文件读取失败'));
    reader.readAsArrayBuffer(file);
  });
};

const startBatchImport = async () => {
  if (importedWallets.value.length === 0) {
    Message.warning('没有要导入的钱包数据');
    return;
  }
  
  isImporting.value = true;
  importProgress.value = 0;
  importResult.value = { success: 0, failed: 0, errors: [] };
  
  // 预加载所有分组，避免重复查询
  const existingGroups = await invoke('get_groups');
  const groupMap = new Map();
  existingGroups.forEach(g => groupMap.set(g.name, g.id));
  
  for (let i = 0; i < importedWallets.value.length; i++) {
    const wallet = importedWallets.value[i];
    try {
      // 验证必要字段
      if (!wallet.name || !wallet.chain_type || !wallet.group_name) {
        throw new Error(`第 ${i + 1} 条记录缺少必要字段`);
      }

      if (!wallet.private_key && !wallet.mnemonic) {
        throw new Error(`第 ${i + 1} 条记录缺少私钥或助记词`);
      }
      
      if (!['evm', 'solana'].includes(wallet.chain_type)) {
        throw new Error(`第 ${i + 1} 条记录的链类型无效，应为 evm 或 solana`);
      }
      
      // 查找或创建分组
      let groupId = groupMap.get(wallet.group_name);
      
      if (!groupId) {
        // 自动创建分组
        const newGroupId = await invoke('create_group', {
          request: { parent_id: null, name: wallet.group_name, chain_type: wallet.chain_type }
        });
        groupId = newGroupId;
        groupMap.set(wallet.group_name, groupId);
      }

      const mnemonic = (wallet.mnemonic || '').trim();
      const privateKey = (wallet.private_key || '').trim();
      const walletAddress = (wallet.address || '').trim();
      const sealedMnemonic = mnemonic ? await sealTransportSecret(mnemonic) : null;
      const sealedPrivateKey = privateKey ? await sealTransportSecret(privateKey) : null;
      
      await invoke('create_wallets', {
        request: {
          group_id: groupId,
          name: wallet.name,
          chain_type: wallet.chain_type,
          mode: mnemonic ? 'mnemonic_import' : 'private_key_import',
          sealed_mnemonic: sealedMnemonic,
          sealed_private_key: sealedPrivateKey,
          count: 1,
          start_index: 0,
          word_count: null,
          remark: wallet.remark || null,
          preview_limit: 0,
          include_secrets: false,
          transport_token: transportToken.value
        },
        address: walletAddress || null  // 传入地址用于验证（可选）
      });
      
      importResult.value.success++;
    } catch (e) {
      importResult.value.failed++;
      importResult.value.errors.push(`第 ${i + 1} 条记录: ${e.message}`);
    }
    
    importProgress.value = Math.round(((i + 1) / importedWallets.value.length) * 100);
  }
  
  isImporting.value = false;
  await loadGroups();
  await loadWallets();
};

const closeBatchImportModal = () => {
  showBatchImportModal.value = false;
  importedWallets.value = [];
  importProgress.value = 0;
  importResult.value = { success: 0, failed: 0, errors: [] };
  fileList.value = [];
};

const downloadTemplate = async () => {
  // 使用xlsx库动态生成模板文件
  const worksheet = XLSX.utils.aoa_to_sheet([
    ['name', 'chain_type', 'group_name', 'address', 'private_key', 'mnemonic', 'remark'],
    ['示例钱包1', 'evm', '个人钱包', '', '0x(64位十六进制私钥)', '', ''],
    ['示例钱包2', 'evm', '个人钱包', '', '', 'work man father ...', ''],
    ['示例钱包3', 'solana', 'Solana钱包', '', '(base58 keypair)', '', '']
  ]);
  const workbook = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(workbook, worksheet, 'Sheet1');
  
  // 保存文件
  const filePath = await downloadWithDialog('import_model.xlsx', '钱包导入模板.xlsx');
  
  if (filePath) {
    // 将工作簿转换为二进制数据
    const excelBuffer = XLSX.write(workbook, { bookType: 'xlsx', type: 'array' });
    await invoke('save_file', { 
      filePath, 
      content: new Uint8Array(excelBuffer) 
    });
    
    openDirectory(filePath);
    Notification.success({
      content: '模板已保存',
      duration: 5000,
      position: 'topLeft',
    });
  }
};

// Export Function
const handleExportWallets = async () => {
  if (selectedWalletIds.value.length === 0) {
    Message.warning('请先选择要导出的钱包');
    return;
  }
  showExportModal.value = true;
};

const confirmExport = async () => {
  if (!exportPassword.value) {
    Message.error('请输入密码');
    return;
  }

  isExporting.value = true;
  try {
    const data = await invoke('export_wallets', {
      ids: selectedWalletIds.value,
      password: exportPassword.value
    });

    const exportData = data.map(w => ({
      名称: w.name || '',
      地址: w.address,
      类型: w.chain_type === 'evm' ? 'EVM' : 'SOL',
      私钥: w.private_key || '',
      助记词: w.mnemonic || '',
      助记词序号: w.mnemonic_index ?? '',
      备注: w.remark || '',
      钱包ID: w.id
    }));

    const worksheet = XLSX.utils.json_to_sheet(exportData);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, '钱包数据');

    const filePath = await save({
      defaultPath: '钱包数据导出.xlsx',
      filters: [{ name: 'Excel Files', extensions: ['xlsx'] }]
    });

    if (filePath) {
      const excelBuffer = XLSX.write(workbook, { bookType: 'xlsx', type: 'array' });
      await invoke('save_file', {
        filePath,
        content: new Uint8Array(excelBuffer)
      });

      openDirectory(filePath);
      Notification.success({
        content: `成功导出 ${exportData.length} 个钱包`,
        duration: 5000,
        position: 'topLeft',
      });
    }

    showExportModal.value = false;
    exportPassword.value = '';
  } catch (e) {
    Message.error('导出失败: ' + (e?.message || e));
  } finally {
    isExporting.value = false;
  }
};
</script>

<template>
  <TitleBar
    :title="windowTitle"
    :custom-close="true"
    :disable-close="isCriticalOperation || bulkGenerating || isImporting"
    @before-close="handleBeforeClose"
  />
  <!-- 全屏遮罩层 - 关键操作进行中 -->
  <div v-if="isCriticalOperation || bulkGenerating || isImporting" class="critical-operation-overlay" @click.stop>
    <div class="critical-operation-content">
      <a-spin size="large" />
      <p class="critical-operation-text">
        {{ isCriticalOperation ? '正在修改密码并重新加密数据...' : (bulkGenerating ? '正在生成、加密并保存钱包...' : '正在批量导入钱包...') }}
      </p>
      <p class="critical-operation-hint">请勿关闭程序或刷新页面</p>
    </div>
  </div>
  <div class="wallet-manager-page">
    <div v-if="isUnlocked" class="main-layout">
        <div class="sidebar">
            <div class="sidebar-header">
                <span>分组列表</span>
                <div>
                    <a-button type="text" size="mini" @click="handleDeleteGroup" :disabled="!selectedGroupKeys[0] || ['SYSTEM_EVM', 'SYSTEM_SOLANA'].includes(selectedGroupKeys[0])" class="group-action-btn delete-btn">
                        <template #icon><icon-delete style="font-size: 18px;" /></template>
                    </a-button>
                    <a-button type="text" size="mini" @click="showAddGroupModal = true" class="group-action-btn add-btn">
                        <template #icon><icon-plus style="font-size: 18px;"/></template>
                    </a-button>
                </div>
            </div>
            <div class="sidebar-tree-wrapper">
                <a-tree
                    :data="groups"
                    :field-names="{ key: 'id', title: 'name', children: 'children' }"
                    block-node
                    v-model:selected-keys="selectedGroupKeys"
                    v-model:expanded-keys="expandedKeys"
                    @select="onGroupSelect"
                    @node-click="onNodeClick"
                >
                    <template #icon="{ node }">
                        <span class="tree-folder-icon" :class="{ 'is-expanded': expandedKeys.includes(node.id), 'is-system': node.isSystem }">
                            <icon-folder v-if="node.children && node.children.length > 0" />
                            <icon-file v-else />
                        </span>
                    </template>
                    <template #title="node">
                        <a-dropdown trigger="contextMenu" alignPoint class="tree-node-dropdown">
                            <div class="tree-node-content" :class="{ 'is-selected': selectedGroupKeys.includes(node.id) }" @click.stop="onNodeClick(node)">
                                <span class="node-label">{{ node.name }}</span>
                            </div>
                            <template #content>
                                <a-doption @click="() => {
                                    selectedGroupKeys = [node.id];
                                    newGroupName = '';
                                    showAddGroupModal = true;
                                }">新增子节点</a-doption>
                                <a-doption @click="() => {
                                    renamingGroupId = node.id;
                                    renameGroupName = node.name;
                                    showRenameGroupModal = true;
                                }" :disabled="node.isSystem">重命名</a-doption>
                                <a-doption @click="() => {
                                    selectedGroupKeys = [node.id];
                                    handleDeleteGroup();
                                }" :disabled="node.isSystem">删除</a-doption>
                            </template>
                        </a-dropdown>
                    </template>
                </a-tree>
            </div>
        </div>
        <div class="content">
            <div class="toolbar">
                <!-- View Type Switcher -->
                <a-radio-group v-model="currentViewType" type="button" style="margin-right: 16px;">
                    <a-radio value="full_wallet">完整钱包</a-radio>
                    <a-radio value="address_only">仅地址</a-radio>
                </a-radio-group>

                <!-- Full Wallet Actions -->
                <template v-if="currentViewType === 'full_wallet'">
                    <a-button type="primary" @click="showAddWalletModal = true">添加钱包</a-button>
                    <a-button style="margin-left: 10px;" type="primary" status="success" @click="showBatchImportModal = true">批量导入</a-button>
                    <a-button style="margin-left: 10px;" type="outline" @click="downloadTemplate">下载模板</a-button>
                    <a-button
                      style="margin-left: 10px;"
                      type="outline"
                      :disabled="selectedWalletIds.length === 0"
                      @click="handleExportWallets"
                    >
                      导出 ({{ selectedWalletIds.length }})
                    </a-button>
                    <a-button
                      style="margin-left: 10px;"
                      type="primary"
                      status="danger"
                      :disabled="selectedWalletIds.length === 0"
                      @click="handleBatchDeleteWallets"
                    >
                      批量删除 ({{ selectedWalletIds.length }})
                    </a-button>
                </template>

                <!-- Watch Address Actions -->
                <template v-else>
                    <a-button type="primary" @click="showAddWalletModal = true">添加地址</a-button>
                    <a-button
                      style="margin-left: 10px;"
                      type="outline"
                      :disabled="selectedWatchAddressIds.length === 0"
                      @click="handleExportWatchAddresses"
                    >
                      导出 ({{ selectedWatchAddressIds.length }})
                    </a-button>
                    <a-button
                      style="margin-left: 10px;"
                      type="primary"
                      status="danger"
                      :disabled="selectedWatchAddressIds.length === 0"
                      @click="handleBatchDeleteWatchAddresses"
                    >
                      批量删除 ({{ selectedWatchAddressIds.length }})
                    </a-button>
                </template>

                <a-button
                  style="float: right;"
                  type="outline"
                  @click="showChangePasswordModal = true"
                >
                  修改密码
                </a-button>
            </div>

            <!-- Full Wallet Table -->
            <template v-if="currentViewType === 'full_wallet'">
            <VirtualScrollerTable
              class="wallet-table"
              :columns="walletColumns"
              :data="wallets"
              :row-selection="walletRowSelection"
              :selected-keys="selectedWalletIds"
              @update:selected-keys="selectedWalletIds = $event"
              @row-click="handleRowClick"
              row-key="id"
              height="100%"
              page-type="wallet_manager"
              :empty-data="wallets.length === 0"
            >
              <template #private_key_status="{ record }">
                <a-tag v-if="record.has_private_key" color="green">已加密</a-tag>
                <a-tag v-else color="arcoblue">无私钥</a-tag>
              </template>
              <template #private_key="{ record }">
                <a-button v-if="record.has_private_key" type="primary" status="success" size="mini" @click.stop="handleRevealPrivateKey(record)">安全复制</a-button>
                <span v-else>-</span>
              </template>
              <template #chain_type="{ record }">
                <a-tag :color="record.chain_type === 'evm' ? 'blue' : 'purple'">
                  {{ record.chain_type === 'evm' ? 'EVM' : 'SOL' }}
                </a-tag>
              </template>
              <template #optional="{ record }">
                <a-button
                  type="secondary"
                  size="mini"
                  @click.stop="handleEditWallet(record)"
                >
                  <template #icon><icon-edit style="font-size: 16px;" /></template>
                </a-button>
                <a-button
                  type="secondary"
                  size="mini"
                  status="danger"
                  style="margin-left: 10px;"
                  @click.stop="handleDeleteWallet(record)"
                >
                  <template #icon><icon-delete style="font-size: 16px;" /></template>
                </a-button>
              </template>
            </VirtualScrollerTable>
            <!-- 数据汇总信息 -->
            <div class="table-summary">
                <a-divider direction="vertical" />
                <template v-if="currentViewType === 'full_wallet'">
                    <span class="summary-item">
                        <span class="summary-label">当前分组钱包：</span>
                        <span class="summary-value">{{ wallets.length }}</span>
                    </span>
                    <span class="summary-item">
                        <span class="summary-label">已选：</span>
                        <span class="summary-value">{{ selectedWalletIds.length }}</span>
                    </span>
                    <span class="summary-item">
                        <span class="summary-label">EVM：</span>
                        <span class="summary-value evm">{{ wallets.filter(w => w.chain_type === 'evm').length }}</span>
                    </span>
                    <span class="summary-item">
                        <span class="summary-label">SOL：</span>
                        <span class="summary-value solana">{{ wallets.filter(w => w.chain_type === 'solana').length }}</span>
                    </span>
                    <span class="summary-item">
                        <span class="summary-label">有私钥：</span>
                        <span class="summary-value success">{{ wallets.filter(w => w.has_private_key).length }}</span>
                    </span>
                </template>
                <template v-else>
                    <span class="summary-item">
                        <span class="summary-label">当前分组地址：</span>
                        <span class="summary-value">{{ watchAddresses.length }}</span>
                    </span>
                    <span class="summary-item">
                        <span class="summary-label">已选：</span>
                        <span class="summary-value">{{ selectedWatchAddressIds.length }}</span>
                    </span>
                    <span class="summary-item">
                        <span class="summary-label">EVM：</span>
                        <span class="summary-value evm">{{ watchAddresses.filter(w => w.chain_type === 'evm').length }}</span>
                    </span>
                    <span class="summary-item">
                        <span class="summary-label">SOL：</span>
                        <span class="summary-value solana">{{ watchAddresses.filter(w => w.chain_type === 'solana').length }}</span>
                    </span>
                </template>
                <a-divider direction="vertical" />
            </div>
            </template>

            <!-- Watch Address Table -->
            <template v-if="currentViewType === 'address_only'">
            <VirtualScrollerTable
              class="wallet-table"
              :columns="watchAddressColumns"
              :data="watchAddresses"
              :row-selection="watchAddressRowSelection"
              :selected-keys="selectedWatchAddressIds"
              @update:selected-keys="selectedWatchAddressIds = $event"
              @row-click="handleWatchAddressRowClick"
              row-key="id"
              height="100%"
              page-type="wallet_manager"
              :empty-data="watchAddresses.length === 0"
            >
              <template #chain_type="{ record }">
                <a-tag :color="record.chain_type === 'evm' ? 'blue' : 'purple'">
                  {{ record.chain_type === 'evm' ? 'EVM' : 'SOL' }}
                </a-tag>
              </template>
              <template #optional="{ record }">
                <a-button
                  type="secondary"
                  size="mini"
                  @click.stop="handleEditWatchAddress(record)"
                >
                  <template #icon><icon-edit style="font-size: 16px;" /></template>
                </a-button>
                <a-button
                  type="secondary"
                  size="mini"
                  status="danger"
                  style="margin-left: 10px;"
                  @click.stop="handleDeleteWatchAddress(record)"
                >
                  <template #icon><icon-delete style="font-size: 16px;" /></template>
                </a-button>
              </template>
            </VirtualScrollerTable>
            <!-- Watch Address Summary is included in the main summary above -->
            </template>
        </div>
    </div>

    <!-- Init Modal -->
    <a-modal
      :visible="showInitModal"
      title="设置主密码"
      :closable="false"
      :mask-closable="false"
      :esc-to-close="false"
      :footer="false"
      mask-animation-name="none"
      modal-animation-name="none"
      :mask-style="{ background: 'rgba(0, 0, 0, 0.6)' }"
    >
        <a-form layout="vertical">
            <a-alert type="warning" style="margin-bottom: 10px;">
                此密码用于加密存储所有私钥，请务必牢记。丢失密码将无法找回数据！
            </a-alert>
            <a-form-item label="主密码">
                <a-input-password v-model="initPassword" ref="initPasswordRef" class="styled-password-input">
                    <template #prefix>
                        <icon-lock style="color: var(--color-text-3); font-size: 14px;" />
                    </template>
                </a-input-password>
            </a-form-item>
            <a-form-item label="确认密码">
                <a-input-password v-model="initPasswordConfirm" @keyup.enter="handleInit" class="styled-password-input">
                    <template #prefix>
                        <icon-lock style="color: var(--color-text-3); font-size: 14px;" />
                    </template>
                </a-input-password>
            </a-form-item>
            <div style="display: flex; justify-content: flex-end; gap: 12px; margin-top: 20px;">
                <a-button @click="handleExit">退出</a-button>
                <a-button type="primary" @click="handleInit">初始化</a-button>
            </div>
        </a-form>
    </a-modal>

    <!-- Unlock Modal -->
    <a-modal
      :visible="showUnlockModal"
      title="解锁钱包管理"
      :closable="false"
      :mask-closable="false"
      :esc-to-close="false"
      :footer="false"
      mask-animation-name="none"
      modal-animation-name="none"
      :mask-style="{ background: 'rgba(0, 0, 0, 0.6)' }"
    >
        <a-form layout="vertical">
            <a-form-item label="主密码">
                <a-input-password v-model="passwordInput" ref="passwordInputRef" @keyup.enter="handleUnlock" class="styled-password-input">
                    <template #prefix>
                        <icon-lock style="color: var(--color-text-3); font-size: 14px;" />
                    </template>
                </a-input-password>
            </a-form-item>
            <div style="display: flex; justify-content: flex-end; gap: 12px; margin-top: 20px;">
                <a-button @click="handleExit">退出</a-button>
                <a-button type="primary" @click="handleUnlock">解锁</a-button>
            </div>
        </a-form>
    </a-modal>

    <!-- Change Password Modal -->
    <a-modal v-model:visible="showChangePasswordModal" title="修改主密码" :footer="false" @before-open="() => {
        nextTick(() => {
            if (changePasswordOldRef.value) changePasswordOldRef.value.focus();
        });
    }">
        <a-form layout="vertical" :model="changePasswordForm">
            <a-form-item label="当前密码">
                <a-input-password v-model="changePasswordForm.oldPassword" ref="changePasswordOldRef" @keyup.enter="() => changePasswordNewRef.value?.focus()" class="styled-password-input">
                    <template #prefix>
                        <icon-lock style="color: var(--color-text-3); font-size: 14px;" />
                    </template>
                </a-input-password>
            </a-form-item>
            <a-form-item label="新密码">
                <a-input-password v-model="changePasswordForm.newPassword" ref="changePasswordNewRef" @keyup.enter="() => changePasswordConfirmRef.value?.focus()" class="styled-password-input">
                    <template #prefix>
                        <icon-lock style="color: var(--color-text-3); font-size: 14px;" />
                    </template>
                </a-input-password>
            </a-form-item>
            <a-form-item label="确认新密码">
                <a-input-password v-model="changePasswordForm.confirmPassword" ref="changePasswordConfirmRef" @keyup.enter="handleChangePassword" class="styled-password-input">
                    <template #prefix>
                        <icon-lock style="color: var(--color-text-3); font-size: 14px;" />
                    </template>
                </a-input-password>
            </a-form-item>
            <div style="text-align: right; margin-top: 20px;">
                <a-button type="primary" :loading="changePasswordLoading" @click="handleChangePassword">确认修改</a-button>
            </div>
        </a-form>
    </a-modal>

    <!-- Add Group Modal -->
    <a-modal v-model:visible="showAddGroupModal" title="新建分组" :footer="false">
        <a-form layout="vertical">
            <a-form-item label="父分组">
                <span>{{ getSelectedGroupName() }}</span>
            </a-form-item>
            <a-form-item label="分组名称">
                <a-input v-model="newGroupName" ref="newGroupNameRef" placeholder="请输入分组名称" @keyup.enter="handleAddGroup" />
            </a-form-item>
            <div style="text-align: right; margin-top: 20px;">
                <a-button @click="showAddGroupModal = false">取消</a-button>
                <a-button type="primary" style="margin-left: 8px;" @click="handleAddGroup">确认</a-button>
            </div>
        </a-form>
    </a-modal>

    <!-- Rename Group Modal -->
    <a-modal v-model:visible="showRenameGroupModal" title="重命名分组" @before-ok="handleRenameGroup" @opened="handleRenameGroupOpened">
        <a-form layout="vertical">
            <a-form-item label="分组名称">
                <a-input ref="renameGroupNameRef" v-model="renameGroupName" placeholder="请输入新的分组名称" @keyup.enter="handleRenameGroup" />
            </a-form-item>
        </a-form>
    </a-modal>

    <!-- Export Modal -->
    <a-modal v-model:visible="showExportModal" title="导出钱包数据" :footer="false">
        <a-alert type="warning" style="margin-bottom: 16px;">
            为保障数据安全，导出需要二次认证。请输入当前主密码解密私钥后导出。
        </a-alert>
        <a-form layout="vertical">
            <a-form-item label="主密码">
                <a-input-password ref="exportPasswordRef" v-model="exportPassword" @keyup.enter="confirmExport" class="styled-password-input">
                    <template #prefix>
                        <icon-lock style="color: var(--color-text-3); font-size: 14px;" />
                    </template>
                </a-input-password>
            </a-form-item>
            <div style="text-align: right; margin-top: 20px;">
                <a-button type="primary" :loading="isExporting" @click="confirmExport">确认导出</a-button>
            </div>
        </a-form>
    </a-modal>

    <!-- Add Wallet Modal -->
    <a-modal
      v-model:visible="showAddWalletModal"
      title="添加钱包"
      width="980px"
      :body-style="{ maxHeight: '72vh', overflowY: 'auto' }"
      :closable="!bulkGenerating"
      :mask-closable="false"
      :esc-to-close="false"
      :footer="bulkGenerating ? false : undefined"
      @before-ok="handleAddWallet"
      @before-open="async () => {
          // Auto-select chain type based on system group
          const key = selectedGroupKeys[0];
          if (key === 'SYSTEM_EVM') newWallet.chain_type = 'evm';
          else if (key === 'SYSTEM_SOLANA') newWallet.chain_type = 'solana';
          // Build group options for cascader
          await buildGroupOptions();
      }"
      @close="() => {
          if (bulkGenerating) {
              return false;
          }
      }"
    >
        <a-tabs v-model:active-key="addWalletActiveTab">
            <a-tab-pane key="import" title="手动添加">
                <a-form layout="vertical" :model="newWallet">
                    <div class="add-wallet-two-col">
                        <div class="add-wallet-col">
                            <a-form-item label="链类型">
                                <a-select v-model="newWallet.chain_type">
                                    <a-option value="evm">EVM</a-option>
                                    <a-option value="solana">Solana</a-option>
                                </a-select>
                            </a-form-item>
                         
                           
                            <a-form-item label="组名称" required>
                                <a-cascader 
                                    v-model="newWallet.group_path" 
                                    v-model:input-value="newWallet.group_search"
                                    :options="groupOptions" 
                                    :field-names="cascaderFieldNames"
                                    placeholder="请选择组名称"
                                    @change="handleGroupChange"
                                    allow-clear
                                    allow-search
                                    path-mode
                                    check-strictly
                                    :style="{ width: '100%' }"
                                />
                            </a-form-item>
                            <a-form-item label="钱包名称 (可选)">
                                <a-input v-model="newWallet.name" />
                            </a-form-item>
                             <!-- <a-form-item label="地址">
                                <a-input v-model="newWallet.address" disabled placeholder="自动生成并保存" />
                            </a-form-item> -->
                            
                        </div>

                        <div class="add-wallet-col">
                             <a-form-item label="导入方式">
                                <a-radio-group v-model="importKeyMode" type="button">
                                    <a-radio value="private_key">私钥</a-radio>
                                    <a-radio value="mnemonic">助记词</a-radio>
                                </a-radio-group>
                            </a-form-item>
                            
                           
                            <a-form-item label="新增子分组 (可选)">
                                <div style="display: flex; gap: 8px; width: 100%;">
                                    <a-input v-model="newWallet.new_group_name" placeholder="将创建在当前所选节点下" />
                                    <a-button type="outline" @click="handleCreateGroupInAddWallet">新增</a-button>
                                </div>
                            </a-form-item>
                            <template v-if="importKeyMode === 'mnemonic'">
                                <div class="add-wallet-subgrid">
                                    <a-form-item label="钱包数量">
                                        <a-input-number v-model="mnemonicWalletCount" :min="1" :max="100000" style="width: 100%;" />
                                    </a-form-item>
                                    <a-form-item label="起始序号">
                                        <a-input-number v-model="mnemonicStartIndex" :min="0" :max="1000000" style="width: 100%;" />
                                    </a-form-item>
                                </div>
                            </template>
                            <a-form-item  v-if="importKeyMode === 'private_key'" label="备注 (可选)">
                                <a-input v-model="newWallet.remark" placeholder="请输入备注信息" />
                            </a-form-item>
                        </div>
                    </div>

                    <template v-if="importKeyMode === 'mnemonic'">
                        <a-form-item label="助记词" required>
                            <a-textarea v-model="newWallet.mnemonic" :auto-size="{ minRows: 3, maxRows: 6 }" placeholder="将加密存储" />
                        </a-form-item>
                    </template>

                    <template v-else>
                        <a-form-item label="私钥" required>
                            <a-input-password v-model="newWallet.private_key" placeholder="将加密存储" />
                        </a-form-item>
                    </template>
                    <template  v-if="importKeyMode === 'mnemonic'">
                         <a-form-item label="备注 (可选)">
                                <a-input v-model="newWallet.remark" placeholder="请输入备注信息" />
                            </a-form-item>
                    </template>
                </a-form>
            </a-tab-pane>
            <a-tab-pane key="bulk" title="批量生成">
                <a-form layout="vertical" :model="newWallet">
                    <div class="add-wallet-two-col">
                        <div class="add-wallet-col">
                            <a-form-item label="链类型">
                                <a-select v-model="newWallet.chain_type">
                                    <a-option value="evm">EVM</a-option>
                                    <a-option value="solana">Solana</a-option>
                                </a-select>
                            </a-form-item>
                            <a-form-item label="组名称" required>
                                <a-cascader 
                                    v-model="newWallet.group_path" 
                                    v-model:input-value="newWallet.group_search"
                                    :options="groupOptions" 
                                    :field-names="cascaderFieldNames"
                                    placeholder="请选择组名称"
                                    @change="handleGroupChange"
                                    allow-clear
                                    allow-search
                                    path-mode
                                    check-strictly
                                    :style="{ width: '100%' }"
                                />
                            </a-form-item>
                            <div class="add-wallet-subgrid">
                                <a-form-item label="词数">
                                    <a-select v-model="bulkWordCount">
                                        <a-option :value="12">12</a-option>
                                        <a-option :value="24">24</a-option>
                                    </a-select>
                                </a-form-item>
                                <a-form-item label="钱包数量">
                                    <a-input-number v-model="bulkWalletCount" :min="1" :max="100000" style="width: 100%;" />
                                </a-form-item>
                            </div>
                            <a-form-item label="名称前缀 (可选)">
                                <a-input v-model="newWallet.name" />
                            </a-form-item>
                        </div>

                        <div class="add-wallet-col">
                            <a-form-item label="生成模式">
                                <a-radio-group v-model="bulkMode" type="button">
                                    <a-radio value="same_mnemonic">同助记词</a-radio>
                                    <a-radio value="different_mnemonic">不同助记词</a-radio>
                                </a-radio-group>
                            </a-form-item>
                            <a-form-item label="新增子分组 (可选)">
                                <div style="display: flex; gap: 8px; width: 100%;">
                                    <a-input v-model="newWallet.new_group_name" placeholder="将创建在当前所选节点下" />
                                    <a-button type="outline" @click="handleCreateGroupInAddWallet">新增</a-button>
                                </div>
                            </a-form-item>
                            
                            <a-form-item v-if="bulkMode === 'same_mnemonic'" label="起始序号">
                                <a-input-number v-model="bulkStartIndex" :min="0" :max="1000000" style="width: 100%;" />
                            </a-form-item>
                            <a-form-item label="备注 (可选)">
                                <a-input v-model="newWallet.remark" placeholder="请输入备注信息" />
                            </a-form-item>
                        </div>
                    </div>

                    <a-alert v-if="bulkGenerating" type="info" title="正在生成并保存，请稍候..." show-icon />

                </a-form>
            </a-tab-pane>
            <a-tab-pane key="watch" title="仅地址">
                <a-form layout="vertical" :model="newWatchAddress">
                    <div class="add-wallet-two-col">
                        <div class="add-wallet-col">
                            <a-form-item label="链类型">
                                <a-select v-model="newWatchAddress.chain_type">
                                    <a-option value="evm">EVM</a-option>
                                    <a-option value="solana">Solana</a-option>
                                </a-select>
                            </a-form-item>
                            <a-form-item label="组名称" required>
                                <a-cascader
                                    v-model="newWatchAddress.group_path"
                                    v-model:input-value="newWatchAddress.group_search"
                                    :options="groupOptions"
                                    :field-names="cascaderFieldNames"
                                    placeholder="请选择组名称"
                                    @change="(value, selectedOptions) => {
                                        if (!value) {
                                            newWatchAddress.group_id = null;
                                            newWatchAddress.group_name = '';
                                            return;
                                        }
                                        const path = Array.isArray(value) ? value : [value];
                                        const lastValue = path[path.length - 1];
                                        newWatchAddress.group_id = parseDbId(lastValue);
                                        const firstValue = path[0];
                                        if (firstValue === 'SYSTEM_EVM') newWatchAddress.chain_type = 'evm';
                                        if (firstValue === 'SYSTEM_SOLANA') newWatchAddress.chain_type = 'solana';
                                        if (selectedOptions && selectedOptions.length > 0 && parseDbId(lastValue) !== null) {
                                            const lastOption = selectedOptions[selectedOptions.length - 1];
                                            newWatchAddress.group_name = lastOption.label;
                                        } else {
                                            newWatchAddress.group_name = '';
                                        }
                                    }"
                                    allow-clear
                                    allow-search
                                    path-mode
                                    check-strictly
                                    :style="{ width: '100%' }"
                                />
                            </a-form-item>
                            <a-form-item label="新增子分组 (可选)">
                                <div style="display: flex; gap: 8px; width: 100%;">
                                    <a-input v-model="newWatchAddress.new_group_name" placeholder="将创建在当前所选节点下" />
                                    <a-button type="outline" @click="async () => {
                                        try {
                                            if (!newWatchAddress.group_path || newWatchAddress.group_path.length === 0) {
                                                Message.warning('请先选择一个父节点');
                                                return;
                                            }
                                            const name = newWatchAddress.new_group_name?.trim();
                                            if (!name) {
                                                Message.warning('分组名称不能为空');
                                                return;
                                            }
                                            const path = Array.isArray(newWatchAddress.group_path) ? newWatchAddress.group_path : [];
                                            const firstValue = path[0];
                                            const chainType = firstValue === 'SYSTEM_EVM' ? 'evm' : (firstValue === 'SYSTEM_SOLANA' ? 'solana' : newWatchAddress.chain_type);
                                            const lastValue = path[path.length - 1];
                                            const parentId = parseDbId(lastValue);
                                            const newGroupId = await invoke('create_group', { request: { parent_id: parentId, name, chain_type: chainType } });
                                            await loadGroups();
                                            await buildGroupOptions();
                                            newWatchAddress.group_path = [...path, newGroupId];
                                            newWatchAddress.group_id = newGroupId;
                                            newWatchAddress.group_name = name;
                                            newWatchAddress.new_group_name = '';
                                            Message.success('创建分组成功');
                                        } catch (e) {
                                            Message.error('创建分组失败: ' + (e?.message || e));
                                        }
                                    }">新增</a-button>
                                </div>
                            </a-form-item>
                            <a-form-item label="名称前缀 (可选)">
                                <a-input v-model="newWatchAddress.name_prefix" placeholder="批量添加时自动命名，如：仅地址 #1" />
                            </a-form-item>
                            <a-form-item label="备注 (可选)">
                                <a-input v-model="newWatchAddress.remark" placeholder="请输入备注信息" />
                            </a-form-item>
                        </div>

                        <div class="add-wallet-col">
                            <a-form-item label="地址列表" required>
                                <a-textarea
                                    v-model="newWatchAddress.addresses_text"
                                    :auto-size="{ minRows: 12, maxRows: 12 }"
                                    placeholder="每行一个地址，支持批量粘贴&#10;示例：&#10;0x1234...&#10;0x5678...&#10;0xabcd..."
                                />
                            </a-form-item>
                            <a-form-item label="地址数量">
                                <a-tag color="blue">{{ (newWatchAddress.addresses_text || '').split('\n').filter(l => l.trim().length > 0).length }}</a-tag>
                                <span style="margin-left: 8px; color: var(--color-text-3); font-size: 12px;">每行一个地址</span>
                            </a-form-item>
                            
                        </div>
                    </div>

                    <a-alert type="info" title="提示：仅地址仅保存地址，可用于批量查询余额和转账（需手动导入私钥），不支持导出私钥" show-icon />
                </a-form>
            </a-tab-pane>
        </a-tabs>
    </a-modal>

    <!-- Edit Wallet Modal -->
    <a-modal v-model:visible="showEditWalletModal" title="编辑钱包" :footer="false" width="500px">
        <a-form layout="vertical">
            <a-form-item label="钱包名称">
                <a-input v-model="editingWallet.name" placeholder="请输入钱包名称" />
            </a-form-item>
            <a-form-item label="备注信息">
                <a-input v-model="editingWallet.remark" placeholder="请输入备注信息" />
            </a-form-item>
            <div style="text-align: right; margin-top: 20px;">
                <a-button @click="showEditWalletModal = false">取消</a-button>
                <a-button type="primary" style="margin-left: 8px;" @click="handleSaveWallet">保存</a-button>
            </div>
        </a-form>
    </a-modal>

    <!-- Edit Watch Address Modal -->
    <a-modal v-model:visible="showEditWatchAddressModal" title="编辑仅地址" :footer="false" width="500px">
        <a-form layout="vertical">
            <a-form-item label="名称">
                <a-input v-model="editingWatchAddress.name" placeholder="请输入名称" />
            </a-form-item>
            <a-form-item label="备注信息">
                <a-input v-model="editingWatchAddress.remark" placeholder="请输入备注信息" />
            </a-form-item>
            <div style="text-align: right; margin-top: 20px;">
                <a-button @click="showEditWatchAddressModal = false">取消</a-button>
                <a-button type="primary" style="margin-left: 8px;" @click="handleSaveWatchAddress">保存</a-button>
            </div>
        </a-form>
    </a-modal>

    <!-- Secret Reveal Modal -->
    <SecretRevealModal
      :visible="showSecretRevealModal"
      :sealedPrivateKey="currentSealedPrivateKey"
      :password="sessionPassword"
      :transportToken="transportToken"
      :transportAesKey="transportAesKey"
      :title="'安全复制 - ' + currentWalletName"
      @update:visible="showSecretRevealModal = $event"
    />

    <!-- Batch Import Modal -->
    <a-modal
      v-model:visible="showBatchImportModal"
      title="批量导入钱包"
      :footer="false"
      width="800px"
      :closable="!isImporting"
      :mask-closable="false"
      :esc-to-close="false"
    >
        <div v-if="!isImporting">
            <a-upload
                v-model:file-list="fileList"
                :multiple="false"
                accept=".xlsx,.xls,.csv"
                :show-upload-list="false"
                @change="handleFileUpload"
            >
                <a-button type="primary">
                    选择文件 (支持 Excel/CSV)
                </a-button>
            </a-upload>
            <div style="margin-top: 16px; font-size: 14px; color: #666;">
                请确保文件包含以下列：name (名称), chain_type (链类型，值为 evm 或 solana), group_name (组名称)
                <br>
                可选列：private_key (私钥), mnemonic (助记词)（二选一）
            </div>
            
            <div v-if="importedWallets.length > 0" style="margin-top: 20px;">
                <h4>预览导入数据 ({{ importedWallets.length }} 条)</h4>
                <a-table :data="importedWallets" :columns="importColumns" :pagination="false" style="margin-top: 10px;">
                    <template #private_key="{ record }">
                        {{ record.private_key ? '******' : '' }}
                    </template>
                    <template #mnemonic="{ record }">
                        {{ record.mnemonic ? '******' : '' }}
                    </template>
                </a-table>
                <div style="margin-top: 20px; text-align: right;">
                    <a-button @click="showBatchImportModal = false">取消</a-button>
                    <a-button type="primary" style="margin-left: 8px;" @click="startBatchImport">开始导入</a-button>
                </div>
            </div>
        </div>
        
        <div v-else>
            <a-progress :percent="importProgress" status="active" />
            <div style="margin-top: 20px; text-align: center;">
                正在导入... 请耐心等待
            </div>
        </div>
        
        <div v-if="importResult.success > 0 || importResult.failed > 0" style="margin-top: 20px;">
            <a-alert
                :type="importResult.failed > 0 ? 'warning' : 'success'"
                :title="`导入完成：成功 ${importResult.success} 条，失败 ${importResult.failed} 条`"
                show-icon
            />
            <div v-if="importResult.errors.length > 0" style="margin-top: 10px;">
                <h5>失败原因：</h5>
                <a-list bordered>
                    <a-list-item v-for="(error, index) in importResult.errors" :key="index">
                        {{ error }}
                    </a-list-item>
                </a-list>
            </div>
            <div style="margin-top: 20px; text-align: right;">
                <a-button type="primary" @click="closeBatchImportModal">关闭</a-button>
            </div>
        </div>
    </a-modal>
  </div>
</template>

<style scoped>
.wallet-manager-page {
    height: 100vh;
    padding-top: 40px;
    display: flex;
    flex-direction: column;
    background: var(--color-bg-1);
    color: var(--color-text-1);
}
.main-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
}
.sidebar {
    width: 250px;
    border-right: 1px solid var(--color-border);
    padding: 10px;
    display: flex;
    flex-direction: column;
    background: var(--color-bg-2);
    overflow: hidden;
}

.sidebar-tree-wrapper {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: none;
    -ms-overflow-style: none;
}

.sidebar-tree-wrapper::-webkit-scrollbar {
    display: none;
}
.sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
    padding: 0 10px;
}

.add-wallet-two-col {
    display: flex;
    gap: 16px;
    align-items: flex-start;
}

.add-wallet-col {
    flex: 1;
    min-width: 0;
}

.add-wallet-subgrid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
}

@media (max-width: 900px) {
    .add-wallet-two-col {
        flex-direction: column;
    }
    .add-wallet-subgrid {
        grid-template-columns: 1fr;
    }
}
.content {
    flex: 1;
    padding: 10px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
}
.toolbar {
    flex-shrink: 0;
    margin-bottom: 10px;
}
.wallet-table {
    flex: 1;
    overflow: hidden;
    min-height: 0;
}

/* 确保树节点标题区域填满整行 */
:deep(.arco-tree-node-title) {
    width: 100%;
    flex: 1; /* 关键：允许子元素撑开 */
    padding: 0; /* 可选：如果你希望点击区域包含原本的内边距，需去除父级内边距 */
}

/* 确保文本包裹层（如果有）也填满 */
:deep(.arco-tree-node-title-text) {
    width: 100%;
}

/* 让 Dropdown 触发器撑满所有可用空间 */
.tree-node-dropdown {
    display: block;
    width: 100%;
    height: 100%;
}

/* 内容区域样式，可在此处把 padding 加回来以保持视觉一致 */
.tree-node-content {
    width: 100%;
    height: 100%;
    padding: 0 4px; /* 恢复视觉上的内边距 */
    display: flex;
    align-items: center;
    transition: color 0.2s, font-weight 0.2s, background-color 0.2s;
}

.tree-node-content.is-selected {
    color: rgb(var(--primary-6));
    font-weight: 600;
    background-color: var(--color-fill-2);
}

/* 美化密码输入框样式 */
.styled-password-input {
    border-radius: 8px;
    transition: all 0.3s ease;
}

.styled-password-input:hover {
    border-color: rgb(var(--primary-5));
    box-shadow: 0 0 0 2px rgba(var(--primary-6), 0.1);
}

.styled-password-input:focus-within {
    border-color: rgb(var(--primary-6));
    box-shadow: 0 0 0 3px rgba(var(--primary-6), 0.15);
}

:deep(.arco-input-password) {
    border-radius: 8px;
    padding-left: 8px;
}

:deep(.arco-input-password-wrapper) {
    border-radius: 8px;
    overflow: hidden;
}

/* 分组操作按钮样式 */
.group-action-btn {
    border-radius: 6px;
    transition: all 0.2s ease;
}

.group-action-btn:hover {
    background-color: var(--color-fill-2) !important;
}

.delete-btn {
    color: #ff4d4f;
}

.delete-btn:hover {
    color: #ff7875 !important;
    background-color: rgba(255, 77, 79, 0.1) !important;
}

.add-btn {
    color: #52c41a;
}

.add-btn:hover {
    color: #73d13d !important;
    background-color: rgba(82, 196, 26, 0.1) !important;
}

.group-action-btn:disabled {
    color: var(--color-text-3) !important;
    background-color: transparent !important;
}

/* 关键操作遮罩层样式 */
.critical-operation-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 9999;
    backdrop-filter: blur(4px);
}

.critical-operation-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 40px 60px;
    background: var(--color-bg-2);
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    border: 1px solid var(--color-border);
}

.critical-operation-text {
    font-size: 18px;
    font-weight: 500;
    color: var(--color-text-1);
    margin: 0;
}

.critical-operation-hint {
    font-size: 14px;
    color: var(--color-text-3);
    margin: 0;
}

/* 钱包管理加载遮罩 */
.wallet-loading-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--color-bg-1);
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 9999;
}

/* 全局加载遮罩 - 覆盖模态框 */
.wallet-global-loading-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: var(--color-bg-1);
    display: none;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 10000;
    font-size: 16px;
    color: var(--color-text-2);
}

.wallet-global-loading-overlay .loading-spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--color-fill-3);
    border-top-color: rgb(var(--primary-6));
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

.wallet-global-loading-overlay p {
    margin-top: 16px;
    margin: 16px 0 0 0;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

/* 树形结构文件夹图标美化 */
.tree-folder-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    margin-right: 6px;
    font-size: 18px;
    color: var(--color-text-3);
    transition: all 0.2s ease;
    flex-shrink: 0;
}

.tree-folder-icon.is-system {
    color: rgb(var(--primary-5));
}

.tree-folder-icon.is-expanded {
    color: rgb(var(--primary-6));
    transform: scale(1.1);
}

/* 树节点容器优化 */
:deep(.arco-tree-node) {
    padding: 2px 0;
    border-radius: 6px;
    margin-bottom: 2px;
}

:deep(.arco-tree-node:hover) {
    background-color: var(--color-fill-2);
}

:deep(.arco-tree-node-selected) {
    background-color: rgba(var(--primary-6), 0.1) !important;
}

/* 树节点标题区域 */
:deep(.arco-tree-node-title) {
    width: 100%;
    flex: 1;
    padding: 4px 8px;
    border-radius: 6px;
}

/* 节点标签样式 */
.node-label {
    font-size: 14px;
    color: var(--color-text-1);
    transition: color 0.2s ease;
}

.tree-node-content.is-selected .node-label {
    color: rgb(var(--primary-6));
    font-weight: 500;
}

/* 树节点图标和内容对齐 */
:deep(.arco-tree-node-icon) {
    margin-right: 0;
}

/* 展开/折叠指示器样式 */
:deep(.arco-tree-switcher) {
    width: 20px;
    height: 20px;
}

:deep(.arco-tree-switcher-icon) {
    color: var(--color-text-3);
}

:deep(.arco-tree-switcher:hover .arco-tree-switcher-icon) {
    color: rgb(var(--primary-6));
}

/* 表格汇总信息样式 */
.table-summary {
    position: absolute;
    bottom: 8px;
    left: 10px;
    right: 10px;
    display: flex;
    align-items: center;
    padding: 6px 12px;
    font-size: 13px;
    color: var(--color-text-2);
    gap: 8px;
    z-index: 1;
    pointer-events: none;
}

.summary-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0px 10px;
    background: rgba(var(--color-fill-2), 0.85);
    border-radius: 4px;
    backdrop-filter: blur(4px);
}

.summary-label {
    color: var(--color-text-3);
}

.summary-value {
    font-weight: 600;
    color: var(--color-text-1);
    min-width: 24px;
    text-align: center;
}

.summary-value.evm {
    color: rgb(var(--primary-6));
}

.summary-value.solana {
    color: #722ed1;
}

.summary-value.success {
    color: #52c41a;
}

</style>
