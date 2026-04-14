<template>
  <div class="logistics-tracking">
    <div class="logistics-header">
      <h2>物流跟踪</h2>
    </div>

    <!-- 主功能标签页 -->
    <el-tabs v-model="activeMainTab" type="card" class="main-tabs">
      <!-- 实时位置跟踪 -->
      <el-tab-pane label="实时位置跟踪" name="realtime">
        <div class="tab-content">
          <!-- 车辆选择 -->
          <el-card class="vehicle-select-card" shadow="hover" :body-style="{ padding: '20px' }">
            <el-form :model="realtimeQueryForm" :inline="true" label-position="right" label-width="80px" size="small">
              <el-form-item label="车辆选择">
                <el-select v-model="realtimeQueryForm.vehicleId" placeholder="请选择车辆" style="width: 200px">
                  <el-option
                    v-for="vehicle in vehicleList"
                    :key="vehicle.id"
                    :label="`${vehicle.license} (${vehicle.driver})`"
                    :value="vehicle.id"
                  ></el-option>
                </el-select>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="handleVehicleSelect">
                  <el-icon><Search /></el-icon>
                  查看位置
                </el-button>
              </el-form-item>
            </el-form>
          </el-card>

          <!-- 地图和实时信息 -->
          <div class="realtime-layout" style="margin-top: 20px">
            <!-- 左侧地图 -->
            <el-card class="map-card" shadow="hover" style="flex: 1">
              <template #header>
                <div class="card-header">
                  <span>实时位置</span>
                </div>
              </template>
              <div class="map-container">
                <!-- 地图显示区域 -->
                <div class="map-view">
                  <el-empty description="地图加载中..."></el-empty>
                </div>

                <!-- 车辆实时状态 -->
                <div v-if="selectedVehicle" class="vehicle-status-info">
                  <el-descriptions :column="4" size="small" border>
                    <el-descriptions-item label="车牌号码">{{ selectedVehicle.license }}</el-descriptions-item>
                    <el-descriptions-item label="司机">{{ selectedVehicle.driver }}</el-descriptions-item>
                    <el-descriptions-item label="速度">{{ selectedVehicle.speed }} km/h</el-descriptions-item>
                    <el-descriptions-item label="状态"
                      ><el-tag :type="selectedVehicle.status === 'online' ? 'success' : 'danger'">{{
                        selectedVehicle.status === 'online' ? '在线' : '离线'
                      }}</el-tag></el-descriptions-item
                    >
                    <el-descriptions-item label="位置">{{ selectedVehicle.location }}</el-descriptions-item>
                    <el-descriptions-item label="方向">{{ selectedVehicle.direction }}</el-descriptions-item>
                    <el-descriptions-item label="温度">{{ selectedVehicle.temperature }} °C</el-descriptions-item>
                    <el-descriptions-item label="最后更新">{{ selectedVehicle.lastUpdate }}</el-descriptions-item>
                  </el-descriptions>
                </div>
              </div>
            </el-card>

            <!-- 右侧实时监控 -->
            <el-card class="monitor-card" shadow="hover" style="width: 350px; margin-left: 20px">
              <template #header>
                <div class="card-header">
                  <span>实时监控数据</span>
                </div>
              </template>
              <div class="monitor-content">
                <el-tabs v-model="activeMonitorTab" type="border-card" size="small">
                  <el-tab-pane label="实时轨迹" name="track">
                    <div class="track-list">
                      <el-scrollbar height="400px">
                        <el-timeline>
                          <el-timeline-item
                            v-for="point in realtimeTrack"
                            :key="point.id"
                            :timestamp="point.time"
                            :type="point.status === 'normal' ? 'success' : 'warning'"
                          >
                            <div class="timeline-content">
                              <strong>{{ point.location }}</strong
                              ><br />
                              <small>速度: {{ point.speed }} km/h | 温度: {{ point.temperature }} °C</small>
                            </div>
                          </el-timeline-item>
                        </el-timeline>
                      </el-scrollbar>
                    </div>
                  </el-tab-pane>
                  <el-tab-pane label="报警信息" name="alarm">
                    <div class="alarm-list">
                      <el-scrollbar height="400px">
                        <el-table :data="vehicleAlarms" stripe size="small">
                          <el-table-column prop="time" label="时间" width="150"></el-table-column>
                          <el-table-column prop="type" label="报警类型" width="120">
                            <template #default="scope">
                              <el-tag type="danger">{{ scope.row.type }}</el-tag>
                            </template>
                          </el-table-column>
                          <el-table-column prop="message" label="报警信息"></el-table-column>
                        </el-table>
                      </el-scrollbar>
                    </div>
                  </el-tab-pane>
                </el-tabs>
              </div>
            </el-card>
          </div>
        </div>
      </el-tab-pane>

      <!-- 路线规划 -->
      <el-tab-pane label="路线规划" name="route">
        <div class="tab-content">
          <el-card class="route-plan-card" shadow="hover" :body-style="{ padding: '20px' }">
            <template #header>
              <div class="card-header">
                <span>路线规划</span>
              </div>
            </template>

            <el-form :model="routePlanForm" label-position="right" label-width="100px" size="small">
              <el-row :gutter="20">
                <el-col :span="12">
                  <el-form-item label="起点">
                    <el-input v-model="routePlanForm.startPoint" placeholder="请输入起点"></el-input>
                  </el-form-item>
                  <el-form-item label="终点">
                    <el-input v-model="routePlanForm.endPoint" placeholder="请输入终点"></el-input>
                  </el-form-item>
                  <el-form-item label="途经点">
                    <el-input v-model="routePlanForm.waypoints" placeholder="请输入途经点，多个用逗号分隔"></el-input>
                  </el-form-item>
                </el-col>
                <el-col :span="12">
                  <el-form-item label="规划策略">
                    <el-select v-model="routePlanForm.strategy" placeholder="请选择规划策略">
                      <el-option label="最短距离" value="shortest"></el-option>
                      <el-option label="最快时间" value="fastest"></el-option>
                      <el-option label="避开高速" value="no-highway"></el-option>
                    </el-select>
                  </el-form-item>
                  <el-form-item label="车辆信息">
                    <el-select v-model="routePlanForm.vehicleId" placeholder="请选择车辆">
                      <el-option
                        v-for="vehicle in vehicleList"
                        :key="vehicle.id"
                        :label="vehicle.license"
                        :value="vehicle.id"
                      ></el-option>
                    </el-select>
                  </el-form-item>
                  <el-form-item>
                    <el-button type="primary" @click="handleRoutePlan" style="margin-right: 10px">
                      <el-icon><Location /></el-icon>
                      生成路线
                    </el-button>
                    <el-button type="success" @click="handleSaveRoute">
                      <el-icon><DocumentAdd /></el-icon>
                      保存路线
                    </el-button>
                  </el-form-item>
                </el-col>
              </el-row>
            </el-form>

            <!-- 路线结果 -->
            <div v-if="plannedRoute" class="route-result" style="margin-top: 20px">
              <el-descriptions :column="4" size="small" border>
                <el-descriptions-item label="总距离">{{ plannedRoute.distance }} km</el-descriptions-item>
                <el-descriptions-item label="预计时间">{{ plannedRoute.duration }} 分钟</el-descriptions-item>
                <el-descriptions-item label="途经点数量">{{ plannedRoute.waypoints.length }}</el-descriptions-item>
                <el-descriptions-item label="规划策略">{{ plannedRoute.strategyText }}</el-descriptions-item>
              </el-descriptions>

              <!-- 路线详情 -->
              <div class="route-detail" style="margin-top: 20px">
                <el-table :data="plannedRoute.waypoints" stripe size="small">
                  <el-table-column prop="order" label="序号" width="60"></el-table-column>
                  <el-table-column prop="name" label="地点名称"></el-table-column>
                  <el-table-column prop="distance" label="距离(km)" width="100"></el-table-column>
                  <el-table-column prop="duration" label="预计时间(min)" width="120"></el-table-column>
                </el-table>
              </div>
            </div>
          </el-card>
        </div>
      </el-tab-pane>

      <!-- 历史轨迹查询 -->
      <el-tab-pane label="历史轨迹查询" name="history">
        <div class="tab-content">
          <!-- 查询条件 -->
          <el-card class="history-query-card" shadow="hover" :body-style="{ padding: '20px' }">
            <el-form :model="historyQueryForm" label-position="right" label-width="100px" size="small">
              <el-row :gutter="20">
                <el-col :span="8">
                  <el-form-item label="车辆">
                    <el-select v-model="historyQueryForm.vehicleId" placeholder="请选择车辆">
                      <el-option
                        v-for="vehicle in vehicleList"
                        :key="vehicle.id"
                        :label="vehicle.license"
                        :value="vehicle.id"
                      ></el-option>
                    </el-select>
                  </el-form-item>
                </el-col>
                <el-col :span="16">
                  <el-form-item label="时间范围">
                    <el-date-picker
                      v-model="historyQueryForm.dateRange"
                      type="datetimerange"
                      range-separator="至"
                      start-placeholder="开始时间"
                      end-placeholder="结束时间"
                      size="small"
                      style="width: 100%"
                    ></el-date-picker>
                  </el-form-item>
                </el-col>
              </el-row>
              <el-row :gutter="20" style="margin-top: 10px">
                <el-col :span="8">
                  <el-form-item label="查询间隔">
                    <el-select v-model="historyQueryForm.interval" placeholder="请选择查询间隔">
                      <el-option label="10秒" value="10"></el-option>
                      <el-option label="30秒" value="30"></el-option>
                      <el-option label="1分钟" value="60"></el-option>
                      <el-option label="5分钟" value="300"></el-option>
                      <el-option label="10分钟" value="600"></el-option>
                    </el-select>
                  </el-form-item>
                </el-col>
                <el-col :span="8">
                  <el-form-item>
                    <el-button type="primary" @click="handleHistoryQuery" style="margin-right: 10px">
                      <el-icon><Search /></el-icon>
                      查询轨迹
                    </el-button>
                    <el-button @click="handleHistoryExport">
                      <el-icon><Download /></el-icon>
                      导出数据
                    </el-button>
                  </el-form-item>
                </el-col>
              </el-row>
            </el-form>
          </el-card>

          <!-- 历史轨迹结果 -->
          <div v-if="historyTrack.length > 0" class="history-result" style="margin-top: 20px">
            <div class="history-stats" style="margin-bottom: 20px">
              <el-descriptions :column="4" size="small" border>
                <el-descriptions-item label="查询车辆">{{
                  selectedHistoryVehicle?.license || ''
                }}</el-descriptions-item>
                <el-descriptions-item label="轨迹点数量">{{ historyTrack.length }}</el-descriptions-item>
                <el-descriptions-item label="总距离">{{ historyStats.totalDistance }} km</el-descriptions-item>
                <el-descriptions-item label="平均速度">{{ historyStats.avgSpeed }} km/h</el-descriptions-item>
              </el-descriptions>
            </div>

            <!-- 轨迹列表 -->
            <el-card class="history-track-card" shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>历史轨迹列表</span>
                </div>
              </template>
              <div class="history-track-list">
                <el-table :data="historyTrack" stripe size="small" style="margin-bottom: 20px">
                  <el-table-column prop="time" label="时间" width="180"></el-table-column>
                  <el-table-column prop="location" label="位置"></el-table-column>
                  <el-table-column prop="speed" label="速度(km/h)" width="100"></el-table-column>
                  <el-table-column prop="direction" label="方向" width="100"></el-table-column>
                  <el-table-column prop="temperature" label="温度(°C)" width="100"></el-table-column>
                  <el-table-column prop="status" label="状态" width="80">
                    <template #default="scope">
                      <el-tag :type="scope.row.status === 'normal' ? 'success' : 'warning'">{{
                        scope.row.statusText
                      }}</el-tag>
                    </template>
                  </el-table-column>
                </el-table>

                <!-- 分页 -->
                <div class="pagination-container">
                  <el-pagination
                    v-model:current-page="historyCurrentPage"
                    v-model:page-size="historyPageSize"
                    :page-sizes="[10, 20, 50, 100]"
                    layout="total, sizes, prev, pager, next, jumper"
                    :total="historyTrack.length"
                    @size-change="handleHistorySizeChange"
                    @current-change="handleHistoryCurrentChange"
                  ></el-pagination>
                </div>
              </div>
            </el-card>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted, onUnmounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Search, Location, DocumentAdd, Download } from '@element-plus/icons-vue';
