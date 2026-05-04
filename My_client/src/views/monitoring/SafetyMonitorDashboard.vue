<template>
  <div class="safety-dashboard">
    <!-- 顶部标题 -->
    <div class="dashboard-header">
      <h1 class="dashboard-title">企业运营安全监管大屏</h1>
      <div class="header-info">
        <el-date-picker
          v-model="currentDate"
          type="datetime"
          readonly
          format="YYYY-MM-DD HH:mm:ss"
          value-format="YYYY-MM-DD HH:mm:ss"
          style="font-size: 16px; font-weight: bold"
        />
        <el-tag type="success" size="large">实时更新</el-tag>
      </div>
    </div>

    <!-- 统计概览 -->
    <el-row :gutter="20" class="mb-24">
      <el-col :xs="24" :sm="12" :md="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon safe">
              <el-icon><Check /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ safeVehicles }}</div>
              <div class="stat-label">安全车辆</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="12" :md="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon warning">
              <el-icon><WarningFilled /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ warningVehicles }}</div>
              <div class="stat-label">预警车辆</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="12" :md="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon danger">
              <el-icon><Close /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ dangerVehicles }}</div>
              <div class="stat-label">危险车辆</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="12" :md="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon total">
              <el-icon><Van /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ totalVehicles }}</div>
              <div class="stat-label">总车辆数</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 中间区域 -->
    <el-row :gutter="20" class="mb-24">
      <!-- 安全评分趋势 -->
      <el-col :xs="24" :lg="12">
        <el-card shadow="hover" class="chart-card">
          <template #header>
            <div class="card-header">
              <span>安全评分趋势</span>
              <el-select v-model="timeRange" size="small" @change="handleTimeRangeChange">
                <el-option label="今日" value="today" />
                <el-option label="本周" value="week" />
                <el-option label="本月" value="month" />
                <el-option label="本年" value="year" />
              </el-select>
            </div>
          </template>
          <div ref="safetyScoreRef" class="chart-container"></div>
        </el-card>
      </el-col>

      <!-- 风险分布 -->
      <el-col :xs="24" :lg="12">
        <el-card shadow="hover" class="chart-card">
          <template #header>
            <span>风险分布</span>
          </template>
          <div class="risk-distribution">
            <div ref="riskPieRef" class="pie-chart"></div>
            <div class="risk-details">
              <div class="risk-item" v-for="item in riskData" :key="item.name">
                <div class="risk-item-header">
                  <span class="risk-name">{{ item.name }}</span>
                  <span class="risk-value">{{ item.value }}%</span>
                </div>
                <el-progress
                  :percentage="item.value"
                  :color="item.color"
                  :stroke-width="10"
                  :show-text="false"
                  class="risk-progress"
                />
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 底部区域 -->
    <el-row :gutter="20">
      <!-- 实时预警信息 -->
      <el-col :xs="24" :lg="8">
        <el-card shadow="hover" class="alert-card">
          <template #header>
            <div class="card-header">
              <span>实时预警信息</span>
              <el-button type="primary" size="small" @click="handleClearAlerts"> 清空 </el-button>
            </div>
          </template>
          <el-scrollbar height="300px">
            <div class="alert-list">
              <div v-for="alert in alerts" :key="alert.id" class="alert-item" :class="`alert-${alert.level}`">
                <div class="alert-header">
                  <el-tag
                    :type="alert.level === 'high' ? 'danger' : alert.level === 'medium' ? 'warning' : 'info'"
                    size="small"
                  >
                    {{ alert.level === 'high' ? '高危' : alert.level === 'medium' ? '中危' : '低危' }}
                  </el-tag>
                  <span class="alert-time">{{ alert.time }}</span>
                </div>
                <div class="alert-content">
                  <div class="alert-vehicle">{{ alert.vehicle }}</div>
                  <div class="alert-desc">{{ alert.description }}</div>
                </div>
              </div>
              <div v-if="alerts.length === 0" class="no-alerts">
                <el-empty description="暂无预警信息" />
              </div>
            </div>
          </el-scrollbar>
        </el-card>
      </el-col>

      <!-- 安全指标 -->
      <el-col :xs="24" :lg="8">
        <el-card shadow="hover" class="metrics-card">
          <template #header>
            <span>安全指标</span>
          </template>
          <div class="metrics-grid">
            <div class="metric-item" v-for="metric in safetyMetrics" :key="metric.name">
              <div class="metric-label">{{ metric.name }}</div>
              <div class="metric-value">{{ metric.value }}</div>
              <div class="metric-trend" :class="metric.trend < 0 ? 'trend-down' : 'trend-up'">
                <el-icon>{{ metric.trend < 0 ? 'ArrowDown' : 'ArrowUp' }}</el-icon>
                <span>{{ Math.abs(metric.trend) }}%</span>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 车辆状态分布 -->
      <el-col :xs="24" :lg="8">
        <el-card shadow="hover" class="chart-card">
          <template #header>
            <span>车辆状态分布</span>
          </template>
          <div ref="vehicleStatusRef" class="chart-container"></div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
