// 统计数据服务
import api from '@/api/index';

export interface Statistics {
  totalVehicles?: number;
  onlineVehicles?: number;
  offlineVehicles?: number;
  alarmVehicles?: number;
  todayMileage?: number;
  todayAlarm?: number;
  hourlyTrend?: HourlyTrend[];
  alarmStats?: AlarmStat[];
  sensorTrend?: SensorTrend;
}

export interface HourlyTrend {
  hour: string;
  online: number;
  alarm: number;
}

export interface AlarmStat {
  type: string;
  count: number;
}

export interface SensorTrend {
  fuel?: SensorDataPoint[];
  waterTemp?: SensorDataPoint[];
  engineRpm?: SensorDataPoint[];
  loadWeight?: SensorDataPoint[];
}

export interface SensorDataPoint {
  time: string;
  value: number;
}

/**
 * 获取统计数据
 */
export function getStatistics(params?: { startTime?: string; endTime?: string; groupId?: number }) {
  return api.get('/api/statistics', { params });
}

/**
 * 获取车辆状态分布
 */
export function getVehicleStatusDistribution() {
  return api.get('/api/statistics/vehicle-status');
}

/**
 * 获取车辆分组统计
 */
export function getVehicleGroupStats() {
  return api.get('/api/statistics/vehicle-groups');
}

/**
 * 获取车辆类型分布
 */
export function getVehicleTypeDistribution() {
  return api.get('/api/statistics/vehicle-types');
}

/**
 * 获取报警统计
 */
export function getAlarmStats(params?: { startTime?: string; endTime?: string; type?: string }) {
  return api.get('/api/statistics/alarm-stats', { params });
}

/**
 * 获取24小时趋势
 */
export function getHourlyTrend(params?: { startTime?: string; endTime?: string }) {
  return api.get('/api/statistics/hourly-trend', { params });
}

/**
 * 获取传感器数据趋势
 */
export function getSensorTrend(params: { vehicleId: number; sensorType: string; startTime: string; endTime: string }) {
  return api.get('/api/statistics/sensor-trend', { params });
}

/**
 * 获取里程统计
 */
export function getMileageStats(params?: { vehicleId?: number; startTime?: string; endTime?: string }) {
  return api.get('/api/statistics/mileage-stats', { params });
}

/**
 * 获取油耗统计
 */
export function getFuelStats(params?: { vehicleId?: number; startTime?: string; endTime?: string }) {
  return api.get('/api/statistics/fuel-stats', { params });
}

/**
 * 导出统计报表
 */
export function exportStatistics(params: { startTime: string; endTime: string; type: string }) {
  return api.post('/api/statistics/export', params);
}


