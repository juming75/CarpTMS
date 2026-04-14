/* global Event, MessageEvent, CloseEvent, WebSocket */
// 导入日志服务
import { logService } from './logService';

// WebSocket 服务配置
type WebSocketConfig = {
  url: string;
  reconnectInterval: number;
  maxReconnectAttempts: number;
  pingInterval: number;
  messageQueueMaxSize: number;
  messageExpiryTime: number; // 消息过期时间（毫秒）
};

// WebSocket 事件类型
type WebSocketEventType =
  | 'message'
  | 'connect'
  | 'disconnect'
  | 'error'
  | 'reconnect_attempt'
  | 'reconnect_success'
  | 'reconnect_failed';

// WebSocket 消息处理器类型
type WebSocketMessageHandler<T = Record<string, unknown>> = (data: T) => void;

// 消息优先级枚举
enum MessagePriority {
  LOW = 0,
  NORMAL = 1,
  HIGH = 2,
  URGENT = 3,
}

// 消息队列项类型
type MessageQueueItem<T = Record<string, unknown>> = {
  data: T;
  timestamp: number;
  retryCount: number;
  priority: MessagePriority;
  expiry: number; // 消息过期时间戳
};

export class WebSocketService {
  private ws: WebSocket | null = null;
  private config: WebSocketConfig;
  private reconnectAttempts = 0;
  private reconnectTimer: number | null = null;
  private pingTimer: number | null = null;
  private handlers: Map<WebSocketEventType, Set<WebSocketMessageHandler<Record<string, unknown>>>> = new Map();
  private messageQueue: MessageQueueItem<Record<string, unknown>>[] = [];
  private isConnecting = false;
  private isManualClose = false;
  private lastPongTime = 0;
  private isHealthy = false;

  constructor(wsUrl?: string) {
    // 默认配置
    this.config = {
      url: wsUrl || this.getWebSocketUrl(),
      reconnectInterval: 1000, // 初始重连间隔
      maxReconnectAttempts: 10,
      pingInterval: 30000, // 30秒发送一次ping
      messageQueueMaxSize: 100, // 消息队列最大容量
      messageExpiryTime: 60000, // 消息过期时间（60秒）
    };

    // 设置网络状态监听器
    this.setupNetworkListeners();
  }

  // 获取WebSocket URL（直接连接到后端WebSocket服务器）
  private getWebSocketUrl(): string {
    try {
      // 直接连接到后端WebSocket服务器，不通过Vite代理
      // 后端服务运行在8082端口
      return 'ws://localhost:8082/ws';
    } catch (error) {
      console.error('获取WebSocket URL失败:', error);
      return 'ws://localhost:8082/ws'; // 默认备用URL
    }
  }

  // 检查WebSocket服务器是否可用
  public async checkServerAvailability(): Promise<boolean> {
    return new Promise((resolve) => {
      // 尝试直接连接WebSocket服务器来检查可用性
      const wsUrl = this.config.url;
      const ws = new window.WebSocket(wsUrl);
      
      // 设置超时
      const timeout = setTimeout(() => {
        ws.close();
        logService.logWebSocket('WebSocket服务器检查超时', 'warn', {
          url: wsUrl,
          timeout: 3000,
        });
        resolve(false);
      }, 3000);
      
      ws.onopen = () => {
        clearTimeout(timeout);
        ws.close();
        logService.logWebSocket('WebSocket服务器可用', 'debug', {
          url: wsUrl,
        });
        resolve(true);
      };
      
      ws.onerror = () => {
        clearTimeout(timeout);
        ws.close();
        logService.logWebSocket('WebSocket服务器检查失败', 'warn', {
          url: wsUrl,
        });
        resolve(false);
      };
      
      ws.onclose = () => {
        clearTimeout(timeout);
      };
    });
  }

