<script setup>
import { ref, onMounted, reactive, watch, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Message, Notification, Modal } from '@arco-design/web-vue';
import { IconPlus, IconDelete } from '@arco-design/web-vue/es/icon';
import { useRouter } from 'vue-router';
import * as XLSX from 'xlsx';
import TitleBar from '@/components/TitleBar.vue';
import VirtualScrollerTable from '@/components/VirtualScrollerTable.vue';
import { downloadWithDialog, openDirectory } from '@/utils/downloadWithDialog';
import { sealSecret, openSealedSecret } from '@/utils/secretCrypto';

const router = useRouter();
const windowTitle = ref('钱包管理');

const handleBeforeClose = () => {
  router.push({ name: 'main' });
};

const isUnlocked = ref(false);
const showInitModal = ref(false);
const showUnlockModal = ref(false);
const passwordInput = ref('');
const passwordInputRef = ref(null);
const initPassword = ref('');
const initPasswordConfirm = ref('');
const sessionPassword = ref('');

// 监听弹窗显示，自动聚焦输入框
watch(showUnlockModal, (newVal) => {
  if (newVal && passwordInputRef.value) {
    // 在下一个DOM更新周期聚焦，确保弹窗已渲染
    nextTick(() => {
      passwordInputRef.value.focus();
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
// Rename Group
const showRenameGroupModal = ref(false);
const renameGroupName = ref('');
const renamingGroupId = ref(null);

// Modals
const showAddGroupModal = ref(false);
const newGroupName = ref('');
const newGroupNameRef = ref(null);
const showAddWalletModal = ref(false);

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
const importKeyMode = ref('mnemonic');
const mnemonicWalletCount = ref(1);
const mnemonicStartIndex = ref(0);

const bulkMode = ref('same_mnemonic');
const bulkWordCount = ref(12);
const bulkWalletCount = ref(10);
const bulkStartIndex = ref(0);
const bulkGenerating = ref(false);
const bulkResults = ref([]);

const maskSecret = (val) => {
  if (!val) return '';
  const s = String(val);
  if (s.length <= 12) return s;
  return `${s.slice(0, 6)}...${s.slice(-6)}`;
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
    const plain = await openSealedSecret(sealed, sessionPassword.value);
    await copyToClipboard(plain);
  } catch (e) {
    Message.error('解密失败: ' + (e?.message || e));
  }
};

const resetAddWalletForm = () => {
  addWalletActiveTab.value = 'import';
  importKeyMode.value = 'mnemonic';
  mnemonicWalletCount.value = 1;
  mnemonicStartIndex.value = 0;
  bulkMode.value = 'same_mnemonic';
  bulkWordCount.value = 12;
  bulkWalletCount.value = 10;
  bulkStartIndex.value = 0;
  bulkGenerating.value = false;
  bulkResults.value = [];

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

const walletColumns = [
  { title: '序号', align: 'center', width: 53, slotName: 'index' },
  { title: '名称', align: 'center', dataIndex: 'name', width: 120, ellipsis: true, tooltip: true },
  { title: '地址', align: 'center', dataIndex: 'address', width: 220, ellipsis: true, tooltip: true },
  { title: '类型', align: 'center', dataIndex: 'chain_type', width: 80, ellipsis: true, tooltip: true },
  { title: '备注', align: 'center', dataIndex: 'remark', ellipsis: true, tooltip: true },
  { title: '私钥状态', align: 'center', slotName: 'private_key_status', width: 90, ellipsis: true, tooltip: true },
  { title: '操作', align: 'center', slotName: 'optional', width: 55, ellipsis: true, tooltip: true },
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
  try {
    await invoke('init_wallet_manager_tables');
    const isSet = await invoke('is_password_set');
    if (!isSet) {
      showInitModal.value = true;
    } else {
      showUnlockModal.value = true;
    }
  } catch (e) {
    Message.error('初始化失败: ' + e);
  }
});

  const handleUnlock = async () => {
    try {
      const success = await invoke('verify_password', { request: { password: passwordInput.value } });
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
      Message.error('解锁失败: ' + e);
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
      await invoke('init_password', { request: { password: initPassword.value } });
      sessionPassword.value = initPassword.value;
      isUnlocked.value = true;
      showInitModal.value = false;
      loadGroups();
      loadWallets();
    } catch (e) {
      Message.error('初始化失败: ' + e);
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
        return false;
      }
      
      const selectedKey = selectedGroupKeys.value[0];
      if (!selectedKey) {
          Message.warning('请选择一个生态分组或现有分组');
          return false;
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
              return false;
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
      return false;
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
        content: '确定要删除该分组吗？删除后该分组下的子分组将被一并删除，该分组下的钱包将变为未分组状态。',
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

  const handleAddWallet = async () => {
    try {
      if (!newWallet.chain_type || !Array.isArray(newWallet.group_path) || newWallet.group_path.length === 0) {
        Message.error('请选择链类型与分组');
        return;
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
          const sealedMnemonic = await sealSecret(mnemonic, sessionPassword.value);
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
              password: sessionPassword.value
            }
          });
        } else {
          const privateKey = (newWallet.private_key || '').trim();
          if (!privateKey) {
            Message.error('请输入私钥');
            return false;
          }
          const sealedPrivateKey = await sealSecret(privateKey, sessionPassword.value);
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
              password: sessionPassword.value
            }
          });
        }

        await loadWallets();
        Message.success('添加成功');
        showAddWalletModal.value = false;
        return true;
      }

      bulkGenerating.value = true;
      bulkResults.value = [];
      const mode = bulkMode.value === 'same_mnemonic' ? 'generate_same_mnemonic' : 'generate_different_mnemonic';
      const count = Number(bulkWalletCount.value) || 1;
      const startIndex = Number(bulkStartIndex.value) || 0;
      const wordCount = Number(bulkWordCount.value) || 12;

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
          password: sessionPassword.value
        }
      });

      const list = Array.isArray(created) ? created : [];
      bulkResults.value = await Promise.all(
        list.map(async (w) => {
          let mnemonicMasked = '';
          let privateKeyMasked = '';
          try {
            if (w?.sealed_mnemonic) {
              mnemonicMasked = maskSecret(await openSealedSecret(w.sealed_mnemonic, sessionPassword.value));
            }
          } catch (_) {}
          try {
            if (w?.sealed_private_key) {
              privateKeyMasked = maskSecret(await openSealedSecret(w.sealed_private_key, sessionPassword.value));
            }
          } catch (_) {}
          return { ...w, mnemonic_masked: mnemonicMasked, private_key_masked: privateKeyMasked };
        })
      );
      bulkGenerating.value = false;
      await loadWallets();
      Message.success(`已生成并保存 ${bulkResults.value.length} 个钱包`);
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
      const sealedMnemonic = mnemonic ? await sealSecret(mnemonic, sessionPassword.value) : null;
      const sealedPrivateKey = privateKey ? await sealSecret(privateKey, sessionPassword.value) : null;
      
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
          password: sessionPassword.value
        }
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
    ['name', 'chain_type', 'group_name', 'private_key', 'mnemonic', 'remark'],
    ['示例钱包1', 'evm', '个人钱包', '0x(64位十六进制私钥)', '', ''],
    ['示例钱包2', 'evm', '个人钱包', '', 'work man father ...', ''],
    ['示例钱包3', 'solana', 'Solana钱包', '(base58 keypair)', '', '']
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
</script>

