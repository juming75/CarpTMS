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
            <el-form-item label="文本内容">
              <el-input
                v-model="ledTextForm.content"
                type="textarea"
                :rows="3"
                placeholder="请输入要显示的LED文本内容（最多50个字符）"
                maxlength="50"
                show-word-limit
              />
            </el-form-item>
            <el-form-item label="显示模式">
              <el-radio-group v-model="ledTextForm.displayMode">
                <el-radio value="static">静态显示</el-radio>
                <el-radio value="scroll">滚动显示</el-radio>
                <el-radio value="flash">闪烁显示</el-radio>
              </el-radio-group>
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="sendLEDText" :disabled="!canSendLED || selectedVehicles.length === 0" :loading="sendingLED">
                发送LED文本
              </el-button>
              <el-button @click="resetLEDTextForm">重置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 载重参数 -->
        <el-tab-pane label="载重参数" name="loadParams">
          <el-form :model="loadParams" label-width="150px" class="params-form">
            <el-divider content-position="left">基础参数</el-divider>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="标定系数">
                  <el-input-number
                    v-model="loadParams.calibrationCoefficient"
                    :min="0"
                    :max="99999"
                    :precision="2"
                    :step="0.01"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="空车重量(kg)">
                  <el-input-number
                    v-model="loadParams.emptyWeight"
                    :min="0"
                    :max="99999"
                    :precision="2"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
            </el-row>
            <el-divider content-position="left">高级参数</el-divider>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="满载重量(kg)">
                  <el-input-number
                    v-model="loadParams.fullLoadWeight"
                    :min="0"
                    :max="199999"
                    :precision="2"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="超载阈值(%)">
                  <el-input-number
                    v-model="loadParams.overloadThreshold"
                    :min="0"
                    :max="200"
                    :precision="1"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
            </el-row>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="采样间隔(ms)">
                  <el-input-number
                    v-model="loadParams.samplingInterval"
                    :min="100"
                    :max="10000"
                    :step="100"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="滤波系数">
                  <el-input-number
                    v-model="loadParams.filterFactor"
                    :min="0"
                    :max="1"
                    :precision="2"
                    :step="0.01"
                    style="width: 100%"
                  />
                </el-form-item>
              </el-col>
            </el-row>
            <el-form-item>
              <el-button type="primary" @click="sendLoadParams" :disabled="selectedVehicles.length === 0" :loading="sendingLoadParams">
                下发载重参数
              </el-button>
              <el-button @click="resetLoadParams">重置</el-button>
              <el-button @click="loadCurrentLoadParams">读取当前参数</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 终端参数 -->
        <el-tab-pane label="终端参数" name="terminalParams">
          <el-form :model="terminalParams" label-width="150px" class="params-form">
            <el-divider content-position="left">基本设置</el-divider>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="心跳间隔(秒)">
                  <el-input-number v-model="terminalParams.heartbeatInterval" :min="10" :max="300" style="width: 100%" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="TCP超时(秒)">
                  <el-input-number v-model="terminalParams.tcpTimeout" :min="5" :max="120" style="width: 100%" />
                </el-form-item>
              </el-col>
            </el-row>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="定位间隔(秒)">
                  <el-input-number v-model="terminalParams.locationInterval" :min="5" :max="3600" style="width: 100%" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="休眠间隔(秒)">
                  <el-input-number v-model="terminalParams.sleepInterval" :min="30" :max="3600" style="width: 100%" />
                </el-form-item>
              </el-col>
            </el-row>
            <el-divider content-position="left">报警设置</el-divider>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="超速阈值(km/h)">
                  <el-input-number v-model="terminalParams.speedThreshold" :min="60" :max="200" style="width: 100%" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="疲劳驾驶(小时)">
                  <el-input-number v-model="terminalParams.fatigueThreshold" :min="1" :max="24" :precision="1" :step="0.5" style="width: 100%" />
                </el-form-item>
              </el-col>
            </el-row>
            <el-form-item>
              <el-button type="primary" @click="sendTerminalParams" :disabled="selectedVehicles.length === 0" :loading="sendingTerminalParams">
                下发终端参数
              </el-button>
              <el-button @click="resetTerminalParams">重置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- GPS参数 -->
        <el-tab-pane label="GPS参数" name="gpsParams">
          <el-form :model="gpsParams" label-width="150px" class="params-form">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="定位模式">
                  <el-select v-model="gpsParams.positionMode" style="width: 100%">
                    <el-option label="GPS定位" value="gps" />
                    <el-option label="北斗定位" value="beidou" />
                    <el-option label="GPS+北斗" value="hybrid" />
                  </el-select>
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="海拔高度补偿(m)">
                  <el-input-number v-model="gpsParams.altitudeOffset" :min="-500" :max="500" style="width: 100%" />
                </el-form-item>
              </el-col>
            </el-row>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="最小卫星数">
                  <el-input-number v-model="gpsParams.minSatellites" :min="3" :max="12" style="width: 100%" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="PDOP阈值">
                  <el-input-number v-model="gpsParams.pdopThreshold" :min="1" :max="10" :precision="1" :step="0.5" style="width: 100%" />
                </el-form-item>
              </el-col>
            </el-row>
            <el-form-item>
              <el-button type="primary" @click="sendGPSParams" :disabled="selectedVehicles.length === 0" :loading="sendingGPSParams">
                下发GPS参数
              </el-button>
              <el-button @click="resetGPSParams">重置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 通信参数 -->
        <el-tab-pane label="通信参数" name="commParams">
          <el-form :model="commParams" label-width="150px" class="params-form">
            <el-divider content-position="left">服务器配置</el-divider>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="主服务器IP">
                  <el-input v-model="commParams.primaryServerIp" placeholder="例如: 192.168.1.100" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="主服务器端口">
                  <el-input-number v-model="commParams.primaryServerPort" :min="1" :max="65535" style="width: 100%" />
                </el-form-item>
              </el-col>
            </el-row>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="备用服务器IP">
                  <el-input v-model="commParams.backupServerIp" placeholder="可选" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="备用服务器端口">
                  <el-input-number v-model="commParams.backupServerPort" :min="1" :max="65535" style="width: 100%" />
                </el-form-item>
              </el-col>
            </el-row>
            <el-divider content-position="left">APN设置</el-divider>
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="APN名称">
                  <el-input v-model="commParams.apnName" placeholder="运营商APN" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="APN用户名">
                  <el-input v-model="commParams.apnUsername" placeholder="可选" />
                </el-form-item>
              </el-col>
            </el-row>
            <el-form-item label="APN密码">
              <el-input v-model="commParams.apnPassword" type="password" show-password placeholder="可选" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="sendCommParams" :disabled="selectedVehicles.length === 0" :loading="sendingCommParams">
                下发通信参数
              </el-button>
              <el-button @click="resetCommParams">重置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
      </el-tabs>
    </el-card>

    <!-- 命令编辑器对话框 -->
    <el-dialog
      v-model="showCommandEditor"
      :title="'配置 - ' + (currentSetting?.name || '')"
      width="700px"
      destroy-on-close
    >
      <div v-if="currentSetting" class="command-editor-content">
        <el-alert
          :title="currentSetting.description"
          type="info"
          :closable="false"
          show-icon
          class="mb-4"
        />

        <!-- 根据不同设置类型显示不同的表单 -->
        <component :is="getSettingEditorComponent(currentSetting.id)" />

        <el-divider>目标车辆</el-divider>
        <el-tag
          v-for="vid in selectedVehicles"
          :key="vid"
          closable
          @close="removeVehicle(vid)"
          class="mr-2 mb-2"
        >
          {{ getVehiclePlate(vid) }}
        </el-tag>
        <el-empty v-if="selectedVehicles.length === 0" description="请先选择车辆" :image-size="60" />
      </div>

      <template #footer>
        <el-button @click="showCommandEditor = false">取消</el-button>
        <el-button
          type="primary"
          :disabled="selectedVehicles.length === 0"
          :loading="sendingCommand"
          @click="executeCommandFromEditor"
        >
          下发指令
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  Van,
  Setting,
  Menu,
  Refresh,
  InfoFilled,
  Warning,
  Check,
  Edit,
  Delete,
  Cpu,
  Location,
  Connection,
  TrendCharts,
} from '@element-plus/icons-vue';
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
  data?: T[];
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
const terminalSettings = ref<TerminalSetting[]>([
  { id: 'loadParams', name: '载重参数', description: '设置车辆载重相关参数，包括标定系数、空车重量等', type: 'load' },
  { id: 'terminalParams', name: '终端参数', description: '设置终端基本参数，包括心跳、超时、定位间隔等', type: 'terminal' },
  { id: 'gpsParams', name: 'GPS参数', description: '设置GPS相关参数，包括定位模式、卫星数要求等', type: 'gps' },
  { id: 'commParams', name: '通信参数', description: '设置通信相关参数，包括服务器IP、APN等', type: 'comm' },
]);

