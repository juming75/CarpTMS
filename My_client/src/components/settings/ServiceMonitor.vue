<template>
  <!-- ====== 后端服务状态概览 ====== -->
  <el-card style="margin-bottom: 20px">
    <template #header>
      <div class="card-header">
        <span>后端服务状态</span>
        <div class="header-actions">
          <el-tag v-if="responseTimeMs > 0 && currentStatus !== 'down'" size="small" type="info" effect="plain">
            {{ responseTimeMs }}ms
          </el-tag>
          <el-button type="primary" size="small" @click="manualCheck" :loading="isChecking">
            <el-icon><Refresh /></el-icon> 立即检测
          </el-button>
        </div>
      </div>
    </template>

    <div class="backend-overview">
      <!-- 整体状态 -->
      <div class="status-row">
        <span class="label">整体状态</span>
        <el-tag :type="statusTagType" size="large" effect="dark">
          <el-icon style="margin-right: 4px">
            <CircleCheckFilled v-if="currentStatus === 'healthy'" />
            <WarningFilled v-else-if="currentStatus === 'degraded'" />
            <CircleCloseFilled v-else-if="currentStatus === 'down'" />
            <Loading v-else />
          </el-icon>
          {{ statusLabel }}
        </el-tag>
      </div>

      <!-- 响应时间 -->
      <div class="status-row">
        <span class="label">响应时间</span>
        <span class="value" :class="{ warning: responseTimeMs > 1000, danger: responseTimeMs > 3000 }">
          {{ responseTimeMs > 0 ? `${responseTimeMs} ms` : '-' }}
        </span>
      </div>

      <!-- 检测时间 -->
      <div class="status-row">
        <span class="label">上次检测</span>
        <span class="value">{{ lastCheckTime }}</span>
      </div>

      <!-- 网络状态 -->
      <div class="status-row">
        <span class="label">网络状态</span>
        <span class="value" :class="{ offline: isOffline }">{{ isOffline ? '离线' : '在线' }}</span>
      </div>

      <!-- 依赖状态 -->
      <div v-if="lastResult?.details" class="deps-row">
        <span class="label">依赖服务</span>
        <div class="deps-list">
          <el-tag :type="getDepTagType(lastResult.details.database)" size="small" effect="plain">
            <el-icon><Coin /></el-icon> 数据库: {{ lastResult.details.database }}
          </el-tag>
          <el-tag :type="getDepTagType(lastResult.details.redis)" size="small" effect="plain">
            <el-icon><Connection /></el-icon> Redis: {{ lastResult.details.redis }}
          </el-tag>
          <el-tag v-if="lastResult.details.hostname" size="small" effect="plain">
            <el-icon><Monitor /></el-icon> {{ lastResult.details.hostname }}
          </el-tag>
        </div>
      </div>

      <!-- 自愈状态 -->
      <div v-if="selfHealInfo" class="status-row">
        <span class="label">自愈引擎</span>
        <el-tooltip :content="`PID: ${selfHealInfo.process?.pid || '-'} | 运行: ${selfHealInfo.process?.uptime_human || '-'}`" placement="top">
          <el-tag :type="selfHealInfo.status === 'ok' ? 'success' : 'warning'" size="small">
            运行中 (PID: {{ selfHealInfo.process?.pid || '-' }})
          </el-tag>
        </el-tooltip>
      </div>

      <!-- 错误信息 -->
      <el-alert v-if="lastResult?.error" :title="lastResult.error" type="error" :closable="false" show-icon style="margin-top: 8px" />
    </div>
  </el-card>

  <!-- ====== WebSocket 连接监控（从原 WebSocketMonitor 集成） ====== -->
  <el-card style="margin-bottom: 20px">
    <template #header>
      <div class="card-header">
        <span>WebSocket 连接监控</span>
        <el-button type="primary" size="small" @click="wsRefreshStats" :loading="false">
          <el-icon><Refresh /></el-icon> 刷新统计
        </el-button>
      </div>
    </template>

    <div class="websocket-section">
      <!-- 连接状态卡片 -->
      <div class="ws-status-grid">
        <div class="ws-stat-box">
          <span class="box-label">连接状态</span>
          <span :class="['box-value', wsStatusClass]">{{ wsStatusText }}</span>
        </div>
        <div class="ws-stat-box">
          <span class="box-label">协议类型</span>
          <span class="box-value text-muted">{{ wsStats.protocol || '-' }}</span>
        </div>
        <div class="ws-stat-box">
          <span class="box-label">服务器地址</span>
          <span class="box-value text-muted font-mono">{{ wsServerUrl || '-' }}</span>
        </div>
        <div class="ws-stat-box">
          <span class="box-label">重连次数</span>
          <span class="box-value accent">{{ wsStats.reconnectAttempts }}</span>
        </div>
      </div>

      <!-- 统计网格 -->
      <div class="stats-mini-grid">
        <div class="mini-stat-item">
          <div class="mini-stat-value">{{ wsMessageCount }}</div>
          <div class="mini-stat-label">消息数量</div>
        </div>
        <div class="mini-stat-item">
          <div class="mini-stat-value">{{ formatDuration(wsUptime) }}</div>
          <div class="mini-stat-label">运行时间</div>
        </div>
        <div class="mini-stat-item">
          <div class="mini-stat-value">{{ wsLastActivity }}</div>
          <div class="mini-stat-label">最后活动</div>
        </div>
        <div class="mini-stat-item">
          <div class="mini-stat-value" :class="wsQueueLength > 0 ? 'text-danger' : ''">{{ wsQueueLength }}</div>
          <div class="mini-stat-label">队列长度</div>
        </div>
      </div>

      <!-- 心跳状态 -->
      <div class="hb-panel">
        <h4>❤️ 心跳状态</h4>
        <div class="hb-stats-grid">
          <div class="hb-cell">
            <span class="hb-key">发送</span>
            <span class="hb-val">{{ heartbeatStats.totalPings }}</span>
          </div>
          <div class="hb-cell">
            <span class="hb-key">响应</span>
            <span class="hb-val success">{{ heartbeatStats.successfulPongs }}</span>
          </div>
          <div class="hb-cell">
            <span class="hb-key">失败</span>
            <span :class="['hb-val', heartbeatStats.failures > 0 ? 'error' : '']">
              {{ heartbeatStats.failures }}
            </span>
          </div>
          <div class="hb-cell">
            <span class="hb-key">质量</span>
            <span :class="['hb-val', getHbQualityClass(heartbeatStats.quality)]">
              {{ (heartbeatStats.quality * 100).toFixed(1) }}%
            </span>
          </div>
        </div>
      </div>

      <!-- 消息队列 -->
      <div class="queue-panel">
        <h4>📨 消息队列</h4>
        <div class="queue-detail">
          <span>当前队列长度: <strong>{{ wsQueueLength }}</strong></span>
          <span v-if="wsQueueLength > 0" class="queue-warn">⚠️ 有 {{ wsQueueLength }} 条消息等待发送</span>
          <span v-else class="queue-ok">✅ 队列为空</span>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="ws-actions-bar">
        <el-button type="primary" @click="wsManualReconnect" :disabled="isWsConnecting">
          {{ isWsConnecting ? '连接中...' : '手动重连' }}
        </el-button>
        <el-button type="warning" @click="wsForceReset" :disabled="!wsStats.isConnected">
          强制重置
        </el-button>
        <el-button @click="wsExportLogs">
          导出日志
        </el-button>
      </div>

      <!-- 事件日志 -->
      <div class="ws-logs-area">
        <div class="logs-header" @click="toggleWsLogs">
          <span>📋 事件日志 (最近{{ wsEventLogs.length }}条)</span>
          <span>{{ showWsLogs ? '▼' : '▶' }}</span>
        </div>
        <div v-if="showWsLogs" class="logs-body">
          <div
            v-for="(log, idx) in wsEventLogs"
            :key="idx"
            :class="['log-line', `log-${log.type}`]"
          >
            <span class="log-time">{{ log.time }}</span>
            <span class="log-msg">{{ log.message }}</span>
          </div>
          <div v-if="wsEventLogs.length === 0" class="log-empty">暂无事件记录</div>
        </div>
      </div>
    </div>
  </el-card>

  <!-- ====== 服务监测与控制 ====== -->
  <el-card>
    <template #header>
      <div class="card-header">
        <span>服务监测与控制</span>
        <el-button type="primary" size="small" @click="check_service_status" :loading="loading">检查状态</el-button>
      </div>
    </template>

    <div class="service-monitor">
      <div class="service-item" v-for="service in services" :key="service.name">
        <div class="service-info">
          <div class="service-name">{{ service.name }}</div>
          <div class="service-status">
            <el-tag :type="service.status === 'running' ? 'success' : 'danger'">
              {{ service.status === 'running' ? '运行中' : '已停止' }}
            </el-tag>
          </div>
          <div class="service-details">{{ service.details }}</div>
        </div>
        <div class="service-actions">
          <el-button type="primary" size="small" @click="start_service(service.name)" :disabled="service.status === 'running'">启动</el-button>
          <el-button type="danger" size="small" @click="stop_service(service.name)" :disabled="service.status !== 'running'">停止</el-button>
          <el-button type="info" size="small" @click="restart_service(service.name)">重启</el-button>
        </div>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onBeforeUnmount } from 'vue';
