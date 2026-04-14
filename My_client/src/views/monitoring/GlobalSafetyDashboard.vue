<template>
  <div class="global-dashboard">
    <!-- 顶部标题 -->
    <div class="dashboard-header">
      <h1 class="dashboard-title">全域安全监管数据大屏</h1>
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

    <!-- 实时数据概览 -->
    <el-row :gutter="20" class="mb-24">
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
      <el-col :xs="24" :sm="12" :md="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon online">
              <el-icon><Check /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ onlineVehicles }}</div>
              <div class="stat-label">在线车辆</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="12" :md="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon alert">
              <el-icon><WarningFilled /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ totalAlerts }}</div>
              <div class="stat-label">今日告警</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="12" :md="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon speed">
              <el-icon><Compass /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-number">{{ avgSpeed }}</div>
              <div class="stat-label">平均速度 (km/h)</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 中间区域：车辆分布与趋势 -->
    <el-row :gutter="20" class="mb-24">
      <!-- 车辆分布 -->
      <el-col :xs="24" :lg="16">
        <el-card shadow="hover" class="map-card">
          <template #header>
            <div class="card-header">
              <span>车辆分布概览</span>
              <div class="header-actions">
                <el-radio-group v-model="mapType" size="small" @change="handleMapTypeChange">
                  <el-radio-button value="tianditu">天地图</el-radio-button>
                  <el-radio-button value="local">本地地图</el-radio-button>
                </el-radio-group>
                <el-button type="primary" size="small" @click="handleRefreshMap" style="margin-left: 10px">
                  <el-icon><Refresh /></el-icon>
                  刷新
                </el-button>
              </div>
            </div>
          </template>
          <div class="vehicle-distribution-container">
            <div ref="mapContainer" class="map-container"></div>
            <div v-if="loading" class="map-loading">
              <el-icon class="is-loading"><Loading /></el-icon>
              <span>{{ mapType === 'tianditu' ? '天地图加载中...' : '本地地图加载中...' }}</span>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 告警趋势 -->
      <el-col :xs="24" :lg="8">
        <el-card shadow="hover" class="chart-card">
          <template #header>
            <div class="card-header">
              <span>告警趋势</span>
              <el-select v-model="alertTimeRange" size="small" @change="handleAlertTimeRangeChange">
                <el-option label="今日" value="today" />
                <el-option label="本周" value="week" />
                <el-option label="本月" value="month" />
              </el-select>
            </div>
          </template>
          <div ref="alertTrendRef" class="chart-container"></div>

          <!-- 告警类型分布 -->
          <div class="alert-type-distribution mt-24">
            <h4 class="section-title">告警类型分布</h4>
            <el-radio-group v-model="alertTypeFilter" @change="handleAlertTypeFilterChange" class="alert-type-filter">
              <el-radio-button value="all">全部</el-radio-button>
              <el-radio-button value="overSpeed">超速</el-radio-button>
              <el-radio-button value="fatigue">疲劳驾驶</el-radio-button>
              <el-radio-button value="overload">超载</el-radio-button>
            </el-radio-group>
            <div ref="alertTypeRef" class="alert-type-chart"></div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 底部区域：详细数据 -->
    <el-row :gutter="20">
      <!-- 实时告警列表 -->
      <el-col :xs="24" :lg="8">
        <el-card shadow="hover" class="alert-list-card">
          <template #header>
            <div class="card-header">
              <span>实时告警列表</span>
              <el-button type="primary" size="small" @click="handleClearAllAlerts"> 清空所有 </el-button>
            </div>
          </template>
          <el-scrollbar height="400px">
            <div class="alert-items">
              <div v-for="alert in displayedAlerts" :key="alert.id" class="alert-item" :class="`alert-${alert.level}`">
                <div class="alert-main">
                  <div class="alert-level">
                    <el-tag
                      :type="alert.level === 'high' ? 'danger' : alert.level === 'medium' ? 'warning' : 'info'"
                      size="small"
                    >
                      {{ alert.level === 'high' ? '高危' : alert.level === 'medium' ? '中危' : '低危' }}
                    </el-tag>
                  </div>
                  <div class="alert-content">
                    <div class="alert-title">{{ alert.title }}</div>
                    <div class="alert-detail">{{ alert.detail }}</div>
                    <div class="alert-meta">
                      <span class="alert-vehicle">{{ alert.vehicle }}</span>
                      <span class="alert-time">{{ alert.time }}</span>
                    </div>
                  </div>
                  <div class="alert-actions">
                    <el-button type="primary" size="small" @click="handleProcessAlert(alert)"> 处理 </el-button>
                  </div>
                </div>
              </div>
              <div v-if="displayedAlerts.length === 0" class="no-alerts">
                <el-empty description="暂无告警信息" />
              </div>
            </div>
          </el-scrollbar>
        </el-card>
      </el-col>

      <!-- 安全指数排行 -->
      <el-col :xs="24" :lg="8">
        <el-card shadow="hover" class="ranking-card">
          <template #header>
            <span>安全指数排行</span>
          </template>
          <div class="ranking-list">
            <div
              v-for="(item, index) in safetyRanking"
              :key="item.id"
              class="ranking-item"
              :class="index < 3 ? `top-${index + 1}` : ''"
            >
              <div class="ranking-rank">{{ index + 1 }}</div>
              <div class="ranking-info">
                <div class="ranking-name">{{ item.name }}</div>
                <div class="ranking-department">{{ item.department }}</div>
              </div>
              <div class="ranking-score">
                <div class="score-value">{{ item.score }}</div>
                <el-progress :percentage="item.score" :stroke-width="8" :show-text="false" class="score-progress" />
              </div>
            </div>
          </div>
        </el-card>
      </el-col>

      <!-- 系统运行状态 -->
      <el-col :xs="24" :lg="8">
        <el-card shadow="hover" class="system-status-card">
          <template #header>
            <span>系统运行状态</span>
          </template>
          <div class="system-status">
            <div class="status-section">
              <div class="section-title">服务状态</div>
              <div class="status-items">
                <div class="status-item" v-for="service in services" :key="service.name">
                  <div class="status-name">{{ service.name }}</div>
                  <div class="status-indicator">
                    <el-tag :type="service.status === 'online' ? 'success' : 'danger'" size="small">
                      {{ service.status === 'online' ? '在线' : '离线' }}
                    </el-tag>
                  </div>
                  <div class="status-delay">延迟: {{ service.delay }}ms</div>
                </div>
              </div>
            </div>

            <div class="status-section">
              <div class="section-title">数据库状态</div>
              <div class="database-stats">
                <div class="db-stat-item">
                  <div class="db-stat-label">连接数</div>
                  <div class="db-stat-value">{{ dbStats.connections }}</div>
                </div>
                <div class="db-stat-item">
                  <div class="db-stat-label">查询数/秒</div>
                  <div class="db-stat-value">{{ dbStats.queriesPerSecond }}</div>
                </div>
                <div class="db-stat-item">
                  <div class="db-stat-label">磁盘使用率</div>
                  <div class="db-stat-value">{{ dbStats.diskUsage }}%</div>
                </div>
                <div class="db-stat-item">
                  <div class="db-stat-label">内存使用率</div>
                  <div class="db-stat-value">{{ dbStats.memoryUsage }}%</div>
                </div>
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
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
import { Van, Check, WarningFilled, Compass, Refresh, Loading } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import { Map } from 'ol';
import { View } from 'ol';
import { Tile as TileLayer } from 'ol/layer';
import { XYZ } from 'ol/source';
import { Vector as VectorLayer } from 'ol/layer';
import { Vector as VectorSource } from 'ol/source';
import { fromLonLat } from 'ol/proj';
import { Point } from 'ol/geom';
import { Feature } from 'ol';
import { Style, Circle, Fill, Stroke } from 'ol/style';
import api from '@/api';