// 获取设置图标
const getSettingIcon = (type: string) => {
  const iconMap: Record<string, any> = {
    load: TrendCharts,
    terminal: Cpu,
    gps: Location,
    comm: Connection,
  };
  return iconMap[type] || Setting;
};

// 获取设置编辑器组件ID
const getSettingEditorComponent = (id: string) => {
  return `editor-${id}`;
};

// 终端命令编辑器
const showCommandEditor = ref(false);
const currentSetting = ref<TerminalSetting | null>(null);
const sendingCommand = ref(false);

// 打开终端命令编辑器
const openCommandEditor = (row: TerminalSetting) => {
  currentSetting.value = row;
  showCommandEditor.value = true;
  // 自动切换到对应的标签页
  activeTab.value = row.id;
};

// 加载车辆数据
const loadVehicleData = async () => {
  loadingVehicleData.value = true;
  try {
    const response = await api.get('/api/vehicles') as ApiResponse<Vehicle>;
    console.log('使用api获取车辆数据成功:', response);
    vehicleList.value = response.items || response.data || [];

    // 计算终端状态数据
    const onlineCount = vehicleList.value.filter((v) => v.status === 1).length;
    const offlineCount = vehicleList.value.filter((v) => v.status === 2).length;
    const abnormalCount = vehicleList.value.filter((v) => v.status === 3).length;

    onlineTerminalsCount.value = onlineCount;
    offlineTerminalsCount.value = offlineCount;
    abnormalTerminalsCount.value = abnormalCount;

    ElMessage.success(`车辆数据加载成功，共 ${vehicleList.value.length} 辆`);
  } catch (error) {
    console.error('加载车辆数据失败:', error);
    ElMessage.error('加载车辆数据失败');
  } finally {
    loadingVehicleData.value = false;
  }
};