// 按需导入ECharts核心模块和需要的图表类型
import * as echarts from 'echarts/core';
import { PieChart, BarChart, LineChart } from 'echarts/charts';
import { TitleComponent, TooltipComponent, LegendComponent, GridComponent, DatasetComponent, TransformComponent } from 'echarts/components';
import { LabelLayout, UniversalTransition } from 'echarts/features';
import { CanvasRenderer } from 'echarts/renderers';

// 注册必要的组件
echarts.use([
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DatasetComponent,
  TransformComponent,
  PieChart,
  BarChart,
  LineChart,
  LabelLayout,
  UniversalTransition,
  CanvasRenderer,
]);
import { ElMessage } from 'element-plus';
import { Check, WarningFilled, Close, Van } from '@element-plus/icons-vue';

// 当前时间
const currentDate = ref(new Date());

// 时间范围
const timeRange = ref('today');

// 统计数据
const safeVehicles = ref(0);
const warningVehicles = ref(0);
const dangerVehicles = ref(0);
const totalVehicles = ref(0);

// 图表引用
const safetyScoreRef = ref<HTMLElement>();
const riskPieRef = ref<HTMLElement>();
const vehicleStatusRef = ref<HTMLElement>();

// 图表实例
let safetyScoreChart: echarts.ECharts | null = null;
let riskPieChart: echarts.ECharts | null = null;
let vehicleStatusChart: echarts.ECharts | null = null;

// 风险数据类型
interface RiskDataItem {
  name: string;
  value: number;
  color: string;
}

// 预警信息类型
interface AlertItem {
  id: number;
  level: string;
  time: string;
  vehicle: string;
  description: string;
}

// 安全指标类型
interface SafetyMetric {
  name: string;
  value: number;
  trend: number;
}

// 风险数据
const riskData = reactive<RiskDataItem[]>([]);

// 预警信息
const alerts = ref<AlertItem[]>([]);

// 安全指标
const safetyMetrics = ref<SafetyMetric[]>([]);

// 初始化安全评分趋势图
const initSafetyScoreChart = () => {
  if (!safetyScoreRef.value) return;

  safetyScoreChart = echarts.init(safetyScoreRef.value);

  const option = {
    tooltip: {
      trigger: 'axis',
      formatter: '{b}: {c}分',
    },
    xAxis: {
      type: 'category',
      data: [],
    },
    yAxis: {
      type: 'value',
      min: 0,
      max: 100,
      name: '安全评分',
    },
    series: [
      {
        data: [],
        type: 'line',
        smooth: true,
        symbol: 'circle',
        symbolSize: 8,
        itemStyle: {
          color: '#10b981',
        },
        lineStyle: {
          width: 3,
          color: '#10b981',
        },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(16, 185, 129, 0.3)' },
            { offset: 1, color: 'rgba(16, 185, 129, 0.1)' },
          ]),
        },
      },
    ],
  };

  safetyScoreChart.setOption(option);
};

// 初始化风险分布饼图
const initRiskPieChart = () => {
  if (!riskPieRef.value) return;

  riskPieChart = echarts.init(riskPieRef.value);

  const option = {
    tooltip: {
      trigger: 'item',
      formatter: '{b}: {c}% ({d}%)',
    },
    legend: {
      orient: 'vertical',
      right: 10,
      top: 'center',
      show: false,
    },
    series: [
      {
        name: '风险分布',
        type: 'pie',
        radius: ['40%', '70%'],
        center: ['35%', '50%'],
        avoidLabelOverlap: false,
        itemStyle: {
          borderRadius: 10,
          borderColor: '#fff',
          borderWidth: 2,
        },
        label: {
          show: false,
        },
        emphasis: {
          label: {
            show: true,
            fontSize: 20,
            fontWeight: 'bold',
          },
        },
        labelLine: {
          show: false,
        },
        data: riskData,
      },
    ],
  };

  riskPieChart.setOption(option);
};

// 初始化车辆状态分布
const initVehicleStatusChart = () => {
  if (!vehicleStatusRef.value) return;

  vehicleStatusChart = echarts.init(vehicleStatusRef.value);

  const option = {
    tooltip: {
      trigger: 'item',
      formatter: '{b}: {c}辆 ({d}%)',
    },
    xAxis: {
      type: 'category',
      data: ['安全', '预警', '危险', '离线'],
    },
    yAxis: {
      type: 'value',
      name: '车辆数量',
    },
    series: [
      {
        data: [0, 0, 0, 0],
        type: 'bar',
        barWidth: '60%',
        itemStyle: {
          color: function (params: { dataIndex: number }) {
            const colors = ['#10b981', '#f59e0b', '#ef4444', '#9ca3af'];
            return colors[params.dataIndex];
          },
        },
        label: {
          show: true,
          position: 'top',
        },
      },
    ],
  };

  vehicleStatusChart.setOption(option);
};

// 处理时间范围变化
const handleTimeRangeChange = () => {
  // 模拟数据更新
  console.log('时间范围变化:', timeRange.value);
  ElMessage.info('数据已更新');
};

