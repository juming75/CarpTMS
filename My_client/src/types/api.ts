// API相关类型定义

// 通用API响应
interface ApiResponse<T = unknown> {
  code: number;
  message: string;
  data?: T;
  [key: string]: unknown;
}

// 订单API响应
export interface OrderApiResponse extends ApiResponse {
  data?: unknown;
}

// 物流API响应
export interface LogisticsApiResponse extends ApiResponse {
  data?: unknown;
}

// 车辆API响应
export interface VehicleApiResponse extends ApiResponse {
  data?: unknown;
}

// 用户API响应
export interface UserApiResponse extends ApiResponse {
  data?: unknown;
}

// 部门API响应
export interface DepartmentApiResponse extends ApiResponse {
  data?: unknown;
}

// 组织单元API响应
export interface OrganizationUnitApiResponse extends ApiResponse {
  data?: unknown;
}

// 角色API响应
export interface RoleApiResponse extends ApiResponse {
  data?: unknown;
}

// 车辆团队API响应
export interface VehicleTeamApiResponse extends ApiResponse {
  data?: unknown;
}

// 设备API响应
export interface DeviceApiResponse extends ApiResponse {
  data?: unknown;
}

// 监控API响应
export interface MonitorApiResponse extends ApiResponse {
  data?: unknown;
}

// 报表API响应
export interface ReportApiResponse extends ApiResponse {
  data?: unknown;
}

// 状态查询API响应
export interface StatusQueryApiResponse extends ApiResponse {
  data?: unknown;
}

// 登录用户数据
export interface LoginUserData {
  token: string;
  user: {
    user_id: number;
    username: string;
    real_name: string;
    role: string;
    [key: string]: unknown;
  };
}

// 登录API响应
export interface LoginResponse extends ApiResponse {
  data?: LoginUserData;
}

// 通用分页请求参数
export interface PaginationParams {
  page: number;
  page_size: number;
  [key: string]: unknown;
}

// 通用ID请求参数
export interface IdParams {
  id: number;
}

// 通用搜索参数
export interface SearchParams {
  keyword: string;
  [key: string]: unknown;
}
