// 位置相关类型定义

// 电子围栏表单数据
export interface FenceFormData {
  fence_name: string;
  fence_type: 'circle' | 'polygon' | 'rectangle';
  center_latitude: number | null;
  center_longitude: number | null;
  radius: number | null;
  status: 'active' | 'inactive';
  description: string;
  [key: string]: unknown;
}

// 位置表单数据
export interface PositionFormData {
  location_name: string;
  latitude: number | null;
  longitude: number | null;
  address: string;
  description: string;
  [key: string]: unknown;
}

// 地点表单数据
export interface PlaceFormData {
  place_name: string;
  address: string;
  contact_person: string;
  contact_phone: string;
  contact_email: string;
  latitude: number | null;
  longitude: number | null;
  description: string;
  [key: string]: unknown;
}

// 路线表单数据
export interface RouteFormData {
  route_name: string;
  start_point: string;
  start_latitude: number | null;
  start_longitude: number | null;
  end_point: string;
  end_latitude: number | null;
  end_longitude: number | null;
  waypoints: any | null;
  distance: number | null;
  estimated_duration: number | null;
  description: string;
  [key: string]: unknown;
}

// 表单数据联合类型
export type LocationFormData = FenceFormData | PositionFormData | PlaceFormData | RouteFormData;

// 类型映射
export interface TypeMap {
  fence: string;
  position: string;
  place: string;
  route: string;
}
