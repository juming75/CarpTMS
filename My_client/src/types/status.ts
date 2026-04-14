// 状态查询相关类型定义

// 状态类型
export type StatusType = 'vehicle' | 'order' | 'driver' | 'device';

// 状态查询表单
export interface StatusQueryForm {
  type: StatusType;
  id: string;
  dateRange: Date[];
  status: string;
}

// 车辆状态数据
export interface VehicleStatusData {
  vehicleId: number;
  licensePlate: string;
  status: string;
  position: {
    latitude: number;
    longitude: number;
    speed: number;
    direction: number;
  };
  lastUpdate: string;
  [key: string]: unknown;
}

// 订单状态数据
export interface OrderStatusData {
  orderId: number;
  orderNo: string;
  status: string;
  logisticsInfo: {
    trackingNo: string;
    carrier: string;
    status: string;
  };
  lastUpdate: string;
  [key: string]: unknown;
}

// 司机状态数据
export interface DriverStatusData {
  driverId: number;
  name: string;
  phone: string;
  status: string;
  currentVehicle?: {
    id: number;
    licensePlate: string;
  };
  lastUpdate: string;
  [key: string]: unknown;
}

// 设备状态数据
export interface DeviceStatusData {
  deviceId: number;
  serialNo: string;
  status: string;
  signalStrength: number;
  batteryLevel: number;
  lastUpdate: string;
  [key: string]: unknown;
}

// 状态数据联合类型
export type StatusData = VehicleStatusData | OrderStatusData | DriverStatusData | DeviceStatusData;

// 状态API响应
export interface StatusResponse {
  code: number;
  message: string;
  data: StatusData;
}

// 状态历史API响应
export interface StatusHistoryResponse {
  code: number;
  message: string;
  items: StatusData[];
  total: number;
  page: number;
  pageSize: number;
}