  // 连接WebSocket
  public async connect(): Promise<boolean> {
    return new Promise(async (resolve) => {
      const WebSocket_OPEN = 1;
      if (this.ws?.readyState === WebSocket_OPEN || this.isConnecting) {
        logService.logWebSocket('WebSocket已经连接或正在连接中', 'debug', {
          url: this.config.url,
          readyState: this.ws?.readyState,
        });
        resolve(false);
        return;
      }

      // 检查服务器是否可用
      const isServerAvailable = await this.checkServerAvailability();
      if (!isServerAvailable) {
        logService.logWebSocket('WebSocket服务器不可用，延迟重连', 'warn', {
          url: this.config.url,
        });

        // 延迟一段时间后再尝试连接
        setTimeout(() => {
          this.attemptReconnect();
        }, 3000); // 减少延迟时间，默认5秒

        resolve(false);
        return;
      }

      this.isConnecting = true;
      this.isManualClose = false;
      this.lastPongTime = Date.now();

      logService.logWebSocket('WebSocket连接开始', 'debug', {
        url: this.config.url,
      });

      try {
        // 检查WebSocket是否可用
        if (typeof window === 'undefined' || typeof window.WebSocket === 'undefined') {
          throw new Error('WebSocket is not supported in this environment');
        }

        // 清理之前的连接
        if (this.ws) {
          try {
            this.ws.close(1000, 'Reconnecting');
          } catch (error) {
            logService.logWebSocket('关闭旧WebSocket连接失败', 'warn', {
              url: this.config.url,
            }, error as Error);
          }
          // 清理事件处理器
          this.ws.onopen = null;
          this.ws.onmessage = null;
          this.ws.onclose = null;
          this.ws.onerror = null;
          this.ws = null;
        }

        this.ws = new window.WebSocket(this.config.url);

        // 设置连接超时
        const connectionTimeout = window.setTimeout(() => {
          if (this.isConnecting) {
            this.isConnecting = false;
            this.isHealthy = false;

            logService.logWebSocket('WebSocket连接超时', 'error', {
              url: this.config.url,
            });

            // 关闭连接
            if (this.ws) {
              try {
                this.ws.close(1000, 'Connection timeout');
              } catch (error) {
                logService.logWebSocket('关闭WebSocket连接失败', 'warn', {
                  url: this.config.url,
                }, error as Error);
              }
              // 清理事件处理器
              this.ws.onopen = null;
              this.ws.onmessage = null;
              this.ws.onclose = null;
              this.ws.onerror = null;
              this.ws = null;
            }

            // 触发错误事件
            this.emit('error', {
              error: 'WebSocket连接超时',
              details: 'Connection timeout after 5 seconds',
            });

            // 尝试重连
            this.attemptReconnect();

            resolve(false);
          }
        }, 5000); // 减少超时时间，默认10秒

        this.ws.onopen = () => {
          // 清除连接超时定时器
          window.clearTimeout(connectionTimeout);

          this.isConnecting = false;
          this.reconnectAttempts = 0;
          this.isHealthy = true;
          this.lastPongTime = Date.now();

          logService.logWebSocket('WebSocket连接成功', 'info', {
            url: this.config.url,
          });

          // 触发连接成功事件
          this.emit('connect', {
            message: 'WebSocket连接成功',
            timestamp: Date.now(),
          });

          // 启动ping/pong机制
          this.startPingPong();

          // 发送缓存的消息
          this.flushMessageQueue();

          resolve(true);
        };

        this.ws.onmessage = (event: MessageEvent) => {
          try {
            // 处理ping/pong消息
            if (event.data === 'pong') {
              this.lastPongTime = Date.now();
              this.isHealthy = true;
              return;
            }

            // 尝试解析JSON消息
            const data = JSON.parse(event.data as string);

            // 处理JSON格式的pong消息
            if (data.msg_type === 'pong') {
              this.lastPongTime = Date.now();
              this.isHealthy = true;
              return;
            }

            logService.logWebSocket('WebSocket收到消息', 'debug', {
              url: this.config.url,
              messageType: data.msg_type || data.type || 'unknown',
              messageLength: (event.data as string).length,
            });
            this.emit('message', data);
          } catch (error) {
            logService.logWebSocket(
              'WebSocket消息解析错误',
              'error',
              {
                url: this.config.url,
                rawData: event.data,
              },
              error as Error
            );
            this.emit('error', {
              error: 'Message parsing error',
              details: (error as Error).message,
              rawData: event.data,
            });
          }
        };

        this.ws.onclose = (event: CloseEvent) => {
          // 清除连接超时定时器
          window.clearTimeout(connectionTimeout);

          this.isConnecting = false;
          this.isHealthy = false;

          logService.logWebSocket('WebSocket连接关闭', 'warn', {
            url: this.config.url,
            code: event.code,
            reason: event.reason,
            wasClean: event.wasClean,
          });

          // 停止ping/pong机制
          this.stopPingPong();

          // 触发断开连接事件
          this.emit('disconnect', {
            message: 'WebSocket连接关闭',
            code: event.code,
            reason: event.reason,
            wasClean: event.wasClean,
          });

          // 如果不是手动关闭，尝试重连
          if (!this.isManualClose) {
            this.attemptReconnect();
          }

          resolve(false);
        };

        this.ws.onerror = (_error: Event) => {
          // 清除连接超时定时器
          window.clearTimeout(connectionTimeout);

          this.isConnecting = false;
          this.isHealthy = false;

          logService.logWebSocket(
            'WebSocket连接错误',
            'error',
            {
              url: this.config.url,
            },
            new Error('WebSocket connection error')
          );

          // 触发错误事件
          this.emit('error', {
            error: 'WebSocket连接错误',
            details: 'Unknown error',
          });

          resolve(false);
        };
      } catch (error) {
        this.isConnecting = false;
        this.isHealthy = false;

        logService.logWebSocket(
          'WebSocket初始化失败',
          'error',
          {
            url: this.config.url,
          },
          error as Error
        );

        // 触发错误事件
        this.emit('error', {
          error: 'WebSocket初始化失败',
          details: (error as Error).message,
        });

        // 尝试重连
        this.attemptReconnect();

        resolve(false);
      }
    });
  }

