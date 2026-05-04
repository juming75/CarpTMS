/**
 * 协议管理器
 * 统一管理新旧协议适配器，提供协议版本检测和自动切换功能
 */
import { logService } from '../logService';
import { LegacyProtocolAdapter } from './legacyProtocolAdapter';
import { NewProtocolAdapter } from './newProtocolAdapter';

/**
 * 协议版本类型
 */
export type ProtocolVersion = 'old' | 'new' | 'auto';

/**
 * 登录响应类型
 */
export interface LoginResponse {
  success: boolean;
  userId?: string;
  token?: string;
  error?: string;
}

/**
 * 车辆基础类型
 */
export interface Vehicle {
  id?: number;
  license_plate?: string;
  vehicle_name?: string;
  status?: number;
  [key: string]: unknown;
}

/**
 * 车辆响应数据
 */
export interface VehicleResponse {
  vehicle?: Vehicle;
  vehicles?: Vehicle[];
  error?: string;
}

/**
 * 称重数据响应
 */
export interface WeighingResponse {
  [key: string]: unknown;
  error?: string;
}

/**
 * 协议请求类型
 */
export type ProtocolRequest = Record<string, unknown>;

/**
 * 协议响应类型
 */
export type ProtocolResponse = Record<string, unknown>;

/**
 * 协议适配器接口
 */
export interface IProtocolAdapter {
  buildLoginRequest(username: string, password: string): ProtocolRequest;
  parseLoginResponse(response: ProtocolResponse): LoginResponse;
  buildVehicleCreateRequest(vehicleData: Vehicle): ProtocolRequest;
  buildVehicleRequest(vehicleId?: number): ProtocolRequest;
  parseVehicleResponse(response: ProtocolResponse, isList?: boolean): Record<string, unknown>;
  buildWeighingRequest(params?: Record<string, unknown>): ProtocolRequest;
  parseWeighingResponse(response: ProtocolResponse): WeighingResponse;
  getProtocolVersion(): string;
  serializeRequest(request: ProtocolRequest): string;
  deserializeResponse(jsonString: string): ProtocolResponse;
  validateRequest(request: ProtocolRequest): boolean;
  validateResponse(response: ProtocolResponse): boolean;
  isNewProtocolResponse?(response: ProtocolResponse): boolean;
  isLegacyResponse?(response: ProtocolResponse): boolean;
}

/**
 * 协议管理器配置
 */
export interface ProtocolManagerConfig {
  defaultProtocol?: ProtocolVersion;
  autoDetectProtocol?: boolean;
  fallbackProtocol?: ProtocolVersion;
  enableProtocolSwitching?: boolean;
  retryWithFallback?: boolean;
  maxRetries?: number;
}

/**
 * 协议管理器类
 */
export class ProtocolManager {
  private config: ProtocolManagerConfig;
  private legacyAdapter: LegacyProtocolAdapter;
  private newAdapter: NewProtocolAdapter;
  private currentProtocol: ProtocolVersion;
  private retryCount: number = 0;

  constructor(config?: Partial<ProtocolManagerConfig>) {
    this.config = {
      defaultProtocol: 'auto',
      autoDetectProtocol: true,
      fallbackProtocol: 'old',
      enableProtocolSwitching: true,
      retryWithFallback: true,
      maxRetries: 2,
      ...config,
    };

    // 初始化适配器
    this.legacyAdapter = new LegacyProtocolAdapter();
    this.newAdapter = new NewProtocolAdapter();

    // 设置当前协议
    this.currentProtocol = (this.config.defaultProtocol === 'auto' ? 'old' : this.config.defaultProtocol) as ProtocolVersion;

    logService.info('[协议管理器] 初始化', {
      config: this.config,
      currentProtocol: this.currentProtocol,
    });
  }

  /**
   * 获取当前适配器
   */
  private getAdapter(protocol?: ProtocolVersion): IProtocolAdapter {
    const targetProtocol = protocol || this.currentProtocol;
    return (targetProtocol === 'new' ? this.newAdapter : this.legacyAdapter) as unknown as IProtocolAdapter;
  }