<template>
  <TitleBar :title="windowTitle" :custom-close="true" @before-close="handleBeforeClose" />
  <div class="wallet-manager-page">
    <div v-if="isUnlocked" class="main-layout">
        <div class="sidebar">
            <div class="sidebar-header">
                <span>分组列表</span>
                <div>
                    <a-button type="text" size="mini" status="danger" @click="handleDeleteGroup" :disabled="!selectedGroupKeys[0] || ['SYSTEM_EVM', 'SYSTEM_SOLANA'].includes(selectedGroupKeys[0])">
                        <template #icon><icon-delete style="font-size: 18px;" /></template>
                    </a-button>
                    <a-button type="text" size="mini" status="success" @click="showAddGroupModal = true">
                        <template #icon><icon-plus style="font-size: 18px;"/></template>
                    </a-button>
                </div>
            </div>
            <a-tree 
                :data="groups"
                :field-names="{ key: 'id', title: 'name', children: 'children' }"
                block-node
                v-model:selected-keys="selectedGroupKeys"
                v-model:expanded-keys="expandedKeys"
                @select="onGroupSelect"
                @node-click="onNodeClick"
            >
                <template #title="node">
                    <a-dropdown trigger="contextMenu" alignPoint class="tree-node-dropdown">
                        <div class="tree-node-content" :class="{ 'is-selected': selectedGroupKeys.includes(node.id) }" @click.stop="onNodeClick(node)">{{ node.name }}</div>
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
        <div class="content">
            <div class="toolbar">
                <a-button type="primary" @click="showAddWalletModal = true">添加钱包</a-button>
                <a-button style="margin-left: 10px;" type="primary" status="success" @click="showBatchImportModal = true">批量导入</a-button>
                <a-button style="margin-left: 10px;" type="outline" @click="downloadTemplate">下载模板</a-button>
            </div>
            <VirtualScrollerTable
              class="wallet-table"
              :columns="walletColumns"
              :data="wallets"
              :row-selection="walletRowSelection"
              :selected-keys="selectedWalletIds"
              @update:selected-keys="selectedWalletIds = $event"
              row-key="id"
              height="100%"
              page-type="wallet_manager"
              :empty-data="wallets.length === 0"
            >
              <template #private_key_status="{ record }">
                <a-tag v-if="record.has_private_key" color="green">已加密</a-tag>
                <a-tag v-else color="arcoblue">无私钥</a-tag>
              </template>
              <template #optional="{ record }">
                <a-button
                  type="text"
                  size="mini"
                  status="danger"
                  @click.stop="handleDeleteWallet(record)"
                >
                  <template #icon><icon-delete style="font-size: 16px;" /></template>
                </a-button>
              </template>
            </VirtualScrollerTable>
        </div>
    </div>

    <!-- Init Modal -->
    <a-modal v-model:visible="showInitModal" title="设置主密码" :closable="false" :mask-closable="false" :footer="false">
        <a-form layout="vertical">
            <a-alert type="warning" style="margin-bottom: 10px;">
                此密码用于加密存储所有私钥，请务必牢记。丢失密码将无法找回数据！
            </a-alert>
            <a-form-item label="主密码">
                <a-input-password v-model="initPassword" />
            </a-form-item>
            <a-form-item label="确认密码">
                <a-input-password v-model="initPasswordConfirm" />
            </a-form-item>
            <div style="text-align: right; margin-top: 20px;">
                <a-button type="primary" @click="handleInit">初始化</a-button>
            </div>
        </a-form>
    </a-modal>

    <!-- Unlock Modal -->
    <a-modal v-model:visible="showUnlockModal" title="解锁钱包管理" :closable="false" :mask-closable="false" :footer="false">
        <a-form layout="vertical">
            <a-form-item label="主密码">
                <a-input-password v-model="passwordInput" ref="passwordInputRef" @keyup.enter="handleUnlock" />
            </a-form-item>
            <div style="text-align: right; margin-top: 20px;">
                <a-button type="primary" @click="handleUnlock">解锁</a-button>
            </div>
        </a-form>
    </a-modal>

    <!-- Add Group Modal -->
    <a-modal v-model:visible="showAddGroupModal" title="新建分组" @before-ok="handleAddGroup">
        <a-form layout="vertical">
            <a-form-item label="父分组">
                <span>{{ getSelectedGroupName() }}</span>
            </a-form-item>
            <a-form-item label="分组名称">
                <a-input v-model="newGroupName" ref="newGroupNameRef" placeholder="请输入分组名称" />
            </a-form-item>
        </a-form>
    </a-modal>

    <!-- Rename Group Modal -->
    <a-modal v-model:visible="showRenameGroupModal" title="重命名分组" @before-ok="handleRenameGroup">
        <a-form layout="vertical">
            <a-form-item label="分组名称">
                <a-input v-model="renameGroupName" placeholder="请输入新的分组名称" />
            </a-form-item>
        </a-form>
    </a-modal>

    <!-- Add Wallet Modal -->
    <a-modal v-model:visible="showAddWalletModal" title="添加钱包" width="980px" :body-style="{ maxHeight: '72vh', overflowY: 'auto' }" @before-ok="handleAddWallet" @before-open="async () => {
        // Auto-select chain type based on system group
        const key = selectedGroupKeys[0];
        if (key === 'SYSTEM_EVM') newWallet.chain_type = 'evm';
        else if (key === 'SYSTEM_SOLANA') newWallet.chain_type = 'solana';
        // Build group options for cascader
        await buildGroupOptions();
    }">
        <a-tabs v-model:active-key="addWalletActiveTab">
            <a-tab-pane key="import" title="导入/添加">
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
                            <a-form-item label="新增子分组 (可选)">
                                <div style="display: flex; gap: 8px; width: 100%;">
                                    <a-input v-model="newWallet.new_group_name" placeholder="将创建在当前所选节点下" />
                                    <a-button type="outline" @click="handleCreateGroupInAddWallet">新增</a-button>
                                </div>
                            </a-form-item>
                            <a-form-item label="钱包名称 (可选)">
                                <a-input v-model="newWallet.name" />
                            </a-form-item>
                        </div>

                        <div class="add-wallet-col">
                            <a-form-item label="导入方式">
                                <a-radio-group v-model="importKeyMode" type="button">
                                    <a-radio value="mnemonic">助记词</a-radio>
                                    <a-radio value="private_key">私钥</a-radio>
                                </a-radio-group>
                            </a-form-item>

                            <template v-if="importKeyMode === 'mnemonic'">
                                <div class="add-wallet-subgrid">
                                    <a-form-item label="钱包数量">
                                        <a-input-number v-model="mnemonicWalletCount" :min="1" :max="100" style="width: 100%;" />
                                    </a-form-item>
                                    <a-form-item label="起始序号">
                                        <a-input-number v-model="mnemonicStartIndex" :min="0" :max="1000000" style="width: 100%;" />
                                    </a-form-item>
                                </div>
                            </template>

                            <a-form-item label="地址">
                                <a-input v-model="newWallet.address" disabled placeholder="自动生成并保存" />
                            </a-form-item>
                            <a-form-item label="备注 (可选)">
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
                            <a-form-item label="新增子分组 (可选)">
                                <div style="display: flex; gap: 8px; width: 100%;">
                                    <a-input v-model="newWallet.new_group_name" placeholder="将创建在当前所选节点下" />
                                    <a-button type="outline" @click="handleCreateGroupInAddWallet">新增</a-button>
                                </div>
                            </a-form-item>
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
                            <div class="add-wallet-subgrid">
                                <a-form-item label="词数">
                                    <a-select v-model="bulkWordCount">
                                        <a-option :value="12">12</a-option>
                                        <a-option :value="24">24</a-option>
                                    </a-select>
                                </a-form-item>
                                <a-form-item label="钱包数量">
                                    <a-input-number v-model="bulkWalletCount" :min="1" :max="100" style="width: 100%;" />
                                </a-form-item>
                            </div>
                            <a-form-item v-if="bulkMode === 'same_mnemonic'" label="起始序号">
                                <a-input-number v-model="bulkStartIndex" :min="0" :max="1000000" style="width: 100%;" />
                            </a-form-item>
                            <a-form-item label="备注 (可选)">
                                <a-input v-model="newWallet.remark" placeholder="请输入备注信息" />
                            </a-form-item>
                        </div>
                    </div>

                    <a-alert v-if="bulkGenerating" type="info" title="正在生成并保存，请稍候..." show-icon />

                    <div v-if="bulkResults.length > 0" style="margin-top: 12px;">
                        <a-table :data="bulkResults" :pagination="false" size="small">
                            <a-table-column title="地址" data-index="address" :width="260" ellipsis tooltip />
                            <a-table-column title="序号" data-index="mnemonic_index" :width="70" />
                            <a-table-column title="助记词" :width="220">
                                <template #cell="{ record }">
                                    <span>{{ record.mnemonic_masked }}</span>
                                    <a-button v-if="record.sealed_mnemonic" type="text" size="mini" @click="copySealedSecret(record.sealed_mnemonic)">复制</a-button>
                                </template>
                            </a-table-column>
                            <a-table-column title="私钥" :width="220">
                                <template #cell="{ record }">
                                    <span>{{ record.private_key_masked }}</span>
                                    <a-button v-if="record.sealed_private_key" type="text" size="mini" @click="copySealedSecret(record.sealed_private_key)">复制</a-button>
                                </template>
                            </a-table-column>
                        </a-table>
                    </div>
                </a-form>
            </a-tab-pane>
        </a-tabs>
    </a-modal>

    <!-- Batch Import Modal -->
    <a-modal v-model:visible="showBatchImportModal" title="批量导入钱包" :footer="false" width="800px">
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
    padding: 20px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
.toolbar {
    flex-shrink: 0;
    margin-bottom: 20px;
}
.wallet-table {
    flex: 1;
    overflow: hidden;
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
</style>
