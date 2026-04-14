import { ElMessage } from 'element-plus';

import axios from 'axios';
import type { AxiosResponse } from 'axios';


import type { ApiResponse, PaginationResponse, Vehicle, WeighingData, Order, Driver, Department, Organization, VehicleGroup, Alarm, Node, SystemSettings, ServiceStatus } from '@/types';
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
  const token = localStorage.getItem('access_token');
  if (!token) return false;

  const expTime = parseTokenExpiration(token);
  if (!expTime) return false;

  const now = Date.now();
  const timeLeft = expTime - now;

  console.log('Token过期检查:', {
    now: now,
    expTime: expTime,
    timeLeft: timeLeft,
    threshold: TOKEN_REFRESH_THRESHOLD,
    isExpiringSoon: timeLeft < TOKEN_REFRESH_THRESHOLD,
  });

  return timeLeft < TOKEN_REFRESH_THRESHOLD;
};

// 刷新Token的函数
const refreshToken = async (): Promise<string | null> => {
  try {
    const token = localStorage.getItem('access_token');
    const refreshToken = localStorage.getItem('refresh_token');

    if (!token || !refreshToken) {
      console.log('无法刷新Token：缺少有效token或refresh_token');
      return null;
    }

    console.log('开始刷新Token...');
    // 注意：这里直接用 axios.post，不用 api.post，否则会递归触发拦截器
    const response = await axios.post(
      '/api/auth/refresh',
      {
        refresh_token: refreshToken,
      },
      {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      }
    );

    const data = (response as any).data ?? response.data;
    if (data && data.access_token && data.refresh_token) {
      // 保存新的token和refresh_token
      localStorage.setItem('access_token', data.access_token);
      localStorage.setItem('refresh_token', data.refresh_token);
      console.log('Token和Refresh Token刷新成功');

      // 通知所有等待的请求
      refreshSubscribers.forEach((callback) => callback(data.access_token));
      refreshSubscribers = [];

      return data.access_token;
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
const api = axios.create({
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
    let token = localStorage.getItem('access_token');

    if (token) {
      console.log('添加Authorization头:', `Bearer ${token.substring(0, 20)}...`);
      config.headers.Authorization = `Bearer ${token}`;
    } else {
      console.log('没有找到token');
    }

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
// 注意：此拦截器将所有响应解包为 T（response.data），因此 api.get<T>() 返回 Promise<T> 而非 Promise<AxiosResponse<T>>
api.interceptors.response.use(
  <T>(response: AxiosResponse<T>): T => {
    // 缓存成功的GET请求响应
    if (
      response.config &&
      typeof response.config.method === 'string' &&
      response.config.method.toUpperCase() === 'GET' &&
      response.status === 200 &&
      response.config.responseType !== 'blob' // 不缓存blob类型的响应
    ) {
      // @ts-ignore - 缓存 T 类型数据
      apiCache.set(response.config, response.data);
    }

    // 特殊处理blob类型的响应（用于文件下载）
    if (response.config && response.config.responseType === 'blob') {
      return response as any;
    }

    // response.data 的实际类型是 T（由调用方通过 api.get<T>() 指定）
    const data = response.data as any;

    // 检查响应结构，统一API响应格式
    if (data) {
      // 处理标准API响应格式: {"data": {...}}
      if (data.data !== undefined) {
        return data.data as T;
      }
      // 处理简化响应格式: {"access_token": "...", "user": {...}}
      if (data.access_token || data.token) {
        return data as T;
      }
      // 处理其他响应格式，统一为标准格式
      if (Array.isArray(data)) {
        return { list: data } as T;
      }
      // 检查是否是分页格式
      if (data.items || data.list) {
        return data as T;
      }
      // 其他情况，直接返回数据
      return data as T;
    }
    return data as T;
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
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
        localStorage.removeItem('userInfo');
        return showError('登录已过期，请重新登录');
      }

      // 检查是否有refresh_token
      const refreshToken = localStorage.getItem('refresh_token');
      if (!refreshToken) {
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
        localStorage.removeItem('userInfo');
        return showError('登录已过期，请重新登录');
      }

      // 尝试刷新Token
      try {
        // 直接用 axios.post，不用 api.post，避免递归
        const refreshResponse = await axios.post(
          '/api/auth/refresh',
          {
            refresh_token: refreshToken,
          },
          {
            headers: {
              Authorization: `Bearer ${localStorage.getItem('access_token')}`,
            },
          }
        );

        const data = (refreshResponse as any).data ?? refreshResponse.data;
        if (data && data.access_token && data.refresh_token) {
          // 保存新的token和refresh_token
          localStorage.setItem('access_token', data.access_token);
          localStorage.setItem('refresh_token', data.refresh_token);

          // 重新发送原请求
          config.headers.Authorization = `Bearer ${data.access_token}`;
          return api(config);
        }
      } catch {
        // 刷新失败，清除本地token
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
        case 400: {
          const badRequestMsg = error.response.data?.message || '请求参数错误';
          return showError(`请求失败: ${badRequestMsg}`);
        }
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
        default: {
          const defaultMsg = error.response.data?.message || `服务器错误(${status})`;
          return showError(`请求失败: ${defaultMsg}`);
        }
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

// ====== 车辆管理 API ======
// 所有方法返回已解包的数据（由拦截器返回），即 Promise<T> 而非 Promise<AxiosResponse<T>>
export const vehicleApi = {
  // 获取所有车辆
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      console.log('直接调用后端API获取车辆列表');
      return await api.get<ApiResponse<{ items: Vehicle[]; total: number; page: number; page_size: number }>>(
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
      return await api.get<ApiResponse<Vehicle>>(`/api/vehicles/${id}`);
    } catch (error) {
      console.error(`获取车辆 ${id} 失败:`, error);
      throw error;
    }
  },

  // 创建车辆
  create: async (data: Omit<Vehicle, 'vehicle_id'>) => {
    try {
      return await api.post<ApiResponse<Vehicle>>('/api/vehicles', data);
    } catch (error) {
      console.error('创建车辆失败:', error);
      throw error;
    }
  },

  // 更新车辆
  update: async (id: number, data: Partial<Vehicle>) => {
    try {
      return await api.put<ApiResponse<Vehicle>>(`/api/vehicles/${id}`, data);
    } catch (error) {
      console.error(`更新车辆 ${id} 失败:`, error);
      throw error;
    }
  },

  // 删除车辆
  delete: async (id: number) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/vehicles/${id}`);
    } catch (error) {
      console.error(`删除车辆 ${id} 失败:`, error);
      throw error;
    }
  },
};

// ====== 称重数据 API ======
export const weighingApi = {
  getAll: async (params?: Record<string, string | number | boolean>) => {
    return await api.get<ApiResponse<WeighingData[]>>('/api/weighing', { params });
  },
  create: async (data: Omit<WeighingData, 'id'>) => {
    return await api.post<ApiResponse<WeighingData>>('/api/weighing', data);
  },
  getHistory: async (params?: Record<string, string | number | boolean>) => {
    return await api.get<ApiResponse<WeighingData[]>>('/api/weighing/history', { params });
  },
};

// ====== 报表 API ======
export const reportApi = {
  getTemplates: async () => api.get('/api/reports/templates'),
  getData: async (params?: Record<string, string | number | boolean>) =>
    api.get('/api/reports/data', { params }),
  generate: async (data: Record<string, string | number | boolean | object>) =>
    api.post('/api/reports/generate', data),
  export: async (params?: Record<string, string | number | boolean>) =>
    api.get('/api/reports/export', { params, responseType: 'blob' }),
};

// ====== 数据同步 API ======
export const syncApi = {
  upload: async (data: Record<string, string | number | boolean | object>) =>
    api.post('/api/sync/upload', data),
  download: async (params?: Record<string, string | number | boolean>) =>
    api.get('/api/sync/download', { params }),
  getStatus: async () => api.get('/api/sync/status'),
};

// ====== 订单管理 API ======
export const orderApi = {
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ items: Order[]; total: number; page: number; page_size: number }>>(
        '/api/orders',
        { params }
      );
    } catch (error) {
      console.error('获取订单列表失败:', error);
      throw error;
    }
  },
  getById: async (id: number) => {
    try {
      return await api.get<ApiResponse<Order>>(`/api/orders/${id}`);
    } catch (error) {
      console.error(`获取订单 ${id} 失败:`, error);
      throw error;
    }
  },
  create: async (data: Omit<Order, 'order_id'>) => {
    try {
      return await api.post<ApiResponse<Order>>('/api/orders', data);
    } catch (error) {
      console.error('创建订单失败:', error);
      throw error;
    }
  },
  update: async (id: number, data: Partial<Order>) => {
    try {
      return await api.put<ApiResponse<Order>>(`/api/orders/${id}`, data);
    } catch (error) {
      console.error(`更新订单 ${id} 失败:`, error);
      throw error;
    }
  },
  delete: async (id: number) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/orders/${id}`);
    } catch (error) {
      console.error(`删除订单 ${id} 失败:`, error);
      throw error;
    }
  },
};

// ====== 报警 API ======
export const alarmApi = {
  getStats: async () => {
    try {
      return await api.get<ApiResponse<{ total: number; unprocessed: number; high_level: number }>>('/api/alerts/stats');
    } catch (error) {
      console.error('获取报警统计数据失败:', error);
      throw error;
    }
  },
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ items: Alarm[]; total: number; page: number; page_size: number }>>(
        '/api/alerts',
        { params }
      );
    } catch (error) {
      console.error('获取报警列表失败:', error);
      throw error;
    }
  },
  getQuickProcess: async () => {
    try {
      return await api.get<ApiResponse<Array<{ id: number; name: string; action: string }>>>('/api/alerts/quick-process');
    } catch (error) {
      console.error('获取快速处理项失败:', error);
      throw error;
    }
  },
  getTrend: async () => {
    try {
      return await api.get<ApiResponse<Array<{ time: string; count: number }>>>('/api/alerts/trend');
    } catch (error) {
      console.error('获取报警趋势失败:', error);
      throw error;
    }
  },
  getTypes: async () => {
    try {
      return await api.get<ApiResponse<Array<{ type: string; count: number; percentage: number }>>>('/api/alerts/types');
    } catch (error) {
      console.error('获取报警类型分布失败:', error);
      throw error;
    }
  },
  process: async (id: number, data: { processing_result: string; processing_user_id: number }) => {
    try {
      return await api.put<ApiResponse<Alarm>>(`/api/alerts/${id}/process`, data);
    } catch (error) {
      console.error(`处理报警 ${id} 失败:`, error);
      throw error;
    }
  },
};

