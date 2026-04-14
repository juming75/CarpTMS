/**
 * 错误处理器
 * 统一处理应用中的各种错误，提供友好的错误信息和恢复建议
 */

import { logService } from './logService';

// 错误类型枚举
export enum ErrorType {
  NETWORK = 'network',
  AUTHENTICATION = 'authentication',
  VALIDATION = 'validation',
  DATABASE = 'database',
  PROTOCOL = 'protocol',
  UNKNOWN = 'unknown',
}

// 错误严重级别
export enum ErrorSeverity {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  CRITICAL = 'critical',
}

// 应用错误类
export class AppError extends Error {
  public readonly type: ErrorType;
  public readonly severity: ErrorSeverity;
  public readonly userMessage: string;
  public readonly technicalDetails?: string;
  public readonly recoverySuggestion?: string;
  public readonly originalError?: Error;
  public readonly timestamp: Date;
  public readonly requestId: string;

  constructor(
    message: string,
    type: ErrorType = ErrorType.UNKNOWN,
    severity: ErrorSeverity = ErrorSeverity.MEDIUM,
    options?: {
      userMessage?: string;
      technicalDetails?: string;
      recoverySuggestion?: string;
      originalError?: Error;
      requestId?: string;
    }
  ) {
    super(message);
    this.name = 'AppError';
    this.type = type;
    this.severity = severity;
    this.userMessage = options?.userMessage || message;
    this.technicalDetails = options?.technicalDetails;
    this.recoverySuggestion = options?.recoverySuggestion;
    this.originalError = options?.originalError;
    this.timestamp = new Date();
    this.requestId = options?.requestId || logService.generateRequestId();

    // 保持正确的堆栈跟踪
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, AppError);
    }
  }

  toJSON() {
    return {
      name: this.name,
      message: this.message,
      type: this.type,
      severity: this.severity,
      userMessage: this.userMessage,
      technicalDetails: this.technicalDetails,
      recoverySuggestion: this.recoverySuggestion,
      originalError: this.originalError?.message,
      timestamp: this.timestamp,
      requestId: this.requestId,
    };
  }
}

// 网络错误
export class NetworkError extends AppError {
  constructor(message: string, options?: { originalError?: Error; url?: string; requestId?: string }) {
    super(message, ErrorType.NETWORK, ErrorSeverity.HIGH, {
      userMessage: '网络连接失败，请检查网络设置',
      recoverySuggestion: '请检查网络连接，确保服务器地址正确，然后重试',
      technicalDetails: options?.url ? `请求URL: ${options.url}` : undefined,
      originalError: options?.originalError,
      requestId: options?.requestId,
    });
    this.name = 'NetworkError';
  }
}

// 认证错误
export class AuthenticationError extends AppError {
  constructor(message: string, options?: { originalError?: Error; username?: string; requestId?: string }) {
    super(message, ErrorType.AUTHENTICATION, ErrorSeverity.HIGH, {
      userMessage: '认证失败，请检查用户名和密码',
      recoverySuggestion: '请检查用户名和密码是否正确，如忘记密码请联系管理员',
      technicalDetails: options?.username ? `用户名: ${options.username}` : undefined,
      originalError: options?.originalError,
      requestId: options?.requestId,
    });
    this.name = 'AuthenticationError';
  }
}

// 验证错误
export class ValidationError extends AppError {
  constructor(message: string, options?: { originalError?: Error; field?: string; requestId?: string }) {
    super(message, ErrorType.VALIDATION, ErrorSeverity.LOW, {
      userMessage: '输入数据验证失败',
      recoverySuggestion: '请检查输入数据是否符合要求',
      technicalDetails: options?.field ? `字段: ${options.field}` : undefined,
      originalError: options?.originalError,
      requestId: options?.requestId,
    });
    this.name = 'ValidationError';
  }
}

// 数据库错误
export class DatabaseError extends AppError {
  constructor(
    message: string,
    options?: { originalError?: Error; table?: string; operation?: string; requestId?: string }
  ) {
    super(message, ErrorType.DATABASE, ErrorSeverity.CRITICAL, {
      userMessage: '数据库操作失败',
      recoverySuggestion: '请稍后重试，如问题持续存在请联系技术支持',
      technicalDetails:
        options?.table && options?.operation ? `表: ${options.table}, 操作: ${options.operation}` : undefined,
      originalError: options?.originalError,
      requestId: options?.requestId,
    });
    this.name = 'DatabaseError';
  }
}

// 协议错误
export class ProtocolError extends AppError {
  constructor(message: string, options?: { originalError?: Error; protocol?: string; requestId?: string }) {
    super(message, ErrorType.PROTOCOL, ErrorSeverity.HIGH, {
      userMessage: '协议通信失败',
      recoverySuggestion: '请检查服务器状态，或尝试切换协议版本',
      technicalDetails: options?.protocol ? `协议: ${options.protocol}` : undefined,
      originalError: options?.originalError,
      requestId: options?.requestId,
    });
    this.name = 'ProtocolError';
  }
}

// 错误处理器类
export class ErrorHandler {
  /**
   * 处理错误
   */
  static handle(error: Error | AppError | unknown): AppError {
    // 如果已经是 AppError，直接返回
    if (error instanceof AppError) {
      this.logError(error);
      return error;
    }

    // 如果是普通 Error，转换为 AppError
    if (error instanceof Error) {
      const appError = this.convertToAppError(error);
      this.logError(appError);
      return appError;
    }

    // 未知错误
    const unknownError = new AppError('发生未知错误', ErrorType.UNKNOWN, ErrorSeverity.MEDIUM, {
      userMessage: '发生未知错误，请稍后重试',
      technicalDetails: String(error),
    });
    this.logError(unknownError);
    return unknownError;
  }

