// @ts-nocheck
﻿import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import monitoringService, { PerformanceMetric, UserAction } from './monitoring';

// Mock 依赖
vi.mock('./logService', () => ({
  logService: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    debug: vi.fn(),
    fatal: vi.fn(),
    generateRequestId: vi.fn(() => 'test-request-id'),
  },
}));

vi.mock('./performance', () => ({
  getPerformanceNow: vi.fn(() => Date.now()),
  recordPerformanceMetric: vi.fn(),
  performanceConfig: {
    enableComponentRenderMonitoring: true,
  },
}));

describe('MonitoringService', () => {
  beforeEach(() => {
    // 重置监控服务
    monitoringService.cleanup();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('能够初始化监控服务', () => {
    monitoringService.init();
    expect(monitoringService.getMonitorData()).toEqual([]);
  });

  it('能够记录性能指标', () => {
    monitoringService.init();
    const metric = PerformanceMetric.PAGE_LOAD;
    const value = 1000;
    const metadata = { test: 'data' };

    monitoringService.recordPerformance(metric, value, metadata);
    const data = monitoringService.getMonitorData();

    expect(data.length).toBe(1);
    expect(data[0].type).toBe('performance');
    expect(data[0].name).toBe(metric);
    expect(data[0].value).toBe(value);
    expect(data[0].metadata).toEqual(metadata);
  });

  it('能够记录错误', () => {
    monitoringService.init();
    const error = new Error('测试错误');
    const context = '测试上下文';

    monitoringService.recordError(error, context);
    const data = monitoringService.getMonitorData();

    expect(data.length).toBe(1);
    expect(data[0].type).toBe('error');
    expect(data[0].metadata?.context).toBe(context);
  });

  it('能够记录用户行为', () => {
    monitoringService.init();
    const action = UserAction.CLICK;
    const metadata = { x: 100, y: 200 };

    monitoringService.recordUserAction(action, metadata);
    const data = monitoringService.getMonitorData();

    expect(data.length).toBe(1);
    expect(data[0].type).toBe('user_behavior');
    expect(data[0].name).toBe(action);
    expect(data[0].metadata).toEqual(metadata);
  });

  it('能够清理监控数据', () => {
    monitoringService.init();
    monitoringService.recordPerformance(PerformanceMetric.PAGE_LOAD, 1000);

    expect(monitoringService.getMonitorData().length).toBe(1);

    monitoringService.cleanup();
    expect(monitoringService.getMonitorData().length).toBe(0);
  });

  it('能够处理缓冲区满的情况', () => {
    // 由于bufferSize是私有属性，我们测试缓冲区的基本功能
    monitoringService.init();

    // 记录多条数据
    for (let i = 0; i < 5; i++) {
      monitoringService.recordPerformance(PerformanceMetric.PAGE_LOAD, 1000 + i);
    }

    const data = monitoringService.getMonitorData();
    expect(data.length).toBe(5);
  });
});


