<script setup name="TransferConfigForm">
import { ref, computed } from 'vue';
import { Icon } from '@iconify/vue';

const props = defineProps({
  form: {
    type: Object,
    required: true,
  },
  formRef: {
    type: Object,
    default: null,
  },
  enableMultiThread: {
    type: [String, Boolean],
    default: false,
  },
  threadCount: {
    type: Number,
    default: 1,
  },
  multiWindowCount: {
    type: Number,
    default: 1,
  },
});

const emit = defineEmits([
  'update:enable-multi-thread',
  'update:thread-count',
  'update:multi-window-count',
  'open-multi-window',
]);

const localForm = computed({
  get: () => props.form,
  set: (value) => {},
});

const localEnableMultiThread = computed({
  get: () => props.enableMultiThread,
  set: (value) => emit('update:enable-multi-thread', value),
});

const localThreadCount = computed({
  get: () => props.threadCount,
  set: (value) => emit('update:thread-count', value),
});

const localMultiWindowCount = computed({
  get: () => props.multiWindowCount,
  set: (value) => emit('update:multi-window-count', value),
});

const debouncedOpenMultipleWindow = ref(null);
</script>

<template>
  <div style="display: flex; padding-top: 10px; flex-shrink: 0">
    <a-form
      ref="formRef"
      :model="form"
      :style="{ width: '100%' }"
      layout="horizontal"
      label-align="left"
    >
      <a-row style="display: flex; gap: 5px">
        <div style="flex: 9">
          <a-form-item
            field="send_type"
            label="发送模式"
            :label-col-props="{ span: 6 }"
          >
            <a-radio-group v-model="form.send_type" type="button">
              <a-radio value="1">全部</a-radio>
              <a-radio value="2">指定数值</a-radio>
              <a-radio value="3">范围随机</a-radio>
              <a-radio value="4">剩余随机</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item
            v-if="form.send_type === '2'"
            field="amount_from"
            label="数量来源"
            tooltip="如果选择表格数据则应导入带有转账数量的表格数据"
            :label-col-props="{ span: 6 }"
          >
            <a-radio-group v-model="form.amount_from" type="button">
              <a-radio value="1">表格数据</a-radio>
              <a-radio value="2">自定义</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item
            v-if="form.send_type === '2' && form.amount_from === '2'"
            field="send_count"
            label="发送数量"
            :label-col-props="{ span: 6 }"
          >
            <a-input v-model="form.send_count" />
          </a-form-item>
          <a-form-item
            v-if="form.send_type === '3' || form.send_type === '4'"
            field="send_count_scope"
            :label="form.send_type === '3' ? '发送数量从' : '剩余数量从'"
            :label-col-props="{ span: 6 }"
          >
            <a-space>
              <a-input
                v-model="form.send_min_count"
                placeholder="最小"
                style="width: 66px"
              />
              <span style="margin: 0 8px">至</span>
              <a-input
                v-model="form.send_max_count"
                placeholder="最大"
                style="width: 85px"
              />
              <span style="margin-left: 10px">范围内随机生成</span>
            </a-space>
          </a-form-item>
          <a-form-item
            v-if="form.send_type === '3' || form.send_type === '4'"
            field="amount_precision"
            label="金额精度"
            tooltip="金额小数点位数"
            :label-col-props="{ span: 6 }"
          >
            <a-input v-model="form.amount_precision" />
          </a-form-item>
        </div>
        <a-divider direction="vertical" style="height: 100%; margin: 0" />
        <div style="flex: 9">
          <a-form-item
            field="limit_type"
            label="Gas Limit 配置"
            :label-col-props="{ span: 7 }"
          >
            <a-radio-group v-model="form.limit_type" type="button">
              <a-radio value="1">自动获取</a-radio>
              <a-radio value="2">指定数值</a-radio>
              <a-radio value="3">范围随机</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item
            v-if="form.limit_type === '2'"
            field="limit_count"
            label="Gas Limit 数量"
            :label-col-props="{ span: 7 }"
          >
            <a-input v-model="form.limit_count" />
          </a-form-item>
          <a-form-item
            v-if="form.limit_type === '3'"
            field="limit_count_scope"
            label="Gas Limit 范围"
            :label-col-props="{ span: 7 }"
          >
            <a-space>
              <a-input
                v-model="form.limit_min_count"
                placeholder="最小"
                style="width: 90px"
              />
              <span style="margin: 0 8px">至</span>
              <a-input
                v-model="form.limit_max_count"
                placeholder="最大"
                style="width: 90px"
              />
            </a-space>
          </a-form-item>
          <a-form-item
            field="gas_price_type"
            label="Gas Price 方式"
            :label-col-props="{ span: 7 }"
          >
            <a-radio-group v-model="form.gas_price_type" type="button">
              <a-radio value="1">自动获取</a-radio>
              <a-radio value="2">指定数值</a-radio>
              <a-radio value="3">加价抢跑</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item
            v-if="form.gas_price_type === '2'"
            field="gas_price"
            label="Gas Price"
            :label-col-props="{ span: 7 }"
          >
            <a-input v-model="form.gas_price" />
          </a-form-item>
          <a-form-item
            v-if="form.gas_price_type === '3'"
            field="gas_price_rate"
            label="提高比例"
            :label-col-props="{ span: 7 }"
          >
            <a-input v-model="form.gas_price_rate">
              <template #append>%</template>
            </a-input>
          </a-form-item>
          <a-form-item
            v-if="form.gas_price_type === '1' || form.gas_price_type === '3'"
            field="max_gas_price"
            label="最大 Gas Price"
            :label-col-props="{ span: 7 }"
          >
            <a-input
              v-model="form.max_gas_price"
              placeholder="为空时则不设置上限（单位：Gwei）"
            />
          </a-form-item>
        </div>
        <a-divider direction="vertical" style="height: 100%; margin: 0" />
        <div style="flex: 8">
          <a-form-item label="" :label-col-props="{ span: 0 }">
            <a-space :size="8" align="center" style="display: flex; align-items: center">
              <a-switch
                v-model="localEnableMultiThread"
                checked-value="1"
                unchecked-value="0"
                style="margin-right: 10px"
              >
                <template #checked>多线程</template>
                <template #unchecked>单线程</template>
              </a-switch>
              <template
                v-if="localEnableMultiThread === '1' || localEnableMultiThread === true"
              >
                <span>线程数</span>
                <a-input-number
                  v-model="localThreadCount"
                  :min="1"
                  :max="999"
                  :step="1"
                  :default-value="1"
                  size="small"
                  style="width: 90px; margin-left: 10px"
                />
                <a-tag
                  v-if="threadCount > 90"
                  color="#ff4d4f"
                  style="font-size: 10px; margin-left: 10px"
                >狂暴</a-tag
                >
              </template>
              <template v-else>
                <span>时间间隔</span>
                <a-input
                  v-model="form.min_interval"
                  placeholder="最小"
                  style="width: 55px; margin-left: 10px"
                />
                <span style="margin: 0 8px">至</span>
                <a-input
                  v-model="form.max_interval"
                  placeholder="最大"
                  style="width: 55px; margin-right: 10px"
                />
                秒
              </template>
            </a-space>
          </a-form-item>
          <a-form-item
            field="error_retry"
            label="失败自动重试"
            tooltip="开启失败自动重试功能后，存在多次转账风险，请谨慎使用"
            :label-col-props="{ span: 8 }"
            :wrapper-col-props="{ span: 16 }"
          >
            <a-switch
              v-model="form.error_retry"
              checked-value="1"
              unchecked-value="0"
            >
              <template #checked>开启</template>
              <template #unchecked>关闭</template>
            </a-switch>
          </a-form-item>
          <a-form-item
            field="multi_window"
            label="窗口多开"
            tooltip="如果需要同时打开多个转账窗口，可设置窗口数量进行同配置窗口多开"
            :label-col-props="{ span: 7 }"
            :wrapper-col-props="{ span: 16 }"
          >
            <a-input-group style="width: 100%">
              <a-input-number
                v-model="localMultiWindowCount"
                :min="1"
                :max="9"
                :step="1"
                :default-value="1"
                placeholder="窗口数"
                style="width: 50%"
              />
              <a-button status="success" @click="debouncedOpenMultipleWindow?.()">
                <template #icon>
                  <Icon icon="mdi:content-copy" />
                </template>
              </a-button>
            </a-input-group>
          </a-form-item>
        </div>
      </a-row>
    </a-form>
  </div>
</template>

<style scoped>
.arco-form-item {
  padding: 5px 10px;
  margin-bottom: 8px;
}

.container :deep(.arco-form-item-label-col) {
  margin-bottom: 0;
}

.container :deep(.arco-form-item-wrapper-col) {
  flex: 1;
}

.container :deep(.arco-form-item) {
  margin-bottom: 8px;
  padding: 4px 10px;
}

.container :deep(.arco-form-item-label) {
  line-height: 32px;
}

.arco-radio-button.arco-radio-checked {
  color: #ffffff;
  background-color: #165dff;
}
</style>
