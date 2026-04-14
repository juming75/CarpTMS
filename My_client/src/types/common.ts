// 通用类型定义

// 通用对象类型
export interface GenericObject {
  [key: string]: unknown;
}

// 通用数组类型
export type GenericArray = unknown[];

// 通用回调函数类型
export type GenericCallback = (...args: unknown[]) => void;

// 通用事件类型
export interface GenericEvent {
  type: string;
  target: unknown;
  [key: string]: unknown;
}

// 通用选项类型
export interface GenericOption {
  value: string | number;
  label: string;
  [key: string]: unknown;
}

// 通用分页参数
export interface PaginationParams {
  page: number;
  page_size: number;
  [key: string]: unknown;
}

// 通用API响应
export interface ApiResponse<T = unknown> {
  code: number;
  message: string;
  data?: T;
  [key: string]: unknown;
}

// 通用分页响应
export interface PaginatedResponse<T = unknown> extends ApiResponse {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}
