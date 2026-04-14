// 车辆相关类型定义

// 车辆状态类型
export type VehicleStatus = 'idle' | 'in_service' | 'maintenance' | 'out_of_service';

// 车辆类型
export type VehicleType = 'truck' | 'van' | 'car' | 'suv';

// 车辆查询表单
export interface VehicleQueryForm {
  plateNo: string;
  model: string;
  status: string;
  type: string;
}

// 车辆详情
export interface Vehicle {
  id: number;
  plateNo: string;
  model: string;
  type: VehicleType;
  status: VehicleStatus;
  vin: string;
  engineNo: string;
  purchaseDate: string;
  lastMaintenanceDate: string;
  nextMaintenanceDate: string;
  mileage: number;
  fuelType: string;
  capacity: number;
  driverId?: number;
  driverName?: string;
  [key: string]: unknown;
}

// 车辆表单
export interface VehicleForm {
  id: number;
  plateNo: string;
  model: string;
  type: VehicleType;
  status: VehicleStatus;
  vin: string;
  engineNo: string;
  purchaseDate: string;
  lastMaintenanceDate: string;
  nextMaintenanceDate: string;
  mileage: number;
  fuelType: string;
  capacity: number;
  driverId?: number;
  [key: string]: unknown;
}

// 车辆API响应
export interface VehicleResponse {
  code: number;
  message: string;
  data: Vehicle;
}

// 车辆列表API响应
export interface VehicleListResponse {
  code: number;
  message: string;
  items: Vehicle[];
  total: number;
  page: number;
  pageSize: number;
}

// 车辆状态映射
export interface VehicleStatusMap {
  idle: string;
  in_service: string;
  maintenance: string;
  out_of_service: string;
}

// 车辆类型映射
export interface VehicleTypeMap {
  truck: string;
  van: string;
  car: string;
  suv: string;
}
