import { api } from './config';
import type { ApiResponse } from '@/types';

// ====== 认证 API ======
export const authApi = {
  // 登录
  login: async (username: string, password: string) => {
    // 使用真实的后端登录
    console.log('使用真实后端进行登录', { username, password });
    try {
      const response = await api.post('/api/auth/login', { username, password });
      console.log('登录响应:', response);
      
      // 标准化响应格式：后端返回 { code, message, data: { access_token, refresh_token, user } }
      const res = response as any;
      if (res && (res.access_token || res.data?.access_token)) {
        return {
          access_token: res.access_token || res.data?.access_token,
          refresh_token: res.refresh_token || res.data?.refresh_token,
          user: res.user || res.data?.user,
        };
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

  // 获取当前用户信息
  getCurrentUser: async (id: number) => {
    return await api.get<ApiResponse<unknown>>(`/api/auth/user/${id}`);
  },
};