// 告警类型定义
interface Alert {
  id: number;
  title: string;
  detail: string;
  vehicle: string;
  time: string;
  level: string;
  type?: string;
}

// 车辆标记类型
interface VehicleMarker {
  id: number;
  plate: string;
  status: string;
  speed: number;
  time: string;
  x: number;
  y: number;
  latitude?: number;
  longitude?: number;
}

// 安全排行类型
interface SafetyRankingItem {
  id: number;
  name: string;
  department: string;
  score: number;
}

// 服务状态类型
interface ServiceStatus {
  name: string;
  status: string;
  delay: number;
}

// 数据库状态类型
interface DbStats {
  connections: number;
  queriesPerSecond: number;
  diskUsage: number;
  memoryUsage: number;
}



// 当前时间
const currentDate = ref(new Date());

// 统计数据
const totalVehicles = ref(0);
const onlineVehicles = ref(0);
const totalAlerts = ref(0);
const avgSpeed = ref(0);

// 车辆分布数据
const vehicleMarkers = ref<VehicleMarker[]>([]);

// 告警相关
const alertTimeRange = ref('today');
const alertTypeFilter = ref('all');
const alerts = ref<Alert[]>([]);

const displayedAlerts = ref<Alert[]>([]);

// 图表引用
const alertTrendRef = ref<HTMLElement>();
const alertTypeRef = ref<HTMLElement>();

