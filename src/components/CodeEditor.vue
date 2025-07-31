<template>
  <div class="code-editor">
    <div class="editor-container">
      <!-- 行号区域 -->
      <div 
        ref="lineNumbersRef" 
        class="line-numbers"
        @scroll="handleLineNumberScroll"
      >
        <div 
          v-for="(line, index) in lineNumbers" 
          :key="index"
          :class="[
            'line-number',
            { 
              'selected': selectedLines.includes(index + 1),
              'error': errorLines.includes(index + 1)
            }
          ]"
          @click="toggleLineSelection(index + 1)"
        >
          {{ index + 1 }}
        </div>
      </div>
      
      <!-- 内容区域 -->
      <div class="content-area">
        <textarea
          ref="textareaRef"
          :value="modelValue"
          :placeholder="placeholder"
          class="content-textarea"
          @input="handleInput"
          @scroll="handleTextareaScroll"
          @keydown="handleKeydown"
          spellcheck="false"
        ></textarea>
        
        <!-- 错误高亮层 -->
        <div 
          ref="highlightRef"
          class="highlight-layer"
        >
          <div 
            v-for="(line, index) in contentLines" 
            :key="index"
            :class="[
              'highlight-line',
              { 'error-highlight': errorLines.includes(index + 1) }
            ]"
          >
            <!-- 只显示空格占位，不显示文字内容 -->
            &nbsp;
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted } from 'vue'

// Props
const props = defineProps({
  modelValue: {
    type: String,
    default: ''
  },
  placeholder: {
    type: String,
    default: ''
  },
  errorLines: {
    type: Array,
    default: () => []
  }
})

// Emits
const emit = defineEmits(['update:modelValue', 'scroll', 'line-select'])

// Refs
const textareaRef = ref(null)
const lineNumbersRef = ref(null)
const highlightRef = ref(null)

// 响应式数据
const selectedLines = ref([])
const isScrolling = ref(false)

// 计算属性
const contentLines = computed(() => {
  return props.modelValue.split('\n')
})

const lineNumbers = computed(() => {
  const lines = contentLines.value.length
  return Array.from({ length: Math.max(lines, 20) }, (_, i) => i + 1)
})

// 方法
const handleInput = (event) => {
  emit('update:modelValue', event.target.value)
}

const toggleLineSelection = (lineNumber) => {
  const index = selectedLines.value.indexOf(lineNumber)
  if (index > -1) {
    selectedLines.value.splice(index, 1)
  } else {
    selectedLines.value.push(lineNumber)
  }
  
  // 发出行号选择事件
  emit('line-select', {
    lineNumber,
    selectedLines: [...selectedLines.value]
  })
}

const handleTextareaScroll = () => {
  if (isScrolling.value) return
  
  isScrolling.value = true
  const textarea = textareaRef.value
  const lineNumbers = lineNumbersRef.value
  const highlight = highlightRef.value
  
  if (lineNumbers) {
    lineNumbers.scrollTop = textarea.scrollTop
  }
  if (highlight) {
    highlight.scrollTop = textarea.scrollTop
    highlight.scrollLeft = textarea.scrollLeft
  }
  
  // 发出滚动事件，传递滚动信息
  emit('scroll', {
    scrollTop: textarea.scrollTop,
    scrollLeft: textarea.scrollLeft
  })
  
  nextTick(() => {
    isScrolling.value = false
  })
}

const handleLineNumberScroll = () => {
  if (isScrolling.value) return
  
  isScrolling.value = true
  const textarea = textareaRef.value
  const lineNumbers = lineNumbersRef.value
  const highlight = highlightRef.value
  
  if (textarea) {
    textarea.scrollTop = lineNumbers.scrollTop
  }
  if (highlight) {
    highlight.scrollTop = lineNumbers.scrollTop
  }
  
  nextTick(() => {
    isScrolling.value = false
  })
}

const handleKeydown = (event) => {
  // 处理Tab键缩进
  if (event.key === 'Tab') {
    event.preventDefault()
    const textarea = event.target
    const start = textarea.selectionStart
    const end = textarea.selectionEnd
    const value = textarea.value
    
    // 插入两个空格代替Tab
    const newValue = value.substring(0, start) + '  ' + value.substring(end)
    emit('update:modelValue', newValue)
    
    nextTick(() => {
      textarea.selectionStart = textarea.selectionEnd = start + 2
    })
  }
}

// 监听器
watch(() => props.modelValue, () => {
  nextTick(() => {
    syncScroll()
  })
})

watch(() => props.errorLines, () => {
  nextTick(() => {
    syncScroll()
  })
})

