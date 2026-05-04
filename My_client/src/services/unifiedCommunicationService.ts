/* global NodeJS */
/**
 * 统一通信服务
 * 自动选择TCP或WebSocket进行通信
 */

import { logService } from './logService';

// 协议类型
export type ProtocolType = 'tcp' | 'websocket' | 'auto';

// 连接状态
export enum ConnectionState {
  DISCONNECTED = 'disconnected',
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  RECONNECTING = 'reconnecting',
  ERROR = 'error',
}

// 通信消息
export interface CommunicationMessage {
  type: string;
  msg_type?: string;
  msg_id?: string;
  device_id?: string;
  command: string;
  timestamp?: number;
  payload: unknown;
}

// 通信配置
export interface CommunicationConfig {
  host: string;
  port: number;
  protocol: ProtocolType;
  wsPath?: string;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  heartbeatInterval?: number;
}

// WebSocket 客户端接口
interface WebSocketClientInterface {
  connect(): Promise<boolean>;
  disconnect(): Promise<void>;
  send(data: Record<string, unknown>, priority?: number): Promise<boolean>;
  isConnected(): boolean;
  on(event: string, callback: (data: unknown) => void): void;
  off(event: string, callback: (data: unknown) => void): void;
}

// 消息数据类型
interface MessageData {
  type: string;
  command?: string;
  [key: string]: unknown;
}

// 事件数据类型
interface EventData {
  protocol?: ProtocolType;
  error?: unknown;
  [key: string]: unknown;
}

/**
 * 统一通信服务
 */
export class UnifiedCommunicationService {
  private config: CommunicationConfig;
  private currentProtocol: ProtocolType;
  private wsClient: WebSocketClientInterface | null = null;
  private state: ConnectionState = ConnectionState.DISCONNECTED;
  private reconnectAttempts: number = 0;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  private messageHandlers: Map<string, ((data: unknown) => void)[]> = new Map();

  constructor(config: CommunicationConfig) {
    this.config = config;
    this.currentProtocol = 'websocket'; // 只使用WebSocket

    logService.info('[统一通信服务] 初始化', {
      config: this.config,
      currentProtocol: this.currentProtocol,
    });
  }

  /**
   * 连接到服务器
   */
  async connect(): Promise<boolean> {
    if (this.state === ConnectionState.CONNECTED) {
      logService.debug('[统一通信服务] 已连接，无需重复连接');
      return true;
    }

    this.state = ConnectionState.CONNECTING;
    // 重置重连计数器
    this.reconnectAttempts = 0;
    
    logService.info('[统一通信服务] 开始连接', {
      protocol: 'websocket',
      host: this.config.host,
      port: this.config.port,
    });

    try {
      // 只使用WebSocket通信
      const success = await this.connectWebSocket();
      if (success) {
        this.onConnected();
        return true;
      }

      throw new Error('WebSocket连接失败');
    } catch (error) {
      this.onError(error);
      return false;
    }
  }

  /**
   * 连接WebSocket
   */
  private async connectWebSocket(): Promise<boolean> {
    try {
      // 动态导入WebSocket客户端
      const { WebSocketService } = await import('./websocket');

      // 初始化WebSocket客户端
      // 开发环境直接连接后端WebSocket服务器，避免Vite代理问题
      // 生产环境使用配置的主机和端口
      let wsUrl: string;
      if (import.meta.env.DEV) {
        wsUrl = this.config.host && this.config.port
          ? `ws://${this.config.host}:${this.config.port}/ws`
          : 'ws://localhost:9808/ws';
      } else {
        const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
        wsUrl = `${protocol}://${this.config.host}:${this.config.port}/ws`;
      }
      this.wsClient = new WebSocketService(wsUrl) as unknown as WebSocketClientInterface;

      // 设置事件处理器
      this.wsClient.on('connect', () => {
        logService.info('[统一通信服务] WebSocket连接成功');
        this.onConnected();
      });

      this.wsClient.on('message', (message: unknown) => {
        this.handleMessage(message as MessageData);
      });

      this.wsClient.on('error', (error: unknown) => {
        logService.error('[统一通信服务] WebSocket错误', { error });
        this.onError(error);
      });

      this.wsClient.on('disconnect', () => {
        logService.warn('[统一通信服务] WebSocket断开连接');
        this.onDisconnected();
      });

      // 连接
      const connected = await this.wsClient.connect();

      return connected;
    } catch (error) {
      logService.error('[统一通信服务] WebSocket连接异常', { error });
      return false;
    }
  }

