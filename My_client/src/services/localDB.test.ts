// @ts-nocheck
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { isElectron, localVehicleService, localWeighingService, localSyncService, getDbInfo, getAppVersion, getElectronVersion } from './localDB';

// 类型定义
interface MockElectronAPI {
  getLocalVehicles?: () => Promise<unknown>;
  saveLocalVehicle?: (vehicle: unknown) => Promise<unknown>;
  deleteLocalVehicle?: (vehicleId: number) => Promise<unknown>;
  getLocalWeighingData?: (limit?: number) => Promise<unknown>;
  getUnsyncedData?: () => Promise<unknown>;
  markAsSynced?: (data: unknown) => Promise<unknown>;
  logSync?: (log: unknown) => Promise<unknown>;
  getDbInfo?: () => Promise<unknown>;
  getAppVersion?: () => Promise<string>;
  getElectronVersion?: () => Promise<string>;
  on?: (channel: string, callback: (...args: unknown[]) => void) => void;
  removeListener?: (channel: string, callback: (...args: unknown[]) => void) => void;
}

// 类型断言函数
function getMockWindow() {
  return window as unknown as { electronAPI?: MockElectronAPI };
}

describe('localDB services', () => {
  beforeEach(() => {
    // 在每个测试前删除electronAPI，确保测试环境干净
    const mockWindow = getMockWindow();
    delete mockWindow.electronAPI;
  });

  afterEach(() => {
    // 清理所有模拟
    vi.restoreAllMocks();
  });

  describe('isElectron', () => {
    it('should return false when not in Electron environment', () => {
      // 确保window.electronAPI未定义
      const mockWindow = getMockWindow();
      delete mockWindow.electronAPI;
      expect(isElectron()).toBe(false);
    });

    it('should return true when in Electron environment', () => {
      // 模拟Electron环境
      const mockWindow = getMockWindow();
      mockWindow.electronAPI = {
        getLocalVehicles: vi.fn(),
      };
      expect(isElectron()).toBe(true);
    });
  });

  describe('localVehicleService', () => {
    it('should return empty array when not in Electron environment', async () => {
      const mockWindow = getMockWindow();
      delete mockWindow.electronAPI;
      const result = await localVehicleService.getAll();
      expect(result).toEqual([]);
    });

    it('should call electronAPI.getLocalVehicles when in Electron environment', async () => {
      // 模拟Electron环境
      const mockVehicles = [{ vehicle_id: 1, vehicle_name: 'Test Vehicle' }];
      const mockWindow = getMockWindow();
      mockWindow.electronAPI = {
        getLocalVehicles: vi.fn().mockResolvedValue(mockVehicles),
      };

      const result = await localVehicleService.getAll();
      expect(mockWindow.electronAPI.getLocalVehicles).toHaveBeenCalled();
      expect(result).toEqual(mockVehicles);
    });

    it('should call electronAPI.saveLocalVehicle when in Electron environment', async () => {
      const mockVehicle = { id: 1, plateNo: 'Test Vehicle', type: 'truck', model: 'Test', status: 'idle' as const, vin: '123', engineNo: '456', purchaseDate: '2026-01-01', lastMaintenanceDate: '2026-01-01', nextMaintenanceDate: '2026-06-01', mileage: 0, fuelType: 'gasoline', capacity: 1000 };
      const mockResult = { success: true };

      const mockWindow = getMockWindow();
      mockWindow.electronAPI = {
        saveLocalVehicle: vi.fn().mockResolvedValue(mockResult),
      };

      const result = await localVehicleService.save(mockVehicle);
      expect(mockWindow.electronAPI.saveLocalVehicle).toHaveBeenCalledWith(mockVehicle);
      expect(result).toEqual(mockResult);
    });

    it('should call electronAPI.deleteLocalVehicle when in Electron environment', async () => {
      const mockResult = { success: true };

      const mockWindow = getMockWindow();
      mockWindow.electronAPI = {
        deleteLocalVehicle: vi.fn().mockResolvedValue(mockResult),
      };

      const result = await localVehicleService.delete(1);
      expect(mockWindow.electronAPI.deleteLocalVehicle).toHaveBeenCalledWith(1);
      expect(result).toEqual(mockResult);
    });
  });

  describe('localWeighingService', () => {
    it('should return empty array when not in Electron environment', async () => {
      const mockWindow = getMockWindow();
      delete mockWindow.electronAPI;
      const result = await localWeighingService.getAll();
      expect(result).toEqual([]);
    });

    it('should call electronAPI.getLocalWeighingData when in Electron environment', async () => {
      const mockWeighingData = [{ id: 1, vehicle_id: 1, weighing_time: '2023-01-01' }];

      const mockWindow = getMockWindow();
      mockWindow.electronAPI = {
        getLocalWeighingData: vi.fn().mockResolvedValue(mockWeighingData),
      };

      const result = await localWeighingService.getAll(50);
      expect(mockWindow.electronAPI.getLocalWeighingData).toHaveBeenCalledWith(50);
      expect(result).toEqual(mockWeighingData);
    });
  });

  describe('localSyncService', () => {
    it('should return default values when not in Electron environment', async () => {
      const mockWindow = getMockWindow();
      delete mockWindow.electronAPI;
      const result = await localSyncService.getUnsynced();
      expect(result).toEqual({ vehicles: [], weighingData: [] });
    });
  });

  describe('getDbInfo', () => {
    it('should return default values when not in Electron environment', async () => {
      const mockWindow = getMockWindow();
      delete mockWindow.electronAPI;
      const result = await getDbInfo();
      expect(result).toEqual({ path: 'unknown', vehicleCount: 0, weighingCount: 0 });
    });
  });

  describe('version functions', () => {
    it('should return default app version when not in Electron environment', async () => {
      const mockWindow = getMockWindow();
      delete mockWindow.electronAPI;
      const result = await getAppVersion();
      expect(result).toBe('1.0.0');
    });

    it('should return default electron version when not in Electron environment', async () => {
      const mockWindow = getMockWindow();
      delete mockWindow.electronAPI;
      const result = await getElectronVersion();
      expect(result).toBe('unknown');
    });
  });
});