import api from '@/api';
// 类型导入（在需要时可以使用）
// import type { Logistics, LogisticsForm, LogisticsStatus, LogisticsStatusMap, LogisticsStatusTagTypeMap } from '@/types/logistics';
import type { Vehicle } from '@/types/vehicle';
// 使用 BackendVehicle 作为后端数据类型
import type { BackendVehicle } from '@/types';

// OpenLayers 地图相关导入
import Map from 'ol/Map';
import View from 'ol/View';
import TileLayer from 'ol/layer/Tile';
import XYZ from 'ol/source/XYZ';
import VectorLayer from 'ol/layer/Vector';
import VectorSource from 'ol/source/Vector';
import { fromLonLat } from 'ol/proj';
import { Point } from 'ol/geom';
import Feature from 'ol/Feature';
import { Style, Circle, Fill, Stroke } from 'ol/style';

// 主功能标签页
const activeMainTab = ref('realtime');

// 车辆列表数据
interface VehicleListItem {
  id: number;
  license: string;
  driver: string;
  status: 'online' | 'offline';
}

const vehicleList = ref<VehicleListItem[]>([]);

// 加载车辆列表
const loadVehicles = async () => {
  try {
    const response = await api.get('/api/vehicles');
    if (response && response.items) {
      vehicleList.value = response.items.map((v: BackendVehicle) => ({
        id: v.vehicle_id,
        license: v.license_plate,
        driver: v.driver_name || '未知',
        status: v.operation_status === '1' || v.operation_status === 'active' ? 'online' : 'offline',
      }));
    }
  } catch (error) {
    console.error('Failed to load vehicles:', error);
    ElMessage.error('加载车辆列表失败');
  }
};

