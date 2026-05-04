<template>
  <div class="backend-status-indicator" :class="[`status-${currentStatus}`]" @click="showDetail">
    <el-tooltip :content="tooltipText" placement="bottom" :disabled="!tooltipText">
      <div class="status-wrapper">
        <!-- 状态图标 -->
        <div class="status-icon" :class="{ pulsing: currentStatus === 'down' || currentStatus === 'checking' }">
          <el-icon v-if="currentStatus === 'healthy'" :size="14"><CircleCheckFilled /></el-icon>
          <el-icon v-else-if="currentStatus === 'degraded'" :size="14"><WarningFilled /></el-icon>
          <el-icon v-else-if="currentStatus === 'down'" :size="14"><CircleCloseFilled /></el-icon>
          <el-icon v-else :size="14"><Loading /></el-icon>
        </div>

        <!-- 状态文本 -->
        <span class="status-text">{{ statusLabel }}</span>

        <!-- 响应时间（仅 healthy/degraded 显示） -->
        <span v-if="responseTimeMs > 0 && currentStatus !== 'down'" class="response-time">
          {{ responseTimeMs }}ms
        </span>

        <!-- 离线标识 -->
        <el-tag v-if="isOffline" size="small" type="danger" effect="dark" class="offline-tag">离线</el-tag>
      </div>
    </el-tooltip>

    <!-- 详情抽屉 -->
    <el-drawer
      v-model="drawerVisible"
      title="后端服务状态"
      direction="rtl"
      size="400px"
      :with-header="true"
    >
      <div class="status-detail">
        <!-- 整体状态卡片 -->
        <el-card shadow="hover" class="overview-card">
          <template #header>
            <div class="card-header">
              <span>整体状态</span>
              <el-tag :type="statusTagType" size="large" effect="dark">{{ statusLabel }}</el-tag>
            </div>
          </template>
          <div class="overview-content">
            <div class="stat-item">
              <span class="label">当前响应时间</span>
              <span class="value" :class="{ warning: responseTimeMs > 1000, danger: responseTimeMs > 3000 }">
                {{ responseTimeMs > 0 ? `${responseTimeMs} ms` : '-' }}
              </span>
            </div>
            <div class="stat-item">
              <span class="label">上次检查时间</span>
              <span class="value">{{ lastCheckTime }}</span>
            </div>
            <div class="stat-item">
              <span class="label">前次状态</span>
              <span class="value">{{ previousStatusLabel }}</span>
            </div>
            <div class="stat-item">
              <span class="label">网络状态</span>
              <span class="value" :class="{ offline: isOffline }">{{ isOffline ? '离线' : '在线' }}</span>
            </div>
          </div>
        </el-card>

        <!-- 依赖状态（深度检查结果） -->
        <el-card v-if="lastResult?.details" shadow="hover" class="deps-card">
          <template #header><span>依赖服务状态</span></template>
          <div class="deps-list">
            <div class="dep-item">
              <el-icon color="#67C23A"><Coin /></el-icon>
              <span class="dep-name">数据库</span>
              <el-tag :type="getDepTagType(lastResult.details.database)" size="small">
                {{ lastResult.details.database }}
              </el-tag>
            </div>
            <div class="dep-item">
              <el-icon color="#E6A23C"><Connection /></el-icon>
              <span class="dep-name">Redis</span>
              <el-tag :type="getDepTagType(lastResult.details.redis)" size="small">
                {{ lastResult.details.redis }}
              </el-tag>
            </div>
            <div v-if="lastResult.details.hostname" class="dep-item">
              <el-icon><Monitor /></el-icon>
              <span class="dep-name">主机</span>
              <span class="dep-value">{{ lastResult.details.hostname }}</span>
            </div>
            <div v-if="lastResult.details.cache" class="dep-item cache-info">
              <span class="dep-name">缓存命中率</span>
              <el-progress
                :percentage="Math.round(lastResult.details.cache.hit_rate * 100)"
                :stroke-width="6"
                :show-text="true"
                style="width: 120px"
              />
            </div>
          </div>
        </el-card>

        <!-- 错误信息 -->
        <el-card v-if="lastResult?.error" shadow="hover" class="error-card">
          <template #header><span>错误详情</span></template>
          <el-alert :title="lastResult.error" type="error" :closable="false" show-icon />
        </el-card>

        <!-- 手动检查按钮 -->
        <div class="action-bar">
          <el-button type="primary" @click="manualCheck" :loading="isChecking">
            <el-icon><Refresh /></el-icon> 立即检查
          </el-button>
          <el-button @click="drawerVisible = false">关闭</el-button>
        </div>
      </div>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import {
  CircleCheckFilled,
  CircleCloseFilled,
  WarningFilled,
  Loading,
  Coin,
  Connection,
  Monitor,
  Refresh,
} from '@element-plus/icons-vue';
import { backendMonitor, type BackendStatus, type HealthCheckResult } from '@/services/backendMonitor';

