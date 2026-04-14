// 监控服务
// 集成性能监控、错误监控和用户行为监控
/* global PerformanceEntry, RequestInit, PerformanceNavigationTiming, PerformancePaintTiming, MouseEvent, SubmitEvent, HTMLFormElement */

import { recordPerformanceMetric, performanceConfig } from './performance';
import { ErrorHandler, AppError, ErrorSeverity } from './errorHandler';
import { logService } from './logService';

interface PerformanceEntryWithAttribution extends PerformanceEntry {
  attribution: unknown[];
}

interface PerformanceEventTimingEntry extends PerformanceEntry {
  entryType: string;
  name: string;
  startTime: number;
  processingStart: number;
  processingEnd: number;
}

interface LayoutShiftEntry extends PerformanceEntry {
  hadRecentInput: boolean;
  value: number;
}

interface MemoryInfo {
  usedJSHeapSize: number;
  totalJSHeapSize: number;
  jsHeapSizeLimit: number;
}

// 监控类型
enum MonitorType {
  PERFORMANCE = 'performance',
  ERROR = 'error',
  USER_BEHAVIOR = 'user_behavior',
}

// 性能指标类型
enum PerformanceMetric {
  PAGE_LOAD = 'page_load',
  COMPONENT_RENDER = 'component_render',
  API_REQUEST = 'api_request',
  DATA_PROCESSING = 'data_processing',
  MEMORY_USAGE = 'memory_usage',
  CPU_USAGE = 'cpu_usage',
}

// 用户行为类型
enum UserAction {
  CLICK = 'click',
  NAVIGATION = 'navigation',
  FORM_SUBMIT = 'form_submit',
  SEARCH = 'search',
  FILTER = 'filter',
}

// 监控数据接口
interface MonitorData {
  id: string;
  type: MonitorType;
  timestamp: Date;
  category: string;
  name: string;
  value?: number;
  metadata?: Record<string, unknown>;
}

// 监控服务类
class MonitoringService {
  private buffer: MonitorData[] = [];
  private bufferSize = 100;
  private flushInterval = 5000; // 5秒
  private flushTimer: ReturnType<typeof setTimeout> | null = null;
  private isInitialized = false;

  // 初始化监控服务
  init() {
    if (this.isInitialized) return;

    this.isInitialized = true;
    this.setupPerformanceMonitoring();
    this.setupErrorMonitoring();
    this.setupUserBehaviorMonitoring();
    this.startFlushTimer();

    logService.info('监控服务初始化成功');
  }

  // 设置性能监控
  private setupPerformanceMonitoring() {
    if (typeof window !== 'undefined' && window.performance) {
      // 监听页面加载性能
      if (performanceConfig.enableComponentRenderMonitoring) {
        this.monitorPageLoad();
      }

      // 监听内存使用
      this.monitorMemoryUsage();

      // 监听网络请求
      this.monitorNetworkRequests();

      // 监听长任务
      this.monitorLongTasks();

      // 监听首次输入延迟
      this.monitorFirstInputDelay();

      // 监听累积布局偏移
      this.monitorCumulativeLayoutShift();
    }
  }

  // 监控网络请求
  private monitorNetworkRequests() {
    if (typeof window !== 'undefined' && window.performance && 'measure' in window.performance) {
      // 保存当前实例的引用
      const self = this;

      // 重写fetch
      const originalFetch = window.fetch;
      window.fetch = async (...args) => {
        const url = args[0] as string;
        const start = window.performance.now();

        try {
          const response = await originalFetch(...args);
          const end = window.performance.now();
          const duration = end - start;

          self.recordPerformance(PerformanceMetric.API_REQUEST, duration, {
            url,
            status: response.status,
            statusText: response.statusText,
            method: (args[1] as RequestInit)?.method || 'GET',
          });

          return response;
        } catch (error) {
          const end = window.performance.now();
          const duration = end - start;

          self.recordPerformance(PerformanceMetric.API_REQUEST, duration, {
            url,
            error: String(error),
            method: (args[1] as RequestInit)?.method || 'GET',
          });

          throw error;
        }
      };

      // 重写XMLHttpRequest
      if (window.XMLHttpRequest) {
        const originalXHROpen = window.XMLHttpRequest.prototype.open;
        window.XMLHttpRequest.prototype.open = function (method: string, url: string | URL, async: boolean = true, username?: string | null, password?: string | null) {
          const start = window.performance.now();

          this.addEventListener('load', () => {
            const end = window.performance.now();
            const duration = end - start;

            self.recordPerformance(PerformanceMetric.API_REQUEST, duration, {
              url: url as string,
              status: this.status,
              statusText: this.statusText,
              method,
            });
          });

          this.addEventListener('error', () => {
            const end = window.performance.now();
            const duration = end - start;

            self.recordPerformance(PerformanceMetric.API_REQUEST, duration, {
              url: url as string,
              error: 'Network error',
              method,
            });
          });

          return originalXHROpen.call(this, method, url, async, username, password);
        };
      }
    }
  }

