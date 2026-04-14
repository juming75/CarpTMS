<template>
  <div class="data-visualization">
    <el-row :gutter="20">
      <!-- 统计卡片 -->
      <el-col :span="24">
        <el-row :gutter="20" class="stats-row">
          <el-col :span="6" v-for="stat in statsCards" :key="stat.title">
            <stat-card
              :label="stat.title"
              :value="stat.value"
              :icon="stat.icon"
              :color="stat.color"
              :trend="stat.trend"
            />
          </el-col>
        </el-row>
      </el-col>

      <!-- 图表区域 -->
      <el-col :span="16" class="chart-section">
        <chart-card title="实时车辆状态分布">
          <div ref="statusChartRef" class="chart-container"></div>
        </chart-card>
      </el-col>

      <el-col :span="8" class="chart-section">
        <chart-card title="车辆分组统计">
          <div ref="groupChartRef" class="chart-container"></div>
        </chart-card>
      </el-col>

      <el-col :span="24" class="chart-section">
        <chart-card title="24小时车辆活跃趋势">
          <div ref="trendChartRef" class="chart-container"></div>
        </chart-card>
      </el-col>

      <el-col :span="12" class="chart-section">
        <chart-card title="车辆类型分布">
          <div ref="typeChartRef" class="chart-container"></div>
        </chart-card>
      </el-col>

      <el-col :span="12" class="chart-section">
        <chart-card title="报警类型统计">
          <div ref="alarmChartRef" class="chart-container"></div>
        </chart-card>
      </el-col>

      <el-col :span="24" class="chart-section">
        <chart-card title="传感器数据趋势">
          <el-tabs v-model="activeSensorTab" class="sensor-tabs">
            <el-tab-pane label="油量" name="fuel">
              <div ref="fuelChartRef" class="chart-container"></div>
            </el-tab-pane>
            <el-tab-pane label="水温" name="waterTemp">
              <div ref="waterTempChartRef" class="chart-container"></div>
            </el-tab-pane>
            <el-tab-pane label="发动机转速" name="engineRpm">
              <div ref="engineRpmChartRef" class="chart-container"></div>
            </el-tab-pane>
            <el-tab-pane label="载重" name="loadWeight">
              <div ref="loadWeightChartRef" class="chart-container"></div>
            </el-tab-pane>
          </el-tabs>
        </chart-card>
      </el-col>

      <!-- 实时数据表格 -->
      <el-col :span="24" class="data-table-section">
        <el-card class="data-card">
          <template #header>
            <div class="card-header">
              <span>实时数据监控</span>
              <el-button-group>
                <el-button :icon="Refresh" size="small" @click="refreshData">刷新</el-button>
                <el-button :icon="Download" size="small" @click="exportData">导出</el-button>
              </el-button-group>
            </div>
          </template>

          <el-table :data="realtimeData" stripe max-height="400">
            <el-table-column prop="licensePlate" label="车牌号" width="120" fixed />
            <el-table-column prop="vehicleType" label="类型" width="100" />
            <el-table-column label="状态" width="80">
              <template #default="{ row }">
                <el-tag :type="getStatusType(row.status)">
                  {{ getStatusText(row.status) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="speed" label="速度 (km/h)" width="100" />
            <el-table-column prop="fuel" label="油量 (L)" width="100" />
            <el-table-column prop="waterTemp" label="水温 (°C)" width="100" />
            <el-table-column prop="loadWeight" label="载重 (kg)" width="100" />
            <el-table-column prop="mileage" label="里程 (km)" width="100" />
            <el-table-column prop="gpsTime" label="GPS时间" width="160">
              <template #default="{ row }">
                {{ formatTime(row.gpsTime) }}
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, nextTick } from 'vue';
import { Refresh, Download } from '@element-plus/icons-vue';
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

import StatCard from './stat-card/StatCard.vue';
import ChartCard from './chart-card/ChartCard.vue';
import { getStatistics } from '@/services/statistics';

// 类型定义
interface VehicleData {
  licensePlate: string;
  vehicleType: string;
  status: number;
  speed: number;
  fuel: number;
  waterTemp: number;
  loadWeight: number;
  mileage: number;
  gpsTime: string;
  groupName?: string;
}

interface HourlyTrend {
  hour: string;
  online: number;
  alarm: number;
}

interface AlarmStat {
  type: string;
  count: number;
}

interface SensorData {
  time: string;
  value: number;
}

interface SensorTrend {
  fuel?: SensorData[];
  waterTemp?: SensorData[];
  engineRpm?: SensorData[];
  loadWeight?: SensorData[];
}

interface StatisticsData {
  totalVehicles: number;
  onlineVehicles: number;
  todayMileage: number;
  todayAlarm: number;
  hourlyTrend: HourlyTrend[];
  alarmStats: AlarmStat[];
  sensorTrend: SensorTrend;
}

// Refs
const statusChartRef = ref<HTMLElement>();
const groupChartRef = ref<HTMLElement>();
const trendChartRef = ref<HTMLElement>();
const typeChartRef = ref<HTMLElement>();
const alarmChartRef = ref<HTMLElement>();
const fuelChartRef = ref<HTMLElement>();
const waterTempChartRef = ref<HTMLElement>();
const engineRpmChartRef = ref<HTMLElement>();
const loadWeightChartRef = ref<HTMLElement>();

const activeSensorTab = ref('fuel');
const realtimeData = ref<VehicleData[]>([]);
const charts = ref<echarts.ECharts[]>([]);
const refreshTimer = ref<number | null>(null);

// 统计卡片数据
const statsCards = reactive([
  { title: '总车辆数', value: 0, icon: 'Van', color: '#409eff', trend: 0 },
  { title: '在线车辆', value: 0, icon: 'VideoPlay', color: '#67C23A', trend: 0 },
  { title: '今日行驶', value: 0, icon: 'Position', color: '#E6A23C', trend: 0 },
  { title: '今日报警', value: 0, icon: 'Warning', color: '#F56C6C', trend: 0 },
]);

// 加载统计数据
const loadStatistics = async () => {
  try {
    const statsData = await getStatistics();

    // 更新统计卡片
    statsCards[0].value = statsData.data?.totalVehicles || 0;
    statsCards[1].value = statsData.data?.onlineVehicles || 0;
    statsCards[2].value = statsData.data?.todayMileage || 0;
    statsCards[3].value = statsData.data?.todayAlarm || 0;

    // 等待 DOM 更新后初始化图表
    await nextTick();
    initCharts([], statsData.data);
  } catch (error) {
    console.error('Failed to load statistics:', error);
  }
};

// 初始化图表
const initCharts = (vehicles: VehicleData[], stats: StatisticsData | undefined) => {
  // 状态分布饼图
  initStatusChart(vehicles);

  // 分组统计柱状图
  initGroupChart(vehicles);

  // 趋势折线图
  initTrendChart(stats?.hourlyTrend || []);

  // 类型分布图
  initTypeChart(vehicles);

  // 报警统计图
  initAlarmChart(stats?.alarmStats || []);

  // 传感器趋势图
  initSensorCharts(stats?.sensorTrend || {});
};

// 状态分布饼图
const initStatusChart = (vehicles: VehicleData[]) => {
  if (!statusChartRef.value) return;

  const chart = echarts.init(statusChartRef.value);
  charts.value.push(chart);

  const statusCount = {
    online: vehicles.filter((v) => v.status === 1).length,
    offline: vehicles.filter((v) => v.status === 2).length,
    alarm: vehicles.filter((v) => v.status === 3).length,
  };

  const option = {
    tooltip: {
      trigger: 'item',
      formatter: '{a} <br/>{b}: {c} ({d}%)',
    },
    legend: {
      orient: 'vertical',
      right: 10,
      top: 'center',
    },
    series: [
      {
        name: '车辆状态',
        type: 'pie',
        radius: ['40%', '70%'],
        avoidLabelOverlap: false,
        itemStyle: {
          borderRadius: 10,
          borderColor: '#fff',
          borderWidth: 2,
        },
        label: {
          show: false,
          position: 'center',
        },
        emphasis: {
          label: {
            show: true,
            fontSize: 20,
            fontWeight: 'bold',
          },
        },
        data: [
          { value: statusCount.online, name: '在线', itemStyle: { color: '#67C23A' } },
          { value: statusCount.offline, name: '离线', itemStyle: { color: '#909399' } },
          { value: statusCount.alarm, name: '报警', itemStyle: { color: '#F56C6C' } },
        ],
      },
    ],
  };

  chart.setOption(option);
};

// 分组统计柱状图
const initGroupChart = (vehicles: VehicleData[]) => {
  if (!groupChartRef.value) return;

  const chart = echarts.init(groupChartRef.value);
  charts.value.push(chart);

  // 按分组统计
  const groupMap = new Map<string, number>();
  vehicles.forEach((v) => {
    if (v.groupName) {
      const count = groupMap.get(v.groupName) || 0;
      groupMap.set(v.groupName, count + 1);
    }
  });

  const groupNames = Array.from(groupMap.keys());
  const groupCounts = Array.from(groupMap.values());

  const option = {
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'shadow',
      },
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true,
    },
    xAxis: {
      type: 'category',
      data: groupNames,
      axisLabel: {
        interval: 0,
        rotate: 30,
      },
    },
    yAxis: {
      type: 'value',
    },
    series: [
      {
        name: '车辆数',
        type: 'bar',
        data: groupCounts,
        itemStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: '#83bff6' },
            { offset: 0.5, color: '#188df0' },
            { offset: 1, color: '#188df0' },
          ]),
        },
        barWidth: '40%',
      },
    ],
  };

  chart.setOption(option);
};

