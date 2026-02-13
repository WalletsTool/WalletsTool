/**
 * 增强的错误处理和日志工具
 */

export interface EnhancedError {
  code: string;
  message: string;
  details?: Record<string, any>;
  timestamp: number;
  stack?: string;
}

export class ErrorHandler {
  static createError(code: string, message: string, details?: Record<string, any>): EnhancedError {
    return {
      code,
      message,
      details,
      timestamp: Date.now()
    };
  }

  static formatError(error: any): EnhancedError {
    if (error && typeof error === 'object') {
      if ('code' in error && 'message' in error) {
        // 已经是 EnhancedError 格式
        return error as EnhancedError;
      }
      
      // 处理标准 Error 对象
      if (error instanceof Error) {
        return {
          code: 'UNKNOWN_ERROR',
          message: error.message,
          details: {
            name: error.name,
            stack: error.stack
          },
          timestamp: Date.now()
        };
      }
      
      // 处理其他对象
      return {
        code: 'API_ERROR',
        message: error.message || '未知错误',
        details: error,
        timestamp: Date.now()
      };
    }
    
    // 处理字符串错误
    return {
      code: 'STRING_ERROR',
      message: String(error),
      timestamp: Date.now()
    };
  }

  static getErrorMessage(error: any): string {
    const enhanced = this.formatError(error);
    return `${enhanced.message} [${enhanced.code}]`;
  }
}

export class Logger {
  static log(level: 'info' | 'warn' | 'error' | 'debug', message: string, data?: any) {
    const timestamp = new Date().toISOString();
    const logEntry = `[${timestamp}] [${level.toUpperCase()}] ${message}`;
    
    switch (level) {
      case 'info':
        console.log(logEntry, data || '');
        break;
      case 'warn':
        console.warn(logEntry, data || '');
        break;
      case 'error':
        console.error(logEntry, data || '');
        break;
      case 'debug':
        console.debug(logEntry, data || '');
        break;
    }
  }

  static info(message: string, data?: any) {
    this.log('info', message, data);
  }

  static warn(message: string, data?: any) {
    this.log('warn', message, data);
  }

  static error(message: string, data?: any) {
    this.log('error', message, data);
  }

  static debug(message: string, data?: any) {
    this.log('debug', message, data);
  }
}