  // 监控长任务
  private monitorLongTasks() {
    if (typeof window !== 'undefined' && 'LongTaskTiming' in window && window.PerformanceObserver) {
      const observer = new window.PerformanceObserver((list) => {
        list.getEntries().forEach((entry) => {
          if (entry.duration > 50) {
            // 超过50ms的任务
            this.recordPerformance('long_task' as PerformanceMetric, entry.duration, {
              entryType: entry.entryType,
              startTime: entry.startTime,
              duration: entry.duration,
              attribution: (entry as PerformanceEntryWithAttribution).attribution || [],
            });
          }
        });
      });

      observer.observe({ entryTypes: ['longtask'] });
    }
  }

  // 监控首次输入延迟
  private monitorFirstInputDelay() {
    if (typeof window !== 'undefined' && 'PerformanceEventTiming' in window && window.PerformanceObserver) {
      const observer = new window.PerformanceObserver((list) => {
        list.getEntries().forEach((entry) => {
          const entryWithProcessing = entry as PerformanceEventTimingEntry;
          this.recordPerformance(
            'first_input_delay' as PerformanceMetric,
            entryWithProcessing.processingStart - entryWithProcessing.startTime,
            {
              entryType: entry.entryType,
              name: entry.name,
              startTime: entry.startTime,
              processingStart: entryWithProcessing.processingStart,
              processingEnd: entryWithProcessing.processingEnd,
            }
          );
        });
      });

      observer.observe({ type: 'first-input', buffered: true });
    }
  }

  // 监控累积布局偏移
  private monitorCumulativeLayoutShift() {
    if (typeof window !== 'undefined' && 'LayoutShift' in window && window.PerformanceObserver) {
      const observer = new window.PerformanceObserver((list) => {
        let cumulativeScore = 0;
        list.getEntries().forEach((entry) => {
          const layoutEntry = entry as LayoutShiftEntry;
          if (!layoutEntry.hadRecentInput) {
            cumulativeScore += layoutEntry.value;
          }
        });

        if (cumulativeScore > 0) {
          this.recordPerformance('cumulative_layout_shift' as PerformanceMetric, cumulativeScore, {
            score: cumulativeScore,
          });
        }
      });

      observer.observe({ type: 'layout-shift', buffered: true });
    }
  }

  // 设置错误监控
  private setupErrorMonitoring() {
    // 已在 errorHandler.ts 中设置
  }

  // 设置用户行为监控
  private setupUserBehaviorMonitoring() {
    if (typeof window !== 'undefined') {
      // 点击事件监控
      document.addEventListener('click', this.trackClick.bind(this), true);

      // 表单提交监控
      document.addEventListener('submit', this.trackFormSubmit.bind(this), true);

      // 页面导航监控
      if (window.history) {
        this.trackNavigation();
      }
    }
  }

  // 启动刷新定时器
  private startFlushTimer() {
    this.flushTimer = setInterval(() => {
      this.flushBuffer();
    }, this.flushInterval);
  }

