// 全局窗口对象类型定义

import type { CommunicationMessage } from '../services/unifiedCommunicationService';

// 监控服务类型
export interface MonitoringService {
  init(): void;
  recordPerformance(metric: string, value: number, metadata?: Record<string, unknown>): void;
  recordError(error: Error | unknown, context?: string): void;
  recordUserAction(action: string, metadata?: Record<string, unknown>): void;
  getMonitorData(): unknown[];
  cleanup(): void;
}

// 新服务器服务类型
export interface NewServerService {
  connect(): Promise<boolean>;
  send(message: CommunicationMessage): Promise<unknown>;
  on(event: string, handler: Function): void;
  off(event: string, handler: Function): void;
  switchProtocol(protocol: 'tcp' | 'websocket'): Promise<boolean>;
  getCurrentProtocol(): string;
  isConnected(): boolean;
  getStats(): unknown;
  disconnect(): Promise<void>;
}

// 旧服务器服务类型
export interface LegacyServerService {
  connect(): Promise<boolean>;
  send(data: unknown): Promise<unknown>;
  on(event: string, handler: Function): void;
  off(event: string, handler: Function): void;
  isConnected(): Promise<boolean>;
  getStats(): unknown;
  disconnect(): Promise<void>;
}

// 同步服务类型
export interface SyncService {
  initialize(): Promise<void>;
  triggerSync(): Promise<void>;
  destroy(): void;
  [key: string]: unknown;
}

// 地图API类型声明
interface AMap {
  Map: unknown;
  LngLat: unknown;
  Icon: unknown;
  Marker: unknown;
  Polyline: unknown;
  Size: unknown;
}

interface BMap {
  Map: unknown;
  Point: unknown;
  Marker: unknown;
  Polyline: unknown;
  InfoWindow: unknown;
}

// 地图实例类型
export interface MapInstanceType {
  setCenter: (center: [number, number] | unknown) => void;
  add: (overlay: unknown) => void;
  addOverlay: (overlay: unknown) => void;
  openInfoWindow: (infoWindow: unknown, point: unknown) => void;
  closeInfoWindow: () => void;
  setViewport: (points: unknown[]) => void;
  setFitView: (overlay: unknown) => void;
  on: (event: string, callback: () => void) => void;
  centerAndZoom: (point: unknown, zoom: number) => void;
  resize: () => void;
}

// 扩展全局Window接口
declare global {
  interface Window {
    $ws?: unknown;
    $sync?: SyncService;
    $monitoring?: MonitoringService;
    $newServerService?: NewServerService;
    $legacyServerService?: LegacyServerService;
    AMap?: AMap;
    BMap?: BMap;
  }
}
