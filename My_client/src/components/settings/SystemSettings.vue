<template>
  <div class="system-settings">
    <!-- 基础配置 -->
    <el-card>
      <template #header>
        <div class="card-header">
          <span>基础配置</span>
          <el-button type="primary" size="small" @click="handle_save" :loading="loading">保存设置</el-button>
        </div>
      </template>

      <el-form label-width="120px" style="max-width: 600px">
        <el-form-item label="服务器地址">
          <el-input v-model="settings.serverUrl" placeholder="http://localhost:8081" />
        </el-form-item>

        <el-form-item label="数据同步间隔">
          <el-input-number v-model="settings.syncInterval" :min="1" :max="60" />
          <span style="margin-left: 10px">分钟</span>
        </el-form-item>

        <el-form-item label="自动同步">
          <el-switch v-model="settings.autoSync" />
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 注意：服务监测、OpenAPI平台、性能监控已移至独立标签页显示，避免重复 -->
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import { ElMessage } from 'element-plus';
import api from '@/api';

// 后端字段为 snake_case，前端显示用 camelCase
const settings = reactive({
  serverUrl: 'http://127.0.0.1:8081',
  syncInterval: 5,
  autoSync: true,
});

const loading = ref(false);

const load_settings = async () => {
  try {
    // 后端返回 snake_case，转为 camelCase
    const response = await api.get('/api/settings') as any;
    if (response) {
      settings.serverUrl = response.server_url ?? settings.serverUrl;
      settings.syncInterval = response.sync_interval ?? settings.syncInterval;
      settings.autoSync = response.auto_sync ?? settings.autoSync;
    }
  } catch (error) {
    console.error('加载系统设置失败:', error);
  }
};

const handle_save = async () => {
  loading.value = true;
  try {
    // 发送前将 camelCase 转为 snake_case
    const payload = {
      server_url: settings.serverUrl,
      sync_interval: settings.syncInterval,
      auto_sync: settings.autoSync,
    };
    await api.put('/api/settings', payload);
    ElMessage.success('设置已保存');
  } catch (error) {
    console.error('保存设置失败:', error);
    ElMessage.error('保存设置失败');
  } finally {
    loading.value = false;
  }
};

// 初始化加载设置
load_settings();
</script>

<style scoped>
.system-settings {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
