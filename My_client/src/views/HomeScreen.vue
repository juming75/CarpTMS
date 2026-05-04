<template>
  <div class="home-screen">
    <!-- 页面标题 -->
    <div class="screen-header">
      <h1>运营安全监控大屏</h1>
      <div class="header-info">
        <el-button type="primary" @click="handleRefresh" :loading="isRefreshing">
          <el-icon><Refresh /></el-icon>
          {{ isRefreshing ? '刷新中...' : '刷新数据' }}
        </el-button>
        <el-tag size="small" :type="serverStatus.color" effect="dark">{{ serverStatus.text }}</el-tag>
      </div>
    </div>

    <!-- 统计卡片区域 -->
    <div class="stats-section">
      <el-row :gutter="20">
        <!-- 企业总数 -->
        <el-col :span="4">
          <el-card shadow="hover" class="stat-card">
            <div class="stat-content">
              <div class="stat-label">企业总数</div>
              <div class="stat-value">{{ statsData.companyCount }}</div>
            </div>
          </el-card>
        </el-col>

        <!-- 车辆总数 -->
        <el-col :span="4">
          <el-card shadow="hover" class="stat-card">
            <div class="stat-content">
              <div class="stat-label">车辆总数</div>
              <div class="stat-value">{{ statsData.vehicleCount }}</div>
            </div>
          </el-card>
        </el-col>

        <!-- 运输行业类别分布 -->
        <el-col :span="4">
          <el-card shadow="hover" class="stat-card">
            <div class="stat-content">
              <div class="stat-label">运输行业类别分布</div>
              <div class="stat-value">{{ statsData.industryDistribution }}</div>
            </div>
          </el-card>
        </el-col>

        <!-- 主动安全车辆数 -->
        <el-col :span="4">
          <el-card shadow="hover" class="stat-card">
            <div class="stat-content">
              <div class="stat-label">主动安全车辆数</div>
              <div class="stat-value">{{ statsData.activeSafetyVehicles }}辆</div>
            </div>
          </el-card>
        </el-col>

        <!-- 今日报警数 -->
        <el-col :span="4">
          <el-card shadow="hover" class="stat-card alarm-card">
            <div class="stat-content">
              <div class="stat-label">今日报警数</div>
              <div class="stat-value">{{ statsData.todayAlarms }}条</div>
            </div>
          </el-card>
        </el-col>

        <!-- 今日查岗情况 -->
        <el-col :span="4">
          <el-card shadow="hover" class="stat-card">
            <div class="stat-content">
              <div class="stat-label">今日查岗情况</div>
              <div class="stat-value">{{ statsData.todayChecks }}次</div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 中间图表区域 -->
    <div class="charts-section">
      <el-row :gutter="20">
        <!-- 车辆上线情况 -->
        <el-col :span="12">
          <el-card shadow="hover" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>车辆上线情况</span>
              </div>
            </template>
            <div class="chart-content">
              <div class="online-status">
                <div class="status-item">
                  <span class="status-label">服务器状态</span>
                  <span class="status-value">运行中</span>
                </div>
                <div class="status-item">
                  <span class="status-label">服务器地址</span>
                  <span class="status-value">{{ serverAddress }}</span>
                </div>
                <div class="status-item">
                  <span class="status-label">时间</span>
                  <span class="status-value">{{ currentTime }}</span>
                </div>
                <div class="status-item">
                  <span class="status-label">实时在线</span>
                  <span class="status-value">{{ statsData.onlineStatus.realTime }}</span>
                </div>
                <div class="status-item">
                  <span class="status-label">今日上线</span>
                  <span class="status-value">{{ statsData.onlineStatus.today }}</span>
                </div>
                <div class="status-item">
                  <span class="status-label">3日离线</span>
                  <span class="status-value">{{ statsData.onlineStatus.threeDaysOffline }}</span>
                </div>
                <div class="status-item">
                  <span class="status-label">7日离线</span>
                  <span class="status-value">{{ statsData.onlineStatus.sevenDaysOffline }}</span>
                </div>
                <div class="status-item">
                  <span class="status-label">30日离线</span>
                  <span class="status-value">{{ statsData.onlineStatus.thirtyDaysOffline }}</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>

        <!-- 实时高中低风险总数 -->
        <el-col :span="12">
          <el-card shadow="hover" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>实时风险等级分布</span>
              </div>
            </template>
            <div class="chart-content">
              <div class="risk-levels">
                <div class="risk-item high-risk">
                  <span class="risk-label">高风险</span>
                  <span class="risk-value">{{ statsData.riskLevels.high }}</span>
                </div>
                <div class="risk-item medium-risk">
                  <span class="risk-label">中风险</span>
                  <span class="risk-value">{{ statsData.riskLevels.medium }}</span>
                </div>
                <div class="risk-item low-risk">
                  <span class="risk-label">低风险</span>
                  <span class="risk-value">{{ statsData.riskLevels.low }}</span>
                </div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 底部图表区域 -->
    <div class="bottom-charts-section">
      <el-row :gutter="20">
        <!-- 驾驶员异常分布 -->
        <el-col :span="8">
          <el-card shadow="hover" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>驾驶员异常分布</span>
              </div>
            </template>
            <div class="chart-content">
              <el-empty description="驾驶员异常分布图表"></el-empty>
            </div>
          </el-card>
        </el-col>

        <!-- 七日安全对比 -->
        <el-col :span="8">
          <el-card shadow="hover" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>七日安全对比</span>
              </div>
            </template>
            <div class="chart-content">
              <el-empty description="七日安全对比图表"></el-empty>
            </div>
          </el-card>
        </el-col>

        <!-- 近30日风险趋势 -->
        <el-col :span="8">
          <el-card shadow="hover" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>近30日风险趋势</span>
              </div>
            </template>
            <div class="chart-content">
              <el-empty description="近30日风险趋势图表"></el-empty>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 车辆运行状态表格 -->
    <div class="table-section">
      <el-card shadow="hover" class="table-card">
        <template #header>
          <div class="card-header">
            <span>车辆运行状态</span>
          </div>
        </template>
        <div class="table-content">
          <el-table :data="vehicleStatusData" stripe size="small">
            <el-table-column prop="companyName" label="企业名称"></el-table-column>
            <el-table-column prop="total" label="总数"></el-table-column>
            <el-table-column prop="online" label="上线数"></el-table-column>
            <el-table-column prop="onlineRate" label="上线率">
              <template #default="scope">
                <el-progress
                  :percentage="scope.row.onlineRate"
                  :stroke-width="8"
                  :format="(percentage) => `${percentage}%`"
                ></el-progress>
              </template>
            </el-table-column>
            <el-table-column prop="activeSafetyTotal" label="主动安全总数"></el-table-column>
            <el-table-column prop="activeSafetyOnline" label="主动安全上线数"></el-table-column>
            <el-table-column prop="activeSafetyRate" label="主动安全上线率">
              <template #default="scope">
                <el-progress
                  :percentage="scope.row.activeSafetyRate"
                  :stroke-width="8"
                  :format="(percentage) => `${percentage}%`"
                ></el-progress>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </el-card>
    </div>

    <!-- 行驶里程统计 -->
    <div class="mileage-section">
      <el-row :gutter="20">
        <el-col :span="8">
          <el-card shadow="hover" class="mileage-card">
            <div class="mileage-content">
              <div class="mileage-label">车辆安全行驶累计里程</div>
              <div class="mileage-value">{{ statsData.mileage.total }}km</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="8">
          <el-card shadow="hover" class="mileage-card">
            <div class="mileage-content">
              <div class="mileage-label">本月安全行驶累计里程</div>
              <div class="mileage-value">{{ statsData.mileage.month }}km</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="8">
          <el-card shadow="hover" class="mileage-card">
            <div class="mileage-content">
              <div class="mileage-label">本日安全行驶累计里程</div>
              <div class="mileage-value">{{ statsData.mileage.day }}km</div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted, onUnmounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Refresh } from '@element-plus/icons-vue';
