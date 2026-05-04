import { ElMessage } from 'element-plus';
import axios from 'axios';
import type { AxiosResponse } from 'axios';
import type { ApiResponse, PaginationResponse, Vehicle, WeighingData, Order, Driver, Department, Organization, VehicleGroup, Alarm, Node, SystemSettings, ServiceStatus } from '@/types';
import { apiCache } from '@/services/apiCache';

// 扩展Axios配置
declare module 'axios' {
  interface AxiosRequestConfig {
    retry?: number;
    retryDelay?: number;
    retryableStatusCodes?: number[];
    retryCount?: number;
  }
}

export const checkBackendHealth = async (): Promise<boolean> => {
  try {
    const response = await api.get('/api/health', {
      timeout: 3000,
    });
    return response.status === 200;
  } catch (error) {
    console.error('后端服务健康检查失败:', error);
    return false;
  }
};

// API 基础配置 - 完全使用HttpOnly Cookie
const api = axios.create({
  baseURL: '',
  timeout: 15000,
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true, // 允许携带凭证（HttpOnly Cookie）
});

// 请求拦截器
api.interceptors.request.use(
  (config) => {
    // 完全依赖HttpOnly Cookie，不手动添加Authorization header
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

api.interceptors.response.use(
  <T>(response: AxiosResponse<T>): T => {
    if (response.config && response.config.responseType === 'blob') {
      return response as any;
    }

    const data = response.data as any;

    if (data) {
      if (data.code !== undefined && data.data !== undefined) {
        if (data.code !== 200 && data.code !== 201) {
          const msg = data.message || '请求失败';
          ElMessage.error(msg);
          return Promise.reject(new Error(msg)) as any;
        }
        if (
          response.config &&
          typeof response.config.method === 'string' &&
          response.config.method.toUpperCase() === 'GET' &&
          response.config.responseType !== 'blob'
        ) {
          apiCache.set(response.config, response.data);
        }
        return data.data as T;
      }
      if (data.data !== undefined) {
        if (
          response.config &&
          typeof response.config.method === 'string' &&
          response.config.method.toUpperCase() === 'GET' &&
          response.config.responseType !== 'blob'
        ) {
          apiCache.set(response.config, response.data);
        }
        return data.data as T;
      }
      if (Array.isArray(data)) {
        return { list: data } as T;
      }
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

    const config = error.config;

    // 处理401错误：Token过期或无效
    if (error.response?.status === 401) {
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      localStorage.removeItem('userInfo');
      localStorage.removeItem('user');
      localStorage.removeItem('userId');
      try {
        const { useAuthStore } = await import('../stores/useAuthStore');
        const authStore = useAuthStore();
        authStore.clear_auth();
      } catch {
        // store可能未初始化
      }
      window.location.href = '/login';
      return showError('登录已过期，请重新登录');
    }

    // 处理其他HTTP状态码
    if (error.response) {
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
      return showError(`请求失败: ${error.message}`);
    }
  }
);

// ====== 车辆管理 API ======
export const vehicleApi = {
  getAll: async (params?: Record<string, string | number | boolean>) => {
    try {
      return await api.get<ApiResponse<{ items: Vehicle[]; total: number; page: number; page_size: number }>>(
        '/api/vehicles',
        { params }
      );
    } catch (error) {
      console.error('获取车辆列表失败:', error);
      throw error;
    }
  },
  getById: async (id: number) => {
    try {
      return await api.get<ApiResponse<Vehicle>>(`/api/vehicles/${id}`);
    } catch (error) {
      console.error(`获取车辆 ${id} 失败:`, error);
      throw error;
    }
  },
  create: async (data: Omit<Vehicle, 'vehicle_id'>) => {
    try {
      return await api.post<ApiResponse<Vehicle>>('/api/vehicles', data);
    } catch (error) {
      console.error('创建车辆失败:', error);
      throw error;
    }
  },
  update: async (id: number, data: Partial<Vehicle>) => {
    try {
      return await api.put<ApiResponse<Vehicle>>(`/api/vehicles/${id}`, data);
    } catch (error) {
      console.error(`更新车辆 ${id} 失败:`, error);
      throw error;
    }
  },
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
    try {
      const response = await api.post<ApiResponse<unknown>>('/api/auth/login', { username, password });
      return response;
    } catch (error) {
      console.error('登录错误:', error);
      throw error;
    }
  },
  logout: async () => api.post('/api/auth/logout', {}),
  getCurrentUser: async () => api.get<ApiResponse<unknown>>('/api/auth/user'),
  changePassword: async (oldPassword: string, newPassword: string) => {
    try {
      const response = await api.post<ApiResponse<{ message: string }>>('/api/auth/change-password', {
        old_password: oldPassword,
        new_password: newPassword,
      });
      return response;
    } catch (error) {
      console.error('密码修改错误:', error);
      throw error;
    }
  },
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
      console.error('检查服务状态失败', error);
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
      console.error(`更新组织单位 ${id} 状态失败`, error);
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
      return await api.get<ApiResponse<PaginationResponse<unknown>>>('/api/nodes', { params });
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

// ====== 统一调度 API ======
export const getDispatchDevices = async () => {
  try {
    return await api.get<ApiResponse<any[]>>('/api/dispatch/devices');
  } catch (error) {
    console.error('获取调度设备列表失败:', error);
    throw error;
  }
};

export const getDispatchGroups = async () => {
  try {
    return await api.get<ApiResponse<any[]>>('/api/dispatch/groups');
  } catch (error) {
    console.error('获取调度组列表失败', error);
    throw error;
  }
};

export const sendDispatchCommand = async (data: {
  command_type: string;
  target_devices: number[];
  target_type: string;
  parameters: Record<string, any>;
}) => {
  try {
    return await api.post<ApiResponse<any>>('/api/dispatch/commands', data);
  } catch (error) {
    console.error('发送调度指令失败', error);
    throw error;
  }
};

export const getCommandStatus = async (commandId: string) => {
  try {
    return await api.get<ApiResponse<any>>(`/api/dispatch/commands/${commandId}`);
  } catch (error) {
    console.error('获取指令状态失败', error);
    throw error;
  }
};

export default api;
