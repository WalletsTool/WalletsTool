<script setup>
import { ref } from 'vue';
import { Message } from '@arco-design/web-vue';
import {
  IconBook,
  IconCode,
  IconCopy,
  IconCheck
} from '@arco-design/web-vue/es/icon';

const activeCategory = ref('wallet');
const searchQuery = ref('');
const copiedMethod = ref('');

// API 分类
const categories = [
  { id: 'wallet', name: '钱包连接', icon: 'wallet' },
  { id: 'sign', name: '签名操作', icon: 'edit' },
  { id: 'tx', name: '交易操作', icon: 'send' },
  { id: 'browser', name: '浏览器操作', icon: 'browser' },
  { id: 'utils', name: '工具函数', icon: 'tool' },
];

// API 方法定义
const apiMethods = {
  wallet: [
    {
      name: 'connectMetaMask',
      description: '连接 MetaMask 钱包',
      params: [{ name: 'options', type: 'Object', desc: '连接选项，如 { expectedChainId: "0x1" }' }],
      returns: 'Promise<string>',
      example: "await connectMetaMask({ expectedChainId: '0x1' })"
    },
    {
      name: 'connectOKXWallet',
      description: '连接 OKX Wallet',
      params: [{ name: 'options', type: 'Object', desc: '连接选项，如 { chainId: "0x1" }' }],
      returns: 'Promise<string>',
      example: "await connectOKXWallet({ chainId: '0x1' })"
    },
    {
      name: 'switchNetwork',
      description: '切换钱包网络',
      params: [{ name: 'chainId', type: 'string', desc: '目标网络ID (十六进制字符串)' }],
      returns: 'Promise<boolean>',
      example: "await switchNetwork('0x1')"
    },
    {
      name: 'getCurrentAddress',
      description: '获取当前连接的钱包地址',
      params: [],
      returns: 'Promise<string>',
      example: "const address = await getCurrentAddress()"
    },
  ],
  sign: [
    {
      name: 'signMessage',
      description: '签名消息',
      params: [
        { name: 'message', type: 'string', desc: '要签名的消息' },
        { name: 'options', type: 'Object', desc: '可选参数' }
      ],
      returns: 'Promise<string>',
      example: "await signMessage('Hello, World!')"
    },
    {
      name: 'signTransaction',
      description: '签名交易',
      params: [{ name: 'tx', type: 'Object', desc: '交易对象 { to, value, data }' }],
      returns: 'Promise<string>',
      example: "await signTransaction({ to: '0x...', value: '0.1', data: '0x...' })"
    },
    {
      name: 'signTypedData',
      description: '签名 EIP-712 类型数据',
      params: [
        { name: 'domain', type: 'Object', desc: 'EIP-712 Domain' },
        { name: 'types', type: 'Object', desc: '类型定义' },
        { name: 'value', type: 'Object', desc: '要签名的值' }
      ],
      returns: 'Promise<string>',
      example: "await signTypedData(domain, types, value)"
    },
  ],
  tx: [
    {
      name: 'sendNativeTransfer',
      description: '发送原生币转账',
      params: [
        { name: 'to', type: 'string', desc: '接收地址' },
        { name: 'amount', type: 'string', desc: '金额 (如 "0.1 ETH")' },
        { name: 'options', type: 'Object', desc: '可选参数' }
      ],
      returns: 'Promise<Object>',
      example: "await sendNativeTransfer('0x...', '0.1 ETH')"
    },
    {
      name: 'approveToken',
      description: 'ERC-20 Token 授权',
      params: [
        { name: 'tokenAddress', type: 'string', desc: 'Token 合约地址' },
        { name: 'spender', type: 'string', desc: '授权给谁' },
        { name: 'amount', type: 'string', desc: '授权数量 (如 "1000 USDC")' }
      ],
      returns: 'Promise<Object>',
      example: "await approveToken('0x...', '0x...', '1000 USDC')"
    },
    {
      name: 'transferToken',
      description: 'ERC-20 Token 转账',
      params: [
        { name: 'tokenAddress', type: 'string', desc: 'Token 合约地址' },
        { name: 'to', type: 'string', desc: '接收地址' },
        { name: 'amount', type: 'string', desc: '转账数量' }
      ],
      returns: 'Promise<Object>',
      example: "await transferToken('0x...', '0x...', '100 USDC')"
    },
    {
      name: 'waitForTransaction',
      description: '等待交易确认',
      params: [
        { name: 'txHash', type: 'string', desc: '交易哈希' },
        { name: 'confirmations', type: 'number', desc: '确认数 (默认1)' }
      ],
      returns: 'Promise<Object>',
      example: "await waitForTransaction('0x...', 1)"
    },
  ],
  browser: [
    {
      name: 'waitForSelector',
      description: '等待元素出现',
      params: [
        { name: 'selector', type: 'string', desc: 'CSS 选择器' },
        { name: 'timeout', type: 'number', desc: '超时时间 (毫秒, 默认30000)' }
      ],
      returns: 'Promise<Element>',
      example: "await waitForSelector('.submit-btn', 10000)"
    },
    {
      name: 'waitForSelectorHidden',
      description: '等待元素消失',
      params: [
        { name: 'selector', type: 'string', desc: 'CSS 选择器' },
        { name: 'timeout', type: 'number', desc: '超时时间 (毫秒, 默认30000)' }
      ],
      returns: 'Promise<boolean>',
      example: "await waitForSelectorHidden('.loading')"
    },
    {
      name: 'waitForPageLoad',
      description: '等待页面加载',
      params: [
        { name: 'url', type: 'string', desc: '期望的 URL' },
        { name: 'timeout', type: 'number', desc: '超时时间 (毫秒, 默认60000)' }
      ],
      returns: 'Promise<boolean>',
      example: "await waitForPageLoad('https://...')"
    },
    {
      name: 'clickElement',
      description: '点击元素',
      params: [
        { name: 'selector', type: 'string', desc: 'CSS 选择器' },
        { name: 'options', type: 'Object', desc: '点击选项' }
      ],
      returns: 'Promise<void>',
      example: "await clickElement('.submit-btn')"
    },
    {
      name: 'inputText',
      description: '输入文本',
      params: [
        { name: 'selector', type: 'string', desc: 'CSS 选择器' },
        { name: 'text', type: 'string', desc: '输入文本' },
        { name: 'options', type: 'Object', desc: '输入选项' }
      ],
      returns: 'Promise<void>',
      example: "await inputText('#address', '0x...')"
    },
    {
      name: 'getElementText',
      description: '获取元素文本',
      params: [{ name: 'selector', type: 'string', desc: 'CSS 选择器' }],
      returns: 'Promise<string>',
      example: "const text = await getElementText('.balance')"
    },
    {
      name: 'executeScript',
      description: '执行任意 JavaScript',
      params: [
        { name: 'fn', type: 'string|Function', desc: 'JavaScript 代码或函数' },
        { name: 'args', type: '...any', desc: '参数' }
      ],
      returns: 'Promise<any>',
      example: "await executeScript(() => document.title)"
    },
  ],
  utils: [
    {
      name: 'randomDelay',
      description: '随机延迟',
      params: [
        { name: 'minMs', type: 'number', desc: '最小延迟 (毫秒, 默认1000)' },
        { name: 'maxMs', type: 'number', desc: '最大延迟 (毫秒, 默认3000)' }
      ],
      returns: 'Promise<void>',
      example: "await randomDelay(2000, 5000)"
    },
    {
      name: 'humanLikeClick',
      description: '模拟人类点击 (带随机偏移)',
      params: [
        { name: 'selector', type: 'string', desc: 'CSS 选择器' },
        { name: 'options', type: 'Object', desc: '点击选项' }
      ],
      returns: 'Promise<void>',
      example: "await humanLikeClick('.submit-btn')"
    },
    {
      name: 'getBalance',
      description: '获取钱包余额',
      params: [{ name: 'tokenAddress', type: 'string', desc: 'Token 地址 (空则为主币)' }],
      returns: 'Promise<string>',
      example: "const balance = await getBalance() // 主币\nconst usdc = await getBalance('0x...') // Token"
    },
    {
      name: 'getGasPrices',
      description: '获取 Gas 价格',
      params: [],
      returns: 'Promise<Object>',
      example: "const { slow, standard, fast } = await getGasPrices()"
    },
    {
      name: 'log',
      description: '输出日志',
      params: [
        { name: 'level', type: 'string', desc: '日志级别 (info, warn, error, success)' },
        { name: 'message', type: 'string', desc: '日志消息' }
      ],
      returns: 'void',
      example: "log('info', '开始执行...')"
    },
  ],
};