import { ElMessage } from 'element-plus';
import { Coin, Connection, Monitor, Refresh, CircleCheckFilled, CircleCloseFilled, WarningFilled, Loading } from '@element-plus/icons-vue';
import api from '@/api';
import { backendMonitor, type BackendStatus, type HealthCheckResult } from '@/services/backendMonitor';
import {
  getUnifiedCommunicationService,
  resetUnifiedCommunicationService,
} from '@/services/unifiedCommunicationService';

// ═════════════════════════════════════════════════
// 后端服务状态
// ═════════════════════════════════════════════════

const currentStatus = ref<BackendStatus>('unknown');
const lastResult = ref<HealthCheckResult | null>(null);
const previousStatus = ref<BackendStatus>('unknown');
const isChecking = ref(false);
const isOffline = ref(typeof navigator !== 'undefined' ? !navigator.onLine : false);
const selfHealInfo = ref<any>(null);

const statusLabels: Record<BackendStatus, string> = {
  unknown: '未知', healthy: '正常', degraded: '降级', down: '异常', checking: '检测中',
};

const statusLabel = computed(() => statusLabels[currentStatus.value]);
const responseTimeMs = computed(() => lastResult.value?.responseTimeMs ?? 0);

const lastCheckTime = computed(() => {
  if (!lastResult.value?.timestamp) return '-';
  return new Date(lastResult.value.timestamp).toLocaleString('zh-CN', { hour12: false });
});