  /**
   * 设置协议版本
   */
  setProtocol(version: ProtocolVersion): void {
    if (!this.config.enableProtocolSwitching) {
      logService.warn('[协议管理器] 协议切换已禁用', { currentProtocol: this.currentProtocol });
      return;
    }

    logService.info('[协议管理器] 切换协议版本', {
      oldProtocol: this.currentProtocol,
      newProtocol: version,
    });

    this.currentProtocol = version;
    this.retryCount = 0;

    // 保存到 localStorage
    localStorage.setItem('protocolVersion', version);
  }

  /**
   * 获取当前协议版本
   */
  getProtocol(): ProtocolVersion {
    return this.currentProtocol;
  }

  /**
   * 自动检测协议版本
   */
  autoDetectProtocol(response: ProtocolResponse): ProtocolVersion {
    if (!this.config.autoDetectProtocol) {
      return this.currentProtocol;
    }

    // 尝试新协议格式
    if (this.newAdapter.isNewProtocolResponse?.(response)) {
      if (this.currentProtocol !== 'new') {
        logService.info('[协议管理器] 自动检测到新协议格式，切换协议');
        this.setProtocol('new');
      }
      return 'new';
    }

    // 尝试旧协议格式
    if (this.legacyAdapter.isLegacyResponse(response)) {
      if (this.currentProtocol !== 'old') {
        logService.info('[协议管理器] 自动检测到旧协议格式，切换协议');
        this.setProtocol('old');
      }
      return 'old';
    }

    logService.warn('[协议管理器] 无法自动检测协议格式，使用当前协议', {
      currentProtocol: this.currentProtocol,
    });

    return this.currentProtocol;
  }

  /**
   * 构建登录请求
   */
  buildLoginRequest(username: string, password: string, protocol?: ProtocolVersion): string {
    const adapter = this.getAdapter(protocol);
    const request = adapter.buildLoginRequest(username, password);
    return adapter.serializeRequest(request);
  }

  /**
   * 解析登录响应
   */
  async parseLoginResponse(jsonString: string, protocol?: ProtocolVersion): Promise<LoginResponse> {
    try {
      const adapter = this.getAdapter(protocol);
      const response = adapter.deserializeResponse(jsonString);

      // 验证响应格式
      if (!adapter.validateResponse(response)) {
        throw new Error('响应格式验证失败');
      }

      // 解析登录响应
      const result = adapter.parseLoginResponse(response);

      // 自动检测协议版本
      if (protocol === 'auto') {
        const detectedProtocol = this.autoDetectProtocol(response);
        logService.debug('[协议管理器] 登录响应协议检测', {
          detectedProtocol,
          result,
        });
      }

      return result;
    } catch (error) {
      logService.error('[协议管理器] 解析登录响应失败', { error, jsonString });

      // 尝试回退协议
      if (this.config.retryWithFallback && this.retryCount < this.config.maxRetries!) {
        this.retryCount++;
        const fallbackProtocol = this.currentProtocol === 'new' ? 'old' : 'new';
        logService.info('[协议管理器] 尝试回退协议', {
          retryCount: this.retryCount,
          fallbackProtocol,
        });
        return this.parseLoginResponse(jsonString, fallbackProtocol);
      }

      throw error;
    }
  }

  /**
   * 构建车辆创建请求
   */
  buildVehicleCreateRequest(vehicleData: Vehicle, protocol?: ProtocolVersion): string {
    const adapter = this.getAdapter(protocol);
    const request = adapter.buildVehicleCreateRequest(vehicleData);
    return adapter.serializeRequest(request);
  }

  /**
   * 构建车辆查询请求
   */
  buildVehicleRequest(vehicleId?: number, protocol?: ProtocolVersion): string {
    const adapter = this.getAdapter(protocol);
    const request = adapter.buildVehicleRequest(vehicleId);
    return adapter.serializeRequest(request);
  }

  /**
   * 解析车辆响应
   */
  async parseVehicleResponse(
    jsonString: string,
    isList: boolean = false,
    protocol?: ProtocolVersion
  ): Promise<VehicleResponse> {
    try {
      const adapter = this.getAdapter(protocol);
      const response = adapter.deserializeResponse(jsonString);

      // 验证响应格式
      if (!adapter.validateResponse(response)) {
        throw new Error('响应格式验证失败');
      }

      // 解析车辆响应
      const result = adapter.parseVehicleResponse(response, isList);

      // 自动检测协议版本
      if (protocol === 'auto') {
        const detectedProtocol = this.autoDetectProtocol(response);
        logService.debug('[协议管理器] 车辆响应协议检测', {
          detectedProtocol,
          isList,
        });
      }

      return result;
    } catch (error) {
      logService.error('[协议管理器] 解析车辆响应失败', { error, jsonString });

      // 尝试回退协议
      if (this.config.retryWithFallback && this.retryCount < this.config.maxRetries!) {
        this.retryCount++;
        const fallbackProtocol = this.currentProtocol === 'new' ? 'old' : 'new';
        logService.info('[协议管理器] 尝试回退协议', {
          retryCount: this.retryCount,
          fallbackProtocol,
        });
        return this.parseVehicleResponse(jsonString, isList, fallbackProtocol);
      }

      throw error;
    }
  }

