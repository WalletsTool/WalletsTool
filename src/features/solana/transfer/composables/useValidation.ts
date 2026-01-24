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
          checkPriorityFee()
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
        Number(form.amount_precision) < 10; // Solana max decimals is 9
    if (!bool) {
      Notification.error({
        content: '金额精度必须为数字且大于0小于10',
        position: 'topLeft',
      });
      formRef.value.setFields({
        amount_precision: {
          status: 'error',
          message: '应大于0小于10',
        },
      });
      return false;
    } else {
      return true;
    }
  }

  function checkPriorityFee() {
    // Priority Fee logic for Solana (optional or microLamports)
    // Assuming gas_price field is reused for Priority Fee
    if (form.gas_price_type === '1') { // Default
      return true;
    } else if (form.gas_price_type === '2') { // Custom
      const bool =
          /^\d+$/.test(form.gas_price) && Number(form.gas_price) >= 0;
      if (!bool) {
        Notification.error({
          content: '优先费必须为非负整数 (MicroLamports)',
          position: 'topLeft',
        });
        formRef.value.setFields({
          gas_price: {
            status: 'error',
            message: '必须为非负整数',
          },
        });
        return false;
      } else {
        return true;
      }
    } 
    // Type 3 (Rate) might not apply to Solana directly unless we implement dynamic fee fetching
    // For now, allow it but validation might be tricky. Let's keep it simple.
    return true; 
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
    checkPriorityFee, // Renamed from checkGasPrice
    checkDelay,
  };
}
