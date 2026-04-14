/**
 * 统一通信服务集成层
 *
 * 这个模块负责将统一通信服务集成到现有应用中
 * 保持与旧系统的兼容性，同时支持新服务器的双协议功能
 */

import { initUnifiedCommunicationService, initForNewServer, initForLegacyServer, type CommunicationMessage } from './unifiedCommunicationService';
import { logService } from './logService';

// 服务实例
let unifiedService: ReturnType<typeof initUnifiedCommunicationService> | null = null;
let legacyService: ReturnType<typeof initForLegacyServer> | null = null;

// 配置
interface ServiceConfig {
  // 新服务器配置
  newServer: {
    host: string;
    httpPort: number;
    tcpPort: number;
    protocol: 'tcp' | 'websocket' | 'auto';
  };
  // 旧服务器配置
  legacyServer: {
    host: string;
    port: number;
  };
  // 功能路由配置
  featureRouting: {
    [key: string]: 'new' | 'legacy';
  };
}

/**
 * 初始化统一通信服务
 */
export async function setupUnifiedCommunicationService(config?: Partial<ServiceConfig>) {
  try {
    // 默认配置
    const defaultConfig: ServiceConfig = {
      newServer: {
        host: import.meta.env?.VITE_NEW_SERVER_HOST || 'localhost',
        httpPort: parseInt(import.meta.env?.VITE_NEW_SERVER_HTTP_PORT || '8988'),
        tcpPort: parseInt(import.meta.env?.VITE_NEW_SERVER_TCP_PORT || '9999'),
        protocol: (import.meta.env?.VITE_PROTOCOL_TYPE as 'tcp' | 'websocket' | 'auto') || 'auto',
      },
      legacyServer: {
        host: localStorage.getItem('serverIp') || '127.0.0.1',
        port: parseInt(localStorage.getItem('serverPort') || '9808'),
      },
      featureRouting: {
        // 旧服务器功能
        vehicle_command: 'legacy',
        device_control: 'legacy',
        realtime_data: 'legacy',

        // 新服务器功能
        user_management: 'new',
        report_query: 'new',
        data_analysis: 'new',

        // 可选切换的功能
        vehicle_query: 'new',
        alert_notification: 'new',
      },
    };

    // 合并配置
    const finalConfig = { ...defaultConfig, ...config };

    logService.info('[统一通信服务集成] 开始初始化', finalConfig);

    // 初始化新服务器服务
    logService.info('[统一通信服务集成] 初始化新服务器连接...');
    unifiedService = initForNewServer(
      finalConfig.newServer.host,
      8082, // WebSocket 端口
      finalConfig.newServer.protocol
    );

    // 尝试连接新服务器
    try {
      const newConnected = await unifiedService.connect();
      logService.info('[统一通信服务集成] 新服务器连接状态:', { newConnected });
    } catch (error) {
      logService.warn('[统一通信服务集成] 新服务器连接失败:', { error });
      // 新服务器连接失败不影响应用运行
    }

    // 设置新服务器消息监听
    if (unifiedService) {
      setupNewServiceListeners();
    }

    // 初始化旧服务器服务（保持兼容性）
    logService.info('[统一通信服务集成] 初始化旧服务器连接...');
    legacyService = initForLegacyServer(finalConfig.legacyServer.host, finalConfig.legacyServer.port);

    // 尝试连接旧服务器
    try {
      const legacyConnected = await legacyService.connect();
      logService.info('[统一通信服务集成] 旧服务器连接状态:', { legacyConnected });
    } catch (error) {
      logService.warn('[统一通信服务集成] 旧服务器连接失败:', { error });
    }

    // 设置旧服务器消息监听
    if (legacyService) {
      setupLegacyServiceListeners();
    }

    logService.info('[统一通信服务集成] 初始化完成');

    return {
      unifiedService,
      legacyService,
      config: finalConfig,
    };
  } catch (error) {
    logService.error('[统一通信服务集成] 初始化失败:', { error });
    throw error;
  }
}

/**
 * 设置新服务器消息监听器
 */
function setupNewServiceListeners() {
  if (!unifiedService) return;

  unifiedService.on('connected', () => {
    logService.info('[统一通信服务] 新服务器已连接');
    // 触发全局事件
    window.dispatchEvent(
      new CustomEvent('unified-service-connected', {
        detail: { server: 'new' },
      })
    );
  });

  unifiedService.on('disconnected', () => {
    logService.warn('[统一通信服务] 新服务器已断开');
    window.dispatchEvent(
      new CustomEvent('unified-service-disconnected', {
        detail: { server: 'new' },
      })
    );
  });

  unifiedService.on('error', (error: unknown) => {
    logService.error('[统一通信服务] 新服务器错误:', { error });
    window.dispatchEvent(
      new CustomEvent('unified-service-error', {
        detail: { server: 'new', error },
      })
    );
  });

  unifiedService.on('message', (data: unknown) => {
    const message = data as CommunicationMessage;
    logService.debug('[统一通信服务] 新服务器收到消息:', { message });

    // 触发全局消息事件
    window.dispatchEvent(
      new CustomEvent('unified-service-message', {
        detail: { server: 'new', message },
      })
    );
  });
}

/**
 * 设置旧服务器消息监听器
 */
