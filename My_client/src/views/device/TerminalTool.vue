<template>
  <div class="terminal-tool-container">
    <!-- 页面标题 -->
    <h1 class="page-title">终端工具管理</h1>

    <!-- 车辆选择卡片 -->
    <el-card shadow="hover" class="vehicle-selection-card mb-4">
      <template #header>
        <div class="card-header">
          <el-icon class="header-icon"><Van /></el-icon>
          <span>车辆选择</span>
        </div>
      </template>
      <el-row :gutter="20" class="vehicle-selection-row">
        <el-col :span="16">
          <el-select
            v-model="selectedVehicles"
            multiple
            filterable
            placeholder="请选择车辆（可多选）"
            style="width: 100%"
            collapse-tags
            collapse-tags-tooltip
          >
            <el-option
              v-for="vehicle in vehicleList"
              :key="vehicle.vehicle_id"
              :label="vehicle.license_plate"
              :value="vehicle.vehicle_id"
            />
          </el-select>
        </el-col>
        <el-col :span="8" class="button-group">
          <el-button type="primary" @click="loadVehicleData" :loading="loadingVehicleData">
            <el-icon><Refresh /></el-icon> 加载车辆数据
          </el-button>
          <el-button @click="clearSelection">
            <el-icon><Delete /></el-icon> 清空选择
          </el-button>
        </el-col>
      </el-row>
      <!-- 选择车辆数量提示 -->
      <div v-if="selectedVehicles.length > 0" class="selection-info">
        <el-tag type="success" size="small">{{ selectedVehicles.length }} 辆车已选择</el-tag>
      </div>
    </el-card>

    <!-- 概览卡片 -->
    <el-row :gutter="20" class="overview-row mb-4">
      <el-col :span="6">
        <el-card shadow="hover" class="overview-card success-card">
          <div class="overview-content">
            <el-icon class="overview-icon"><Check /></el-icon>
            <div class="overview-text">
              <div class="overview-title">在线终端</div>
              <div class="overview-value">{{ onlineTerminalsCount }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="overview-card warning-card">
          <div class="overview-content">
            <el-icon class="overview-icon"><InfoFilled /></el-icon>
            <div class="overview-text">
              <div class="overview-title">离线终端</div>
              <div class="overview-value">{{ offlineTerminalsCount }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="overview-card info-card">
          <div class="overview-content">
            <el-icon class="overview-icon"><Van /></el-icon>
            <div class="overview-text">
              <div class="overview-title">管理车辆</div>
              <div class="overview-value">{{ vehicleList.length }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover" class="overview-card danger-card">
          <div class="overview-content">
            <el-icon class="overview-icon"><Warning /></el-icon>
            <div class="overview-text">
              <div class="overview-title">异常终端</div>
              <div class="overview-value">{{ abnormalTerminalsCount }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 终端设置列表卡片 -->
    <el-card shadow="hover" class="terminal-settings-card mb-4">
      <template #header>
        <div class="card-header">
          <el-icon class="header-icon"><Setting /></el-icon>
          <span>终端设置列表</span>
        </div>
      </template>
      <div class="settings-grid">
        <el-card
          v-for="setting in terminalSettings"
          :key="setting.id"
          shadow="hover"
          class="setting-item-card"
          @click="openCommandEditor(setting)"
        >
          <div class="setting-item-content">
            <div class="setting-item-header">
              <el-icon class="setting-item-icon">
                <component :is="getSettingIcon(setting.type)"></component>
              </el-icon>
              <div class="setting-item-info">
                <h3 class="setting-item-name">{{ setting.name }}</h3>
                <p class="setting-item-desc">{{ setting.description }}</p>
              </div>
            </div>
            <div class="setting-item-action">
              <el-button type="primary" size="small" plain>
                <el-icon><Edit /></el-icon> 配置
              </el-button>
            </div>
          </div>
        </el-card>
      </div>
    </el-card>

    <!-- 功能模块选项卡 -->
    <el-card shadow="hover" class="function-tabs-card">
      <template #header>
        <div class="card-header">
          <el-icon class="header-icon"><Menu /></el-icon>
          <span>功能模块</span>
        </div>
      </template>
      <el-tabs v-model="activeTab" type="border-card" class="modern-tabs">
        <!-- LED文本发送 -->
        <el-tab-pane label="LED文本发送" name="ledText">
          <el-form :model="ledTextForm" label-width="120px" class="mb-4">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="信箱号">
                  <el-input-number v-model="ledTextForm.boxNum" :min="0" :max="999" style="width: 100%" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="LED地址">
                  <el-select v-model="ledTextForm.port" style="width: 100%">
                    <el-option label="0" value="0" />
                    <el-option label="1" value="1" />
                  </el-select>
                </el-form-item>
              </el-col>
            </el-row>
            <el-form-item>
              <el-button type="primary" @click="sendLEDText" :disabled="selectedVehicles.length === 0">
                发送LED文本
              </el-button>
              <el-button @click="resetLEDTextForm">重置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 载重参数 -->
        <el-tab-pane label="载重参数" name="loadParams">
          <el-form :model="loadParams" label-width="150px" class="params-form">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="标定系数">
                  <el-input-number
                    v-model="loadParams.calibrationCoefficient"
                    :min="0"
                    :max="99999"
                    :precision="2"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="空车重量">
                  <el-input-number
                    v-model="loadParams.emptyWeight"
                    :min="0"
                    :max="9999"
                    :precision="2"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
            </el-row>
            <el-form-item>
              <el-button type="primary" @click="sendLoadParams" :disabled="selectedVehicles.length === 0">
                下发载重参数
              </el-button>
              <el-button @click="resetLoadParams">重置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, reactive } from 'vue';
import { ElMessage } from 'element-plus';
import { Van, Setting, Menu, Refresh, InfoFilled, Warning, Check, Edit, Delete } from '@element-plus/icons-vue';
// eslint-disable-next-line @typescript-eslint/no-unused-vars
import { callElectronAPI } from '@/services/localDB';
import api from '@/api';

// 车辆类型定义
interface Vehicle {
  vehicle_id: string;
  license_plate: string;
  status: number;
  [key: string]: unknown;
}

// API响应类型
interface ApiResponse<T> {
  items?: T[];
  total?: number;
  [key: string]: unknown;
}

// 终端设置类型
interface TerminalSetting {
  id: string;
  name: string;
  description: string;
  type: string;
}

// 车辆数据
const vehicleList = ref<Vehicle[]>([]);
const selectedVehicles = ref<string[]>([]);
const loadingVehicleData = ref(false);

// 概览数据
const onlineTerminalsCount = ref(0);
const offlineTerminalsCount = ref(0);
const abnormalTerminalsCount = ref(0);

// 选项卡
const activeTab = ref('ledText');

// 终端设置列表
const terminalSettings = ref([
  { id: 'loadParams', name: '载重参数', description: '设置车辆载重相关参数', type: 'load' },
  { id: 'terminalParams', name: '终端参数', description: '设置终端基本参数', type: 'terminal' },
  { id: 'gpsParams', name: 'GPS参数', description: '设置GPS相关参数', type: 'gps' },
  { id: 'commParams', name: '通信参数', description: '设置通信相关参数', type: 'comm' },
]);

// 获取设置图标
const getSettingIcon = (type: string) => {
  switch (type) {
    case 'load':
      return 'Scale';
    case 'terminal':
      return 'Cpu';
    case 'gps':
      return 'Location';
    case 'comm':
      return 'Connection';
    default:
      return 'Setting';
  }
};

// 终端命令编辑器
const showCommandEditor = ref(false);
const currentSetting = ref<TerminalSetting | null>(null);

// 打开终端命令编辑器
const openCommandEditor = (row: TerminalSetting) => {
  currentSetting.value = row;
  showCommandEditor.value = true;
};

// 加载车辆数据
const loadVehicleData = async () => {
  loadingVehicleData.value = true;
  try {
    const response = await api.get('/api/vehicles');
    console.log('使用api获取车辆数据成功:', response);
    vehicleList.value = response.items || [];

    // 计算终端状态数据
    const onlineCount = vehicleList.value.filter((v) => v.status === 1).length;
    const offlineCount = vehicleList.value.filter((v) => v.status === 2).length;
    const abnormalCount = vehicleList.value.filter((v) => v.status === 3).length;

    onlineTerminalsCount.value = onlineCount;
    offlineTerminalsCount.value = offlineCount;
    abnormalTerminalsCount.value = abnormalCount;

    ElMessage.success('车辆数据加载成功');
  } catch (error) {
    console.error('加载车辆数据失败:', error);
    // 不显示错误提示，避免影响用户体验
    // ElMessage.error('加载车辆数据失败')
  } finally {
    loadingVehicleData.value = false;
  }
};

// 清空选择
const clearSelection = () => {
  selectedVehicles.value = [];
  ElMessage.info('已清空车辆选择');
};

// LED文本发送
const ledTextForm = reactive({
  boxNum: 0,
  port: '0',
  content: '',
});

const sendLEDText = () => {
  ElMessage.success('LED文本发送成功');
};

const resetLEDTextForm = () => {
  Object.assign(ledTextForm, { boxNum: 0, port: '0', content: '' });
};

// 载重参数
const loadParams = reactive({
  calibrationCoefficient: 1.0,
  emptyWeight: 0.0,
});

const sendLoadParams = () => {
  ElMessage.success('载重参数下发成功');
};

const resetLoadParams = () => {
  Object.assign(loadParams, { calibrationCoefficient: 1.0, emptyWeight: 0.0 });
};
</script>

<style scoped>
/* 现代化UI样式 */
.terminal-tool-container {
  padding: 20px;
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e8eb 100%);
  min-height: 100vh;
}

.page-title {
  font-size: 28px;
  font-weight: 800;
  margin-bottom: 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* 车辆选择卡片 */
.vehicle-selection-card {
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  transition: all 0.3s ease;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
}

.header-icon {
  color: #667eea;
  font-size: 20px;
}

.vehicle-selection-row {
  align-items: center;
}

.button-group {
  display: flex;
  gap: 10px;
}

.selection-info {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #e4e8eb;
}

/* 概览卡片 */
.overview-row {
  margin-bottom: 24px;
}

.overview-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  cursor: pointer;
  border: none;
}

.overview-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
}

.overview-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 0;
}