  /**
   * 将普通 Error 转换为 AppError
   */
  private static convertToAppError(error: Error): AppError {
    const message = error.message.toLowerCase();

    // 忽略 ResizeObserver 错误（非关键错误）
    if (message.includes('resizeobserver') && message.includes('loop completed with undelivered notifications')) {
      // 直接返回一个非关键的 AppError，不显示给用户
      return new AppError('ResizeObserver 错误', ErrorType.OTHER, ErrorSeverity.LOW, {
        userMessage: '',
        technicalDetails: error.message,
        originalError: error,
      });
    }

    // 网络错误
    if (message.includes('network') || message.includes('fetch') || message.includes('connection')) {
      return new NetworkError(error.message, { originalError: error });
    }

    // 认证错误
    if (message.includes('unauthorized') || message.includes('forbidden') || message.includes('401')) {
      return new AuthenticationError(error.message, { originalError: error });
    }

    // 验证错误
    if (message.includes('validation') || message.includes('invalid')) {
      return new ValidationError(error.message, { originalError: error });
    }

    // 数据库错误
    if (message.includes('database') || message.includes('sqlite') || message.includes('sql')) {
      return new DatabaseError(error.message, { originalError: error });
    }

    // 协议错误
    if (message.includes('protocol') || message.includes('parse')) {
      return new ProtocolError(error.message, { originalError: error });
    }

    // 默认未知错误
    return new AppError(error.message, ErrorType.UNKNOWN, ErrorSeverity.MEDIUM, {
      originalError: error,
    });
  }

  /**
   * 记录错误
   */
  private static logError(error: AppError): void {
    const logData = {
      type: error.type,
      severity: error.severity,
      userMessage: error.userMessage,
      technicalDetails: error.technicalDetails,
      recoverySuggestion: error.recoverySuggestion,
      originalError: error.originalError?.message,
      requestId: error.requestId,
    };

    switch (error.severity) {
      case ErrorSeverity.CRITICAL:
        logService.fatal(error.message, logData, error.originalError);
        break;
      case ErrorSeverity.HIGH:
        logService.error(error.message, logData, error.originalError);
        break;
      case ErrorSeverity.MEDIUM:
        logService.warn(error.message, logData);
        break;
      case ErrorSeverity.LOW:
        logService.debug(error.message, logData);
        break;
    }
  }

  /**
   * 异步错误处理包装器
   */
  static async wrapAsync<T>(
    fn: () => Promise<T>,
    context?: string
  ): Promise<{ success: true; data: T } | { success: false; error: AppError }> {
    try {
      const data = await fn();
      return { success: true, data };
    } catch (error) {
      const appError = this.handle(error);
      if (context) {
        logService.error(`异步操作失败: ${context}`, {
          requestId: appError.requestId,
          error: appError.message,
        });
      }
      return { success: false, error: appError };
    }
  }

  /**
   * 同步错误处理包装器
   */
  static wrapSync<T>(fn: () => T, context?: string): { success: true; data: T } | { success: false; error: AppError } {
    try {
      const data = fn();
      return { success: true, data };
    } catch (error) {
      const appError = this.handle(error);
      if (context) {
        logService.error(`同步操作失败: ${context}`, {
          requestId: appError.requestId,
          error: appError.message,
        });
      }
      return { success: false, error: appError };
    }
  }

  /**
   * 获取用户友好的错误信息
   */
  static getUserMessage(error: Error | AppError | unknown): string {
    const appError = error instanceof AppError ? error : this.handle(error);
    return appError.userMessage;
  }

  /**
   * 获取恢复建议
   */
  static getRecoverySuggestion(error: Error | AppError | unknown): string | undefined {
    const appError = error instanceof AppError ? error : this.handle(error);
    return appError.recoverySuggestion;
  }

  /**
   * 检查错误是否可恢复
   */
  static isRecoverable(error: Error | AppError | unknown): boolean {
    const appError = error instanceof AppError ? error : this.handle(error);
    return appError.severity !== ErrorSeverity.CRITICAL;
  }
}

// 全局错误处理
if (typeof window !== 'undefined') {
  // 浏览器环境：未捕获的 Promise rejection
  window.addEventListener('unhandledrejection', (event) => {
    const error = ErrorHandler.handle(event.reason);
    console.error('[ErrorHandler] 未处理的 Promise rejection:', error);

    // 阻止默认的控制台输出
    event.preventDefault();
  });

  // 浏览器环境：未捕获的错误
  window.addEventListener('error', (event) => {
    const error = ErrorHandler.handle(event.error || new Error(event.message));
    console.error('[ErrorHandler] 未捕获的错误:', error);
  });
}

// 导出便捷函数
export const handleError = ErrorHandler.handle.bind(ErrorHandler);
export const wrapAsync = ErrorHandler.wrapAsync.bind(ErrorHandler);
export const wrapSync = ErrorHandler.wrapSync.bind(ErrorHandler);
export const getUserMessage = ErrorHandler.getUserMessage.bind(ErrorHandler);
export const getRecoverySuggestion = ErrorHandler.getRecoverySuggestion.bind(ErrorHandler);
export const isRecoverable = ErrorHandler.isRecoverable.bind(ErrorHandler);

export default ErrorHandler;


