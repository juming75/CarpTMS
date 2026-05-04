// Ansible 类型定义

// 主机信息
export interface Host {
  id: string;
  name: string;
  ansible_host: string;
  ansible_port?: number;
  ansible_user: string;
  groups: string[];
  variables: Record<string, unknown>;
  status: HostStatus;
}

export type HostStatus = 'online' | 'offline' | 'unreachable' | 'unknown';

// 服务器组
export interface ServerGroup {
  id: string;
  name: string;
  hosts: Host[];
  variables: Record<string, unknown>;
}

// Playbook 信息
export interface PlaybookInfo {
  name: string;
  path: string;
  description: string;
  category: string;
}

// Playbook 执行请求
export interface PlaybookExecuteRequest {
  playbook: string;
  inventory: string;
  extra_vars?: Record<string, unknown>;
  limit?: string;
  check_mode?: boolean;
  tags?: string[];
}

// 任务执行结果
export interface TaskResult {
  host: string;
  task_name: string;
  status: TaskStatus;
  changed: boolean;
  output?: string;
  error?: string;
  duration_ms: number;
}

export type TaskStatus = 'ok' | 'changed' | 'failed' | 'skipped' | 'unreachable';

// 执行状态
export type ExecutionStatus = 'pending' | 'running' | 'success' | 'failed' | 'cancelled' | 'timeout';

// 执行摘要
export interface ExecutionSummary {
  total: number;
  ok: number;
  changed: number;
  failed: number;
  skipped: number;
  unreachable: number;
}

// Playbook 执行结果
export interface PlaybookResult {
  execution_id: string;
  playbook: string;
  status: ExecutionStatus;
  started_at: string;
  finished_at?: string;
  results: TaskResult[];
  summary: ExecutionSummary;
  logs: string;
}

// 快速命令
export interface QuickCommand {
  hosts: string;
  module: string;
  args: string;
  inventory: string;
}

// 执行历史记录
export interface ExecutionHistory {
  id: string;
  user_id: string;
  user_name: string;
  playbook_name: string;
  status: ExecutionStatus;
  hosts_count: number;
  started_at: string;
  finished_at?: string;
  summary?: ExecutionSummary;
}

// 库存源
export interface InventorySource {
  id: string;
  name: string;
  path: string;
  source_type: 'file' | 'dynamic' | 'manual';
  last_updated: string;
}
