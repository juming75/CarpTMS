import { api } from './config';
import type { ApiResponse, Driver } from '@/types';

// ====== 司机管理 API ======
export const driverApi = {
  // 获取所有司机
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ list: Driver[]; total: number; page: number; page_size: number }>>(
        '/api/drivers',
        { params }
      );
    } catch (error) {
      console.error('获取司机列表失败:', error);
      throw error;
    }
  },

  // 获取单个司机
  getById: async (id: number) => {
    try {
      return await api.get<ApiResponse<Driver>>(`/api/drivers/${id}`);
    } catch (error) {
      console.error(`获取司机 ${id} 失败:`, error);
      throw error;
    }
  },

  // 创建司机
  create: async (data: Omit<Driver, 'driver_id'>) => {
    try {
      return await api.post<ApiResponse<Driver>>('/api/drivers', data);
    } catch (error) {
      console.error('创建司机失败:', error);
      throw error;
    }
  },

  // 更新司机
  update: async (id: number, data: Partial<Driver>) => {
    try {
      return await api.put<ApiResponse<Driver>>(`/api/drivers/${id}`, data);
    } catch (error) {
      console.error(`更新司机 ${id} 失败:`, error);
      throw error;
    }
  },

  // 删除司机
  delete: async (id: number) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/drivers/${id}`);
    } catch (error) {
      console.error(`删除司机 ${id} 失败:`, error);
      throw error;
    }
  },
};


