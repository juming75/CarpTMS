/**
 * 后端状态监测服务
 *
 * 职责：
 * 1. 定期通过 HTTP /api/health/live 检测后端存活状态
 * 2. 通过深度检查 /api/health 检测后端健康状态（DB、Redis、Cache）
 * 3. 维护后端连接状态机
 * 4. 提供全局响应式状态，供 UI 层展示（如顶部状态栏）
 * 5. 智能抑制日志噪音：连续失败时只记录摘要
 * 6. 支持网络状态感知：online/offline 自动调整检测策略
 */

import { logService } from './logService';

// ==================== 类型定义 ====================

export type BackendStatus = 'unknown' | 'healthy' | 'degraded' | 'down' | 'checking';

export interface HealthCheckResult {
  status: BackendStatus;
  responseTimeMs: number;
  timestamp: number;
  details?: {
    database: string;
    redis: string;
    cache?: { hit_rate: number; hits: number; misses: number };
    hostname?: string;
  };
  error?: string;
}

export interface BackendMonitorConfig {
  /** 健康检查端点（轻量级存活探测） */
  livenessUrl: string;
  /** 深度健康检查端点 */
  deepHealthUrl: string;
  /** 存活探测间隔（ms）- 后端正常时使用 */
  livenessIntervalMs: number;
  /** 深度检查间隔（ms）- 较低频率 */
  deepHealthIntervalMs: number;
  /** HTTP 超时时间（ms） */
  timeoutMs: number;
  /** 连续失败多少次判定为 DOWN */
  failureThreshold: number;
  /** 连续成功多少次判定为恢复 */
  recoveryThreshold: number;
}

export type StatusChangeCallback = (status: BackendStatus, previousStatus: BackendStatus, result: HealthCheckResult) => void;

// ==================== 默认配置 ====================

const DEFAULT_CONFIG: BackendMonitorConfig = {
  // 开发环境使用 Vite 代理的相对路径
  livenessUrl: '/api/health/live',
  deepHealthUrl: '/api/health',
  livenessIntervalMs: 100000, // 100秒
  deepHealthIntervalMs: 300000, // 300秒(5分钟)
  timeoutMs: 10000,
  failureThreshold: 3,
  recoveryThreshold: 2,
};

// ==================== 单例 ====================

class BackendMonitorService {
  private config: BackendMonitorConfig;
  private currentStatus: BackendStatus = 'unknown';
  private previousStatus: BackendStatus = 'unknown';
  private consecutiveFailures = 0;
  private consecutiveSuccesses = 0;
  private lastResult: HealthCheckResult | null = null;

  private livenessTimer: ReturnType<typeof setInterval> | null = null;
  private deepHealthTimer: ReturnType<typeof setTimeout> | null = null;
  private isChecking = false;

  private listeners: Set<StatusChangeCallback> = new Set();
  private verboseLogLimit = 2; // 只详细记录前N次失败

  constructor(config?: Partial<BackendMonitorConfig>) {
    this.config = { ...DEFAULT_CONFIG, ...config };

    // 监听网络状态变化
    if (typeof window !== 'undefined') {
      window.addEventListener('online', this.handleNetworkOnline);
      window.addEventListener('offline', this.handleNetworkOffline);
    }
  }

  // ==================== 公共 API ====================

  /** 启动监测 */
  start(): void {
    if (this.livenessTimer) return; // 已启动

    logService.logWebSocket('BackendMonitor 已启动', 'info', {
      livenessInterval: `${this.config.livenessIntervalMs}ms`,
      deepHealthInterval: `${this.config.deepHealthIntervalMs}ms`,
    });

    // 立即执行首次检查
    this.performLivenessCheck();

    // 定期存活检查
    this.livenessTimer = window.setInterval(() => {
      this.performLivenessCheck();
    }, this.config.livenessIntervalMs);

    // 定期深度检查
    this.scheduleDeepHealthCheck();
  }

  /** 停止监测 */
  stop(): void {
    if (this.livenessTimer) {
      clearInterval(this.livenessTimer);
      this.livenessTimer = null;
    }
    if (this.deepHealthTimer) {
      clearTimeout(this.deepHealthTimer);
      this.deepHealthTimer = null;
    }
    this.currentStatus = 'unknown';

    if (typeof window !== 'undefined') {
      window.removeEventListener('online', this.handleNetworkOnline);
      window.removeEventListener('offline', this.handleNetworkOffline);
    }

    logService.logWebSocket('BackendMonitor 已停止', 'info');
  }

