// 监控相关类型定义

// 监控状态类型
export type MonitorStatus = 'normal' | 'warning' | 'alert' | 'critical';

// 警报类型
export type AlarmType = 'speed' | 'location' | 'engine' | 'fuel' | 'temperature' | 'battery';

// 监控查询表单
export interface MonitorQueryForm {
  vehicleId: string;
  dateRange: Date[];
  status: string;
  alarmType: string;
}

// 车辆位置
export interface VehiclePosition {
  id: number;
  vehicleId: number;
  latitude: number;
  longitude: number;
  speed: number;
  direction: number;
  altitude: number;
  timestamp: string;
  status: MonitorStatus;
  [key: string]: unknown;
}

// 警报信息
export interface Alarm {
  id: number;
  vehicleId: number;
  vehiclePlate: string;
  type: AlarmType;
  message: string;
  level: MonitorStatus;
  timestamp: string;
  isProcessed: boolean;
  [key: string]: unknown;
}

// 监控数据
export interface MonitorData {
  vehicleId: number;
  vehiclePlate: string;
  position: VehiclePosition;
  alarms: Alarm[];
  status: MonitorStatus;
  [key: string]: unknown;
}

// 监控API响应
export interface MonitorResponse {
  code: number;
  message: string;
  data: MonitorData;
}

// 监控列表API响应
export interface MonitorListResponse {
  code: number;
  message: string;
  items: MonitorData[];
  total: number;
  page: number;
  pageSize: number;
}

// 警报列表API响应
export interface AlarmListResponse {
  code: number;
  message: string;
  items: Alarm[];
  total: number;
  page: number;
  pageSize: number;
}

// 监控状态映射
export interface MonitorStatusMap {
  normal: string;
  warning: string;
  alert: string;
  critical: string;
}

// 警报类型映射
export interface AlarmTypeMap {
  speed: string;
  location: string;
  engine: string;
  fuel: string;
  temperature: string;
  battery: string;
}