const statusTagType = computed(() => {
  switch (currentStatus.value) {
    case 'healthy': return 'success';
    case 'degraded': return 'warning';
    case 'down': return 'danger';
    default: return 'info';
  }
});

function getDepTagType(status?: string): '' | 'success' | 'warning' | 'danger' {
  if (!status || status === 'unknown') return '';
  if (status === 'ok' || status === 'connected') return 'success';
  if (status === 'warn' || status === 'disconnected') return 'warning';
  return 'danger';
}

async function manualCheck() {
  isChecking.value = true;
  try {
    lastResult.value = await backendMonitor.checkNow();
    currentStatus.value = backendMonitor.getStatus();
  } finally {
    isChecking.value = false;
  }
}

async function fetchSelfHealInfo() {
  try {
    const res = await api.get('/api/selfheal/watchdog/status');
    selfHealInfo.value = res;
  } catch {
    selfHealInfo.value = null;
  }
}

// ═════════════════════════════════════════════════
// WebSocket 监控（从原 WebSocketMonitor 集成）
// ═════════════════════════════════════════════════

const isWsConnecting = ref(false);
const showWsLogs = ref(false);

const wsStats = reactive({
  state: 'disconnected' as string,
  protocol: '',
  isConnected: false,
  reconnectAttempts: 0,
});

