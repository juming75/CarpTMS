<template>
  <div class="unified-dispatch-container">
    <div class="header">
      <h2>统一调度中心</h2>
      <div class="header-actions">
        <el-button type="primary" @click="refreshDevices">
          <el-icon><Refresh /></el-icon>
          刷新设备
        </el-button>
        <el-button type="success" @click="showCommandDialog = true">
          <el-icon><ChatDotRound /></el-icon>
          发送指令
        </el-button>
      </div>
    </div>

    <!-- 统计卡片 -->
    <el-row :gutter="16" class="stats-row">
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <el-icon :size="32" color="#409eff"><Van /></el-icon>
            <div class="stat-info">
              <div class="stat-value">{{ vehicleCount }}</div>
              <div class="stat-label">车载终端</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <el-icon :size="32" color="#67c23a"><TakeawayBox /></el-icon>
            <div class="stat-info">
              <div class="stat-value">{{ droneCount }}</div>
              <div class="stat-label">无人机</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <el-icon :size="32" color="#e6a23c"><Phone /></el-icon>
            <div class="stat-info">
              <div class="stat-value">{{ radioCount }}</div>
              <div class="stat-label">对讲机</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <el-icon :size="32" color="#909399"><Connection /></el-icon>
            <div class="stat-info">
              <div class="stat-value">{{ onlineCount }}</div>
              <div class="stat-label">在线设备</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 设备列表 -->
    <el-container class="main-content">
      <el-aside width="350px" class="device-panel">
        <el-input v-model="searchText" placeholder="搜索设备" clearable style="margin-bottom: 12px">
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
        <el-tabs v-model="activeTab">
          <el-tab-pane label="全部" name="all" />
          <el-tab-pane label="车载终端" name="vehicle" />
          <el-tab-pane label="无人机" name="drone" />
          <el-tab-pane label="对讲机" name="radio" />
        </el-tabs>
        <div class="device-list">
          <div
            v-for="device in filteredDevices"
            :key="`${device.device_type}-${device.id}`"
            class="device-card"
            :class="{ selected: selectedDevices.includes(`${device.device_type}-${device.id}`) }"
            @click="toggleDevice(device)"
          >
            <div class="device-header">
              <div class="device-icon" :class="device.device_type">
                <el-icon :size="20">
                  <Van v-if="device.device_type === 'vehicle'" />
                  <TakeawayBox v-else-if="device.device_type === 'drone'" />
                  <Phone v-else />
                </el-icon>
              </div>
              <div class="device-meta">
                <span class="device-name">{{ device.name }}</span>
                <span class="device-status" :class="device.status">
                  {{ statusText(device.status) }}
                </span>
              </div>
            </div>
            <div v-if="device.location" class="device-location">
              <el-icon><Location /></el-icon>
              <span>{{ device.location.latitude.toFixed(4) }}, {{ device.location.longitude.toFixed(4) }}</span>
            </div>
            <div class="device-details">
              <el-progress
                v-if="device.battery"
                :percentage="device.battery"
                :stroke-width="4"
                :show-text="false"
                :color="batteryColor(device.battery)"
              />
              <span class="battery-text" v-if="device.battery">{{ Math.round(device.battery) }}%</span>
            </div>
          </div>
        </div>
      </el-aside>

      <!-- 地图和调度区域 -->
      <el-main class="dispatch-area">
        <el-tabs v-model="mainTab">
          <el-tab-pane label="设备分布" name="map">
            <div class="map-placeholder">
              <el-icon :size="64" color="#c0c4cc"><MapLocation /></el-icon>
              <p>设备分布地图</p>
              <p class="hint">{{ devices.length }} 个设备在线</p>
            </div>
          </el-tab-pane>
          <el-tab-pane label="调度指令" name="commands">
            <div class="command-list">
              <el-empty v-if="commandHistory.length === 0" description="暂无调度指令" />
              <el-timeline v-else>
                <el-timeline-item
                  v-for="cmd in commandHistory"
                  :key="cmd.id"
                  :timestamp="cmd.created_at"
                  placement="top"
                >
                  <el-card>
                    <h4>{{ commandTypeText(cmd.command_type) }}</h4>
                    <p>目标: {{ cmd.target_devices.length }} 个 {{ deviceTypeText(cmd.target_type) }}</p>
                    <el-tag :type="commandStatusType(cmd.status)">{{ commandStatusText(cmd.status) }}</el-tag>
                  </el-card>
                </el-timeline-item>
              </el-timeline>
            </div>
          </el-tab-pane>
        </el-tabs>
      </el-main>
    </el-container>

    <!-- 发送指令对话框 -->
    <el-dialog v-model="showCommandDialog" title="发送调度指令" width="500px">
      <el-form :model="commandForm" label-width="80px">
        <el-form-item label="指令类型">
          <el-select v-model="commandForm.command_type" style="width: 100%">
            <el-option label="轨迹跟踪" value="track" />
            <el-option label="视频流请求" value="video_stream" />
            <el-option label="语音通话" value="voice_call" />
            <el-option label="群组通话" value="group_call" />
            <el-option label="发送消息" value="message" />
            <el-option label="返航（无人机）" value="return_home" />
            <el-option label="紧急停止" value="emergency_stop" />
            <el-option label="位置查询" value="position_query" />
          </el-select>
        </el-form-item>
        <el-form-item label="已选设备">
          <el-tag v-for="(id, idx) in selectedDevices" :key="id" closable @close="removeSelected(idx)">
            {{ id }}
          </el-tag>
          <span v-if="selectedDevices.length === 0" class="text-muted">请从左侧选择设备</span>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCommandDialog = false">取消</el-button>
        <el-button type="primary" @click="sendCommand">发送</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import {
  Refresh, ChatDotRound, Van, TakeawayBox, Phone, Connection,
  Search, Location, MapLocation
} from '@element-plus/icons-vue';
import { getDispatchDevices, sendDispatchCommand } from '@/api';

