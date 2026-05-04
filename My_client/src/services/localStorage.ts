import type { Vehicle } from '@/types';

// 本地存储键名常量
const STORAGE_KEYS = {
  VEHICLES: 'CarpTMS_vehicles',
  OFFLINE_OPERATIONS: 'CarpTMS_offline_operations',
  LAST_SYNC_TIME: 'CarpTMS_last_sync_time',
  APP_SETTINGS: 'CarpTMS_settings',
  API_CACHE: 'CarpTMS_api_cache',
};

// 操作类型定义
export enum OfflineOperationType {
  CREATE = 'create',
  UPDATE = 'update',
  DELETE = 'delete',
}

// 缓存项接口
interface CacheItem {
  data: unknown;
  timestamp: number;
  expiration: number;
}

// 离线操作记录
export interface OfflineOperation {
  id: string;
  type: OfflineOperationType;
  entity: string;
  entityId?: number | string;
  data: unknown;
  timestamp: number;
  synced: boolean;
}

// 本地存储服务类
export class LocalStorageService {
  // 保存车辆数据到本地存储
  saveVehicles(vehicles: Vehicle[]): void {
    try {
      const data = JSON.stringify(vehicles);
      localStorage.setItem(STORAGE_KEYS.VEHICLES, data);
      console.log('[本地存储] 车辆数据保存成功，共', vehicles.length, '条记录');
    } catch (error) {
      console.error('[本地存储] 保存车辆数据失败:', error);
    }
  }

  // 从本地存储获取车辆数据
  getVehicles(): Vehicle[] {
    try {
      const data = localStorage.getItem(STORAGE_KEYS.VEHICLES);
      if (data) {
        const vehicles = JSON.parse(data) as Vehicle[];
        console.log('[本地存储] 获取车辆数据成功，共', vehicles.length, '条记录');
        return vehicles;
      }
      return [];
    } catch (error) {
      console.error('[本地存储] 获取车辆数据失败:', error);
      return [];
    }
  }

  // 保存单个车辆到本地存储
  saveVehicle(vehicle: Vehicle): void {
    try {
      const vehicles = this.getVehicles();
      const existingIndex = vehicles.findIndex((v) => v.vehicle_id === vehicle.vehicle_id);

      if (existingIndex >= 0) {
        // 更新现有车辆
        vehicles[existingIndex] = vehicle;
      } else {
        // 添加新车辆
        vehicles.push(vehicle);
      }

      this.saveVehicles(vehicles);
      console.log('[本地存储] 单个车辆保存成功:', vehicle.vehicle_id);
    } catch (error) {
      console.error('[本地存储] 保存单个车辆失败:', error);
    }
  }

  // 从本地存储删除车辆
  deleteVehicle(vehicleId: number): void {
    try {
      const vehicles = this.getVehicles();
      const filteredVehicles = vehicles.filter((v) => v.vehicle_id !== vehicleId);
      this.saveVehicles(filteredVehicles);
      console.log('[本地存储] 车辆删除成功:', vehicleId);
    } catch (error) {
      console.error('[本地存储] 删除车辆失败:', error);
    }
  }

