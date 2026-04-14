// 日志级别类型
export type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'fatal';

// 日志上下文类型
export interface LogContext {
  [key: string]: unknown;
  url?: string;
  method?: string;
  status?: number;
  requestId?: string;
  userId?: number;
  terminalId?: string;
}

// 日志条目类型
export interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  context: LogContext;
  stack?: string;
  [key: string]: unknown;
}

// 日志服务配置
export interface LogConfig {
  enabled: boolean;
  level: LogLevel;
  logToConsole: boolean;
  logToServer: boolean;
  serverUrl?: string;
  appName: string;
  environment: string;
}

// 可观测性平台配置
export interface ObservabilityConfig {
  enabled: boolean;
  platform: 'datadog' | 'newrelic' | 'prometheus' | 'custom';
  apiKey?: string;
  endpoint?: string;
}

// 日志服务类
export class LogService {
  private config: LogConfig;
  private observabilityConfig: ObservabilityConfig;
  private requestIdCounter = 0;

  constructor(config?: Partial<LogConfig>, observabilityConfig?: Partial<ObservabilityConfig>) {
    this.config = {
      enabled: true,
      level: 'info',
      logToConsole: true,
      logToServer: false,
      appName: 'claw-tms-client',
      environment: import.meta.env.MODE || 'development',
      ...config,
    };

    this.observabilityConfig = {
      enabled: false,
      platform: 'custom',
      ...observabilityConfig,
    };
  }

  // 生成唯一请求ID
  public generateRequestId(): string {
    this.requestIdCounter++;
    return `${Date.now()}-${this.requestIdCounter}-${Math.random().toString(36).substring(2, 9)}`;
  }

  // 记录日志
  public log(level: LogLevel, message: string, context: LogContext = {}, error?: Error): void {
    if (!this.config.enabled) return;

    // 检查日志级别
    const levels: LogLevel[] = ['debug', 'info', 'warn', 'error', 'fatal'];
    if (levels.indexOf(level) < levels.indexOf(this.config.level)) {
      return;
    }

    // 创建日志条目
    const logEntry: LogEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      context: {
        appName: this.config.appName,
        environment: this.config.environment,
        userId: localStorage.getItem('userId') ? parseInt(localStorage.getItem('userId')!) : undefined,
        ...context,
      },
      ...(error && { stack: error.stack }),
    };

    // 发送到控制台
    if (this.config.logToConsole) {
      this.logToConsole(logEntry);
    }

    // 发送到服务器
    if (this.config.logToServer && this.config.serverUrl) {
      this.logToServer(logEntry);
    }

