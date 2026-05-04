<template>
  <div class="unified-communication-panel">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>统一通信服务控制面板</span>
          <el-button size="small" @click="refreshStatus">刷新状态</el-button>
        </div>
      </template>

      <!-- 新服务器状态 -->
      <div class="server-status-section">
        <h4>新服务器 (TCP + WebSocket)</h4>
        <div class="status-row">
          <span class="label">连接状态:</span>
          <el-tag :type="newServerConnected ? 'success' : 'danger'" size="small">
            {{ newServerConnected ? '已连接' : '未连接' }}
          </el-tag>
        </div>
        <div class="status-row">
          <span class="label">当前协议:</span>
          <el-tag type="info" size="small">{{ newServerProtocol }}</el-tag>
        </div>
        <div class="status-row">
          <span class="label">地址:</span>
          <span class="value">{{ newServerConfig.host }}:{{ newServerConfig.port }}</span>
        </div>

        <!-- 协议切换 -->
        <div class="protocol-switch">
          <el-button-group>
            <el-button
              size="small"
              :type="newServerProtocol === 'websocket' ? 'primary' : 'default'"
              @click="switchProtocol('websocket')"
              :disabled="!newServerConnected"
            >
              WebSocket
            </el-button>
            <el-button
              size="small"
              :type="newServerProtocol === 'tcp' ? 'primary' : 'default'"
              @click="switchProtocol('tcp')"
              :disabled="!newServerConnected"
            >
              TCP
            </el-button>
          </el-button-group>
        </div>
      </div>

      <el-divider />

      <!-- 旧服务器状态 -->
      <div class="server-status-section">
        <h4>旧服务器 (TCP)</h4>
        <div class="status-row">
          <span class="label">连接状态:</span>
          <el-tag :type="legacyServerConnected ? 'success' : 'danger'" size="small">
            {{ legacyServerConnected ? '已连接' : '未连接' }}
          </el-tag>
        </div>
        <div class="status-row">
          <span class="label">协议:</span>
          <el-tag type="info" size="small">TCP</el-tag>
        </div>
        <div class="status-row">
          <span class="label">地址:</span>
          <span class="value">{{ legacyServerConfig.host }}:{{ legacyServerConfig.port }}</span>
        </div>
      </div>

      <el-divider />

      <!-- 快速操作 -->
      <div class="quick-actions">
        <h4>快速操作</h4>
        <div class="action-buttons">
          <el-button type="primary" size="small" @click="sendTestMessage" :disabled="!newServerConnected">
            发送测试消息
          </el-button>
          <el-button size="small" @click="loginTest" :disabled="!newServerConnected"> 测试登录 </el-button>
          <el-button size="small" @click="getVehicles" :disabled="!newServerConnected"> 获取车辆 </el-button>
        </div>
      </div>

      <el-divider />

      <!-- 消息日志 -->
      <div class="message-log">
        <div class="log-header">
          <h4>消息日志</h4>
          <el-button size="small" text @click="clearLog">清空</el-button>
        </div>
        <div class="log-container">
          <div v-for="(log, index) in messageLogs" :key="index" :class="['log-entry', log.type]">
            <span class="log-time">{{ formatTime(log.timestamp) }}</span>
            <span class="log-type">{{ log.server }}</span>
            <span class="log-message">{{ log.message }}</span>
          </div>
          <div v-if="messageLogs.length === 0" class="empty-log">暂无消息</div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { ElMessage } from 'element-plus';

// 类型声明
interface ServiceStatus {
  newServer?: {
    connected: boolean;
    protocol: string;
  };
  legacyServer?: {
    connected: boolean;
  };
}

interface UnifiedConfig {
  newServer: {
    host: string;
    httpPort: number;
  };
  legacyServer: {
    host: string;
    port: number;
  };
}

interface CommunicationMessage {
  type: string;
  command: string;
  timestamp: number;
  payload: unknown;
}

interface CustomEventWithDetail<T> extends Event {
  detail: T;
}

// 类型断言辅助函数
function getWindowWithServices() {
  return window as unknown as {
    $getServiceStatus?: () => ServiceStatus;
    $unifiedConfig?: UnifiedConfig;
    $switchProtocol?: (protocol: 'tcp' | 'websocket') => Promise<boolean>;
    $sendMessage?: (message: CommunicationMessage) => Promise<unknown>;
  };
}

