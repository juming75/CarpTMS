// 性能监控和优化工具

// 获取性能时间戳的兼容方法
export function getPerformanceNow(): number {
  if (typeof window !== 'undefined' && window.performance && window.performance.now) {
    return window.performance.now();
  }
  return Date.now();
}

// 组件渲染性能监控
export const measureComponentRender = (componentName: string, fn: () => void) => {
  const start = getPerformanceNow();
  fn();
  const end = getPerformanceNow();
  console.log(`${componentName} rendered in ${end - start}ms`);
};

// 数据处理性能监控
export const measureDataProcessing = (operationName: string, fn: () => void) => {
  const start = getPerformanceNow();
  fn();
  const end = getPerformanceNow();
  console.log(`${operationName} processed in ${end - start}ms`);
};

// 防抖函数
export function debounce<T extends (...args: unknown[]) => unknown>(func: T, wait: number): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}

// 节流函数
export function throttle<T extends (...args: unknown[]) => unknown>(func: T, limit: number): (...args: Parameters<T>) => void {
  let inThrottle = false;

  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

// 批量处理数据
export function batchProcess<T>(data: T[], batchSize: number, processFn: (batch: T[]) => void) {
  const batches = [];
  for (let i = 0; i < data.length; i += batchSize) {
    batches.push(data.slice(i, i + batchSize));
  }

  batches.forEach((batch, index) => {
    setTimeout(() => processFn(batch), index * 100); // 100ms间隔处理每个批次
  });
}

// 虚拟滚动配置
export const virtualScrollConfig = {
  itemHeight: 40, // 每个项目的高度
  buffer: 10, // 缓冲区大小
  threshold: 200, // 滚动阈值
};

// 图片懒加载配置
export const lazyLoadConfig = {
  threshold: 0.1, // 元素可见度阈值
  rootMargin: '0px 0px 200px 0px', // 根元素外边距
};

// 性能监控配置
export const performanceConfig = {
  enableComponentRenderMonitoring: true, // 是否启用组件渲染监控
  enableDataProcessingMonitoring: true, // 是否启用数据处理监控
  logLevel: 'info', // 日志级别: debug, info, warn, error
};

// 记录性能指标
export const recordPerformanceMetric = (name: string, value: number, metadata?: unknown) => {
  if (performanceConfig.logLevel === 'debug' || performanceConfig.logLevel === 'info') {
    console.log(`Performance metric: ${name} = ${value}ms`, metadata);
  }
};


