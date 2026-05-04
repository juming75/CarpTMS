import { api } from '@/services/apiClient';
import type { 
  ApiResponse, 
  Vehicle, 
  WeighingData, 
  Order, 
  Driver, 
  Department, 
  Organization, 
  VehicleGroup, 
  Alarm, 
  Node, 
  SystemSettings, 
  ServiceStatus 
} from '@/types';

export interface PaginatedApiResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface AlarmStats {
  total: number;
  unprocessed: number;
  high_level: number;
}

export interface AlarmTrendItem {
  time: string;
  count: number;
}

export interface AlarmTypeStats {
  type: string;
  count: number;
  percentage: number;
}

export interface QuickProcessItem {
  id: number;
  name: string;
  action: string;
}

export interface VehicleStats {
  total_vehicles: number;
  online_vehicles: number;
  offline_vehicles: number;
}

export interface DeviceStats {
  total_devices: number;
  online_devices: number;
  offline_devices: number;
}

export interface WeighingStats {
  total_weighing: number;
  total_weight: number;
  average_weight: number;
}

export interface AuthChangePasswordRequest {
  old_password: string;
  new_password: string;
}

export interface AlarmProcessRequest {
  processing_result: string;
  processing_user_id: number;
}

export interface DispatchCommand {
  command_type: string;
  target_devices: number[];
  target_type: string;
  parameters: Record<string, unknown>;
}

export const checkBackendHealth = async (): Promise<boolean> => {
  try {
    const response = await api.get('/api/health', {
      timeout: 3000,
    });
    return response.status === 200;
  } catch (error) {
    console.error('后端服务健康检查失败', error);
    return false;
  }
};

export const vehicleApi = {
  getAll: async (params?: Record<string, string | number | boolean>): Promise<PaginatedApiResponse<Vehicle>> => {
    try {
      return await api.get<PaginatedApiResponse<Vehicle>>(
        '/api/vehicles',
        { params }
      );
    } catch (error) {
      console.error('获取车辆列表失败:', error);
      throw error;
    }
  },

  getById: async (id: number): Promise<Vehicle> => {
    try {
      return await api.get<Vehicle>(`/api/vehicles/${id}`);
    } catch (error) {
      console.error(`获取车辆 ${id} 失败:`, error);
      throw error;
    }
  },

  create: async (data: Omit<Vehicle, 'vehicle_id'>): Promise<Vehicle> => {
    try {
      return await api.post<Vehicle>('/api/vehicles', data);
    } catch (error) {
      console.error('创建车辆失败:', error);
      throw error;
    }
  },

  update: async (id: number, data: Partial<Vehicle>): Promise<Vehicle> => {
    try {
      return await api.put<Vehicle>(`/api/vehicles/${id}`, data);
    } catch (error) {
      console.error(`更新车辆 ${id} 失败:`, error);
      throw error;
    }
  },

  delete: async (id: number): Promise<void> => {
    try {
      return await api.delete<void>(`/api/vehicles/${id}`);
    } catch (error) {
      console.error(`删除车辆 ${id} 失败:`, error);
      throw error;
    }
  },
};

export const weighingApi = {
  getAll: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<WeighingData[]>> => {
    return await api.get<ApiResponse<WeighingData[]>>('/api/weighing', { params });
  },
  create: async (data: Omit<WeighingData, 'id'>): Promise<ApiResponse<WeighingData>> => {
    return await api.post<ApiResponse<WeighingData>>('/api/weighing', data);
  },
  getHistory: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<WeighingData[]>> => {
    return await api.get<ApiResponse<WeighingData[]>>('/api/weighing/history', { params });
  },
};

export const reportApi = {
  getTemplates: async (): Promise<ApiResponse<unknown[]>> => 
    api.get<ApiResponse<unknown[]>>('/api/reports/templates'),
  
  getData: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<unknown>> =>
    api.get<ApiResponse<unknown>>('/api/reports/data', { params }),
  
  generate: async (data: Record<string, string | number | boolean | object>): Promise<ApiResponse<unknown>> =>
    api.post<ApiResponse<unknown>>('/api/reports/generate', data),
  
  export: async (params?: Record<string, string | number | boolean>) =>
    api.get('/api/reports/export', { params, responseType: 'blob' }),
};