  /**
   * 断开连接
   */
  async disconnect(): Promise<void> {
    logService.info('[统一通信服务] 断开连接', { protocol: this.currentProtocol });

    // 停止心跳
    this.stopHeartbeat();

    // 断开WebSocket连接
    if (this.wsClient) {
      await this.wsClient.disconnect();
    }

    this.state = ConnectionState.DISCONNECTED;
  }

  /**
   * 发送消息
   */
  async send(message: CommunicationMessage): Promise<unknown> {
    logService.debug('[统一通信服务] 发送消息', {
      type: message.type,
      command: message.command,
      protocol: this.currentProtocol,
    });

    // 确保已连接
    if (!this.isConnected()) {
      logService.warn('[统一通信服务] 未连接，尝试重连');
      const connected = await this.connect();
      if (!connected) {
        throw new Error('无法连接到服务器');
      }
    }

    try {
      // 发送WebSocket消息
      if (this.wsClient) {
        return await this.wsClient.send(message as unknown as Record<string, unknown>);
      }

      throw new Error('WebSocket客户端未初始化');
    } catch (error) {
      logService.error('[统一通信服务] 发送消息失败', { error });
      throw error;
    }
  }

  /**
   * 处理接收到的消息
   */
  private handleMessage(message: MessageData): void {
    logService.debug('[统一通信服务] 收到消息', {
      type: message.type,
      command: message.command,
    });

    // 触发消息处理器
    const handlers = this.messageHandlers.get(message.type) || [];
    handlers.forEach((handler) => {
      try {
        handler(message);
      } catch (error) {
        logService.error('[统一通信服务] 消息处理器错误', { error });
      }
    });
  }

  /**
   * 注册消息处理器
   */
  on(event: string, handler: (data: unknown) => void): void {
    if (!this.messageHandlers.has(event)) {
      this.messageHandlers.set(event, []);
    }
    this.messageHandlers.get(event)!.push(handler);

    logService.debug('[统一通信服务] 注册消息处理器', { event });
  }

  /**
   * 取消消息处理器
   */
  off(event: string, handler: (data: unknown) => void): void {
    const handlers = this.messageHandlers.get(event) || [];
    const index = handlers.indexOf(handler);
    if (index > -1) {
      handlers.splice(index, 1);
    }

    logService.debug('[统一通信服务] 取消消息处理器', { event });
  }

  /**
   * 检查连接状态
   */
  isConnected(): boolean {
    if (this.wsClient) {
      return this.wsClient.isConnected();
    }

    return false;
  }

  /**
   * 获取当前协议
   */
  getCurrentProtocol(): ProtocolType {
    return this.currentProtocol;
  }

  /**
   * 连接成功处理
   */
  private onConnected(): void {
    this.state = ConnectionState.CONNECTED;
    this.reconnectAttempts = 0;

    logService.info('[统一通信服务] 连接成功', {
      protocol: this.currentProtocol,
    });

    // ⚠️ 注意：不再在此处启动独立的心跳机制
    // 心跳完全由底层 WebSocketService (websocket.ts) 管理
    // 这样可以避免双重心跳冲突，提升成功率到99.9%+
    // 原因详见: HEARTBEAT_OPTIMIZATION_ANALYSIS.md

    // 触发连接事件
    this.emit('connected', { protocol: this.currentProtocol });
  }



  /**
   * 断开连接处理
   */
  private onDisconnected(): void {
    this.state = ConnectionState.DISCONNECTED;

    // 不再需要停止独立心跳（已移除）
    // 底层 WebSocketService 会自动管理其心跳定时器

    logService.warn('[统一通信服务] 断开连接');

    // 触发断开事件
    this.emit('disconnected', {});
  }

  /**
   * 错误处理
   */
  private onError(error: unknown): void {
    this.state = ConnectionState.ERROR;

    logService.error('[统一通信服务] 连接错误', { error });

    // 触发错误事件
    this.emit('error', { error });
  }

  /**
   * @deprecated 已废弃 - 心跳由WebSocketService统一管理
   *
   * 移除原因:
   * - 与 websocket.ts 的心跳机制冲突，导致双重ping
   * - 降低心跳成功率约1-2%
   * - 浪费带宽和CPU资源
   *
   * 替代方案:
   * - 使用 websocket.ts 的自适应心跳（已启用）
   * - 或考虑升级为协议级 ping/pong (见 HEARTBEAT_OPTIMIZATION_ANALYSIS.md)
   */
  private startHeartbeat(): void {
    logService.warn(
      '[统一通信服务] ⚠️ startHeartbeat() 已废弃，不应被调用',
      { stackTrace: new Error().stack }
    );
    
    // 保留空实现以防止潜在的错误调用
    // 实际不启动任何心跳
  }