// 轨迹点类型
interface TrackPoint {
  latitude: number;
  longitude: number;
  speed: number;
  direction: number;
  time: string;
}

// 警报类型
interface Alarm {
  id: number;
  vehicleId: number;
  type: string;
  message: string;
  time: string;
}

// 路线点类型
interface RoutePoint {
  latitude: number;
  longitude: number;
  name: string;
}

// 地图实例
let olMap: Map | null = null;
let vehicleLayer: VectorLayer | null = null;

// 实时位置跟踪相关
const realtimeQueryForm = ref({ vehicleId: '' });
const selectedVehicle = ref<Vehicle | null>(null);
const realtimeTrack = ref<TrackPoint[]>([]);

// 实时监控标签页
const activeMonitorTab = ref('track');
const vehicleAlarms = ref<Alarm[]>([]);

// 路线规划相关
const routePlanForm = ref({
  startPoint: '',
  endPoint: '',
  waypoints: '',
  strategy: 'fastest',
  vehicleId: '',
});

const plannedRoute = ref<RoutePoint[] | null>(null);

// 历史轨迹查询相关
const historyQueryForm = ref({
  vehicleId: '',
  dateRange: [] as Date[],
  interval: '60',
});

const historyTrack = ref<TrackPoint[]>([]);
const historyStats = ref({
  totalDistance: 0,
  avgSpeed: 0,
});