export const syncApi = {
  upload: async (data: Record<string, string | number | boolean | object>): Promise<ApiResponse<unknown>> =>
    api.post<ApiResponse<unknown>>('/api/sync/upload', data),
  
  download: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<unknown>> =>
    api.get<ApiResponse<unknown>>('/api/sync/download', { params }),
  
  getStatus: async (): Promise<ApiResponse<unknown>> => 
    api.get<ApiResponse<unknown>>('/api/sync/status'),
};

export const orderApi = {
  getAll: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<PaginatedApiResponse<Order>>> => {
    try {
      return await api.get<ApiResponse<PaginatedApiResponse<Order>>>(
        '/api/orders',
        { params }
      );
    } catch (error) {
      console.error('获取订单列表失败:', error);
      throw error;
    }
  },

  getById: async (id: number): Promise<ApiResponse<Order>> => {
    try {
      return await api.get<ApiResponse<Order>>(`/api/orders/${id}`);
    } catch (error) {
      console.error(`获取订单 ${id} 失败:`, error);
      throw error;
    }
  },

  create: async (data: Omit<Order, 'order_id'>): Promise<ApiResponse<Order>> => {
    try {
      return await api.post<ApiResponse<Order>>('/api/orders', data);
    } catch (error) {
      console.error('创建订单失败:', error);
      throw error;
    }
  },

  update: async (id: number, data: Partial<Order>): Promise<ApiResponse<Order>> => {
    try {
      return await api.put<ApiResponse<Order>>(`/api/orders/${id}`, data);
    } catch (error) {
      console.error(`更新订单 ${id} 失败:`, error);
      throw error;
    }
  },

  delete: async (id: number): Promise<ApiResponse<void>> => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/orders/${id}`);
    } catch (error) {
      console.error(`删除订单 ${id} 失败:`, error);
      throw error;
    }
  },
};

export const alarmApi = {
  getStats: async (): Promise<ApiResponse<AlarmStats>> => {
    try {
      return await api.get<ApiResponse<AlarmStats>>('/api/alerts/stats');
    } catch (error) {
      console.error('获取报警统计数据失败:', error);
      throw error;
    }
  },

  getAll: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<PaginatedApiResponse<Alarm>>> => {
    try {
      return await api.get<ApiResponse<PaginatedApiResponse<Alarm>>>(
        '/api/alerts',
        { params }
      );
    } catch (error) {
      console.error('获取报警列表失败:', error);
      throw error;
    }
  },

  getQuickProcess: async (): Promise<ApiResponse<QuickProcessItem[]>> => {
    try {
      return await api.get<ApiResponse<QuickProcessItem[]>>('/api/alerts/quick-process');
    } catch (error) {
      console.error('获取快速处理项失败:', error);
      throw error;
    }
  },

  getTrend: async (): Promise<ApiResponse<AlarmTrendItem[]>> => {
    try {
      return await api.get<ApiResponse<AlarmTrendItem[]>>('/api/alerts/trend');
    } catch (error) {
      console.error('获取报警趋势失败:', error);
      throw error;
    }
  },

  getTypes: async (): Promise<ApiResponse<AlarmTypeStats[]>> => {
    try {
      return await api.get<ApiResponse<AlarmTypeStats[]>>('/api/alerts/types');
    } catch (error) {
      console.error('获取报警类型分布失败:', error);
      throw error;
    }
  },

  process: async (id: number, data: AlarmProcessRequest): Promise<ApiResponse<Alarm>> => {
    try {
      return await api.put<ApiResponse<Alarm>>(`/api/alerts/${id}/process`, data);
    } catch (error) {
      console.error(`处理报警 ${id} 失败:`, error);
      throw error;
    }
  },
};

export const authApi = {
  login: async (username: string, password: string): Promise<ApiResponse<unknown>> => {
    try {
      const response = await api.post<ApiResponse<unknown>>('/api/auth/login', { username, password });
      return response;
    } catch (error) {
      console.error('登录错误:', error);
      throw error;
    }
  },

  logout: async (): Promise<void> => {
    await api.post('/api/auth/logout', {});
  },

  getCurrentUser: async (): Promise<ApiResponse<unknown>> => 
    api.get<ApiResponse<unknown>>('/api/auth/user'),

  changePassword: async (oldPassword: string, newPassword: string): Promise<ApiResponse<{ message: string }>> => {
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

export const driverApi = {
  getAll: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<PaginatedApiResponse<Driver>>> => {
    try {
      return await api.get<ApiResponse<PaginatedApiResponse<Driver>>>(
        '/api/drivers',
        { params }
      );
    } catch (error) {
      console.error('获取司机列表失败:', error);
      throw error;
    }
  },

  getById: async (id: number): Promise<ApiResponse<Driver>> => {
    try {
      return await api.get<ApiResponse<Driver>>(`/api/drivers/${id}`);
    } catch (error) {
      console.error(`获取司机 ${id} 失败:`, error);
      throw error;
    }
  },

  create: async (data: Omit<Driver, 'driver_id'>): Promise<ApiResponse<Driver>> => {
    try {
      return await api.post<ApiResponse<Driver>>('/api/drivers', data);
    } catch (error) {
      console.error('创建司机失败:', error);
      throw error;
    }
  },

  update: async (id: number, data: Partial<Driver>): Promise<ApiResponse<Driver>> => {
    try {
      return await api.put<ApiResponse<Driver>>(`/api/drivers/${id}`, data);
    } catch (error) {
      console.error(`更新司机 ${id} 失败:`, error);
      throw error;
    }
  },

  delete: async (id: number): Promise<ApiResponse<void>> => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/drivers/${id}`);
    } catch (error) {
      console.error(`删除司机 ${id} 失败:`, error);
      throw error;
    }
  },
};

