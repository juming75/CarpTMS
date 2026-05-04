// Ansible API 服务

import api from '../index';
import type {
  Host,
  ServerGroup,
  PlaybookInfo,
  PlaybookExecuteRequest,
  PlaybookResult,
  QuickCommand,
  TaskResult,
  ExecutionHistory,
  InventorySource,
} from '@/types/ansible';

// API 基础路径
const ANSIBLE_API = '/api/ansible';

/**
 * 健康检查
 */
export const checkAnsibleHealth = async (): Promise<{ status: string; message: string }> => {
  const response = await api.get(`${ANSIBLE_API}/health`);
  return response.data;
};

/**
 * 获取可用 Playbook 列表
 */
export const listPlaybooks = async (): Promise<PlaybookInfo[]> => {
  const response = await api.get(`${ANSIBLE_API}/playbooks`);
  return response.data.data || [];
};

/**
 * 执行 Playbook
 */
export const executePlaybook = async (request: PlaybookExecuteRequest): Promise<PlaybookResult> => {
  const response = await api.post(`${ANSIBLE_API}/playbook/execute`, request);
  return response.data.data;
};

/**
 * 获取执行状态
 */
export const getExecutionStatus = async (executionId: string): Promise<PlaybookResult> => {
  const response = await api.get(`${ANSIBLE_API}/playbook/${executionId}/status`);
  return response.data.data;
};

/**
 * 获取主机列表
 */
export const listHosts = async (): Promise<Host[]> => {
  const response = await api.get(`${ANSIBLE_API}/hosts`);
  return response.data.data || [];
};

/**
 * 获取服务器组列表
 */
export const listGroups = async (): Promise<ServerGroup[]> => {
  const response = await api.get(`${ANSIBLE_API}/groups`);
  return response.data.data || [];
};

/**
 * 执行快速命令
 */
export const executeCommand = async (command: QuickCommand): Promise<TaskResult[]> => {
  const response = await api.post(`${ANSIBLE_API}/command`, command);
  return response.data.data || [];
};

/**
 * Ping 主机
 */
export const pingHosts = async (hosts: string, inventory: string): Promise<TaskResult[]> => {
  const response = await api.post(`${ANSIBLE_API}/ping`, { hosts, inventory });
  return response.data.data || [];
};

/**
 * 获取库存信息
 */
export const getInventory = async (): Promise<InventorySource> => {
  const response = await api.get(`${ANSIBLE_API}/inventory`);
  return response.data.data;
};

/**
 * 获取执行历史
 */
export const getExecutionHistory = async (): Promise<ExecutionHistory[]> => {
  const response = await api.get(`${ANSIBLE_API}/history`);
  return response.data.data || [];
};
