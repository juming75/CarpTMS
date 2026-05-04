export interface PerformanceMetric {
  id: string;
  name: string;
  value: number;
  unit: string;
  timestamp: number;
  type: 'memory' | 'cpu' | 'network' | 'render' | 'api';
}

export interface APIMetric {
  url: string;
  method: string;
  duration: number;
  status: number;
  timestamp: number;
  success: boolean;
}

export interface MemoryInfo {
  usedJSHeapSize: number;
  totalJSHeapSize: number;
  jsHeapSizeLimit: number;
}

class PerformanceMonitor {
  private metrics: PerformanceMetric[] = [];
  private apiMetrics: APIMetric[] = [];
  private maxMetrics = 100;
  private maxAPIMetrics = 50;
  private intervalId: number | null = null;
  private listeners: Set<() => void> = new Set();

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    this.listeners.forEach((listener) => listener());
  }

  start(): void {
    if (this.intervalId) return;
    this.intervalId = window.setInterval(() => {
      this.collectMetrics();
    }, 2000);
    this.collectMetrics();
  }

  stop(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  private collectMetrics(): void {
    this.collectMemoryMetrics();
    this.collectRenderMetrics();
    this.notify();
  }

  private collectMemoryMetrics(): void {
    if (window.performance && (window.performance as unknown as { memory: MemoryInfo }).memory) {
      const memory = (window.performance as unknown as { memory: MemoryInfo }).memory;
      const usedMB = (memory.usedJSHeapSize / (1024 * 1024)).toFixed(2);
      const totalMB = (memory.totalJSHeapSize / (1024 * 1024)).toFixed(2);

      this.addMetric({
        id: `memory-used-${Date.now()}`,
        name: '已用内存',
        value: parseFloat(usedMB),
        unit: 'MB',
        timestamp: Date.now(),
        type: 'memory',
      });

      this.addMetric({
        id: `memory-total-${Date.now()}`,
        name: '总内存',
        value: parseFloat(totalMB),
        unit: 'MB',
        timestamp: Date.now(),
        type: 'memory',
      });
    }
  }

  private collectRenderMetrics(): void {
    const timing = window.performance.timing;
    const navigationStart = timing.navigationStart;

    const paintTiming = (window as unknown as { performance: { getEntriesByType: (type: string) => Array<{ startTime: number; duration: number }> } }).performance.getEntriesByType('paint');
    
    paintTiming.forEach((entry) => {
      this.addMetric({
        id: `render-${entry.startTime}-${Date.now()}`,
        name: '首屏渲染',
        value: entry.startTime,
        unit: 'ms',
        timestamp: Date.now(),
        type: 'render',
      });
    });

    if (timing.loadEventEnd - navigationStart > 0) {
      this.addMetric({
        id: `load-time-${Date.now()}`,
        name: '页面加载时间',
        value: timing.loadEventEnd - navigationStart,
        unit: 'ms',
        timestamp: Date.now(),
        type: 'render',
      });
    }
  }

  trackAPI(url: string, method: string, duration: number, status: number): void {
    const metric: APIMetric = {
      url,
      method,
      duration,
      status,
      timestamp: Date.now(),
      success: status >= 200 && status < 300,
    };

    this.apiMetrics.unshift(metric);
    if (this.apiMetrics.length > this.maxAPIMetrics) {
      this.apiMetrics.pop();
    }
    this.notify();
  }

  private addMetric(metric: PerformanceMetric): void {
    this.metrics.unshift(metric);
    if (this.metrics.length > this.maxMetrics) {
      this.metrics.pop();
    }
  }

  getMetrics(): PerformanceMetric[] {
    return [...this.metrics];
  }

  getAPIMetrics(): APIMetric[] {
    return [...this.apiMetrics];
  }

  getMemoryMetrics(): PerformanceMetric[] {
    return this.metrics.filter((m) => m.type === 'memory');
  }

  getAPISummary(): { avgDuration: number; successRate: number; count: number } {
    if (this.apiMetrics.length === 0) {
      return { avgDuration: 0, successRate: 0, count: 0 };
    }

    const avgDuration =
      this.apiMetrics.reduce((sum, m) => sum + m.duration, 0) / this.apiMetrics.length;
    const successRate =
      (this.apiMetrics.filter((m) => m.success).length / this.apiMetrics.length) * 100;

    return {
      avgDuration: parseFloat(avgDuration.toFixed(2)),
      successRate: parseFloat(successRate.toFixed(2)),
      count: this.apiMetrics.length,
    };
  }

  getCurrentMemoryUsage(): number {
    if (window.performance && (window.performance as unknown as { memory: MemoryInfo }).memory) {
      const memory = (window.performance as unknown as { memory: MemoryInfo }).memory;
      return parseFloat(((memory.usedJSHeapSize / memory.jsHeapSizeLimit) * 100).toFixed(2));
    }
    return 0;
  }

  reset(): void {
    this.metrics = [];
    this.apiMetrics = [];
    this.notify();
  }
}

export const performanceMonitor = new PerformanceMonitor();

export function initPerformanceMonitoring(): void {
  performanceMonitor.start();
}

export function trackAPICall(url: string, method: string, duration: number, status: number): void {
  performanceMonitor.trackAPI(url, method, duration, status);
}