// ====== 认证 API ======
export const authApi = {
  login: async (username: string, password: string) => {
    console.log('使用真实后端进行登录', { username, password });
    try {
      const response = await api.post<ApiResponse<unknown>>('/api/auth/login', { username, password });
      console.log('登录响应:', response);
      return response;
    } catch (error) {
      console.error('登录错误:', error);
      throw error;
    }
  },
  logout: async () => api.post('/api/auth/logout', {}),
  getCurrentUser: async (id: number) =>
    api.get<ApiResponse<unknown>>(`/api/auth/user/${id}`),
};

// ====== 司机管理 API ======
export const driverApi = {
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ items: Driver[]; total: number; page: number; page_size: number }>>(
        '/api/drivers',
        { params }
      );
    } catch (error) {
      console.error('获取司机列表失败:', error);
      throw error;
    }
  },
  getById: async (id: number) => {
    try {
      return await api.get<ApiResponse<Driver>>(`/api/drivers/${id}`);
    } catch (error) {
      console.error(`获取司机 ${id} 失败:`, error);
      throw error;
    }
  },
  create: async (data: Omit<Driver, 'driver_id'>) => {
    try {
      return await api.post<ApiResponse<Driver>>('/api/drivers', data);
    } catch (error) {
      console.error('创建司机失败:', error);
      throw error;
    }
  },
  update: async (id: number, data: Partial<Driver>) => {
    try {
      return await api.put<ApiResponse<Driver>>(`/api/drivers/${id}`, data);
    } catch (error) {
      console.error(`更新司机 ${id} 失败:`, error);
      throw error;
    }
  },
  delete: async (id: number) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/drivers/${id}`);
    } catch (error) {
      console.error(`删除司机 ${id} 失败:`, error);
      throw error;
    }
  },
};

// ====== 系统设置 API ======
export const settingsApi = {
  getSettings: async () => {
    try {
      return await api.get<ApiResponse<SystemSettings>>('/api/settings');
    } catch (error) {
      console.error('获取系统设置失败:', error);
      throw error;
    }
  },
  updateSettings: async (data: Partial<SystemSettings>) => {
    try {
      return await api.put<ApiResponse<SystemSettings>>('/api/settings', data);
    } catch (error) {
      console.error('更新系统设置失败:', error);
      throw error;
    }
  },
  getCommunicationSettings: async () => {
    try {
      return await api.get<ApiResponse<SystemSettings['communication']>>('/api/settings/communication');
    } catch (error) {
      console.error('获取通信设置失败:', error);
      throw error;
    }
  },
  updateCommunicationSettings: async (data: SystemSettings['communication']) => {
    try {
      return await api.put<ApiResponse<SystemSettings['communication']>>('/api/settings/communication', data);
    } catch (error) {
      console.error('更新通信设置失败:', error);
      throw error;
    }
  },
  checkServiceStatus: async () => {
    try {
      return await api.get<ApiResponse<ServiceStatus[]>>('/api/services/status');
    } catch (error) {
      console.error('检查服务状态失败:', error);
      throw error;
    }
  },
  startService: async (serviceName: string) => {
    try {
      return await api.post<ApiResponse<ServiceStatus>>(`/api/services/${serviceName}/start`, {});
    } catch (error) {
      console.error(`启动服务 ${serviceName} 失败:`, error);
      throw error;
    }
  },
  stopService: async (serviceName: string) => {
    try {
      return await api.post<ApiResponse<ServiceStatus>>(`/api/services/${serviceName}/stop`, {});
    } catch (error) {
      console.error(`停止服务 ${serviceName} 失败:`, error);
      throw error;
    }
  },
  restartService: async (serviceName: string) => {
    try {
      return await api.post<ApiResponse<ServiceStatus>>(`/api/services/${serviceName}/restart`, {});
    } catch (error) {
      console.error(`重启服务 ${serviceName} 失败:`, error);
      throw error;
    }
  },
};

// ====== 部门管理 API ======
export const departmentApi = {
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ items: Department[]; total: number; page: number; page_size: number }>>(
        '/api/departments',
        { params }
      );
    } catch (error) {
      console.error('获取部门列表失败:', error);
      throw error;
    }
  },
  getById: async (id: number) => {
    try {
      return await api.get<ApiResponse<Department>>(`/api/departments/${id}`);
    } catch (error) {
      console.error(`获取部门 ${id} 失败:`, error);
      throw error;
    }
  },
  create: async (data: Omit<Department, 'department_id'>) => {
    try {
      return await api.post<ApiResponse<Department>>('/api/departments', data);
    } catch (error) {
      console.error('创建部门失败:', error);
      throw error;
    }
  },
  update: async (id: number, data: Partial<Department>) => {
    try {
      return await api.put<ApiResponse<Department>>(`/api/departments/${id}`, data);
    } catch (error) {
      console.error(`更新部门 ${id} 失败:`, error);
      throw error;
    }
  },
  delete: async (id: number) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/departments/${id}`);
    } catch (error) {
      console.error(`删除部门 ${id} 失败:`, error);
      throw error;
    }
  },
};

