/**
 * 旧协议适配器
 * 专门处理与 .NET 3.5 旧服务器的通信协议
 */
import { logService } from '../logService';

// 旧协议命令类型
export type LegacyCommand =
  | 'Apply_Login' // 登录请求
  | 'Answer_Login' // 登录响应
  | 'Vehicle_Create' // 创建车辆
  | 'Vehicle_Request' // 查询车辆
  | 'Weighing_Request'; // 称重数据请求

// 旧协议请求格式
export interface LegacyRequest<T = unknown> {
  command: LegacyCommand;
  data?: T;
  method?: string; // HTTP方法，用于某些命令
  vehicle_id?: number; // 车辆ID
  [key: string]: unknown;
}

// 旧协议响应格式
export interface LegacyResponse<T = unknown> {
  command?: LegacyCommand;
  data: T;
  [key: string]: unknown;
}

// 登录请求数据
export interface LoginRequestData {
  username: string;
  password: string;
}

// 登录响应数据
export interface LoginResponseData {
  flag: number; // 1=成功, 0=失败
  user_id?: string;
  error?: string;
}

// 车辆请求数据
export interface VehicleRequestData {
  vehicle_name?: string;
  device_id?: string;
  own_no?: string;
  own_name?: string;
  own_phone?: string;
  group_id?: number;
  is_simulation?: boolean;
  simulation_source?: string;
  [key: string]: unknown;
}

// 车辆响应数据
export interface VehicleResponseData {
  vehicle?: Record<string, unknown>;
  vehicles?: Record<string, unknown>[];
  error?: string;
}

// 称重请求参数
export interface WeighingRequestParams {
  vehicle_id?: number;
  start_time?: string;
  end_time?: string;
  [key: string]: unknown;
}

// 称重响应数据
export interface WeighingResponseData {
  items?: Record<string, unknown>[];
  total?: number;
  error?: string;
}

/**
 * 旧协议适配器类
 */
export class LegacyProtocolAdapter {
  private protocolVersion = 'old';
  private readonly commandMap: Record<string, { action: string; resource: string }>;
  private readonly fieldMap: Record<string, string>;

  constructor() {
    logService.info('[旧协议适配器] 初始化', { protocolVersion: this.protocolVersion });

    // 命令映射表
    this.commandMap = {
      Apply_Login: { action: 'login', resource: 'auth' },
      Vehicle_Create: { action: 'create', resource: 'vehicle' },
      Vehicle_Request: { action: 'get', resource: 'vehicle' },
      Weighing_Request: { action: 'get', resource: 'weighing' },
    };

    // 字段映射表（旧字段 -> 新字段）
    this.fieldMap = {
      user_id: 'userId',
      vehicle_id: 'vehicleId',
      device_id: 'deviceId',
      create_time: 'createdAt',
      update_time: 'updatedAt',
      group_id: 'groupId',
      own_no: 'ownNo',
      own_name: 'ownName',
      own_phone: 'ownPhone',
    };
  }

  /**
   * 构建登录请求
   */
  buildLoginRequest(username: string, password: string): LegacyRequest<LoginRequestData> {
    const request: LegacyRequest<LoginRequestData> = {
      command: 'Apply_Login',
      data: {
        username,
        password,
      },
    };

    logService.debug('[旧协议适配器] 构建登录请求', {
      username,
      command: request.command,
    });

    return request;
  }

  /**
   * 解析登录响应
   */
  parseLoginResponse(response: LegacyResponse<LoginResponseData>): { success: boolean; userId?: string; error?: string } {
    const data = response.data;

    logService.debug('[旧协议适配器] 解析登录响应', { flag: data.flag });

    if (data.flag === 1) {
      return {
        success: true,
        userId: data.user_id,
      };
    } else {
      return {
        success: false,
        error: data.error || '登录失败',
      };
    }
  }

  /**
   * 构建车辆创建请求
   */
  buildVehicleCreateRequest(vehicleData: VehicleRequestData): LegacyRequest<VehicleRequestData> {
    const request: LegacyRequest<VehicleRequestData> = {
      command: 'Vehicle_Create',
      data: {
        ...vehicleData,
        vehicle_id: Date.now(),
        create_time: new Date().toISOString(),
        is_simulation: vehicleData.is_simulation || false,
        simulation_source: vehicleData.simulation_source || 'legacy_adapter',
      },
    };

    logService.debug('[旧协议适配器] 构建车辆创建请求', {
      vehicleName: vehicleData.vehicle_name,
      command: request.command,
    });

    return request;
  }