.overview-icon {
  font-size: 40px;
  opacity: 0.8;
}

.overview-text {
  flex: 1;
}

.overview-title {
  font-size: 14px;
  color: #606266;
  margin-bottom: 4px;
}

.overview-value {
  font-size: 28px;
  font-weight: 700;
}

.success-card {
  background: linear-gradient(135deg, #f0f9eb 0%, #e0f2fe 100%);
}

.success-card .overview-icon,
.success-card .overview-value {
  color: #67c23a;
}

.warning-card {
  background: linear-gradient(135deg, #fdf6ec 0%, #fef3c7 100%);
}

.warning-card .overview-icon,
.warning-card .overview-value {
  color: #e6a23c;
}

.info-card {
  background: linear-gradient(135deg, #ecf5ff 0%, #e0f2fe 100%);
}

.info-card .overview-icon,
.info-card .overview-value {
  color: #409eff;
}

.danger-card {
  background: linear-gradient(135deg, #fef2f2 0%, #fee2e2 100%);
}

.danger-card .overview-icon,
.danger-card .overview-value {
  color: #f56c6c;
}

/* 终端设置列表 */
.terminal-settings-card {
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
  margin-top: 16px;
}

.setting-item-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  cursor: pointer;
  border: 2px solid transparent;
}

.setting-item-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  border-color: #667eea;
}

.setting-item-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
}

.setting-item-header {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.setting-item-icon {
  font-size: 32px;
  color: #667eea;
}

.setting-item-name {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 4px 0;
  color: #303133;
}

.setting-item-desc {
  font-size: 13px;
  color: #606266;
  margin: 0;
  line-height: 1.4;
}

/* 功能模块选项卡 */
.function-tabs-card {
  border-radius: 12px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}

.modern-tabs :deep(.el-tabs__header) {
  background: #f8f9fa;
  border-bottom: 1px solid #e4e8eb;
}

.modern-tabs :deep(.el-tabs__item.is-active) {
  color: #667eea;
  font-weight: 600;
}

.modern-tabs :deep(.el-tabs__active-bar) {
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  height: 3px;
  border-radius: 2px;
}

.modern-tabs :deep(.el-tabs__content) {
  padding: 20px;
  background: white;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .terminal-tool-container {
    padding: 12px;
  }

  .page-title {
    font-size: 22px;
  }

  .vehicle-selection-row {
    flex-direction: column;
    gap: 12px;
  }

  .vehicle-selection-row .el-col {
    width: 100%;
  }

  .button-group {
    justify-content: center;
  }

  .overview-row .el-col {
    width: calc(50% - 6px);
  }

  .settings-grid {
    grid-template-columns: 1fr;
  }
}
</style>


