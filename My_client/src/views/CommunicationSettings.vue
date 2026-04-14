<template>
  <div class="communication-settings">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>通信设置</span>
          <el-button type="primary" size="small" @click="handleSave">保存设置</el-button>
        </div>
      </template>
      <el-form label-width="120px" style="max-width: 600px">
        <el-form-item label="服务器IP">
          <el-input v-model="communicationSettings.serverIp" placeholder="请输入服务器IP" />
        </el-form-item>
        <el-form-item label="服务器端口">
          <el-input-number v-model="communicationSettings.serverPort" :min="1" :max="65535" />
        </el-form-item>
        <el-form-item label="心跳间隔">
          <el-input-number v-model="communicationSettings.heartbeatInterval" :min="1" :max="60" />
          <span style="margin-left: 10px">秒</span>
        </el-form-item>
        <el-form-item label="超时时间">
          <el-input-number v-model="communicationSettings.timeout" :min="1" :max="60" />
          <span style="margin-left: 10px">秒</span>
        </el-form-item>
        <el-form-item label="重连次数">
          <el-input-number v-model="communicationSettings.reconnectCount" :min="1" :max="10" />
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
  </div>
</template>

<script setup lang="ts">

// 将驼峰字段名转为蛇形
const toSnakeCase = (obj: any): any => {
  const result: any = {};
  for (const key in obj) {
    const snakeKey = key.replace(/[A-Z]/g, (m) => '_' + m[0].toLowerCase());
    result[snakeKey] = obj[key];
  }
  return result;
};

import { reactive, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import api from '@/api';

// 通信设置数据
const communicationSettings = reactive({
  serverIp: localStorage.getItem('serverIp') || '127.0.0.1',
  serverPort: parseInt(localStorage.getItem('serverPort') || '9808'),
  heartbeatInterval: 30,
  timeout: 10,
  reconnectCount: 3,
  protocol: 'tcp',
  compression: true,
  encryption: true,
});

// 加载通信设置
const loadCommunicationSettings = async () => {
  try {
    const response = await api.get('/api/settings/communication');
    if (response) {
      Object.assign(communicationSettings, response);
    }
  } catch (error) {
    console.error('加载通信设置失败:', error);
  }
};

// 保存设置
const handleSave = async () => {
  try {
    // 保存到后端API
    await api.put('/api/settings/communication', toSnakeCase(communicationSettings));

    // 保存到localStorage作为备份
    localStorage.setItem('serverIp', communicationSettings.serverIp);
    localStorage.setItem('serverPort', communicationSettings.serverPort.toString());
    localStorage.setItem('communicationSettings', JSON.stringify(communicationSettings));

    ElMessage.success('通信设置已保存');
  } catch (error) {
    console.error('保存通信设置失败:', error);
    ElMessage.error('保存通信设置失败');
  }
};

// 初始化时加载通信设置
onMounted(() => {
  loadCommunicationSettings();
});
</script>

<style scoped>
.communication-settings {
  padding: 20px;
  background-color: #f5f7fa;
  min-height: 100vh;
}
</style>


