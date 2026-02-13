import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
import { handleError, ErrorContext } from '../utils/ErrorHandling';
import { Logger } from '../utils/LoggingSystem';

// ==================== 类型定义 ====================

export interface AirdropWallet {
  id: number;
  name: string;
  address: string;
  encrypted_private_key: string;
  label: string | null;
  group_name: string;
  proxy: string;
  chain_type: string;
  created_at: string;
  updated_at: string;
}

export interface CreateWalletRequest {
  name: string;
  address: string;
  private_key: string;
  label?: string;
  group_name?: string;
  proxy?: string;
  chain_type?: string;
}

export interface UpdateWalletRequest {
  id: number;
  name?: string;
  address?: string;
  private_key?: string;
  label?: string;
  group_name?: string;
  proxy?: string;
  chain_type?: string;
}

export interface BrowserProfile {
  id: number;
  name: string;
  description: string | null;
  user_agent: string | null;
  viewport_width: number;
  viewport_height: number;
  device_scale_factor: number;
  locale: string;
  timezone_id: string;
  proxy_type: string;
  proxy_host: string | null;
  proxy_port: number | null;
  proxy_username: string | null;
  proxy_password: string | null;
  canvas_spoof: boolean;
  webgl_spoof: boolean;
  audio_spoof: boolean;
  timezone_spoof: boolean;
  geolocation_spoof: boolean;
  font_spoof: boolean;
  webrtc_spoof: boolean;
  navigator_override: boolean;
  webdriver_override: boolean;
  custom_headers: string | null;
  headless: boolean;
  extensions: string | null;
  is_default: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateProfileRequest {
  name: string;
  description?: string;
  user_agent?: string;
  viewport_width?: number;
  viewport_height?: number;
  device_scale_factor?: number;
  locale?: string;
  timezone_id?: string;
  proxy_type?: string;
  proxy_host?: string;
  proxy_port?: number;
  proxy_username?: string;
  proxy_password?: string;
  canvas_spoof?: boolean;
  webgl_spoof?: boolean;
  audio_spoof?: boolean;
  timezone_spoof?: boolean;
  geolocation_spoof?: boolean;
  font_spoof?: boolean;
  webrtc_spoof?: boolean;
  navigator_override?: boolean;
  webdriver_override?: boolean;
  custom_headers?: string;
  headless?: boolean;
  extensions?: string;
  is_default?: boolean;
}

export interface AutomationScript {
  id: number;
  name: string;
  description: string | null;
  content: string;
  compiled_content: string | null;
  version: number;
  is_system: boolean;
  required_apis: string | null;
  author: string | null;
  tags: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateScriptRequest {
  name: string;
  description?: string;
  content: string;
  required_apis?: string[];
  author?: string;
  tags?: string[];
}

export interface AutomationTask {
  id: number;
  name: string;
  description: string | null;
  script_id: number;
  wallet_ids: string;
  profile_strategy: string;
  specific_profile_id: number | null;
  schedule_type: string;
  schedule_config: string;
  concurrency: number;
  timeout_seconds: number;
  retry_times: number;
  retry_interval_seconds: number;
  status: string;
  last_run_time: string | null;
  next_run_time: string | null;
  total_runs: number;
  success_runs: number;
  failed_runs: number;
  created_at: string;
  updated_at: string;
}

export interface CreateTaskRequest {
  name: string;
  description?: string;
  script_id: number;
  wallet_ids: number[];
  profile_strategy?: string;
  specific_profile_id?: number;
  schedule_type?: string;
  schedule_config: Record<string, any>;
  concurrency?: number;
  timeout_seconds?: number;
  retry_times?: number;
  retry_interval_seconds?: number;
}

export interface UpdateTaskRequest {
  id: number;
  name?: string;
  description?: string;
  script_id?: number;
  wallet_ids?: number[];
  profile_strategy?: string;
  specific_profile_id?: number;
  schedule_type?: string;
  schedule_config?: Record<string, any>;
  concurrency?: number;
  timeout_seconds?: number;
  retry_times?: number;
  retry_interval_seconds?: number;
  status?: string;
}

export interface TaskExecution {
  id: number;
  task_id: number;
  wallet_id: number;
  profile_id: number | null;
  status: string;
  start_time: string | null;
  end_time: string | null;
  duration_ms: number | null;
  error_message: string | null;
  result_data: string | null;
  logs: string | null;
  created_at: string;
}

export interface TaskExecutionStats {
  total_executions: number;
  success_count: number;
  failed_count: number;
  success_rate: number;
}

export interface BatchGenerateRequest {
  count: number;
  proxy_type?: string;
  proxy_host_prefix?: string;
  proxy_port_start?: number;
  enable_all_spoofs?: boolean;
}

// ==================== API 服务 ====================

/**
 * 初始化浏览器自动化表
 */
export async function initBrowserAutomationTables(): Promise<void> {
  try {
    await invoke('init_browser_automation_tables');
    Logger.info('Browser automation tables initialized successfully');
  } catch (error) {
    const context: ErrorContext = {
      operation: 'initBrowserAutomationTables',
      component: 'Database'
    };
    throw handleError(error, context);
  }
}

/**
 * 钱包管理服务
 */
export const walletService = {
  async getWallets(): Promise<AirdropWallet[]> {
    try {
      const wallets = await invoke('get_airdrop_wallets');
      Logger.info(`Retrieved ${wallets.length} wallets`);
      return wallets;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'getWallets',
        component: 'WalletService'
      };
      throw handleError(error, context);
    }
  },

