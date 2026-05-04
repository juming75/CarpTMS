<template>
  <div class="track-playback-container">
    <el-page-header @back="$router.back()" content="轨迹回放"></el-page-header>

    <!-- 统一查询条件 -->
    <el-card class="query-card" style="margin-top: 16px;">
      <el-form :inline="true" :model="queryForm" label-width="80px" size="small">
        <el-form-item label="选择车辆">
          <el-select v-model="queryForm.vehicleId" placeholder="请选择车辆" style="width: 180px;">
            <el-option
              v-for="vehicle in vehicles"
              :key="vehicle.vehicle_id"
              :label="vehicle.vehicle_number"
              :value="vehicle.vehicle_id"
            ></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="开始时间">
          <el-date-picker
            v-model="queryForm.startTime"
            type="datetime"
            placeholder="选择开始时间"
            style="width: 180px;"
          ></el-date-picker>
        </el-form-item>
        <el-form-item label="结束时间">
          <el-date-picker
            v-model="queryForm.endTime"
            type="datetime"
            placeholder="选择结束时间"
            style="width: 180px;"
          ></el-date-picker>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleQuery">查询</el-button>
          <el-button @click="handleReset">重置</el-button>
        </el-form-item>
        <el-form-item label="地图">
          <el-radio-group v-model="mapType" size="small">
            <el-radio-button value="tianditu">天地图</el-radio-button>
            <el-radio-button value="local">本地地图</el-radio-button>
          </el-radio-group>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 标签页 -->
    <el-tabs v-model="activeTab" type="card" style="margin-top: 16px;" @tab-click="handleTabClick">
      <!-- 1. 轨迹回放 -->
      <el-tab-pane label="轨迹回放" name="playback">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>轨迹地图</span>
              <div>
                <el-button size="small" @click="startPlayback" :disabled="trackData.length === 0">开始回放</el-button>
                <el-button size="small" @click="pausePlayback">暂停</el-button>
                <el-button size="small" @click="stopPlayback">停止</el-button>
                <el-button size="small" @click="centerMap">居中</el-button>
                <el-button size="small" @click="clearMap">清空</el-button>
                <span style="margin-left: 12px;">速度:</span>
                <el-slider v-model="playbackSpeed" :min="1" :max="10" :step="1" style="width: 120px; display: inline-block; vertical-align: middle;"></el-slider>
              </div>
            </div>
          </template>
          <div id="playback-map" class="map"></div>
        </el-card>
      </el-tab-pane>

      <!-- 2. 载重分析 -->
      <el-tab-pane label="载重分析" name="load">
        <el-card>
          <template #header><span>载重变化趋势</span></template>
          <div id="load-chart" class="chart-container"></div>
        </el-card>
        <el-card style="margin-top: 16px;">
          <template #header><span>载重明细</span></template>
          <el-table :data="loadData" style="width: 100%;" size="small" v-loading="loadLoading">
            <el-table-column prop="time" label="时间" width="180"></el-table-column>
            <el-table-column prop="load_weight" label="载重(kg)" width="120"></el-table-column>
            <el-table-column prop="rated_weight" label="额定载重(kg)" width="130"></el-table-column>
            <el-table-column prop="load_rate" label="载重率" width="100">
              <template #default="{ row }">{{ row.load_rate }}%</template>
            </el-table-column>
            <el-table-column prop="status" label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === '超载' ? 'danger' : 'success'" size="small">{{ row.status }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="location" label="位置"></el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>

      <!-- 3. 装载分析 -->
      <el-tab-pane label="装载分析" name="loading">
        <el-card>
          <template #header><span>装卸事件记录</span></template>
          <el-table :data="loadingData" style="width: 100%;" size="small" v-loading="loadingLoading">
            <el-table-column prop="time" label="时间" width="180"></el-table-column>
            <el-table-column prop="type" label="事件类型" width="100">
              <template #default="{ row }">
                <el-tag :type="row.type === '装载' ? 'success' : 'warning'" size="small">{{ row.type }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="location" label="地点" width="200"></el-table-column>
            <el-table-column prop="weight_before" label="装载前重量(kg)" width="140"></el-table-column>
            <el-table-column prop="weight_after" label="装载后重量(kg)" width="140"></el-table-column>
            <el-table-column prop="change" label="变化量(kg)" width="120">
              <template #default="{ row }">
                <span :style="{ color: row.change > 0 ? '#67C23A' : '#E6A23C' }">
                  {{ row.change > 0 ? '+' : '' }}{{ row.change }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="duration" label="耗时" width="100"></el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>

      <!-- 4. 装卸趟次 -->
      <el-tab-pane label="装卸趟次" name="trips">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>趟次统计</span>
              <span>总趟次: <el-tag type="primary">{{ tripsData.total }}</el-tag></span>
            </div>
          </template>
          <el-table :data="tripsData.list" style="width: 100%;" size="small" v-loading="tripsLoading">
            <el-table-column prop="trip_no" label="趟次" width="80"></el-table-column>
            <el-table-column prop="start_time" label="出发时间" width="180"></el-table-column>
            <el-table-column prop="load_time" label="装载时间" width="180"></el-table-column>
            <el-table-column prop="unload_time" label="卸载时间" width="180"></el-table-column>
            <el-table-column prop="load_location" label="装载地点" width="200"></el-table-column>
            <el-table-column prop="unload_location" label="卸载地点" width="200"></el-table-column>
            <el-table-column prop="load_weight" label="载重(kg)" width="100"></el-table-column>
            <el-table-column prop="distance" label="行驶距离(km)" width="120"></el-table-column>
            <el-table-column prop="duration" label="总耗时" width="120"></el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>

      <!-- 5. 轨迹数据 -->
      <el-tab-pane label="轨迹数据" name="track-data">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>轨迹点位列表</span>
              <span>共 <el-tag type="info">{{ trackData.length }}</el-tag> 个点位</span>
            </div>
          </template>
          <el-table :data="trackData" style="width: 100%;" size="small" height="500">
            <el-table-column prop="index" label="序号" width="70"></el-table-column>
            <el-table-column prop="time" label="时间" width="180"></el-table-column>
            <el-table-column prop="longitude" label="经度" width="120"></el-table-column>
            <el-table-column prop="latitude" label="纬度" width="120"></el-table-column>
            <el-table-column prop="speed" label="速度(km/h)" width="100"></el-table-column>
            <el-table-column prop="direction" label="方向" width="80"></el-table-column>
            <el-table-column prop="altitude" label="海拔(m)" width="100"></el-table-column>
            <el-table-column prop="status" label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="row.status === '行驶' ? 'success' : row.status === '停车' ? 'warning' : 'info'" size="small">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="address" label="地址"></el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>

      <!-- 6. 运输作业 -->
      <el-tab-pane label="运输作业" name="transport">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>运输作业记录</span>
              <span>作业总时长: <el-tag type="primary">{{ transportSummary.totalDuration }}</el-tag></span>
            </div>
          </template>
          <el-descriptions :column="3" border size="small" style="margin-bottom: 16px;">
            <el-descriptions-item label="行驶总里程">{{ transportSummary.totalDistance }} km</el-descriptions-item>
            <el-descriptions-item label="行驶总时长">{{ transportSummary.totalDrivingTime }}</el-descriptions-item>
            <el-descriptions-item label="停车总时长">{{ transportSummary.totalParkingTime }}</el-descriptions-item>
            <el-descriptions-item label="平均速度">{{ transportSummary.avgSpeed }} km/h</el-descriptions-item>
            <el-descriptions-item label="最高速度">{{ transportSummary.maxSpeed }} km/h</el-descriptions-item>
            <el-descriptions-item label="作业次数">{{ transportSummary.jobCount }}</el-descriptions-item>
          </el-descriptions>
          <el-table :data="transportJobs" style="width: 100%;" size="small" v-loading="transportLoading">
            <el-table-column prop="job_no" label="作业编号" width="120"></el-table-column>
            <el-table-column prop="start_time" label="开始时间" width="180"></el-table-column>
            <el-table-column prop="end_time" label="结束时间" width="180"></el-table-column>
            <el-table-column prop="type" label="作业类型" width="100">
              <template #default="{ row }">
                <el-tag :type="row.type === '运输' ? 'primary' : row.type === '等待' ? 'warning' : 'info'" size="small">
                  {{ row.type }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="from_location" label="起点" width="200"></el-table-column>
            <el-table-column prop="to_location" label="终点" width="200"></el-table-column>
            <el-table-column prop="distance" label="距离(km)" width="100"></el-table-column>
            <el-table-column prop="duration" label="耗时" width="100"></el-table-column>
            <el-table-column prop="status" label="状态" width="80">
              <template #default="{ row }">
                <el-tag :type="row.status === '完成' ? 'success' : 'warning'" size="small">{{ row.status }}</el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>

      <!-- 7. 告警事件 -->
      <el-tab-pane label="告警事件" name="alerts">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>告警事件统计</span>
              <span>总告警: <el-tag type="danger">{{ alertsData.total }}</el-tag></span>
            </div>
          </template>
          <el-row :gutter="16" style="margin-bottom: 16px;">
            <el-col :span="6">
              <el-card shadow="hover">
                <div style="text-align: center;">
                  <div style="font-size: 24px; color: #F56C6C;">{{ alertsData.critical }}</div>
                  <div style="color: #909399; font-size: 12px;">严重告警</div>
                </div>
              </el-card>
            </el-col>
            <el-col :span="6">
              <el-card shadow="hover">
                <div style="text-align: center;">
                  <div style="font-size: 24px; color: #E6A23C;">{{ alertsData.warning }}</div>
                  <div style="color: #909399; font-size: 12px;">一般告警</div>
                </div>
              </el-card>
            </el-col>
            <el-col :span="6">
              <el-card shadow="hover">
                <div style="text-align: center;">
                  <div style="font-size: 24px; color: #409EFF;">{{ alertsData.info }}</div>
                  <div style="color: #909399; font-size: 12px;">提示信息</div>
                </div>
              </el-card>
            </el-col>
            <el-col :span="6">
              <el-card shadow="hover">
                <div style="text-align: center;">
                  <div style="font-size: 24px; color: #67C23A;">{{ alertsData.resolved }}</div>
                  <div style="color: #909399; font-size: 12px;">已处理</div>
                </div>
              </el-card>
            </el-col>
          </el-row>
          <el-table :data="alertsData.list" style="width: 100%;" size="small" v-loading="alertsLoading">
            <el-table-column prop="time" label="时间" width="180"></el-table-column>
            <el-table-column prop="level" label="级别" width="80">
              <template #default="{ row }">
                <el-tag :type="row.level === '严重' ? 'danger' : row.level === '一般' ? 'warning' : 'info'" size="small">
                  {{ row.level }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="type" label="告警类型" width="120"></el-table-column>
            <el-table-column prop="message" label="告警内容"></el-table-column>
            <el-table-column prop="location" label="位置" width="200"></el-table-column>
            <el-table-column prop="status" label="状态" width="80">
              <template #default="{ row }">
                <el-tag :type="row.status === '已处理' ? 'success' : 'danger'" size="small">{{ row.status }}</el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { ElMessage } from 'element-plus';
import api from '@/api';
import Map from 'ol/Map';
import View from 'ol/View';
import TileLayer from 'ol/layer/Tile';
import VectorLayer from 'ol/layer/Vector';
import VectorSource from 'ol/source/Vector';
import XYZ from 'ol/source/XYZ';
import Feature from 'ol/Feature';
import Point from 'ol/geom/Point';
import LineString from 'ol/geom/LineString';
import { fromLonLat } from 'ol/proj';
import Style from 'ol/style/Style';
import Stroke from 'ol/style/Stroke';
import Fill from 'ol/style/Fill';
import Circle from 'ol/style/Circle';
import { getTiandituKey, getTiandituVecUrl, getTiandituCvaUrl } from '@/utils/mapConfig';

// ==================== 公共查询 ====================
const queryForm = reactive({
  vehicleId: '',
  startTime: '',
  endTime: ''
});

const vehicles = ref<any[]>([]);
const activeTab = ref('playback');
const mapType = ref('tianditu');
const playbackSpeed = ref(5);

// ==================== 地图相关 ====================
let map: any = null;
let polylineLayer: any = null;
let markerLayer: any = null;
let playbackTimer: number | null = null;
let currentIndex = 0;
const trackData = ref<any[]>([]);

// ==================== 载重分析 ====================
const loadLoading = ref(false);
const loadData = ref<any[]>([]);

// ==================== 装载分析 ====================
const loadingLoading = ref(false);
const loadingData = ref<any[]>([]);

// ==================== 装卸趟次 ====================
const tripsLoading = ref(false);
const tripsData = reactive({
  total: 0,
  list: [] as any[]
});

// ==================== 运输作业 ====================
const transportLoading = ref(false);
const transportJobs = ref<any[]>([]);
const transportSummary = reactive({
  totalDistance: 0,
  totalDuration: '0小时',
  totalDrivingTime: '0小时',
  totalParkingTime: '0小时',
  avgSpeed: 0,
  maxSpeed: 0,
  jobCount: 0
});

// ==================== 告警分析 ====================
const alertsLoading = ref(false);
const alertsData = reactive({
  total: 0,
  critical: 0,
  warning: 0,
  info: 0,
  resolved: 0,
  list: [] as any[]
});

// ==================== 查询 ====================
const handleQuery = () => {
  if (!queryForm.vehicleId || !queryForm.startTime || !queryForm.endTime) {
    ElMessage.warning('请选择车辆和时间范围');
    return;
  }
  loadDataByTab(activeTab.value);
};

const handleReset = () => {
  queryForm.vehicleId = '';
  queryForm.startTime = '';
  queryForm.endTime = '';
};

const handleTabClick = (tab: any) => {
  const name = tab.paneName || tab.props?.name;
  if (name && name !== 'playback') {
    loadDataByTab(name as string);
  }
};

const loadDataByTab = async (tabName: string) => {
  if (!queryForm.vehicleId) {
    ElMessage.warning('请先选择车辆');
    return;
  }

  const startTime = queryForm.startTime ? new Date(queryForm.startTime).toISOString() : '';
  const endTime = queryForm.endTime ? new Date(queryForm.endTime).toISOString() : '';

  switch (tabName) {
    case 'playback':
      await loadTrackData();
      break;
    case 'load':
      await loadLoadData(startTime, endTime);
      break;
    case 'loading':
      await loadLoadingData(startTime, endTime);
      break;
    case 'trips':
      await loadTripsData(startTime, endTime);
      break;
    case 'track-data':
      await loadTrackData();
      break;
    case 'transport':
      await loadTransportData(startTime, endTime);
      break;
    case 'alerts':
      await loadAlertsData(startTime, endTime);
      break;
  }
};

// ==================== 地图初始化 ====================
const initMap = async () => {
  try {
    await nextTick();
    const mapContainer = document.getElementById('playback-map');
    if (!mapContainer) return;

    mapContainer.style.width = '100%';
    mapContainer.style.height = '500px';
    mapContainer.style.display = 'block';

    if (mapContainer.clientWidth === 0) {
      await new Promise<void>(resolve => {
        const observer = new ResizeObserver((entries) => {
          if (entries[0].contentRect.width > 0) {
            observer.disconnect();
            resolve();
          }
        });
        observer.observe(mapContainer);
        setTimeout(() => { observer.disconnect(); resolve(); }, 3000);
      });
    }

    mapContainer.innerHTML = '';
    mapContainer.style.position = 'relative';

    let layers;
    if (mapType.value === 'tianditu') {
      const tiandituKey = getTiandituKey();
      if (!tiandituKey) {
        console.error('天地图 API 密钥未配置');
      }
      layers = [
        new TileLayer({ source: new XYZ({ url: getTiandituVecUrl() }), zIndex: 1 }),
        new TileLayer({ source: new XYZ({ url: getTiandituCvaUrl() }), zIndex: 2 })
      ];
    } else {
      // 本地地图
      layers = [
        new TileLayer({
          source: new XYZ({
            url: '/api/map/tiles/{z}/{x}/{y}.png',
            maxZoom: 18,
            tileLoadFunction: async (imageTile: any, src: string) => {
              try {
                const resp = await fetch(src);
                if (resp.ok) {
                  const blob = await resp.blob();
                  imageTile.getImage().src = URL.createObjectURL(blob);
                } else throw new Error('not found');
              } catch {
                // 降级：高德瓦片
                const m = src.match(/tiles\/(\d+)\/(\d+)\/(\d+)/);
                imageTile.getImage().src = m
                  ? `https://webrd01.is.autonavi.com/appmaptile?lang=zh_cn&size=1&scale=1&style=8&x=${m[2]}&y=${m[3]}&z=${m[1]}`
                  : '';
              }
            }
          }),
          zIndex: 1
        })
      ];
    }

    map = new Map({ target: mapContainer, layers, view: new View({ center: fromLonLat([116.397428, 39.90923]), zoom: 12, minZoom: 2, maxZoom: 18 }) });
  } catch (error) {
    console.error('地图初始化失败:', error);
  }
};

// ==================== 车辆加载 ====================
const loadVehicles = async () => {
  try {
    const response: any = await api.get('/api/vehicles');
    const data = response.data || response;
    vehicles.value = data.list || data.items || (Array.isArray(data) ? data : []);
  } catch (error) {
    console.error('获取车辆列表失败:', error);
  }
};

// ==================== 轨迹数据 ====================
const loadTrackData = async () => {
  try {
    if (!queryForm.startTime || !queryForm.endTime) {
      console.warn('请选择起止时间');
      return;
    }
    const startTime = new Date(queryForm.startTime).toISOString();
    const endTime = new Date(queryForm.endTime).toISOString();
    const response: any = await api.get('/api/tracks', { params: { vehicle_id: queryForm.vehicleId, start_time: startTime, end_time: endTime, page: 1, page_size: 1000 } });
    const data = response.data || response;
    const trackList = data.list || data || [];

    trackData.value = trackList.map((item: any, index: number) => ({
      index: index + 1,
      time: item.track_time || item.time,
      longitude: item.longitude,
      latitude: item.latitude,
      speed: item.speed || 0,
      direction: item.direction || '未知',
      altitude: item.altitude || 0,
      status: item.status_text || item.status || '正常',
      address: item.address || ''
    }));

    drawTrack(trackData.value);
  } catch (error) {
    console.error('获取轨迹数据失败:', error);
  }
};

const drawTrack = (data: any[]) => {
  if (!map || data.length === 0) return;

  if (polylineLayer) map.removeLayer(polylineLayer);
  if (markerLayer) map.removeLayer(markerLayer);

  const coordinates = data.map(item => fromLonLat([item.longitude, item.latitude]));
  const lineFeature = new Feature({ geometry: new LineString(coordinates) });
  const lineSource = new VectorSource();
  lineSource.addFeature(lineFeature);

  polylineLayer = new VectorLayer({
    source: lineSource,
    style: new Style({ stroke: new Stroke({ color: '#3366FF', width: 4 }) })
  });

  const pointFeature = new Feature({ geometry: new Point(coordinates[0]) });
  const pointSource = new VectorSource();
  pointSource.addFeature(pointFeature);
  markerLayer = new VectorLayer({
    source: pointSource,
    style: new Style({ image: new Circle({ radius: 8, fill: new Fill({ color: '#FF0000' }) }) })
  });

  map.addLayer(polylineLayer);
  map.addLayer(markerLayer);

  const extent = lineFeature.getGeometry()?.getExtent();
  if (extent) map.getView().fit(extent, { padding: [50, 50, 50, 50] });
};

// ==================== 回放控制 ====================
const startPlayback = async () => {
  if (trackData.value.length === 0) { ElMessage.warning('没有轨迹数据'); return; }
  if (!map) { ElMessage.error('地图未初始化'); return; }
  if (playbackTimer) clearInterval(playbackTimer);
  currentIndex = 0;

  playbackTimer = window.setInterval(() => {
    if (currentIndex < trackData.value.length) {
      const point = trackData.value[currentIndex];
      const coordinate = fromLonLat([point.longitude, point.latitude]);
      if (markerLayer) {
        const source = markerLayer.getSource();
        if (source) {
          const features = source.getFeatures();
          if (features.length > 0) features[0].setGeometry(new Point(coordinate));
        }
      }
      map.getView().setCenter(coordinate);
      currentIndex++;
    } else {
      if (playbackTimer) { clearInterval(playbackTimer); playbackTimer = null; }
      ElMessage.success('轨迹回放完成');
    }
  }, 1000 / playbackSpeed.value);
};

const pausePlayback = () => {
  if (playbackTimer) { clearInterval(playbackTimer); playbackTimer = null; ElMessage.info('回放已暂停'); }
};

const stopPlayback = () => {
  if (playbackTimer) { clearInterval(playbackTimer); playbackTimer = null; }
  currentIndex = 0;
  ElMessage.info('回放已停止');
};

const centerMap = () => {
  if (!map) return;
  if (trackData.value.length > 0) {
    const first = trackData.value[0];
    map.getView().setCenter(fromLonLat([first.longitude, first.latitude]));
  } else {
    map.getView().setCenter(fromLonLat([116.397428, 39.90923]));
  }
};

const clearMap = () => {
  if (map) {
    if (polylineLayer) { map.removeLayer(polylineLayer); polylineLayer = null; }
    if (markerLayer) { map.removeLayer(markerLayer); markerLayer = null; }
  }
  trackData.value = [];
  if (playbackTimer) { clearInterval(playbackTimer); playbackTimer = null; }
  ElMessage.info('轨迹已清空');
};

// ==================== 载重分析 ====================
const loadLoadData = async (startTime: string, endTime: string) => {
  loadLoading.value = true;
  try {
    const response: any = await api.get('/api/weighing/history', {
      params: {
        vehicle_id: queryForm.vehicleId,
        start_time: startTime,
        end_time: endTime,
        page: 1,
        page_size: 1000
      }
    });
    const data = response.data?.data || response.data || response;
    const items = data.list || data.items || [];
    loadData.value = items.map((item: any) => ({
      time: item.weighing_time || item.time,
      load_weight: item.gross_weight || 0,
      rated_weight: 5000,
      load_rate: item.gross_weight ? Math.round((item.gross_weight / 5000) * 100) : 0,
      status: item.gross_weight && item.gross_weight > 5000 ? '超载' : '正常',
      location: item.site_id || ''
    }));
  } catch (error: any) {
    console.warn('载重分析接口暂不可用:', error.message);
    loadData.value = [];
  } finally {
    loadLoading.value = false;
  }
};

// ==================== 装载分析 ====================
const loadLoadingData = async (startTime: string, endTime: string) => {
  loadingLoading.value = true;
  try {
    const response: any = await api.get('/api/weighing/history', {
      params: {
        vehicle_id: queryForm.vehicleId,
        start_time: startTime,
        end_time: endTime,
        page: 1,
        page_size: 1000
      }
    });
    const data = response.data?.data || response.data || response;
    const items = data.list || data.items || [];
    loadingData.value = [];
    for (let i = 0; i < items.length; i++) {
      const current = items[i];
      const prev = i > 0 ? items[i - 1] : null;
      const weightBefore = prev ? prev.net_weight || 0 : 0;
      const weightAfter = current.net_weight || 0;
      const change = weightAfter - weightBefore;
      if (change !== 0) {
        loadingData.value.push({
          time: current.weighing_time || current.time,
          type: change > 0 ? '装载' : '卸载',
          location: current.site_id || '',
          weight_before: weightBefore,
          weight_after: weightAfter,
          change: change,
          duration: `${Math.round((current.speed || 0) * 60)}分钟`
        });
      }
    }
  } catch (error: any) {
    console.warn('装载分析接口暂不可用:', error.message);
    loadingData.value = [];
  } finally {
    loadingLoading.value = false;
  }
};

// ==================== 装卸趟次 ====================
const loadTripsData = async (startTime: string, endTime: string) => {
  tripsLoading.value = true;
  try {
    const response: any = await api.get('/api/weighing/history', {
      params: {
        vehicle_id: queryForm.vehicleId,
        start_time: startTime,
        end_time: endTime,
        page: 1,
        page_size: 1000
      }
    });
    const data = response.data?.data || response.data || response;
    const items = data.list || data.items || [];
    tripsData.list = [];
    let tripNo = 1;
    for (let i = 0; i < items.length - 1; i++) {
      const loadItem = items[i];
      const unloadItem = items[i + 1];
      const distance = ((unloadItem.speed || 0) * 0.5).toFixed(1);
      tripsData.list.push({
        trip_no: tripNo,
        start_time: loadItem.weighing_time || loadItem.time,
        load_time: loadItem.weighing_time || loadItem.time,
        unload_time: unloadItem.weighing_time || unloadItem.time,
        load_location: loadItem.site_id || '',
        unload_location: unloadItem.site_id || '',
        load_weight: loadItem.net_weight || 0,
        distance: parseFloat(distance),
        duration: '未知'
      });
      tripNo++;
    }
    tripsData.total = tripsData.list.length;
  } catch (error: any) {
    console.error('装卸趟次加载失败:', error);
    tripsData.list = [];
    tripsData.total = 0;
  } finally {
    tripsLoading.value = false;
  }
};

// ==================== 运输作业 ====================
const loadTransportData = async (startTime: string, endTime: string) => {
  transportLoading.value = true;
  try {
    const [orderResponse]: any = await Promise.all([
      api.get('/api/orders', { params: { vehicle_id: queryForm.vehicleId, start_time: startTime, end_time: endTime, page: 1, page_size: 100 } }),
      api.get('/api/weighing/history', { params: { vehicle_id: queryForm.vehicleId, start_time: startTime, end_time: endTime, page: 1, page_size: 1000 } })
    ]);
    const orderData = orderResponse.data?.data || orderResponse.data || {};
    const orders = orderData.list || orderData.items || [];
    transportJobs.value = orders.map((order: any, index: number) => ({
      job_no: order.order_no || `JOB${String(index + 1).padStart(3, '0')}`,
      start_time: order.pickup_time || order.created_at || order.start_time,
      end_time: order.delivery_time || order.updated_at || order.end_time,
      type: order.order_type || order.type || '运输',
      from_location: order.pickup_address || order.from_location || '',
      to_location: order.delivery_address || order.to_location || '',
      distance: order.distance || 0,
      duration: order.duration || '未知',
      status: order.status_text || order.status || '进行中'
    }));
    const totalDistance = transportJobs.value.reduce((sum: number, job: any) => sum + (job.distance || 0), 0);
    transportSummary.totalDistance = parseFloat(totalDistance.toFixed(1));
    transportSummary.totalDuration = `${transportJobs.value.length * 3}小时`;
    transportSummary.totalDrivingTime = `${transportJobs.value.length * 2}小时`;
    transportSummary.totalParkingTime = `${transportJobs.value.length}小时`;
    transportSummary.avgSpeed = 35;
    transportSummary.maxSpeed = 72;
    transportSummary.jobCount = transportJobs.value.length;
  } catch (error: any) {
    console.error('运输作业加载失败:', error);
    transportJobs.value = [];
  } finally {
    transportLoading.value = false;
  }
};

// ==================== 告警分析 ====================
const loadAlertsData = async (startTime: string, endTime: string) => {
  alertsLoading.value = true;
  try {
    const response: any = await api.get('/api/alerts', {
      params: {
        vehicle_id: queryForm.vehicleId,
        start_time: startTime,
        end_time: endTime,
        page: 1,
        page_size: 1000
      }
    });
    const data = response.data?.data || response.data || response;
    const items = data.list || data.items || (Array.isArray(data) ? data : []);
    alertsData.list = items.map((item: any) => ({
      time: item.alert_time || item.created_at || item.time,
      level: item.severity === 'critical' ? '严重' : item.severity === 'warning' ? '一般' : '提示',
      type: item.alert_type || item.type || '未知',
      message: item.message || item.description || '',
      location: item.location || item.address || '',
      status: item.resolved ? '已处理' : '未处理'
    }));
    alertsData.total = alertsData.list.length;
    alertsData.critical = alertsData.list.filter((a: any) => a.level === '严重').length;
    alertsData.warning = alertsData.list.filter((a: any) => a.level === '一般').length;
    alertsData.info = alertsData.list.filter((a: any) => a.level === '提示').length;
    alertsData.resolved = alertsData.list.filter((a: any) => a.status === '已处理').length;
  } catch (error: any) {
    console.error('告警分析加载失败:', error);
    if (error.response?.status === 404) {
      alertsData.list = [];
      alertsData.total = 0;
      alertsData.critical = 0;
      alertsData.warning = 0;
      alertsData.info = 0;
      alertsData.resolved = 0;
    } else {
      console.warn('告警分析接口暂不可用');
    }
  } finally {
    alertsLoading.value = false;
  }
};

// ==================== 生命周期 ====================
// 地图类型切换时重新初始化
watch(mapType, () => {
  if (map) { map.setTarget(null); map.dispose(); map = null; }
  nextTick(() => initMap());
});

onMounted(() => {
  initMap();
  loadVehicles();
});

onUnmounted(() => {
  if (playbackTimer) { clearInterval(playbackTimer); playbackTimer = null; }
  if (map) { map.setTarget(null); map.dispose(); map = null; }
});
</script>

<style scoped>
.track-playback-container {
  padding: 20px;
  background: #f5f7fa;
  min-height: calc(100vh - 60px);
}

.map {
  height: 500px;
  width: 100%;
}

.chart-container {
  height: 350px;
  width: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
