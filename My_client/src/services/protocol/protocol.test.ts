// @ts-nocheck
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
        Command: 'Login',
        Username: 'ED',
        Password: '888888',
        TerminalID: '1',
        Timestamp: expect.any(String),
      });
    });

    it('应该处理空用户名和密码', () => {
      const request = adapter.buildLoginRequest('', '');

      expect(request).toEqual({
        Command: 'Login',
        Username: '',
        Password: '',
        TerminalID: '1',
        Timestamp: expect.any(String),
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

      expect(request).toEqual({
        Command: 'AddVehicle',
        Vehicle: {
          VehicleName: 'Test Vehicle',
          DeviceID: 'DEV001',
          OwnNo: 'OWN001',
        },
        Timestamp: expect.any(String),
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

      expect(request).toEqual({
        url: 'http://127.0.0.1:9808/api/login',
        options: {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            username: 'ED',
            password: '888888',
            terminal_id: '1',
          }),
        },
      });
    });
  });

  describe('parseLoginResponse', () => {
    it('应该正确解析成功登录响应', () => {
      const response = {
        code: 200,
        message: 'success',
        data: {
          userId: 123,
          token: 'test-token',
          user: {
            username: 'admin',
            name: 'Admin User',
          },
        },
      };

      const result = adapter.parseLoginResponse(response);

      expect(result.success).toBe(true);
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
});

describe('ProtocolManager', () => {
  let manager;

  beforeEach(() => {
    // 重置协议管理器
    manager = protocolManager;
    manager.setProtocol('new', 'http://127.0.0.1:9808');
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('buildLoginRequest', () => {
    it('应该使用新协议构建登录请求', () => {
      const request = manager.buildLoginRequest('ED', '888888');

      expect(request).toHaveProperty('url');
      expect(request.url).toContain('/api/login');
      expect(request.options).toHaveProperty('method', 'POST');
    });

    it('应该使用旧协议构建登录请求', () => {
      manager.setProtocol('legacy');

      const request = manager.buildLoginRequest('ED', '888888');

      expect(request).toEqual({
        Command: 'Login',
        Username: 'ED',
        Password: '888888',
        TerminalID: '1',
        Timestamp: expect.any(String),
      });
    });
  });

  describe('parseLoginResponse', () => {
    it('应该正确解析新协议响应', () => {
      const response = {
        success: true,
        data: {
          userId: 123,
          token: 'test-token',
        },
      };

      const result = manager.parseLoginResponse(response);

      expect(result.success).toBe(true);
      expect(result.userId).toBe(123);
    });

    it('应该正确解析旧协议响应', () => {
      manager.setProtocol('legacy');

      const response = {
        Status: 'Success',
        UserID: '123',
      };

      const result = manager.parseLoginResponse(response);

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
    it('应该自动检测到新协议', async () => {
      const detected = manager.autoDetectProtocol({ code: 200, message: 'success' });
      expect(detected).toBe('new');
    });

    it('应该回退到旧协议', async () => {
      const detected = manager.autoDetectProtocol({ command: 'Vehicle_Request' });
      expect(detected).toBe('old');
    });
  });
});