const heartbeatStats = reactive({
  totalPings: 0,
  successfulPongs: 0,
  failures: 0,
  quality: 1.0,
});

const wsServerUrl = ref('');
const wsMessageCount = ref(0);
const wsUptime = ref(0);
const wsLastActivity = ref('-');
const wsQueueLength = ref(0);

interface WsEventLog {
  type: 'info' | 'warn' | 'error' | 'success';
  time: string;
  message: string;
}
const wsEventLogs = ref<WsEventLog[]>([]);
const MAX_WS_LOGS = 50;

let uptimeTimer: number | null = null;

// WS 计算属性
const wsStatusClass = computed(() => {
  if (wsStats.isConnected) return 'status-connected';
  if (wsStats.state === 'connecting' || wsStats.state === 'reconnecting') return 'status-connecting';
  if (wsStats.state === 'error') return 'status-error';
  return 'status-disconnected';
});

const wsStatusText = computed(() => {
  const m: Record<string, string> = {
    connected: '已连接', connecting: '连接中', reconnecting: '重连中',
    disconnected: '已断开', error: '错误',
  };
  return m[wsStats.state] || wsStats.state;
});

function toggleWsLogs() { showWsLogs.value = !showWsLogs.value; }

function addWsLog(type: WsEventLog['type'], msg: string) {
  const t = new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
  wsEventLogs.value.unshift({ type, time: t, message: msg });
  if (wsEventLogs.value.length > MAX_WS_LOGS) wsEventLogs.value.pop();
}

function wsRefreshStats() {
  const svc = getUnifiedCommunicationService();
  if (!svc) { addWsLog('warn', '统一通信服务未初始化'); return; }
  const s = svc.getStats();
  Object.assign(wsStats, s);
}

async function wsManualReconnect() {
  const svc = getUnifiedCommunicationService();
  if (!svc) { addWsLog('error', '无法重连：服务未初始化'); return; }
  isWsConnecting.value = true;
  addWsLog('info', '开始手动重连...');
  try {
    await svc.disconnect();
    await new Promise(r => setTimeout(r, 500));
    const ok = await svc.connect();
    ok ? addWsLog('success', '✅ 手动重连成功') : addWsLog('error', '❌ 手动重连失败');
    if (ok) wsUptime.value = 0;
  } catch (e: any) {
    addWsLog('error', `重连异常: ${e}`);
  } finally {
    isWsConnecting.value = false;
    wsRefreshStats();
  }
}

function wsForceReset() {
  if (!confirm('确定要强制重置WebSocket连接吗？这将断开当前连接。')) return;
  addWsLog('warn', '强制重置连接...');
  const ok = resetUnifiedCommunicationService(true);
  if (ok) {
    addWsLog('success', '✅ 重置成功，需要重新初始化');
    wsStats.isConnected = false; wsStats.state = 'disconnected';
  } else {
    addWsLog('error', '❌ 重置失败');
  }
}