// 趋势折线图
const initTrendChart = (hourlyTrend: HourlyTrend[]) => {
  if (!trendChartRef.value) return;

  const chart = echarts.init(trendChartRef.value);
  charts.value.push(chart);

  const hours = hourlyTrend.map((t) => t.hour);
  const onlineCounts = hourlyTrend.map((t) => t.online);
  const alarmCounts = hourlyTrend.map((t) => t.alarm);

  const option = {
    tooltip: {
      trigger: 'axis',
    },
    legend: {
      data: ['在线数', '报警数'],
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true,
    },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: hours,
    },
    yAxis: {
      type: 'value',
    },
    series: [
      {
        name: '在线数',
        type: 'line',
        smooth: true,
        data: onlineCounts,
        itemStyle: { color: '#67C23A' },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(103, 194, 58, 0.3)' },
            { offset: 1, color: 'rgba(103, 194, 58, 0.1)' },
          ]),
        },
      },
      {
        name: '报警数',
        type: 'line',
        smooth: true,
        data: alarmCounts,
        itemStyle: { color: '#F56C6C' },
      },
    ],
  };

  chart.setOption(option);
};

// 类型分布图
const initTypeChart = (vehicles: VehicleData[]) => {
  if (!typeChartRef.value) return;

  const chart = echarts.init(typeChartRef.value);
  charts.value.push(chart);

  const typeMap = new Map<string, number>();
  vehicles.forEach((v) => {
    const count = typeMap.get(v.vehicleType) || 0;
    typeMap.set(v.vehicleType, count + 1);
  });

  const types = Array.from(typeMap.keys());
  const counts = Array.from(typeMap.values());

  const option = {
    tooltip: {
      trigger: 'item',
    },
    series: [
      {
        type: 'pie',
        radius: '50%',
        data: types.map((type, index) => ({
          value: counts[index],
          name: type,
        })),
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)',
          },
        },
      },
    ],
  };

  chart.setOption(option);
};