  // 监控页面加载性能
  private monitorPageLoad() {
    if (typeof window !== 'undefined' && window.performance) {
      window.addEventListener('load', () => {
        const navigationTiming = window.performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
        if (navigationTiming) {
          const metrics = {
            pageLoadTime: navigationTiming.loadEventEnd - navigationTiming.startTime,
            domContentLoaded: navigationTiming.domContentLoadedEventEnd - navigationTiming.startTime,
            firstPaint:
              window.performance.getEntriesByType('paint').find((p: PerformancePaintTiming) => p.name === 'first-paint')?.startTime || 0,
            firstContentfulPaint:
              window.performance.getEntriesByType('paint').find((p: PerformancePaintTiming) => p.name === 'first-contentful-paint')
                ?.startTime || 0,
          };

          this.recordPerformance(PerformanceMetric.PAGE_LOAD, metrics.pageLoadTime, metrics);
        }
      });
    }
  }

  // 监控内存使用
  private monitorMemoryUsage() {
    if (typeof window !== 'undefined' && window.performance && (window.performance as unknown as { memory?: MemoryInfo }).memory) {
      setInterval(() => {
        const memory = (window.performance as unknown as { memory?: MemoryInfo }).memory;
        if (memory) {
          this.recordPerformance(PerformanceMetric.MEMORY_USAGE, memory.usedJSHeapSize / (1024 * 1024), {
            usedHeapSize: memory.usedJSHeapSize,
            totalHeapSize: memory.totalJSHeapSize,
            heapSizeLimit: memory.jsHeapSizeLimit,
          });
        }
      }, 30000);
    }
  }

  // 追踪点击事件
  private trackClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const selector = this.getElementSelector(target);
    const text = target.textContent?.trim() || '';

