/**
 * Electron 主进程日志工具
 * 提供增强的日志记录功能，支持文件输出和结构化日志
 */
import path from 'path';
import fs from 'fs';
import { app } from 'electron';

/**
 * 日志级别
 */
const LogLevel = {
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
  FATAL: 4,
};

/**
 * 日志级别名称
 */
const LogLevelNames = {
  0: 'DEBUG',
  1: 'INFO',
  2: 'WARN',
  3: 'ERROR',
  4: 'FATAL',
};

/**
 * 日志级别颜色
 */
const LogLevelColors = {
  0: '\x1b[36m', // Cyan
  1: '\x1b[32m', // Green
  2: '\x1b[33m', // Yellow
  3: '\x1b[31m', // Red
  4: '\x1b[35m', // Magenta
};

const resetColor = '\x1b[0m';

/**
 * 日志配置
 */
class LoggerConfig {
  constructor() {
    this.level = LogLevel.INFO;
    this.logToConsole = true;
    this.logToFile = true;
    this.logDir = path.join(app.getPath('userData'), 'logs');
    this.maxFileSize = 10 * 1024 * 1024; // 10MB
    this.maxFiles = 10;
    this.enableTimestamp = true;
    this.enableContext = true;

    // 确保日志目录存在
    if (!fs.existsSync(this.logDir)) {
      fs.mkdirSync(this.logDir, { recursive: true });
    }
  }
}

/**
 * 日志记录器类
 */
class Logger {
  constructor(config = {}) {
    this.config = { ...new LoggerConfig(), ...config };
    this.currentLogFile = this.getLogFileName();
    this.requestIdCounter = 0;

    // 监听文件大小，实现日志轮转
    this.startLogRotation();
  }

  /**
   * 生成日志文件名
   */
  getLogFileName() {
    const date = new Date().toISOString().split('T')[0];
    return path.join(this.config.logDir, `CarpTMS-${date}.log`);
  }

  /**
   * 开始日志轮转监控
   */
  startLogRotation() {
    setInterval(() => {
      this.rotateLogFile();
    }, 60000); // 每分钟检查一次
  }

  /**
   * 日志轮转
   */
  rotateLogFile() {
    try {
      if (!fs.existsSync(this.currentLogFile)) {
        return;
      }

      const stats = fs.statSync(this.currentLogFile);

      // 如果文件大小超过限制，进行轮转
      if (stats.size >= this.config.maxFileSize) {
        const timestamp = Date.now();
        const archiveName = this.currentLogFile.replace('.log', `-${timestamp}.log`);

        // 重命名当前日志文件
        fs.renameSync(this.currentLogFile, archiveName);

        // 清理旧日志文件
        this.cleanupOldLogFiles();

        // 更新当前日志文件名
        this.currentLogFile = this.getLogFileName();

        console.log(`[Logger] 日志轮转完成: ${archiveName}`);
      }
    } catch (error) {
      console.error('[Logger] 日志轮转失败:', error);
    }
  }

  /**
   * 清理旧日志文件
   */
  cleanupOldLogFiles() {
    try {
      const files = fs
        .readdirSync(this.config.logDir)
        .filter((file) => file.startsWith('CarpTMS-') && file.endsWith('.log'))
        .map((file) => ({
          name: file,
          path: path.join(this.config.logDir, file),
          time: fs.statSync(path.join(this.config.logDir, file)).mtime.getTime(),
        }))
        .sort((a, b) => b.time - a.time);

      // 删除超过最大文件数的旧日志
      if (files.length > this.config.maxFiles) {
        files.slice(this.config.maxFiles).forEach((file) => {
          fs.unlinkSync(file.path);
          console.log(`[Logger] 删除旧日志文件: ${file.name}`);
        });
      }
    } catch (error) {
      console.error('[Logger] 清理旧日志文件失败:', error);
    }
  }

  /**
   * 生成唯一请求ID
   */
  generateRequestId() {
    this.requestIdCounter++;
    return `${Date.now()}-${this.requestIdCounter}-${Math.random().toString(36).substring(2, 9)}`;
  }

