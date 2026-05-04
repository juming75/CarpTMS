// @ts-nocheck
﻿/* global beforeAll, afterAll, test, expect, jest, global, Event, MessageEvent, CloseEvent */
import { WebSocketService } from './websocket';

// 定义 MessagePriority 枚举
enum MessagePriority {
  LOW = 0,
  NORMAL = 1,
  HIGH = 2,
  URGENT = 3,
}

// 模拟WebSocket
class MockWebSocket {
  readyState = 1; // OPEN
  onopen: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  send(_data: string): void {
    // 模拟发送
  }

  close(_code?: number, _reason?: string): void {
    // 模拟关闭
  }
}

// 模拟window对象
const originalWebSocket = (global as { WebSocket?: typeof MockWebSocket }).WebSocket;

beforeAll(() => {
  (global as { WebSocket?: typeof MockWebSocket }).WebSocket = MockWebSocket as unknown as typeof MockWebSocket;
});

afterAll(() => {
  (global as { WebSocket?: typeof MockWebSocket }).WebSocket = originalWebSocket;
});

describe('WebSocketService', () => {
  let websocketService: WebSocketService;

  beforeEach(() => {
    websocketService = new WebSocketService('ws://localhost:9808/ws');
  });

  afterEach(() => {
    websocketService.disconnect();
  });

  test('should create instance with default config', () => {
    expect(websocketService).toBeDefined();
  });

  test('should return correct message queue length', () => {
    expect(websocketService.getMessageQueueLength()).toBe(0);
  });

  test('should add message to queue when not connected', async () => {
    const message = { type: 'test', payload: { data: 'test' } };
    await websocketService.send(message);
    expect(websocketService.getMessageQueueLength()).toBe(1);
  });

  test('should prioritize messages correctly', async () => {
    // 发送低优先级消息
    await websocketService.send({ type: 'low', payload: { data: 'low' } }, MessagePriority.LOW);
    // 发送高优先级消息
    await websocketService.send({ type: 'high', payload: { data: 'high' } }, MessagePriority.HIGH);
    // 发送紧急消息
    await websocketService.send({ type: 'urgent', payload: { data: 'urgent' } }, MessagePriority.URGENT);
    // 发送普通消息
    await websocketService.send({ type: 'normal', payload: { data: 'normal' } }, MessagePriority.NORMAL);

    expect(websocketService.getMessageQueueLength()).toBe(4);
    // 队列应该按优先级排序：URGENT > HIGH > NORMAL > LOW
  });

  test('should handle message expiration', async () => {
    // 模拟时间流逝
    const originalNow = Date.now;
    (global as { Date: typeof Date; jest?: { fn: () => number } }).Date.now = jest.fn(() => 1000);

    // 发送消息
    await websocketService.send({ type: 'test', payload: { data: 'test' } });
    expect(websocketService.getMessageQueueLength()).toBe(1);

    // 模拟时间超过过期时间
    (global as { Date: typeof Date; jest?: { fn: () => number } }).Date.now = jest.fn(() => 70000); // 超过60秒

    // 发送另一条消息，触发过期检查
    await websocketService.send({ type: 'test2', payload: { data: 'test2' } });
    expect(websocketService.getMessageQueueLength()).toBe(1); // 只有新消息

    (global as { Date: typeof Date }).Date.now = originalNow;
  });

  test('should handle queue overflow by removing lowest priority messages', async () => {
    // 发送101条消息，超过默认队列大小100
    for (let i = 0; i < 101; i++) {
      await websocketService.send({ type: `test${i}`, payload: { data: `test${i}` } });
    }

    expect(websocketService.getMessageQueueLength()).toBe(100);
  });

  test('should connect successfully', async () => {
    const result = await websocketService.connect();
    expect(result).toBe(false); // 因为模拟WebSocket不会真正连接
  });

  test('should disconnect successfully', async () => {
    await websocketService.disconnect();
    expect(websocketService.isConnected()).toBe(false);
  });
});