  /**
   * 构建车辆查询请求
   */
  buildVehicleRequest(vehicleId?: number): LegacyRequest<Record<string, unknown>> {
    const request: LegacyRequest<Record<string, unknown>> = {
      command: 'Vehicle_Request',
      method: 'GET',
      vehicle_id: vehicleId,
    };

    logService.debug('[旧协议适配器] 构建车辆查询请求', {
      vehicleId,
      command: request.command,
    });

    return request;
  }

  /**
   * 解析车辆响应
   */
  parseVehicleResponse(response: LegacyResponse<VehicleResponseData>, isList = false): VehicleResponseData {
    logService.debug('[旧协议适配器] 解析车辆响应', { isList });

    return response.data;
  }

  /**
   * 构建称重数据请求
   */
  buildWeighingRequest(params?: WeighingRequestParams): LegacyRequest<WeighingRequestParams> {
    const request: LegacyRequest<WeighingRequestParams> = {
      command: 'Weighing_Request',
      method: 'GET',
      params,
    };

    logService.debug('[旧协议适配器] 构建称重数据请求', {
      command: request.command,
    });

    return request;
  }

  /**
   * 解析称重数据响应
   */
  parseWeighingResponse(response: LegacyResponse<WeighingResponseData>): WeighingResponseData {
    logService.debug('[旧协议适配器] 解析称重数据响应');

    return response.data;
  }

  /**
   * 检测响应是否为旧协议格式
   */
  isLegacyResponse(response: unknown): boolean {
    return typeof response === 'object' && response !== null && (
      'command' in response ||
      ('data' in response && typeof (response as { data: unknown }).data === 'object' && (response as { data: unknown }).data !== null && (
        'flag' in (response as { data: { flag?: number } }).data ||
        'vehicle' in (response as { data: { vehicle?: unknown } }).data
      ))
    );
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
   * 获取命令映射
   */
  getCommandMap(): Record<string, { action: string; resource: string }> {
    return this.commandMap;
  }

  /**
   * 获取字段映射
   */
  getFieldMap(): Record<string, string> {
    return this.fieldMap;
  }

  /**
   * 序列化请求为 JSON 字符串
   */
  serializeRequest(request: LegacyRequest): string {
    return JSON.stringify(request);
  }

  /**
   * 反序列化响应 JSON 字符串
   */
  deserializeResponse(jsonString: string): LegacyResponse {
    try {
      return JSON.parse(jsonString);
    } catch (error) {
      logService.error('[旧协议适配器] 反序列化响应失败', { error, jsonString });
      throw new Error('响应数据格式错误');
    }
  }

  /**
   * 验证请求格式
   */
  validateRequest(request: LegacyRequest): boolean {
    if (!request.command) {
      logService.error('[旧协议适配器] 请求缺少 command 字段', { request });
      return false;
    }

    const validCommands: LegacyCommand[] = [
      'Apply_Login',
      'Answer_Login',
      'Vehicle_Create',
      'Vehicle_Request',
      'Weighing_Request',
    ];
    if (!validCommands.includes(request.command)) {
      logService.error('[旧协议适配器] 无效的 command 字段', { command: request.command });
      return false;
    }

    return true;
  }

  /**
   * 验证响应格式
   */
  validateResponse(response: LegacyResponse): boolean {
    if (!response.data) {
      logService.error('[旧协议适配器] 响应缺少 data 字段', { response });
      return false;
    }

    return true;
  }

  /**
   * 构建错误响应
   */
  buildErrorResponse(command: LegacyCommand, errorMessage: string): LegacyResponse {
    return {
      command,
      data: {
        flag: 0,
        error: errorMessage,
      },
    };
  }

  /**
   * 提取请求类型
   */
  extractRequestType(request: LegacyRequest): string {
    return this.commandMap[request.command]?.action || 'unknown';
  }

  /**
   * 提取资源类型
   */
  extractResourceType(request: LegacyRequest): string {
    return this.commandMap[request.command]?.resource || 'unknown';
  }
}

/**
 * 创建旧协议适配器实例
 */
export function createLegacyProtocolAdapter(): LegacyProtocolAdapter {
  return new LegacyProtocolAdapter();
}

/**
 * 导出单例实例
 */
export const legacyProtocolAdapter = new LegacyProtocolAdapter();


