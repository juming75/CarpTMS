import { api } from './config';
import type { ApiResponse, Order } from '@/types';

// ====== 订单管理 API ======
export const orderApi = {
  // 获取所有订单
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ list: Order[]; total: number; page: number; page_size: number }>>(
        '/api/orders',
        { params }
      );
    } catch (error) {
      console.error('获取订单列表失败:', error);
      throw error;
    }
  },

  // 获取单个订单
  getById: async (id: number) => {
    try {
      return await api.get<ApiResponse<Order>>(`/api/orders/${id}`);
    } catch (error) {
      console.error(`获取订单 ${id} 失败:`, error);
      throw error;
    }
  },

  // 创建订单
  create: async (data: Omit<Order, 'order_id'>) => {
    try {
      return await api.post<ApiResponse<Order>>('/api/orders', data);
    } catch (error) {
      console.error('创建订单失败:', error);
      throw error;
    }
  },

  // 更新订单
  update: async (id: number, data: Partial<Order>) => {
    try {
      return await api.put<ApiResponse<Order>>(`/api/orders/${id}`, data);
    } catch (error) {
      console.error(`更新订单 ${id} 失败:`, error);
      throw error;
    }
  },

  // 删除订单
  delete: async (id: number) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/orders/${id}`);
    } catch (error) {
      console.error(`删除订单 ${id} 失败:`, error);
      throw error;
    }
  },
};