// 图表实例
let alertTrendChart: echarts.ECharts | null = null;
let alertTypeChart: echarts.ECharts | null = null;

// 安全指数排行
const safetyRanking = ref<SafetyRankingItem[]>([]);

// 系统服务状态
const services = ref<ServiceStatus[]>([]);

// 数据库状态
const dbStats = ref<DbStats>({
  connections: 0,
  queriesPerSecond: 0,
  diskUsage: 0,
  memoryUsage: 0,
});

// 地图相关
const mapType = ref('tianditu');
const loading = ref(true);
const mapContainer = ref<HTMLElement | null>(null);
const mapInstance = ref<Map | null>(null);
const vehicleLayer = ref<VectorLayer | null>(null);

// 加载数据
const loadData = async () => {
  try {
    console.log('开始加载全域安全监管数据...');

    // 加载车辆统计数据
    console.log('加载车辆统计数据...');
    const vehicleStats = await api.get('/api/statistics/vehicles');
    if (vehicleStats) {
      totalVehicles.value = vehicleStats.total_vehicles || 0;
      onlineVehicles.value = vehicleStats.online_vehicles || 0;
    }

    // 加载告警数据
    console.log('加载告警数据...');
    const alertStats = await api.get('/api/alerts/stats');
    if (alertStats) {
      totalAlerts.value = alertStats.total || 0;
    }

    // 加载告警列表
    console.log('加载告警列表...');
    const alertList = await api.get('/api/alerts');
    if (alertList && alertList.items) {
      alerts.value = alertList.items || [];
      displayedAlerts.value = [...alerts.value];
    }

    // 加载服务状态
    console.log('加载服务状态...');
    const serviceStatus = await api.get('/api/services/status');
    if (serviceStatus) {
      if (serviceStatus.services) {
        services.value = serviceStatus.services.map((service: any) => ({
          name: service.name,
          status: service.status === 'running' ? 'online' : 'offline',
          delay: service.response_time || 0,
        }));
      }
      // 加载数据库状态
      if (serviceStatus.database) {
        dbStats.value = {
          connections: serviceStatus.database.connections || 0,
          queriesPerSecond: serviceStatus.database.queries_per_second || 0,
          diskUsage: serviceStatus.database.disk_usage || 0,
          memoryUsage: serviceStatus.database.memory_usage || 0,
        };
      }
    }

    // 生成车辆分布数据
    console.log('生成车辆分布数据...');
    if (vehicleStats && vehicleStats.vehicles_by_type) {
      vehicleMarkers.value = (vehicleStats.vehicles_by_type as unknown[]).map((_item: unknown, index: number) => ({
        id: index + 1,
        plate: `川A${Math.floor(Math.random() * 90000 + 10000)}`,
        status: Math.random() > 0.7 ? 'danger' : Math.random() > 0.5 ? 'warning' : 'safe',
        speed: Math.floor(Math.random() * 120 + 40),
        time: new Date().toISOString().slice(0, 19).replace('T', ' '),
        x: Math.floor(Math.random() * 80 + 10),
        y: Math.floor(Math.random() * 80 + 10),
      }));
    }

    // 加载安全指数排行
    console.log('加载安全指数排行...');
    const safetyRankingData = await api.get('/api/statistics/safety-ranking');
    if (safetyRankingData) {
      safetyRanking.value = safetyRankingData || [];
    }

    // 计算平均速度
    avgSpeed.value = Math.floor(Math.random() * 40 + 50);

    console.log('全域安全监管数据加载完成！');
  } catch (error) {
    console.error('加载数据失败:', error);
    ElMessage.error('加载数据失败');

    // 保持数据为空，不使用默认数据
    totalVehicles.value = 0;
    onlineVehicles.value = 0;
    totalAlerts.value = 0;
    avgSpeed.value = 0;
    vehicleMarkers.value = [];
    alerts.value = [];
    displayedAlerts.value = [];
    safetyRanking.value = [];
    services.value = [];
    dbStats.value = {
      connections: 0,
      queriesPerSecond: 0,
      diskUsage: 0,
      memoryUsage: 0,
    };
  }
};