// 状态
const newServerConnected = ref(false);
const newServerProtocol = ref('disconnected');
const newServerConfig = ref({ host: 'localhost', port: 8082 });
const legacyServerConnected = ref(false);
const legacyServerConfig = ref({ host: '127.0.0.1', port: 9808 });
const messageLogs = ref<
  Array<{
    timestamp: number;
    type: 'info' | 'success' | 'error' | 'warning';
    server: string;
    message: string;
  }>
>([]);

// 事件监听器
let eventListeners: Array<() => void> = [];

onMounted(() => {
  // 监听新服务器事件
  const onNewConnected = () => {
    newServerConnected.value = true;
    addLog('success', '新服务器', '已连接');
  };

  const onNewDisconnected = () => {
    newServerConnected.value = false;
    newServerProtocol.value = 'disconnected';
    addLog('warning', '新服务器', '已断开');
  };

  const onNewError = (e: Event) => {
    const customEvent = e as CustomEventWithDetail<{ error: string }>;
    addLog('error', '新服务器', `错误: ${customEvent.detail?.error || '未知错误'}`);
  };

  const onNewMessage = (e: Event) => {
    const customEvent = e as CustomEventWithDetail<{ message: { command: string } }>;
    const message = customEvent.detail?.message;
    addLog('info', '新服务器', `收到: ${message?.command || '未知命令'}`);
  };

  const onLegacyConnected = () => {
    legacyServerConnected.value = true;
    addLog('success', '旧服务器', '已连接');
  };

  const onLegacyDisconnected = () => {
    legacyServerConnected.value = false;
    addLog('warning', '旧服务器', '已断开');
  };

  const onLegacyError = (e: Event) => {
    const customEvent = e as CustomEventWithDetail<{ error: string }>;
    addLog('error', '旧服务器', `错误: ${customEvent.detail?.error || '未知错误'}`);
  };

  // 注册事件监听器
  window.addEventListener('unified-service-connected', onNewConnected);
  window.addEventListener('unified-service-disconnected', onNewDisconnected);
  window.addEventListener('unified-service-error', onNewError);
  window.addEventListener('unified-service-message', onNewMessage);
  window.addEventListener('legacy-service-connected', onLegacyConnected);
  window.addEventListener('legacy-service-disconnected', onLegacyDisconnected);
  window.addEventListener('legacy-service-error', onLegacyError);

  // 保存引用以便清理
  eventListeners = [
    () => window.removeEventListener('unified-service-connected', onNewConnected),
    () => window.removeEventListener('unified-service-disconnected', onNewDisconnected),
    () => window.removeEventListener('unified-service-error', onNewError),
    () => window.removeEventListener('unified-service-message', onNewMessage),
    () => window.removeEventListener('legacy-service-connected', onLegacyConnected),
    () => window.removeEventListener('legacy-service-disconnected', onLegacyDisconnected),
    () => window.removeEventListener('legacy-service-error', onLegacyError),
  ];

  // 初始化状态
  refreshStatus();
});

onUnmounted(() => {
  // 清理事件监听器
  eventListeners.forEach((cleanup) => cleanup());
});

// 刷新状态
function refreshStatus() {
  const win = getWindowWithServices();
  if (win.$getServiceStatus) {
    const status = win.$getServiceStatus();

    if (status.newServer) {
      newServerConnected.value = status.newServer.connected;
      newServerProtocol.value = status.newServer.protocol;
    }

    if (status.legacyServer) {
      legacyServerConnected.value = status.legacyServer.connected;
    }

    // 从配置中读取服务器地址
    const config = win.$unifiedConfig;
    if (config) {
      newServerConfig.value = {
        host: config.newServer.host,
        port: config.newServer.httpPort,
      };
      legacyServerConfig.value = {
        host: config.legacyServer.host,
        port: config.legacyServer.port,
      };
    }
  }
}

// 切换协议
async function switchProtocol(protocol: 'tcp' | 'websocket') {
  try {
    addLog('info', '新服务器', `正在切换到 ${protocol.toUpperCase()}...`);

    const win = getWindowWithServices();
    if (win.$switchProtocol) {
      const success = await win.$switchProtocol(protocol);

      if (success) {
        newServerProtocol.value = protocol;
        addLog('success', '新服务器', `已切换到 ${protocol.toUpperCase()}`);
        ElMessage.success(`协议已切换到 ${protocol.toUpperCase()}`);
      } else {
        addLog('error', '新服务器', '协议切换失败');
        ElMessage.error('协议切换失败');
      }
    }
  } catch (error) {
    addLog('error', '新服务器', `切换异常: ${error}`);
    ElMessage.error('协议切换异常');
  }
}