// ====== 组织单位管理 API ======
export const organizationApi = {
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ items: Organization[]; total: number; page: number; page_size: number }>>(
        '/api/organizations',
        { params }
      );
    } catch (error) {
      console.error('获取组织单位列表失败:', error);
      throw error;
    }
  },
  getById: async (id: string) => {
    try {
      return await api.get<ApiResponse<Organization>>(`/api/organizations/${id}`);
    } catch (error) {
      console.error(`获取组织单位 ${id} 失败:`, error);
      throw error;
    }
  },
  create: async (data: Omit<Organization, 'organization_id'>) => {
    try {
      return await api.post<ApiResponse<Organization>>('/api/organizations', data);
    } catch (error) {
      console.error('创建组织单位失败:', error);
      throw error;
    }
  },
  update: async (id: string, data: Partial<Organization>) => {
    try {
      return await api.put<ApiResponse<Organization>>(`/api/organizations/${id}`, data);
    } catch (error) {
      console.error(`更新组织单位 ${id} 失败:`, error);
      throw error;
    }
  },
  delete: async (id: string) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/organizations/${id}`);
    } catch (error) {
      console.error(`删除组织单位 ${id} 失败:`, error);
      throw error;
    }
  },
  updateStatus: async (id: string, status: string) => {
    try {
      return await api.put<ApiResponse<Organization>>(`/api/organizations/${id}/status`, { status });
    } catch (error) {
      console.error(`更新组织单位 ${id} 状态失败:`, error);
      throw error;
    }
  },
  getSettings: async (organizationId: string, settingKey: string) => {
    try {
      return await api.get<ApiResponse<unknown>>(`/api/organizations/${organizationId}/settings/${settingKey}`);
    } catch (error) {
      console.error(`获取组织 ${organizationId} 设置失败:`, error);
      throw error;
    }
  },
  updateSettings: async (
    organizationId: string,
    settingKey: string,
    data: Record<string, string | number | boolean | object>
  ) => {
    try {
      return await api.put<ApiResponse<unknown>>(
        `/api/organizations/${organizationId}/settings/${settingKey}`,
        data
      );
    } catch (error) {
      console.error(`更新组织 ${organizationId} 设置失败:`, error);
      throw error;
    }
  },
};

// ====== 车队管理 API ======
export const vehicleTeamApi = {
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ items: VehicleGroup[]; total: number; page: number; page_size: number }>>(
        '/api/vehicle-groups',
        { params }
      );
    } catch (error) {
      console.error('获取车队列表失败:', error);
      throw error;
    }
  },
  getById: async (id: number) => {
    try {
      return await api.get<ApiResponse<VehicleGroup>>(`/api/vehicle-groups/${id}`);
    } catch (error) {
      console.error(`获取车队 ${id} 失败:`, error);
      throw error;
    }
  },
  create: async (data: Omit<VehicleGroup, 'group_id'>) => {
    try {
      return await api.post<ApiResponse<VehicleGroup>>('/api/vehicle-groups', data);
    } catch (error) {
      console.error('创建车队失败:', error);
      throw error;
    }
  },
  update: async (id: number, data: Partial<VehicleGroup>) => {
    try {
      return await api.put<ApiResponse<VehicleGroup>>(`/api/vehicle-groups/${id}`, data);
    } catch (error) {
      console.error(`更新车队 ${id} 失败:`, error);
      throw error;
    }
  },
  delete: async (id: number) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/vehicle-groups/${id}`);
    } catch (error) {
      console.error(`删除车队 ${id} 失败:`, error);
      throw error;
    }
  },
};