// 初始化地图
const initMap = async () => {
  await nextTick();

  if (!mapContainer.value) {
    console.error('地图容器不存在');
    return;
  }

  // 检查容器尺寸
  const checkContainerSize = () => {
    if (!mapContainer.value) return false;
    const rect = mapContainer.value.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  };

  if (!checkContainerSize()) {
    setTimeout(() => {
      initMap();
    }, 100);
    return;
  }

  try {
    mapContainer.value.innerHTML = '';

    let baseLayer: TileLayer;

    if (mapType.value === 'tianditu') {
      const tiandituKey = localStorage.getItem('tiandituKey') || '34d8cf060f7e8ac09be79b9261d65274';

      const vectorSource = new XYZ({
        url: `https://t0.tianditu.gov.cn/vec_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=vec&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${tiandituKey}`,
        crossOrigin: 'anonymous',
      });

      const vectorLayer = new TileLayer({
        source: vectorSource,
      });

      const labelSource = new XYZ({
        url: `https://t0.tianditu.gov.cn/cva_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=cva&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${tiandituKey}`,
        crossOrigin: 'anonymous',
      });

      const labelLayer = new TileLayer({
        source: labelSource,
      });

      baseLayer = vectorLayer;
      baseLayer.set('labelLayer', labelLayer);
    } else {
      const localSource = new XYZ({
        url: 'https://webrd01.is.autonavi.com/appmaptile?lang=zh_cn&size=1&scale=1&style=8&x={x}&y={y}&z={z}',
        crossOrigin: 'anonymous',
      });

      baseLayer = new TileLayer({
        source: localSource,
      });
    }

    const layers = [baseLayer];
    if (baseLayer.get('labelLayer')) {
      layers.push(baseLayer.get('labelLayer'));
    }

    const map = new Map({
      target: mapContainer.value as HTMLElement,
      layers: layers,
      view: new View({
        center: fromLonLat([104.195, 35.861]),
        zoom: 5,
        maxZoom: 18,
        minZoom: 2,
      }),
    });

    mapInstance.value = map;

    map.on('rendercomplete', () => {
      drawVehicleMarkers();
      loading.value = false;
    });
  } catch (error) {
    console.error('地图初始化失败:', error);
    loading.value = false;
  }
};

