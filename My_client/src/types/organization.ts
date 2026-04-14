// 组织相关类型定义

// 用户角色类型
export type UserRole = 'admin' | 'manager' | 'employee' | 'driver';

// 用户状态类型
export type UserStatus = 'active' | 'inactive' | 'suspended';

// 部门状态类型
export type DepartmentStatus = 'active' | 'inactive';

// 组织单元类型
export type OrganizationUnitType = 'company' | 'department' | 'team';

// 用户查询表单
export interface UserQueryForm {
  username: string;
  name: string;
  role: string;
  status: string;
  departmentId: string;
}

// 用户详情
export interface User {
  id: number;
  username: string;
  name: string;
  email: string;
  phone: string;
  role: UserRole;
  status: UserStatus;
  departmentId: number;
  departmentName?: string;
  createTime: string;
  lastLoginTime?: string;
  [key: string]: unknown;
}

// 用户表单
export interface UserForm {
  id: number;
  username: string;
  name: string;
  email: string;
  phone: string;
  password?: string;
  role: UserRole;
  status: UserStatus;
  departmentId: number;
  [key: string]: unknown;
}

// 部门查询表单
export interface DepartmentQueryForm {
  name: string;
  status: string;
  parentId: string;
}

// 部门详情
export interface Department {
  id: number;
  name: string;
  code: string;
  status: DepartmentStatus;
  parentId: number;
  parentName?: string;
  description?: string;
  createTime: string;
  [key: string]: unknown;
}

// 部门表单
export interface DepartmentForm {
  id: number;
  name: string;
  code: string;
  status: DepartmentStatus;
  parentId: number;
  description?: string;
  [key: string]: unknown;
}

// 组织单元查询表单
export interface OrganizationUnitQueryForm {
  name: string;
  type: string;
  status: string;
}

// 组织单元详情
export interface OrganizationUnit {
  id: number;
  name: string;
  type: OrganizationUnitType;
  code: string;
  parentId: number;
  parentName?: string;
  description?: string;
  createTime: string;
  [key: string]: unknown;
}

// 组织单元表单
export interface OrganizationUnitForm {
  id: number;
  name: string;
  type: OrganizationUnitType;
  code: string;
  parentId: number;
  description?: string;
  [key: string]: unknown;
}

// 角色查询表单
export interface RoleQueryForm {
  name: string;
  code: string;
}

// 角色详情
export interface Role {
  id: number;
  name: string;
  code: string;
  description?: string;
  permissions: string[];
  createTime: string;
  [key: string]: unknown;
}

// 角色表单
export interface RoleForm {
  id: number;
  name: string;
  code: string;
  description?: string;
  permissions: string[];
  [key: string]: unknown;
}

// 车辆团队查询表单
export interface VehicleTeamQueryForm {
  name: string;
  leaderId: string;
}

// 车辆团队详情
export interface VehicleTeam {
  id: number;
  name: string;
  leaderId: number;
  leaderName?: string;
  description?: string;
  vehicleIds: number[];
  createTime: string;
  [key: string]: unknown;
}

// 车辆团队表单
export interface VehicleTeamForm {
  id: number;
  name: string;
  leaderId: number;
  description?: string;
  vehicleIds: number[];
  [key: string]: unknown;
}

// 通用API响应
export interface OrganizationApiResponse {
  code: number;
  message: string;
  data?: unknown;
}

// 通用列表响应
export interface OrganizationListResponse<T = unknown> {
  code: number;
  message: string;
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}