import type { Vehicle } from '@/types';
import { vehicleApi } from '@/api/vehicle';

// 调试代码
console.log('vehicleApi导入:', vehicleApi);
console.log('vehicleApi.getAll:', typeof vehicleApi.getAll);

// 统计数据类型
interface StatsData {
  companyCount: number;
  vehicleCount: number;
  industryDistribution: string;
  onlineStatus: {
    realTime: number;
    today: number;
    threeDaysOffline: number;
    sevenDaysOffline: number;
    thirtyDaysOffline: number;
  };
  activeSafetyVehicles: number;
  todayAlarms: number;
  riskLevels: {
    high: number;
    medium: number;
    low: number;
  };
  todayChecks: number;
  mileage: {
    total: number;
    month: number;
    day: number;
  };
}

// 车辆运行状态类型
interface VehicleStatusItem {
  companyName: string;
  total: number;
  online: number;
  onlineRate: number;
  activeSafetyTotal: number;
  activeSafetyOnline: number;
  activeSafetyRate: number;
}

// 统计数据
const statsData = ref<StatsData>({
  companyCount: 0,
  vehicleCount: 0,
  industryDistribution: '0类',
  onlineStatus: {
    realTime: 0,
    today: 0,
    threeDaysOffline: 0,
    sevenDaysOffline: 0,
    thirtyDaysOffline: 0,
  },
  activeSafetyVehicles: 0,
  todayAlarms: 0,
  riskLevels: {
    high: 0,
    medium: 0,
    low: 0,
  },
  todayChecks: 0,
  mileage: {
    total: 0,
    month: 0,
    day: 0,
  },
});