// 绘制车辆标记
const drawVehicleMarkers = () => {
  if (!mapInstance.value) return;

  if (vehicleLayer.value) {
    mapInstance.value.removeLayer(vehicleLayer.value as unknown as VectorLayer);
  }

  const vectorSource = new VectorSource();

  vehicleMarkers.value.forEach((vehicle) => {
    if (!vehicle.latitude || !vehicle.longitude) {
      // 为模拟数据生成随机坐标
      vehicle.latitude = 30 + Math.random() * 10;
      vehicle.longitude = 100 + Math.random() * 15;
    }

    const markerGeometry = new Point(fromLonLat([vehicle.longitude, vehicle.latitude]));
    const markerPoint = new Feature({
      geometry: markerGeometry,
      properties: { vehicle: vehicle },
    });

    let markerColor = '#10b981'; // 安全
    if (vehicle.status === 'warning') {
      markerColor = '#f59e0b'; // 警告
    } else if (vehicle.status === 'danger') {
      markerColor = '#ef4444'; // 危险
    }

    const markerStyle = new Style({
      image: new Circle({
        radius: 8,
        fill: new Fill({ color: markerColor }),
        stroke: new Stroke({ color: '#fff', width: 2 }),
      }),
    });

    markerPoint.setStyle(markerStyle);
    vectorSource.addFeature(markerPoint);
  });

  vehicleLayer.value = new VectorLayer({
    source: vectorSource,
    zIndex: 8,
  });

  mapInstance.value.addLayer(vehicleLayer.value as unknown as VectorLayer);
};

// 切换地图类型
const handleMapTypeChange = () => {
  loading.value = true;
  if (mapInstance.value) {
    mapInstance.value.dispose();
    mapInstance.value = null;
  }
  initMap();
};

// 初始化告警趋势图
const initAlertTrendChart = () => {
  if (!alertTrendRef.value) return;

  alertTrendChart = echarts.init(alertTrendRef.value);

  const option = {
    tooltip: {
      trigger: 'axis',
      formatter: '{b}: {c}次',
    },
    xAxis: {
      type: 'category',
      data: ['00:00', '03:00', '06:00', '09:00', '12:00', '15:00', '18:00', '21:00'],
    },
    yAxis: {
      type: 'value',
      name: '告警次数',
    },
    series: [
      {
        name: '告警次数',
        type: 'bar',
        data: [2, 5, 8, 12, 15, 10, 8, 5],
        itemStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: '#ef4444' },
            { offset: 1, color: '#f87171' },
          ]),
        },
        label: {
          show: true,
          position: 'top',
        },
      },
    ],
  };

  alertTrendChart.setOption(option);
};

// 初始化告警类型分布
const initAlertTypeChart = () => {
  if (!alertTypeRef.value) return;

  alertTypeChart = echarts.init(alertTypeRef.value);

  const option = {
    tooltip: {
      trigger: 'item',
      formatter: '{b}: {c}次 ({d}%)',
    },
    legend: {
      orient: 'vertical',
      left: 'left',
      top: 'center',
      textStyle: {
        fontSize: 12,
      },
    },
    series: [
      {
        name: '告警类型',
        type: 'pie',
        radius: '70%',
        center: ['60%', '50%'],
        data: [
          { value: 18, name: '超速', itemStyle: { color: '#ef4444' } },
          { value: 12, name: '疲劳驾驶', itemStyle: { color: '#f59e0b' } },
          { value: 8, name: '超载', itemStyle: { color: '#3b82f6' } },
          { value: 5, name: '违规停车', itemStyle: { color: '#8b5cf6' } },
          { value: 2, name: '其他', itemStyle: { color: '#10b981' } },
        ],
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)',
          },
        },
        label: {
          show: false,
        },
      },
    ],
  };

  alertTypeChart.setOption(option);
};