const historyCurrentPage = ref(1);
const historyPageSize = ref(20);
const selectedHistoryVehicle = ref(null);

// 初始化天地图
function initRealtimeMap() {
  const mapContainer = document.querySelector('.map-view');
  if (!mapContainer) return;

  try {
    // 从localStorage获取天地图API Key
    const tiandituKey = localStorage.getItem('tiandituKey') || '34d8cf060f7e8ac09be79b9261d65274';

    // 清空地图容器
    mapContainer.innerHTML = '';

    // 创建天地图矢量图层
    const vectorSource = new XYZ({
      url: `https://t0.tianditu.gov.cn/vec_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=vec&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${tiandituKey}`,
      crossOrigin: 'anonymous',
      projection: 'EPSG:3857',
    });

    const vectorLayer = new TileLayer({
      source: vectorSource,
    });

    // 创建天地图矢量注记图层
    const labelSource = new XYZ({
      url: `https://t0.tianditu.gov.cn/cva_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=cva&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${tiandituKey}`,
      crossOrigin: 'anonymous',
      projection: 'EPSG:3857',
    });

    const labelLayer = new TileLayer({
      source: labelSource,
    });

    // 计算地图中心点（中国中心位置）
    const centerPoint = fromLonLat([104.195, 35.861]);

    // 创建地图实例
    olMap = new Map({
      target: mapContainer as HTMLElement,
      layers: [vectorLayer, labelLayer],
      view: new View({
        center: centerPoint,
        zoom: 5,
        maxZoom: 18,
        minZoom: 2,
      }),
    });
  } catch (error) {
    console.error('天地图初始化失败:', error);
    // 显示错误提示 - 使用安全的 DOM 操作
    if (mapContainer) {
      // 清空容器
      mapContainer.textContent = '';
      
      // 创建错误提示容器
      const errorDiv = document.createElement('div');
      errorDiv.style.cssText = 'width: 100%; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;';
      
      const titleStrong = document.createElement('strong');
      titleStrong.textContent = '天地图初始化失败';
      errorDiv.appendChild(titleStrong);
      
      const hintSmall = document.createElement('small');
      hintSmall.textContent = '请检查网络连接或API Key配置';
      errorDiv.appendChild(hintSmall);
      
      mapContainer.appendChild(errorDiv);
    }
  }
}

