import { invoke } from '@tauri-apps/api/core';
import { Message } from '@arco-design/web-vue';

/**
 * 代理测试工具类
 * 提供代理连接测试、健康检查和性能评估功能
 */
export class ProxyTester {
  /**
   * 测试代理连接
   * @param proxyConfig 代理配置
   * @returns 测试结果
   */
  static async testProxyConnection(proxyConfig: {
    type: string;
    host?: string;
    port?: number;
    username?: string;
    password?: string;
  }): Promise<{
    success: boolean;
    latency: number;
    error?: string;
    ip?: string;
  }> {
    const startTime = Date.now();
    
    try {
      // 调用 Rust 后端进行代理测试
      const result = await invoke('test_proxy_connection', { 
        proxyConfig 
      });
      
      const latency = Date.now() - startTime;
      return {
        success: true,
        latency,
        ...result
      };
    } catch (error) {
      const latency = Date.now() - startTime;
      return {
        success: false,
        latency,
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }

  /**
   * 批量测试多个代理
   * @param proxies 代理列表
   * @returns 测试结果数组
   */
  static async testMultipleProxies(proxies: Array<{
    type: string;
    host?: string;
    port?: number;
    username?: string;
    password?: string;
  }>): Promise<Array<{
    success: boolean;
    latency: number;
    error?: string;
    ip?: string;
  }>> {
    // 并发测试，但限制并发数避免资源耗尽
    const batchSize = 10;
    const results: any[] = [];
    
    for (let i = 0; i < proxies.length; i += batchSize) {
      const batch = proxies.slice(i, i + batchSize);
      const batchResults = await Promise.all(
        batch.map(proxy => this.testProxyConnection(proxy))
      );
      results.push(...batchResults);
    }
    
    return results;
  }

  /**
   * 获取代理健康状态
   * @param proxyId 代理ID
   * @returns 健康状态
   */
  static async getProxyHealth(proxyId: number): Promise<{
    status: 'healthy' | 'degraded' | 'unhealthy';
    lastChecked: string;
    successRate: number;
    averageLatency: number;
  }> {
    try {
      const result = await invoke('get_proxy_health', { proxyId });
      return result;
    } catch (error) {
      console.error('Failed to get proxy health:', error);
      throw error;
    }
  }

  /**
   * 显示代理测试结果
   * @param result 测试结果
   */
  static showTestResult(result: {
    success: boolean;
    latency: number;
    error?: string;
    ip?: string;
  }) {
    if (result.success) {
      let message = `✅ 代理连接成功！\n`;
      message += `延迟: ${result.latency}ms\n`;
      if (result.ip) {
        message += `IP地址: ${result.ip}\n`;
      }
      Message.success(message);
    } else {
      let message = `❌ 代理连接失败！\n`;
      message += `延迟: ${result.latency}ms\n`;
      if (result.error) {
        message += `错误: ${result.error}\n`;
      }
      Message.error(message);
    }
  }

  /**
   * 验证代理配置的有效性
   * @param config 代理配置
   * @returns 验证结果
   */
  static validateProxyConfig(config: {
    type: string;
    host?: string;
    port?: number;
    username?: string;
    password?: string;
  }): { valid: boolean; errors: string[] } {
    const errors: string[] = [];
    
    // 验证代理类型
    if (!['direct', 'http', 'socks5'].includes(config.type)) {
      errors.push('无效的代理类型');
    }
    
    // 如果不是 direct 类型，需要验证主机和端口
    if (config.type !== 'direct') {
      if (!config.host || config.host.trim() === '') {
        errors.push('代理主机不能为空');
      }
      
      if (!config.port || config.port < 1 || config.port > 65535) {
        errors.push('代理端口必须在 1-65535 范围内');
      }
    }
    
    return {
      valid: errors.length === 0,
      errors
    };
  }
}

export default ProxyTester;