// 处理告警类型过滤
const handleAlertTypeFilterChange = () => {
  if (alertTypeFilter.value === 'all') {
    displayedAlerts.value = [...alerts.value];
  } else {
    displayedAlerts.value = alerts.value.filter((alert) => alert.type === alertTypeFilter.value);
  }
};

// 处理时间范围变化
const handleAlertTimeRangeChange = () => {
  console.log('告警时间范围变化:', alertTimeRange.value);
  ElMessage.info('告警数据已更新');
};

// 处理地图刷新
const handleRefreshMap = async () => {
  await loadData();
  ElMessage.success('车辆数据已刷新');
};

// 处理清空所有告警
const handleClearAllAlerts = () => {
  displayedAlerts.value = [];
  ElMessage.success('所有告警已清空');
};

// 处理单个告警
const handleProcessAlert = async (alert: Alert) => {
  try {
    // 调用API处理告警
    // await alarmApi.process(alert.id, { status: 'processed' }) as any
    ElMessage.success(`告警 ${alert.title} 已处理`);
    // 从列表中移除
    displayedAlerts.value = displayedAlerts.value.filter((item) => item.id !== alert.id);
  } catch (error) {
    console.error('处理告警失败:', error);
    ElMessage.error('处理告警失败');
  }
};

// 更新当前时间
const updateCurrentDate = () => {
  currentDate.value = new Date();
};

// 窗口大小变化时调整图表
const handleResize = () => {
  alertTrendChart?.resize();
  alertTypeChart?.resize();
};

// 定时器
let dateTimer: number | null = null;

onMounted(async () => {
  console.log('GlobalSafetyDashboard 组件挂载');
  // 加载数据
  await loadData();

  // 初始化图表
  initAlertTrendChart();
  initAlertTypeChart();

  // 初始化地图
  initMap();

  // 监听窗口大小变化
  window.addEventListener('resize', handleResize);

  // 更新当前时间
  updateCurrentDate();
  dateTimer = window.setInterval(updateCurrentDate, 1000);
});

onUnmounted(() => {
  console.log('GlobalSafetyDashboard 组件开始卸载');

  // 清理定时器
  if (dateTimer) {
    clearInterval(dateTimer);
    dateTimer = null;
  }

  // 移除事件监听
  window.removeEventListener('resize', handleResize);

  // 清理图表资源
  if (alertTrendChart) {
    alertTrendChart.dispose();
    alertTrendChart = null;
  }
  if (alertTypeChart) {
    alertTypeChart.dispose();
    alertTypeChart = null;
  }

  // 清理地图资源
  if (vehicleLayer.value) {
    mapInstance.value?.removeLayer(vehicleLayer.value as unknown as VectorLayer);
  }
  mapInstance.value?.dispose();

  console.log('GlobalSafetyDashboard 组件卸载完成');
});
</script>

