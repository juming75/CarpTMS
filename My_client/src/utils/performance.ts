/**
 * 前端性能优化工具函数
 * 包含防抖、节流、虚拟滚动等性能优化工具
 */

/**
 * 防抖函数 - 在事件停止触发后延迟执行
 * @param fn 要执行的函数
 * @param delay 延迟时间（毫秒）
 * @returns 防抖后的函数
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timer: ReturnType<typeof setTimeout> | null = null;

  return function (this: any, ...args: Parameters<T>) {
    if (timer) {
      clearTimeout(timer);
    }
    timer = setTimeout(() => {
      fn.apply(this, args);
      timer = null;
    }, delay);
  };
}

/**
 * 节流函数 - 限制函数执行频率
 * @param fn 要执行的函数
 * @param limit 时间限制（毫秒）
 * @returns 节流后的函数
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  limit: number
): (...args: Parameters<T>) => void {
  let lastCall = 0;
  let timeout: ReturnType<typeof setTimeout> | null = null;

  return function (this: any, ...args: Parameters<T>) {
    const now = Date.now();

    if (now - lastCall >= limit) {
      lastCall = now;
      fn.apply(this, args);
    } else if (!timeout) {
      timeout = setTimeout(() => {
        lastCall = Date.now();
        fn.apply(this, args);
        timeout = null;
      }, limit - (now - lastCall));
    }
  };
}

/**
 * 创建带取消功能的防抖函数
 */
export function createDebounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): { debouncedFn: (...args: Parameters<T>) => void; cancel: () => void } {
  let timer: ReturnType<typeof setTimeout> | null = null;

  const debouncedFn = function (this: any, ...args: Parameters<T>) {
    if (timer) {
      clearTimeout(timer);
    }
    timer = setTimeout(() => {
      fn.apply(this, args);
      timer = null;
    }, delay);
  };

  const cancel = () => {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  };

  return { debouncedFn, cancel };
}

/**
 * 简易虚拟滚动实现 - 用于大列表渲染优化
 */
export class VirtualScroll<T> {
  private container: HTMLElement;
  private items: T[];
  private itemHeight: number;
  private visibleCount: number = 0;
  private scrollTop: number = 0;
  private renderFn: (item: T, index: number) => HTMLElement;
  private containerElement: HTMLElement;

  constructor(
    container: HTMLElement,
    items: T[],
    itemHeight: number,
    renderFn: (item: T, index: number) => HTMLElement
  ) {
    this.container = container;
    this.items = items;
    this.itemHeight = itemHeight;
    this.renderFn = renderFn;
    
    // 创建内部容器
    this.containerElement = document.createElement('div');
    this.containerElement.style.position = 'relative';
    this.containerElement.style.height = `${items.length * itemHeight}px`;
    this.container.appendChild(this.containerElement);

    // 绑定滚动事件
    this.container.addEventListener('scroll', this.handleScroll);
    this.update();
  }

  private handleScroll = () => {
    this.scrollTop = this.container.scrollTop;
    this.update();
  };

  private update = () => {
    const startIndex = Math.floor(this.scrollTop / this.itemHeight);
    const endIndex = Math.min(
      startIndex + this.visibleCount + 2,
      this.items.length
    );

    // 清空现有内容
    this.containerElement.innerHTML = '';

    // 渲染可见项
    for (let i = startIndex; i < endIndex; i++) {
      const itemEl = this.renderFn(this.items[i], i);
      itemEl.style.position = 'absolute';
      itemEl.style.top = `${i * this.itemHeight}px`;
      itemEl.style.left = '0';
      itemEl.style.right = '0';
      this.containerElement.appendChild(itemEl);
    }
  };

  // 更新可见数量
  updateVisibleCount(height: number) {
    this.visibleCount = Math.ceil(height / this.itemHeight);
    this.update();
  }

  // 更新数据
  setItems(items: T[]) {
    this.items = items;
    this.containerElement.style.height = `${items.length * this.itemHeight}px`;
    this.update();
  }

  // 销毁
  destroy() {
    this.container.removeEventListener('scroll', this.handleScroll);
    this.containerElement.remove();
  }
}

/**
 * 内存监控工具
 * 用于检测和报告潜在的内存泄漏
 */
export class MemoryMonitor {
  private static instance: MemoryMonitor;
  private snapshots: { timestamp: number; usedJSHeapSize: number }[] = [];
  private intervalId: ReturnType<typeof setInterval> | null = null;
  private readonly maxSnapshots = 60; // 保留最近60个快照

