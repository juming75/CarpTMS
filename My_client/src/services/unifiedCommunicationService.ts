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
      logService.warn('[统一通信服务] 已连接，无需重复连接');
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

      // 初始化WebSocket客户端（使用正确的URL）
      const wsUrl = `ws://${this.config.host}:${this.config.port}/ws`;
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

    // 启动心跳
    this.startHeartbeat();

    // 触发连接事件
    this.emit('connected', { protocol: this.currentProtocol });
  }



  /**
   * 断开连接处理
   */
  private onDisconnected(): void {
    this.state = ConnectionState.DISCONNECTED;
    this.stopHeartbeat();

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
   * 启动心跳
   */
  private startHeartbeat(): void {
    this.stopHeartbeat();

    const interval = this.config.heartbeatInterval || 30;

    this.heartbeatTimer = setInterval(async () => {
      try {
        logService.debug('[统一通信服务] 发送心跳');
        await this.send({
          type: 'heartbeat',
          command: 'ping',
          timestamp: Date.now(),
          payload: {},
        });
      } catch (error) {
        logService.error('[统一通信服务] 心跳失败', { error });
        this.onError(error);
      }
    }, interval * 1000);

    logService.debug('[统一通信服务] 心跳已启动', { interval });
  }

  /**
   * 停止心跳
   */
  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
      logService.debug('[统一通信服务] 心跳已停止');
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

// 全局实例
let unifiedService: UnifiedCommunicationService | null = null;

/**
 * 初始化统一通信服务
 */
export function initUnifiedCommunicationService(config: CommunicationConfig): UnifiedCommunicationService {
  unifiedService = new UnifiedCommunicationService(config);
  return unifiedService;
}

/**
 * 获取统一通信服务实例
 */
export function getUnifiedCommunicationService(): UnifiedCommunicationService | null {
  return unifiedService;
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
