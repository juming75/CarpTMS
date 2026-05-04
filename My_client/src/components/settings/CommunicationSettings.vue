<template>
  <el-card style="margin-top: 20px">
    <template #header>
      <div class="card-header">
        <span>通信设置</span>
        <el-button type="primary" size="small" @click="handle_save" :loading="loading">保存设置</el-button>
      </div>
    </template>

    <el-form label-width="120px" style="max-width: 600px">
      <el-form-item label="服务器IP">
        <el-input v-model="communicationSettings.server_ip" placeholder="请输入服务器IP" />
      </el-form-item>

      <el-form-item label="服务器端口">
        <el-input-number v-model="communicationSettings.server_port" :min="1" :max="65535" />
      </el-form-item>

      <el-form-item label="心跳间隔">
        <el-input-number v-model="communicationSettings.heartbeat_interval" :min="1" :max="60" />
        <span style="margin-left: 10px">秒</span>
      </el-form-item>

      <el-form-item label="超时时间">
        <el-input-number v-model="communicationSettings.timeout" :min="1" :max="60" />
        <span style="margin-left: 10px">秒</span>
      </el-form-item>

      <el-form-item label="重连次数">
        <el-input-number v-model="communicationSettings.reconnect_count" :min="1" :max="10" />
        <span style="margin-left: 10px">次</span>
      </el-form-item>

      <el-form-item label="通信协议">
        <el-select v-model="communicationSettings.protocol" placeholder="请选择通信协议">
          <el-option label="TCP" value="tcp" />
          <el-option label="UDP" value="udp" />
        </el-select>
      </el-form-item>

      <el-form-item label="数据压缩">
        <el-switch v-model="communicationSettings.compression" />
      </el-form-item>

      <el-form-item label="加密通信">
        <el-switch v-model="communicationSettings.encryption" />
      </el-form-item>
    </el-form>
  </el-card>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import { ElMessage } from 'element-plus';
import api from '@/api';

// 后端返回 snake_case，前端直接使用 snake_case 字段名
const communicationSettings = reactive({
  server_ip: '127.0.0.1',
  server_port: 8988,
  heartbeat_interval: 30,
  timeout: 10,
  reconnect_count: 3,
  protocol: 'tcp',
  compression: true,
  encryption: true,
});

const loading = ref(false);

const loadCommunicationSettings = async () => {
  try {
    const response = await api.get('/api/settings/communication') as any;
    if (response) {
      Object.assign(communicationSettings, response);
    }
  } catch (error) {
    console.error('加载通信设置失败:', error);
  }
};

const handle_save = async () => {
  loading.value = true;
  try {
    // 后端使用 snake_case，直接发送
    await api.put('/api/settings/communication', communicationSettings);
    ElMessage.success('设置已保存');
  } catch (error) {
    console.error('保存设置失败:', error);
    ElMessage.error('保存设置失败');
  } finally {
    loading.value = false;
  }
};

// 初始化加载设置
loadCommunicationSettings();
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