    // 发送到可观测性平台
    if (this.observabilityConfig.enabled) {
      this.sendToObservabilityPlatform(logEntry);
    }
  }

  // 发送日志到控制台
  private logToConsole(logEntry: LogEntry): void {
    const { level, message, context, timestamp } = logEntry;
    const prefix = `[${timestamp}] [${level.toUpperCase()}] [${this.config.appName}]`;

    // 为不同级别添加不同颜色
    const colors = {
      debug: '\x1b[36m', // Cyan
      info: '\x1b[32m', // Green
      warn: '\x1b[33m', // Yellow
      error: '\x1b[31m', // Red
      fatal: '\x1b[35m', // Magenta
    };

    const resetColor = '\x1b[0m';
    const color = colors[level] || resetColor;

    // 格式化日志消息
    const formattedMessage = `${color}${prefix} ${message}${resetColor}`;

    // 输出日志
    switch (level) {
      case 'debug':
        console.debug(formattedMessage, context);
        break;
      case 'info':
        console.info(formattedMessage, context);
        break;
      case 'warn':
        console.warn(formattedMessage, context);
        break;
      case 'error':
      case 'fatal':
        console.error(formattedMessage, context);
        break;
    }
  }

  // 发送日志到服务器
  private logToServer(logEntry: LogEntry): void {
    try {
      // 检查fetch是否可用
      if (typeof fetch === 'undefined') {
        console.warn('fetch is not available, cannot send log to server');
        return;
      }

      // 模拟发送日志
      console.log('发送日志到服务器:', logEntry);
    } catch (error) {
      console.error('Error sending log to server:', error);
    }
  }

  // 发送日志到可观测性平台
  private sendToObservabilityPlatform(logEntry: LogEntry): void {
    try {
      const { platform } = this.observabilityConfig;

      switch (platform) {
        case 'datadog':
          this.sendToDatadog(logEntry);
          break;
        case 'newrelic':
          this.sendToNewRelic(logEntry);
          break;
        case 'custom':
          this.sendToCustomPlatform(logEntry);
          break;
        default:
          console.warn(`Unsupported observability platform: ${platform}`);
      }
    } catch (error) {
      console.error('Error sending log to observability platform:', error);
    }
  }

  // 发送到Datadog
  private sendToDatadog(logEntry: LogEntry): void {
    // 实现Datadog日志发送逻辑
    if (!this.observabilityConfig.apiKey || !this.observabilityConfig.endpoint) {
      console.warn('Datadog API key or endpoint not configured');
      return;
    }

    // 检查fetch是否可用
    if (typeof fetch === 'undefined') {
      console.warn('fetch is not available, cannot send log to Datadog');
      return;
    }

    // Datadog日志格式转换和发送
    const datadogLog = {
      ...logEntry,
      service: this.config.appName,
      ddsource: 'browser',
      ddtags: `env:${this.config.environment},level:${logEntry.level}`,
    };

    // 模拟发送到Datadog
    console.log('发送日志到Datadog:', datadogLog);
  }

  // 发送到New Relic
  private sendToNewRelic(logEntry: LogEntry): void {
    // 实现New Relic日志发送逻辑
    if (!this.observabilityConfig.apiKey || !this.observabilityConfig.endpoint) {
      console.warn('New Relic API key or endpoint not configured');
      return;
    }

    // 检查fetch是否可用
    if (typeof fetch === 'undefined') {
      console.warn('fetch is not available, cannot send log to New Relic');
      return;
    }

    const newRelicLog = {
      ...logEntry,
      attributes: {
        serviceName: this.config.appName,
        environment: this.config.environment,
      },
    };

    // 模拟发送到New Relic
    console.log('发送日志到New Relic:', newRelicLog);
  }

  // 发送到自定义可观测性平台
  private sendToCustomPlatform(logEntry: LogEntry): void {
    // 实现自定义平台日志发送逻辑
    if (!this.observabilityConfig.endpoint) {
      console.warn('Custom observability endpoint not configured');
      return;
    }

    // 模拟发送到自定义平台
    console.log('发送日志到自定义平台:', logEntry);
  }

  // 快捷方法：debug级别日志
  public debug(message: string, context?: LogContext): void {
    this.log('debug', message, context);
  }

  // 快捷方法：info级别日志
  public info(message: string, context?: LogContext): void {
    this.log('info', message, context);
  }

  // 快捷方法：warn级别日志
  public warn(message: string, context?: LogContext): void {
    this.log('warn', message, context);
  }

  // 快捷方法：error级别日志
  public error(message: string, context?: LogContext, error?: Error): void {
    this.log('error', message, context, error);
  }

  // 快捷方法：fatal级别日志
  public fatal(message: string, context?: LogContext, error?: Error): void {
    this.log('fatal', message, context, error);
  }

  // API请求日志
  public logApiRequest(message: string, level: LogLevel, context: LogContext, error?: Error): void {
    this.log(
      level,
      message,
      {
        type: 'api',
        ...context,
      },
      error
    );
  }

  // WebSocket日志
  public logWebSocket(message: string, level: LogLevel, context: LogContext, error?: Error): void {
    this.log(
      level,
      message,
      {
        type: 'websocket',
        ...context,
      },
      error
    );
  }

  // 终端命令日志
  public logTerminalCommand(message: string, level: LogLevel, context: LogContext, error?: Error): void {
    this.log(
      level,
      message,
      {
        type: 'terminal',
        ...context,
      },
      error
    );
  }
}

// 创建并导出日志服务实例
export const logService = new LogService(
  {
    enabled: true,
    level: import.meta.env.DEV ? 'debug' : 'info',
    logToConsole: true,
    logToServer: import.meta.env.DEV ? false : true,
    serverUrl: '/api/logs',
    appName: 'claw-tms-client',
    environment: import.meta.env.MODE || 'development',
  },
  {
    enabled: false,
    platform: 'custom',
    endpoint: '/api/observability',
  }
);

// 导出全局日志方法，方便直接使用
export const log = {
  debug: (message: string, context?: LogContext) => logService.debug(message, context),
  info: (message: string, context?: LogContext) => logService.info(message, context),
  warn: (message: string, context?: LogContext) => logService.warn(message, context),
  error: (message: string, context?: LogContext, error?: Error) => logService.error(message, context, error),
  fatal: (message: string, context?: LogContext, error?: Error) => logService.fatal(message, context, error),
  logApiRequest: (message: string, level: LogLevel, context: LogContext, error?: Error) =>
    logService.logApiRequest(message, level, context, error),
  logWebSocket: (message: string, level: LogLevel, context: LogContext, error?: Error) =>
    logService.logWebSocket(message, level, context, error),
  logTerminalCommand: (message: string, level: LogLevel, context: LogContext, error?: Error) =>
    logService.logTerminalCommand(message, level, context, error),
};