    this.recordUserAction(UserAction.CLICK, {
      selector,
      text,
      x: event.clientX,
      y: event.clientY,
      timestamp: new Date().toISOString(),
    });
  }

  // 追踪表单提交
  private trackFormSubmit(event: SubmitEvent) {
    const form = event.target as HTMLFormElement;
    if (!form || typeof form !== 'object') return;
    const action = form.action || '';
    const method = form.method || 'GET';

    this.recordUserAction(UserAction.FORM_SUBMIT, {
      action,
      method,
      fields: this.extractFormFields(form),
    });
  }

  // 追踪导航
  private trackNavigation() {
    if (typeof window !== 'undefined' && window.history) {
      const originalPushState = window.history.pushState;
      const originalReplaceState = window.history.replaceState;

      window.history.pushState = function (...args) {
        const url = args[2] as string;
        window.dispatchEvent(new CustomEvent('navigation', { detail: { url, type: 'pushState' } }));
        return originalPushState.apply(this, args);
      };

      window.history.replaceState = function (...args) {
        const url = args[2] as string;
        window.dispatchEvent(new CustomEvent('navigation', { detail: { url, type: 'replaceState' } }));
        return originalReplaceState.apply(this, args);
      };

      window.addEventListener('popstate', (_event) => {
        window.dispatchEvent(
          new CustomEvent('navigation', { detail: { url: window.location.href, type: 'popstate' } })
        );
      });

      window.addEventListener('navigation', (event: Event) => {
        const customEvent = event as CustomEvent<{ url: string; type: string }>;
        const detail = customEvent.detail;
        this.recordUserAction(UserAction.NAVIGATION, {
          url: detail.url,
          type: detail.type,
          referrer: document.referrer,
        });
      });
    }
  }

  // 获取元素选择器
  private getElementSelector(element: HTMLElement): string {
    if (!element) return '';

    const parts: string[] = [];
    let current: HTMLElement | null = element;

    while (current && current !== document.body) {
      const tagName = current.tagName.toLowerCase();
      const id = current.id ? `#${current.id}` : '';
      let className = '';

      // 确保 className 是字符串类型
      if (current.className && typeof current.className === 'string') {
        className = `.${current.className
          .split(' ')
          .filter((c) => c)
          .join('.')}`;
      }

      if (id) {
        parts.unshift(`${tagName}${id}`);
        break;
      } else if (className) {
        parts.unshift(`${tagName}${className}`);
      } else {
        parts.unshift(tagName);
      }

      current = current.parentElement;
    }

    return parts.join(' > ');
  }

  // 提取表单字段
  private extractFormFields(form: HTMLFormElement): Record<string, string> {
    const fields: Record<string, string> = {};
    if (typeof window === 'undefined' || typeof window.FormData === 'undefined' || !form) return fields;

    try {
      const formData = new window.FormData(form);

      formData.forEach((value, key) => {
        // 不记录敏感字段
        if (!['password', 'token', 'creditcard'].includes(key.toLowerCase())) {
          fields[key] = String(value);
        } else {
          fields[key] = '[REDACTED]';
        }
      });
    } catch (error) {
      console.warn('提取表单字段失败:', error);
    }

    return fields;
  }

  // 记录性能指标
  recordPerformance(metric: PerformanceMetric, value: number, metadata?: Record<string, unknown>) {
    const data: MonitorData = {
      id: `perf_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type: MonitorType.PERFORMANCE,
      timestamp: new Date(),
      category: metric === 'page_load' ? 'PAGE_LOAD' : metric,
      name: metric,
      value,
      metadata,
    };

    this.buffer.push(data);
    recordPerformanceMetric(metric, value, metadata as Record<string, unknown>);
    this.checkBuffer();
  }

  // 记录错误
  recordError(error: Error | AppError | unknown, context?: string) {
    const appError = ErrorHandler.handle(error);
    const data: MonitorData = {
      id: `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type: MonitorType.ERROR,
      timestamp: new Date(),
      category: appError.type,
      name: appError.name,
      metadata: {
        message: appError.message,
        userMessage: appError.userMessage,
        severity: appError.severity,
        technicalDetails: appError.technicalDetails,
        recoverySuggestion: appError.recoverySuggestion,
        requestId: appError.requestId,
        context,
      },
    };

    this.buffer.push(data);
    this.checkBuffer();

    // 严重错误立即上报
    if (appError.severity === ErrorSeverity.CRITICAL) {
      this.flushBuffer();
    }
  }

  // 记录用户行为
  recordUserAction(action: UserAction, metadata?: Record<string, unknown>) {
    const data: MonitorData = {
      id: `user_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type: MonitorType.USER_BEHAVIOR,
      timestamp: new Date(),
      category: action === 'click' ? 'CLICK' : action,
      name: action,
      metadata,
    };

    this.buffer.push(data);
    this.checkBuffer();
  }

  // 检查缓冲区大小
  private checkBuffer() {
    if (this.buffer.length >= this.bufferSize) {
      this.flushBuffer();
    }
  }

  // 刷新缓冲区
  private flushBuffer() {
    if (this.buffer.length === 0) return;

    const dataToSend = [...this.buffer];
    this.buffer = [];

    // 发送监控数据
    this.sendMonitoringData(dataToSend);
  }

  // 发送监控数据
  private sendMonitoringData(data: MonitorData[]) {
    // 在开发环境中仅打印
    if (import.meta.env?.DEV) {
      console.log('监控数据:', data);
      return;
    }

    // 生产环境中可以发送到服务器
    // 这里可以集成第三方监控服务如 Sentry、New Relic 等
    try {
      // 模拟发送数据
      console.log('发送监控数据到服务器:', data.length, '条');

      // 实际实现可以使用 fetch 或 axios
      /*
      fetch('/api/monitoring', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(data)
      })
      */
    } catch (error) {
      console.error('发送监控数据失败:', error);
    }
  }

  // 获取监控数据
  getMonitorData(): MonitorData[] {
    return [...this.buffer];
  }

  // 清理
  cleanup() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
    this.buffer = [];
    this.isInitialized = false;
  }
}

// 创建监控服务实例
const monitoringService = new MonitoringService();

// 导出监控服务
export { monitoringService as MonitoringService };

// 导出便捷函数
export const recordPerformance = monitoringService.recordPerformance.bind(monitoringService);
export const recordError = monitoringService.recordError.bind(monitoringService);
export const recordUserAction = monitoringService.recordUserAction.bind(monitoringService);
export const getMonitorData = monitoringService.getMonitorData.bind(monitoringService);

// 导出类型
export type { MonitorData };
export { MonitorType, PerformanceMetric, UserAction };

export default monitoringService;