function wsExportLogs() {
  const txt = wsEventLogs.value.map(l => `[${l.time}] [${l.type.toUpperCase()}] ${l.message}`).join('\n');
  const blob = new Blob([txt], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a'); a.href = url;
  a.download = `websocket-logs-${Date.now()}.txt`; a.click(); URL.revokeObjectURL(url);
  addWsLog('success', '📥 日志已导出');
}

function formatDuration(sec: number): string {
  if (sec < 60) return `${sec}秒`;
  if (sec < 3600) return `${Math.floor(sec / 60)}分${sec % 60}秒`;
  const h = Math.floor(sec / 3600), m = Math.floor((sec % 3600) / 60);
  return `${h}时${m}分`;
}

function getHbQualityClass(q: number): string {
  if (q >= 0.9) return 'success'; if (q >= 0.7) return 'warning'; return 'error';
}

// ═════════════════════════════════════════════════
// 服务监测与控制（原有逻辑）
// ═════════════════════════════════════════════════

interface LocalService {
  name: string; status: string; details: string; port: number; [key: string]: unknown;
}
const services = ref<LocalService[]>([
  { name: 'HTTP API服务', status: 'unknown', details: '端口: 8081', port: 8081 },
  { name: 'JT808网关服务', status: 'unknown', details: '端口: 8988', port: 8988 },
  { name: 'WebSocket服务', status: 'unknown', details: '端口: 8089', port: 8089 },
  { name: '客户端服务', status: 'unknown', details: '端口: 9808', port: 9808 },
]);

const loading = ref(false);
const service_name_map: Record<string, string> = {
  'HTTP API服务': 'database', 'JT808网关服务': 'jt808',
  'WebSocket服务': 'websocket', '客户端服务': 'redis',
};

const check_service_status = async () => {
  loading.value = true;
  try {
    const response = await api.get('/api/services/status') as any;
    if (response?.services) {
      for (const s of services.value) {
        const bs = response.services.find((x: any) => x.name === service_name_map[s.name]);
        if (bs) s.status = bs.status;
      }
    }
    ElMessage.success('服务状态检查完成');
  } catch (e) { console.error(e); ElMessage.error('检查服务状态失败'); }
  finally { loading.value = false; }
};

const start_service = async (name: string) => {
  try {
    await api.post(`/api/services/${service_name_map[name] || name}/start`, {}) as any;
    services.value.find(s => s.name === name)!.status = 'running';
    ElMessage.success(`${name} 启动成功`);
  } catch (e: any) { console.error(e); ElMessage.error(`启动服务 ${name} 失败`); }
};
const stop_service = async (name: string) => {
  try {
    await api.post(`/api/services/${service_name_map[name] || name}/stop`, {}) as any;
    services.value.find(s => s.name === name)!.status = 'stopped';
    ElMessage.success(`${name} 停止成功`);
  } catch (e: any) { console.error(e); ElMessage.error(`停止服务 ${name} 失败`); }
};
const restart_service = async (name: string) => {
  try {
    await api.post(`/api/services/${service_name_map[name] || name}/restart`, {}) as any;
    services.value.find(s => s.name === name)!.status = 'running';
    ElMessage.success(`${name} 重启成功`);
  } catch (e: any) { console.error(e); ElMessage.error(`重启服务 ${name} 失败`); }
};

// ═══════ 生命周期 ═══════

let unsubscribe: (() => void) | null = null;

onMounted(() => {
  // 后端服务状态初始化
  currentStatus.value = backendMonitor.getStatus();
  lastResult.value = backendMonitor.getLastResult();
  unsubscribe = backendMonitor.onStatusChange((s, prev, r) => {
    currentStatus.value = s; previousStatus.value = prev; lastResult.value = r;
  });
  backendMonitor.start();
  fetchSelfHealInfo();

  // WebSocket 初始化
  wsRefreshStats();
  const ws = getUnifiedCommunicationService();
  if (ws) {
    wsServerUrl.value = 'ws://localhost:9808/ws';
    ws.on('connected', () => { addWsLog('success', '✅ WebSocket连接成功'); wsRefreshStats(); });
    ws.on('disconnected', () => { addWsLog('warn', '⚠️ WebSocket连接断开'); wsRefreshStats(); });
    ws.on('error', (d: any) => { addWsLog('error', `❌ 错误: ${d.error || '未知错误'}`); wsRefreshStats(); });
    ws.on('message', () => {
      wsMessageCount.value++;
      wsLastActivity.value = new Date().toLocaleTimeString('zh-CN', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });
    });
    addWsLog('info', '🚀 监控组件已启动');
  } else { addWsLog('warn', '⚠️ 统一通信服务未找到'); }

  // 运行计时器
  uptimeTimer = window.setInterval(() => { if (wsStats.isConnected) wsUptime.value++; }, 1000);

  // 服务状态初始化
  check_service_status();
});

onBeforeUnmount(() => {
  unsubscribe?.();
  if (uptimeTimer) clearInterval(uptimeTimer);
});
</script>