  // 记录离线操作
  recordOfflineOperation(operation: Omit<OfflineOperation, 'id' | 'timestamp' | 'synced'>): void {
    try {
      const operations = this.getOfflineOperations();
      const newOperation: OfflineOperation = {
        ...operation,
        id: `op_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        timestamp: Date.now(),
        synced: false,
      };
      operations.push(newOperation);
      this.saveOfflineOperations(operations);
      console.log('[本地存储] 离线操作记录成功:', newOperation.type, newOperation.entity);
    } catch (error) {
      console.error('[本地存储] 记录离线操作失败:', error);
    }
  }

  // 保存离线操作列表
  saveOfflineOperations(operations: OfflineOperation[]): void {
    try {
      const data = JSON.stringify(operations);
      localStorage.setItem(STORAGE_KEYS.OFFLINE_OPERATIONS, data);
      console.log('[本地存储] 离线操作保存成功，共', operations.length, '条记录');
    } catch (error) {
      console.error('[本地存储] 保存离线操作失败:', error);
    }
  }

  // 获取离线操作列表
  getOfflineOperations(): OfflineOperation[] {
    try {
      const data = localStorage.getItem(STORAGE_KEYS.OFFLINE_OPERATIONS);
      if (data) {
        const operations = JSON.parse(data) as OfflineOperation[];
        console.log('[本地存储] 获取离线操作成功，共', operations.length, '条记录');
        return operations;
      }
      return [];
    } catch (error) {
      console.error('[本地存储] 获取离线操作失败:', error);
      return [];
    }
  }

  // 标记离线操作为已同步
  markOperationAsSynced(operationId: string): void {
    try {
      const operations = this.getOfflineOperations();
      const updatedOperations = operations.map((op) => (op.id === operationId ? { ...op, synced: true } : op));
      this.saveOfflineOperations(updatedOperations);
      console.log('[本地存储] 操作标记为已同步:', operationId);
    } catch (error) {
      console.error('[本地存储] 标记操作同步失败:', error);
    }
  }

  // 清除已同步的离线操作
  clearSyncedOperations(): void {
    try {
      const operations = this.getOfflineOperations();
      const unsyncedOperations = operations.filter((op) => !op.synced);
      this.saveOfflineOperations(unsyncedOperations);
      console.log('[本地存储] 已清除已同步的操作，剩余', unsyncedOperations.length, '条未同步');
    } catch (error) {
      console.error('[本地存储] 清除已同步操作失败:', error);
    }
  }

  // 保存最后同步时间
  saveLastSyncTime(time: number): void {
    try {
      localStorage.setItem(STORAGE_KEYS.LAST_SYNC_TIME, time.toString());
      console.log('[本地存储] 最后同步时间保存成功:', new Date(time).toISOString());
    } catch (error) {
      console.error('[本地存储] 保存最后同步时间失败:', error);
    }
  }

  // 获取最后同步时间
  getLastSyncTime(): number {
    try {
      const timeStr = localStorage.getItem(STORAGE_KEYS.LAST_SYNC_TIME);
      if (timeStr) {
        const time = parseInt(timeStr);
        console.log('[本地存储] 获取最后同步时间成功:', new Date(time).toISOString());
        return time;
      }
      return 0;
    } catch (error) {
      console.error('[本地存储] 获取最后同步时间失败:', error);
      return 0;
    }
  }

  // 保存应用设置
  saveSettings(settings: unknown): void {
    try {
      const data = JSON.stringify(settings);
      localStorage.setItem(STORAGE_KEYS.APP_SETTINGS, data);
      console.log('[本地存储] 应用设置保存成功');
    } catch (error) {
      console.error('[本地存储] 保存应用设置失败:', error);
    }
  }

  // 获取应用设置
  getSettings(): unknown {
    try {
      const data = localStorage.getItem(STORAGE_KEYS.APP_SETTINGS);
      if (data) {
        const settings = JSON.parse(data);
        console.log('[本地存储] 获取应用设置成功');
        return settings;
      }
      return {};
    } catch (error) {
      console.error('[本地存储] 获取应用设置失败:', error);
      return {};
    }
  }

  // 缓存API响应
  cacheApiResponse(key: string, data: unknown, expirationMs: number = 3600000): void {
    try {
      const cacheItem = {
        data,
        timestamp: Date.now(),
        expiration: expirationMs,
      };
      const cache = this.getApiCache();
      cache[key] = cacheItem;
      this.saveApiCache(cache);
      console.log('[本地存储] API响应缓存成功:', key);
    } catch (error) {
      console.error('[本地存储] 缓存API响应失败:', error);
    }
  }

  // 获取缓存的API响应
  getCachedApiResponse(key: string): unknown | null {
    try {
      const cache = this.getApiCache();
      const cacheItem = cache[key] as CacheItem;

      if (!cacheItem) {
        return null;
      }

      // 检查缓存是否过期
      const now = Date.now();
      if (now - cacheItem.timestamp > cacheItem.expiration) {
        // 缓存过期，删除
        delete cache[key];
        this.saveApiCache(cache);
        console.log('[本地存储] API缓存已过期:', key);
        return null;
      }

      console.log('[本地存储] 获取API缓存成功:', key);
      return cacheItem.data;
    } catch (error) {
      console.error('[本地存储] 获取API缓存失败:', error);
      return null;
    }
  }

  // 保存API缓存
  private saveApiCache(cache: Record<string, unknown>): void {
    try {
      const data = JSON.stringify(cache);
      localStorage.setItem(STORAGE_KEYS.API_CACHE, data);
    } catch (error) {
      console.error('[本地存储] 保存API缓存失败:', error);
    }
  }

  // 获取API缓存
  private getApiCache(): Record<string, unknown> {
    try {
      const data = localStorage.getItem(STORAGE_KEYS.API_CACHE);
      if (data) {
        return JSON.parse(data);
      }
      return {};
    } catch (error) {
      console.error('[本地存储] 获取API缓存失败:', error);
      return {};
    }
  }

  // 清除所有本地存储数据
  clearAll(): void {
    try {
      Object.values(STORAGE_KEYS).forEach((key) => {
        localStorage.removeItem(key);
      });
      console.log('[本地存储] 所有数据已清除');
    } catch (error) {
      console.error('[本地存储] 清除所有数据失败:', error);
    }
  }

  // 获取存储使用情况
  getStorageUsage(): {
    totalSize: number;
    usedSize: number;
    usedPercentage: number;
    details: Record<string, number>;
  } {
    try {
      const details: Record<string, number> = {};
      let usedSize = 0;

      // 计算每个键的大小
      Object.entries(STORAGE_KEYS).forEach(([name, key]) => {
        const value = localStorage.getItem(key);
        if (value) {
          // 使用字符串长度作为大小估算，避免使用 Blob
          const size = value.length;
          details[name] = size;
          usedSize += size;
        } else {
          details[name] = 0;
        }
      });

      // 估算本地存储总大小（通常为5MB）
      const totalSize = 5 * 1024 * 1024;
      const usedPercentage = (usedSize / totalSize) * 100;

      console.log('[本地存储] 存储使用情况:', {
        usedSize,
        totalSize,
        usedPercentage,
      });

      return {
        totalSize,
        usedSize,
        usedPercentage,
        details,
      };
    } catch (error) {
      console.error('[本地存储] 获取存储使用情况失败:', error);
      return {
        totalSize: 5 * 1024 * 1024,
        usedSize: 0,
        usedPercentage: 0,
        details: {},
      };
    }
  }
}

// 创建本地存储服务实例
export const localStorageService = new LocalStorageService();


