<script setup>
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from 'vue';
import { Icon } from '@iconify/vue';

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['update:visible', 'complete']);

const currentStep = ref(0);
const spotlightPosition = ref({ top: 0, left: 0, width: 0, height: 0 });
const tooltipPosition = ref({ top: 0, left: 0, showBelow: true });

const steps = [
  {
    target: 'chain-selector',
    title: '选择区块链',
    content: '点击此处选择您要操作的区块链网络，支持多条主流公链',
    icon: 'mdi:web'
  },
  {
    target: 'table-section',
    title: '表格数据操作',
    content: '点击右侧按钮可导入文件或手动录入钱包，也可下载模板。导入数据后可在表格中查看和管理',
    icon: 'mdi:table'
  },
  {
    target: 'config-section',
    title: '配置转账参数',
    content: '在此区域配置发送模式、Gas限制、Gas价格等参数，配置完成后点击下方"执行转账"按钮开始运行',
    icon: 'mdi:cog'
  },
  {
    target: 'menu-button',
    title: '功能菜单',
    content: '点击打开右侧功能菜单，可进行钱包录入、文件导入、数据筛选等操作',
    icon: 'mdi:menu'
  }
];

const totalSteps = computed(() => steps.length);
const isLastStep = computed(() => currentStep.value === totalSteps.value - 1);
const progressPercent = computed(() => ((currentStep.value + 1) / totalSteps.value) * 100);

function getTargetElement() {
  const targets = {
    'chain-selector': document.getElementById('chain-selector'),
    'table-section': document.getElementById('table-section'),
    'config-section': document.getElementById('config-section'),
    'menu-button': document.getElementById('menu-button')
  };
  return targets[steps[currentStep.value].target];
}

function updateTooltipPosition() {
  const element = getTargetElement();
  if (element) {
    const rect = element.getBoundingClientRect();
    const tooltipHeight = 180;
    const tooltipWidth = 300;
    const padding = 12;
    
    spotlightPosition.value = {
      top: rect.top,
      left: rect.left,
      width: rect.width,
      height: rect.height
    };
    
    const showBelow = rect.bottom + padding + tooltipHeight <= window.innerHeight;
    let top = showBelow ? rect.bottom + padding : rect.top - tooltipHeight - padding;
    let left = rect.left;
    
    if (left + tooltipWidth > window.innerWidth) {
      left = window.innerWidth - tooltipWidth - padding;
    }
    
    if (left < padding) {
      left = padding;
    }
    
    tooltipPosition.value = {
      top: top,
      left: left,
      showBelow: showBelow
    };
  } else {
    setTimeout(() => {
      updateTooltipPosition();
    }, 100);
  }
}

function nextStep() {
  if (isLastStep.value) {
    completeGuide();
  } else {
    currentStep.value++;
    nextTick(() => {
      updateTooltipPosition();
    });
  }
}

function prevStep() {
  if (currentStep.value > 0) {
    currentStep.value--;
    nextTick(() => {
      updateTooltipPosition();
    });
  }
}

function skipGuide() {
  localStorage.setItem('transfer_guide_completed', 'true');
  closeGuide();
}

function completeGuide() {
  localStorage.setItem('transfer_guide_completed', 'true');
  emit('complete');
  closeGuide();
}

function closeGuide() {
  emit('update:visible', false);
}

function handleResize() {
  if (props.visible) {
    updateTooltipPosition();
  }
}

function handleScroll() {
  if (props.visible) {
    updateTooltipPosition();
  }
}

onMounted(() => {
  if (props.visible) {
    setTimeout(() => {
      updateTooltipPosition();
    }, 300);
  }
  window.addEventListener('resize', handleResize);
  window.addEventListener('scroll', handleScroll, true);
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize);
  window.removeEventListener('scroll', handleScroll, true);
});

