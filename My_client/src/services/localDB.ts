import type { Vehicle, WeighingData, LocalVehicle, LocalWeighingData } from '@/types';

// Electron API 类型声明
declare global {
  interface Window {
    electronAPI: {
      // 车辆管理
      getLocalVehicles: () => Promise<LocalVehicle[]>;
      saveLocalVehicle: (vehicle: Vehicle) => Promise<unknown>;
      deleteLocalVehicle: (vehicleId: number) => Promise<unknown>;

      // 称重数据
      getLocalWeighingData: (limit?: number) => Promise<LocalWeighingData[]>;
      saveLocalWeighingData: (data: WeighingData) => Promise<unknown>;

      // 数据同步
      getUnsyncedData: () => Promise<{
        vehicles: LocalVehicle[];
        weighingData: LocalWeighingData[];
      }>;
      markAsSynced: (data: { type: string; ids: number[]; serverIds: number[] }) => Promise<unknown>;
      logSync: (log: { syncType: string; recordCount: number; status: number; errorMessage?: string }) => Promise<unknown>;

      // 数据库信息
      getDbInfo: () => Promise<{ path: string; vehicleCount: number; weighingCount: number }>;

      // 系统信息
      getAppVersion: () => Promise<string>;
      getElectronVersion: () => Promise<string>;

      // 事件监听
      on: (channel: string, callback: (...args: unknown[]) => void) => void;
      removeListener: (channel: string, callback: (...args: unknown[]) => void) => void;
    };
  }
}

// 检查是否在 Electron 环境
export const isElectron = (): boolean => {
  return typeof window.electronAPI !== 'undefined';
};

// 通用 Electron API 调用包装函数
export const callElectronAPI = async <T>(
  method: string,
  args: unknown[] = [],
  defaultReturnValue: T,
  showWarning: boolean = true
): Promise<T> => {
  if (!isElectron()) {
    if (showWarning) {
      console.warn('Not in Electron environment');
    }
    return defaultReturnValue;
  }

  try {
    // @ts-ignore - 动态调用 Electron API 方法
    const result = await (window.electronAPI[method] as Function)(...args);
    return result as T;
  } catch (error) {
    console.error(`Failed to call Electron API ${method}:`, error);
    return defaultReturnValue;
  }
};

// ====== 车辆管理 ======
export const localVehicleService = {
  // 获取本地车辆
  getAll: async (): Promise<LocalVehicle[]> => {
    return await callElectronAPI<LocalVehicle[]>('getLocalVehicles', [], []);
  },

  // 保存本地车辆
  save: async (vehicle: Vehicle): Promise<unknown> => {
    return await callElectronAPI<unknown>('saveLocalVehicle', [vehicle], null);
  },

  // 删除本地车辆
  delete: async (vehicleId: number): Promise<unknown> => {
    return await callElectronAPI<unknown>('deleteLocalVehicle', [vehicleId], null);
  },
};

// ====== 称重数据 ======
export const localWeighingService = {
  // 获取本地称重数据
  getAll: async (limit = 100): Promise<LocalWeighingData[]> => {
    return await callElectronAPI<LocalWeighingData[]>('getLocalWeighingData', [limit], []);
  },

  // 保存本地称重数据
  save: async (data: WeighingData): Promise<unknown> => {
    return await callElectronAPI<unknown>('saveLocalWeighingData', [data], null);
  },
};

// ====== 数据同步 ======
export const localSyncService = {
  // 获取未同步数据
  getUnsynced: async () => {
    return await callElectronAPI<{ vehicles: LocalVehicle[]; weighingData: LocalWeighingData[] }>('getUnsyncedData', [], { vehicles: [], weighingData: [] }, false);
  },

  // 标记为已同步
  markSynced: async (type: string, ids: number[], serverIds: number[]) => {
    return await callElectronAPI<unknown>('markAsSynced', [{ type, ids, serverIds }], null, false);
  },

  // 记录同步日志
  logSync: async (syncType: string, recordCount: number, status: number, errorMessage?: string) => {
    return await callElectronAPI<unknown>('logSync', [{ syncType, recordCount, status, errorMessage }], null, false);
  },
};

// ====== 数据库信息 ======
export const getDbInfo = async () => {
  return await callElectronAPI<{ path: string; vehicleCount: number; weighingCount: number }>('getDbInfo', [], { path: 'unknown', vehicleCount: 0, weighingCount: 0 }, false);
};

// ====== 系统信息 ======
export const getAppVersion = async () => {
  return await callElectronAPI<string>('getAppVersion', [], '1.0.0', false);
};

export const getElectronVersion = async () => {
  return await callElectronAPI<string>('getElectronVersion', [], 'unknown', false);
};