// 过滤方法
const filteredMethods = ref(apiMethods.wallet);

const filterMethods = () => {
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    const allMethods = Object.values(apiMethods).flat();
    filteredMethods.value = allMethods.filter(m =>
      m.name.toLowerCase().includes(query) ||
      m.description.toLowerCase().includes(query)
    );
  } else {
    filteredMethods.value = apiMethods[activeCategory.value] || [];
  }
};

const changeCategory = (categoryId) => {
  activeCategory.value = categoryId;
  searchQuery.value = '';
  filteredMethods.value = apiMethods[categoryId];
};

// 复制代码
const copyCode = async (text) => {
  try {
    await navigator.clipboard.writeText(text);
    copiedMethod.value = text;
    Message.success('已复制到剪贴板');
    setTimeout(() => {
      copiedMethod.value = '';
    }, 2000);
  } catch (e) {
    Message.error('复制失败');
  }
};

// 插入代码到编辑器
const emit = defineEmits(['insert-code']);
const insertCode = (example) => {
  emit('insert-code', example);
};
</script>

<template>
  <div class="api-helper">
    <div class="helper-header">
      <div class="header-title">
        <icon-code />
        <span>API 参考</span>
      </div>
    </div>

    <!-- 搜索 -->
    <div class="search-box">
      <a-input-search
        v-model="searchQuery"
        placeholder="搜索 API..."
        @search="filterMethods"
        @clear="filterMethods"
        allow-clear
      />
    </div>

    <!-- 分类标签 -->
    <div class="category-tabs">
      <div
        v-for="cat in categories"
        :key="cat.id"
        class="category-tab"
        :class="{ active: activeCategory === cat.id && !searchQuery }"
        @click="changeCategory(cat.id)"
      >
        {{ cat.name }}
      </div>
    </div>

    <!-- API 列表 -->
    <div class="api-list">
      <div v-for="method in filteredMethods" :key="method.name" class="api-item">
        <div class="api-header" @click="method.expanded = !method.expanded">
          <div class="api-name">
            <code>{{ method.name }}</code>
          </div>
          <div class="api-desc">{{ method.description }}</div>
        </div>

        <div class="api-details" v-if="method.expanded">
          <!-- 参数 -->
          <div class="detail-section">
            <div class="section-title">参数</div>
            <div class="param-list">
              <div v-for="param in method.params" :key="param.name" class="param-item">
                <code class="param-name">{{ param.name }}</code>
                <span class="param-type">{{ param.type }}</span>
                <span class="param-desc">- {{ param.desc }}</span>
              </div>
              <div v-if="method.params.length === 0" class="no-params">无参数</div>
            </div>
          </div>

          <!-- 返回值 -->
          <div class="detail-section">
            <div class="section-title">返回值</div>
            <code class="return-type">{{ method.returns }}</code>
          </div>

          <!-- 示例 -->
          <div class="detail-section">
            <div class="section-title">示例</div>
            <div class="code-example">
              <div class="code-header">
                <span>JavaScript</span>
                <a-space>
                  <a-button size="mini" @click="insertCode(method.example)">
                    插入代码
                  </a-button>
                  <a-button size="mini" @click="copyCode(method.example)">
                    <template #icon><icon-copy /></template>
                  </a-button>
                </a-space>
              </div>
              <pre><code>{{ method.example }}</code></pre>
            </div>
          </div>
        </div>
      </div>

      <div v-if="filteredMethods.length === 0" class="no-results">
        <p>未找到相关 API</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.api-helper {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-2);
  border-radius: 8px;
  border: 1px solid var(--color-border);
  overflow: hidden;
}