// ====== 统计数据 API ======
export const statisticsApi = {
  getVehicleStatistics: async () => {
    try {
      return await api.get<ApiResponse<{ total_vehicles: number; online_vehicles: number; offline_vehicles: number }>>(
        '/api/statistics/vehicles'
      );
    } catch (error) {
      console.error('获取车辆统计信息失败:', error);
      throw error;
    }
  },
  getDeviceStatistics: async () => {
    try {
      return await api.get<ApiResponse<{ total_devices: number; online_devices: number; offline_devices: number }>>(
        '/api/statistics/devices'
      );
    } catch (error) {
      console.error('获取设备统计信息失败:', error);
      throw error;
    }
  },
  getWeighingStatistics: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ total_weighing: number; total_weight: number; average_weight: number }>>(
        '/api/statistics/weighing',
        { params }
      );
    } catch (error) {
      console.error('获取称重数据统计信息失败:', error);
      throw error;
    }
  },
  getSafetyRanking: async () => {
    try {
      return await api.get<ApiResponse<unknown>>('/api/statistics/safety-ranking');
    } catch (error) {
      console.error('获取安全指数排行失败:', error);
      throw error;
    }
  },
};

// ====== 装卸节点 API ======
export const nodeApi = {
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      console.log('使用位置API获取装卸节点数据');
      return await api.get<ApiResponse<PaginationResponse<unknown>>>('/api/location/places', { params });
    } catch (error) {
      console.error('获取装卸节点列表失败:', error);
      throw error;
    }
  },
  getById: async (id: number) => {
    try {
      return await api.get<ApiResponse<Node>>(`/api/nodes/${id}`);
    } catch (error) {
      console.error(`获取装卸节点 ${id} 失败:`, error);
      throw error;
    }
  },
  create: async (data: Omit<Node, 'id'>) => {
    try {
      return await api.post<ApiResponse<Node>>('/api/nodes', data);
    } catch (error) {
      console.error('创建装卸节点失败:', error);
      throw error;
    }
  },
  update: async (id: number, data: Partial<Node>) => {
    try {
      return await api.put<ApiResponse<Node>>(`/api/nodes/${id}`, data);
    } catch (error) {
      console.error(`更新装卸节点 ${id} 失败:`, error);
      throw error;
    }
  },
  delete: async (id: number) => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/nodes/${id}`);
    } catch (error) {
      console.error(`删除装卸节点 ${id} 失败:`, error);
      throw error;
    }
  },
};

export default api;
