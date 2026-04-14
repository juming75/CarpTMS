import { ElMessage } from 'element-plus';
import axios from 'axios';
import { apiCache } from '@/services/apiCache';

// Token自动刷新相关配置
const TOKEN_REFRESH_THRESHOLD = 5 * 60 * 1000; // 提前5分钟刷新token
let isRefreshing = false; // 是否正在刷新token
let refreshSubscribers: Array<(token: string) => void> = []; // 等待刷新的请求队列

// 扩展Axios配置，添加重试支持和刷新标记
declare module 'axios' {
  interface AxiosRequestConfig {
    retry?: number;
    retryDelay?: number;
    retryableStatusCodes?: number[];
    retryCount?: number;
    _isRefreshRequest?: boolean; // 标记是否是刷新token的请求
  }
}

// 从 cookie 中获取 token
const getTokenFromCookie = (name: string): string | null => {
  // 只从 cookie 中获取，不使用 localStorage
  const cookieValue = document.cookie
    .split('; ')
    .find(row => row.startsWith(`${name}=`))
    ?.split('=')[1];
  if (cookieValue) {
    return decodeURIComponent(cookieValue);
  }
  
  // 如果没有，返回 null
  return null;
};

// 从Token中解析过期时间（假设JWT格式）
const parseTokenExpiration = (token: string): number | null => {
  try {
    if (!token) return null;

    // 解析JWT token的payload部分
    const payload = token.split('.')[1];
    if (!payload) return null;

    const decoded = JSON.parse(window.atob(payload.replace(/-/g, '+').replace(/_/g, '/')));
    return decoded.exp ? decoded.exp * 1000 : null; // 转换为毫秒
  } catch (error) {
    console.error('解析Token过期时间失败:', error);
    return null;
  }
};

// 检查Token是否即将过期（提前5分钟）
const isTokenExpiringSoon = (): boolean => {
  const token = getTokenFromCookie('access_token');
  if (!token) return false;

  const expTime = parseTokenExpiration(token);
  if (!expTime) return false;

  const now = Date.now();
  const timeLeft = expTime - now;

  // Token过期检查（不记录敏感信息）
  return timeLeft < TOKEN_REFRESH_THRESHOLD;
};

// 刷新Token的函数
const refreshToken = async (): Promise<string | null> => {
  try {
    const token = getTokenFromCookie('access_token');
    const refreshToken = getTokenFromCookie('refresh_token');

    if (!token || !refreshToken) {
      console.log('无法刷新Token：缺少有效token或refreshToken');
      return null;
    }

    // 开始刷新Token
    const response = await axios.post(
      '/api/auth/refresh',
      {
        refresh_token: refreshToken,
      },
      {
        _isRefreshRequest: true, // 标记为刷新请求
        headers: {
          Authorization: `Bearer ${token}`,
        },
      }
    );

    if (response.data && response.data.access_token && response.data.refresh_token) {
      // 保存新的token和refresh_token到 cookie
      const newToken = response.data.access_token;
      document.cookie = `access_token=${newToken}; path=/; HttpOnly; SameSite=Strict; Secure`;
      document.cookie = `refresh_token=${response.data.refresh_token}; path=/; HttpOnly; SameSite=Strict; Secure`;
      // Token刷新成功

      // 通知所有等待的请求
      refreshSubscribers.forEach((callback) => callback(newToken));
      refreshSubscribers = [];

      return newToken;
    }
    // 处理标准API响应格式: {"code": 200, "message": "success", "data": {...}}
    else if (response.data && response.data.data && response.data.data.access_token && response.data.data.refresh_token) {
      // 保存新的token和refresh_token到 cookie
      const newToken = response.data.data.access_token;
      document.cookie = `access_token=${newToken}; path=/; HttpOnly; SameSite=Strict; Secure`;
      document.cookie = `refresh_token=${response.data.data.refresh_token}; path=/; HttpOnly; SameSite=Strict; Secure`;
      // Token刷新成功

      // 通知所有等待的请求
      refreshSubscribers.forEach((callback) => callback(newToken));
      refreshSubscribers = [];

      return newToken;
    }

    console.error('Token刷新响应格式不正确');
    return null;
  } catch (error) {
    console.error('Token刷新失败:', error);

    // 通知所有等待的请求失败
    refreshSubscribers.forEach((callback) => callback(''));
    refreshSubscribers = [];

    return null;
  }
};

