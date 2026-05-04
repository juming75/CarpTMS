/**
 * 新协议适配器
 * 处理与 Rust Actix 新服务器的通信协议
 */
import { logService } from '../logService';

// 新协议动作类型
export type NewAction = 'login' | 'create' | 'update' | 'delete' | 'get';

// 新协议资源类型
export type NewResource = 'auth' | 'vehicle' | 'weighing';

// 新协议请求格式
export interface NewRequest<T = unknown> {
  action: NewAction;
  params: T;
  resource: NewResource;
  resource_id?: number;
  method?: string;
  timestamp: number;
  version: string;
}

// 新协议响应格式
export interface NewResponse<T = unknown> {
  code: number;
  message: string;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: unknown;
  };
  timestamp?: number;
  version?: string;
}

// 登录请求参数
export interface LoginParams {
  username: string;
  password: string;
}

// 登录响应数据
export interface LoginResponseData {
  userId: string;
  token: string;
}

// 车辆数据
export interface VehicleData {
  vehicle_id?: number;
  license_plate: string;
  vehicle_type: string;
  status: number;
  [key: string]: unknown;
}

// 称重数据
export interface WeighingParams {
  vehicle_id?: number;
  start_time?: string;
  end_time?: string;
  [key: string]: unknown;
}

/**
 * 新协议适配器类
 */
export class NewProtocolAdapter {
  private protocolVersion = 'new';
  private readonly DEFAULT_VERSION = '1.0';

  constructor() {
    logService.info('[新协议适配器] 初始化', { protocolVersion: this.protocolVersion });
  }

  /**
   * 构建登录请求
   */
  buildLoginRequest(username: string, password: string): NewRequest<LoginParams> {
    const request: NewRequest<LoginParams> = {
      action: 'login',
      params: {
        username,
        password,
      },
      resource: 'auth',
      timestamp: Date.now(),
      version: this.DEFAULT_VERSION,
    };

    logService.debug('[新协议适配器] 构建登录请求', {
      username,
      action: request.action,
      resource: request.resource,
    });

    return request;
  }

  /**
   * 解析登录响应
   */
  parseLoginResponse(response: NewResponse<LoginResponseData>): { success: boolean; userId?: string; token?: string; error?: string } {
    logService.debug('[新协议适配器] 解析登录响应', { code: response.code });

    if (response.code === 200) {
      return {
        success: true,
        userId: response.data?.userId,
        token: response.data?.token,
      };
    } else {
      return {
        success: false,
        error: response.message || response.error?.message || '登录失败',
      };
    }
  }

  /**
   * 构建车辆创建请求
   */
  buildVehicleCreateRequest(vehicleData: VehicleData): NewRequest<Record<string, unknown>> {
    const request: NewRequest<Record<string, unknown>> = {
      action: 'create',
      params: this.convertToCamelCase(vehicleData) as Record<string, unknown>,
      resource: 'vehicle',
      resource_id: undefined,
      method: 'POST',
      timestamp: Date.now(),
      version: this.DEFAULT_VERSION,
    };

    logService.debug('[新协议适配器] 构建车辆创建请求', {
      action: request.action,
      resource: request.resource,
    });

    return request;
  }

  /**
   * 构建车辆查询请求
   */
  buildVehicleRequest(vehicleId?: number): NewRequest<Record<string, unknown>> {
    const request: NewRequest<Record<string, unknown>> = {
      action: 'get',
      params: {},
      resource: 'vehicle',
      resource_id: vehicleId,
      method: 'GET',
      timestamp: Date.now(),
      version: this.DEFAULT_VERSION,
    };

    logService.debug('[新协议适配器] 构建车辆查询请求', {
      action: request.action,
      resource: request.resource,
      resourceId: vehicleId,
    });

    return request;
  }

  /**
   * 解析车辆响应
   */
  parseVehicleResponse(response: NewResponse<unknown>, isList = false): Record<string, unknown> {
    logService.debug('[新协议适配器] 解析车辆响应', { code: response.code, isList });

    if (response.code === 200) {
      const data = this.convertToSnakeCase(response.data || {}) as Record<string, unknown>;

      if (isList) {
        return {
          vehicles: (data.items as unknown[]) || (data.vehicles as unknown[]) || [],
        };
      } else {
        return {
          vehicle: data,
        };
      }
    } else {
      return {
        error: response.message || response.error?.message || '获取车辆数据失败',
      };
    }
  }

  /**
   * 构建称重数据请求
   */
  buildWeighingRequest(params?: WeighingParams): NewRequest<Record<string, unknown>> {
    const request: NewRequest<Record<string, unknown>> = {
      action: 'get',
      params: this.convertToCamelCase(params || {}) as Record<string, unknown>,
      resource: 'weighing',
      method: 'GET',
      timestamp: Date.now(),
      version: this.DEFAULT_VERSION,
    };

    logService.debug('[新协议适配器] 构建称重数据请求', {
      action: request.action,
      resource: request.resource,
    });

    return request;
  }