  async createWallet(request: CreateWalletRequest): Promise<AirdropWallet> {
    try {
      const wallet = await invoke('create_airdrop_wallet', { request });
      Logger.info(`Created wallet: ${wallet.name}`);
      return wallet;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'createWallet',
        component: 'WalletService',
        metadata: { walletName: request.name }
      };
      throw handleError(error, context);
    }
  },

  async updateWallet(request: UpdateWalletRequest): Promise<AirdropWallet> {
    try {
      const wallet = await invoke('update_airdrop_wallet', { request });
      Logger.info(`Updated wallet ID: ${wallet.id}`);
      return wallet;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'updateWallet',
        component: 'WalletService',
        metadata: { walletId: request.id }
      };
      throw handleError(error, context);
    }
  },

  async deleteWallet(id: number): Promise<void> {
    try {
      await invoke('delete_airdrop_wallet', { id });
      Logger.info(`Deleted wallet ID: ${id}`);
    } catch (error) {
      const context: ErrorContext = {
        operation: 'deleteWallet',
        component: 'WalletService',
        metadata: { walletId: id }
      };
      throw handleError(error, context);
    }
  },

  async importWallets(wallets: CreateWalletRequest[]): Promise<{ wallets: AirdropWallet[]; errors: string[] }> {
    try {
      const result = await invoke<[AirdropWallet[], string[]]>('import_airdrop_wallets', { request: { wallets } });
      Logger.info(`Imported ${result[0].length} wallets with ${result[1].length} errors`);
      return { wallets: result[0], errors: result[1] };
    } catch (error) {
      const context: ErrorContext = {
        operation: 'importWallets',
        component: 'WalletService',
        metadata: { walletCount: wallets.length }
      };
      throw handleError(error, context);
    }
  },

  async getPrivateKey(id: number): Promise<string> {
    try {
      const privateKey = await invoke('get_wallet_private_key', { id });
      Logger.info(`Retrieved private key for wallet ID: ${id}`);
      return privateKey;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'getPrivateKey',
        component: 'WalletService',
        metadata: { walletId: id }
      };
      throw handleError(error, context);
    }
  },
};

/**
 * 浏览器环境配置服务
 */
export const profileService = {
  async getProfiles(): Promise<BrowserProfile[]> {
    try {
      const profiles = await invoke('get_browser_profiles');
      Logger.info(`Retrieved ${profiles.length} browser profiles`);
      return profiles;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'getProfiles',
        component: 'ProfileService'
      };
      throw handleError(error, context);
    }
  },

  async createProfile(request: CreateProfileRequest): Promise<BrowserProfile> {
    try {
      const profile = await invoke('create_browser_profile', { request });
      Logger.info(`Created profile: ${profile.name}`);
      return profile;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'createProfile',
        component: 'ProfileService',
        metadata: { profileName: request.name }
      };
      throw handleError(error, context);
    }
  },

  async updateProfile(request: Partial<CreateProfileRequest> & { id: number }): Promise<BrowserProfile> {
    try {
      const profile = await invoke('update_browser_profile', { request });
      Logger.info(`Updated profile ID: ${profile.id}`);
      return profile;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'updateProfile',
        component: 'ProfileService',
        metadata: { profileId: request.id }
      };
      throw handleError(error, context);
    }
  },

  async deleteProfile(id: number): Promise<void> {
    try {
      await invoke('delete_browser_profile', { id });
      Logger.info(`Deleted profile ID: ${id}`);
    } catch (error) {
      const context: ErrorContext = {
        operation: 'deleteProfile',
        component: 'ProfileService',
        metadata: { profileId: id }
      };
      throw handleError(error, context);
    }
  },

  async batchGenerate(request: BatchGenerateRequest): Promise<BrowserProfile[]> {
    try {
      const profiles = await invoke('batch_generate_profiles', { request });
      Logger.info(`Generated ${profiles.length} profiles in batch`);
      return profiles;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'batchGenerate',
        component: 'ProfileService',
        metadata: { count: request.count }
      };
      throw handleError(error, context);
    }
  },

  /**
   * 测试代理连接
   */
  async testProxyConnection(profileId: number): Promise<{ success: boolean; latencyMs: number; error?: string }> {
    try {
      const result = await invoke('test_proxy_connection', { profileId });
      Logger.info(`Proxy test for profile ${profileId}: ${result.success ? 'success' : 'failed'}`);
      return result;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'testProxyConnection',
        component: 'ProfileService',
        metadata: { profileId }
      };
      throw handleError(error, context);
    }
  },
};

