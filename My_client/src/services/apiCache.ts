import type { AxiosRequestConfig } from 'axios';

// 缓存条目类型定义
interface CacheItem<T> {
  data: T;
  timestamp: number;
  ttl: number;
}

// API缓存服务类
export class ApiCache {
  private cache: Map<string, CacheItem<unknown>> = new Map();
  private defaultTtl = 60 * 1000; // 默认缓存1分钟

  // 生成缓存键
  private generateKey(config: AxiosRequestConfig): string {
    if (!config) {
      return 'unknown:unknown:{}:{}';
    }
    const url = config.url || '';
    const method = typeof config.method === 'string' ? config.method.toUpperCase() : 'GET';
    const params = config.params ? JSON.stringify(config.params) : '';
    const data = config.data ? JSON.stringify(config.data) : '';

    return `${method}:${url}:${params}:${data}`;
  }

  // 设置缓存
  set<T>(config: AxiosRequestConfig, data: T, ttl?: number): void {
    const key = this.generateKey(config);
    const cacheItem: CacheItem<unknown> = {
      data,
      timestamp: Date.now(),
      ttl: ttl || this.defaultTtl,
    };
    this.cache.set(key, cacheItem);
  }

  // 获取缓存
  get<T>(config: AxiosRequestConfig): T | null {
    const key = this.generateKey(config);
    const cacheItem = this.cache.get(key);

    if (!cacheItem) {
      return null;
    }

    // 检查缓存是否过期
    if (Date.now() - cacheItem.timestamp > cacheItem.ttl) {
      this.cache.delete(key);
      return null;
    }

    return cacheItem.data as T;
  }

  // 清除指定缓存
  delete(config: AxiosRequestConfig): void {
    const key = this.generateKey(config);
    this.cache.delete(key);
  }

  // 清除所有缓存
  clear(): void {
    this.cache.clear();
  }

  // 清除特定URL前缀的缓存
  clearByUrlPrefix(prefix: string): void {
    for (const key of this.cache.keys()) {
      if (key.includes(prefix)) {
        this.cache.delete(key);
      }
    }
  }

  // 获取缓存大小
  getSize(): number {
    return this.cache.size;
  }
}

// 创建全局API缓存实例
export const apiCache = new ApiCache();