  /**
   * 解析称重数据响应
   */
  parseWeighingResponse(response: NewResponse<unknown>): Record<string, unknown> {
    logService.debug('[新协议适配器] 解析称重数据响应', { code: response.code });

    if (response.code === 200) {
      return this.convertToSnakeCase(response.data || {}) as Record<string, unknown>;
    } else {
      return {
        error: response.message || response.error?.message || '获取称重数据失败',
      };
    }
  }

  /**
   * 检测响应是否为新协议格式
   */
  isNewProtocolResponse(response: unknown): boolean {
    return typeof response === 'object' && response !== null && 'code' in response && 'message' in response;
  }

  /**
   * 字段转换：蛇形命名 -> 驼峰命名
   */
  toCamelCase(str: string): string {
    return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
  }

  /**
   * 字段转换：驼峰命名 -> 蛇形命名
   */
  toSnakeCase(str: string): string {
    return str.replace(/([A-Z])/g, '_$1').toLowerCase();
  }

  /**
   * 对象转换：蛇形命名 -> 驼峰命名
   */
  convertToCamelCase(obj: unknown): unknown {
    if (obj === null || typeof obj !== 'object') {
      return obj;
    }

    if (Array.isArray(obj)) {
      return obj.map((item) => this.convertToCamelCase(item));
    }

    const result: Record<string, unknown> = {};
    for (const key in obj as object) {
      if (Object.prototype.hasOwnProperty.call(obj, key)) {
        const newKey = this.toCamelCase(key);
        result[newKey] = this.convertToCamelCase((obj as Record<string, unknown>)[key]);
      }
    }
    return result;
  }

  /**
   * 对象转换：驼峰命名 -> 蛇形命名
   */
  convertToSnakeCase(obj: unknown): unknown {
    if (obj === null || typeof obj !== 'object') {
      return obj;
    }

    if (Array.isArray(obj)) {
      return obj.map((item) => this.convertToSnakeCase(item));
    }

    const result: Record<string, unknown> = {};
    for (const key in obj as object) {
      if (Object.prototype.hasOwnProperty.call(obj, key)) {
        const newKey = this.toSnakeCase(key);
        result[newKey] = this.convertToSnakeCase((obj as Record<string, unknown>)[key]);
      }
    }
    return result;
  }

  /**
   * 获取协议版本
   */
  getProtocolVersion(): string {
    return this.protocolVersion;
  }

  /**
   * 序列化请求为 JSON 字符串
   */
  serializeRequest(request: NewRequest): string {
    return JSON.stringify(request);
  }

  /**
   * 反序列化响应 JSON 字符串
   */
  deserializeResponse(jsonString: string): NewResponse {
    try {
      return JSON.parse(jsonString);
    } catch (error) {
      logService.error('[新协议适配器] 反序列化响应失败', { error, jsonString });
      throw new Error('响应数据格式错误');
    }
  }

  /**
   * 验证请求格式
   */
  validateRequest(request: NewRequest): boolean {
    if (!request.action) {
      logService.error('[新协议适配器] 请求缺少 action 字段', { request });
      return false;
    }

    if (!request.resource) {
      logService.error('[新协议适配器] 请求缺少 resource 字段', { request });
      return false;
    }

    const validActions: NewAction[] = ['login', 'create', 'update', 'delete', 'get'];
    if (!validActions.includes(request.action)) {
      logService.error('[新协议适配器] 无效的 action 字段', { action: request.action });
      return false;
    }

    return true;
  }

  /**
   * 验证响应格式
   */
  validateResponse(response: NewResponse): boolean {
    if (response.code === undefined) {
      logService.error('[新协议适配器] 响应缺少 code 字段', { response });
      return false;
    }

    if (!response.message) {
      logService.error('[新协议适配器] 响应缺少 message 字段', { response });
      return false;
    }

    return true;
  }

  /**
   * 构建错误响应
   */
  buildErrorResponse(action: string, errorMessage: string, code: number = 500): NewResponse {
    return {
      code,
      message: errorMessage,
      error: {
        code: `${action.toUpperCase()}_ERROR`,
        message: errorMessage,
      },
      timestamp: Date.now(),
      version: this.DEFAULT_VERSION,
    };
  }

  /**
   * HTTP 方法映射到新协议 action
   */
  mapHttpMethodToAction(method: string): NewAction {
    const map: Record<string, NewAction> = {
      GET: 'get',
      POST: 'create',
      PUT: 'update',
      DELETE: 'delete',
    };
    return map[method] || 'get';
  }

  /**
   * 新协议 action 映射到 HTTP 方法
   */
  mapActionToHttpMethod(action: NewAction): string {
    const map: Record<NewAction, string> = {
      get: 'GET',
      create: 'POST',
      update: 'PUT',
      delete: 'DELETE',
      login: 'POST',
    };
    return map[action];
  }
}

/**
 * 创建新协议适配器实例
 */
export function createNewProtocolAdapter(): NewProtocolAdapter {
  return new NewProtocolAdapter();
}

/**
 * 导出单例实例
 */
export const newProtocolAdapter = new NewProtocolAdapter();