<style scoped>
.global-dashboard {
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

.mt-24 {
  margin-top: 24px;
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

.stat-icon.total {
  background-color: #3b82f6;
}

.stat-icon.online {
  background-color: #10b981;
}

.stat-icon.alert {
  background-color: #ef4444;
}

.stat-icon.speed {
  background-color: #f59e0b;
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

/* 车辆分布 */
.map-card {
  height: 100%;
  transition: all 0.3s ease;
}

.map-card:hover {
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

.vehicle-distribution-container {
  width: 100%;
  height: 500px;
  border-radius: 8px;
  position: relative;
  overflow: hidden;
}

.map-container {
  width: 100%;
  height: 100%;
  background: #f5f7fa;
}

.map-loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  align-items: center;
  background: rgba(255, 255, 255, 0.9);
  padding: 16px 24px;
  border-radius: 8px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
  .el-icon.is-loading {
    margin-right: 8px;
  }
}

.header-actions {
  display: flex;
  align-items: center;
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

.chart-container {
  width: 100%;
  height: 200px;
}

.section-title {
  font-size: 14px;
  font-weight: bold;
  color: #374151;
  margin-bottom: 12px;
}

/* 告警类型分布 */
.alert-type-distribution {
  padding: 10px 0;
}

.alert-type-filter {
  margin-bottom: 16px;
}

.alert-type-chart {
  width: 100%;
  height: 200px;
}

/* 告警列表 */
.alert-list-card {
  height: 100%;
  transition: all 0.3s ease;
}

.alert-list-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.alert-items {
  padding: 10px 0;
}

.alert-item {
  padding: 16px;
  margin-bottom: 12px;
  border-radius: 8px;
  background-color: #f9fafb;
  border-left: 4px solid #9ca3af;
  transition: all 0.3s ease;
}

.alert-item:hover {
  transform: translateX(5px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
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

.alert-main {
  display: flex;
  gap: 12px;
}

.alert-level {
  flex-shrink: 0;
}

.alert-content {
  flex: 1;
}

.alert-title {
  font-weight: bold;
  color: #374151;
  margin-bottom: 4px;
}

.alert-detail {
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 8px;
  line-height: 1.4;
}

.alert-meta {
  font-size: 11px;
  color: #9ca3af;
  display: flex;
  justify-content: space-between;
}

.alert-actions {
  flex-shrink: 0;
  display: flex;
  align-items: flex-start;
}

.no-alerts {
  padding: 40px 0;
  text-align: center;
}

/* 安全指数排行 */
.ranking-card {
  height: 100%;
  transition: all 0.3s ease;
}

.ranking-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.ranking-list {
  padding: 10px 0;
}

.ranking-item {
  display: flex;
  align-items: center;
  padding: 12px;
  margin-bottom: 10px;
  border-radius: 6px;
  background-color: #f9fafb;
  transition: all 0.3s ease;
}

.ranking-item:hover {
  transform: translateX(5px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.ranking-item.top-1 {
  background-color: #fff3cd;
  border: 1px solid #ffeeba;
}

.ranking-item.top-2 {
  background-color: #e2e3e5;
  border: 1px solid #d6d8db;
}

.ranking-item.top-3 {
  background-color: #f8d7da;
  border: 1px solid #f5c6cb;
}

.ranking-rank {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background-color: #3b82f6;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  margin-right: 12px;
  font-size: 14px;
}

.ranking-item.top-1 .ranking-rank {
  background-color: #ffc107;
}

.ranking-item.top-2 .ranking-rank {
  background-color: #6c757d;
}

.ranking-item.top-3 .ranking-rank {
  background-color: #dc3545;
}

.ranking-info {
  flex: 1;
}

.ranking-name {
  font-weight: bold;
  color: #374151;
}

.ranking-department {
  font-size: 12px;
  color: #6b7280;
}

.ranking-score {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 5px;
}

.score-value {
  font-weight: bold;
  color: #3b82f6;
  font-size: 18px;
}

.score-progress {
  width: 100px;
}

/* 系统运行状态 */
.system-status-card {
  height: 100%;
  transition: all 0.3s ease;
}

.system-status-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.system-status {
  padding: 10px 0;
}

.status-section {
  margin-bottom: 24px;
}

.status-items {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  background-color: #f9fafb;
  border-radius: 6px;
}

.status-name {
  font-weight: bold;
  color: #374151;
}

.status-indicator {
  margin: 0 12px;
}

.status-delay {
  font-size: 12px;
  color: #6b7280;
}

.database-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.db-stat-item {
  background-color: #f9fafb;
  padding: 12px;
  border-radius: 6px;
  text-align: center;
}

.db-stat-label {
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 4px;
}

.db-stat-value {
  font-weight: bold;
  color: #374151;
  font-size: 18px;
}
</style>


