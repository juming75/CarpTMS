import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { jwtBlacklist } from '../services/jwtBlacklist';

interface User {
  id: number;
  username: string;
  role: string;
  permissions: string[];
}

export const useAuthStore = defineStore('auth', () => {
  // 完全使用 HttpOnly Cookie 方案，前端不再存储 token
  // token 由后端通过 HttpOnly Cookie 管理，前端只存用户基本信息（非敏感）
  const user = ref<User | null>(JSON.parse(localStorage.getItem('user') || 'null'));
  // access_token 和 refresh_token 不再由前端管理，完全依赖后端 Cookie
  const is_authenticated = computed(() => !!user.value);

  const login = (new_user: User) => {
    user.value = new_user;
    localStorage.setItem('user', JSON.stringify(new_user));
    // 注意：token 由后端通过 HttpOnly Cookie 设置，前端不触碰
  };

  const logout = async () => {
    try {
      const { authApi } = await import('../api/index');
      await authApi.logout();
    } catch (error) {
      console.error('Logout error:', error);
    }
    user.value = null;
    localStorage.removeItem('user');
    localStorage.removeItem('userId');
    // Cookie 由后端清除或自然过期
  };

  const update_user = (updated_user: Partial<User>) => {
    if (user.value) {
      user.value = { ...user.value, ...updated_user };
      localStorage.setItem('user', JSON.stringify(user.value));
    }
  };

  const restore_session = async () => {
    try {
      const { default: api } = await import('../api/index');
      const userId = localStorage.getItem('userId');
      if (!userId) {
        user.value = null;
        localStorage.removeItem('user');
        return;
      }
      const response = await api.get(`/api/auth/user/${userId}`);
      const userData = response?.data?.user || response?.data || response;
      if (userData && typeof userData === 'object' && 'username' in userData) {
        user.value = userData as User;
        localStorage.setItem('user', JSON.stringify(userData));
      }
    } catch {
      user.value = null;
      localStorage.removeItem('user');
      localStorage.removeItem('userId');
    }
  };

  const clear_auth = () => {
    user.value = null;
    localStorage.removeItem('user');
    localStorage.removeItem('userId');
    // 清理所有可能的token存储
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    localStorage.removeItem('userInfo');
    // 清理JWT黑名单
    jwtBlacklist.clearAll();
  };

  return {
    user,
    is_authenticated,
    login,
    logout,
    update_user,
    restore_session,
    clear_auth
  };
});


