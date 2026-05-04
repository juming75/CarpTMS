// 报表相关类型定义

// 报表类型
export type ReportType = 'sales' | 'logistics' | 'vehicle' | 'finance';

// 报表查询表单
export interface ReportQueryForm {
  startDate: string;
  endDate: string;
  reportType: ReportType;
  filters: Record<string, unknown>;
}

// 销售报表数据
export interface SalesReport {
  date: string;
  orderCount: number;
  totalAmount: number;
  averageAmount: number;
  [key: string]: unknown; // 允许其他属性
}

// 物流报表数据
export interface LogisticsReport {
  date: string;
  deliveryCount: number;
  totalDistance: number;
  averageTime: number;
  [key: string]: unknown; // 允许其他属性
}

// 车辆报表数据
export interface VehicleReport {
  vehicleId: number;
  licensePlate: string;
  totalDistance: number;
  fuelConsumption: number;
  maintenanceCount: number;
  [key: string]: unknown; // 允许其他属性
}

// 财务报表数据
export interface FinanceReport {
  date: string;
  income: number;
  expense: number;
  profit: number;
  [key: string]: unknown; // 允许其他属性
}

// 报表API响应
export interface ReportResponse {
  code: number;
  message: string;
  data: unknown; // 暂时保留unknown，后续会替换为具体类型
}

// 报表列表API响应
export interface ReportListResponse<T = unknown> {
  code: number;
  message: string;
  items: T[];
  total: number;
  [key: string]: unknown; // 允许其他属性
}