export const settingsApi = {
  getSettings: async (): Promise<ApiResponse<SystemSettings>> => {
    try {
      return await api.get<ApiResponse<SystemSettings>>('/api/settings');
    } catch (error) {
      console.error('获取系统设置失败:', error);
      throw error;
    }
  },

  updateSettings: async (data: Partial<SystemSettings>): Promise<ApiResponse<SystemSettings>> => {
    try {
      return await api.put<ApiResponse<SystemSettings>>('/api/settings', data);
    } catch (error) {
      console.error('更新系统设置失败:', error);
      throw error;
    }
  },

  getCommunicationSettings: async (): Promise<ApiResponse<SystemSettings['communication']>> => {
    try {
      return await api.get<ApiResponse<SystemSettings['communication']>>('/api/settings/communication');
    } catch (error) {
      console.error('获取通信设置失败:', error);
      throw error;
    }
  },

  updateCommunicationSettings: async (data: SystemSettings['communication']): Promise<ApiResponse<SystemSettings['communication']>> => {
    try {
      return await api.put<ApiResponse<SystemSettings['communication']>>('/api/settings/communication', data);
    } catch (error) {
      console.error('更新通信设置失败:', error);
      throw error;
    }
  },

  checkServiceStatus: async (): Promise<ApiResponse<ServiceStatus[]>> => {
    try {
      return await api.get<ApiResponse<ServiceStatus[]>>('/api/services/status');
    } catch (error) {
      console.error('检查服务状态失败', error);
      throw error;
    }
  },

  startService: async (serviceName: string): Promise<ApiResponse<ServiceStatus>> => {
    try {
      return await api.post<ApiResponse<ServiceStatus>>(`/api/services/${serviceName}/start`, {});
    } catch (error) {
      console.error(`启动服务 ${serviceName} 失败:`, error);
      throw error;
    }
  },

  stopService: async (serviceName: string): Promise<ApiResponse<ServiceStatus>> => {
    try {
      return await api.post<ApiResponse<ServiceStatus>>(`/api/services/${serviceName}/stop`, {});
    } catch (error) {
      console.error(`停止服务 ${serviceName} 失败:`, error);
      throw error;
    }
  },

  restartService: async (serviceName: string): Promise<ApiResponse<ServiceStatus>> => {
    try {
      return await api.post<ApiResponse<ServiceStatus>>(`/api/services/${serviceName}/restart`, {});
    } catch (error) {
      console.error(`重启服务 ${serviceName} 失败:`, error);
      throw error;
    }
  },
};