// ==================== 状态 ====================

const currentStatus = ref<BackendStatus>('unknown');
const lastResult = ref<HealthCheckResult | null>(null);
const previousStatus = ref<BackendStatus>('unknown');
const drawerVisible = ref(false);
const isChecking = ref(false);
const isOffline = ref(!navigator.onLine);

// ==================== 计算属性 ====================

const statusLabels: Record<BackendStatus, string> = {
  unknown: '未知',
  healthy: '正常',
  degraded: '降级',
  down: '异常',
  checking: '检测中',
};

const statusLabel = computed(() => statusLabels[currentStatus.value]);

const tooltipText = computed(() => {
  if (isOffline.value) return '网络已断开';
  switch (currentStatus.value) {
    case 'healthy': return '后端运行正常';
    case 'degraded': return '后端部分功能降级，点击查看详情';
    case 'down': return '后端连接失败，点击查看详情';
    case 'checking': return '正在检测后端...';
    default: return '等待检测';
  }
});

const responseTimeMs = computed(() => lastResult.value?.responseTimeMs ?? 0);

const lastCheckTime = computed(() => {
  if (!lastResult.value?.timestamp) return '-';
  const d = new Date(lastResult.value.timestamp);
  return d.toLocaleString('zh-CN', { hour12: false });
});

const previousStatusLabel = computed(() => statusLabels[previousStatus.value] ?? '-');

const statusTagType = computed(() => {
  switch (currentStatus.value) {
    case 'healthy': return 'success';
    case 'degraded': return 'warning';
    case 'down': return 'danger';
    default: return 'info';
  }
});

// ==================== 方法 ====================

function getDepTagType(status?: string): '' | 'success' | 'warning' | 'danger' {
  if (!status || status === 'unknown') return '';
  if (status === 'ok' || status === 'connected') return 'success';
  if (status === 'warn' || status === 'disconnected') return 'warning';
  return 'danger';
}

function showDetail() {
  drawerVisible.value = true;
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

// ==================== 监听器订阅 ====================

let unsubscribe: (() => void) | null = null;

onMounted(() => {
  // 同步初始状态
  currentStatus.value = backendMonitor.getStatus();
  lastResult.value = backendMonitor.getLastResult();

  // 订阅状态变更
  unsubscribe = backendMonitor.onStatusChange((status, prev, result) => {
    currentStatus.value = status;
    previousStatus.value = prev;
    lastResult.value = result;
  });

  // 启动监测（如果尚未启动）
  backendMonitor.start();
});

onBeforeUnmount(() => {
  unsubscribe?.();
});
</script>

<style scoped>
.backend-status-indicator {
  display: inline-flex;
  align-items: center;
  cursor: pointer;
  user-select: none;
  padding: 4px 8px;
  border-radius: 16px;
  transition: all 0.3s ease;
  font-size: 13px;
}

.status-wrapper {
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 状态背景色 */
.status-healthy { background-color: #f0f9eb; color: #67c23a; }
.status-degraded { background-color: #fdf6ec; color: #e6a23c; }
.status-down { background-color: #fef0f0; color: #f56c6c; }
.status-checking { background-color: #ecf5ff; color: #409eff; }
.status-unknown { background-color: #f4f4f5; color: #909399; }

/* 状态图标 */
.status-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.pulsing {
  animation: pulse 1.2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.92); }
}

.status-text {
  font-weight: 500;
  white-space: nowrap;
}

.response-time {
  color: #909399;
  font-size: 11px;
  padding: 1px 4px;
  background: rgba(255,255,255,0.7);
  border-radius: 3px;
}

.offline-tag {
  margin-left: 4px;
}

/* 详情面板 */
.status-detail {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.overview-card .card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.overview-content {
  display: grid;
  gap: 10px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px solid #f0f0f0;
}

.stat-item .label {
  color: #606266;
}

.stat-item .value {
  font-weight: 500;
  color: #303133;
}

.stat-item .value.warning { color: #e6a23c; }
.stat-item .value.danger { color: #f56c6c; }
.stat-item .value.offline { color: #f56c6c; }

.deps-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.dep-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dep-name {
  color: #606266;
  min-width: 70px;
}

.dep-value {
  color: #303133;
  font-family: monospace;
}

.action-bar {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 8px;
}
</style>