// 报警统计图
const initAlarmChart = (alarmStats: AlarmStat[]) => {
  if (!alarmChartRef.value) return;

  const chart = echarts.init(alarmChartRef.value);
  charts.value.push(chart);

  const alarmTypes = alarmStats.map((a) => a.type);
  const alarmCounts = alarmStats.map((a) => a.count);

  const option = {
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'shadow',
      },
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true,
    },
    xAxis: {
      type: 'category',
      data: alarmTypes,
    },
    yAxis: {
      type: 'value',
    },
    series: [
      {
        name: '报警数',
        type: 'bar',
        data: alarmCounts,
        itemStyle: {
          color: '#F56C6C',
        },
      },
    ],
  };

  chart.setOption(option);
};

// 传感器趋势图
const initSensorCharts = (sensorTrend: SensorTrend) => {
  const sensorCharts = [
    { ref: fuelChartRef, key: 'fuel', label: '油量 (L)', unit: 'L' },
    { ref: waterTempChartRef, key: 'waterTemp', label: '水温 (°C)', unit: '°C' },
    { ref: engineRpmChartRef, key: 'engineRpm', label: '转速 (RPM)', unit: 'RPM' },
    { ref: loadWeightChartRef, key: 'loadWeight', label: '载重 (kg)', unit: 'kg' },
  ];

  sensorCharts.forEach(({ ref, key, label, unit }) => {
    if (!ref.value) return;

    const chart = echarts.init(ref.value);
    charts.value.push(chart);

    const data = (sensorTrend as Record<string, SensorData[]>)[key] || [];
    const times = data.map((d: SensorData) => d.time);
    const values = data.map((d: SensorData) => d.value);

    const option = {
      tooltip: {
        trigger: 'axis',
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: times,
      },
      yAxis: {
        type: 'value',
        name: unit,
      },
      series: [
        {
          name: label,
          type: 'line',
          smooth: true,
          data: values,
          itemStyle: { color: '#409eff' },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: 'rgba(64, 158, 255, 0.3)' },
              { offset: 1, color: 'rgba(64, 158, 255, 0.1)' },
            ]),
          },
        },
      ],
    };

    chart.setOption(option);
  });
};