// 清空选择
const clearSelection = () => {
  selectedVehicles.value = [];
  ElMessage.info('已清空车辆选择');
};

// 移除选中的车辆
const removeVehicle = (vehicleId: string) => {
  const index = selectedVehicles.value.indexOf(vehicleId);
  if (index > -1) {
    selectedVehicles.value.splice(index, 1);
  }
};

// 获取车牌号
const getVehiclePlate = (vehicleId: string): string => {
  const vehicle = vehicleList.value.find(v => v.vehicle_id === vehicleId);
  return vehicle?.license_plate || vehicleId;
};

// ==================== LED文本发送 ====================
const ledTextForm = reactive({
  boxNum: 0,
  port: '0',
  content: '',
  displayMode: 'static' as 'static' | 'scroll' | 'flash',
});
const sendingLED = ref(false);

const canSendLED = computed(() => {
  return ledTextForm.content.trim().length > 0 && ledTextForm.content.trim().length <= 50;
});

const sendLEDText = async () => {
  if (!canSendLED.value) {
    ElMessage.warning('请输入有效的LED文本内容（1-50个字符）');
    return;
  }

  try {
    await ElMessageBox.confirm(
      `确定向 ${selectedVehicles.value.length} 辆车发送LED文本吗？\n\n内容: ${ledTextForm.content}`,
      '确认发送',
      { confirmButtonText: '确定发送', cancelButtonText: '取消', type: 'warning' }
    );

    sendingLED.value = true;

    // 调用API发送LED文本
    await api.post('/api/devices/terminal/led', {
      vehicle_ids: selectedVehicles.value,
      box_number: ledTextForm.boxNum,
      port: parseInt(ledTextForm.port),
      content: ledTextForm.content.trim(),
      display_mode: ledTextForm.displayMode,
    });

    ElMessage.success('LED文本发送成功！已下发到 ' + selectedVehicles.value.length + ' 辆车');
    resetLEDTextForm();
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('LED文本发送失败:', error);
      ElMessage.error(error?.response?.data?.message || error?.message || 'LED文本发送失败');
    }
  } finally {
    sendingLED.value = false;
  }
};

