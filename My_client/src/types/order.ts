// 订单相关类型定义

// 订单状态类型
export type OrderStatus = 'pending' | 'processing' | 'completed' | 'cancelled';

// 订单查询表单
export interface OrderQueryForm {
  orderNo: string;
  customer: string;
  status: string;
  dateRange: Date[];
}

// 订单项
export interface OrderItem {
  id?: number;
  name: string;
  quantity: number;
  unitPrice: number;
  total: number;
}

// 物流信息
export interface LogisticsInfo {
  company: string;
  trackingNo: string;
  status: string;
}

// 订单详情
export interface Order {
  id: number;
  orderNo: string;
  customer: string;
  contact: string;
  phone: string;
  status: OrderStatus;
  totalAmount: number;
  createTime: string;
  paid: boolean;
  remark?: string;
  items?: OrderItem[];
  logistics?: LogisticsInfo;
  [key: string]: unknown;
}

// 订单表单
export interface OrderForm {
  id: number;
  orderNo: string;
  customer: string;
  contact: string;
  phone: string;
  status: OrderStatus;
  paid: boolean;
  remark: string;
  [key: string]: unknown;
}

// 订单API响应
export interface OrderResponse {
  code: number;
  message: string;
  data: Order;
}

// 订单列表API响应
export interface OrderListResponse {
  code: number;
  message: string;
  items: Order[];
  total: number;
  page: number;
  pageSize: number;
}

// 订单状态映射
export interface StatusMap {
  pending: string;
  processing: string;
  completed: string;
  cancelled: string;
}

// 订单状态标签类型映射
export interface StatusTagTypeMap {
  pending: string;
  processing: string;
  completed: string;
  cancelled: string;
}
