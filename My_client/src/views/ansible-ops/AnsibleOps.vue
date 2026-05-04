<template>
  <div class="ansible-ops">
    <!-- 页面标题 -->
    <div class="page-header">
      <h2>自动化运维</h2>
      <div class="header-actions">
        <el-button type="primary" @click="refreshData">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- 状态概览 -->
    <el-row :gutter="20" class="status-overview">
      <el-col :span="6">
        <el-card shadow="hover">
          <div class="stat-card">
            <div class="stat-icon success"><Monitor /></div>
            <div class="stat-info">
              <div class="stat-value">{{ hostStats.online }}</div>
              <div class="stat-label">在线主机</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover">
          <div class="stat-card">
            <div class="stat-icon warning"><Cpu /></div>
            <div class="stat-info">
              <div class="stat-value">{{ hostStats.total }}</div>
              <div class="stat-label">服务器组</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover">
          <div class="stat-card">
            <div class="stat-icon info"><Document /></div>
            <div class="stat-info">
              <div class="stat-value">{{ playbooks.length }}</div>
              <div class="stat-label">可用剧本</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover">
          <div class="stat-card">
            <div class="stat-icon primary"><Timer /></div>
            <div class="stat-info">
              <div class="stat-value">{{ recentExecutions }}</div>
              <div class="stat-label">今日执行</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 标签页 -->
    <el-tabs v-model="activeTab" class="main-tabs">
      <!-- 主机管理 -->
      <el-tab-pane label="主机管理" name="hosts">
        <div class="tab-content">
          <el-card>
            <template #header>
              <div class="card-header">
                <span>服务器清单</span>
                <el-button type="primary" size="small" @click="pingAllHosts">
                  <el-icon><Connection /></el-icon>
                  批量 Ping
                </el-button>
              </div>
            </template>
            <el-table :data="hosts" stripe style="width: 100%">
              <el-table-column prop="name" label="主机名" width="150" />
              <el-table-column prop="ansible_host" label="IP 地址" width="150" />
              <el-table-column prop="ansible_user" label="用户" width="120" />
              <el-table-column label="状态" width="100">
                <template #default="{ row }">
                  <el-tag :type="getStatusType(row.status)" size="small">
                    {{ getStatusText(row.status) }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column label="分组" width="200">
                <template #default="{ row }">
                  <el-tag v-for="group in row.groups" :key="group" size="small" style="margin-right: 4px">
                    {{ group }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column label="操作" width="200">
                <template #default="{ row }">
                  <el-button size="small" @click="pingHost(row)">Ping</el-button>
                  <el-button size="small" type="primary" @click="quickCommand(row)">
                    命令
                  </el-button>
                </template>
              </el-table-column>
            </el-table>
          </el-card>
        </div>
      </el-tab-pane>

      <!-- Playbook 管理 -->
      <el-tab-pane label="剧本管理" name="playbooks">
        <div class="tab-content">
          <el-card>
            <template #header>
              <div class="card-header">
                <span>运维剧本</span>
              </div>
            </template>
            <el-row :gutter="20">
              <el-col v-for="pb in playbooks" :key="pb.name" :span="8">
                <el-card shadow="hover" class="playbook-card">
                  <template #header>
                    <div class="playbook-header">
                      <el-icon size="24"><Document /></el-icon>
                      <span>{{ pb.name }}</span>
                    </div>
                  </template>
                  <div class="playbook-info">
                    <p><strong>路径：</strong>{{ pb.path }}</p>
                    <p><strong>描述：</strong>{{ pb.description }}</p>
                    <p><strong>分类：</strong>{{ pb.category }}</p>
                  </div>
                  <div class="playbook-actions">
                    <el-button type="primary" @click="runPlaybook(pb, false)">
                      执行
                    </el-button>
                    <el-button @click="runPlaybook(pb, true)">
                      预演
                    </el-button>
                  </div>
                </el-card>
              </el-col>
            </el-row>
          </el-card>
        </div>
      </el-tab-pane>

      <!-- 快速命令 -->
      <el-tab-pane label="快速命令" name="command">
        <div class="tab-content">
          <el-card>
            <template #header>
              <div class="card-header">
                <span>快速命令执行</span>
              </div>
            </template>
            <el-form :model="commandForm" label-width="100px">
              <el-form-item label="目标主机">
                <el-input v-model="commandForm.hosts" placeholder="如：web_servers, all" />
              </el-form-item>
              <el-form-item label="模块">
                <el-select v-model="commandForm.module" placeholder="选择模块" style="width: 100%">
                  <el-option label="shell - 执行命令" value="shell" />
                  <el-option label="yum - 包管理" value="yum" />
                  <el-option label="apt - 包管理" value="apt" />
                  <el-option label="copy - 复制文件" value="copy" />
                  <el-option label="file - 文件操作" value="file" />
                  <el-option label="service - 服务管理" value="service" />
                  <el-option label="ping - 连通性测试" value="ping" />
                </el-select>
              </el-form-item>
              <el-form-item label="命令参数">
                <el-input
                  v-model="commandForm.args"
                  type="textarea"
                  :rows="3"
                  placeholder="输入命令参数，如：name=httpd state=present"
                />
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="executeQuickCommand" :loading="commandLoading">
                  执行命令
                </el-button>
                <el-button @click="resetCommandForm">重置</el-button>
              </el-form-item>
            </el-form>

            <!-- 命令执行结果 -->
            <div v-if="commandResults.length > 0" class="command-results">
              <h4>执行结果</h4>
              <el-table :data="commandResults" stripe>
                <el-table-column prop="host" label="主机" width="150" />
                <el-table-column label="状态" width="100">
                  <template #default="{ row }">
                    <el-tag :type="row.status === 'ok' ? 'success' : 'danger'" size="small">
                      {{ row.status }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="output" label="输出" />
              </el-table>
            </div>
          </el-card>
        </div>
      </el-tab-pane>

      <!-- 执行历史 -->
      <el-tab-pane label="执行历史" name="history">
        <div class="tab-content">
          <el-card>
            <template #header>
              <div class="card-header">
                <span>执行历史</span>
              </div>
            </template>
            <el-table :data="executionHistory" stripe style="width: 100%">
              <el-table-column prop="playbook_name" label="剧本名称" width="200" />
              <el-table-column prop="user_name" label="执行人" width="120" />
              <el-table-column label="状态" width="100">
                <template #default="{ row }">
                  <el-tag :type="getExecutionStatusType(row.status)" size="small">
                    {{ row.status }}
                  </el-tag>
                </template>
              </el-table-column>
              <el-table-column label="主机数" width="80">
                <template #default="{ row }">
                  {{ row.hosts_count }}
                </template>
              </el-table-column>
              <el-table-column label="开始时间" width="180">
                <template #default="{ row }">
                  {{ formatDate(row.started_at) }}
                </template>
              </el-table-column>
              <el-table-column label="执行摘要">
                <template #default="{ row }">
                  <span v-if="row.summary">
                    成功: {{ row.summary.ok }} / 
                    变更: {{ row.summary.changed }} / 
                    失败: {{ row.summary.failed }}
                  </span>
                  <span v-else>-</span>
                </template>
              </el-table-column>
            </el-table>
          </el-card>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- Playbook 执行对话框 -->
    <el-dialog v-model="playbookDialogVisible" title="执行 Playbook" width="600px">
      <el-form :model="playbookForm" label-width="120px">
        <el-form-item label="剧本">
          <el-input v-model="playbookForm.playbook" disabled />
        </el-form-item>
        <el-form-item label="目标主机">
          <el-input v-model="playbookForm.limit" placeholder="留空表示所有主机" />
        </el-form-item>
        <el-form-item label="执行模式">
          <el-switch v-model="playbookForm.check_mode" active-text="预演模式" inactive-text="正式执行" />
        </el-form-item>
        <el-form-item label="额外变量">
          <el-input
            v-model="playbookForm.extra_vars"
            type="textarea"
            :rows="3"
            placeholder='{"key": "value"}'
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="playbookDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="confirmExecutePlaybook" :loading="executing">
          确定执行
        </el-button>
      </template>
    </el-dialog>

    <!-- 执行结果对话框 -->
    <el-dialog v-model="resultDialogVisible" title="执行结果" width="80%">
      <div v-if="executionResult" class="execution-result">
        <el-alert
          :title="executionResult.status === 'success' ? '执行成功' : '执行失败'"
          :type="executionResult.status === 'success' ? 'success' : 'error'"
          :closable="false"
        />
        <div class="result-summary">
          <el-row :gutter="20">
            <el-col :span="4">
              <div class="summary-item">
                <span class="label">总主机</span>
                <span class="value">{{ executionResult.summary.total }}</span>
              </div>
            </el-col>
            <el-col :span="4">
              <div class="summary-item success">
                <span class="label">成功</span>
                <span class="value">{{ executionResult.summary.ok }}</span>
              </div>
            </el-col>
            <el-col :span="4">
              <div class="summary-item warning">
                <span class="label">变更</span>
                <span class="value">{{ executionResult.summary.changed }}</span>
              </div>
            </el-col>
            <el-col :span="4">
              <div class="summary-item danger">
                <span class="label">失败</span>
                <span class="value">{{ executionResult.summary.failed }}</span>
              </div>
            </el-col>
          </el-row>
        </div>
        <div class="result-details">
          <h4>详细信息</h4>
          <el-table :data="executionResult.results" max-height="300">
            <el-table-column prop="host" label="主机" width="150" />
            <el-table-column prop="task_name" label="任务" width="200" />
            <el-table-column label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="getTaskStatusType(row.status)" size="small">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="变更" width="80">
              <template #default="{ row }">
                <el-tag :type="row.changed ? 'warning' : 'info'" size="small">
                  {{ row.changed ? '是' : '否' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="output" label="输出" />
          </el-table>
        </div>
        <div class="result-logs">
          <h4>执行日志</h4>
          <pre>{{ executionResult.logs }}</pre>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import {
  Refresh,
  Cpu,
  Monitor,
  Document,
  Timer,
  Connection,
} from '@element-plus/icons-vue';
import {
  listHosts,
  listGroups,
  listPlaybooks,
  executePlaybook,
  executeCommand,
  pingHosts,
} from '@/api/ansible';
import type {
  Host,
  ServerGroup,
  PlaybookInfo,
  PlaybookResult,
  TaskResult,
  ExecutionHistory,
} from '@/types/ansible';

// 状态
const activeTab = ref('hosts');
const loading = ref(false);
const commandLoading = ref(false);
const executing = ref(false);

// 统计数据
const hostStats = reactive({
  online: 0,
  total: 0,
});

// 数据
const hosts = ref<Host[]>([]);
const groups = ref<ServerGroup[]>([]);
const playbooks = ref<PlaybookInfo[]>([]);
const executionHistory = ref<ExecutionHistory[]>([]);
const commandResults = ref<TaskResult[]>([]);

// 对话框状态
const playbookDialogVisible = ref(false);
const resultDialogVisible = ref(false);
const executionResult = ref<PlaybookResult | null>(null);

// 表单数据
const commandForm = reactive({
  hosts: 'all',
  module: 'shell',
  args: 'uptime',
  inventory: 'inventory/prod/hosts.yaml',
});

const playbookForm = reactive({
  playbook: '',
  path: '',
  limit: '',
  check_mode: false,
  extra_vars: '',
});

// 计算属性
const recentExecutions = computed(() => {
  const today = new Date().toDateString();
  return executionHistory.value.filter((h) => {
    return new Date(h.started_at).toDateString() === today;
  }).length;
});

// 方法
const getStatusType = (status: string) => {
  switch (status) {
    case 'online':
      return 'success';
    case 'offline':
      return 'danger';
    case 'unreachable':
      return 'warning';
    default:
      return 'info';
  }
};

const getStatusText = (status: string) => {
  switch (status) {
    case 'online':
      return '在线';
    case 'offline':
      return '离线';
    case 'unreachable':
      return '不可达';
    default:
      return '未知';
  }
};

const getExecutionStatusType = (status: string) => {
  switch (status) {
    case 'success':
      return 'success';
    case 'failed':
      return 'danger';
    case 'running':
      return 'primary';
    case 'pending':
      return 'info';
    default:
      return 'warning';
  }
};

const getTaskStatusType = (status: string) => {
  switch (status) {
    case 'ok':
      return 'success';
    case 'changed':
      return 'warning';
    case 'failed':
      return 'danger';
    case 'skipped':
      return 'info';
    default:
      return 'warning';
  }
};

const formatDate = (dateStr: string) => {
  return new Date(dateStr).toLocaleString('zh-CN');
};

// 加载数据
const loadHosts = async () => {
  try {
    hosts.value = await listHosts();
    hostStats.online = hosts.value ? hosts.value.filter((h) => h.status === 'online').length : 0;
  } catch (error) {
    console.warn('主机列表接口暂不可用:', (error as any)?.message);
    hosts.value = [];
  }
};

const loadGroups = async () => {
  try {
    groups.value = await listGroups();
    hostStats.total = groups.value ? groups.value.length : 0;
  } catch (error) {
    console.warn('服务器组接口暂不可用:', (error as any)?.message);
    groups.value = [];
  }
};

const loadPlaybooks = async () => {
  try {
    playbooks.value = await listPlaybooks();
  } catch (error) {
    console.warn('剧本列表接口暂不可用:', (error as any)?.message);
    playbooks.value = [];
  }
};

const refreshData = async () => {
  loading.value = true;
  try {
    await Promise.all([loadHosts(), loadGroups(), loadPlaybooks()]);
    ElMessage.success('刷新成功');
  } finally {
    loading.value = false;
  }
};

// 执行 Playbook
const runPlaybook = (pb: PlaybookInfo, checkMode: boolean) => {
  playbookForm.playbook = pb.name;
  playbookForm.path = pb.path;
  playbookForm.check_mode = checkMode;
  playbookForm.limit = '';
  playbookForm.extra_vars = '';
  playbookDialogVisible.value = true;
};

const confirmExecutePlaybook = async () => {
  executing.value = true;
  try {
    const result = await executePlaybook({
      playbook: playbookForm.path,
      inventory: 'inventory/prod/hosts.yaml',
      limit: playbookForm.limit || undefined,
      check_mode: playbookForm.check_mode,
      extra_vars: playbookForm.extra_vars
        ? JSON.parse(playbookForm.extra_vars)
        : undefined,
    });
    executionResult.value = result;
    resultDialogVisible.value = true;
    playbookDialogVisible.value = false;
  } catch (error: any) {
    console.error('执行 Playbook 失败:', error);
    ElMessage.error(error.message || '执行 Playbook 失败');
  } finally {
    executing.value = false;
  }
};

// 快速命令
const executeQuickCommand = async () => {
  if (!commandForm.hosts || !commandForm.args) {
    ElMessage.warning('请填写完整信息');
    return;
  }

  commandLoading.value = true;
  try {
    commandResults.value = await executeCommand({
      hosts: commandForm.hosts,
      module: commandForm.module,
      args: commandForm.args,
      inventory: commandForm.inventory,
    });
    ElMessage.success('命令执行完成');
  } catch (error: any) {
    console.error('执行命令失败:', error);
    ElMessage.error(error.message || '执行命令失败');
  } finally {
    commandLoading.value = false;
  }
};

const resetCommandForm = () => {
  commandForm.hosts = 'all';
  commandForm.module = 'shell';
  commandForm.args = 'uptime';
  commandResults.value = [];
};

// Ping 主机
const pingHost = async (host: Host) => {
  try {
    const results = await pingHosts(host.name, 'inventory/prod/hosts.yaml');
    const result = results.find((r) => r.host === host.name);
    if (result && result.status === 'ok') {
      ElMessage.success(`${host.name} Ping 成功`);
    } else {
      ElMessage.error(`${host.name} Ping 失败`);
    }
  } catch (error) {
    ElMessage.error('Ping 操作失败');
  }
};

const pingAllHosts = async () => {
  try {
    ElMessage.info('开始批量 Ping...');
    const results = await pingHosts('all', 'inventory/prod/hosts.yaml');
    const successCount = results.filter((r) => r.status === 'ok').length;
    ElMessage.success(`批量 Ping 完成: ${successCount}/${results.length} 成功`);
  } catch (error) {
    ElMessage.error('批量 Ping 失败');
  }
};

// 生命周期
onMounted(() => {
  refreshData();
});
</script>

<style scoped>
.ansible-ops {
  padding: 20px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-header h2 {
  margin: 0;
}

.status-overview {
  margin-bottom: 20px;
}

.stat-card {
  display: flex;
  align-items: center;
  padding: 10px;
}

.stat-icon {
  width: 50px;
  height: 50px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 15px;
  font-size: 24px;
  color: white;
}

.stat-icon.success {
  background: linear-gradient(135deg, #67c23a, #85ce61);
}

.stat-icon.warning {
  background: linear-gradient(135deg, #e6a23c, #f56c6c);
}

.stat-icon.info {
  background: linear-gradient(135deg, #409eff, #66b1ff);
}

.stat-icon.primary {
  background: linear-gradient(135deg, #909399, #c0c4cc);
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 28px;
  font-weight: bold;
  color: #303133;
}

.stat-label {
  font-size: 14px;
  color: #909399;
}

.main-tabs {
  background: white;
  padding: 20px;
  border-radius: 8px;
}

.tab-content {
  padding-top: 10px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.playbook-card {
  margin-bottom: 20px;
}

.playbook-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.playbook-info {
  margin-bottom: 15px;
}

.playbook-info p {
  margin: 5px 0;
  font-size: 14px;
  color: #606266;
}

.playbook-actions {
  display: flex;
  gap: 10px;
}

.command-results {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #ebeef5;
}

.command-results h4 {
  margin-bottom: 10px;
}

.execution-result {
  padding: 10px 0;
}

.result-summary {
  margin: 20px 0;
  padding: 20px;
  background: #f5f7fa;
  border-radius: 8px;
}

.summary-item {
  text-align: center;
  padding: 15px;
  background: white;
  border-radius: 8px;
}

.summary-item .label {
  display: block;
  font-size: 14px;
  color: #909399;
}

.summary-item .value {
  display: block;
  font-size: 24px;
  font-weight: bold;
  color: #303133;
}

.summary-item.success .value {
  color: #67c23a;
}

.summary-item.warning .value {
  color: #e6a23c;
}

.summary-item.danger .value {
  color: #f56c6c;
}

.result-details,
.result-logs {
  margin-top: 20px;
}

.result-details h4,
.result-logs h4 {
  margin-bottom: 10px;
}

.result-logs pre {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 15px;
  border-radius: 8px;
  max-height: 300px;
  overflow: auto;
  font-size: 12px;
}
</style>