const resetLEDTextForm = () => {
  Object.assign(ledTextForm, { boxNum: 0, port: '0', content: '', displayMode: 'static' });
};

// ==================== 载重参数 ====================
const loadParams = reactive({
  calibrationCoefficient: 1.0,
  emptyWeight: 0.0,
  fullLoadWeight: 0.0,
  overloadThreshold: 110.0,
  samplingInterval: 1000,
  filterFactor: 0.5,
});
const sendingLoadParams = ref(false);

const sendLoadParams = async () => {
  try {
    await ElMessageBox.confirm(
      `确定向 ${selectedVehicles.value.length} 辆车下发载重参数吗？`,
      '确认下发',
      { confirmButtonText: '确定下发', cancelButtonText: '取消', type: 'warning' }
    );

    sendingLoadParams.value = true;

    await api.post('/api/devices/terminal/load-params', {
      vehicle_ids: selectedVehicles.value,
      calibration_coefficient: loadParams.calibrationCoefficient,
      empty_weight: loadParams.emptyWeight,
      full_load_weight: loadParams.fullLoadWeight,
      overload_threshold: loadParams.overloadThreshold,
      sampling_interval: loadParams.samplingInterval,
      filter_factor: loadParams.filterFactor,
    });

    ElMessage.success('载重参数下发成功！');
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('载重参数下发失败:', error);
      ElMessage.error(error?.response?.data?.message || error?.message || '载重参数下发失败');
    }
  } finally {
    sendingLoadParams.value = false;
  }
};

const resetLoadParams = () => {
  Object.assign(loadParams, {
    calibrationCoefficient: 1.0,
    emptyWeight: 0.0,
    fullLoadWeight: 0.0,
    overloadThreshold: 110.0,
    samplingInterval: 1000,
    filterFactor: 0.5,
  });
};

const loadCurrentLoadParams = async () => {
  if (selectedVehicles.value.length === 0) {
    ElMessage.warning('请先选择一辆车');
    return;
  }

  try {
    ElMessage.info('正在读取当前载重参数...');
    // TODO: 实现从设备读取参数的逻辑
    ElMessage.success('参数读取功能开发中...');
  } catch (error) {
    console.error('读取参数失败:', error);
    ElMessage.error('读取参数失败');
  }
};

// ==================== 终端参数 ====================
const terminalParams = reactive({
  heartbeatInterval: 60,
  tcpTimeout: 30,
  locationInterval: 30,
  sleepInterval: 300,
  speedThreshold: 80,
  fatigueThreshold: 4.0,
});
const sendingTerminalParams = ref(false);

const sendTerminalParams = async () => {
  try {
    await ElMessageBox.confirm(
      `确定向 ${selectedVehicles.value.length} 辆车下发终端参数吗？`,
      '确认下发',
      { confirmButtonText: '确定下发', cancelButtonText: '取消', type: 'warning' }
    );

    sendingTerminalParams.value = true;

    await api.post('/api/devices/terminal/params', {
      vehicle_ids: selectedVehicles.value,
      heartbeat_interval: terminalParams.heartbeatInterval,
      tcp_timeout: terminalParams.tcpTimeout,
      location_interval: terminalParams.locationInterval,
      sleep_interval: terminalParams.sleepInterval,
      speed_threshold: terminalParams.speedThreshold,
      fatigue_threshold: terminalParams.fatigueThreshold,
    });

    ElMessage.success('终端参数下发成功！');
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('终端参数下发失败:', error);
      ElMessage.error(error?.response?.data?.message || error?.message || '终端参数下发失败');
    }
  } finally {
    sendingTerminalParams.value = false;
  }
};

const resetTerminalParams = () => {
  Object.assign(terminalParams, {
    heartbeatInterval: 60,
    tcpTimeout: 30,
    locationInterval: 30,
    sleepInterval: 300,
    speedThreshold: 80,
    fatigueThreshold: 4.0,
  });
};