// 发送测试消息
async function sendTestMessage() {
  try {
    addLog('info', '新服务器', '发送测试消息...');

    const win = getWindowWithServices();
    if (win.$sendMessage) {
      const response = await win.$sendMessage({
        type: 'command',
        command: 'ping',
        timestamp: Date.now(),
        payload: {},
      });

      addLog('success', '新服务器', `测试消息响应: ${JSON.stringify(response)}`);
      ElMessage.success('测试消息发送成功');
    }
  } catch (error) {
    addLog('error', '新服务器', `发送失败: ${error}`);
    ElMessage.error('发送失败');
  }
}

// 测试登录
async function loginTest() {
  try {
    addLog('info', '新服务器', '发送登录请求...');

    const win = getWindowWithServices();
    if (win.$sendMessage) {
      const response = await win.$sendMessage({
        type: 'command',
        command: 'login',
        timestamp: Date.now(),
        payload: {
          username: 'admin',
          password: 'admin123',
        },
      });

      addLog('success', '新服务器', `登录响应: ${JSON.stringify(response)}`);
      ElMessage.success('登录请求已发送');
    }
  } catch (error) {
    addLog('error', '新服务器', `登录失败: ${error}`);
    ElMessage.error('登录失败');
  }
}

// 获取车辆
async function getVehicles() {
  try {
    addLog('info', '新服务器', '获取车辆列表...');

    const win = getWindowWithServices();
    if (win.$sendMessage) {
      const response = await win.$sendMessage({
        type: 'command',
        command: 'get_vehicles',
        timestamp: Date.now(),
        payload: {
          page: 1,
          pageSize: 10,
        },
      });

      addLog('success', '新服务器', `车辆列表: ${JSON.stringify(response)}`);
      ElMessage.success('车辆请求已发送');
    }
  } catch (error) {
    addLog('error', '新服务器', `获取失败: ${error}`);
    ElMessage.error('获取失败');
  }
}

// 添加日志
function addLog(type: 'info' | 'success' | 'error' | 'warning', server: string, message: string) {
  messageLogs.value.unshift({
    timestamp: Date.now(),
    type,
    server,
    message,
  });

  // 保持最近50条日志
  if (messageLogs.value.length > 50) {
    messageLogs.value = messageLogs.value.slice(0, 50);
  }
}

// 清空日志
function clearLog() {
  messageLogs.value = [];
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
.unified-communication-panel {
  max-width: 800px;
  margin: 20px auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.server-status-section {
  margin-bottom: 20px;
}

.server-status-section h4 {
  margin: 0 0 15px 0;
  color: #333;
  font-size: 14px;
  font-weight: 600;
}

.status-row {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
}

.status-row .label {
  width: 80px;
  color: #666;
  font-size: 13px;
}

.status-row .value {
  color: #333;
  font-size: 13px;
}

.protocol-switch {
  margin-top: 15px;
}

.quick-actions h4 {
  margin: 0 0 15px 0;
  color: #333;
  font-size: 14px;
  font-weight: 600;
}

.action-buttons {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.message-log {
  margin-top: 20px;
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.log-header h4 {
  margin: 0;
  color: #333;
  font-size: 14px;
  font-weight: 600;
}

.log-container {
  max-height: 300px;
  overflow-y: auto;
  background: #f5f5f5;
  border-radius: 4px;
  padding: 10px;
}

.log-entry {
  padding: 8px;
  margin-bottom: 5px;
  border-radius: 4px;
  font-size: 12px;
  line-height: 1.5;
}

.log-entry.info {
  background: #e6f7ff;
  border-left: 3px solid #1890ff;
}

.log-entry.success {
  background: #f6ffed;
  border-left: 3px solid #52c41a;
}

.log-entry.warning {
  background: #fffbe6;
  border-left: 3px solid #faad14;
}

.log-entry.error {
  background: #fff1f0;
  border-left: 3px solid #f5222d;
}

.log-time {
  color: #999;
  margin-right: 10px;
  font-family: monospace;
}

.log-type {
  font-weight: bold;
  margin-right: 10px;
  min-width: 60px;
  display: inline-block;
}

.log-message {
  color: #333;
  word-break: break-all;
}

.empty-log {
  text-align: center;
  color: #999;
  padding: 20px;
  font-size: 13px;
}
</style>


