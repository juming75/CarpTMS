<template>
  <div class="unified-communication-demo">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>统一通信服务实际应用演示</span>
          <el-tag :type="isConnected ? 'success' : 'danger'" size="small">
            {{ isConnected ? '已连接' : '未连接' }}
          </el-tag>
        </div>
      </template>

      <!-- 当前协议 -->
      <div class="info-section">
        <h4>连接信息</h4>
        <div class="info-row">
          <span class="label">当前协议:</span>
          <el-tag type="primary" size="small">{{ currentProtocol }}</el-tag>
        </div>
        <div class="info-row">
          <span class="label">服务器:</span>
          <span>{{ serverConfig.host }}:{{ serverConfig.port }}</span>
        </div>
      </div>

      <el-divider />

      <!-- 您的代码应用示例 -->
      <div class="code-example-section">
        <h4>您的代码应用</h4>
        <el-alert title="以下代码已在 main.ts 中应用" type="info" :closable="false" style="margin-bottom: 15px" />
        <pre class="code-block">{{ codeExample }}</pre>
      </div>

      <el-divider />

      <!-- 快速操作 -->
      <div class="quick-actions">
        <h4>快速操作</h4>
        <el-space wrap>
          <el-button type="primary" size="small" @click="testLogin" :disabled="!isConnected"> 测试登录 </el-button>
          <el-button size="small" @click="getVehicles" :disabled="!isConnected"> 获取车辆 </el-button>
          <el-button size="small" @click="switchProtocol('websocket')" :disabled="!isConnected">
            切换到WebSocket
          </el-button>
          <el-button size="small" @click="switchProtocol('tcp')" :disabled="!isConnected"> 切换到TCP </el-button>
          <el-button size="small" @click="refreshStatus"> 刷新状态 </el-button>
        </el-space>
      </div>

      <el-divider />

      <!-- 消息监听日志 -->
      <div class="message-log-section">
        <div class="log-header">
          <h4>消息监听日志</h4>
          <el-button size="small" text @click="clearLog">清空</el-button>
        </div>
        <div class="log-container">
          <div v-for="(log, index) in messageLogs" :key="index" :class="['log-entry', log.type]">
            <span class="log-time">{{ formatTime(log.timestamp) }}</span>
            <span class="log-message">{{ log.message }}</span>
          </div>
          <div v-if="messageLogs.length === 0" class="empty-log">暂无消息</div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted, onUnmounted } from 'vue';
import { ElMessage } from 'element-plus';
import type { CommunicationMessage } from '@/services/unifiedCommunicationService';

// 状态
const isConnected = ref(false);
const currentProtocol = ref('disconnected');
const serverConfig = ref({ host: 'localhost', port: 8082 });
const messageLogs = ref<
  Array<{
    timestamp: number;
    type: 'info' | 'success' | 'error' | 'warning';
    message: string;
  }>
>([]);

// 代码示例
const codeExample = `// 在 main.ts 中已应用：
import { initForNewServer } from './services/unifiedCommunicationService'

// 连接新服务器（自动选择TCP或WebSocket）
const newService = initForNewServer('localhost', 8082, 'auto')
await newService.connect()

// 发送消息（自动使用最优协议）
await newService.send({
  type: 'command',
  command: 'login',
  payload: { username: 'admin', password: 'password' }
})

// 监听消息
newService.on('message', (message) => {
  console.log('收到消息:', message)
})

// 切换协议
await newService.switchProtocol('websocket')`;

// 消息处理器
const messageHandler = (message: CommunicationMessage) => {
  addLog('info', `收到消息: ${message.command}`);
  console.log('[统一通信服务] 收到消息:', message);
};

onMounted(() => {
  // 检查服务是否可用
  const service = window.$newServerService;

  if (service) {
    addLog('success', '统一通信服务已就绪');

    // 注册消息监听（您的代码应用）
    service.on('message', messageHandler);

    // 更新状态
    refreshStatus();

    addLog('info', '已注册消息监听器');
  } else {
    addLog('warning', '统一通信服务未找到，可能在初始化时失败');
  }
});

onUnmounted(() => {
  // 清理消息监听器
  const service = window.$newServerService;
  if (service && messageHandler) {
    service.off('message', messageHandler);
    addLog('info', '已清理消息监听器');
  }
});

// 测试登录
async function testLogin() {
  try {
    addLog('info', '发送登录请求...');

    const service = window.$newServerService;
    if (!service) {
      throw new Error('服务不可用');
    }

    // 您的代码应用：发送消息
    const response = await service.send({
      type: 'command',
      command: 'login',
      timestamp: Date.now(),
      payload: {
        username: 'admin',
        password: 'admin123',
      },
    });

    addLog('success', `登录响应: ${JSON.stringify(response)}`);
    ElMessage.success('登录请求已发送');
  } catch (error) {
    addLog('error', `登录失败: ${error}`);
    ElMessage.error('登录失败');
  }
}

// 获取车辆
async function getVehicles() {
  try {
    addLog('info', '获取车辆列表...');

    const service = window.$newServerService;
    if (!service) {
      throw new Error('服务不可用');
    }

    // 您的代码应用：发送消息
    const response = await service.send({
      type: 'command',
      command: 'get_vehicles',
      timestamp: Date.now(),
      payload: {
        page: 1,
        pageSize: 10,
      },
    });

    addLog('success', `车辆列表: ${JSON.stringify(response)}`);
    ElMessage.success('车辆请求已发送');
  } catch (error) {
    addLog('error', `获取失败: ${error}`);
    ElMessage.error('获取失败');
  }
}

// 切换协议（您的代码应用）
async function switchProtocol(protocol: 'tcp' | 'websocket') {
  try {
    addLog('info', `正在切换到 ${protocol.toUpperCase()}...`);

    const service = window.$newServerService;
    if (!service) {
      throw new Error('服务不可用');
    }

    // 您的代码应用：切换协议
    const success = await service.switchProtocol(protocol);

    if (success) {
      addLog('success', `已切换到 ${protocol.toUpperCase()}`);
      ElMessage.success(`协议已切换到 ${protocol.toUpperCase()}`);
      refreshStatus();
    } else {
      addLog('error', '协议切换失败');
      ElMessage.error('协议切换失败');
    }
  } catch (error) {
    addLog('error', `切换失败: ${error}`);
    ElMessage.error('切换异常');
  }
}

// 刷新状态
function refreshStatus() {
  const service = window.$newServerService;

  if (service) {
    isConnected.value = service.isConnected();
    currentProtocol.value = service.getCurrentProtocol();

    const stats = service.getStats();
    addLog('info', `状态: ${stats.state}, 协议: ${stats.protocol}`);
  } else {
    isConnected.value = false;
    currentProtocol.value = 'disconnected';
  }
}

// 添加日志
function addLog(type: 'info' | 'success' | 'error' | 'warning', message: string) {
  messageLogs.value.unshift({
    timestamp: Date.now(),
    type,
    message,
  });

  // 保持最近50条
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
.unified-communication-demo {
  max-width: 900px;
  margin: 20px auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.info-section h4,
.code-example-section h4,
.quick-actions h4,
.message-log-section h4 {
  margin: 0 0 15px 0;
  color: #333;
  font-size: 14px;
  font-weight: 600;
}

.info-row {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
}

.info-row .label {
  width: 80px;
  color: #666;
  font-size: 13px;
}

.code-block {
  background: #f5f5f5;
  padding: 15px;
  border-radius: 4px;
  overflow-x: auto;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.6;
  color: #333;
}

.message-log-section {
  margin-top: 20px;
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
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