// 刷新数据
const refreshData = () => {
  loadStatistics();
};

// 导出数据
const exportData = () => {
  const data = JSON.stringify(realtimeData.value, null, 2);
  const blob = new window.Blob([data], { type: 'application/json' });
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `实时数据_${new Date().toLocaleString('zh-CN')}.json`;
  a.click();
  window.URL.revokeObjectURL(url);
};

// 工具函数
const getStatusType = (status: number) => {
  switch (status) {
    case 1:
      return 'success';
    case 2:
      return 'info';
    case 3:
      return 'danger';
    default:
      return '';
  }
};

const getStatusText = (status: number) => {
  const statusMap: Record<number, string> = {
    1: '在线',
    2: '离线',
    3: '报警',
    4: '未知',
  };
  return statusMap[status] || '未知';
};

const formatTime = (time: string) => {
  if (!time) return '-';
  return new Date(time).toLocaleString('zh-CN');
};

// 响应式调整图表大小
const handleResize = () => {
  charts.value.forEach((chart) => chart?.resize());
};

// 生命周期
onMounted(() => {
  loadStatistics();
  window.addEventListener('resize', handleResize);
});

onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
  if (refreshTimer.value) {
    window.clearInterval(refreshTimer.value);
  }
  charts.value.forEach((chart) => chart?.dispose());
});
</script>

<style scoped>
.data-visualization {
  padding: 20px;
  background: #f5f5f5;
}

.stats-row {
  margin-bottom: 20px;
}

.chart-section {
  margin-bottom: 20px;
}

.chart-container {
  width: 100%;
  height: 350px;
}

.data-table-section {
  margin-top: 20px;
}

.data-card {
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sensor-tabs {
  margin-top: 10px;
}
</style>


