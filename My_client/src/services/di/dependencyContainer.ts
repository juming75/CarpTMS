/**
 * 依赖注入容器
 * 管理应用中的所有服务实例，支持单例和工厂模式
 */
import { logService, LogService } from '../logService';
// import { degradationService, DegradationService } from '../degradationService'
import { ProtocolManager, protocolManager, createProtocolManager, ProtocolManagerConfig } from '../protocol/protocolManager';

/**
 * 服务类型标识
 */
export enum ServiceType {
  LOG_SERVICE = 'logService',
  DEGRADATION_SERVICE = 'degradationService',
  PROTOCOL_MANAGER = 'protocolManager',
  TCP_CLIENT = 'tcpClient',
  API_COMPATIBILITY_LAYER = 'apiCompatibilityLayer',
}

/**
 * 服务生命周期类型
 */
export enum ServiceLifetime {
  SINGLETON, // 单例：整个应用生命周期内只有一个实例
  TRANSIENT, // 瞬态：每次请求都创建新实例
  SCOPED, // 作用域：在特定作用域内共享实例
}

/**
 * 服务描述符
 */
export interface ServiceDescriptor {
  type: ServiceType;
  lifetime: ServiceLifetime;
  factory?: (...args: unknown[]) => unknown;
  instance?: unknown;
  dependencies?: ServiceType[];
}

/**
 * 服务配置接口
 */
export interface ServiceConfig {
  protocolManager?: ProtocolManagerConfig;
  [key: string]: unknown;
}

/**
 * 依赖注入容器类
 */
export class DependencyContainer {
  private services: Map<ServiceType, ServiceDescriptor> = new Map();
  private config: ServiceConfig = {};

  constructor(config?: ServiceConfig) {
    if (config) {
      this.config = config;
    }

    logService.info('[依赖注入容器] 初始化', { config: this.config });

    // 注册核心服务
    this.registerCoreServices();
  }

  /**
   * 注册核心服务
   */
  private registerCoreServices(): void {
    // 注册日志服务（单例）
    this.register(ServiceType.LOG_SERVICE, ServiceLifetime.SINGLETON, () => logService);

    // 注册降级服务（单例）
    // this.register(ServiceType.DEGRADATION_SERVICE, ServiceLifetime.SINGLETON, () => degradationService)

    // 注册协议管理器（单例，带配置）
    this.register(
      ServiceType.PROTOCOL_MANAGER,
      ServiceLifetime.SINGLETON,
      () => {
        const config = this.config.protocolManager;
        return config ? createProtocolManager(config) : protocolManager;
      },
      [ServiceType.LOG_SERVICE]
    );

    logService.info('[依赖注入容器] 核心服务已注册');
  }

  /**
   * 注册服务
   */
  register(type: ServiceType, lifetime: ServiceLifetime, factory: () => unknown, dependencies?: ServiceType[]): void {
    const descriptor: ServiceDescriptor = {
      type,
      lifetime,
      factory,
      dependencies,
    };

    this.services.set(type, descriptor);

    logService.debug('[依赖注入容器] 注册服务', {
      type,
      lifetime,
      hasDependencies: !!dependencies?.length,
    });
  }

  /**
   * 解析服务
   */
  resolve<T = unknown>(type: ServiceType): T {
    const descriptor = this.services.get(type);

    if (!descriptor) {
      throw new Error(`服务未注册: ${type}`);
    }

    logService.debug('[依赖注入容器] 解析服务', { type, lifetime: descriptor.lifetime });

    // 单例模式：返回现有实例
    if (descriptor.lifetime === ServiceLifetime.SINGLETON && descriptor.instance) {
      return descriptor.instance as T;
    }

    // 解析依赖
    const resolvedDependencies = this.resolveDependencies(descriptor.dependencies);

    // 创建新实例
    const instance = descriptor.factory!(...resolvedDependencies);

    // 单例模式：缓存实例
    if (descriptor.lifetime === ServiceLifetime.SINGLETON) {
      descriptor.instance = instance;
    }

    return instance as T;
  }

  /**
   * 解析依赖
   */
  private resolveDependencies(dependencies?: ServiceType[]): unknown[] {
    if (!dependencies || dependencies.length === 0) {
      return [];
    }

    return dependencies.map((dep) => this.resolve(dep));
  }

  /**
   * 检查服务是否已注册
   */
  isRegistered(type: ServiceType): boolean {
    return this.services.has(type);
  }

  /**
   * 注销服务
   */
  unregister(type: ServiceType): void {
    if (this.services.delete(type)) {
      logService.info('[依赖注入容器] 注销服务', { type });
    }
  }

  /**
   * 清空所有服务
   */
  clear(): void {
    const count = this.services.size;
    this.services.clear();
    logService.info('[依赖注入容器] 清空所有服务', { count });
  }

  /**
   * 获取所有已注册的服务
   */
  getRegisteredServices(): ServiceType[] {
    return Array.from(this.services.keys());
  }

  /**
   * 获取容器统计信息
   */
  getStats(): {
    totalServices: number;
    registeredServices: ServiceType[];
    config: ServiceConfig;
  } {
    return {
      totalServices: this.services.size,
      registeredServices: this.getRegisteredServices(),
      config: this.config,
    };
  }

  /**
   * 快捷方法：获取日志服务
   */
  getLogService(): LogService {
    return this.resolve<LogService>(ServiceType.LOG_SERVICE);
  }

  /**
   * 获取降级服务实例
   */
  // getDegradationService(): DegradationService {
  //   return this.resolve<DegradationService>(ServiceType.DEGRADATION_SERVICE)
  // }

  /**
   * 快捷方法：获取协议管理器
   */
  getProtocolManager(): ProtocolManager {
    return this.resolve<ProtocolManager>(ServiceType.PROTOCOL_MANAGER);
  }
}

/**
 * 创建依赖注入容器实例
 */
export function createDependencyContainer(config?: ServiceConfig): DependencyContainer {
  return new DependencyContainer(config);
}

/**
 * 导出全局容器实例
 */
export const dependencyContainer = new DependencyContainer();


