/**
 * 兼容层 V2 单元测试
 * 测试协议适配器、协议管理器和兼容层核心功能
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { protocolManager } from './protocolManager';
import { LegacyProtocolAdapter } from './legacyProtocolAdapter';
import { NewProtocolAdapter } from './newProtocolAdapter';

describe('LegacyProtocolAdapter', () => {
  let adapter: LegacyProtocolAdapter;

  beforeEach(() => {
    adapter = new LegacyProtocolAdapter();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('buildLoginRequest', () => {
    it('应该正确构建旧协议登录请求', () => {
      const request = adapter.buildLoginRequest('ED', '888888');

      expect(request).toEqual({
        command: 'Apply_Login',
        data: {
          username: 'ED',
          password: '888888',
        },
      });
    });

    it('应该处理空用户名和密码', () => {
      const request = adapter.buildLoginRequest('', '');

      expect(request).toEqual({
        command: 'Apply_Login',
        data: {
          username: '',
          password: '',
        },
      });
    });
  });

  describe('parseLoginResponse', () => {
    it('应该正确解析成功登录响应', () => {
      const response = {
        data: {
          flag: 1,
          user_id: '123',
        },
      };

      const result = adapter.parseLoginResponse(response);

      expect(result.success).toBe(true);
      expect(result.userId).toBe('123');
    });

    it('应该正确解析失败登录响应', () => {
      const response = {
        data: {
          flag: 0,
          error: 'Invalid credentials',
        },
      };

      const result = adapter.parseLoginResponse(response);

      expect(result.success).toBe(false);
      expect(result.error).toBe('Invalid credentials');
    });
  });

  describe('buildVehicleCreateRequest', () => {
    it('应该正确构建车辆创建请求', () => {
      const vehicleData = {
        vehicle_name: 'Test Vehicle',
        device_id: 'DEV001',
        own_no: 'OWN001',
      };

      const request = adapter.buildVehicleCreateRequest(vehicleData);

      expect(request.command).toBe('Vehicle_Create');
      expect(request.data).toHaveProperty('vehicle_name', 'Test Vehicle');
      expect(request.data).toHaveProperty('device_id', 'DEV001');
      expect(request.data).toHaveProperty('own_no', 'OWN001');
    });
  });

  describe('isLegacyResponse', () => {
    it('应该正确识别旧协议响应', () => {
      expect(adapter.isLegacyResponse({ command: 'Apply_Login', data: {} })).toBe(true);
      expect(adapter.isLegacyResponse({ data: { flag: 1 } })).toBe(true);
    });

    it('应该正确识别非旧协议响应', () => {
      expect(adapter.isLegacyResponse({ code: 200, message: 'success' })).toBe(false);
    });
  });

  describe('convertToCamelCase', () => {
    it('应该正确转换蛇形命名到驼峰命名', () => {
      const input = {
        vehicle_id: 1,
        vehicle_name: 'Test',
        user_id: 123,
      };

      const result = adapter.convertToCamelCase(input);

      expect(result).toEqual({
        vehicleId: 1,
        vehicleName: 'Test',
        userId: 123,
      });
    });

    it('应该处理嵌套对象', () => {
      const input = {
        vehicle_id: 1,
        user: {
          user_id: 123,
          user_name: 'Test',
        },
      };

      const result = adapter.convertToCamelCase(input);

      expect(result).toEqual({
        vehicleId: 1,
        user: {
          userId: 123,
          userName: 'Test',
        },
      });
    });
  });

  describe('convertToSnakeCase', () => {
    it('应该正确转换驼峰命名到蛇形命名', () => {
      const input = {
        vehicleId: 1,
        vehicleName: 'Test',
        userId: 123,
      };

      const result = adapter.convertToSnakeCase(input);

      expect(result).toEqual({
        vehicle_id: 1,
        vehicle_name: 'Test',
        user_id: 123,
      });
    });
  });
});

describe('NewProtocolAdapter', () => {
  let adapter: NewProtocolAdapter;

  beforeEach(() => {
    adapter = new NewProtocolAdapter();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('buildLoginRequest', () => {
    it('应该正确构建新协议登录请求', () => {
      const request = adapter.buildLoginRequest('ED', '888888');

      expect(request.action).toBe('login');
      expect(request.resource).toBe('auth');
      expect(request.params).toEqual({
        username: 'ED',
        password: '888888',
      });
      expect(request).toHaveProperty('timestamp');
      expect(request.version).toBe('1.0');
    });
  });

  describe('parseLoginResponse', () => {
    it('应该正确解析成功登录响应', () => {
      const response = {
        code: 200,
        message: 'success',
        data: {
          userId: '123',
          token: 'test-token',
          user: {
            username: 'admin',
            name: 'Admin User',
          },
        },
      };

      const result = adapter.parseLoginResponse(response);

      expect(result.success).toBe(true);
      expect(result.userId).toBe('123');
      expect(result.token).toBe('test-token');
    });

    it('应该正确解析失败登录响应', () => {
      const response = {
        code: 401,
        message: 'Invalid credentials',
      };

      const result = adapter.parseLoginResponse(response);

      expect(result.success).toBe(false);
      expect(result.error).toBe('Invalid credentials');
    });
  });

  describe('isNewProtocolResponse', () => {
    it('应该正确识别新协议响应', () => {
      expect(adapter.isNewProtocolResponse({ code: 200, message: 'success' })).toBe(true);
    });

    it('应该正确识别非新协议响应', () => {
      expect(adapter.isNewProtocolResponse({ command: 'Apply_Login', data: {} })).toBe(false);
    });
  });

  describe('mapHttpMethodToAction', () => {
    it('应该正确映射 HTTP 方法到 action', () => {
      expect(adapter.mapHttpMethodToAction('GET')).toBe('get');
      expect(adapter.mapHttpMethodToAction('POST')).toBe('create');
      expect(adapter.mapHttpMethodToAction('PUT')).toBe('update');
      expect(adapter.mapHttpMethodToAction('DELETE')).toBe('delete');
    });
  });

  describe('mapActionToHttpMethod', () => {
    it('应该正确映射 action 到 HTTP 方法', () => {
      expect(adapter.mapActionToHttpMethod('get')).toBe('GET');
      expect(adapter.mapActionToHttpMethod('create')).toBe('POST');
      expect(adapter.mapActionToHttpMethod('update')).toBe('PUT');
      expect(adapter.mapActionToHttpMethod('delete')).toBe('DELETE');
      expect(adapter.mapActionToHttpMethod('login')).toBe('POST');
    });
  });
});

describe('ProtocolManager', () => {
  let manager;

  beforeEach(() => {
    manager = protocolManager;
    manager.setProtocol('new', 'http://127.0.0.1:9808');
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('buildLoginRequest', () => {
    it('应该使用新协议构建登录请求', () => {
      const request = manager.buildLoginRequest('ED', '888888');

      expect(typeof request).toBe('string');
      const parsed = JSON.parse(request);
      expect(parsed.action).toBe('login');
      expect(parsed.resource).toBe('auth');
      expect(parsed.params).toEqual({ username: 'ED', password: '888888' });
    });

    it('应该使用旧协议构建登录请求', () => {
      manager.setProtocol('legacy');

      const request = manager.buildLoginRequest('ED', '888888');

      expect(typeof request).toBe('string');
      const parsed = JSON.parse(request);
      expect(parsed.command).toBe('Apply_Login');
      expect(parsed.data).toEqual({ username: 'ED', password: '888888' });
    });
  });

  describe('parseLoginResponse', () => {
    it('应该正确解析新协议响应', async () => {
      const response = JSON.stringify({
        code: 200,
        message: 'success',
        data: {
          userId: '123',
          token: 'test-token',
        },
      });

      const result = await manager.parseLoginResponse(response);

      expect(result.success).toBe(true);
      expect(result.userId).toBe('123');
    });

    it('应该正确解析旧协议响应', async () => {
      manager.setProtocol('legacy');

      const response = JSON.stringify({
        data: {
          flag: 1,
          user_id: '123',
        },
      });

      const result = await manager.parseLoginResponse(response);

      expect(result.success).toBe(true);
      expect(result.userId).toBe('123');
    });
  });

  describe('convertSnakeToCamel', () => {
    it('应该正确转换蛇形命名到驼峰命名', () => {
      const input = {
        vehicle_id: 1,
        vehicle_name: 'Test',
        user_id: 123,
      };

      const result = manager.convertSnakeToCamel(input);

      expect(result).toEqual({
        vehicleId: 1,
        vehicleName: 'Test',
        userId: 123,
      });
    });

    it('应该处理嵌套对象', () => {
      const input = {
        vehicle_id: 1,
        user: {
          user_id: 123,
          user_name: 'Test',
        },
      };

      const result = manager.convertSnakeToCamel(input);

      expect(result).toEqual({
        vehicleId: 1,
        user: {
          userId: 123,
          userName: 'Test',
        },
      });
    });
  });

  describe('convertCamelToSnake', () => {
    it('应该正确转换驼峰命名到蛇形命名', () => {
      const input = {
        vehicleId: 1,
        vehicleName: 'Test',
        userId: 123,
      };

      const result = manager.convertCamelToSnake(input);

      expect(result).toEqual({
        vehicle_id: 1,
        vehicle_name: 'Test',
        user_id: 123,
      });
    });
  });

  describe('protocol auto-detection', () => {
    it('应该自动检测到新协议', () => {
      const detected = manager.autoDetectProtocol({ code: 200, message: 'success' });
      expect(detected).toBe('new');
    });

    it('应该回退到旧协议', () => {
      const detected = manager.autoDetectProtocol({ command: 'Vehicle_Request', data: {} });
      expect(detected).toBe('legacy');
    });
  });

  describe('protocol version', () => {
    it('应该返回当前协议版本', () => {
      expect(manager.getProtocol()).toBe('new');

      manager.setProtocol('legacy');
      expect(manager.getProtocol()).toBe('legacy');
    });
  });
});