// ==================== GPS参数 ====================
const gpsParams = reactive({
  positionMode: 'hybrid',
  altitudeOffset: 0,
  minSatellites: 4,
  pdopThreshold: 3.0,
});
const sendingGPSParams = ref(false);

const sendGPSParams = async () => {
  try {
    await ElMessageBox.confirm(
      `确定向 ${selectedVehicles.value.length} 辆车下发GPS参数吗？`,
      '确认下发',
      { confirmButtonText: '确定下发', cancelButtonText: '取消', type: 'warning' }
    );

    sendingGPSParams.value = true;

    await api.post('/api/devices/terminal/gps-params', {
      vehicle_ids: selectedVehicles.value,
      position_mode: gpsParams.positionMode,
      altitude_offset: gpsParams.altitudeOffset,
      min_satellites: gpsParams.minSatellites,
      pdop_threshold: gpsParams.pdopThreshold,
    });

    ElMessage.success('GPS参数下发成功！');
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('GPS参数下发失败:', error);
      ElMessage.error(error?.response?.data?.message || error?.message || 'GPS参数下发失败');
    }
  } finally {
    sendingGPSParams.value = false;
  }
};

const resetGPSParams = () => {
  Object.assign(gpsParams, {
    positionMode: 'hybrid',
    altitudeOffset: 0,
    minSatellites: 4,
    pdopThreshold: 3.0,
  });
};

// ==================== 通信参数 ====================
const commParams = reactive({
  primaryServerIp: '',
  primaryServerPort: 8082,
  backupServerIp: '',
  backupServerPort: 8082,
  apnName: '',
  apnUsername: '',
  apnPassword: '',
});
const sendingCommParams = ref(false);

const sendCommParams = async () => {
  if (!commParams.primaryServerIp) {
    ElMessage.warning('请填写主服务器IP地址');
    return;
  }

  try {
    await ElMessageBox.confirm(
      `确定向 ${selectedVehicles.value.length} 辆车下发通信参数吗？\n\n注意：修改通信参数可能导致终端离线！`,
      '危险操作确认',
      { confirmButtonText: '确定下发', cancelButtonText: '取消', type: 'error', confirmButtonClass: 'el-button--danger' }
    );

    sendingCommParams.value = true;

    await api.post('/api/devices/terminal/comm-params', {
      vehicle_ids: selectedVehicles.value,
      primary_server_ip: commParams.primaryServerIp,
      primary_server_port: commParams.primaryServerPort,
      backup_server_ip: commParams.backupServerIp,
      backup_server_port: commParams.backupServerPort,
      apn_name: commParams.apnName,
      apn_username: commParams.apnUsername,
      apn_password: commParams.apnPassword,
    });

    ElMessage.success('通信参数下发成功！终端可能需要重新连接。');
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('通信参数下发失败:', error);
      ElMessage.error(error?.response?.data?.message || error?.message || '通信参数下发失败');
    }
  } finally {
    sendingCommParams.value = false;
  }
};

const resetCommParams = () => {
  Object.assign(commParams, {
    primaryServerIp: '',
    primaryServerPort: 8082,
    backupServerIp: '',
    backupServerPort: 8082,
    apnName: '',
    apnUsername: '',
    apnPassword: '',
  });
};

// 从命令编辑器执行命令
const executeCommandFromEditor = async () => {
  if (!currentSetting.value) return;

  sendingCommand.value = true;
  try {
    switch (currentSetting.value.id) {
      case 'loadParams':
        await sendLoadParams();
        break;
      case 'terminalParams':
        await sendTerminalParams();
        break;
      case 'gpsParams':
        await sendGPSParams();
        break;
      case 'commParams':
        await sendCommParams();
        break;
      default:
        ElMessage.warning('未知的功能类型');
    }
    showCommandEditor.value = false;
  } finally {
    sendingCommand.value = false;
  }
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

.params-form {
  max-width: 900px;
}

.mr-2 {
  margin-right: 8px;
}

.mb-2 {
  margin-bottom: 8px;
}

.mb-4 {
  margin-bottom: 16px;
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