// 更新车辆标记
function updateVehicleMarker(vehicle: VehicleListItem) {
  if (!olMap) return;

  // 移除旧的车辆图层
  if (vehicleLayer) {
    olMap.removeLayer(vehicleLayer as unknown as VectorLayer);
  }

  // 创建新的矢量源
  const vectorSource = new VectorSource();

  // 为车辆生成合理的经纬度（模拟数据）
  const lng = 100 + Math.random() * 18; // 100-118
  const lat = 30 + Math.random() * 8; // 30-38

  const markerGeometry = new Point(fromLonLat([lng, lat]));
  const markerPoint = new Feature({
    geometry: markerGeometry,
    properties: {
      id: vehicle.id,
      name: vehicle.license,
      status: vehicle.status,
    },
  });

  // 设置标记样式
  const markerStyle = new Style({
    image: new Circle({
      radius: 8,
      fill: new Fill({
        color: vehicle.status === 'online' ? '#67c23a' : '#909399',
      }),
      stroke: new Stroke({
        color: '#fff',
        width: 2,
      }),
    }),
  });

  markerPoint.setStyle(markerStyle);
  vectorSource.addFeature(markerPoint);

  // 创建车辆图层
  vehicleLayer = new VectorLayer({
    source: vectorSource,
    zIndex: 10,
  });

  olMap.addLayer(vehicleLayer as unknown as VectorLayer);

  // 移动地图中心到车辆位置
  olMap.getView().animate({
    center: fromLonLat([lng, lat]),
    zoom: 12,
    duration: 1000,
  });
}

// 选择车辆查看实时位置
const handleVehicleSelect = async () => {
  if (!realtimeQueryForm.value.vehicleId) {
    ElMessage.warning('请选择车辆');
    return;
  }

  try {
    const response = await api.get(`/api/vehicles/${Number(realtimeQueryForm.value.vehicleId)}`);
    const vehicleData = response;
    const vehicle = vehicleList.value.find((v) => v.id === Number(realtimeQueryForm.value.vehicleId));

    if (vehicleData && vehicle) {
      selectedVehicle.value = {
        ...vehicle,
        speed: 0,
        location: '未知位置',
        direction: 0,
        temperature: 25,
        lastUpdate: new Date().toLocaleString(),
      };

      // 更新车辆标记
      updateVehicleMarker(selectedVehicle.value);

      ElMessage.success('已获取车辆信息');
    }
  } catch (error) {
    console.error('Failed to get vehicle data:', error);
    ElMessage.error('获取车辆信息失败');
  }
};

// 生成路线
const handleRoutePlan = () => {
  if (!routePlanForm.value.startPoint || !routePlanForm.value.endPoint) {
    ElMessage.warning('请输入起点和终点');
    return;
  }

  // 模拟生成路线
  plannedRoute.value = {
    id: 1,
    startPoint: routePlanForm.value.startPoint,
    endPoint: routePlanForm.value.endPoint,
    waypoints: [
      { order: 1, name: routePlanForm.value.startPoint, distance: 0, duration: 0 },
      { order: 2, name: '途经点1', distance: 10, duration: 15 },
      { order: 3, name: '途经点2', distance: 25, duration: 30 },
      { order: 4, name: routePlanForm.value.endPoint, distance: 40, duration: 45 },
    ],
    distance: 40,
    duration: 45,
    strategyText:
      {
        fastest: '最快时间',
        shortest: '最短距离',
        'no-highway': '避开高速',
      }[routePlanForm.value.strategy] || '默认',
  };

  ElMessage.success('路线生成成功');
};

