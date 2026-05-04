import { api } from './config';
import type { ApiResponse } from '@/types';

// ====== 认证 API ======
export const authApi = {
  // 登录
  // FIX: 使用 HttpOnly Cookie 方案，后端通过 Set-Cookie 设置 token
  // 前端不再接收/存储 token，只接收用户信息
  login: async (username: string, password: string) => {
    console.log('使用真实后端进行登录', { username });
    try {
      const response = await api.post('/api/auth/login', { username, password });
      console.log('登录响应:', response);
      
      // 标准化响应格式：后端返回 { code, message, data: { user } }
      // token 由后端 HttpOnly Cookie 管理
      const res = response as any;
      const user = res.user || res.data?.user;
      
      if (user) {
        return { user };
      }
      
      return response;
    } catch (error) {
      console.error('登录错误:', error);
      throw error;
    }
  },

  // 登出
  logout: async () => {
    return await api.post('/api/auth/logout', {});
  },

  // 获取当前用户信息（用于恢复会话）
  getCurrentUser: async () => {
    return await api.get<ApiResponse<unknown>>('/api/auth/user');
  },
};