  /**
   * 格式化日志条目
   */
  formatLogEntry(level, message, context = {}, error = null) {
    const entry = {
      timestamp: new Date().toISOString(),
      level: LogLevelNames[level],
      message,
      ...(this.config.enableContext && { context }),
      ...(error && {
        error: {
          message: error.message,
          stack: error.stack,
        },
      }),
    };

    return entry;
  }

  /**
   * 输出日志到控制台
   */
  logToConsole(entry) {
    const { level, message, context, timestamp, error } = entry;
    const color = LogLevelColors[LogLevel[level]] || resetColor;
    const prefix = this.config.enableTimestamp ? `[${timestamp}] [${level}]` : `[${level}]`;

    let formattedMessage = `${color}${prefix} ${message}${resetColor}`;

    if (this.config.enableContext && Object.keys(context).length > 0) {
      formattedMessage += ' ' + JSON.stringify(context);
    }

    switch (level) {
      case 'DEBUG':
        console.debug(formattedMessage);
        break;
      case 'INFO':
        console.info(formattedMessage);
        break;
      case 'WARN':
        console.warn(formattedMessage);
        break;
      case 'ERROR':
      case 'FATAL':
        console.error(formattedMessage);
        if (error) {
          console.error(error.stack);
        }
        break;
    }
  }

  /**
   * 输出日志到文件
   */
  logToFile(entry) {
    try {
      const logLine = JSON.stringify(entry) + '\n';
      fs.appendFileSync(this.currentLogFile, logLine, 'utf8');
    } catch (error) {
      console.error('[Logger] 写入日志文件失败:', error);
    }
  }

  /**
   * 记录日志
   */
  log(level, message, context = {}, error = null) {
    // 检查日志级别
    if (level < this.config.level) {
      return;
    }

    const entry = this.formatLogEntry(level, message, context, error);

    // 输出到控制台
    if (this.config.logToConsole) {
      this.logToConsole(entry);
    }

    // 输出到文件
    if (this.config.logToFile) {
      this.logToFile(entry);
    }
  }

  // 快捷方法
  debug(message, context = {}) {
    this.log(LogLevel.DEBUG, message, context);
  }

  info(message, context = {}) {
    this.log(LogLevel.INFO, message, context);
  }

  warn(message, context = {}) {
    this.log(LogLevel.WARN, message, context);
  }

  error(message, context = {}, error = null) {
    this.log(LogLevel.ERROR, message, context, error);
  }

  fatal(message, context = {}, error = null) {
    this.log(LogLevel.FATAL, message, context, error);
  }

  // 特定场景的日志方法

  /**
   * TCP 连接日志
   */
  logTcpConnection(action, data = {}) {
    this.info(`TCP ${action}`, {
      type: 'tcp',
      host: tcpHost,
      port: tcpPort,
      ...data,
    });
  }

  /**
   * IPC 请求日志
   */
  logIpcRequest(channel, data = {}) {
    this.debug(`IPC 请求: ${channel}`, {
      type: 'ipc',
      channel,
      ...data,
    });
  }

  /**
   * IPC 响应日志
   */
  logIpcResponse(channel, success = true, data = {}) {
    this.debug(`IPC 响应: ${channel}`, {
      type: 'ipc',
      channel,
      success,
      ...data,
    });
  }

  /**
   * 数据库操作日志
   */
  logDatabase(operation, table, data = {}) {
    this.info(`数据库操作: ${operation} ${table}`, {
      type: 'database',
      operation,
      table,
      ...data,
    });
  }

  /**
   * 设置日志级别
   */
  setLevel(level) {
    if (LogLevel[level.toUpperCase()] !== undefined) {
      this.config.level = LogLevel[level.toUpperCase()];
      this.info(`日志级别已设置为: ${level.toUpperCase()}`);
    }
  }

  /**
   * 获取当前配置
   */
  getConfig() {
    return { ...this.config };
  }

  /**
   * 获取日志文件路径
   */
  getLogFilePath() {
    return this.currentLogFile;
  }
}

// 导出单例实例
const logger = new Logger();

// 在开发环境中设置为 DEBUG 级别
if (process.env.NODE_ENV === 'development') {
  logger.setLevel('DEBUG');
}

export default logger;
export { Logger, LogLevel, LogLevelNames };


