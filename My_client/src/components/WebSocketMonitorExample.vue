<template>
  <div class="app-container">
    <!-- 你的主应用内容 -->
    <div class="main-content">
      <h1>CarpTMS 应用</h1>
      <!-- 其他组件... -->
    </div>

    <!-- WebSocket 状态监控面板 -->
    <WebSocketMonitor />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import WebSocketMonitor from '@/components/WebSocketMonitor.vue';
import { 
  initUnifiedCommunicationService,
  getUnifiedCommunicationService 
} from '@/services/unifiedCommunicationService';

onMounted(async () => {
  try {
    // 初始化统一通信服务（带防重复保护）
    const wsService = initUnifiedCommunicationService({
      host: 'localhost',
      port: 8082,
      protocol: 'websocket', // 或 'auto'
      reconnectInterval: 3000,
      maxReconnectAttempts: 10,
      heartbeatInterval: 45, // 与服务端同步
    });

    console.log('✅ 统一通信服务初始化成功:', wsService);

    // 连接到服务器
    const connected = await wsService.connect();
    
    if (connected) {
      console.log('🎉 WebSocket连接成功！');
      
      // 注册消息处理器
      wsService.on('message', (data: any) => {
        console.log('收到消息:', data);
      });
      
      // 监听连接状态变化
      wsService.on('connected', () => {
        console.log('已连接到服务器');
      });
      
      wsService.on('disconnected', () => {
        console.warn('与服务器断开连接');
      });

      wsService.on('error', (error: any) => {
        console.error('通信错误:', error);
      });
    } else {
      console.error('❌ 无法连接到服务器');
    }
  } catch (error) {
    console.error('初始化统一通信服务失败:', error);
    
    // 如果是重复初始化错误，获取现有实例
    if (error.message?.includes('正在初始化')) {
      console.log('⏳ 服务正在初始化中，稍后再试...');
      setTimeout(() => {
        const existingService = getUnifiedCommunicationService();
        if (existingService) {
          console.log('✅ 获取到现有服务实例:', existingService);
        }
      }, 1000);
    }
  }
});
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.main-content {
  flex: 1;
  padding: 20px;
}
</style>