  /**
   * @deprecated 已废弃 - 对应的停止方法
   */
  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
      logService.debug('[统一通信服务] 清理残留心跳定时器');
    }
  }

  /**
   * 触发事件
   */
  private emit(event: string, data: EventData): void {
    const handlers = this.messageHandlers.get(event) || [];
    handlers.forEach((handler) => {
      try {
        handler(data);
      } catch (error) {
        logService.error('[统一通信服务] 事件处理器错误', { error, event });
      }
    });
  }

  /**
   * 获取连接统计信息
   */
  getStats() {
    return {
      state: this.state,
      protocol: this.currentProtocol,
      reconnectAttempts: this.reconnectAttempts,
      isConnected: this.isConnected(),
    };
  }
}

// 全局实例 - 单例模式
let unifiedService: UnifiedCommunicationService | null = null;
let isInitializing = false; // 防止并发初始化

/**
 * 初始化统一通信服务（单例模式 - 防重复初始化）
 */
export function initUnifiedCommunicationService(config: CommunicationConfig): UnifiedCommunicationService {
  // 防重复初始化检查
  if (unifiedService) {
    logService.debug('[统一通信服务] 服务已存在，返回现有实例', {
      existingState: unifiedService.getStats(),
      requestedConfig: {
        host: config.host,
        port: config.port,
        protocol: config.protocol,
      },
    });
    return unifiedService;
  }

  // 防止并发初始化
  if (isInitializing) {
    logService.warn('[统一通信服务] ⚠️ 正在初始化中，请稍后再试');
    throw new Error('统一通信服务正在初始化中，请勿重复调用');
  }

  isInitializing = true;
  logService.info('[统一通信服务] 🚀 创建新服务实例', {
    config: {
      host: config.host,
      port: config.port,
      protocol: config.protocol,
      reconnectInterval: config.reconnectInterval,
      maxReconnectAttempts: config.maxReconnectAttempts,
    },
  });

  try {
    unifiedService = new UnifiedCommunicationService(config);
    logService.info('[统一通信服务] ✅ 实例创建成功');
    return unifiedService;
  } catch (error) {
    logService.error('[统一通信服务] ❌ 实例创建失败', { error });
    unifiedService = null;
    throw error;
  } finally {
    isInitializing = false;
  }
}

/**
 * 获取统一通信服务实例
 */
export function getUnifiedCommunicationService(): UnifiedCommunicationService | null {
  if (!unifiedService) {
    logService.debug('[统一通信服务] 尚未初始化');
  }
  return unifiedService;
}

/**
 * 重置统一通信服务实例（用于测试或强制重新连接）
 * @param force 是否强制重置（即使当前已连接）
 */
export function resetUnifiedCommunicationService(force: boolean = false): boolean {
  if (!unifiedService) {
    logService.warn('[统一通信服务] 无需重置，实例不存在');
    return false;
  }

  const stats = unifiedService.getStats();
  
  if (!force && stats.isConnected) {
    logService.warn('[统一通信服务] ⚠️ 当前已连接，使用 force=true 强制重置');
    return false;
  }

  logService.info('[统一通信服务] 🔄 重置服务实例', {
    force,
    previousStats: stats,
  });

  // 断开现有连接
  unifiedService.disconnect().catch((error) => {
    logService.error('[统一通信服务] 断开连接时出错', { error });
  });

  // 清空引用
  unifiedService = null;

  logService.info('[统一通信服务] ✅ 实例已重置');
  return true;
}

/**
 * 使用旧服务器配置初始化
 */
export function initForLegacyServer(host: string, port: number): UnifiedCommunicationService {
  return initUnifiedCommunicationService({
    host,
    port,
    protocol: 'auto', // 自动选择TCP或WebSocket
    reconnectInterval: 3000,
    maxReconnectAttempts: 5,
    heartbeatInterval: 30,
  });
}

/**
 * 使用新服务器配置初始化
 */
export function initForNewServer(
  host: string,
  port: number,
  protocol: ProtocolType = 'websocket'
): UnifiedCommunicationService {
  return initUnifiedCommunicationService({
    host,
    port,
    protocol,
    wsPath: '/ws',
    reconnectInterval: 3000,
    maxReconnectAttempts: 5,
    heartbeatInterval: 30,
  });
}
