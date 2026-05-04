type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LoggerConfig {
  enabled: boolean;
  level: LogLevel;
  prefix: string;
}

const LOG_LEVELS: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

class Logger {
  private config: LoggerConfig;

  constructor() {
    this.config = {
      enabled: !import.meta.env.PROD,
      level: import.meta.env.PROD ? 'error' : 'debug',
      prefix: '[CarpTMS]',
    };
  }

  setEnabled(enabled: boolean): void {
    this.config.enabled = enabled;
  }

  setLevel(level: LogLevel): void {
    this.config.level = level;
  }

  private shouldLog(level: LogLevel): boolean {
    if (!this.config.enabled) {
      return false;
    }
    return LOG_LEVELS[level] >= LOG_LEVELS[this.config.level];
  }

  private formatMessage(level: LogLevel, ...args: unknown[]): unknown[] {
    const timestamp = new Date().toISOString();
    const levelTag = level.toUpperCase().padEnd(5);
    return [`%c${this.config.prefix}`, 'color: #409eff; font-weight: bold;', `[${timestamp}] [${levelTag}]`, ...args];
  }

  debug(...args: unknown[]): void {
    if (this.shouldLog('debug')) {
      console.debug(...this.formatMessage('debug', ...args));
    }
  }

  info(...args: unknown[]): void {
    if (this.shouldLog('info')) {
      console.info(...this.formatMessage('info', ...args));
    }
  }

  warn(...args: unknown[]): void {
    if (this.shouldLog('warn')) {
      console.warn(...this.formatMessage('warn', ...args));
    }
  }

  error(...args: unknown[]): void {
    if (this.shouldLog('error')) {
      console.error(...this.formatMessage('error', ...args));
    }
  }

  group(label: string): void {
    if (this.shouldLog('debug')) {
      console.group(label);
    }
  }

  groupEnd(): void {
    if (this.shouldLog('debug')) {
      console.groupEnd();
    }
  }

  time(label: string): void {
    if (this.shouldLog('debug')) {
      console.time(label);
    }
  }

  timeEnd(label: string): void {
    if (this.shouldLog('debug')) {
      console.timeEnd(label);
    }
  }

  table(data: unknown): void {
    if (this.shouldLog('debug')) {
      console.table(data);
    }
  }
}

export const logger = new Logger();

export function createLogger(moduleName: string): Logger {
  const moduleLogger = new Logger();
  return moduleLogger;
}

export default logger;
