<template>
  <div class="disaster-recovery">
    <el-card class="status-card">
      <template #header>
        <div class="card-header">
          <span>灾备状态</span>
          <el-button type="primary" @click="refreshStatus">刷新</el-button>
        </div>
      </template>
      <el-descriptions :column="2" border>
        <el-descriptions-item label="备份功能">
          <el-tag :type="drStatus.backup_enabled ? 'success' : 'danger'">
            {{ drStatus.backup_enabled ? '已启用' : '未启用' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="复制功能">
          <el-tag :type="drStatus.replication_enabled ? 'success' : 'info'">
            {{ drStatus.replication_enabled ? '已启用' : '未配置' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="故障转移">
          <el-tag :type="drStatus.failover_enabled ? 'success' : 'info'">
            {{ drStatus.failover_enabled ? '已启用' : '未配置' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="复制状态">
          <el-tag type="info">{{ drStatus.replication_status }}</el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="RPO (分钟)">
          {{ drStatus.rpo_minutes }}
        </el-descriptions-item>
        <el-descriptions-item label="RTO (分钟)">
          {{ drStatus.rto_minutes }}
        </el-descriptions-item>
        <el-descriptions-item label="最后备份时间">
          {{ drStatus.last_backup_time || '暂无' }}
        </el-descriptions-item>
        <el-descriptions-item label="最后备份大小">
          {{ drStatus.last_backup_size ? formatSize(drStatus.last_backup_size) : '暂无' }}
        </el-descriptions-item>
      </el-descriptions>
    </el-card>

    <el-card class="backup-card">
      <template #header>
        <div class="card-header">
          <span>备份管理</span>
          <el-button type="primary" @click="createBackup">创建备份</el-button>
        </div>
      </template>
      <el-table :data="backupList" v-loading="loading">
        <el-table-column prop="id" label="备份ID" width="250" />
        <el-table-column prop="backup_type" label="备份类型" width="120">
          <template #default="{ row }">
            <el-tag>{{ row.backup_type }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="180" />
        <el-table-column prop="size_bytes" label="大小" width="120">
          <template #default="{ row }">
            {{ formatSize(row.size_bytes) }}
          </template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === 'completed' ? 'success' : 'warning'">
              {{ row.status }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" fixed="right" width="200">
          <template #default="{ row }">
            <el-button type="primary" size="small" @click="restoreBackup(row.id)">恢复</el-button>
            <el-button type="danger" size="small" @click="deleteBackup(row.id)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="createBackupDialogVisible" title="创建备份" width="400px">
      <el-form :model="createBackupForm" label-width="100px">
        <el-form-item label="备份类型">
          <el-select v-model="createBackupForm.backup_type" placeholder="请选择备份类型">
            <el-option label="完整备份" value="full" />
            <el-option label="增量备份" value="incremental" />
            <el-option label="差异备份" value="differential" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createBackupDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmCreateBackup">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import api from '@/api';

const loading = ref(false);
const createBackupDialogVisible = ref(false);
const drStatus = ref({
  backup_enabled: false,
  replication_enabled: false,
  failover_enabled: false,
  last_backup_time: null as string | null,
  last_backup_size: null as number | null,
  rpo_minutes: 15,
  rto_minutes: 30,
  replication_status: 'not_configured'
});
const backupList = ref<any[]>([]);
const createBackupForm = ref({
  backup_type: 'full'
});

const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const refreshStatus = async () => {
  try {
    console.log('开始获取灾备状态');
    const response: any = await api.get('/api/dr/status');
    console.log('灾备状态响应:', response);
    const data = response.data || response;
    if (data) {
      drStatus.value = data;
    } else {
      console.error('灾备状态响应为空');
      ElMessage.error('获取灾备状态失败：响应为空');
    }
  } catch (error) {
    console.error('获取灾备状态失败:', error);
    ElMessage.error('获取灾备状态失败');
  }
};

const loadBackups = async () => {
  loading.value = true;
  try {
    console.log('开始加载备份列表');
    const response: any = await api.get('/api/dr/backups');
    console.log('备份列表响应:', response);
    const data = response.data || response;
    if (data && data.backups) {
      backupList.value = data.backups;
    } else {
      console.error('备份列表响应为空或没有backups字段');
      ElMessage.error('加载备份列表失败：响应格式不正确');
    }
  } catch (error) {
    console.error('加载备份列表失败:', error);
    ElMessage.error('加载备份列表失败');
  } finally {
    loading.value = false;
  }
};

const createBackup = () => {
  createBackupForm.value = { backup_type: 'full' };
  createBackupDialogVisible.value = true;
};

const confirmCreateBackup = async () => {
  try {
    await api.post('/api/dr/backup', createBackupForm.value);
    ElMessage.success('备份创建成功');
    createBackupDialogVisible.value = false;
    await refreshStatus();
    await loadBackups();
  } catch (error) {
    console.error('创建备份失败:', error);
    ElMessage.error('创建备份失败');
  }
};

const restoreBackup = async (backupId: string) => {
  try {
    await ElMessageBox.confirm('确定要恢复此备份吗？恢复操作将覆盖当前数据！', '确认恢复', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    });
    await api.post('/api/dr/restore', { backup_id: backupId });
    ElMessage.success('恢复成功');
    await refreshStatus();
    await loadBackups();
  } catch (error) {
    if (error !== 'cancel') {
      console.error('恢复备份失败:', error);
      ElMessage.error('恢复备份失败');
    }
  }
};

const deleteBackup = async (backupId: string) => {
  try {
    await ElMessageBox.confirm('确定要删除此备份吗？', '确认删除', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    });
    await api.delete(`/api/dr/backups/${backupId}`);
    ElMessage.success('删除成功');
    await refreshStatus();
    await loadBackups();
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除备份失败:', error);
      ElMessage.error('删除备份失败');
    }
  }
};

onMounted(async () => {
  await refreshStatus();
  await loadBackups();
});
</script>

<style scoped>
.disaster-recovery {
  padding: 20px;
}

.status-card,
.backup-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