/**
 * 脚本管理服务
 */
export const scriptService = {
  async getScripts(): Promise<AutomationScript[]> {
    try {
      const scripts = await invoke('get_automation_scripts');
      Logger.info(`Retrieved ${scripts.length} automation scripts`);
      return scripts;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'getScripts',
        component: 'ScriptService'
      };
      throw handleError(error, context);
    }
  },

  async createScript(request: CreateScriptRequest): Promise<AutomationScript> {
    try {
      const script = await invoke('create_automation_script', { request });
      Logger.info(`Created script: ${script.name}`);
      return script;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'createScript',
        component: 'ScriptService',
        metadata: { scriptName: request.name }
      };
      throw handleError(error, context);
    }
  },

  async updateScript(request: Partial<CreateScriptRequest> & { id: number }): Promise<AutomationScript> {
    try {
      const script = await invoke('update_automation_script', { request });
      Logger.info(`Updated script ID: ${script.id}`);
      return script;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'updateScript',
        component: 'ScriptService',
        metadata: { scriptId: request.id }
      };
      throw handleError(error, context);
    }
  },

  async deleteScript(id: number): Promise<void> {
    try {
      await invoke('delete_automation_script', { id });
      Logger.info(`Deleted script ID: ${id}`);
    } catch (error) {
      const context: ErrorContext = {
        operation: 'deleteScript',
        component: 'ScriptService',
        metadata: { scriptId: id }
      };
      throw handleError(error, context);
    }
  },

  /**
   * 导入脚本
   * 打开文件对话框选择 .js 或 .json 文件导入
   */
  async importScript(): Promise<{ name: string; content: string; description?: string } | null> {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          { name: '脚本文件', extensions: ['js', 'json'] },
          { name: '所有文件', extensions: ['*'] }
        ]
      });

      if (!selected || Array.isArray(selected)) {
        return null;
      }

      const content = await readTextFile(selected);
      const fileName = selected.split(/[/\\]/).pop() || 'Imported Script';
      const name = fileName.replace(/\.[^/.]+$/, ''); // 移除扩展名

      // 尝试解析为 JSON 格式（包含元数据）
      try {
        const parsed = JSON.parse(content);
        if (parsed.name && parsed.content) {
          Logger.info(`Imported script from JSON: ${parsed.name}`);
          return {
            name: parsed.name,
            content: parsed.content,
            description: parsed.description || ''
          };
        }
      } catch {
        // 不是 JSON 格式，当作纯 JS 脚本处理
      }

      Logger.info(`Imported script from file: ${name}`);
      return {
        name,
        content,
        description: `从 ${fileName} 导入的脚本`
      };
    } catch (error) {
      const context: ErrorContext = {
        operation: 'importScript',
        component: 'ScriptService'
      };
      throw handleError(error, context);
    }
  },

  /**
   * 导出脚本
   * 将脚本保存为 .json 文件（包含元数据）
   */
  async exportScript(script: AutomationScript): Promise<void> {
    try {
      const fileName = `${script.name.replace(/[^a-zA-Z0-9\u4e00-\u9fa5_-]/g, '_')}.json`;
      
      const savePath = await save({
        defaultPath: fileName,
        filters: [
          { name: 'JSON 文件', extensions: ['json'] },
          { name: 'JavaScript 文件', extensions: ['js'] }
        ]
      });

      if (!savePath) {
        throw new Error('用户取消了保存');
      }

      // 导出为 JSON 格式，包含完整元数据
      const exportData = {
        name: script.name,
        description: script.description || '',
        content: script.content,
        version: script.version,
        required_apis: script.required_apis,
        author: script.author,
        tags: script.tags,
        exported_at: new Date().toISOString()
      };

      await writeTextFile(savePath, JSON.stringify(exportData, null, 2));
      Logger.info(`Exported script: ${script.name}`);
    } catch (error) {
      const context: ErrorContext = {
        operation: 'exportScript',
        component: 'ScriptService',
        metadata: { scriptId: script.id, scriptName: script.name }
      };
      throw handleError(error, context);
    }
  },

  /**
   * 验证脚本语法
   */
  async validateScript(content: string): Promise<{ valid: boolean; errors: string[] }> {
    try {
      const result = await invoke('validate_script_syntax', { content });
      Logger.info(`Script validation: ${result.valid ? 'valid' : 'invalid'}`);
      return result;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'validateScript',
        component: 'ScriptService'
      };
      throw handleError(error, context);
    }
  },
};