const devices = ref<any[]>([]);
const searchText = ref('');
const activeTab = ref('all');
const mainTab = ref('map');
const selectedDevices = ref<string[]>([]);
const showCommandDialog = ref(false);
const commandForm = ref({
  command_type: 'position_query',
});
const commandHistory = ref<any[]>([]);

const filteredDevices = computed(() => {
  let result = devices.value;
  if (activeTab.value !== 'all') {
    result = result.filter(d => d.device_type === activeTab.value);
  }
  if (searchText.value) {
    result = result.filter(d =>
      d.name.toLowerCase().includes(searchText.value.toLowerCase())
    );
  }
  return result;
});

const vehicleCount = computed(() => devices.value.filter(d => d.device_type === 'vehicle').length);
const droneCount = computed(() => devices.value.filter(d => d.device_type === 'drone').length);
const radioCount = computed(() => devices.value.filter(d => d.device_type === 'radio').length);
const onlineCount = computed(() => devices.value.filter(d => d.status === 'online').length);

const statusText = (status: string) => {
  const map: Record<string, string> = { online: '在线', offline: '离线', busy: '忙碌', idle: '空闲' };
  return map[status] || status;
};

const batteryColor = (battery: number) => {
  if (battery > 60) return '#67c23a';
  if (battery > 30) return '#e6a23c';
  return '#f56c6c';
};

const commandTypeText = (type: string) => {
  const map: Record<string, string> = {
    track: '轨迹跟踪', video_stream: '视频流', voice_call: '语音通话',
    group_call: '群组通话', message: '消息', return_home: '返航',
    emergency_stop: '紧急停止', position_query: '位置查询',
  };
  return map[type] || type;
};

const deviceTypeText = (type: string) => {
  const map: Record<string, string> = { vehicle: '车载终端', drone: '无人机', radio: '对讲机' };
  return map[type] || type;
};

const commandStatusType = (status: string) => {
  const map: Record<string, string> = { pending: 'info', executing: 'warning', completed: 'success', failed: 'danger' };
  return map[status] || 'info';
};

const commandStatusText = (status: string) => {
  const map: Record<string, string> = { pending: '待执行', executing: '执行中', completed: '已完成', failed: '失败' };
  return map[status] || status;
};

const toggleDevice = (device: any) => {
  const key = `${device.device_type}-${device.id}`;
  const idx = selectedDevices.value.findIndex(id => id === key);
  if (idx > -1) {
    selectedDevices.value.splice(idx, 1);
  } else {
    selectedDevices.value.push(key);
  }
};

const removeSelected = (idx: number) => {
  selectedDevices.value.splice(idx, 1);
};

const refreshDevices = async () => {
  try {
    const res: any = await getDispatchDevices();
    devices.value = (res as any).data || (res.data as any) || [];
    ElMessage.success('设备列表已刷新');
  } catch (e) {
    ElMessage.error('获取设备列表失败');
  }
};

const sendCommand = async () => {
  if (selectedDevices.value.length === 0) {
    ElMessage.warning('请选择至少一个设备');
    return;
  }
  try {
    // 提取设备ID和类型
    const targetDevices = selectedDevices.value.map(id => parseInt(id.split('-')[1]));
    const targetType = selectedDevices.value[0].split('-')[0];
    
    const res = await sendDispatchCommand({
      command_type: commandForm.value.command_type,
      target_devices: targetDevices,
      target_type: targetType,
      parameters: {},
    });
    commandHistory.value.unshift(res.data);
    showCommandDialog.value = false;
    ElMessage.success('指令已发送');
  } catch (e) {
    ElMessage.error('发送指令失败');
  }
};

onMounted(() => {
  refreshDevices();
});
</script>

<style scoped>
.unified-dispatch-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #f5f7fa;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
}

.header h2 {
  margin: 0;
  font-size: 18px;
}

.stats-row {
  padding: 16px 20px;
}

.stat-card :deep(.el-card__body) {
  padding: 16px;
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.stat-label {
  font-size: 12px;
  color: #909399;
}

.main-content {
  flex: 1;
  overflow: hidden;
}

.device-panel {
  background: #fff;
  border-right: 1px solid #e4e7ed;
  padding: 12px;
  overflow-y: auto;
}

.device-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.device-card {
  background: #f5f7fa;
  border-radius: 8px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.2s;
  border: 2px solid transparent;
}

.device-card:hover {
  background: #ecf5ff;
  border-color: #d9ecff;
}

.device-card.selected {
  background: #ecf5ff;
  border-color: #409eff;
}

.device-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.device-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
}

.device-icon.vehicle { background: #409eff; }
.device-icon.drone { background: #67c23a; }
.device-icon.radio { background: #e6a23c; }

.device-meta {
  flex: 1;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.device-name {
  font-weight: 500;
  color: #303133;
}

.device-status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
}

.device-status.online { background: #f0f9eb; color: #67c23a; }
.device-status.offline { background: #f4f4f5; color: #909399; }
.device-status.busy { background: #fdf6ec; color: #e6a23c; }
.device-status.idle { background: #ecf5ff; color: #409eff; }

.device-location {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #909399;
  margin-bottom: 8px;
}

.device-details {
  display: flex;
  align-items: center;
  gap: 8px;
}

.battery-text {
  font-size: 12px;
  color: #909399;
}

.dispatch-area {
  padding: 0;
  overflow: hidden;
}

.map-placeholder {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: #fff;
  color: #909399;
}

.hint {
  font-size: 14px;
  color: #c0c4cc;
}

.command-list {
  padding: 20px;
}

.text-muted {
  color: #909399;
}
</style>