  static getInstance(): MemoryMonitor {
    if (!MemoryMonitor.instance) {
      MemoryMonitor.instance = new MemoryMonitor();
    }
    return MemoryMonitor.instance;
  }

  /**
   * 获取当前内存使用情况
   */
  getMemoryUsage(): { used: number; total: number; limit: number } | null {
    if ('memory' in performance) {
      const memory = (performance as any).memory;
      return {
        used: Math.round(memory.usedJSHeapSize / 1048576), // MB
        total: Math.round(memory.totalJSHeapSize / 1048576),
        limit: Math.round(memory.jsHeapSizeLimit / 1048576),
      };
    }
    return null;
  }

  /**
   * 拍摄内存快照
   */
  takeSnapshot(): void {
    const memory = this.getMemoryUsage();
    if (memory) {
      this.snapshots.push({
        timestamp: Date.now(),
        usedJSHeapSize: memory.used,
      });

      // 保持最多 maxSnapshots 个快照
      if (this.snapshots.length > this.maxSnapshots) {
        this.snapshots.shift();
      }
    }
  }

  /**
   * 分析内存趋势
   */
  analyzeTrend(): { trend: 'stable' | 'increasing' | 'decreasing'; avgGrowth: number } | null {
    if (this.snapshots.length < 5) return null;

    const recent = this.snapshots.slice(-10);
    let totalGrowth = 0;

    for (let i = 1; i < recent.length; i++) {
      totalGrowth += recent[i].usedJSHeapSize - recent[i - 1].usedJSHeapSize;
    }

    const avgGrowth = totalGrowth / (recent.length - 1);

    if (avgGrowth > 0.5) {
      return { trend: 'increasing', avgGrowth };
    } else if (avgGrowth < -0.5) {
      return { trend: 'decreasing', avgGrowth };
    }
    return { trend: 'stable', avgGrowth };
  }

  /**
   * 开始定期监控
   */
  startMonitoring(intervalMs: number = 5000): void {
    if (this.intervalId) return;

    this.intervalId = setInterval(() => {
      this.takeSnapshot();

      // 检测内存泄漏
      const trend = this.analyzeTrend();
      if (trend && trend.trend === 'increasing' && trend.avgGrowth > 5) {
        console.warn(
          `[MemoryMonitor] 潜在内存泄漏检测: 内存平均增长 ${trend.avgGrowth.toFixed(2)}MB/采样`
        );
      }
    }, intervalMs);
  }

  /**
   * 停止监控
   */
  stopMonitoring(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  /**
   * 获取监控报告
   */
  getReport(): object {
    const memory = this.getMemoryUsage();
    const trend = this.analyzeTrend();

    return {
      current: memory,
      trend,
      snapshotsCount: this.snapshots.length,
      firstSnapshot: this.snapshots[0] || null,
      lastSnapshot: this.snapshots[this.snapshots.length - 1] || null,
    };
  }
}

/**
 * 图片懒加载指令
 */
export const lazyLoadDirective = {
  mounted(el: HTMLElement, binding: any) {
    const lazyLoad = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const img = entry.target as HTMLImageElement;
            const src = binding.value;
            if (src) {
              img.src = src;
            }
            lazyLoad.unobserve(img);
          }
        });
      },
      {
        rootMargin: '50px',
        threshold: 0.1,
      }
    );

    lazyLoad.observe(el);

    // 将 observer 存储在元素上，以便后续清理
    (el as any)._lazyLoadObserver = lazyLoad;
  },

  unmounted(el: HTMLElement) {
    const observer = (el as any)._lazyLoadObserver;
    if (observer) {
      observer.disconnect();
    }
  },
};

/**
 * 批量更新工具 - 减少重渲染
 */
export class BatchUpdater<T> {
  private items: T[] = [];
  private updateFn: (items: T[]) => void;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private readonly delay: number;

  constructor(updateFn: (items: T[]) => void, delay: number = 16) {
    this.updateFn = updateFn;
    this.delay = delay; // 默认 16ms，约一帧
  }

  add(item: T): void {
    this.items.push(item);
    this.scheduleUpdate();
  }

  addMany(newItems: T[]): void {
    this.items.push(...newItems);
    this.scheduleUpdate();
  }

  setItems(items: T[]): void {
    this.items = items;
    this.scheduleUpdate();
  }

  private scheduleUpdate(): void {
    if (this.timer) return;

    this.timer = setTimeout(() => {
      this.flush();
    }, this.delay);
  }

  flush(): void {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }

    if (this.items.length > 0) {
      const itemsToUpdate = [...this.items];
      this.items = [];
      this.updateFn(itemsToUpdate);
    }
  }

  clear(): void {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
    this.items = [];
  }
}
