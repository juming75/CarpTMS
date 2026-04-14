<template>
  <div class="track-playback">
    <div class="track-toolbar">
      <el-card class="toolbar-card">
        <div class="toolbar-row">
          <div class="toolbar-section">
            <span class="section-title">轨迹查询</span>
            <el-date-picker
              v-model="dateRange"
              type="datetimerange"
              range-separator="至"
              start-placeholder="开始时间"
              end-placeholder="结束时间"
              size="small"
              :disabled="isPlaying"
            />
          </div>

          <div class="toolbar-section">
            <span class="section-title">车辆选择</span>
            <el-select v-model="selectedVehicle" placeholder="选择车辆" size="small" :disabled="isPlaying" filterable>
              <el-option v-for="vehicle in vehicleList" :key="vehicle.id" :label="vehicle.name" :value="vehicle.id">
                <span>{{ vehicle.name }}</span>
                <span style="color: #999; font-size: 12px; margin-left: 10px">
                  {{ vehicle.plateNumber }}
                </span>
              </el-option>
            </el-select>
          </div>

          <div class="toolbar-section">
            <el-button
              type="primary"
              size="small"
              :icon="Search"
              :loading="isLoading"
              :disabled="!canQuery || isPlaying"
              @click="queryTrack"
            >
              查询轨迹
            </el-button>
            <el-button size="small" :icon="Delete" :disabled="isPlaying || trackData.length === 0" @click="clearTrack">
              清除
            </el-button>
          </div>
        </div>

        <div class="toolbar-row playback-controls" v-if="trackData.length > 0">
          <div class="toolbar-section">
            <span class="section-title">播放控制</span>
            <el-button-group>
              <el-button
                :type="isPlaying ? 'warning' : 'primary'"
                size="small"
                :icon="isPlaying ? VideoPause : VideoPlay"
                @click="togglePlay"
              >
                {{ isPlaying ? '暂停' : '播放' }}
              </el-button>
              <el-button size="small" :icon="Close" @click="stopPlay" :disabled="!isPlaying && currentIndex === 0">
                停止
              </el-button>
              <el-button size="small" :icon="FullScreen" @click="toggleFullScreen"> 全屏 </el-button>
            </el-button-group>

            <span class="playback-info">
              <span class="current-point">{{ currentIndex + 1 }}</span>
              <span class="separator">/</span>
              <span class="total-points">{{ trackData.length }}</span>
            </span>

            <div class="speed-control">
              <span class="speed-label">速度:</span>
              <el-slider
                v-model="playbackSpeed"
                :min="1"
                :max="10"
                :step="1"
                :marks="speedMarks"
                :format-tooltip="(val) => val + 'x'"
              />
            </div>

            <div class="basic-functions">
              <el-button-group>
                <el-button size="small" @click="toggleMotionTrack">运动轨迹</el-button>
                <el-button size="small" @click="togglePointTrack">点轨迹</el-button>
                <el-button size="small" @click="showAllTrack">全部轨迹</el-button>
                <el-button size="small" @click="clearTrack">清除轨迹</el-button>
                <el-button size="small" @click="toggleShowAddress">显示地址</el-button>
                <el-button size="small" @click="queryStopRecords">停车查询</el-button>
                <el-button size="small" @click="exportData">导出数据</el-button>
              </el-button-group>
            </div>
          </div>
        </div>
      </el-card>
    </div>

    <div class="track-main">
      <div class="track-map" ref="trackMapRef"></div>

      <div class="track-info-panel" v-if="trackData.length > 0">
        <el-card class="info-card">
          <template #header>
            <div class="card-header">
              <span>轨迹信息</span>
              <el-tag type="info" size="small"> 总里程: {{ totalMileage }} km </el-tag>
            </div>
          </template>

          <div class="current-info" v-if="currentPointData">
            <div class="info-row">
              <span class="info-label">当前时间:</span>
              <span class="info-value">{{ formatDateTime(currentPointData.gpsTime) }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">位置:</span>
              <span class="info-value">
                ({{ currentPointData.lng.toFixed(6) }}, {{ currentPointData.lat.toFixed(6) }})
              </span>
            </div>
            <div class="info-row">
              <span class="info-label">速度:</span>
              <span class="info-value">{{ currentPointData.speed }} km/h</span>
            </div>
            <div class="info-row">
              <span class="info-label">方向:</span>
              <span class="info-value">{{ getDirection(currentPointData.direction) }}</span>
            </div>
            <div class="info-row">
              <span class="info-label">状态:</span>
              <el-tag :type="currentPointData.status === 'online' ? 'success' : 'danger'" size="small">
                {{ currentPointData.status === 'online' ? '在线' : '离线' }}
              </el-tag>
            </div>
          </div>

          <el-table
            :data="trackData"
            height="300"
            size="small"
            stripe
            highlight-current-row
            :current-row-key="currentIndex"
            @current-change="handleCurrentChange"
          >
            <el-table-column prop="index" label="序号" width="60" />
            <el-table-column prop="gpsTime" label="时间" width="160">
              <template #default="{ row }">
                {{ formatDateTime(row.gpsTime) }}
              </template>
            </el-table-column>
            <el-table-column prop="speed" label="速度(km/h)" width="90" />
            <el-table-column prop="lng" label="经度" width="100">
              <template #default="{ row }">
                {{ row.lng.toFixed(4) }}
              </template>
            </el-table-column>
            <el-table-column prop="lat" label="纬度" width="100">
              <template #default="{ row }">
                {{ row.lat.toFixed(4) }}
              </template>
            </el-table-column>
            <el-table-column prop="status" label="状态" width="70">
              <template #default="{ row }">
                <el-tag :type="row.status === 'online' ? 'success' : 'danger'" size="small">
                  {{ row.status === 'online' ? '在线' : '离线' }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck - OpenLayers 类型定义不兼容
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { Search, Delete, VideoPlay, VideoPause, Close, FullScreen } from '@element-plus/icons-vue';
import Map from 'ol/Map';
import View from 'ol/View';
import TileLayer from 'ol/layer/Tile';
import XYZ from 'ol/source/XYZ';
import VectorLayer from 'ol/layer/Vector';
import VectorSource from 'ol/source/Vector';
import { fromLonLat } from 'ol/proj';
import { LineString, Point } from 'ol/geom';
import Feature from 'ol/Feature';
import { Style, Stroke, Circle, Fill } from 'ol/style';
import type { MapInstanceType } from '@/types/window';

// 地图实例类型
type MapInstance = MapInstanceType | unknown; // 使用定义的地图实例类型

interface TrackPoint {
  index: number;
  lng: number;
  lat: number;
  speed: number;
  direction: number;
  gpsTime: Date;
  status: string;
}

interface Vehicle {
  id: number;
  name: string;
  plateNumber: string;
}

const trackMapRef = ref<HTMLElement | null>(null);

const dateRange = ref<[Date, Date]>([new Date(Date.now() - 24 * 60 * 60 * 1000), new Date()]);

const selectedVehicle = ref<number | null>(null);
const vehicleList = ref<Vehicle[]>([]);
const trackData = ref<TrackPoint[]>([]);
const isLoading = ref(false);
const isPlaying = ref(false);
const currentIndex = ref(0);
const playbackSpeed = ref(1);
const currentPointData = computed(() => {
  if (trackData.value.length > 0 && currentIndex.value < trackData.value.length) {
    return trackData.value[currentIndex.value];
  }
  return null;
});

const totalMileage = computed(() => {
  if (trackData.value.length < 2) return '0.00';
  let total = 0;
  for (let i = 1; i < trackData.value.length; i++) {
    total += calculateDistance(
      trackData.value[i - 1].lat,
      trackData.value[i - 1].lng,
      trackData.value[i].lat,
      trackData.value[i].lng
    );
  }
  return total.toFixed(2);
});

const speedMarks = {
  1: '1x',
  3: '3x',
  5: '5x',
  7: '7x',
  10: '10x',
};

const canQuery = computed(() => {
  return (
    selectedVehicle.value !== null &&
    dateRange.value[0] &&
    dateRange.value[1] &&
    dateRange.value[0] < dateRange.value[1]
  );
});

// 地图相关实例
let olMap: Map | null = null;
let trackLine: Feature | null = null;
let animationTimer: number | null = null;
let mapInstance: MapInstance = null; // 用于高德、百度地图实例

onMounted(() => {
  initMap();
  loadVehicleList();
});

onUnmounted(() => {
  stopPlay();
  if (mapInstance) {
    mapInstance = null;
  }
});

const initMap = () => {
  if (!trackMapRef.value) return;

  trackMapRef.value.innerHTML = `
    <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;">
      <el-icon size="48"><MapLocation /></el-icon>
      <br><br>
      <strong>轨迹回放地图</strong>
      <br>
      <small>请选择车辆和时间范围进行查询</small>
    </div>
  `;

  // 初始化默认地图
  const mapType = localStorage.getItem('mapType') || 'tianditu';
  if (mapType === 'tianditu' && trackMapRef.value) {
    // 显示天地图加载提示
    trackMapRef.value.innerHTML = `
      <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;">
        <el-icon size="48"><Loading /></el-icon>
        <br><br>
        <strong>天地图加载中...</strong>
      </div>
    `;
  }
};

const loadVehicleList = () => {
  try {
    // 从localStorage获取真实的车辆数据，与首页保持一致
    const vehicleTreeDataStr = localStorage.getItem('vehicleTreeData');
    if (vehicleTreeDataStr) {
      const vehicleTreeData = JSON.parse(vehicleTreeDataStr);
      const allVehicles: Vehicle[] = [];

      // 递归遍历车辆树，收集所有车辆
      interface TreeNode {
        type: string;
        id?: number;
        vehicle_name?: string;
        license_plate?: string;
        children?: TreeNode[];
      }

      const collectVehicles = (nodes: TreeNode[]) => {
        for (const node of nodes) {
          if (node.type === 'vehicle') {
            allVehicles.push({
              id: node.id!,
              name: node.vehicle_name!,
              plateNumber: node.license_plate!,
            });
          } else if (node.children && node.children.length > 0) {
            collectVehicles(node.children);
          }
        }
      };

      collectVehicles(vehicleTreeData);

      if (allVehicles.length > 0) {
        vehicleList.value = allVehicles;
        console.log('从localStorage获取的车辆列表:', allVehicles);
        return;
      }
    }

    // 尝试从allVehicles获取
    const allVehiclesStr = localStorage.getItem('allVehicles');
    if (allVehiclesStr) {
      const allVehicles = JSON.parse(allVehiclesStr);
      if (Array.isArray(allVehicles) && allVehicles.length > 0) {
        vehicleList.value = allVehicles.map((v: { id: number; vehicle_name: string; license_plate: string }) => ({
          id: v.id,
          name: v.vehicle_name,
          plateNumber: v.license_plate,
        }));
        console.log('从allVehicles获取的车辆列表:', vehicleList.value);
        return;
      }
    }

    // 都获取失败时，使用默认数据
    vehicleList.value = [
      { id: 111, name: '京A12345', plateNumber: '京A12345' },
      { id: 112, name: '京A12346', plateNumber: '京A12346' },
      { id: 121, name: '京A12347', plateNumber: '京A12347' },
      { id: 211, name: '京B67890', plateNumber: '京B67890' },
    ];
    console.log('使用默认车辆列表:', vehicleList.value);
  } catch (error) {
    console.error('加载车辆列表失败:', error);
    // 异常情况下使用默认数据
    vehicleList.value = [
      { id: 111, name: '京A12345', plateNumber: '京A12345' },
      { id: 112, name: '京A12346', plateNumber: '京A12346' },
      { id: 121, name: '京A12347', plateNumber: '京A12347' },
      { id: 211, name: '京B67890', plateNumber: '京B67890' },
    ];
  }
};

const queryTrack = async () => {
  if (!canQuery.value) {
    ElMessage.warning('请选择车辆和时间范围');
    return;
  }

  isLoading.value = true;
  stopPlay();
  clearTrack();

  try {
    // 1. 尝试从localStorage获取车辆模拟器生成的轨迹数据
    const localStorageKey = `vehicleTracks_${selectedVehicle.value}`;
    const savedTracksStr = localStorage.getItem(localStorageKey);

    if (savedTracksStr) {
      const savedTracks = JSON.parse(savedTracksStr);
      if (Array.isArray(savedTracks) && savedTracks.length > 0) {
        console.log('从localStorage获取的轨迹数据:', savedTracks);

        // 过滤时间范围内的轨迹点
        const startTime = dateRange.value[0].getTime();
        const endTime = dateRange.value[1].getTime();

        const filteredTracks = savedTracks.filter((track: { time: string; lng: number; lat: number; speed: number }) => {
          const trackTime = new Date(track.time).getTime();
          return trackTime >= startTime && trackTime <= endTime;
        });

        if (filteredTracks.length > 0) {
          // 转换为前端需要的格式
          const points: TrackPoint[] = filteredTracks.map((track: { time: string; lng: number; lat: number; speed: number }, index: number) => ({
            index: index + 1,
            lng: track.lng,
            lat: track.lat,
            speed: Math.round(track.speed),
            direction: Math.round(Math.random() * 360), // 模拟器没有保存方向，随机生成
            gpsTime: new Date(track.time),
            status: track.speed > 5 ? 'online' : 'offline',
          }));

          trackData.value = points;

          if (points.length > 0) {
            initTrackMap();
            ElMessage.success(`查询成功，共找到 ${points.length} 个轨迹点`);
            isLoading.value = false;
            return;
          }
        }
      }
    }

    // 2. 如果localStorage没有数据，调用后端API获取轨迹数据
    const start_time = dateRange.value[0].toISOString();
    const end_time = dateRange.value[1].toISOString();
    const vehicle_id = selectedVehicle.value;

    try {
      const response = await window.fetch(
        `/api/tracks?vehicle_id=${vehicle_id}&start_time=${start_time}&end_time=${end_time}`,
        {
          headers: {
            Authorization: `Bearer ${localStorage.getItem('token') || ''}`,
          },
        }
      );

      if (response.ok) {
        const result = await response.json();

        if (result.success && result.data) {
          const points: TrackPoint[] = result.data.map((item: { longitude: string; latitude: string; track_time: string; status: number }, index: number) => ({
            index: index + 1,
            lng: parseFloat(item.longitude),
            lat: parseFloat(item.latitude),
            speed: 0,
            direction: 0,
            gpsTime: new Date(item.track_time),
            status: item.status === 1 ? 'online' : 'offline',
          }));

          trackData.value = points;

          if (points.length > 0) {
            initTrackMap();
            ElMessage.success(`查询成功，共找到 ${points.length} 个轨迹点`);
            isLoading.value = false;
            return;
          }
        }
      }
    } catch (apiError) {
      console.error('API查询失败，使用模拟数据:', apiError);
    }

    // 如果没有数据，显示提示
    ElMessage.warning('未找到轨迹数据');
  } catch (error) {
    console.error('查询轨迹失败:', error);
    ElMessage.error('查询轨迹失败');
  } finally {
    isLoading.value = false;
  }
};

const initTrackMap = () => {
  if (!trackMapRef.value || trackData.value.length === 0) return;

  // 确保地图容器有高度
  if (trackMapRef.value) {
    trackMapRef.value.style.height = '400px'; // 设置固定高度
  }

  const points = trackData.value.map((p) => [p.lng, p.lat]);

  // 只清空一次地图容器
  trackMapRef.value.innerHTML = '<div id="track-map-container" style="width: 100%; height: 100%;"></div>';

  // 从localStorage获取地图类型配置，默认使用天地图
  const mapType = localStorage.getItem('mapType') || 'tianditu';

  console.log('初始化轨迹地图，地图类型:', mapType);

  if (mapType === 'tianditu') {
    // 天地图实现 - 使用OpenLayers
    try {
      // 直接在当前容器创建OpenLayers地图，不再清空容器
      const olContainer = trackMapRef.value;
      if (!olContainer) return;

      // 天地图密钥
      const tiandituKey = '34d8cf060f7e8ac09be79b9261d65274'; // 浏览器端密钥

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

      // 计算地图中心点
      const centerIndex = Math.floor(points.length / 2);
      const centerPoint = fromLonLat([points[centerIndex][0], points[centerIndex][1]]);

      // 创建地图实例
      olMap = new Map({
        target: olContainer,
        layers: [vectorLayer, labelLayer],
        view: new View({
          center: centerPoint,
          zoom: 12,
          maxZoom: 18,
          minZoom: 2,
        }),
      });

      // 地图加载完成后绘制轨迹
      olMap.on('rendercomplete', () => {
        drawTrackWithOpenLayers();
      });
    } catch (e) {
      console.error('天地图初始化失败:', e);
      // 显示简单地图提示
      trackMapRef.value.innerHTML = `
        <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;">
          <el-icon size="48"><MapLocation /></el-icon>
          <br><br>
          <strong>轨迹地图</strong>
          <br>
          <small>天地图初始化失败，请切换其他地图类型</small>
        </div>
      `;
    }
  } else if (mapType === 'gaode' && window.AMap) {
    try {
      // 直接在当前容器创建高德地图
      const container = trackMapRef.value;
      if (!container) return;

      const AMap = window.AMap;
      mapInstance = new (AMap.Map)(container, {
        zoom: 12,
        center: points[Math.floor(points.length / 2)],
      });

      const mapInstanceAny = mapInstance as MapInstanceType;
      mapInstanceAny.on('complete', () => {
        drawTrack();
      });
    } catch (e) {
      console.error('高德地图初始化失败:', e);
      // 显示简单地图提示
      trackMapRef.value.innerHTML = `
        <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;">
          <el-icon size="48"><MapLocation /></el-icon>
          <br><br>
          <strong>轨迹地图</strong>
          <br>
          <small>高德地图初始化失败，请切换其他地图类型</small>
        </div>
      `;
    }
  } else if (mapType === 'baidu' && window.BMap) {
    try {
      // 直接在当前容器创建百度地图
      const container = trackMapRef.value;
      if (!container) return;

      const BMap = window.BMap;
      mapInstance = new (BMap.Map)(container);
      const centerPoint = new (BMap.Point)(
        points[Math.floor(points.length / 2)][0],
        points[Math.floor(points.length / 2)][1]
      );
      const mapInstanceAny = mapInstance as MapInstanceType;
      mapInstanceAny.centerAndZoom(centerPoint, 12);
      drawTrack();
    } catch (e) {
      console.error('百度地图初始化失败:', e);
      // 显示简单地图提示
      trackMapRef.value.innerHTML = `
        <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;">
          <el-icon size="48"><MapLocation /></el-icon>
          <br><br>
          <strong>轨迹地图</strong>
          <br>
          <small>百度地图初始化失败，请切换其他地图类型</small>
        </div>
      `;
    }
  } else {
    // 默认使用简单地图提示
    trackMapRef.value.innerHTML = `
      <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;">
        <el-icon size="48"><MapLocation /></el-icon>
        <br><br>
        <strong>轨迹地图</strong>
        <br>
        <small>请选择天地图、高德地图或百度地图</small>
      </div>
    `;
  }
};

const drawTrackWithOpenLayers = () => {
  if (!olMap || trackData.value.length === 0) return;

  // 创建轨迹点数组
  const trackPoints = trackData.value.map((p) => [p.lng, p.lat]);

  // 创建矢量源
  const vectorSource = new VectorSource();

  // 创建轨迹线 - 使用 fromLonLat 转换坐标
  const linePoints = trackPoints.map((point) => fromLonLat([point[0], point[1]]));
  const lineString = new LineString(linePoints);
  const lineFeature = new Feature({
    geometry: lineString,
  });

  // 设置轨迹线样式
  lineFeature.setStyle(
    new Style({
      stroke: new Stroke({
        color: '#409eff',
        width: 3,
      }),
    })
  );

  vectorSource.addFeature(lineFeature);

  // 创建轨迹图层
  const trackLayer = new VectorLayer({
    source: vectorSource,
    zIndex: 1,
  });

  olMap.addLayer(trackLayer);

  // 添加所有轨迹点标记
  const markerSource = new VectorSource();

  // 遍历所有轨迹点，添加标记
  trackData.value.forEach((point, index) => {
    // 使用 fromLonLat 转换坐标
    const markerGeometry = new Point(fromLonLat([point.lng, point.lat]));
    const markerPoint = new Feature({
      geometry: markerGeometry,
      properties: {
        index: index,
        speed: point.speed,
        time: point.gpsTime,
      },
    });

    // 根据是否是起点/终点设置不同样式
    let markerStyle;
    if (index === 0) {
      // 起点样式
      markerStyle = new Style({
        image: new Circle({
          radius: 6,
          fill: new Fill({ color: '#67c23a' }),
          stroke: new Stroke({ color: '#fff', width: 2 }),
        }),
      });
    } else if (index === trackData.value.length - 1) {
      // 终点样式
      markerStyle = new Style({
        image: new Circle({
          radius: 6,
          fill: new Fill({ color: '#f56c6c' }),
          stroke: new Stroke({ color: '#fff', width: 2 }),
        }),
      });
    } else {
      // 中间点样式
      markerStyle = new Style({
        image: new Circle({
          radius: 4,
          fill: new Fill({ color: '#409eff' }),
          stroke: new Stroke({ color: '#fff', width: 1 }),
        }),
      });
    }

    markerPoint.setStyle(markerStyle);
    markerSource.addFeature(markerPoint);
  });

  const markerLayer = new VectorLayer({
    source: markerSource,
    zIndex: 2,
  });

  olMap.addLayer(markerLayer);

  // 调整地图视图以适应轨迹
  const extent = trackLayer.getSource()?.getExtent();
  if (extent) {
    olMap.getView().fit(extent, {
      padding: [50, 50, 50, 50],
      duration: 1000,
    });
  }
};

const drawTrack = () => {
  if (!mapInstance || trackData.value.length === 0) return;

  const AMap = window.AMap;
  const BMap = window.BMap;
  const mapInstanceAny = mapInstance as MapInstanceType;

  const points = trackData.value.map((p) => {
    if (AMap) {
      return new (AMap.LngLat)(p.lng, p.lat);
    } else if (BMap) {
      return new (BMap.Point)(p.lng, p.lat);
    }
    return [p.lng, p.lat];
  });

  try {
    if (AMap) {
      trackLine = new (AMap.Polyline)({
        path: points,
        strokeColor: '#409eff',
        strokeWeight: 3,
        strokeOpacity: 0.8,
        strokeStyle: 'solid',
      });
      mapInstanceAny.add(trackLine);

      // 添加所有轨迹点标记
      trackData.value.forEach((point, index) => {
        let markerIcon;
        let markerSize = new (AMap.Size)(10, 10);

        if (index === 0) {
          // 起点
          markerIcon = new (AMap.Icon)({
            size: new (AMap.Size)(20, 20),
            image: 'https://webapi.amap.com/theme/v1.3/markers/n/mark_b.png',
            imageSize: new (AMap.Size)(20, 20),
          });
        } else if (index === trackData.value.length - 1) {
          // 终点
          markerIcon = new (AMap.Icon)({
            size: new (AMap.Size)(20, 20),
            image: 'https://webapi.amap.com/theme/v1.3/markers/n/mark_e.png',
            imageSize: new (AMap.Size)(20, 20),
          });
        } else {
          // 中间点 - 使用默认蓝色标记
          markerIcon = new (AMap.Icon)({
            size: markerSize,
            image: 'https://webapi.amap.com/theme/v1.3/markers/n/mark_r.png',
            imageSize: markerSize,
          });
        }

        const marker = new (AMap.Marker)({
          position: new (AMap.LngLat)(point.lng, point.lat),
          icon: markerIcon,
          title: `速度: ${point.speed}km/h\n时间: ${point.gpsTime.toLocaleString()}`,
        });
        mapInstanceAny.add(marker);
      });

      mapInstanceAny.setFitView(trackLine);
    } else if (BMap) {
      trackLine = new (BMap.Polyline)(points, {
        strokeColor: '#409eff',
        strokeWeight: 3,
        strokeOpacity: 0.8,
      });
      mapInstanceAny.addOverlay(trackLine);

      // 添加所有轨迹点标记
      trackData.value.forEach((point, _index) => {
        const mapPoint = new (BMap.Point)(point.lng, point.lat);
        const marker = new (BMap.Marker)(mapPoint);

        // 添加信息窗口
        const infoWindow = new (BMap.InfoWindow)(`<div style='font-size:12px;padding:5px;'>
          <div>速度: ${point.speed}km/h</div>
          <div>时间: ${point.gpsTime.toLocaleString()}</div>
        </div>`);

        marker.addEventListener('mouseover', function () {
          mapInstanceAny.openInfoWindow(infoWindow, mapPoint);
        });

        marker.addEventListener('mouseout', function () {
          mapInstanceAny.closeInfoWindow();
        });

        mapInstanceAny.addOverlay(marker);
      });

      mapInstanceAny.setViewport(points);
    }
  } catch (e) {
    console.error('绘制轨迹失败:', e);
  }
};

const togglePlay = () => {
  if (isPlaying.value) {
    pausePlay();
  } else {
    startPlay();
  }
};

const startPlay = () => {
  if (trackData.value.length === 0) return;

  isPlaying.value = true;

  const AMap = window.AMap;
  const BMap = window.BMap;
  const mapInstanceAny = mapInstance as MapInstanceType;

  const playStep = () => {
    if (!isPlaying.value || currentIndex.value >= trackData.value.length) {
      stopPlay();
      return;
    }

    currentIndex.value++;

    if (currentIndex.value < trackData.value.length) {
      const point = trackData.value[currentIndex.value];

      try {
        if (AMap && mapInstanceAny) {
          mapInstanceAny.setCenter([point.lng, point.lat]);
        } else if (BMap && mapInstanceAny) {
          mapInstanceAny.setCenter(new (BMap.Point)(point.lng, point.lat));
        }
      } catch {
        console.error('地图中心设置失败');
      }

      const interval = Math.max(100, 1000 / playbackSpeed.value);
      animationTimer = window.setTimeout(playStep, interval) as unknown as number;
    } else {
      stopPlay();
      ElMessage.info('轨迹播放完成');
    }
  };

  animationTimer = window.setTimeout(playStep, 500) as unknown as number;
};

const pausePlay = () => {
  isPlaying.value = false;
  if (animationTimer) {
    clearTimeout(animationTimer);
    animationTimer = null;
  }
};

const stopPlay = () => {
  isPlaying.value = false;
  currentIndex.value = 0;
  if (animationTimer) {
    clearTimeout(animationTimer);
    animationTimer = null;
  }
};

const clearTrack = () => {
  stopPlay();
  trackData.value = [];
  currentIndex.value = 0;
  trackLine = null;

  if (mapInstance) {
    mapInstance = null;
  }

  initMap();
};

const handleCurrentChange = (row: TrackPoint | null) => {
  if (row) {
    currentIndex.value = trackData.value.findIndex((p) => p.index === row.index);

    const AMap = window.AMap;
    const BMap = window.BMap;
    const mapInstanceAny = mapInstance as MapInstanceType;

    try {
      if (AMap && mapInstanceAny) {
        mapInstanceAny.setCenter([row.lng, row.lat]);
      } else if (BMap && mapInstanceAny) {
        mapInstanceAny.setCenter(new (BMap.Point)(row.lng, row.lat));
      }
    } catch {
      console.error('地图中心设置失败');
    }
  }
};

const formatDateTime = (date: Date) => {
  return new Date(date).toLocaleString('zh-CN');
};

const getDirection = (deg: number) => {
  const directions = ['北', '东北', '东', '东南', '南', '西南', '西', '西北'];
  const index = Math.round(deg / 45) % 8;
  return directions[index] || '北';
};

const calculateDistance = (lat1: number, lng1: number, lat2: number, lng2: number) => {
  const R = 6371;
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLng = ((lng2 - lng1) * Math.PI) / 180;
  const a =
    Math.sin(dLat / 2) * Math.sin(dLat / 2) +
    Math.cos((lat1 * Math.PI) / 180) * Math.cos((lat2 * Math.PI) / 180) * Math.sin(dLng / 2) * Math.sin(dLng / 2);
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
  return R * c;
};

// 运动轨迹切换
const toggleMotionTrack = () => {
  ElMessage.info('切换运动轨迹模式');
};

// 点轨迹切换
const togglePointTrack = () => {
  ElMessage.info('切换点轨迹模式');
};

// 显示全部轨迹
const showAllTrack = () => {
  ElMessage.info('显示全部轨迹');
};

// 切换显示轨迹地址
const toggleShowAddress = () => {
  ElMessage.info('切换显示轨迹地址');
};

// 查询停车记录
const queryStopRecords = () => {
  ElMessage.info('查询停车记录');
};

// 导出数据
const exportData = () => {
  ElMessage.info('导出轨迹数据');
};

// 全屏切换
const toggleFullScreen = () => {
  const trackPlaybackElement = document.querySelector('.track-playback') as HTMLElement;
  if (!trackPlaybackElement) return;

  if (!document.fullscreenElement) {
    // 进入全屏
    if (trackPlaybackElement.requestFullscreen) {
      trackPlaybackElement.requestFullscreen().then(() => {
        handleFullScreenChange();
      });
    } else if ((trackPlaybackElement as unknown as { webkitRequestFullscreen: () => void }).webkitRequestFullscreen) {
      (trackPlaybackElement as unknown as { webkitRequestFullscreen: () => void }).webkitRequestFullscreen();
      handleFullScreenChange();
    } else if ((trackPlaybackElement as unknown as { msRequestFullscreen: () => void }).msRequestFullscreen) {
      (trackPlaybackElement as unknown as { msRequestFullscreen: () => void }).msRequestFullscreen();
      handleFullScreenChange();
    } else if ((trackPlaybackElement as unknown as { mozRequestFullScreen: () => void }).mozRequestFullScreen) {
      (trackPlaybackElement as unknown as { mozRequestFullScreen: () => void }).mozRequestFullScreen();
      handleFullScreenChange();
    }
  } else {
    // 退出全屏
    if (document.exitFullscreen) {
      document.exitFullscreen().then(() => {
        handleFullScreenChange();
      });
    } else if ((document as unknown as { webkitExitFullscreen: () => void }).webkitExitFullscreen) {
      (document as unknown as { webkitExitFullscreen: () => void }).webkitExitFullscreen();
      handleFullScreenChange();
    } else if ((document as unknown as { msExitFullscreen: () => void }).msExitFullscreen) {
      (document as unknown as { msExitFullscreen: () => void }).msExitFullscreen();
      handleFullScreenChange();
    } else if ((document as unknown as { mozCancelFullScreen: () => void }).mozCancelFullScreen) {
      (document as unknown as { mozCancelFullScreen: () => void }).mozCancelFullScreen();
      handleFullScreenChange();
    }
  }
};

// 处理全屏状态变化
const handleFullScreenChange = () => {
  // 确保地图容器有正确的高度
  const trackMapElement = document.querySelector('.track-map') as HTMLElement;
  if (trackMapElement) {
    // 全屏状态下设置高度为100%
    if (document.fullscreenElement) {
      // 全屏时使用视口高度
      trackMapElement.style.height = '100vh';
      // 确保宽度100%
      trackMapElement.style.width = '100%';
    } else {
      // 非全屏时恢复固定高度
      trackMapElement.style.height = '400px';
      // 恢复自动宽度
      trackMapElement.style.width = 'auto';
    }
  }

  // 更新地图尺寸
  if (olMap) {
    // 延迟更新，确保DOM尺寸已稳定
    setTimeout(() => {
      olMap.updateSize();
      console.log('OpenLayers地图尺寸已更新');
    }, 100);
  }

  // 对于高德地图
  if (window.AMap && mapInstance) {
    setTimeout(() => {
      (mapInstance as unknown as { resize: () => void }).resize();
      console.log('高德地图尺寸已更新');
    }, 100);
  }

  // 对于百度地图
  if (window.BMap && mapInstance) {
    setTimeout(() => {
      (mapInstance as unknown as { resize: () => void }).resize();
      console.log('百度地图尺寸已更新');
    }, 100);
  }
};

watch(dateRange, () => {
  if (dateRange.value[0] >= dateRange.value[1]) {
    ElMessage.warning('开始时间必须小于结束时间');
  }
});
</script>

<style scoped>
.track-playback {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #f5f7fa;
}

.track-toolbar {
  padding: 10px;
}

.toolbar-card {
  margin-bottom: 0;
}

.toolbar-row {
  display: flex;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}

.toolbar-section {
  display: flex;
  align-items: center;
  gap: 10px;
}

.section-title {
  font-weight: bold;
  color: #606266;
  white-space: nowrap;
  font-size: 14px;
}

.playback-controls {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid #ebeef5;
}

.playback-info {
  margin-left: 10px;
  font-size: 14px;
  color: #409eff;
}

.current-point {
  font-size: 18px;
  font-weight: bold;
}

.separator {
  margin: 0 5px;
  color: #c0c4cc;
}

.total-points {
  color: #909399;
}

.speed-control {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-left: 20px;
  width: 150px;
}

.speed-label {
  font-size: 12px;
  color: #909399;
}

.basic-functions {
  margin-left: 20px;
}

.basic-functions .el-button {
  font-size: 12px;
  padding: 4px 8px;
}

.basic-functions .el-button-group {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.track-main {
  flex: 1;
  display: flex;
  gap: 10px;
  padding: 0 10px 10px 10px;
  min-height: 0;
}

.track-map {
  flex: 1;
  border-radius: 4px;
  overflow: hidden;
  background: #fff;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  min-height: 300px; /* 确保地图容器有最小高度 */
  height: 400px; /* 设置固定高度 */
}

.track-info-panel {
  width: 500px;
  display: flex;
  flex-direction: column;
}

.info-card {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.current-info {
  background: #f5f7fa;
  padding: 10px;
  border-radius: 4px;
  margin-bottom: 10px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  padding: 5px 0;
  border-bottom: 1px solid #ebeef5;
}

.info-row:last-child {
  border-bottom: none;
}

.info-label {
  color: #909399;
  font-size: 13px;
}

.info-value {
  color: #303133;
  font-weight: 500;
  font-size: 13px;
}
</style>


