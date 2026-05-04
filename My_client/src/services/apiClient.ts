import axios, { type AxiosRequestConfig, type AxiosResponse } from 'axios';
import { ElMessage } from 'element-plus';
import type { ApiResponse } from '@/types';
import { apiCache } from './apiCache';
import { forceLogout } from './authService';
import { trackAPICall } from '@/utils/performanceMonitor';

declare module 'axios' {
  interface AxiosRequestConfig {
    retry?: number;
    retryDelay?: number;
    retryableStatusCodes?: number[];
    retryCount?: number;
    _isRefreshRequest?: boolean;
    _isAuthEndpoint?: boolean;
    _retry?: boolean;
    _startTime?: number;
  }
}

const LOGIN_ENDPOINT = '/api/auth/login';
const REFRESH_ENDPOINT = '/api/auth/refresh';

const isLoginRequest = (url?: string): boolean => {
  if (!url) return false;
  return url.includes(LOGIN_ENDPOINT);
};

const isRefreshRequest = (url?: string): boolean => {
  if (!url) return false;
  return url.includes(REFRESH_ENDPOINT);
};

const isAuthEndpoint = (url?: string): boolean => {
  return isLoginRequest(url) || isRefreshRequest(url);
};

const api = axios.create({
  baseURL: '',
  timeout: 15000,
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true,
});

const isApiResponse = <T>(value: unknown): value is ApiResponse<T> => {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    'message' in value &&
    'data' in value
  );
};

const shouldCache = (config?: AxiosRequestConfig): boolean => {
  return !!config?.method && config.method.toString().toUpperCase() === 'GET' && config.responseType !== 'blob';
};

const cacheResponse = (config: AxiosRequestConfig | undefined, responseData: unknown): void => {
  if (shouldCache(config)) {
    apiCache.set(config as AxiosRequestConfig, responseData);
  }
};

const rejectWith = (message: string): Promise<never> => {
  return Promise.reject(new Error(message));
};

const clearClientSessionData = (): void => {
  localStorage.removeItem('access_token');
  localStorage.removeItem('refresh_token');
  localStorage.removeItem('userInfo');
};

const refreshToken = async (): Promise<boolean> => {
  try {
    const storedRefreshToken = localStorage.getItem('refresh_token');
    if (!storedRefreshToken) {
      return false;
    }

    const response = await api.post(
      REFRESH_ENDPOINT,
      { refresh_token: storedRefreshToken },
      {
        withCredentials: true,
        _isRefreshRequest: true,
        _isAuthEndpoint: true,
      }
    );

    if (response && typeof response === 'object') {
      const resp = response as unknown as Record<string, unknown>;
      const newAccessToken = resp.access_token as string | undefined;
      const newRefreshToken = resp.refresh_token as string | undefined;
      if (newAccessToken) {
        localStorage.setItem('access_token', newAccessToken);
      }
      if (newRefreshToken) {
        localStorage.setItem('refresh_token', newRefreshToken);
      }
    }

    return true;
  } catch {
    return false;
  }
};

const handle401 = async (config?: AxiosRequestConfig): Promise<never> => {
  const url = config?.url || '';

  if (isLoginRequest(url)) {
    return rejectWith('用户名或密码错误');
  }

  if (isRefreshRequest(url) || config?._isRefreshRequest || config?._retry) {
    clearClientSessionData();
    forceLogout();
    ElMessage.error('登录已过期，请重新登录');
    return rejectWith('登录已过期，请重新登录');
  }

  const retryConfig: AxiosRequestConfig = config ?? {};
  retryConfig._retry = true;
  const refreshed = await refreshToken();
  if (refreshed) {
    const token = localStorage.getItem('access_token');
    if (token && retryConfig.headers) {
      retryConfig.headers['Authorization'] = `Bearer ${token}`;
    }
    return api(retryConfig);
  }

  clearClientSessionData();
  forceLogout();
  ElMessage.error('登录已过期，请重新登录');
  return rejectWith('登录已过期，请重新登录');
};

const isTokenExpired = (token: string): boolean => {
  try {
    const parts = token.split('.');
    if (parts.length !== 3) return true;
    const payload = JSON.parse(atob(parts[1]));
    if (!payload.exp) return false;
    return Date.now() >= payload.exp * 1000;
  } catch {
    return true;
  }
};

api.interceptors.request.use(
  (config) => {
    config._startTime = Date.now();

    if (isAuthEndpoint(config.url)) {
      config._isAuthEndpoint = true;
    }

    const token = localStorage.getItem('access_token');
    if (token && !config.headers['Authorization'] && !config._isAuthEndpoint) {
      if (isTokenExpired(token)) {
        localStorage.removeItem('access_token');
      } else {
        config.headers['Authorization'] = `Bearer ${token}`;
      }
    }

    return config;
  },
  (error) => Promise.reject(error)
);

api.interceptors.response.use(
  <T>(response: AxiosResponse<T>): T => {
    if (response.config._startTime) {
      const duration = Date.now() - response.config._startTime;
      trackAPICall(
        response.config.url || '',
        response.config.method?.toUpperCase() || 'GET',
        duration,
        response.status
      );
    }

    if (response.config && response.config.responseType === 'blob') {
      return response as unknown as T;
    }

    const data = response.data;
    if (isApiResponse<T>(data)) {
      if (data.code !== 200 && data.code !== 201) {
        return rejectWith(data.message || '请求失败') as unknown as T;
      }

      cacheResponse(response.config, response.data);
      return data.data as T;
    }

    if (data && typeof data === 'object' && 'data' in (data as Record<string, unknown>)) {
      cacheResponse(response.config, response.data);
      return (data as unknown as { data: T }).data;
    }

    if (Array.isArray(data)) {
      return { list: data } as unknown as T;
    }

    return data as T;
  },
  async (error) => {
    const config = error.config as AxiosRequestConfig | undefined;
    if (error.response?.status === 401) {
      return handle401(config);
    }

    console.warn('[API Error]', error.response?.status, error.config?.url, error.message);
    return rejectWith(error.response?.data?.message || `服务器错误(${error.response?.status || 'unknown'})`);
  }
);

export { api };