/**
 * 任务管理服务
 */
export const taskService = {
  async getTasks(): Promise<AutomationTask[]> {
    try {
      const tasks = await invoke('get_automation_tasks');
      Logger.info(`Retrieved ${tasks.length} automation tasks`);
      return tasks;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'getTasks',
        component: 'TaskService'
      };
      throw handleError(error, context);
    }
  },

  async createTask(request: CreateTaskRequest): Promise<AutomationTask> {
    try {
      const task = await invoke('create_automation_task', { request });
      Logger.info(`Created task: ${task.name}`);
      return task;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'createTask',
        component: 'TaskService',
        metadata: { taskName: request.name }
      };
      throw handleError(error, context);
    }
  },

  async updateTask(request: UpdateTaskRequest): Promise<AutomationTask> {
    try {
      const task = await invoke('update_automation_task', { request });
      Logger.info(`Updated task ID: ${task.id}`);
      return task;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'updateTask',
        component: 'TaskService',
        metadata: { taskId: request.id }
      };
      throw handleError(error, context);
    }
  },

  async deleteTask(id: number): Promise<void> {
    try {
      await invoke('delete_automation_task', { id });
      Logger.info(`Deleted task ID: ${id}`);
    } catch (error) {
      const context: ErrorContext = {
        operation: 'deleteTask',
        component: 'TaskService',
        metadata: { taskId: id }
      };
      throw handleError(error, context);
    }
  },

  async toggleStatus(id: number): Promise<AutomationTask> {
    try {
      const task = await invoke('toggle_task_status', { id });
      Logger.info(`Toggled task ID: ${id} status to ${task.status}`);
      return task;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'toggleStatus',
        component: 'TaskService',
        metadata: { taskId: id }
      };
      throw handleError(error, context);
    }
  },

  /**
   * 启动任务执行
   */
  async startTaskExecution(taskId: number): Promise<void> {
    try {
      await invoke('start_task_execution', { taskId });
      Logger.info(`Started execution for task ID: ${taskId}`);
    } catch (error) {
      const context: ErrorContext = {
        operation: 'startTaskExecution',
        component: 'TaskService',
        metadata: { taskId }
      };
      throw handleError(error, context);
    }
  },

  /**
   * 停止任务执行
   */
  async stopTaskExecution(taskId: number): Promise<void> {
    try {
      await invoke('stop_task_execution', { taskId });
      Logger.info(`Stopped execution for task ID: ${taskId}`);
    } catch (error) {
      const context: ErrorContext = {
        operation: 'stopTaskExecution',
        component: 'TaskService',
        metadata: { taskId }
      };
      throw handleError(error, context);
    }
  },
};

/**
 * 执行记录服务
 */
export const executionService = {
  async getExecutions(taskId?: number, limit?: number): Promise<TaskExecution[]> {
    try {
      const executions = await invoke('get_task_executions', { taskId, limit });
      Logger.info(`Retrieved ${executions.length} executions`);
      return executions;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'getExecutions',
        component: 'ExecutionService',
        metadata: { taskId, limit }
      };
      throw handleError(error, context);
    }
  },

  async getStats(taskId?: number): Promise<TaskExecutionStats> {
    try {
      const stats = await invoke('get_task_execution_stats', { taskId });
      Logger.info(`Retrieved execution stats for task ID: ${taskId}`);
      return stats;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'getStats',
        component: 'ExecutionService',
        metadata: { taskId }
      };
      throw handleError(error, context);
    }
  },

  /**
   * 获取详细的执行日志
   */
  async getExecutionLogs(executionId: number): Promise<string> {
    try {
      const logs = await invoke('get_execution_logs', { executionId });
      Logger.info(`Retrieved logs for execution ID: ${executionId}`);
      return logs;
    } catch (error) {
      const context: ErrorContext = {
        operation: 'getExecutionLogs',
        component: 'ExecutionService',
        metadata: { executionId }
      };
      throw handleError(error, context);
    }
  },
};