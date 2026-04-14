<template>
  <el-card style="margin-top: 20px">
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
          <el-button
            type="primary"
            size="small"
            @click="start_service(service.name)"
            :disabled="service.status === 'running'"
          >
            启动
          </el-button>
          <el-button
            type="danger"
            size="small"
            @click="stop_service(service.name)"
            :disabled="service.status !== 'running'"
          >
            停止
          </el-button>
          <el-button type="info" size="small" @click="restart_service(service.name)"> 重启 </el-button>
        </div>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import api from '@/api';

// 服务状态类型
interface LocalService {
  name: string;
  status: string;
  details: string;
  port: number;
  [key: string]: unknown;
}

// 服务状态
const services = ref<LocalService[]>([
  {
    name: 'HTTP API服务',
    status: 'unknown',
    details: '端口: 8081',
    port: 8081,
  },
  {
    name: 'JT808网关服务',
    status: 'unknown',
    details: '端口: 8988',
    port: 8988,
  },
  {
    name: 'WebSocket服务',
    status: 'unknown',
    details: '端口: 8089',
    port: 8089,
  },
  {
    name: '客户端服务',
    status: 'unknown',
    details: '端口: 9808',
    port: 9808,
  },
]);

const loading = ref(false);

// 服务名称映射（前端显示名 -> 后端服务名）
const service_name_map: Record<string, string> = {
  'HTTP API服务': 'database',
  'JT808网关服务': 'jt808',
  'WebSocket服务': 'websocket',
  '客户端服务': 'redis',
};

// 检查服务状态
const check_service_status = async () => {
  loading.value = true;
  try {
    const response = await api.get('/api/services/status') as any;
    if (response && response.services) {
      for (const service of services.value) {
        const backend_service = response.services.find((s: any) => {
          return s.name === service_name_map[service.name];
        });
        if (backend_service) {
          service.status = backend_service.status;
        }
      }
    }
    ElMessage.success('服务状态检查完成');
  } catch (error) {
    console.error('检查服务状态失败:', error);
    ElMessage.error('检查服务状态失败');
  } finally {
    loading.value = false;
  }
};

// 启动服务
const start_service = async (service_name: string) => {
  try {
    // 将前端显示名称转换为后端服务名称
    const backend_service_name = service_name_map[service_name] || service_name;
    await api.post(`/api/services/${backend_service_name}/start`, {}) as any;
    const service = services.value.find((s) => s.name === service_name);
    if (service) {
      service.status = 'running';
    }
    ElMessage.success(`${service_name} 启动成功`);
  } catch (error) {
    console.error(`启动服务 ${service_name} 失败:`, error);
    ElMessage.error(`启动服务 ${service_name} 失败`);
  }
};

// 停止服务
const stop_service = async (service_name: string) => {
  try {
    // 将前端显示名称转换为后端服务名称
    const backend_service_name = service_name_map[service_name] || service_name;
    await api.post(`/api/services/${backend_service_name}/stop`, {}) as any;
    const service = services.value.find((s) => s.name === service_name);
    if (service) {
      service.status = 'stopped';
    }
    ElMessage.success(`${service_name} 停止成功`);
  } catch (error) {
    console.error(`停止服务 ${service_name} 失败:`, error);
    ElMessage.error(`停止服务 ${service_name} 失败`);
  }
};

// 重启服务
const restart_service = async (service_name: string) => {
  try {
    // 将前端显示名称转换为后端服务名称
    const backend_service_name = service_name_map[service_name] || service_name;
    await api.post(`/api/services/${backend_service_name}/restart`, {}) as any;
    const service = services.value.find((s) => s.name === service_name);
    if (service) {
      service.status = 'running';
    }
    ElMessage.success(`${service_name} 重启成功`);
  } catch (error) {
    console.error(`重启服务 ${service_name} 失败:`, error);
    ElMessage.error(`重启服务 ${service_name} 失败`);
  }
};

// 初始化检查服务状态
check_service_status();
</script>

<style scoped>
.service-monitor {
  padding: 10px 0;
}

.service-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  margin-bottom: 12px;
  background-color: #f9fafb;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.service-item:hover {
  transform: translateX(5px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.service-info {
  flex: 1;
}

.service-name {
  font-weight: bold;
  color: #374151;
  margin-bottom: 4px;
  font-size: 14px;
}

.service-status {
  margin-bottom: 4px;
}

.service-details {
  font-size: 12px;
  color: #6b7280;
}

.service-actions {
  display: flex;
  gap: 8px;
}

@media (max-width: 768px) {
  .service-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .service-actions {
    align-self: flex-end;
  }
}
</style>