  /** 获取当前状态 */
  getStatus(): BackendStatus {
    return this.currentStatus;
  }

  /** 获取最后一次检查结果 */
  getLastResult(): HealthCheckResult | null {
    return this.lastResult;
  }

  /** 后端是否可用（用于 WebSocket 连接前的预检） */
  isAvailable(): boolean {
    return this.currentStatus === 'healthy' || this.currentStatus === 'degraded';
  }

  /** 注册状态变更监听器 */
  onStatusChange(callback: StatusChangeCallback): () => void {
    this.listeners.add(callback);
    // 返回取消订阅函数
    return () => this.listeners.delete(callback);
  }

  /** 手动触发一次检查 */
  async checkNow(): Promise<HealthCheckResult> {
    await this.performLivenessCheck();
    return this.lastResult!;
  }

  /** 更新配置（运行时可动态调整） */
  updateConfig(partialConfig: Partial<BackendMonitorConfig>): void {
    Object.assign(this.config, partialConfig);

    // 如果间隔改变，重启定时器
    if (partialConfig.livenessIntervalMs && this.livenessTimer) {
      this.stop();
      this.start();
    }
  }

  // ==================== 私有方法 - 存活检查 ====================

  private performLivenessCheck = async (): Promise<void> => {
    if (this.isChecking) return;
    this.isChecking = true;
    this.updateStatus('checking');

    const startTime = performance.now();

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.config.timeoutMs);

      const response = await fetch(this.config.livenessUrl, {
        method: 'GET',
        signal: controller.signal,
        headers: { Accept: 'application/json' },
        cache: 'no-store', // 禁用缓存
      });

      clearTimeout(timeoutId);

      const responseTimeMs = Math.round(performance.now() - startTime);