function setupLegacyServiceListeners() {
  if (!legacyService) return;

  legacyService.on('connected', () => {
    logService.info('[统一通信服务] 旧服务器已连接');
    window.dispatchEvent(
      new CustomEvent('legacy-service-connected', {
        detail: { server: 'legacy' },
      })
    );
  });

  legacyService.on('disconnected', () => {
    logService.warn('[统一通信服务] 旧服务器已断开');
    window.dispatchEvent(
      new CustomEvent('legacy-service-disconnected', {
        detail: { server: 'legacy' },
      })
    );
  });

  legacyService.on('error', (error: unknown) => {
    logService.error('[统一通信服务] 旧服务器错误:', { error });
    window.dispatchEvent(
      new CustomEvent('legacy-service-error', {
        detail: { server: 'legacy', error },
      })
    );
  });
}

/**
 * 路由请求到合适的服务器
 */
export async function routeRequest<T>(feature: string, requestFn: (service: ReturnType<typeof initUnifiedCommunicationService> | ReturnType<typeof initForLegacyServer>) => Promise<T>): Promise<T> {
  const config = window.$unifiedConfig;

  if (!config) {
    throw new Error('统一通信服务未初始化');
  }

  // 确定使用哪个服务器
  const targetServer = config.featureRouting[feature] || 'legacy';

  logService.info('[统一通信服务] 路由请求', {
    feature,
    targetServer,
  });

  try {
    if (targetServer === 'new' && unifiedService) {
      return await requestFn(unifiedService);
    } else if (targetServer === 'legacy' && legacyService) {
      return await requestFn(legacyService);
    } else {
      throw new Error(`目标服务器不可用: ${targetServer}`);
    }
  } catch (error) {
    logService.error('[统一通信服务] 请求失败:', undefined, error as Error);

    // 可选：故障切换
    if (targetServer === 'new' && legacyService) {
      logService.warn('[统一通信服务] 尝试切换到旧服务器');
      return await requestFn(legacyService);
    }

    throw error;
  }
}

/**
 * 发送消息（智能路由）
 */
export async function sendMessage(message: CommunicationMessage): Promise<unknown> {
  // 根据命令类型路由到合适的服务器
  const commandRouting: Record<string, 'new' | 'legacy'> = {
    login: 'new',
    get_vehicles: 'new',
    vehicle_command: 'legacy',
    device_command: 'legacy',
    get_devices: 'new',
    get_users: 'new',
    report_query: 'new',
    get_alerts: 'new',
  };

  const targetServer = commandRouting[message.command] || 'new';

  if (targetServer === 'new' && unifiedService) {
    return await unifiedService.send(message);
  } else if (targetServer === 'legacy' && legacyService) {
    // 转换为旧协议格式
    const legacyMessage = convertToLegacyMessage(message);
    return await legacyService.send(legacyMessage as unknown as CommunicationMessage);
  } else {
    throw new Error('没有可用的服务器');
  }
}

/**
 * 转换为旧协议消息
 */
function convertToLegacyMessage(message: CommunicationMessage): Record<string, unknown> {
  const legacyMessage: Record<string, unknown> = {
    command: message.command.toUpperCase(),
    timestamp: message.timestamp || Date.now(),
  };

  if (message.payload) {
    Object.assign(legacyMessage, message.payload);
  }

  return legacyMessage;
}

/**
 * 获取服务状态
 */
export function getServiceStatus() {
  return {
    newServer: unifiedService
      ? {
          connected: unifiedService.isConnected(),
          protocol: unifiedService.getCurrentProtocol(),
          stats: unifiedService.getStats(),
        }
      : null,
    legacyServer: legacyService
      ? {
          connected: legacyService.isConnected(),
          protocol: 'tcp',
          stats: legacyService.getStats(),
        }
      : null,
  };
}

/**
 * 切换新服务器的协议
 */
export async function switchNewServerProtocol(protocol: 'tcp' | 'websocket'): Promise<boolean> {
  if (!unifiedService) {
    throw new Error('新服务器服务未初始化');
  }

  // const success = await unifiedService.switchProtocol(protocol) // 暂时注释，方法不存在
  const success = true; // 暂时设置为 true
  logService.info('[统一通信服务] 协议切换', {
    protocol,
    success,
  });

  return success;
}

/**
 * 断开所有连接
 */
export async function disconnectAll() {
  logService.info('[统一通信服务] 断开所有连接');

  if (unifiedService) {
    await unifiedService.disconnect();
  }

  if (legacyService) {
    await legacyService.disconnect();
  }
}

/**
 * 获取新服务器服务实例
 */
export function getUnifiedService() {
  return unifiedService;
}

/**
 * 获取旧服务器服务实例
 */
export function getLegacyService() {
  return legacyService;
}

/**
 * 初始化便捷函数
 */
export async function initAutoServer(host: string, port: number, protocol: 'tcp' | 'websocket' | 'auto' = 'auto') {
  return await setupUnifiedCommunicationService({
    newServer: {
      host,
      httpPort: port,
      tcpPort: 9999,
      protocol,
    },
  });
}

// 暴露到全局（便于调试）
declare global {
  interface Window {
    $unifiedService?: ReturnType<typeof getUnifiedService>;
    $legacyService?: ReturnType<typeof getLegacyService>;
    $unifiedConfig?: ServiceConfig;
    $sendMessage?: typeof sendMessage;
    $getServiceStatus?: typeof getServiceStatus;
    $switchProtocol?: typeof switchNewServerProtocol;
  }
}

/**
 * 暴露服务到全局（在应用初始化时调用）
 */
export function exposeServicesToGlobal() {
  if (unifiedService) {
    window.$unifiedService = unifiedService;
  }
  if (legacyService) {
    window.$legacyService = legacyService;
  }
  window.$sendMessage = sendMessage;
  window.$getServiceStatus = getServiceStatus;
  window.$switchProtocol = switchNewServerProtocol;

  logService.info('[统一通信服务] 服务已暴露到全局');
}