.helper-header {
  padding: 12px 15px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-3);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}

.search-box {
  padding: 10px 15px;
  border-bottom: 1px solid var(--color-border);
}

.category-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 10px;
  border-bottom: 1px solid var(--color-border);
}

.category-tab {
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--color-text-3);
}

.category-tab:hover {
  background: var(--color-fill-2);
}

.category-tab.active {
  background: rgba(var(--primary-6), 0.1);
  color: rgb(var(--primary-6));
}

.api-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.api-item {
  margin-bottom: 10px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.api-header {
  padding: 10px 12px;
  cursor: pointer;
  transition: all 0.2s;
  background: var(--color-bg-1);
}

.api-header:hover {
  background: var(--color-fill-2);
}

.api-name code {
  font-size: 14px;
  font-weight: 600;
  color: rgb(var(--primary-6));
  background: rgba(var(--primary-6), 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

.api-desc {
  font-size: 12px;
  color: var(--color-text-3);
  margin-top: 4px;
}

.api-details {
  padding: 12px;
  background: var(--color-bg-2);
  border-top: 1px solid var(--color-border);
}

.detail-section {
  margin-bottom: 12px;
}

.detail-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-4);
  text-transform: uppercase;
  margin-bottom: 6px;
}

.param-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.param-item {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 12px;
}

.param-name {
  background: rgba(var(--primary-6), 0.1);
  color: rgb(var(--primary-6));
  padding: 1px 4px;
  border-radius: 3px;
}

.param-type {
  color: var(--color-text-3);
  font-size: 11px;
}

.param-desc {
  color: var(--color-text-3);
}

.no-params {
  font-size: 12px;
  color: var(--color-text-4);
}

.return-type {
  font-size: 12px;
  color: rgb(var(--success-6));
  background: rgba(var(--success-6), 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

.code-example {
  background: var(--color-bg-1);
  border-radius: 6px;
  overflow: hidden;
  border: 1px solid var(--color-border);
}

.code-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 10px;
  background: var(--color-fill-2);
  font-size: 11px;
  color: var(--color-text-3);
}

.code-example pre {
  margin: 0;
  padding: 10px;
  overflow-x: auto;
  font-family: 'Fira Code', monospace;
  font-size: 12px;
  line-height: 1.5;
  color: var(--color-text-1);
}

.no-results {
  text-align: center;
  padding: 30px;
  color: var(--color-text-4);
}
</style>