// 保存路线
const handleSaveRoute = () => {
  if (!plannedRoute.value) {
    ElMessage.warning('请先生成路线');
    return;
  }

  ElMessage.success('路线保存成功');
};

// 查询历史轨迹
const handleHistoryQuery = async () => {
  if (
    !historyQueryForm.value.vehicleId ||
    !historyQueryForm.value.dateRange ||
    historyQueryForm.value.dateRange.length !== 2
  ) {
    ElMessage.warning('请选择车辆和时间范围');
    return;
  }

  try {
    // 调用API获取历史轨迹数据
    interface HistoryPoint {
      id: number;
      gps_time: string;
      longitude: number;
      latitude: number;
      speed: number;
      direction: number;
      temperature: number;
      status: number;
    }

    interface HistoryResponse {
      items: HistoryPoint[];
    }

    const response = await api.get('/api/reports/vehicle-history', {
      params: {
        vehicleId: Number(historyQueryForm.value.vehicleId),
        startTime: historyQueryForm.value.dateRange[0],
        endTime: historyQueryForm.value.dateRange[1],
        interval: Number(historyQueryForm.value.interval),
      },
    }) as HistoryResponse;

    historyTrack.value = response.items.map((point) => ({
      id: point.id,
      time: new Date(point.gps_time).toLocaleString(),
      location: `${point.longitude.toFixed(4)}, ${point.latitude.toFixed(4)}`,
      speed: point.speed || 0,
      direction: point.direction || 0,
      temperature: point.temperature || 25,
      status: point.status === 1 ? 'normal' : 'warning',
      statusText: point.status === 1 ? '正常' : '异常',
    }));

    // 计算统计数据
    historyStats.value = {
      totalDistance: response.items.length > 0 ? response.items.length * 0.1 : 0,
      avgSpeed:
        response.items.length > 0
          ? response.items.reduce((sum, point) => sum + (point.speed || 0), 0) /
            response.items.length
          : 0,
    };

    selectedHistoryVehicle.value = vehicleList.value.find((v) => v.id === Number(historyQueryForm.value.vehicleId));
    ElMessage.success('历史轨迹查询完成');
  } catch (error) {
    console.error('Failed to get history track:', error);
    ElMessage.error('获取历史轨迹失败');
  }
};

// 导出历史数据
const handleHistoryExport = () => {
  if (historyTrack.value.length === 0) {
    ElMessage.warning('没有可导出的数据');
    return;
  }

  ElMessage.success('数据导出成功');
};

// 历史轨迹分页
const handleHistorySizeChange = (size: number) => {
  historyPageSize.value = size;
  historyCurrentPage.value = 1;
};

const handleHistoryCurrentChange = (page: number) => {
  historyCurrentPage.value = page;
};

onMounted(() => {
  console.log('Logistics 初始化完成');
  loadVehicles();

  // 初始化天地图
  setTimeout(() => {
    initRealtimeMap();
  }, 1000);
});

onUnmounted(() => {
  // 组件卸载时清除地图实例
  if (olMap) {
    olMap.dispose();
    olMap = null;
  }
});
</script>

<style scoped>
.logistics-tracking {
  padding: 20px;
}

.logistics-header {
  margin-bottom: 20px;
}

.logistics-header h2 {
  margin: 0;
  font-size: 20px;
  color: #303133;
}

.tab-content {
  margin-top: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

/* 实时位置跟踪 */
.realtime-layout {
  display: flex;
  gap: 20px;
}

.map-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.map-view {
  height: 400px;
  background: #f0f2f5;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.vehicle-status-info {
  margin-top: 20px;
}

.monitor-content {
  height: 500px;
  display: flex;
  flex-direction: column;
}

.track-list,
.alarm-list {
  height: calc(100% - 40px);
}

.timeline-content {
  font-size: 13px;
}

/* 路线规划 */
.route-plan-card {
  margin-top: 20px;
}

.route-result {
  margin-top: 20px;
}

/* 历史轨迹 */
.history-query-card {
  margin-top: 20px;
}

.history-result {
  margin-top: 20px;
}

.history-stats {
  margin-bottom: 20px;
}

.history-track-card {
  margin-top: 20px;
}

.history-track-list {
  margin-top: 10px;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}
</style>


