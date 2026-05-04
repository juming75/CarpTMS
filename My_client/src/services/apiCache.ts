import type { AxiosRequestConfig } from 'axios';

// 缓存条目类型定义
interface CacheItem<T> {
  data: T;
  timestamp: number;
  ttl: number;
  lastAccess: number;
  accessCount: number;
}

// API缓存服务类
export class ApiCache {
  private cache: Map<string, CacheItem<unknown>> = new Map();
  private defaultTtl = 60 * 1000; // 默认缓存1分钟
  private maxSize = 100; // 最大缓存条目数
  private stats = {
    hits: 0,
    misses: 0,
    evictions: 0,
  };

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

  // LRU淘汰：移除最久未访问的条目
  private evictIfNeeded(): void {
    if (this.cache.size < this.maxSize) {
      return;
    }

    let oldestKey: string | null = null;
    let oldestTime = Infinity;

    for (const [key, item] of this.cache.entries()) {
      if (item.lastAccess < oldestTime) {
        oldestTime = item.lastAccess;
        oldestKey = key;
      }
    }

    if (oldestKey) {
      this.cache.delete(oldestKey);
      this.stats.evictions++;
    }
  }

  // 清除过期缓存
  private cleanExpired(): void {
    const now = Date.now();
    for (const [key, item] of this.cache.entries()) {
      if (now - item.timestamp > item.ttl) {
        this.cache.delete(key);
      }
    }
  }

  // 设置缓存
  set<T>(config: AxiosRequestConfig, data: T, ttl?: number): void {
    this.cleanExpired();
    this.evictIfNeeded();

    const key = this.generateKey(config);
    const cacheItem: CacheItem<unknown> = {
      data,
      timestamp: Date.now(),
      ttl: ttl || this.defaultTtl,
      lastAccess: Date.now(),
      accessCount: 0,
    };
    this.cache.set(key, cacheItem);
  }

  // 获取缓存
  get<T>(config: AxiosRequestConfig): T | null {
    const key = this.generateKey(config);
    const cacheItem = this.cache.get(key);

    if (!cacheItem) {
      this.stats.misses++;
      return null;
    }

    // 检查缓存是否过期
    if (Date.now() - cacheItem.timestamp > cacheItem.ttl) {
      this.cache.delete(key);
      this.stats.misses++;
      return null;
    }

    // 更新访问信息
    cacheItem.lastAccess = Date.now();
    cacheItem.accessCount++;
    this.stats.hits++;

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

  // 获取缓存统计信息
  getStats(): { hits: number; misses: number; hitRate: number; size: number; evictions: number } {
    const total = this.stats.hits + this.stats.misses;
    return {
      hits: this.stats.hits,
      misses: this.stats.misses,
      hitRate: total > 0 ? (this.stats.hits / total) * 100 : 0,
      size: this.cache.size,
      evictions: this.stats.evictions,
    };
  }

  // 重置统计
  resetStats(): void {
    this.stats.hits = 0;
    this.stats.misses = 0;
    this.stats.evictions = 0;
  }
}

// 创建全局API缓存实例
export const apiCache = new ApiCache();


