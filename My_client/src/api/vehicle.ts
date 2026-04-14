import { api } from './config';
import type { ApiResponse, Vehicle } from '@/types';

// ====== 车辆管理 API ======
export const vehicleApi = {
  // 获取所有车辆
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      // 直接调用后端API获取车辆列表
      console.log('直接调用后端API获取车辆列表');
      return await api.get<ApiResponse<{ list: Vehicle[]; total: number; page: number; page_size: number }>>(
        '/api/vehicles',
        { params }
      );
    } catch (error) {
      console.error('获取车辆列表失败:', error);
      throw error;
    }
  },

  // 获取单个车辆
  getById: async (id: number) => {
    try {
      // 直接调用后端API获取单个车辆
      return await api.get<ApiResponse<Vehicle>>(`/api/vehicles/${id}`);
    } catch (error) {
      console.error(`获取车辆 ${id} 失败:`, error);
      throw error;
    }
  },

  // 创建车辆
  create: async (data: Omit<Vehicle, 'vehicle_id'>) => {
    try {
      // 直接调用后端API创建车辆
      return await api.post<ApiResponse<Vehicle>>('/api/vehicles', data);
    } catch (error) {
      console.error('创建车辆失败:', error);
      throw error;
    }
  },

  // 更新车辆
  update: async (id: number, data: Partial<Vehicle>) => {
    try {
      // 直接调用后端API更新车辆
      return await api.put<ApiResponse<Vehicle>>(`/api/vehicles/${id}`, data);
    } catch (error) {
      console.error(`更新车辆 ${id} 失败:`, error);
      throw error;
    }
  },

  // 删除车辆
  delete: async (id: number) => {
    try {
      // 直接调用后端API删除车辆
      return await api.delete<ApiResponse<void>>(`/api/vehicles/${id}`);
    } catch (error) {
      console.error(`删除车辆 ${id} 失败:`, error);
      throw error;
    }
  },
};

// ====== 车队管理 API ======
export const vehicleTeamApi = {
  // 获取所有车队
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ list: any[]; total: number; page: number; page_size: number }>>(
        '/api/vehicle-groups',
        { params }
      );
    } catch (error) {
      console.error('获取车队列表失败:', error);
      throw error;
    }
  },

  // 获取单个车队
  getById: async (id: number) => {
    try {
      return await api.get<ApiResponse<any>>(`/api/vehicle-groups/${id}`);
    } catch (error) {
      console.error(`获取车队 ${id} 失败:`, error);
      throw error;
    }
  },

  // 创建车队
  create: async (data: any) => {
    try {
      return await api.post<ApiResponse<any>>('/api/vehicle-groups', data);
    } catch (error) {
      console.error('创建车队失败:', error);
      throw error;
    }
  },

  // 更新车队
  update: async (id: number, data: Partial<any>) => {
    try {
      return await api.put<ApiResponse<any>>(`/api/vehicle-groups/${id}`, data);
    } catch (error) {
      console.error(`更新车队 ${id} 失败:`, error);
      throw error;
    }
  },

  // 删除车队
  delete: async (id: number) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/vehicle-groups/${id}`);
    } catch (error) {
      console.error(`删除车队 ${id} 失败:`, error);
      throw error;
    }
  },
};


