import { ref } from 'vue';
import { Notification } from '@arco-design/web-vue';

export function useValidation(options = {}) {
  const { form, formRef } = options;

  function validateForm() {
    return new Promise((resolve, reject) => {
      if (
          checkSendType() &&
          checkPrecision() &&
          checkDelay() &&
          checkGasLimit() &&
          checkGasPrice()
      ) {
        resolve();
      } else {
        reject();
      }
    });
  }

  function checkSendType() {
    if (form.send_type === '1') {
      return true;
    } else if (form.send_type === '2') {
      const bool =
          /^\d+(\.\d+)?$/.test(form.send_count) &&
          Number(form.send_count) > 0;
      if (form.amount_from === '2' && !bool) {
        Notification.error({
          content: '发送数量必须为数字且大于0',
          position: 'topLeft',
        });
        formRef.value.setFields({
          send_count: {
            status: 'error',
            message: '发送数量必须为数字且大于0',
          },
        });
        return false;
      } else {
        return true;
      }
    } else if (form.send_type === '3' || form.send_type === '4') {
      const bool =
          /^\d+(\.\d+)?$/.test(form.send_min_count) &&
          /^\d+(\.\d+)?$/.test(form.send_max_count) &&
          Number(form.send_min_count) > 0 &&
          Number(form.send_max_count) > 0;
      if (!bool) {
        const msg =
            form.send_type === '4'
                ? '剩余数量必须为数字且大于0'
                : '发送数量必须为数字且大于0';
        Notification.error(msg);
        formRef.value.setFields({
          send_count_scope: {
            status: 'error',
            message: '数量范围错误',
          },
        });
        return false;
      }
      if (Number(form.send_min_count) > Number(form.send_max_count)) {
        const msg =
            form.send_type === '4'
                ? '最大剩余数量应该大于等于最小剩余数量'
                : '最大发送数量应该大于等于最小发送数量';
        Notification.error(msg);
        formRef.value.setFields({
          send_count_scope: {
            status: 'error',
            message: '数量范围错误',
          },
        });
        return false;
      }
      return true;
    } else {
      Notification.error({ content: '发送类型错误', position: 'topLeft' });
      return false;
    }
  }

  function checkPrecision() {
    const bool =
        /^\d+(\.\d+)?$/.test(form.amount_precision) &&
        Number(form.amount_precision) > 0 &&
        Number(form.amount_precision) < 18;
    if (!bool) {
      Notification.error({
        content: '金额精度必须为数字且大于0小于18',
        position: 'topLeft',
      });
      formRef.value.setFields({
        amount_precision: {
          status: 'error',
          message: '应大于0小于18',
        },
      });
      return false;
    } else {
      return true;
    }
  }

  function checkGasPrice() {
    if (form.gas_price_type === '1') {
      return true;
    } else if (form.gas_price_type === '2') {
      const bool =
          /^\d+(\.\d+)?$/.test(form.gas_price) && Number(form.gas_price) > 0;
      if (!bool) {
        Notification.error({
          content: 'Gas Price必须为数字且大于0',
          position: 'topLeft',
        });
        formRef.value.setFields({
          gas_price: {
            status: 'error',
            message: '必须为数字且大于0',
          },
        });
        return false;
      } else {
        return true;
      }
    } else if (form.gas_price_type === '3') {
      const bool =
          /^\d+$/.test(form.gas_price_rate) &&
          Number(form.gas_price_rate) > 0;
      if (!bool) {
        Notification.error({
          content: 'Gas Price 提高比例应为正整数',
          position: 'topLeft',
        });
        formRef.value.setFields({
          gas_price_rate: {
            status: 'error',
            message: '比例应为正整数',
          },
        });
        return false;
      }
      if (form.max_gas_price) {
        const bool1 =
            /^\d+(\.\d+)?$/.test(form.max_gas_price) &&
            Number(form.max_gas_price) > 0;
        if (!bool1) {
          Notification.error({
            content: '最大 Gas Price 设置必须为数字且大于0',
            position: 'topLeft',
          });
          formRef.value.setFields({
            max_gas_price: {
              status: 'error',
              message: '必须为数字且大于0',
            },
          });
          return false;
        } else {
          return true;
        }
      } else {
        return true;
      }
    } else {
      Notification.error({
        content: 'Gas Price 方式错误',
        position: 'topLeft',
      });
      return false;
    }
  }

  function checkGasLimit() {
    if (form.limit_type === '1') {
      return true;
    } else if (form.limit_type === '2') {
      const bool =
          /^\d+$/.test(form.limit_count) && Number(form.limit_count) > 0;
      if (!bool) {
        Notification.error({
          content: 'Gas Limit 数量必须为正整数',
          position: 'topLeft',
        });
        formRef.value.setFields({
          limit_count: {
            status: 'error',
            message: '数量必须为正整数',
          },
        });
        return false;
      } else {
        return true;
      }
    } else if (form.limit_type === '3') {
      const bool =
          /^\d+$/.test(form.limit_min_count) &&
          Number(form.limit_min_count) > 0 &&
          /^\d+$/.test(form.limit_max_count) &&
          Number(form.limit_max_count) > 0;
      if (!bool) {
        Notification.error({
          content: 'Gas Limit 数量范围必须为正整数',
          position: 'topLeft',
        });
        formRef.value.setFields({
          limit_count_scope: {
            status: 'error',
            message: '数量范围必须为正整数',
          },
        });
        return false;
      }
      if (Number(form.limit_min_count) > Number(form.limit_max_count)) {
        Notification.error({
          content: '最大 Gas Limit 数量应该大于等于最小 Gas Limit 数量',
          position: 'topLeft',
        });
        formRef.value.setFields({
          limit_count_scope: {
            status: 'error',
            message: '范围错误',
          },
        });
        return false;
      }
      return true;
    } else {
      Notification.error({
        content: 'Gas Limit 类型错误',
        position: 'topLeft',
      });
      return false;
    }
  }

  function checkDelay() {
    const bool =
        (form.min_interval === '0' ||
            (/^\d+$/.test(form.min_interval) &&
                Number(form.min_interval) >= 0)) &&
        (form.max_interval === '0' ||
            (/^\d+$/.test(form.max_interval) &&
                Number(form.max_interval) >= 0));
    if (!bool) {
      Notification.error({
        content: '发送间隔必须为正整数或者0',
        position: 'topLeft',
      });
      formRef.value.setFields({
        interval_scope: {
          status: 'error',
          message: '发送间隔必须为正整数或者0',
        },
      });
      return false;
    }
    if (Number(form.min_interval) > Number(form.max_interval)) {
      Notification.error({
        content: '最大间隔应该大于等于最小间隔',
        position: 'topLeft',
      });
      formRef.value.setFields({
        interval_scope: {
          status: 'error',
          message: '最大间隔应该大于等于最小间隔',
        },
      });
      return false;
    }
    return true;
  }

  return {
    validateForm,
    checkSendType,
    checkPrecision,
    checkGasPrice,
    checkGasLimit,
    checkDelay,
  };
}
