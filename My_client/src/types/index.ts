// 导出通用类型
export * from './common';

// 车辆信息（snake_case 风格 - 后端 API 格式）
// 注意：Vehicle 类型也在 ./vehicle.ts 中定义（camelCase 风格）
// 为避免冲突，使用 BackendVehicle 作为别名
export interface BackendVehicle {
  vehicle_id: number;
  license_plate: string;
  vehicle_type: string;
  vehicle_brand: string;
  vehicle_model: string;
  driver_id?: number;
  driver_name?: string;
  operation_status: string;
  latitude?: number;
  longitude?: number;
  speed?: number;
  direction?: number;
  update_time?: string;
}

// 订单信息（snake_case 风格 - 后端 API 格式）
// 注意：Order 类型也在 ./order.ts 中定义（camelCase 风格）
// 为避免冲突，使用 BackendOrder 作为别名
export interface BackendOrder {
  order_id: number;
  order_no: string;
  vehicle_id: number;
  driver_id?: number;
  customer_name: string;
  customer_phone: string;
  origin: string;
  destination: string;
  cargo_type: string;
  cargo_weight: number;
  cargo_volume: number;
  cargo_count: number;
  order_amount: number;
  order_status: number;
  departure_time?: string;
  arrival_time?: string;
  create_time?: string;
  update_time?: string;
}

// 司机信息
export interface Driver {
  driver_id: number;
  driver_name: string;
  phone_number?: string;
  license_number: string;
  email?: string;
  status: number;
  create_time?: string;
  update_time?: string;
}

// 告警信息
export interface Alarm {
  alert_id: number;
  vehicle_id: number;
  alert_type: string;
  priority: number;
  status: number;
  created_at: string;
  processed_at?: string;
}

// 轨迹信息
export interface Track {
  track_id: number;
  order_id: number;
  vehicle_id: number;
  track_time: string;
  latitude: number;
  longitude: number;
  address?: string;
  speed?: number;
  direction?: number;
  status: number;
  remark?: string;
  create_time?: string;
  created_at?: string;
}

// 部门信息
export interface Department {
  department_id: number;
  department_name: string;
  parent_department_id?: number;
  parent_department_name?: string;
  manager_id?: number;
  manager_name?: string;
  phone?: string;
  description?: string;
  create_time?: string;
  update_time?: string;
}

// 响应数据结构
export interface ApiResponse<T = unknown> {
  code: number;
  message: string;
  data: T;
}

// 分页请求参数
export interface PaginationParams {
  page: number;
  page_size: number;
  keyword?: string;
  [key: string]: unknown;
}

// 分页响应数据
export interface PaginationResponse<T> {
  list: T[];
  total: number;
  page: number;
  page_size: number;
  pages: number;
}

// WebSocket消息
export interface WebSocketMessage {
  type: string;
  data: unknown;
  timestamp?: number;
}

// 设备信息
export interface Device {
  device_id: string;
  device_name: string;
  device_type: string;
  device_model: string;
  manufacturer: string;
  serial_number: string;
  communication_type: string;
  sim_card_no?: string;
  ip_address?: string;
  port?: number;
  mac_address?: string;
  install_date?: string;
  install_address?: string;
  install_technician?: string;
  status: number;
  remark?: string;
  create_user_id: number;
  create_time?: string;
  update_time?: string;
  update_user_id?: number;
  vehicle_id?: number;
  last_heartbeat?: string;
}

// 统计数据
export interface Statistics {
  total_vehicles: number;
  online_vehicles: number;
  total_orders: number;
  pending_orders: number;
  total_alerts: number;
  unhandled_alerts: number;
  safety_ranking?: unknown[];
  total_weight?: number;
  active_vehicles?: number;
}

// 称重数据
export interface WeighingData {
  id: number;
  vehicle_id: number;
  weight: number;
  timestamp: string;
  [key: string]: unknown;
}

// 组织信息
export interface Organization {
  unit_id: string;
  name: string;
  type: string;
  parent_id?: number;
  description?: string;
  contact_person?: string;
  contact_phone?: string;
  status: string;
  create_time?: string;
  update_time?: string;
  [key: string]: unknown;
}

// 车辆组信息
export interface VehicleGroup {
  group_id: number;
  group_name: string;
  parent_id?: number;
  parent_name?: string;
  description?: string;
  vehicle_count: number;
  create_time?: string;
  update_time?: string;
  [key: string]: unknown;
}

// 节点信息
export interface Node {
  // 通用字段
  id?: number;
  name?: string;
  type?: string;
  latitude: number;
  longitude: number;
  description?: string;
  create_time?: string;
  update_time?: string;
  
  // 电子围栏字段
  fence_id?: number;
  fence_name?: string;
  fence_type?: string;
  center_latitude?: number;
  center_longitude?: number;
  radius?: number;
  polygon_points?: unknown;
  rectangle_bounds?: unknown;
  status?: string;
  
  // 位置字段
  location_id?: number;
  location_name?: string;
  address?: string;
  
  // 地点字段
  place_id?: number;
  place_name?: string;
  contact_person?: string;
  contact_phone?: string;
  contact_email?: string;
  
  // 路线字段
  route_id?: number;
  route_name?: string;
  start_point?: string;
  start_latitude?: number;
  start_longitude?: number;
  end_point?: string;
  end_latitude?: number;
  end_longitude?: number;
  waypoints?: unknown;
  distance?: number;
  estimated_duration?: number;
  
  [key: string]: unknown;
}

// 系统设置
export interface SystemSettings {
  id: number;
  key: string;
  value: string;
  [key: string]: unknown;
}

// 服务状态
export interface ServiceStatus {
  service: string;
  status: string;
  last_check: string;
  [key: string]: unknown;
}

// 本地车辆信息
export interface LocalVehicle {
  vehicle_id: number;
  vehicle_name: string;
  [key: string]: unknown;
}

// 本地称重数据
export interface LocalWeighingData {
  id: number;
  vehicle_id: number;
  weight: number;
  timestamp: string;
  [key: string]: unknown;
}

// 导出所有类型
export * from './order';
export * from './logistics';
export * from './vehicle';
export * from './organization';
export * from './monitoring';
export * from './reports';
export * from './status';