  /**
   * 构建称重数据请求
   */
  buildWeighingRequest(params?: Record<string, unknown>, protocol?: ProtocolVersion): string {
    const adapter = this.getAdapter(protocol);
    const request = adapter.buildWeighingRequest(params);
    return adapter.serializeRequest(request);
  }

  /**
   * 解析称重数据响应
   */
  async parseWeighingResponse(jsonString: string, protocol?: ProtocolVersion): Promise<WeighingResponse> {
    try {
      const adapter = this.getAdapter(protocol);
      const response = adapter.deserializeResponse(jsonString);

      // 验证响应格式
      if (!adapter.validateResponse(response)) {
        throw new Error('响应格式验证失败');
      }

      // 解析称重数据响应
      const result = adapter.parseWeighingResponse(response);

      // 自动检测协议版本
      if (protocol === 'auto') {
        const detectedProtocol = this.autoDetectProtocol(response);
        logService.debug('[协议管理器] 称重数据响应协议检测', {
          detectedProtocol,
        });
      }

      return result;
    } catch (error) {
      logService.error('[协议管理器] 解析称重数据响应失败', { error, jsonString });

      // 尝试回退协议
      if (this.config.retryWithFallback && this.retryCount < this.config.maxRetries!) {
        this.retryCount++;
        const fallbackProtocol = this.currentProtocol === 'new' ? 'old' : 'new';
        logService.info('[协议管理器] 尝试回退协议', {
          retryCount: this.retryCount,
          fallbackProtocol,
        });
        return this.parseWeighingResponse(jsonString, fallbackProtocol);
      }

      throw error;
    }
  }

  /**
   * 重置重试计数
   */
  resetRetryCount(): void {
    this.retryCount = 0;
  }

  /**
   * 获取配置
   */
  getConfig(): ProtocolManagerConfig {
    return { ...this.config };
  }

  /**
   * 更新配置
   */
  updateConfig(newConfig: Partial<ProtocolManagerConfig>): void {
    this.config = { ...this.config, ...newConfig };
    logService.info('[协议管理器] 配置已更新', { config: this.config });
  }

  /**
   * 获取统计信息
   */
  getStats(): {
    currentProtocol: ProtocolVersion;
    retryCount: number;
    config: ProtocolManagerConfig;
  } {
    return {
      currentProtocol: this.currentProtocol,
      retryCount: this.retryCount,
      config: this.config,
    };
  }

  /**
   * 转换蛇形命名到驼峰命名
   */
  convertSnakeToCamel(input: unknown): unknown {
    if (input === null || typeof input !== 'object') {
      return input;
    }

    if (Array.isArray(input)) {
      return input.map((item) => this.convertSnakeToCamel(item));
    }

    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(input as Record<string, unknown>)) {
      const camelKey = key.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
      result[camelKey] = this.convertSnakeToCamel(value);
    }
    return result;
  }

  /**
   * 转换驼峰命名到蛇形命名
   */
  convertCamelToSnake(input: unknown): unknown {
    if (input === null || typeof input !== 'object') {
      return input;
    }

    if (Array.isArray(input)) {
      return input.map((item) => this.convertCamelToSnake(item));
    }

    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(input as Record<string, unknown>)) {
      const snakeKey = key.replace(/([A-Z])/g, '_$1').toLowerCase();
      result[snakeKey] = this.convertCamelToSnake(value);
    }
    return result;
  }
}

/**
 * 创建协议管理器实例
 */
export function createProtocolManager(config?: Partial<ProtocolManagerConfig>): ProtocolManager {
  return new ProtocolManager(config);
}

/**
 * 导出单例实例
 */
export const protocolManager = new ProtocolManager();