// 同步滚动位置
const syncScroll = (scrollInfo) => {
  if (!textareaRef.value || !lineNumbersRef.value || !highlightRef.value) return
  
  const textarea = textareaRef.value
  const lineNumbers = lineNumbersRef.value
  const highlight = highlightRef.value
  
  if (scrollInfo) {
    // 外部调用，同步到指定位置
    isScrolling.value = true
    textarea.scrollTop = scrollInfo.scrollTop
    textarea.scrollLeft = scrollInfo.scrollLeft
    lineNumbers.scrollTop = scrollInfo.scrollTop
    highlight.scrollTop = scrollInfo.scrollTop
    highlight.scrollLeft = scrollInfo.scrollLeft
    nextTick(() => {
      isScrolling.value = false
    })
  } else {
    // 内部调用，同步当前位置
    lineNumbers.scrollTop = textarea.scrollTop
    highlight.scrollTop = textarea.scrollTop
    highlight.scrollLeft = textarea.scrollLeft
  }
}

// 同步行号选择
const syncLineSelection = (lineSelectionInfo) => {
  if (lineSelectionInfo && lineSelectionInfo.selectedLines) {
    selectedLines.value = [...lineSelectionInfo.selectedLines]
  }
}

// 暴露方法给父组件
defineExpose({
  syncScroll,
  syncLineSelection
})

// 生命周期
onMounted(() => {
  nextTick(() => {
    syncScroll()
  })
})
</script>

<style scoped>
.code-editor {
  width: 100%;
  height: 100%;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  overflow: hidden;
  background: #fff;
}

:root[data-theme="dark"] .code-editor {
  border-color: #424242;
  background: #1e1e1e;
}

.editor-container {
  display: flex;
  height: 100%;
  position: relative;
}

.line-numbers {
  width: 50px;
  background: #f5f5f5;
  border-right: 1px solid #e0e0e0;
  overflow: hidden;
  user-select: none;
  flex-shrink: 0;
}

:root[data-theme="dark"] .line-numbers {
  background: #2d2d2d;
  border-right-color: #424242;
}

.line-number {
  height: 20px;
  line-height: 20px;
  padding: 0 8px;
  text-align: right;
  font-size: 12px;
  color: #999;
  cursor: pointer;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

:root[data-theme="dark"] .line-number {
  color: #666;
}

.line-number:hover {
  background: #e6f7ff;
}

:root[data-theme="dark"] .line-number:hover {
  background: #1f3a8a;
}

.line-number.selected {
  background: #1890ff;
  color: #fff;
}

.line-number.error {
  background: #fff2e8;
  color: #fa8c16;
}

:root[data-theme="dark"] .line-number.error {
  background: #2a1810;
  color: #fa8c16;
}

.content-area {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.content-textarea {
  width: 100%;
  height: 100%;
  border: none;
  outline: none;
  resize: none;
  padding: 0 12px;
  font-size: 14px;
  line-height: 20px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  background: transparent;
  color: #262626;
  position: relative;
  z-index: 2;
  white-space: pre-wrap;
}

:root[data-theme="dark"] .content-textarea {
  color: #e6e6e6;
}

.content-textarea::placeholder {
  color: #bfbfbf;
}

:root[data-theme="dark"] .content-textarea::placeholder {
  color: #666;
}

.highlight-layer {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  padding: 0 12px;
  font-size: 14px;
  line-height: 20px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  pointer-events: none;
  overflow: hidden;
  z-index: 1;
  white-space: pre;
}

.highlight-line {
  height: 20px;
  line-height: 20px;
}

.highlight-line.error-highlight {
  background: rgba(255, 193, 7, 0.2);
}

:root[data-theme="dark"] .highlight-line.error-highlight {
  background: rgba(255, 193, 7, 0.1);
}

/* 滚动条样式 */
.content-textarea::-webkit-scrollbar,
.line-numbers::-webkit-scrollbar,
.highlight-layer::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.content-textarea::-webkit-scrollbar-track,
.line-numbers::-webkit-scrollbar-track,
.highlight-layer::-webkit-scrollbar-track {
  background: #f1f1f1;
}

:root[data-theme="dark"] .content-textarea::-webkit-scrollbar-track,
:root[data-theme="dark"] .line-numbers::-webkit-scrollbar-track,
:root[data-theme="dark"] .highlight-layer::-webkit-scrollbar-track {
  background: #2d2d2d;
}

.content-textarea::-webkit-scrollbar-thumb,
.line-numbers::-webkit-scrollbar-thumb,
.highlight-layer::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 4px;
}

:root[data-theme="dark"] .content-textarea::-webkit-scrollbar-thumb,
:root[data-theme="dark"] .line-numbers::-webkit-scrollbar-thumb,
:root[data-theme="dark"] .highlight-layer::-webkit-scrollbar-thumb {
  background: #555;
}

.content-textarea::-webkit-scrollbar-thumb:hover,
.line-numbers::-webkit-scrollbar-thumb:hover,
.highlight-layer::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}

:root[data-theme="dark"] .content-textarea::-webkit-scrollbar-thumb:hover,
:root[data-theme="dark"] .line-numbers::-webkit-scrollbar-thumb:hover,
:root[data-theme="dark"] .highlight-layer::-webkit-scrollbar-thumb:hover {
  background: #777;
}
</style>