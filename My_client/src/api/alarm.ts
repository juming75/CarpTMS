import { api } from './config';
import type { ApiResponse, Alarm } from '@/types';

// ====== 报警 API ======
export const alarmApi = {
  // 获取报警统计数据
  getStats: async () => {
    try {
      return await api.get<ApiResponse<{ total: number; unprocessed: number; high_level: number }>>('/api/alerts/stats');
    } catch (error) {
      console.error('获取报警统计数据失败:', error);
      throw error;
    }
  },

  // 获取报警列表
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ items: Alarm[]; total: number; page: number; page_size: number }>>('/api/alerts', { params });
    } catch (error) {
      console.error('获取报警列表失败:', error);
      throw error;
    }
  },

  // 获取快速处理项
  getQuickProcess: async () => {
    try {
      return await api.get<ApiResponse<Array<{ id: number; name: string; action: string }>>>('/api/alerts/quick-process');
    } catch (error) {
      console.error('获取快速处理项失败:', error);
      throw error;
    }
  },

  // 获取报警趋势
  getTrend: async () => {
    try {
      return await api.get<ApiResponse<Array<{ time: string; count: number }>>>('/api/alerts/trend');
    } catch (error) {
      console.error('获取报警趋势失败:', error);
      throw error;
    }
  },

  // 获取报警类型分布
  getTypes: async () => {
    try {
      return await api.get<ApiResponse<Array<{ type: string; count: number; percentage: number }>>>('/api/alerts/types');
    } catch (error) {
      console.error('获取报警类型分布失败:', error);
      throw error;
    }
  },

  // 处理报警
  process: async (id: number, data: { processing_result: string; processing_user_id: number }) => {
    try {
      return await api.put<ApiResponse<Alarm>>(`/api/alerts/${id}/process`, data);
    } catch (error) {
      console.error(`处理报警 ${id} 失败:`, error);
      throw error;
    }
  },
};


