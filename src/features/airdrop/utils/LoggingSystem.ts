import { Message } from '@arco-design/web-vue';

// 日志级别枚举
export enum LogLevel {
  DEBUG = 'debug',
  INFO = 'info', 
  WARN = 'warn',
  ERROR = 'error',
  SUCCESS = 'success'
}

// 日志条目接口
export interface LogEntry {
  id: string;
  timestamp: Date;
  level: LogLevel;
  message: string;
  source: string; // 来源模块
  details?: any; // 详细信息
  stack?: string; // 错误堆栈
}

// 日志监听器接口
export interface LogListener {
  onLog(entry: LogEntry): void;
}

// 增强日志系统类
class EnhancedLogger {
  private logs: LogEntry[] = [];
  private listeners: LogListener[] = [];
  private maxLogs = 1000; // 最大日志数量

  // 添加日志监听器
  addListener(listener: LogListener): void {
    this.listeners.push(listener);
  }

  // 移除日志监听器
  removeListener(listener: LogListener): void {
    const index = this.listeners.indexOf(listener);
    if (index > -1) {
      this.listeners.splice(index, 1);
    }
  }

  // 记录日志
  log(level: LogLevel, message: string, source: string, details?: any, error?: Error): void {
    const entry: LogEntry = {
      id: this.generateId(),
      timestamp: new Date(),
      level,
      message,
      source,
      details,
      stack: error?.stack
    };

    // 添加到日志数组
    this.logs.push(entry);
    
    // 限制日志数量
    if (this.logs.length > this.maxLogs) {
      this.logs = this.logs.slice(-this.maxLogs);
    }

    // 通知所有监听器
    this.listeners.forEach(listener => listener.onLog(entry));

    // 显示用户友好的消息
    this.showUserMessage(entry);
  }

  // 快捷方法
  debug(message: string, source: string, details?: any): void {
    this.log(LogLevel.DEBUG, message, source, details);
  }

  info(message: string, source: string, details?: any): void {
    this.log(LogLevel.INFO, message, source, details);
  }

  warn(message: string, source: string, details?: any): void {
    this.log(LogLevel.WARN, message, source, details);
  }

  error(message: string, source: string, details?: any, error?: Error): void {
    this.log(LogLevel.ERROR, message, source, details, error);
  }

  success(message: string, source: string, details?: any): void {
    this.log(LogLevel.SUCCESS, message, source, details);
  }

  // 获取日志
  getLogs(): LogEntry[] {
    return [...this.logs];
  }

  // 清空日志
  clearLogs(): void {
    this.logs = [];
  }

  // 导出日志
  exportLogs(): string {
    return JSON.stringify(this.logs, null, 2);
  }

  // 显示用户友好的消息
  private showUserMessage(entry: LogEntry): void {
    // 只显示重要的日志给用户
    if (entry.level === LogLevel.ERROR || entry.level === LogLevel.SUCCESS || entry.level === LogLevel.WARN) {
      let type: 'error' | 'success' | 'warning' | 'info' = 'info';
      
      switch (entry.level) {
        case LogLevel.ERROR:
          type = 'error';
          break;
        case LogLevel.SUCCESS:
          type = 'success';
          break;
        case LogLevel.WARN:
          type = 'warning';
          break;
        default:
          type = 'info';
      }

      // 避免重复消息
      Message[type]({
        content: `${entry.source}: ${entry.message}`,
        duration: entry.level === LogLevel.ERROR ? 5000 : 3000
      });
    }
  }

  // 生成唯一ID
  private generateId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  }

  // 全局错误处理器
  handleGlobalError(error: Error, source: string = 'Global'): void {
    this.error('未处理的错误', source, { 
      message: error.message,
      name: error.name,
      cause: error.cause 
    }, error);
  }
}

// 创建全局日志实例
export const logger = new EnhancedLogger();

// 全局错误捕获
window.addEventListener('error', (event) => {
  logger.handleGlobalError(event.error, 'Window Error');
});

window.addEventListener('unhandledrejection', (event) => {
  logger.handleGlobalError(event.reason as Error, 'Unhandled Rejection');
});

// 导出快捷函数
export const logDebug = (message: string, source: string, details?: any) => 
  logger.debug(message, source, details);

export const logInfo = (message: string, source: string, details?: any) => 
  logger.info(message, source, details);

export const logWarn = (message: string, source: string, details?: any) => 
  logger.warn(message, source, details);

export const logError = (message: string, source: string, details?: any, error?: Error) => 
  logger.error(message, source, details, error);

export const logSuccess = (message: string, source: string, details?: any) => 
  logger.success(message, source, details);