  // 断开WebSocket连接
  public disconnect(): Promise<void> {
    return new Promise((resolve) => {
      this.isManualClose = true;
      this.isHealthy = false;

      // 取消重连和ping/pong
      this.cancelReconnect();
      this.stopPingPong();

      // 清空消息队列
      this.messageQueue = [];

      // 清理事件处理器
      this.handlers.clear();

      if (this.ws) {
        try {
          this.ws.close(1000, 'Manual disconnect');
        } catch (error) {
          console.error('关闭WebSocket连接失败:', error);
        } finally {
          // 清理WebSocket实例
          this.ws.onopen = null;
          this.ws.onmessage = null;
          this.ws.onclose = null;
          this.ws.onerror = null;
          this.ws = null;
        }
      }

      logService.logWebSocket('WebSocket连接已手动断开', 'info', {
        url: this.config.url,
      });

      resolve();
    });
  }

  // 尝试重连
  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.config.maxReconnectAttempts || this.isManualClose) {
      // 触发重连失败事件
      this.emit('reconnect_failed', {
        message: 'WebSocket重连失败',
        attempts: this.reconnectAttempts,
        maxAttempts: this.config.maxReconnectAttempts,
      });

      logService.logWebSocket('WebSocket重连失败', 'error', {
        url: this.config.url,
        attempts: this.reconnectAttempts,
        maxAttempts: this.config.maxReconnectAttempts,
      });

      return;
    }

    this.reconnectAttempts++;

    // 计算重连间隔（指数退避）
    const backoff = Math.min(
      this.config.reconnectInterval * Math.pow(2, this.reconnectAttempts - 1),
      15000 // 减少最大重连间隔，默认30秒
    );

    // 添加随机抖动，避免多个客户端同时重连
    const jitter = Math.random() * 500; // 减少抖动范围，默认1000
    const reconnectDelay = backoff + jitter;

    // 触发重连尝试事件
    this.emit('reconnect_attempt', {
      message: 'WebSocket尝试重连',
      attempt: this.reconnectAttempts,
      maxAttempts: this.config.maxReconnectAttempts,
      delay: Math.round(reconnectDelay),
    });

    logService.logWebSocket('WebSocket尝试重连', 'warn', {
      url: this.config.url,
      attempt: this.reconnectAttempts,
      maxAttempts: this.config.maxReconnectAttempts,
      delay: Math.round(reconnectDelay),
    });

    // 尝试重连
    this.reconnectTimer = window.setTimeout(async () => {
      logService.logWebSocket('WebSocket执行重连', 'debug', {
        url: this.config.url,
        attempt: this.reconnectAttempts,
        maxAttempts: this.config.maxReconnectAttempts,
      });

      // 在重连之前再次检查服务器是否可用
      const isServerAvailable = await this.checkServerAvailability();
      if (isServerAvailable) {
        this.connect();
      } else {
        logService.logWebSocket('WebSocket服务器仍然不可用，继续延迟重连', 'warn', {
          url: this.config.url,
          attempt: this.reconnectAttempts,
          maxAttempts: this.config.maxReconnectAttempts,
        });
        // 继续尝试重连
        this.attemptReconnect();
      }
    }, reconnectDelay);
  }

  // 设置网络状态监听器
  private setupNetworkListeners(): void {
    // 监听网络恢复事件
    window.addEventListener('online', () => {
      console.log('网络已恢复，尝试重新连接WebSocket');

      // 重置重连尝试次数
      this.reconnectAttempts = 0;

      // 如果不是手动关闭，尝试重新连接
      const WebSocket_OPEN = 1;
      if (!this.isManualClose && (!this.ws || this.ws.readyState !== WebSocket_OPEN)) {
        this.connect();
      }
    });

    // 监听网络断开事件
    window.addEventListener('offline', () => {
      console.log('网络已断开，WebSocket连接将在网络恢复后自动重连');

      // 如果连接已建立，断开连接
      const WebSocket_OPEN = 1;
      if (this.ws && this.ws.readyState === WebSocket_OPEN) {
        try {
          this.ws.close(1001, 'Network disconnected');
        } catch (error) {
          console.error('关闭WebSocket连接失败:', error);
        }
      }
    });
  }

  // 取消重连
  private cancelReconnect(): void {
    if (this.reconnectTimer) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  // 启动ping/pong机制
  private startPingPong(): void {
    this.stopPingPong();

    const WebSocket_OPEN = 1;

    this.pingTimer = window.setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket_OPEN) {
        // 检查pong响应时间
        const now = Date.now();
        if (now - this.lastPongTime > this.config.pingInterval * 1.5) { // 减少超时时间，默认2倍
          // 超过1.5倍ping间隔没有收到pong，认为连接不健康
          this.isHealthy = false;
          logService.logWebSocket('WebSocket连接不健康，没有收到pong响应', 'warn', {
            url: this.config.url,
            lastPongTime: this.lastPongTime,
            currentTime: now,
            pingInterval: this.config.pingInterval,
          });

          // 断开连接并尝试重连
          try {
            if (this.ws && this.ws.readyState === WebSocket_OPEN) {
              this.ws.close(1000, 'Ping timeout');
            }
          } catch (error) {
            logService.logWebSocket('关闭WebSocket连接失败', 'warn', {
              url: this.config.url,
            }, error as Error);
          }
          return;
        }

        // 发送ping消息
        try {
          // 发送符合后端期望格式的ping消息
          const pingMessage = {
            msg_type: 'ping',
            timestamp: Math.floor(Date.now() / 1000), // 转换为Unix时间戳（秒）
            payload: {}, // 发送空对象而不是null，确保后端能够正确解析
          };
          this.ws.send(JSON.stringify(pingMessage));
          logService.logWebSocket('WebSocket发送ping消息', 'debug', {
            url: this.config.url,
            timestamp: pingMessage.timestamp,
          });
        } catch (error) {
          logService.logWebSocket('发送ping消息失败', 'error', {
            url: this.config.url,
          }, error as Error);
          this.isHealthy = false;
        }
      } else {
        // 如果连接已关闭，停止ping/pong
        logService.logWebSocket('WebSocket连接已关闭，停止ping/pong', 'debug', {
          url: this.config.url,
          readyState: this.ws?.readyState,
        });
        this.stopPingPong();
      }
    }, this.config.pingInterval) as unknown as number;
  }

  // 停止ping/pong机制
  private stopPingPong(): void {
    if (this.pingTimer) {
      window.clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  // 发送消息
  public send(data: Record<string, unknown>, priority: MessagePriority = MessagePriority.NORMAL): Promise<boolean> {
    return new Promise((resolve, reject) => {
      const WebSocket_OPEN = 1;

      // 转换消息格式，将 type 字段转换为 msg_type 字段以匹配后端格式
      const formattedData = {
        ...data,
        msg_type: data.type || data.msg_type,
        msg_id: data.msg_id || `msg_${Date.now()}_${Math.floor(Math.random() * 10000)}`, // 生成唯一消息ID
        device_id: data.device_id,
        command: data.command,
        timestamp: data.timestamp || Math.floor(Date.now() / 1000), // 转换为Unix时间戳（秒）
        payload: data.payload !== undefined ? data.payload : {},
      };
      // 删除原始的 type 字段，避免重复
      if ('type' in formattedData) {
        delete formattedData.type;
      }

      // 如果连接正常，直接发送
      if (this.ws && this.ws.readyState === WebSocket_OPEN && this.isHealthy) {
        try {
          this.ws.send(JSON.stringify(formattedData));
          logService.logWebSocket('WebSocket消息发送成功', 'debug', {
            url: this.config.url,
            messageType: formattedData.msg_type,
            messageId: formattedData.msg_id,
            priority: priority,
          });
          resolve(true);
        } catch (error) {
          logService.logWebSocket('发送WebSocket消息失败', 'error', {
            url: this.config.url,
            messageType: formattedData.msg_type,
            messageId: formattedData.msg_id,
            priority: priority,
          }, error as Error);
          this.enqueueMessage(data, priority);
          reject(error);
        }
      } else {
        // 否则将消息加入队列
        logService.logWebSocket('WebSocket未连接或不健康，消息已加入队列', 'warn', {
          url: this.config.url,
          messageType: formattedData.msg_type,
          messageId: formattedData.msg_id,
          priority: priority,
          readyState: this.ws?.readyState,
          isHealthy: this.isHealthy,
        });
        this.enqueueMessage(data, priority);

        // 如果未连接，尝试重新连接
        if (!this.ws || this.ws.readyState !== WebSocket_OPEN) {
          logService.logWebSocket('WebSocket未连接，尝试重新连接', 'debug', {
            url: this.config.url,
          });
          this.connect();
        }
        resolve(false);
      }
    });
  }

  // 将消息加入队列
  private enqueueMessage(data: Record<string, unknown>, priority: MessagePriority = MessagePriority.NORMAL): void {
    // 清理过期消息
    this.cleanExpiredMessages();

    // 限制队列大小
    if (this.messageQueue.length >= this.config.messageQueueMaxSize) {
      // 移除最低优先级的消息
      this.removeLowestPriorityMessage();
      console.warn('WebSocket消息队列已满，已移除最低优先级消息');
    }

    // 添加新消息到队列
    const messageItem = {
      data,
      timestamp: Date.now(),
      retryCount: 0,
      priority,
      expiry: Date.now() + this.config.messageExpiryTime,
    };

    // 按优先级插入消息
    this.insertMessageByPriority(messageItem);
  }

  // 清理过期消息
  private cleanExpiredMessages(): void {
    const now = Date.now();
    const originalLength = this.messageQueue.length;

    this.messageQueue = this.messageQueue.filter((item) => {
      const isExpired = now > item.expiry;
      if (isExpired) {
        console.log('WebSocket消息已过期，已从队列中移除');
      }
      return !isExpired;
    });

    if (this.messageQueue.length < originalLength) {
      console.log(`已清理 ${originalLength - this.messageQueue.length} 条过期消息`);
    }
  }

  // 移除最低优先级的消息
  private removeLowestPriorityMessage(): void {
    if (this.messageQueue.length === 0) return;

    // 找到最低优先级的消息索引
    let lowestPriority = MessagePriority.URGENT;
    let lowestIndex = 0;

    this.messageQueue.forEach((item, index) => {
      if (
        item.priority < lowestPriority ||
        (item.priority === lowestPriority && item.timestamp < this.messageQueue[lowestIndex].timestamp)
      ) {
        lowestPriority = item.priority;
        lowestIndex = index;
      }
    });

    // 移除最低优先级的消息
    this.messageQueue.splice(lowestIndex, 1);
  }

  // 按优先级插入消息
  private insertMessageByPriority(message: MessageQueueItem): void {
    // 找到插入位置
    let insertIndex = 0;
    while (insertIndex < this.messageQueue.length) {
      const existingMessage = this.messageQueue[insertIndex];
      if (
        message.priority > existingMessage.priority ||
        (message.priority === existingMessage.priority && message.timestamp < existingMessage.timestamp)
      ) {
        break;
      }
      insertIndex++;
    }

    // 插入消息
    this.messageQueue.splice(insertIndex, 0, message);
  }

  // 发送队列中的消息
  private flushMessageQueue(): void {
    const WebSocket_OPEN = 1;
    if (!this.ws || this.ws.readyState !== WebSocket_OPEN || !this.isHealthy) {
      logService.logWebSocket('WebSocket未连接或不健康，消息队列发送失败', 'warn', {
        url: this.config.url,
        readyState: this.ws?.readyState,
        isHealthy: this.isHealthy,
        queueLength: this.messageQueue.length,
      });
      return;
    }

    // 清理过期消息
    this.cleanExpiredMessages();

    // 复制队列并清空，避免并发问题
    const messagesToSend = [...this.messageQueue];
    this.messageQueue = [];

    logService.logWebSocket('开始发送消息队列', 'debug', {
      url: this.config.url,
      messageCount: messagesToSend.length,
    });

    // 发送所有消息（已按优先级排序）
    let successCount = 0;
    let failedCount = 0;

    messagesToSend.forEach((item, index) => {
      try {
        if (this.ws && this.ws.readyState === WebSocket_OPEN) {
          // 转换消息格式，将 type 字段转换为 msg_type 字段以匹配后端格式
          const formattedData = {
            ...item.data,
            msg_type: item.data.type || item.data.msg_type,
            msg_id: item.data.msg_id || `msg_${Date.now()}_${Math.floor(Math.random() * 10000)}`, // 生成唯一消息ID
            device_id: item.data.device_id,
            command: item.data.command,
            timestamp: item.data.timestamp || Math.floor(Date.now() / 1000), // 转换为Unix时间戳（秒）
            payload: item.data.payload !== undefined ? item.data.payload : {},
          };
          // 删除原始的 type 字段，避免重复
          if ('type' in formattedData) {
            delete formattedData.type;
          }

          this.ws.send(JSON.stringify(formattedData));
          successCount++;
          logService.logWebSocket('WebSocket消息队列发送成功', 'debug', {
            url: this.config.url,
            messageType: formattedData.msg_type,
            messageId: formattedData.msg_id,
            priority: item.priority,
            index: index + 1,
            total: messagesToSend.length,
          });
        } else {
          failedCount++;
          logService.logWebSocket('WebSocket连接已关闭，消息发送失败', 'warn', {
            url: this.config.url,
            messageType: item.data.type || item.data.msg_type,
            priority: item.priority,
            index: index + 1,
            total: messagesToSend.length,
          });
          this.enqueueMessage(
            {
              ...item.data,
              _retryCount: item.retryCount + 1,
            },
            item.priority
          );
        }
      } catch (error) {
        failedCount++;
        logService.logWebSocket('发送队列消息失败', 'error', {
          url: this.config.url,
          messageType: item.data.type || item.data.msg_type,
          priority: item.priority,
          index: index + 1,
          total: messagesToSend.length,
        }, error as Error);
        // 重新加入队列，但增加重试计数
        this.enqueueMessage(
          {
            ...item.data,
            _retryCount: item.retryCount + 1,
          },
          item.priority
        );
      }
    });

    if (messagesToSend.length > 0) {
      logService.logWebSocket('消息队列发送完成', 'info', {
        url: this.config.url,
        total: messagesToSend.length,
        success: successCount,
        failed: failedCount,
      });
    }
  }

  // 注册事件处理器
  public on(eventType: WebSocketEventType, handler: WebSocketMessageHandler<Record<string, unknown>>): void {
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, new Set());
    }
    this.handlers.get(eventType)?.add(handler);
  }

  // 移除事件处理器
  public off(eventType: WebSocketEventType, handler: WebSocketMessageHandler<Record<string, unknown>>): void {
    this.handlers.get(eventType)?.delete(handler);
  }

  // 触发事件
  private emit(eventType: WebSocketEventType, data: Record<string, unknown>): void {
    this.handlers.get(eventType)?.forEach((handler) => {
      try {
        handler(data);
      } catch (error) {
        console.error(`WebSocket事件处理器错误 (${eventType}):`, error);
      }
    });
  }

  // 获取连接状态
  public getReadyState(): number | null {
    return this.ws?.readyState || null;
  }

  // 获取连接健康状态
  public getHealthStatus(): boolean {
    return this.isHealthy;
  }

  // 获取重连尝试次数
  public getReconnectAttempts(): number {
    return this.reconnectAttempts;
  }

  // 获取消息队列长度
  public getMessageQueueLength(): number {
    return this.messageQueue.length;
  }

  // 检查是否已连接
  public isConnected(): boolean {
    const WebSocket_OPEN = 1;
    return this.ws?.readyState === WebSocket_OPEN && this.isHealthy;
  }

  // 更新服务器配置
  public updateServerConfig(ip: string, port: string): void {
    const newUrl = `ws://${ip}:${port}/ws`;
    if (this.config.url !== newUrl) {
      this.config.url = newUrl;
      // 如果当前已连接，重新连接
      const WebSocket_OPEN = 1;
      if (this.ws && this.ws.readyState === WebSocket_OPEN) {
        this.disconnect();
        this.connect();
      }
    }
  }
}

// 创建WebSocket服务实例
export const webSocketService = new WebSocketService();

// 导出WebSocket事件类型
export type { WebSocketEventType, WebSocketMessageHandler };


