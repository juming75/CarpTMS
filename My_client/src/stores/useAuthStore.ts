import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

interface User {
  id: number;
  username: string;
  role: string;
  permissions: string[];
}

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(JSON.parse(sessionStorage.getItem('user') || 'null'));
  const access_token = ref<string | null>(sessionStorage.getItem('access_token'));
  const refresh_token = ref<string | null>(sessionStorage.getItem('refresh_token'));
  const is_authenticated = computed(() => !!access_token.value && !!user.value);

  const login = (new_user: User, new_access_token: string, new_refresh_token: string) => {
    user.value = new_user;
    access_token.value = new_access_token;
    refresh_token.value = new_refresh_token;
    sessionStorage.setItem('user', JSON.stringify(new_user));
    sessionStorage.setItem('access_token', new_access_token);
    sessionStorage.setItem('refresh_token', new_refresh_token);
  };

  const logout = () => {
    user.value = null;
    access_token.value = null;
    refresh_token.value = null;
    sessionStorage.removeItem('user');
    sessionStorage.removeItem('access_token');
    sessionStorage.removeItem('refresh_token');
  };

  const update_user = (updated_user: Partial<User>) => {
    if (user.value) {
      user.value = { ...user.value, ...updated_user };
      sessionStorage.setItem('user', JSON.stringify(user.value));
    }
  };

  const update_tokens = (new_access_token: string, new_refresh_token: string) => {
    access_token.value = new_access_token;
    refresh_token.value = new_refresh_token;
    sessionStorage.setItem('access_token', new_access_token);
    sessionStorage.setItem('refresh_token', new_refresh_token);
  };

  return {
    user,
    access_token,
    refresh_token,
    is_authenticated,
    login,
    logout,
    update_user,
    update_tokens
  };
});