export const departmentApi = {
  getAll: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<PaginatedApiResponse<Department>>> => {
    try {
      return await api.get<ApiResponse<PaginatedApiResponse<Department>>>(
        '/api/departments',
        { params }
      );
    } catch (error) {
      console.error('获取部门列表失败:', error);
      throw error;
    }
  },

  getById: async (id: number): Promise<ApiResponse<Department>> => {
    try {
      return await api.get<ApiResponse<Department>>(`/api/departments/${id}`);
    } catch (error) {
      console.error(`获取部门 ${id} 失败:`, error);
      throw error;
    }
  },

  create: async (data: Omit<Department, 'department_id'>): Promise<ApiResponse<Department>> => {
    try {
      return await api.post<ApiResponse<Department>>('/api/departments', data);
    } catch (error) {
      console.error('创建部门失败:', error);
      throw error;
    }
  },

  update: async (id: number, data: Partial<Department>): Promise<ApiResponse<Department>> => {
    try {
      return await api.put<ApiResponse<Department>>(`/api/departments/${id}`, data);
    } catch (error) {
      console.error(`更新部门 ${id} 失败:`, error);
      throw error;
    }
  },

  delete: async (id: number): Promise<ApiResponse<void>> => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/departments/${id}`);
    } catch (error) {
      console.error(`删除部门 ${id} 失败:`, error);
      throw error;
    }
  },
};

export const organizationApi = {
  getAll: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<PaginatedApiResponse<Organization>>> => {
    try {
      return await api.get<ApiResponse<PaginatedApiResponse<Organization>>>(
        '/api/organizations',
        { params }
      );
    } catch (error) {
      console.error('获取组织单位列表失败:', error);
      throw error;
    }
  },

  getById: async (id: string): Promise<ApiResponse<Organization>> => {
    try {
      return await api.get<ApiResponse<Organization>>(`/api/organizations/${id}`);
    } catch (error) {
      console.error(`获取组织单位 ${id} 失败:`, error);
      throw error;
    }
  },

  create: async (data: Omit<Organization, 'organization_id'>): Promise<ApiResponse<Organization>> => {
    try {
      return await api.post<ApiResponse<Organization>>('/api/organizations', data);
    } catch (error) {
      console.error('创建组织单位失败:', error);
      throw error;
    }
  },

  update: async (id: string, data: Partial<Organization>): Promise<ApiResponse<Organization>> => {
    try {
      return await api.put<ApiResponse<Organization>>(`/api/organizations/${id}`, data);
    } catch (error) {
      console.error(`更新组织单位 ${id} 失败:`, error);
      throw error;
    }
  },

  delete: async (id: string): Promise<ApiResponse<void>> => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/organizations/${id}`);
    } catch (error) {
      console.error(`删除组织单位 ${id} 失败:`, error);
      throw error;
    }
  },

  updateStatus: async (id: string, status: string): Promise<ApiResponse<Organization>> => {
    try {
      return await api.put<ApiResponse<Organization>>(`/api/organizations/${id}/status`, { status });
    } catch (error) {
      console.error(`更新组织单位 ${id} 状态失败`, error);
      throw error;
    }
  },

  getSettings: async (organizationId: string, settingKey: string): Promise<ApiResponse<unknown>> => {
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
  ): Promise<ApiResponse<unknown>> => {
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

export const vehicleTeamApi = {
  getAll: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<PaginatedApiResponse<VehicleGroup>>> => {
    try {
      return await api.get<ApiResponse<PaginatedApiResponse<VehicleGroup>>>(
        '/api/vehicle-groups',
        { params }
      );
    } catch (error) {
      console.error('获取车队列表失败:', error);
      throw error;
    }
  },

  getById: async (id: number): Promise<ApiResponse<VehicleGroup>> => {
    try {
      return await api.get<ApiResponse<VehicleGroup>>(`/api/vehicle-groups/${id}`);
    } catch (error) {
      console.error(`获取车队 ${id} 失败:`, error);
      throw error;
    }
  },

  create: async (data: Omit<VehicleGroup, 'group_id'>): Promise<ApiResponse<VehicleGroup>> => {
    try {
      return await api.post<ApiResponse<VehicleGroup>>('/api/vehicle-groups', data);
    } catch (error) {
      console.error('创建车队失败:', error);
      throw error;
    }
  },

  update: async (id: number, data: Partial<VehicleGroup>): Promise<ApiResponse<VehicleGroup>> => {
    try {
      return await api.put<ApiResponse<VehicleGroup>>(`/api/vehicle-groups/${id}`, data);
    } catch (error) {
      console.error(`更新车队 ${id} 失败:`, error);
      throw error;
    }
  },

  delete: async (id: number): Promise<ApiResponse<void>> => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/vehicle-groups/${id}`);
    } catch (error) {
      console.error(`删除车队 ${id} 失败:`, error);
      throw error;
    }
  },
};

export const statisticsApi = {
  getVehicleStatistics: async (): Promise<ApiResponse<VehicleStats>> => {
    try {
      return await api.get<ApiResponse<VehicleStats>>('/api/statistics/vehicles');
    } catch (error) {
      console.error('获取车辆统计信息失败:', error);
      throw error;
    }
  },

  getDeviceStatistics: async (): Promise<ApiResponse<DeviceStats>> => {
    try {
      return await api.get<ApiResponse<DeviceStats>>('/api/statistics/devices');
    } catch (error) {
      console.error('获取设备统计信息失败:', error);
      throw error;
    }
  },

  getWeighingStatistics: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<WeighingStats>> => {
    try {
      return await api.get<ApiResponse<WeighingStats>>(
        '/api/statistics/weighing',
        { params }
      );
    } catch (error) {
      console.error('获取称重数据统计信息失败:', error);
      throw error;
    }
  },

  getSafetyRanking: async (): Promise<ApiResponse<unknown>> => {
    try {
      return await api.get<ApiResponse<unknown>>('/api/statistics/safety-ranking');
    } catch (error) {
      console.error('获取安全指数排行失败:', error);
      throw error;
    }
  },
};

export const nodeApi = {
  getAll: async (params?: Record<string, string | number | boolean>): Promise<ApiResponse<PaginatedApiResponse<Node>>> => {
    try {
      return await api.get<ApiResponse<PaginatedApiResponse<Node>>>('/api/nodes', { params });
    } catch (error) {
      console.error('获取装卸节点列表失败:', error);
      throw error;
    }
  },

  getById: async (id: number): Promise<ApiResponse<Node>> => {
    try {
      return await api.get<ApiResponse<Node>>(`/api/nodes/${id}`);
    } catch (error) {
      console.error(`获取装卸节点 ${id} 失败:`, error);
      throw error;
    }
  },

  create: async (data: Omit<Node, 'id'>): Promise<ApiResponse<Node>> => {
    try {
      return await api.post<ApiResponse<Node>>('/api/nodes', data);
    } catch (error) {
      console.error('创建装卸节点失败:', error);
      throw error;
    }
  },

  update: async (id: number, data: Partial<Node>): Promise<ApiResponse<Node>> => {
    try {
      return await api.put<ApiResponse<Node>>(`/api/nodes/${id}`, data);
    } catch (error) {
      console.error(`更新装卸节点 ${id} 失败:`, error);
      throw error;
    }
  },

  delete: async (id: number): Promise<ApiResponse<void>> => {
    try {
      return await api.delete<ApiResponse<void>>(`/api/nodes/${id}`);
    } catch (error) {
      console.error(`删除装卸节点 ${id} 失败:`, error);
      throw error;
    }
  },
};

export const dispatchApi = {
  getDevices: async (): Promise<ApiResponse<unknown[]>> => {
    try {
      return await api.get<ApiResponse<unknown[]>>('/api/dispatch/devices');
    } catch (error) {
      console.error('获取调度设备列表失败:', error);
      throw error;
    }
  },

  getGroups: async (): Promise<ApiResponse<unknown[]>> => {
    try {
      return await api.get<ApiResponse<unknown[]>>('/api/dispatch/groups');
    } catch (error) {
      console.error('获取调度组列表失败', error);
      throw error;
    }
  },

  sendCommand: async (data: DispatchCommand): Promise<ApiResponse<unknown>> => {
    try {
      return await api.post<ApiResponse<unknown>>('/api/dispatch/commands', data);
    } catch (error) {
      console.error('发送调度指令失败', error);
      throw error;
    }
  },

  getCommandStatus: async (commandId: string): Promise<ApiResponse<unknown>> => {
    try {
      return await api.get<ApiResponse<unknown>>(`/api/dispatch/commands/${commandId}`);
    } catch (error) {
      console.error('获取指令状态失败', error);
      throw error;
    }
  },
};

export const getDispatchDevices = async (): Promise<ApiResponse<unknown[]>> => {
  return dispatchApi.getDevices();
};

export const sendDispatchCommand = async (data: DispatchCommand): Promise<ApiResponse<unknown>> => {
  return dispatchApi.sendCommand(data);
};

export default api;