watch(() => props.visible, (newVal) => {
  if (newVal) {
    currentStep.value = 0;
    setTimeout(() => {
      updateTooltipPosition();
    }, 300);
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="guide-fade">
      <div v-if="visible" class="guide-overlay">
        <div class="guide-mask" @click="skipGuide"></div>
        
        <div class="guide-spotlight" :style="{
          top: spotlightPosition.top + 'px',
          left: spotlightPosition.left + 'px',
          width: spotlightPosition.width + 'px',
          height: spotlightPosition.height + 'px'
        }"></div>
        
        <div class="guide-tooltip" :style="{
          top: tooltipPosition.top + 'px',
          left: tooltipPosition.left + 'px'
        }">
          <div class="guide-header">
            <div class="guide-progress">
              <div class="guide-progress-bar" :style="{ width: progressPercent + '%' }"></div>
            </div>
            <div class="guide-title-row">
              <div class="guide-icon">
                <Icon :icon="steps[currentStep].icon" style="font-size: 20px" />
              </div>
              <span class="guide-title">{{ steps[currentStep].title }}</span>
              <span class="guide-step">{{ currentStep + 1 }} / {{ totalSteps }}</span>
            </div>
          </div>
          
          <div class="guide-content">
            <p>{{ steps[currentStep].content }}</p>
          </div>
          
          <div class="guide-actions">
            <a-button size="small" @click="skipGuide">跳过引导</a-button>
            <div class="guide-nav-buttons">
              <a-button 
                v-if="currentStep > 0" 
                size="small" 
                @click="prevStep"
                class="guide-prev-btn"
              >
                <template #icon><Icon icon="mdi:chevron-left" /></template>
                上一步
              </a-button>
              <a-button 
                type="primary" 
                size="small" 
                @click="nextStep"
                class="guide-next-btn"
              >
                {{ isLastStep ? '完成' : '下一步' }}
                <template #icon v-if="!isLastStep"><Icon icon="mdi:chevron-right" /></template>
              </a-button>
            </div>
          </div>
        </div>
        
        <div class="guide-arrow" :class="{ 'arrow-up': tooltipPosition.showBelow, 'arrow-down': !tooltipPosition.showBelow }" :style="{
          top: (tooltipPosition.showBelow ? spotlightPosition.top + spotlightPosition.height : spotlightPosition.top - 12) + 'px',
          left: (spotlightPosition.left + 20) + 'px'
        }"></div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.guide-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 10000;
  pointer-events: none;
}

.guide-mask {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.6);
  pointer-events: auto;
}

.guide-spotlight {
  position: fixed;
  border: 3px solid #165dff;
  border-radius: 8px;
  z-index: 10001;
  box-shadow: 
    0 0 0 9999px rgba(0, 0, 0, 0.6),
    0 0 0 4px rgba(255, 255, 255, 0.3),
    0 0 30px rgba(22, 93, 255, 0.8),
    0 0 60px rgba(22, 93, 255, 0.4),
    inset 0 0 20px rgba(22, 93, 255, 0.1);
  transition: all 0.3s ease;
  pointer-events: none;
  animation: spotlightPulse 1.5s ease-in-out infinite;
}

@keyframes spotlightPulse {
  0%, 100% {
    border-color: #165dff;
    box-shadow: 
      0 0 0 9999px rgba(0, 0, 0, 0.6),
      0 0 0 4px rgba(255, 255, 255, 0.3),
      0 0 30px rgba(22, 93, 255, 0.8),
      0 0 60px rgba(22, 93, 255, 0.4),
      inset 0 0 20px rgba(22, 93, 255, 0.1);
    transform: scale(1);
  }
  50% {
    border-color: #4086ff;
    box-shadow: 
      0 0 0 9999px rgba(0, 0, 0, 0.6),
      0 0 0 6px rgba(255, 255, 255, 0.5),
      0 0 50px rgba(64, 134, 255, 1),
      0 0 100px rgba(64, 134, 255, 0.6),
      inset 0 0 30px rgba(64, 134, 255, 0.2);
    transform: scale(1.02);
  }
}

.guide-tooltip {
  position: fixed;
  background: #ffffff;
  border-radius: 12px;
  padding: 16px;
  min-width: 280px;
  max-width: 320px;
  box-shadow: 
    0 10px 40px rgba(0, 0, 0, 0.3),
    0 0 0 1px rgba(22, 93, 255, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 1);
  pointer-events: auto;
  z-index: 10001;
  transition: all 0.3s ease;
}

.guide-header {
  margin-bottom: 12px;
}

.guide-progress {
  height: 4px;
  background: #e5e6eb;
  border-radius: 2px;
  margin-bottom: 12px;
  overflow: hidden;
}

.guide-progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #165dff, #4086ff);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.guide-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.guide-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: linear-gradient(135deg, #e8f1ff, #f0f5ff);
  border-radius: 8px;
  color: #165dff;
}

.guide-title {
  font-size: 16px;
  font-weight: 600;
  color: #1d2129;
  flex: 1;
}

.guide-step {
  font-size: 12px;
  color: #86909c;
  background: #f2f3f5;
  padding: 2px 8px;
  border-radius: 10px;
}

.guide-content {
  margin-bottom: 16px;
}

.guide-content p {
  font-size: 14px;
  color: #4e5969;
  line-height: 1.6;
  margin: 0;
}

.guide-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.guide-nav-buttons {
  display: flex;
  gap: 8px;
}

.guide-prev-btn,
.guide-next-btn {
  min-width: 80px;
}

.guide-arrow {
  position: fixed;
  width: 0;
  height: 0;
  border-left: 10px solid transparent;
  border-right: 10px solid transparent;
  z-index: 10001;
  transition: all 0.3s ease;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

.guide-arrow.arrow-up {
  border-top: 12px solid #ffffff;
}

.guide-arrow.arrow-down {
  border-bottom: 12px solid #ffffff;
  border-top: none;
}

.guide-fade-enter-active,
.guide-fade-leave-active {
  transition: opacity 0.3s ease;
}

.guide-fade-enter-from,
.guide-fade-leave-to {
  opacity: 0;
}
</style>