      if (response.ok) {
        const data = (await response.json().catch(() => ({}))) as Record<string, unknown>;
        this.handleSuccess({
          status: data.status === 'ok' ? 'healthy' : 'degraded',
          responseTimeMs,
          timestamp: Date.now(),
          details: data as HealthCheckResult['details'],
        });
      } else {
        throw new Error(`HTTP ${response.status}`);
      }
    } catch (error) {
      const responseTimeMs = Math.round(performance.now() - startTime);
      this.handleFailure(responseTimeMs, error instanceof Error ? error.message : String(error));
    } finally {
      this.isChecking = false;
    }
  };

  // ==================== 私有方法 - 深度检查 ====================

  private scheduleDeepHealthCheck(): void {
    this.deepHealthTimer = window.setTimeout(async () => {
      if (!this.livenessTimer) return; // 已停止

      await this.performDeepHealthCheck();

      // 安排下一次
      this.scheduleDeepHealthCheck();
    }, this.config.deepHealthIntervalMs);
  }

  private async performDeepHealthCheck(): Promise<void> {
    // 只有在 healthy 或 degraded 时才做深度检查（避免给 down 的服务器增加压力）
    if (!this.isAvailable()) return;

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.config.timeoutMs);

      const response = await fetch(this.config.deepHealthUrl, {
        method: 'GET',
        signal: controller.signal,
        headers: { Accept: 'application/json' },
        cache: 'no-store',
      });

      clearTimeout(timeoutId);

      if (response.ok) {
        const data = (await response.json()) as HealthCheckResult & { dependencies?: Record<string, unknown> };
        this.lastResult = {
          ...this.lastResult!,
          details: {
            ...this.lastResult!.details,
            database: (data.dependencies?.database as string) || 'unknown',
            redis: (data.dependencies?.redis as string) || 'unknown',
            cache: data.cache as HealthCheckResult['details']['cache'],
            hostname: data.hostname as string,
          },
        };

        // 深度检查发现降级
        if ((data.dependencies?.database as string) === 'error') {
          this.updateStatus('degraded');
          logService.logWebSocket('后端数据库异常', 'warn', {
            url: this.config.deepHealthUrl,
            db_status: data.dependencies?.database,
          });
        }
      }
    } catch {
      // 深度检查失败不影响主状态
    }
  }

  // ==================== 私有方法 - 状态管理 ====================

  private handleSuccess(result: HealthCheckResult): void {
    this.consecutiveFailures = 0;
    this.consecutiveSuccesses++;
    this.lastResult = result;

    // 阈值判断恢复
    if (
      (this.currentStatus === 'down' || this.currentStatus === 'degraded' || this.currentStatus === 'unknown') &&
      this.consecutiveSuccesses >= this.config.recoveryThreshold
    ) {
      this.transitionTo(result.status, result);
    } else if (this.currentStatus === 'checking') {
      // 首次检查成功
      this.transitionTo(result.status, result);
    } else {
      // 保持当前状态但更新结果
      this.currentStatus = result.status;
    }
  }

  private handleFailure(responseTimeMs: number, errorMessage: string): void {
    this.consecutiveSuccesses = 0;
    this.consecutiveFailures++;

    const result: HealthCheckResult = {
      status: 'down',
      responseTimeMs,
      timestamp: Date.now(),
      error: errorMessage,
    };
    this.lastResult = result;

    // 日志抑制策略
    if (this.consecutiveFailures <= this.verboseLogLimit) {
      logService.logWebSocket('后端不可达', 'warn', {
        url: this.config.livenessUrl,
        attempt: this.consecutiveFailures,
        maxAttempts: this.config.failureThreshold,
        error: errorMessage.length > 80 ? `${errorMessage.substring(0, 80)}...` : errorMessage,
        responseTime: responseTimeMs,
      });
    } else if (this.consecutiveFailures === this.verboseLogLimit + 1) {
      logService.logWebSocket(
        `后端持续不可达 (${this.consecutiveFailures}次)，后续仅记录摘要`,
        'warn'
      );
    } else if (this.consecutiveFailures % 10 === 0) {
      // 每10次记录一次摘要
      logService.logWebSocket(`后端持续不可达，已连续失败 ${this.consecutiveFailures} 次`, 'warn');
    }

    // 达到失败阈值 → 判定为 DOWN
    if (this.consecutiveFailures >= this.config.failureThreshold) {
      if (this.currentStatus !== 'down') {
        this.transitionTo('down', result);
      }
    } else if (this.currentStatus === 'checking' || this.currentStatus === 'healthy') {
      // 首次失败或从 healthy 变化
      this.transitionTo('down', result);
    }
  }

  private updateStatus(newStatus: BackendStatus): void {
    if (newStatus === this.currentStatus) return;
    this.previousStatus = this.currentStatus;
    this.currentStatus = newStatus;
  }

  private transitionTo(newStatus: BackendStatus, result: HealthCheckResult): void {
    this.previousStatus = this.currentStatus;
    this.currentStatus = newStatus;

    // 重置计数器
    switch (newStatus) {
      case 'healthy':
      case 'degraded':
        this.consecutiveFailures = 0;
        break;
      case 'down':
        this.consecutiveSuccesses = 0;
        break;
    }

    // 通知监听者
    this.notifyListeners(newStatus, this.previousStatus!, result);

    // 记录关键状态变化
    if (this.previousStatus !== 'checking' && this.previousStatus !== newStatus) {
      const level = newStatus === 'healthy' ? 'info' : 'warn';
      logService.logWebSocket(
        `后端状态: ${this.previousStatus} -> ${newStatus}`,
        level,
        { responseTime: result.responseTimeMs, error: result.error }
      );
    }
  }

  private notifyListeners(status: BackendStatus, previous: BackendStatus, result: HealthCheckResult): void {
    for (const listener of this.listeners) {
      try {
        listener(status, previous, result);
      } catch (e) {
        console.error('[BackendMonitor] Listener error:', e);
      }
    }
  }

  // ==================== 网络事件处理 ====================

  private handleNetworkOnline = (): void => {
    logService.logWebSocket('网络已恢复，立即执行后端检测', 'info');
    this.consecutiveFailures = 0;
    // 立即检查
    this.performLivenessCheck();
  };

  private handleNetworkOffline = (): void => {
    logService.logWebSocket('网络断开', 'warn');
    this.transitionTo('down', {
      status: 'down',
      responseTimeMs: 0,
      timestamp: Date.now(),
      error: 'network_offline',
    });
  };

  // ==================== 清理 =================

  destroy(): void {
    this.stop();
    this.listeners.clear();
  }
}

// ==================== 导出单例 ====================

/** 全局后端监控服务实例 */
export const backendMonitor = new BackendMonitorService();

// 自动导出类型和实例
export default BackendMonitorService;
