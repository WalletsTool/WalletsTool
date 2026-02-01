import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';

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
  await invoke('init_browser_automation_tables');
}

/**
 * 钱包管理服务
 */
export const walletService = {
  async getWallets(): Promise<AirdropWallet[]> {
    return await invoke('get_airdrop_wallets');
  },

  async createWallet(request: CreateWalletRequest): Promise<AirdropWallet> {
    return await invoke('create_airdrop_wallet', { request });
  },

  async updateWallet(request: UpdateWalletRequest): Promise<AirdropWallet> {
    return await invoke('update_airdrop_wallet', { request });
  },

  async deleteWallet(id: number): Promise<void> {
    await invoke('delete_airdrop_wallet', { id });
  },

  async importWallets(wallets: CreateWalletRequest[]): Promise<{ wallets: AirdropWallet[]; errors: string[] }> {
    return await invoke('import_airdrop_wallets', { request: { wallets } });
  },

  async getPrivateKey(id: number): Promise<string> {
    return await invoke('get_wallet_private_key', { id });
  },
};

/**
 * 浏览器环境配置服务
 */
export const profileService = {
  async getProfiles(): Promise<BrowserProfile[]> {
    return await invoke('get_browser_profiles');
  },

  async createProfile(request: CreateProfileRequest): Promise<BrowserProfile> {
    return await invoke('create_browser_profile', { request });
  },

  async updateProfile(request: Partial<CreateProfileRequest> & { id: number }): Promise<BrowserProfile> {
    return await invoke('update_browser_profile', { request });
  },

  async deleteProfile(id: number): Promise<void> {
    await invoke('delete_browser_profile', { id });
  },

  async batchGenerate(request: BatchGenerateRequest): Promise<BrowserProfile[]> {
    return await invoke('batch_generate_profiles', { request });
  },
};

/**
 * 脚本管理服务
 */
export const scriptService = {
  async getScripts(): Promise<AutomationScript[]> {
    return await invoke('get_automation_scripts');
  },

  async createScript(request: CreateScriptRequest): Promise<AutomationScript> {
    return await invoke('create_automation_script', { request });
  },

  async updateScript(request: Partial<CreateScriptRequest> & { id: number }): Promise<AutomationScript> {
    return await invoke('update_automation_script', { request });
  },

  async deleteScript(id: number): Promise<void> {
    await invoke('delete_automation_script', { id });
  },

  /**
   * 导入脚本
   * 打开文件对话框选择 .js 或 .json 文件导入
   */
  async importScript(): Promise<{ name: string; content: string; description?: string } | null> {
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
        return {
          name: parsed.name,
          content: parsed.content,
          description: parsed.description || ''
        };
      }
    } catch {
      // 不是 JSON 格式，当作纯 JS 脚本处理
    }

    return {
      name,
      content,
      description: `从 ${fileName} 导入的脚本`
    };
  },

  /**
   * 导出脚本
   * 将脚本保存为 .json 文件（包含元数据）
   */
  async exportScript(script: AutomationScript): Promise<void> {
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
  }
};

/**
 * 任务管理服务
 */
export const taskService = {
  async getTasks(): Promise<AutomationTask[]> {
    return await invoke('get_automation_tasks');
  },

  async createTask(request: CreateTaskRequest): Promise<AutomationTask> {
    return await invoke('create_automation_task', { request });
  },

  async updateTask(request: UpdateTaskRequest): Promise<AutomationTask> {
    return await invoke('update_automation_task', { request });
  },

  async deleteTask(id: number): Promise<void> {
    await invoke('delete_automation_task', { id });
  },

  async toggleStatus(id: number): Promise<AutomationTask> {
    return await invoke('toggle_task_status', { id });
  },
};

/**
 * 执行记录服务
 */
export const executionService = {
  async getExecutions(taskId?: number, limit?: number): Promise<TaskExecution[]> {
    return await invoke('get_task_executions', { taskId, limit });
  },

  async getStats(taskId?: number): Promise<TaskExecutionStats> {
    return await invoke('get_task_execution_stats', { taskId });
  },
};
