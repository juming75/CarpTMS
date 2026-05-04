// 物流相关类型定义

// 物流状态类型
export type LogisticsStatus = 'pending' | 'in_transit' | 'delivered' | 'cancelled';

// 物流查询表单
export interface LogisticsQueryForm {
  trackingNo: string;
  carrier: string;
  status: string;
  dateRange: Date[];
}

// 物流详情
export interface Logistics {
  id: number;
  trackingNo: string;
  carrier: string;
  status: LogisticsStatus;
  origin: string;
  destination: string;
  estimatedDelivery: string;
  actualDelivery?: string;
  createTime: string;
  updateTime: string;
  [key: string]: unknown;
}

// 物流表单
export interface LogisticsForm {
  id: number;
  trackingNo: string;
  carrier: string;
  status: LogisticsStatus;
  origin: string;
  destination: string;
  estimatedDelivery: string;
  actualDelivery?: string;
  [key: string]: unknown;
}

// 物流API响应
export interface LogisticsResponse {
  code: number;
  message: string;
  data: Logistics;
}

// 物流列表API响应
export interface LogisticsListResponse {
  code: number;
  message: string;
  items: Logistics[];
  total: number;
  page: number;
  pageSize: number;
}

// 物流状态映射
export interface LogisticsStatusMap {
  pending: string;
  in_transit: string;
  delivered: string;
  cancelled: string;
}

// 物流状态标签类型映射
export interface LogisticsStatusTagTypeMap {
  pending: string;
  in_transit: string;
  delivered: string;
  cancelled: string;
}