// 车辆运行状态数据
const vehicleStatusData = ref<VehicleStatusItem[]>([]);

// 当前时间
const currentTime = ref('');

// 车辆数据
const vehicles = ref<Vehicle[]>([]);

// 刷新状态
const isRefreshing = ref(false);

// 服务器状态
const serverStatus = ref({
  text: '服务器运行中',
  color: 'success'
});

// 动态获取服务器地址（根据环境自动切换）
const serverAddress = import.meta.env.DEV 
  ? 'localhost:8082' 
  : `${window.location.hostname}:${window.location.port || (window.location.protocol === 'https:' ? '443' : '80')}`;

// 更新当前时间
const updateCurrentTime = () => {
  const now = new Date();
  currentTime.value = now.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
};

// 获取车辆数据
const fetchVehicles = async () => {
  try {
    console.log('开始获取车辆数据...');
    // vehicleApi.getAll() 已由拦截器解包，返回 { items: [], total, page, page_size }
    const response = await vehicleApi.getAll() as any;
    console.log('车辆数据API响应:', response);

    let vehicleList: any[] = [];
    if (response && Array.isArray(response.list)) {
      vehicleList = response.list;
    } else if (response && Array.isArray(response.items)) {
      vehicleList = response.items;
    } else if (response && Array.isArray(response)) {
      vehicleList = response;
    } else {
      console.error('获取车辆数据失败: 响应格式错误', response);
      ElMessage.error('获取车辆数据失败: 响应格式错误');
      return;
    }
    vehicles.value = vehicleList;
    updateStatsData();
    console.log('获取车辆数据成功，共', vehicles.value.length, '辆车辆');
  } catch (error) {
    console.error('获取车辆数据失败:', error);
    let errorMsg = '网络连接失败';
    if (error instanceof Error) {
      errorMsg = error.message;
    }
    ElMessage.error(`获取车辆数据失败: ${errorMsg}`);
  }
};

// 更新统计数据
const updateStatsData = () => {
  // 统计车辆总数
  statsData.value.vehicleCount = vehicles.value.length;

  // 这里可以根据实际需求更新其他统计数据
  // 例如，统计在线车辆数、风险等级等
};