<style scoped>
.service-monitor { padding: 10px 0; }
.service-item {
  display: flex; justify-content: space-between; align-items: center;
  padding: 16px; margin-bottom: 12px; background-color: #f9fafb;
  border-radius: 8px; transition: all 0.3s ease;
}
.service-item:hover { transform: translateX(5px); box-shadow: 0 4px 12px rgba(0,0,0,0.1); }
.service-info { flex: 1; }
.service-name { font-weight: bold; color: #374151; margin-bottom: 4px; font-size: 14px; }
.service-status { margin-bottom: 4px; }
.service-details { font-size: 12px; color: #6b7280; }
.service-actions { display: flex; gap: 8px; }
@media (max-width: 768px) {
  .service-item { flex-direction: column; align-items: flex-start; gap: 12px; }
  .service-actions { align-self: flex-end; }
}

/* ── WebSocket 集成样式 ── */
.websocket-section { display: flex; flex-direction: column; gap: 16px; }

.ws-status-grid {
  display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 10px;
}
.ws-stat-box {
  display: flex; justify-content: space-between; align-items: center;
  padding: 10px 14px; background: #f9fafb; border-radius: 6px; border: 1px solid #ebeef5;
}
.box-label { color: #909399; font-size: 13px; }
.box-value { font-weight: 600; font-size: 13px; color: #303133; }
.text-muted { color: #909399; }
.font-mono { font-family: monospace; font-size: 11px; }
.accent { color: #667eea; }
.status-connected { color: #67c23a; }
.status-connecting { color: #e6a23c; }
.status-disconnected { color: #909399; }
.status-error { color: #f56c6c; }

.stats-mini-grid {
  display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px;
}
.mini-stat-item {
  background: white; border: 1px solid #ebeef5; border-radius: 6px;
  padding: 12px 8px; text-align: center;
}
.mini-stat-value { font-size: 18px; font-weight: 700; color: #667eea; }
.mini-stat-label { font-size: 11px; color: #909399; margin-top: 4px; }
.text-danger { color: #f56c6c; }

.hb-panel, .queue-panel, .ws-logs-area {
  background: white; border: 1px solid #ebeef5; border-radius: 6px; padding: 14px;
}
.hb-panel h4, .queue-panel h4 { margin: 0 0 10px; font-size: 13px; color: #495057; }
.hb-stats-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
.hb-cell {
  display: flex; justify-content: space-between; align-items: center;
  padding: 6px 10px; background: #f8f9fa; border-radius: 4px; font-size: 12px;
}
.hb-key { color: #6c757d; }
.hb-val { font-weight: 600; color: #212529; }
.hb-val.success { color: #28a745; }
.hb-val.error { color: #dc3545; }
.hb-val.warning { color: #ffc107; }

.queue-detail { display: flex; justify-content: space-between; align-items: center; font-size: 13px; }
.queue-warn { color: #dc3545; font-weight: 500; }
.queue-ok { color: #28a745; }

.ws-actions-bar { display: flex; gap: 8px; flex-wrap: wrap; }

.ws-logs-area .logs-header {
  display: flex; justify-content: space-between; cursor: pointer;
  font-size: 13px; font-weight: 500; color: #495057;
  user-select: none; padding-bottom: 8px; border-bottom: 1px solid #ebeef5; margin-bottom: 8px;
}
.ws-logs-area .logs-header:hover { color: #667eea; }
.logs-body { max-height: 200px; overflow-y: auto; font-family: 'Courier New', monospace; font-size: 11px; }
.log-line { display: flex; gap: 8px; padding: 4px 0; border-bottom: 1px solid #f1f3f5; }
.log-time { color: #6c757d; min-width: 65px; }
.log-msg { color: #212529; word-break: break-all; }
.log-info .log-msg { color: #17a2b8; }
.log-warn .log-msg { color: #ffc107; }
.log-error .log-msg { color: #dc3545; }
.log-success .log-msg { color: #28a745; }
.log-empty { text-align: center; color: #6c757d; padding: 20px 0; font-style: italic; }

@media (max-width: 768px) {
  .stats-mini-grid { grid-template-columns: repeat(2, 1fr); }
  .hb-stats-grid { grid-template-columns: repeat(2, 1fr); }
  .ws-actions-bar { flex-direction: column; }
}
</style>
