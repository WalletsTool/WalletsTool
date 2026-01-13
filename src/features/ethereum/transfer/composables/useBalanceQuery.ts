import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Notification } from '@arco-design/web-vue';

export function useBalanceQuery(options = {}) {
  const {
    data,
    chainValue,
    currentCoin,
    threadCount,
    balanceLoading,
    balanceStopFlag,
    balanceStopStatus,
    balanceTotal,
    balanceCompleted,
    balanceProgress,
    showBalanceProgress,
    toAddressBalanceTotal,
    toAddressBalanceCompleted,
    toAddressBalanceProgress,
    showToAddressBalanceProgress,
    updateBalanceProgress,
    updateToAddressBalanceProgress,
  } = options;

  async function queryBalance() {
    if (!balanceStopStatus.value) {
      Notification.warning({
        content: '请停止或等待执行完成后再查询余额！',
        position: 'topLeft',
      });
      return;
    }

    if (data.value.length === 0) {
      Notification.warning({
        content: '请先导入钱包信息！',
        position: 'topLeft',
      });
      return;
    }

    if (!currentCoin.value || !chainValue.value) {
      Notification.warning({
        content: '请选择一个区块链网络！',
        position: 'topLeft',
      });
      return;
    }

    balanceTotal.value = data.value.length;
    balanceCompleted.value = 0;
    balanceProgress.value = 0;

    if (
        currentCoin.value.coin_type === 'base' ||
        currentCoin.value.coin_type === 'token'
    ) {
      balanceLoading.value = true;
      balanceStopFlag.value = false;
      balanceStopStatus.value = false;

      const totalItems = data.value.length;
      balanceTotal.value = totalItems;
      balanceCompleted.value = 0;
      balanceProgress.value = 0;
      showBalanceProgress.value = totalItems > 0;

      data.value.forEach((item) => {
        item.plat_balance = '';
        item.coin_balance = '';
        item.error_msg = '';
        item.exec_status = '0';
      });

      await queryBalanceInBatches();
    } else {
      Notification.warning({
        content: '查询 coin 类型错误！',
        position: 'topLeft',
      });
    }
  }

  async function queryBalanceInBatches() {
    const BATCH_SIZE = 50;
    const totalItems = data.value.length;
    const totalBatches = Math.ceil(totalItems / BATCH_SIZE);

    let stoppedMidway = false;

    try {
      for (let batchIndex = 0; batchIndex < totalBatches; batchIndex++) {
        const shouldStopAtStart = balanceStopFlag.value;

        const startIndex = batchIndex * BATCH_SIZE;
        const endIndex = Math.min(startIndex + BATCH_SIZE, totalItems);
        const batchData = JSON.parse(
            JSON.stringify(data.value.slice(startIndex, endIndex)),
        );

        await queryBalanceBatch(batchData, startIndex);

        if (balanceStopFlag.value && !shouldStopAtStart) {
          stoppedMidway = true;
          break;
        }
      }

      const successCount = data.value.filter(
          (item) => item.exec_status === '2',
      ).length;
      const failCount = data.value.filter(
          (item) => item.exec_status === '3',
      ).length;
      const totalCount = data.value.length;

      if (stoppedMidway) {
      } else if (successCount === totalCount) {
        Notification.success({
          content: '查询成功！',
          position: 'topLeft',
        });
      } else if (successCount > 0) {
        Notification.warning({
          content: `查询完成！成功 ${successCount} 条，失败 ${failCount} 条`,
          position: 'topLeft',
        });
      } else {
        Notification.error({
          content: '查询失败：所有记录都查询失败',
          position: 'topLeft',
        });
      }
    } catch (error) {
      console.error('分批查询失败:', error);
      Notification.error('查询失败：' + error.message);
    } finally {
      if (balanceStopFlag.value) {
        data.value.forEach((item) => {
          if (item.exec_status === '1' || item.exec_status === '3') {
            item.exec_status = '0';
            item.error_msg = '';
          }
        });
      }

      balanceLoading.value = false;
      balanceStopStatus.value = true;
      showBalanceProgress.value = false;
    }
  }

  async function queryBalanceBatch(batchData, startIndex) {
    const shouldStopAtStart = balanceStopFlag.value;

    if (shouldStopAtStart) {
      return;
    }

    try {
      const params = {
        chain: chainValue.value,
        coin_config: {
          coin_type: currentCoin.value.coin_type,
          contract_address: currentCoin.value.contract_address || '',
          abi: currentCoin.value.abi || '',
        },
        items: batchData.map((item) => ({
          key: item.key,
          address: item.address || '',
          private_key: item.private_key || '',
          plat_balance: null,
          coin_balance: null,
          nonce: null,
          exec_status: '0',
          error_msg: null,
          retry_flag: false,
        })),
        only_coin_config: false,
        thread_count: Number(threadCount.value),
      };

      const result = await invoke('query_balances_with_updates', {
        params,
      });

      if (balanceStopFlag.value && !shouldStopAtStart) {
        return;
      }

      if (result.success || result.items) {
        result.items.forEach((resultItem, index) => {
          const dataIndex = startIndex + index;
          if (data.value[dataIndex]) {
            const originalPrivateKey = data.value[dataIndex].private_key;
            Object.assign(data.value[dataIndex], resultItem);
            data.value[dataIndex].private_key = originalPrivateKey;
          }
        });
      } else {
        if (!balanceStopFlag.value) {
          batchData.forEach((item, index) => {
            const dataIndex = startIndex + index;
            if (data.value[dataIndex]) {
              data.value[dataIndex].exec_status = '3';
              data.value[dataIndex].error_msg =
                  result.error_msg || '查询失败！';
            }
          });
        }
      }

      updateBalanceProgress();
    } catch (error) {
      if (shouldStopAtStart || balanceStopFlag.value) {
        return;
      }

      batchData.forEach((item, index) => {
        const dataIndex = startIndex + index;
        if (data.value[dataIndex]) {
          data.value[dataIndex].exec_status = '3';
          data.value[dataIndex].error_msg = '查询失败！';
        }
      });

      updateBalanceProgress();
    }
  }

  async function queryToAddressBalance() {
    if (!balanceStopStatus.value) {
      Notification.warning({
        content: '请停止或等待执行完成后再查询余额！',
        position: 'topLeft',
      });
      return;
    }

    if (data.value.length === 0) {
      Notification.warning({
        content: '请先导入钱包信息！',
        position: 'topLeft',
      });
      return;
    }

    const itemsWithToAddr = data.value.filter((item) => item.to_addr);
    if (itemsWithToAddr.length === 0) {
      Notification.warning({
        content: '请先设置到账地址！',
        position: 'topLeft',
      });
      return;
    }

    if (
        currentCoin.value.coin_type === 'base' ||
        currentCoin.value.coin_type === 'token'
    ) {
      balanceLoading.value = true;
      balanceStopFlag.value = false;
      balanceStopStatus.value = false;

      const totalItems = itemsWithToAddr.length;
      toAddressBalanceTotal.value = totalItems;
      toAddressBalanceCompleted.value = 0;
      toAddressBalanceProgress.value = 0;
      showToAddressBalanceProgress.value = totalItems > 0;

      data.value.forEach((item) => {
        item.plat_balance = '';
        item.coin_balance = '';
        item.error_msg = '';
        item.exec_status = '0';
      });

      await queryToAddressBalanceInBatches();
    } else {
      Notification.warning({
        content: '查询 coin 类型错误！',
        position: 'topLeft',
      });
    }
  }

  async function queryToAddressBalanceInBatches() {
    const BATCH_SIZE = 50;
    const itemsWithToAddr = data.value.filter((item) => item.to_addr);
    const totalItems = itemsWithToAddr.length;
    const totalBatches = Math.ceil(totalItems / BATCH_SIZE);

    let stoppedMidway = false;

    try {
      for (let batchIndex = 0; batchIndex < totalBatches; batchIndex++) {
        const shouldStopAtStart = balanceStopFlag.value;

        const startIndex = batchIndex * BATCH_SIZE;
        const endIndex = Math.min(startIndex + BATCH_SIZE, totalItems);
        const batchData = JSON.parse(
            JSON.stringify(itemsWithToAddr.slice(startIndex, endIndex)),
        );

        await queryToAddressBalanceBatch(batchData, startIndex);

        if (balanceStopFlag.value && !shouldStopAtStart) {
          stoppedMidway = true;
          break;
        }
      }

      const successCount = data.value.filter(
          (item) => item.exec_status === '2',
      ).length;
      const failCount = data.value.filter(
          (item) => item.exec_status === '3',
      ).length;
      const totalCount = itemsWithToAddr.length;

      if (stoppedMidway) {
      } else if (successCount === totalCount) {
        Notification.success({
          content: `到账地址余额查询成功！共查询 ${totalCount} 个地址`,
          position: 'topLeft',
        });
      } else if (successCount > 0) {
        Notification.warning({
          content: `到账地址余额查询完成！成功 ${successCount} 条，失败 ${failCount} 条`,
          position: 'topLeft',
        });
      } else {
        Notification.error({
          content: '到账地址余额查询失败：所有地址都查询失败',
          position: 'topLeft',
        });
      }
    } catch (error) {
      console.error('分批查询到账地址失败:', error);
      Notification.error('到账地址余额查询失败：' + error.message);
    } finally {
      if (balanceStopFlag.value) {
        data.value.forEach((item) => {
          if (item.exec_status === '1' || item.exec_status === '3') {
            item.exec_status = '0';
            item.error_msg = '';
          }
        });
      }

      balanceLoading.value = false;
      balanceStopStatus.value = true;
      showToAddressBalanceProgress.value = false;
    }
  }

  async function queryToAddressBalanceBatch(batchData, startIndex) {
    const shouldStopAtStart = balanceStopFlag.value;

    if (shouldStopAtStart) {
      return;
    }

    try {
      const queryItems = batchData.map((item) => ({
        key: item.key,
        address: item.to_addr,
        private_key: '',
        plat_balance: null,
        coin_balance: null,
        nonce: null,
        exec_status: '0',
        error_msg: null,
        retry_flag: false,
      }));

      const params = {
        chain: chainValue.value,
        coin_config: {
          coin_type: currentCoin.value.coin_type,
          contract_address: currentCoin.value.contract_address || '',
          abi: currentCoin.value.abi || '',
        },
        items: queryItems,
        only_coin_config: false,
        thread_count: Number(threadCount.value),
      };

      const result = await invoke('query_balances_with_updates', {
        params,
      });

      if (balanceStopFlag.value && !shouldStopAtStart) {
        return;
      }

      if (result.success || result.items) {
        result.items.forEach((resultItem, index) => {
          const originalItem = batchData[index];
          const dataIndex = data.value.findIndex(
              (item) => item.key === originalItem.key,
          );
          if (dataIndex !== -1) {
            const originalPrivateKey = data.value[dataIndex].private_key;
            const originalToAddr = data.value[dataIndex].to_addr;
            Object.assign(data.value[dataIndex], resultItem);
            data.value[dataIndex].private_key = originalPrivateKey;
            data.value[dataIndex].to_addr = originalToAddr;
          }
        });
      } else {
        if (!balanceStopFlag.value) {
          batchData.forEach((item) => {
            const dataIndex = data.value.findIndex(
                (dataItem) => dataItem.key === item.key,
            );
            if (dataIndex !== -1) {
              data.value[dataIndex].exec_status = '3';
              data.value[dataIndex].error_msg =
                  result.error_msg || '查询失败！';
            }
          });
        }
      }

      updateToAddressBalanceProgress();
    } catch (error) {
      if (shouldStopAtStart || balanceStopFlag.value) {
        return;
      }

      const errorMsg = String(error);
      if (
          errorMsg.includes('RPC配置') ||
          errorMsg.includes('RPC节点') ||
          errorMsg.includes('禁用')
      ) {
        Notification.error({
          title: '查询失败',
          content: errorMsg,
          duration: 5000,
          position: 'topLeft',
        });
      }

      batchData.forEach((item) => {
        const dataIndex = data.value.findIndex(
            (dataItem) => dataItem.key === item.key,
        );
        if (dataIndex !== -1) {
          data.value[dataIndex].exec_status = '3';
          data.value[dataIndex].error_msg = '查询失败！';
        }
      });

      updateToAddressBalanceProgress();
    }
  }

  async function stopBalanceQuery() {
    balanceStopFlag.value = true;

    try {
      await invoke('stop_balance_query', {});
    } catch (error) {
      console.error('停止查询请求失败:', error);
    }

    data.value.forEach((item) => {
      if (item.exec_status === '1' || item.exec_status === '3') {
        item.exec_status = '0';
        item.error_msg = '';
      }
    });

    balanceLoading.value = false;
    balanceStopStatus.value = true;
    showBalanceProgress.value = false;
    showToAddressBalanceProgress.value = false;
  }

  return {
    queryBalance,
    queryToAddressBalance,
    stopBalanceQuery,
    queryBalanceInBatches,
    queryToAddressBalanceInBatches,
  };
}