// 处理清空预警
const handleClearAlerts = () => {
  alerts.value = [];
  ElMessage.success('预警信息已清空');
};

// 更新当前时间
const updateCurrentDate = () => {
  currentDate.value = new Date();
};

// 窗口大小变化时调整图表
const handleResize = () => {
  safetyScoreChart?.resize();
  riskPieChart?.resize();
  vehicleStatusChart?.resize();
};

// 模拟数据更新（已禁用）
const simulateDataUpdate = () => {
  // 已清理模拟数据，使用真实API数据
};

// 定时器
let dateTimer: number | null = null;
let dataTimer: number | null = null;

onMounted(() => {
  // 初始化图表
  initSafetyScoreChart();
  initRiskPieChart();
  initVehicleStatusChart();

  // 监听窗口大小变化
  window.addEventListener('resize', handleResize);

  // 更新当前时间
  updateCurrentDate();
  dateTimer = window.setInterval(updateCurrentDate, 1000);

  // 模拟数据更新（每30秒）
  dataTimer = window.setInterval(simulateDataUpdate, 30000);
});

onUnmounted(() => {
  // 清理定时器
  if (dateTimer) {
    clearInterval(dateTimer);
    dateTimer = null;
  }
  if (dataTimer) {
    clearInterval(dataTimer);
    dataTimer = null;
  }

  // 移除事件监听
  window.removeEventListener('resize', handleResize);

  // 销毁图表
  safetyScoreChart?.dispose();
  riskPieChart?.dispose();
  vehicleStatusChart?.dispose();
});
</script>

<style scoped>
.safety-dashboard {
  padding: 20px;
  background-color: #f5f7fa;
  min-height: 100vh;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.dashboard-title {
  font-size: 28px;
  font-weight: bold;
  color: #1f2937;
  margin: 0;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.mb-24 {
  margin-bottom: 24px;
}

/* 统计卡片 */
.stat-card {
  height: 100%;
  transition: all 0.3s ease;
}

.stat-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 0;
}

.stat-icon {
  width: 60px;
  height: 60px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  color: white;
}

.stat-icon.safe {
  background-color: #10b981;
}

.stat-icon.warning {
  background-color: #f59e0b;
}

.stat-icon.danger {
  background-color: #ef4444;
}

.stat-icon.total {
  background-color: #3b82f6;
}

.stat-info {
  flex: 1;
}

.stat-number {
  font-size: 32px;
  font-weight: bold;
  color: #1f2937;
  line-height: 1;
}

.stat-label {
  font-size: 16px;
  color: #6b7280;
  margin-top: 4px;
}

/* 图表卡片 */
.chart-card {
  height: 100%;
  transition: all 0.3s ease;
}

.chart-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header span {
  font-weight: bold;
  font-size: 16px;
}

.chart-container {
  width: 100%;
  height: 300px;
}

/* 风险分布 */
.risk-distribution {
  display: flex;
  gap: 20px;
  height: 300px;
}

.pie-chart {
  flex: 1;
  height: 100%;
}

.risk-details {
  flex: 1;
  padding: 10px;
  display: flex;
  flex-direction: column;
  justify-content: space-around;
}

.risk-item {
  margin-bottom: 20px;
}

.risk-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.risk-name {
  font-weight: bold;
  color: #374151;
}

.risk-value {
  font-weight: bold;
  color: #6b7280;
}

.risk-progress {
  margin-top: 8px;
}

/* 预警信息 */
.alert-card {
  height: 100%;
  transition: all 0.3s ease;
}

.alert-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.alert-list {
  padding: 10px 0;
}

.alert-item {
  padding: 16px;
  margin-bottom: 12px;
  border-radius: 8px;
  background-color: #f9fafb;
  border-left: 4px solid #9ca3af;
}

.alert-item.alert-high {
  border-left-color: #ef4444;
  background-color: #fef2f2;
}

.alert-item.alert-medium {
  border-left-color: #f59e0b;
  background-color: #fffbeb;
}

.alert-item.alert-low {
  border-left-color: #3b82f6;
  background-color: #eff6ff;
}

.alert-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.alert-time {
  font-size: 12px;
  color: #6b7280;
}

.alert-vehicle {
  font-weight: bold;
  color: #374151;
  margin-bottom: 4px;
}

.alert-desc {
  font-size: 14px;
  color: #6b7280;
  line-height: 1.4;
}

.no-alerts {
  padding: 40px 0;
  text-align: center;
}

/* 安全指标 */
.metrics-card {
  height: 100%;
  transition: all 0.3s ease;
}

.metrics-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  padding: 10px 0;
}

.metric-item {
  text-align: center;
  padding: 16px;
  background-color: #f9fafb;
  border-radius: 8px;
}

.metric-label {
  font-size: 14px;
  color: #6b7280;
  margin-bottom: 8px;
}

.metric-value {
  font-size: 24px;
  font-weight: bold;
  color: #1f2937;
  margin-bottom: 4px;
}

.metric-trend {
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.metric-trend.trend-up {
  color: #10b981;
}

.metric-trend.trend-down {
  color: #ef4444;
}
</style>