// 刷新数据
const handleRefresh = async () => {
  try {
    isRefreshing.value = true;
    serverStatus.value = {
      text: '正在刷新数据...',
      color: 'warning'
    };
    
    await fetchVehicles();
    
    serverStatus.value = {
      text: '服务器运行中',
      color: 'success'
    };
    ElMessage.success('数据已刷新');
  } catch (error) {
    console.error('刷新数据失败:', error);
    serverStatus.value = {
      text: '服务器连接失败',
      color: 'danger'
    };
    ElMessage.error('刷新数据失败');
  } finally {
    isRefreshing.value = false;
  }
};

// 初始化数据
const initData = async () => {
  await fetchVehicles();
};

let timeTimer: number | undefined;

onMounted(() => {
  updateCurrentTime();
  timeTimer = setInterval(updateCurrentTime, 1000);
  // 初始化数据
  initData();
});

onUnmounted(() => {
  if (timeTimer) {
    clearInterval(timeTimer);
  }
});
</script>

<style scoped>
.home-screen {
  padding: 20px;
  width: 100%;
  min-height: 100%;
  overflow-y: auto;
  background-color: #f0f2f5;
}

.screen-header {
  margin-bottom: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.screen-header h1 {
  margin: 0;
  font-size: 24px;
  color: #303133;
  font-weight: bold;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 20px;
}

.current-time {
  font-size: 16px;
  color: #606266;
  font-family: 'Courier New', monospace;
  font-weight: bold;
}

/* 统计卡片样式 */
.stats-section {
  margin-bottom: 20px;
}

.stat-card {
  height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  position: relative;
  overflow: hidden;
}

.stat-card::before {
  content: '';
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: linear-gradient(
    to bottom right,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.1) 50%,
    rgba(255, 255, 255, 0) 100%
  );
  transform: rotate(45deg);
  animation: shine 3s infinite;
}

@keyframes shine {
  0% {
    transform: translateX(-100%) rotate(45deg);
  }
  100% {
    transform: translateX(100%) rotate(45deg);
  }
}

.stat-card:hover {
  transform: translateY(-5px) scale(1.02);
  box-shadow: 0 15px 30px rgba(0, 0, 0, 0.15);
  z-index: 10;
}

.stat-card:hover::before {
  animation: shine 1.5s infinite;
}

.stat-content {
  text-align: center;
}

.stat-label {
  font-size: 14px;
  opacity: 0.9;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 28px;
  font-weight: bold;
}

.alarm-card {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

/* 图表卡片样式 */
.charts-section {
  margin-bottom: 20px;
}

.bottom-charts-section {
  margin-bottom: 20px;
}

.chart-card {
  height: 300px;
  display: flex;
  flex-direction: column;
}

.chart-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fafafa;
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: bold;
  color: #303133;
}

/* 车辆上线情况样式 */
.online-status {
  display: flex;
  flex-direction: column;
  gap: 15px;
  width: 100%;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.status-item:hover {
  transform: translateX(5px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.status-label {
  font-size: 14px;
  color: #606266;
  font-weight: 500;
}

.status-value {
  font-size: 20px;
  font-weight: bold;
  color: #409eff;
}

/* 风险等级样式 */
.risk-levels {
  display: flex;
  justify-content: space-around;
  width: 100%;
  padding: 20px;
}

.risk-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 120px;
  height: 120px;
  border-radius: 50%;
  color: white;
  font-weight: bold;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.risk-item:hover {
  transform: scale(1.1);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
}

.high-risk {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.medium-risk {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
}

.low-risk {
  background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
}

.risk-label {
  font-size: 14px;
  margin-bottom: 5px;
  opacity: 0.9;
}

.risk-value {
  font-size: 32px;
  font-weight: bold;
}

/* 表格样式 */
.table-section {
  margin-bottom: 20px;
}

.table-card {
  overflow: hidden;
}

.table-content {
  background: white;
}

/* 行驶里程样式 */
.mileage-section {
  margin-bottom: 20px;
}

.mileage-card {
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
  color: white;
}

.mileage-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.12);
}

.mileage-content {
  text-align: center;
}

.mileage-label {
  font-size: 14px;
  opacity: 0.9;
  margin-bottom: 8px;
}

.mileage-value {
  font-size: 28px;
  font-weight: bold;
}
</style>


