<template>
  <!-- 底部精简状态条：仅显示连接状态和心跳状态 -->
  <div class="ws-status-bar">
    <div class="ws-bar-left" @click="toggleCollapse">
      <span class="bar-label">WebSocket</span>
      <span class="status-dot" :class="statusClass" :title="statusText">{{ statusIcon }}</span>
      <span class="status-text" :class="statusClass">{{ statusText }}</span>

      <template v-if="!isCollapsed">
        <span class="bar-divider">|</span>
        <span class="hb-label">❤️ 心跳</span>
        <span class="hb-value success">↑{{ heartbeatStats.totalPings }}</span>
        <span class="hb-value success">↓{{ heartbeatStats.successfulPongs }}</span>
        <span v-if="heartbeatStats.failures > 0" class="hb-value error">✕{{ heartbeatStats.failures }}</span>
        <span class="hb-quality" :class="getQualityClass(heartbeatStats.quality)">
          {{ (heartbeatStats.quality * 100).toFixed(0) }}%
        </span>
      </template>
    </div>
    <button class="collapse-btn" @click="toggleCollapse">
      {{ isCollapsed ? '展开' : '收起' }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue';
import {
  getUnifiedCommunicationService,
} from '@/services/unifiedCommunicationService';

// 组件状态
const isCollapsed = ref(false);

// WebSocket 统计信息
const stats = reactive({
  state: 'disconnected' as string,
  protocol: '' as string,
  isConnected: false,
  reconnectAttempts: 0,
});

// 心跳统计
const heartbeatStats = reactive({
  totalPings: 0,
  successfulPongs: 0,
  failures: 0,
  quality: 1.0,
});

// 计算属性
const statusClass = computed(() => {
  if (stats.isConnected) return 'status-connected';
  if (stats.state === 'connecting' || stats.state === 'reconnecting') return 'status-connecting';
  if (stats.state === 'error') return 'status-error';
  return 'status-disconnected';
});

const statusText = computed(() => {
  const stateMap: Record<string, string> = {
    connected: '已连接',
    connecting: '连接中',
    reconnecting: '重连中',
    disconnected: '已断开',
    error: '错误',
  };
  return stateMap[stats.state] || stats.state;
});

const statusIcon = computed(() => {
  const iconMap: Record<string, string> = {
    connected: '●',
    connecting: '◐',
    reconnecting: '◑',
    disconnected: '○',
    error: '✕',
  };
  return iconMap[stats.state] || '○';
});

// 方法
function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value;
}

function refreshStats() {
  const ws = getUnifiedCommunicationService();
  if (!ws) return;
  const wsStats = ws.getStats();
  Object.assign(stats, wsStats);
}

function getQualityClass(quality: number): string {
  if (quality >= 0.9) return 'success';
  if (quality >= 0.7) return 'warning';
  return 'error';
}

let uptimeTimer: number | null = null;

onMounted(() => {
  refreshStats();

  const ws = getUnifiedCommunicationService();
  if (ws) {
    ws.on('connected', () => { refreshStats(); });
    ws.on('disconnected', () => { refreshStats(); });
    ws.on('error', () => { refreshStats(); });
  }
});
</script>

<style scoped>
.ws-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 20px;
  background: #f5f7fa;
  border-top: 1px solid #e4e7ed;
  font-size: 12px;
  color: #606266;
}

.ws-bar-left {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  user-select: none;
}

.bar-label {
  font-weight: 600;
  color: #303133;
}

.status-dot {
  font-size: 12px;
}

.status-text {
  font-weight: 500;
}

.status-connected { color: #67c23a; }
.status-connecting { color: #e6a23c; }
.status-disconnected { color: #909399; }
.status-error { color: #f56c6c; }

.bar-divider {
  color: #dcdfe6;
  margin: 0 2px;
}

.hb-label {
  margin-left: 4px;
}

.hb-value {
  font-family: 'Courier New', monospace;
  font-weight: 600;
  font-size: 11px;
}
.hb-value.success { color: #67c23a; }
.hb-value.error { color: #f56c6c; }

.hb-quality {
  font-family: 'Courier New', monospace;
  font-weight: 600;
  font-size: 11px;
  padding: 0 3px;
  border-radius: 3px;
}
.hb-quality.success { color: #67c23a; background: #f0f9eb; }
.hb-quality.warning { color: #e6a23c; background: #fdf6ec; }
.hb-quality.error { color: #f56c6c; background: #fef0f0; }

.collapse-btn {
  padding: 2px 10px;
  font-size: 11px;
  color: #909399;
  background: transparent;
  border: 1px solid #dcdfe6;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.collapse-btn:hover {
  color: #409eff;
  border-color: #409eff;
}
</style>