// 订阅Token刷新结果
const subscribeTokenRefresh = (callback: (token: string) => void): void => {
  refreshSubscribers.push(callback);
};

// 检查网络状态
const isOnline = (): boolean => {
  return typeof window !== 'undefined' && typeof window.navigator !== 'undefined' && window.navigator.onLine;
};

// API 基础配置
export const api = axios.create({
  baseURL: '', // 使用相对路径，通过Vite代理发送请求
  timeout: 15000, // 增加超时时间
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true, // 允许携带凭证
});

// 请求拦截器
api.interceptors.request.use(
  async (config) => {
    // 添加Authorization头
    let token = getTokenFromCookie('access_token');

    if (token) {
      // Authorization头已添加
      config.headers.Authorization = `Bearer ${token}`;
    }

    // 检查是否有缓存的响应 - 注意：不能在请求拦截器中直接返回响应数据
    // 因为这会导致axios将响应数据当作配置对象处理
    // 缓存逻辑已移至响应拦截器中处理

    // 如果不是刷新请求，检查Token是否即将过期
    if (config && !config._isRefreshRequest && token && isTokenExpiringSoon() && !isRefreshing && isOnline()) {
      return new Promise((resolve) => {
        // 如果正在刷新，将请求加入等待队列
        if (isRefreshing) {
          subscribeTokenRefresh((newToken) => {
            if (newToken) {
              config.headers.Authorization = `Bearer ${newToken}`;
            }
            resolve(config);
          });
          return;
        }

        // 否则，触发刷新
        isRefreshing = true;

        refreshToken()
          .then((newToken) => {
            if (newToken) {
              config.headers.Authorization = `Bearer ${newToken}`;
            }
            resolve(config);
          })
          .finally(() => {
            isRefreshing = false;
          });
      });
    }

    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// 响应拦截器 - 实现错误处理和缓存
api.interceptors.response.use(
  (response) => {
    // 缓存成功的GET请求响应
    if (
      response.config &&
      typeof response.config.method === 'string' &&
      response.config.method.toUpperCase() === 'GET' &&
      response.status === 200
    ) {
      apiCache.set(response.config, response.data);
    }

    // 检查响应结构，如果是API响应格式，直接返回data部分
    if (response.data) {
      // 处理后端API响应格式: {"code": 200, "message": "success", "data": {...}}
      if (response.data.code !== undefined && response.data.data !== undefined) {
        // 返回完整的响应对象，让调用者可以访问 code, message 和 data
        return response.data;
      }
      // 处理其他响应格式
      else {
        return response.data;
      }
    }
    return response;
  },
  async (error) => {
    // 统一的错误提示函数
    const showError = (message: string) => {
      ElMessage.error(message);
      return Promise.reject(error);
    };

    // 非模拟token的情况下，正常处理错误
    const config = error.config;

    // 处理401错误：Token过期或无效
    if (error.response?.status === 401) {
      // 检查是否是刷新请求本身失败
      if (config?._isRefreshRequest) {
        // 清除cookie中的token
      document.cookie = 'access_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC; HttpOnly; SameSite=Strict; Secure';
      document.cookie = 'refresh_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC; HttpOnly; SameSite=Strict; Secure';
        return showError('登录已过期，请重新登录');
      }

      // 检查是否有refresh_token
      const refreshToken = getTokenFromCookie('refresh_token');
      if (!refreshToken) {
        // 清除cookie中的token
      document.cookie = 'access_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC; HttpOnly; SameSite=Strict; Secure';
      document.cookie = 'refresh_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC; HttpOnly; SameSite=Strict; Secure';
        return showError('登录已过期，请重新登录');
      }

      // 尝试刷新Token
      try {
        const refreshResponse = await axios.post(
          '/api/auth/refresh',
          {
            refresh_token: refreshToken,
          },
          {
            _isRefreshRequest: true,
            headers: {
              Authorization: `Bearer ${getTokenFromCookie('access_token')}`,
            },
          }
        );

        if (refreshResponse.data && refreshResponse.data.access_token && refreshResponse.data.refresh_token) {
          // 保存新的token和refresh_token到 cookie
          const newToken = refreshResponse.data.access_token;
          document.cookie = `access_token=${newToken}; path=/; HttpOnly; SameSite=Strict; Secure`;
          document.cookie = `refresh_token=${refreshResponse.data.refresh_token}; path=/; HttpOnly; SameSite=Strict; Secure`;

          // 重新发送原请求
          config.headers.Authorization = `Bearer ${newToken}`;
          return api(config);
        }
        // 处理标准API响应格式: {"code": 200, "message": "success", "data": {...}}
        else if (refreshResponse.data && refreshResponse.data.data && refreshResponse.data.data.access_token && refreshResponse.data.data.refresh_token) {
          // 保存新的token和refresh_token到 cookie
          const newToken = refreshResponse.data.data.access_token;
          document.cookie = `access_token=${newToken}; path=/; HttpOnly; SameSite=Strict; Secure`;
          document.cookie = `refresh_token=${refreshResponse.data.data.refresh_token}; path=/; HttpOnly; SameSite=Strict; Secure`;

          // 重新发送原请求
          config.headers.Authorization = `Bearer ${newToken}`;
          return api(config);
        }
      } catch {
      // 刷新失败，清除cookie中的token
      document.cookie = 'access_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC; HttpOnly; SameSite=Strict; Secure';
      document.cookie = 'refresh_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC; HttpOnly; SameSite=Strict; Secure';
      // 同时清除localStorage中的token
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      localStorage.removeItem('userInfo');
      return showError('登录已过期，请重新登录');
    }
    }

    // 处理其他HTTP状态码
    if (error.response) {
      // 服务器返回了错误状态码
      const status = error.response.status;

      switch (status) {
        case 400:
          const badRequestMsg = error.response.data?.message || '请求参数错误';
          return showError(`请求失败: ${badRequestMsg}`);
        case 403:
          return showError('没有权限访问该资源');
        case 404:
          return showError('请求的资源不存在');
        case 500:
          return showError('服务器内部错误，请稍后重试');
        case 502:
          return showError('服务器网关错误，请稍后重试');
        case 503:
          return showError('服务器暂时不可用，请稍后重试');
        case 504:
          return showError('服务器网关超时，请稍后重试');
        default:
          const defaultMsg = error.response.data?.message || `服务器错误(${status})`;
          return showError(`请求失败: ${defaultMsg}`);
      }
    } else if (error.code) {
      // 网络错误或超时
      switch (error.code) {
        case 'ECONNABORTED':
          return showError('请求超时，请检查网络连接或服务器状态');
        case 'ERR_NETWORK':
          return showError('网络连接失败，请检查您的网络设置');
        case 'ERR_CONNECTION_REFUSED':
          return showError('服务器连接被拒绝，请检查服务器是否正常运行');
        default:
          return showError(`网络请求失败: ${error.message}`);
      }
    } else {
      // 其他未知错误
      return showError(`请求失败: ${error.message}`);
    }
  }
);

// 健康检查函数：检测后端服务是否可用
export const checkBackendHealth = async (): Promise<boolean> => {
  try {
    console.log('开始检测后端服务健康状态...');

    // 检查HTTP服务
    const response = await axios.get('/api/health', {
      timeout: 3000,
      withCredentials: true,
    });
    console.log('后端服务健康检查结果:', response.status, response.data);
    return response.status === 200;
  } catch (error) {
    console.error('后端服务健康检查失败:', error);
    return false;
  }
};


