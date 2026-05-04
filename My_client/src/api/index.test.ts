import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  checkBackendHealth,
  vehicleApi,
  orderApi,
  alarmApi,
  authApi,
  driverApi,
  settingsApi,
  departmentApi,
  organizationApi,
  vehicleTeamApi,
  statisticsApi,
  nodeApi,
  dispatchApi,
} from './index';

// Mock the API client
vi.mock('@/services/apiClient', () => ({
  api: {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
  },
}));

import { api } from '@/services/apiClient';

describe('API 模块', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('checkBackendHealth', () => {
    it('should return true when health check passes', async () => {
      (api.get as any).mockResolvedValueOnce({ status: 200 });

      const result = await checkBackendHealth();

      expect(result).toBe(true);
      expect(api.get).toHaveBeenCalledWith('/api/health', { timeout: 3000 });
    });

    it('should return false when health check fails', async () => {
      (api.get as any).mockRejectedValueOnce(new Error('连接失败'));

      const result = await checkBackendHealth();

      expect(result).toBe(false);
    });
  });

  describe('vehicleApi', () => {
    it('should get all vehicles', async () => {
      const mockResponse = { items: [], total: 0, page: 1, page_size: 10 };
      (api.get as any).mockResolvedValueOnce(mockResponse);

      const result = await vehicleApi.getAll();

      expect(result).toEqual(mockResponse);
      expect(api.get).toHaveBeenCalledWith('/api/vehicles', expect.anything());
    });

    it('should get vehicle by id', async () => {
      const mockVehicle = { vehicle_id: 1, plate_number: '京A12345' };
      (api.get as any).mockResolvedValueOnce(mockVehicle);

      const result = await vehicleApi.getById(1);

      expect(result).toEqual(mockVehicle);
      expect(api.get).toHaveBeenCalledWith('/api/vehicles/1');
    });

    it('should create a vehicle', async () => {
      const mockNewVehicle = { plate_number: '京B12345' };
      const mockResponse = { vehicle_id: 2, plate_number: '京B12345' };
      (api.post as any).mockResolvedValueOnce(mockResponse);

      const result = await vehicleApi.create(mockNewVehicle);

      expect(result).toEqual(mockResponse);
      expect(api.post).toHaveBeenCalledWith('/api/vehicles', mockNewVehicle);
    });

    it('should update a vehicle', async () => {
      const updateData = { plate_number: '京C12345' };
      const mockResponse = { vehicle_id: 1, plate_number: '京C12345' };
      (api.put as any).mockResolvedValueOnce(mockResponse);

      const result = await vehicleApi.update(1, updateData);

      expect(result).toEqual(mockResponse);
      expect(api.put).toHaveBeenCalledWith('/api/vehicles/1', updateData);
    });

    it('should delete a vehicle', async () => {
      (api.delete as any).mockResolvedValueOnce(undefined);

      await vehicleApi.delete(1);

      expect(api.delete).toHaveBeenCalledWith('/api/vehicles/1');
    });

    it('should throw error when get vehicles fails', async () => {
      const testError = new Error('网络错误');
      (api.get as any).mockRejectedValueOnce(testError);

      await expect(vehicleApi.getAll()).rejects.toThrow('网络错误');
    });
  });

  describe('orderApi', () => {
    it('should get all orders', async () => {
      const mockResponse = { data: { items: [], total: 0, page: 1, page_size: 10 } };
      (api.get as any).mockResolvedValueOnce(mockResponse);

      const result = await orderApi.getAll();

      expect(result).toEqual(mockResponse);
    });

    it('should get order by id', async () => {
      const mockResponse = { data: { order_id: 1, order_number: 'ORD001' } };
      (api.get as any).mockResolvedValueOnce(mockResponse);

      const result = await orderApi.getById(1);

      expect(result).toEqual(mockResponse);
    });
  });

  describe('alarmApi', () => {
    it('should get alarm stats', async () => {
      const mockStats = { total: 10, unprocessed: 5, high_level: 3 };
      (api.get as any).mockResolvedValueOnce({ data: mockStats });

      const result = await alarmApi.getStats();

      expect(result.data).toEqual(mockStats);
      expect(api.get).toHaveBeenCalledWith('/api/alerts/stats');
    });

    it('should get all alarms', async () => {
      const mockAlerts = { items: [], total: 0, page: 1, page_size: 10 };
      (api.get as any).mockResolvedValueOnce({ data: mockAlerts });

      const result = await alarmApi.getAll();

      expect(result.data).toEqual(mockAlerts);
    });
  });

  describe('authApi', () => {
    it('should login successfully', async () => {
      const mockLoginResponse = { data: { token: 'test-token' } };
      (api.post as any).mockResolvedValueOnce(mockLoginResponse);

      const result = await authApi.login('testuser', 'testpass');

      expect(result).toEqual(mockLoginResponse);
      expect(api.post).toHaveBeenCalledWith('/api/auth/login', {
        username: 'testuser',
        password: 'testpass',
      });
    });

    it('should logout successfully', async () => {
      (api.post as any).mockResolvedValueOnce(undefined);

      await authApi.logout();

      expect(api.post).toHaveBeenCalledWith('/api/auth/logout', {});
    });
  });

  describe('settingsApi', () => {
    it('should get settings', async () => {
      const mockSettings = { home_page_name: '测试平台' };
      (api.get as any).mockResolvedValueOnce({ data: mockSettings });

      const result = await settingsApi.getSettings();

      expect(result.data).toEqual(mockSettings);
    });

    it('should update settings', async () => {
      const updateData = { home_page_name: '新平台' };
      (api.put as any).mockResolvedValueOnce({ data: updateData });

      const result = await settingsApi.updateSettings(updateData);

      expect(result.data).toEqual(updateData);
    });
  });

  describe('statisticsApi', () => {
    it('should get vehicle statistics', async () => {
      const mockStats = { total_vehicles: 10, online_vehicles: 8, offline_vehicles: 2 };
      (api.get as any).mockResolvedValueOnce({ data: mockStats });

      const result = await statisticsApi.getVehicleStatistics();

      expect(result.data).toEqual(mockStats);
    });
  });

  describe('nodeApi', () => {
    it('should get all nodes', async () => {
      const mockNodes = { items: [], total: 0, page: 1, page_size: 10 };
      (api.get as any).mockResolvedValueOnce({ data: mockNodes });

      const result = await nodeApi.getAll();

      expect(result.data).toEqual(mockNodes);
    });
  });

  describe('dispatchApi', () => {
    it('should get dispatch devices', async () => {
      const mockDevices = [{ id: 1, name: '设备1' }];
      (api.get as any).mockResolvedValueOnce({ data: mockDevices });

      const result = await dispatchApi.getDevices();

      expect(result.data).toEqual(mockDevices);
    });
  });

  describe('departmentApi', () => {
    it('should get all departments', async () => {
      const mockResponse = { data: { items: [], total: 0, page: 1, page_size: 10 } };
      (api.get as any).mockResolvedValueOnce(mockResponse);

      const result = await departmentApi.getAll();

      expect(result).toEqual(mockResponse);
    });
  });

  describe('driverApi', () => {
    it('should get all drivers', async () => {
      const mockResponse = { data: { items: [], total: 0, page: 1, page_size: 10 } };
      (api.get as any).mockResolvedValueOnce(mockResponse);

      const result = await driverApi.getAll();

      expect(result).toEqual(mockResponse);
    });
  });

  describe('organizationApi', () => {
    it('should get all organizations', async () => {
      const mockResponse = { data: { items: [], total: 0, page: 1, page_size: 10 } };
      (api.get as any).mockResolvedValueOnce(mockResponse);

      const result = await organizationApi.getAll();

      expect(result).toEqual(mockResponse);
    });
  });

  describe('vehicleTeamApi', () => {
    it('should get all vehicle teams', async () => {
      const mockResponse = { data: { items: [], total: 0, page: 1, page_size: 10 } };
      (api.get as any).mockResolvedValueOnce(mockResponse);

      const result = await vehicleTeamApi.getAll();

      expect(result).toEqual(mockResponse);
    });
  });
});
