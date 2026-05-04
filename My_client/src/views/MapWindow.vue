<template>
  <div class="map-window">
    <!-- 顶部标题栏 -->
    <div class="top-header">
      <h1>{{ settingsStore.homePageName }}</h1>
      <div class="header-actions">
        <el-button type="primary" size="small" @click="handleRefresh" :loading="isRefreshing">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="main-content">
      <!-- 左侧车辆列表（企业-车队-车辆层级） -->
      <VehicleSidebar
        :vehicle-tree-data="vehicleTreeData"
        :vehicle-count="vehicleCount"
        @node-click="handleNodeClick"
      />

      <!-- 调整手柄 -->
      <div class="resize-handle" ref="resizeHandle" @mousedown="startResizeSidebar"></div>

      <!-- 右侧地图区域 -->
      <div class="map-container" ref="mapContainer">
        <!-- 地图类型切换 -->
        <MapToolbar
          v-model="mapType"
          @change="handleMapTypeChange"
        />

        <!-- 地图容器（单容器，由 initializeMap 根据 mapType 渲染不同底图） -->
        <div class="map-wrapper">
          <div class="map" ref="mapRef"></div>
          <!-- 车辆标记 -->
          <MapMarkers
            :map-vehicles="mapVehiclesDisplay"
            :selected-vehicle="selectedVehicle"
            @select="selectVehicle"
          />
        </div>

        <!-- 底部数据工具栏 -->
        <div class="data-toolbar">
          <!-- 工具栏标题 -->
          <div class="toolbar-header">
            <div class="toolbar-title">数据工具栏</div>
            <div class="toolbar-stats">
              <span>总数: {{ vehicleCount }}</span>
              <span>在线: {{ onlineCount }}</span>
              <span>离线: {{ offlineCount }}</span>
              <span>报警: {{ alarmCount }}</span>
            </div>
          </div>

          <!-- 工具栏内容 -->
          <div class="toolbar-content">
            <!-- 数据表格 -->
            <div class="data-table">
              <!-- 实时信息表格 -->
              <div v-if="activeTab === 'realtime'" class="active-tab-realtime">
                <div class="table-header">
                  <div class="header-item" @mousedown="startResize(0, $event)">序号</div>
                  <div class="header-item" @mousedown="startResize(1, $event)">车辆号码</div>
                  <div class="header-item" @mousedown="startResize(2, $event)">时间</div>
                  <div class="header-item" @mousedown="startResize(3, $event)">速度方向</div>
                  <div class="header-item" @mousedown="startResize(4, $event)">定位状态</div>
                  <div class="header-item" @mousedown="startResize(5, $event)">里程(公里)</div>
                  <div class="header-item" @mousedown="startResize(6, $event)">总重(公斤)</div>
                  <div class="header-item" @mousedown="startResize(7, $event)">载重(公斤)</div>
                  <div class="header-item" @mousedown="startResize(8, $event)">报警</div>
                </div>
                <div class="table-body">
                  <div v-for="(vehicle, index) in mapVehicles" :key="vehicle.id" class="table-row">
                    <div class="row-item">{{ index + 1 }}</div>
                    <div class="row-item">{{ vehicle.license_plate }}</div>
                    <div class="row-item">{{ new Date().toLocaleString() }}</div>
                    <div class="row-item">速度: {{ vehicle.speed }}km/h 方向: {{ vehicle.direction }}</div>
                    <div class="row-item">定位</div>
                    <div class="row-item">{{ vehicle.mileage }}</div>
                    <div class="row-item">{{ vehicle.totalWeight }}</div>
                    <div class="row-item">{{ vehicle.loadWeight }}</div>
                    <div class="row-item" :class="{ alarm: vehicle.hasAlarm }">{{ vehicle.hasAlarm ? '1' : '0' }}</div>
                  </div>
                </div>
              </div>

              <!-- 车辆报警表格 -->
              <div v-else-if="activeTab === 'alarm'" class="active-tab-alarm">
                <div class="table-header">
                  <div class="header-item" @mousedown="startResize(0, $event)">序号</div>
                  <div class="header-item" @mousedown="startResize(1, $event)">车牌</div>
                  <div class="header-item" @mousedown="startResize(2, $event)">报警时间</div>
                  <div class="header-item" @mousedown="startResize(3, $event)">报警类型</div>
                  <div class="header-item" @mousedown="startResize(4, $event)">报警内容</div>
                  <div class="header-item" @mousedown="startResize(5, $event)">处理状态</div>
                </div>
                <div class="table-body">
                  <div v-if="mapVehicles.filter((v) => v.hasAlarm).length === 0" class="table-row">
                    <div class="row-item">没有车辆报警</div>
                  </div>
                  <div
                    v-else
                    v-for="(vehicle, index) in mapVehicles.filter((v) => v.hasAlarm)"
                    :key="vehicle.id"
                    class="table-row"
                  >
                    <div class="row-item">{{ index + 1 }}</div>
                    <div class="row-item">{{ vehicle.license_plate }}</div>
                    <div class="row-item">{{ new Date().toLocaleString() }}</div>
                    <div class="row-item">超载报警</div>
                    <div class="row-item">载重 {{ vehicle.loadWeight }} 超过限定值</div>
                    <div class="row-item">未处理</div>
                  </div>
                </div>
              </div>

              <!-- 交互内容表格 -->
              <div v-else-if="activeTab === 'interaction'" class="active-tab-interaction">
                <div class="table-header">
                  <div class="header-item" @mousedown="startResize(0, $event)">序号</div>
                  <div class="header-item" @mousedown="startResize(1, $event)">车牌</div>
                  <div class="header-item" @mousedown="startResize(2, $event)">感知终端</div>
                  <div class="header-item" @mousedown="startResize(3, $event)">接收/发送时间</div>
                  <div class="header-item" @mousedown="startResize(4, $event)">指令</div>
                  <div class="header-item" @mousedown="startResize(5, $event)">消息内容</div>
                </div>
                <div class="table-body">
                  <div v-for="(vehicle, index) in mapVehicles" :key="vehicle.id" class="table-row">
                    <div class="row-item">{{ index + 1 }}</div>
                    <div class="row-item">{{ vehicle.license_plate }}</div>
                    <div class="row-item">
                      终端: TERM{{ vehicle.id }}, 传感器1: SENS{{ vehicle.id }}-1, 传感器2: SENS{{ vehicle.id }}-2
                    </div>
                    <div class="row-item">{{ new Date().toLocaleString() }}</div>
                    <div class="row-item">位置查询</div>
                    <div class="row-item">
                      实时位置: {{ vehicle.location }}, 速度: {{ vehicle.speed }}km/h, 载重: {{ vehicle.loadWeight }}kg
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 标签页 -->
            <div class="toolbar-tabs">
              <el-tabs v-model="activeTab" size="small" @tab-change="handleTabChange">
                <el-tab-pane label="实时信息" name="realtime"></el-tab-pane>
                <el-tab-pane label="车辆报警" name="alarm"></el-tab-pane>
                <el-tab-pane label="交互内容" name="interaction"></el-tab-pane>
              </el-tabs>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 选中车辆详情面板 -->
    <VehicleInfoPanel
      v-if="selectedVehicle"
      :selected-vehicle="selectedVehicleDisplay"
      @close="selectedVehicle = null"
    />

    <!-- 底部信息栏 -->
    <div class="bottom-info-bar">
      <div class="info-item">
        <span class="info-label">日期：</span>
        <span class="info-value">{{ currentDate }}</span>
      </div>
      <div class="info-item">
        <span class="info-label">服务器IP：</span>
        <span class="info-value">{{ serverIp }}:{{ serverPort }}</span>
      </div>
      <div class="info-item">
        <span class="info-label">在线车辆：</span>
        <span class="info-value online">{{ onlineCount }}</span>
      </div>
      <div class="info-item">
        <span class="info-label">离线车辆：</span>
        <span class="info-value offline">{{ offlineCount }}</span>
      </div>
      <div class="info-item">
        <span class="info-label">今日已处理报警：</span>
        <span class="info-value processed">{{ processedAlarmCount }}</span>
      </div>
      <div class="info-item">
        <span class="info-label">未处理报警：</span>
        <span class="info-value unprocessed">{{ unprocessedAlarmCount }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, computed, onMounted, onUnmounted, defineAsyncComponent, nextTick } from 'vue';
import { ElMessage, ElScrollbar } from 'element-plus';
// 图标懒加载
const Van = defineAsyncComponent(() => import('@element-plus/icons-vue').then((m) => ({ default: m.Van })));
const Refresh = defineAsyncComponent(() => import('@element-plus/icons-vue').then((m) => ({ default: m.Refresh })));
import { useSettingsStore } from '@/stores/settings';
import api from '@/api';
import { formatZoomLevel } from '@/utils/mapParser';

// 异步组件导入
const MapToolbar = defineAsyncComponent(() => import('./components/map/MapToolbar.vue'));
const VehicleSidebar = defineAsyncComponent(() => import('./components/map/VehicleSidebar.vue'));
const MapMarkers = defineAsyncComponent(() => import('./components/map/MapMarkers.vue'));
const VehicleInfoPanel = defineAsyncComponent(() => import('./components/map/VehicleInfoPanel.vue'));
// MapLegend 组件已隐藏（用户不再需要显示地图图层面板）
// const MapLegend = defineAsyncComponent(() => import('./components/map/MapLegend.vue'));

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
import { getTiandituVecUrl, getTiandituCvaUrl, getTiandituKey } from '@/utils/mapConfig';

// 防抖函数
function debounce<T extends (...args: unknown[]) => void>(func: T, wait: number): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;
  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(() => {
      func(...args);
    }, wait);
  };
}

// 节流函数
function throttle<T extends (...args: unknown[]) => void>(func: T, limit: number): (...args: Parameters<T>) => void {
  let inThrottle = false;
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args);
      inThrottle = true;
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

const settingsStore = useSettingsStore();

// 地图实例
let olMap: Map | null = null;
let vehicleLayer: VectorLayer | null = null;
let dateInterval: ReturnType<typeof setInterval> | null = null;
let resizeObserver: any | null = null;

// 通用鼠标事件类型
type MouseEvent = globalThis.MouseEvent;



interface MapLayerInfo {
  id: number;
  file: string;
  visible: boolean;
  minZoom: number;
  maxZoom: number;
  label?: string;
}

interface GeoSetInfo {
  name: string;
  projection: string;
  center: string;
  zoomLevel: number;
  layers: MapLayerInfo[];
}

const isRefreshing = ref(false);
const mapRef = ref<HTMLElement>();
const localMapRef = ref<HTMLElement>();
const activeTab = ref('realtime');
const mapType = ref('tianditu');
const tiandituKey = ref('');
const gaodeKey = ref('');
const localMapInfo = ref<GeoSetInfo | null>(null);

// 列宽调整相关变量
const resizingColumn = ref<number | null>(null);
const startX = ref(0);
const startWidth = ref(0);

// 侧边栏宽度调整相关变量
const mapContainer = ref<HTMLElement>();
const resizeHandle = ref<HTMLElement>();
const isResizingSidebar = ref(false);
const sidebarStartWidth = ref(0);
const mouseStartX = ref(0);

// 后端车辆数据类型
interface BackendVehicle {
  vehicle_id: number;
  license_plate: string;
  status: number;
  alarm?: boolean;
  speed?: string;
  direction?: string;
  mileage?: string;
  load_weight?: string;
  total_weight?: string;
  location?: string;
}

// 地图车辆数据类型
interface MapVehicle {
  id: number;
  license_plate: string;
  status: 'online' | 'offline';
  alarm: boolean;
  speed: string;
  direction: string;
  mileage: string;
  loadWeight: string;
  totalWeight: string;
  location: string;
  mapX: number;
  mapY: number;
}

interface VehicleNode {
  id: string;
  name: string;
  type: 'company' | 'group' | 'vehicle';
  status?: 'online' | 'offline';
  hasAlarm?: boolean;
  speed?: string;
  direction?: string;
  mileage?: string;
  loadWeight?: string;
  totalWeight?: string;
  location?: string;
  x?: number;
  y?: number;
  children?: VehicleNode[];
}

const vehicleTreeData = ref<VehicleNode[]>([]);
const mapVehicles = ref<MapVehicle[]>([]);
const selectedVehicle = ref<MapVehicle | null>(null);

// ========== P2-3: 地图标记分片渲染优化 ==========
// 分片渲染配置
const MARKER_CHUNK_SIZE = 100;  // 每批渲染标记数
const RENDER_INTERVAL_MS = 16;  // 约60fps
const MAX_VISIBLE_MARKERS = 500; // 最大可见标记数（超出后聚合显示）

// 分片渲染状态
let markerRenderIndex = 0;
let markerRenderTimer: number | null = null;
let allMapVehicles: MapVehicle[] = [];  // 缓存所有车辆数据

// 分片渲染函数 - 避免一次性渲染大量 DOM
function startChunkedMarkerRender(vehicles: MapVehicle[]) {
  // 停止之前的渲染
  if (markerRenderTimer !== null) {
    cancelAnimationFrame(markerRenderTimer);
    markerRenderTimer = null;
  }
  
  allMapVehicles = vehicles;
  markerRenderIndex = 0;
  
  if (vehicles.length <= MARKER_CHUNK_SIZE) {
    // 数量较少，直接渲染
    mapVehicles.value = vehicles;
    return;
  }
  
  // 数量较多，使用分片渲染
  const renderChunk = () => {
    const endIndex = Math.min(markerRenderIndex + MARKER_CHUNK_SIZE, vehicles.length);
    const currentChunk = allMapVehicles.slice(0, endIndex);
    mapVehicles.value = currentChunk;
    
    markerRenderIndex = endIndex;
    
    if (markerRenderIndex < vehicles.length) {
      markerRenderTimer = requestAnimationFrame(renderChunk);
    } else {
      markerRenderTimer = null;
      console.log(`[MapWindow] 分片渲染完成: ${vehicles.length} 个标记`);
    }
  };
  
  renderChunk();
}

// 停止分片渲染
function stopChunkedMarkerRender() {
  if (markerRenderTimer !== null) {
    cancelAnimationFrame(markerRenderTimer);
    markerRenderTimer = null;
  }
}

// 地图视野内标记聚合（简化版）
function getClusteredMarkers(vehicles: MapVehicle[], zoomLevel: number): MapVehicle[] {
  // 根据缩放级别决定显示策略
  if (zoomLevel >= 12) {
    // 高缩放级别，显示所有标记（带分片）
    return vehicles;
  } else if (zoomLevel >= 8) {
    // 中等缩放，只显示在线和有报警的车辆
    return vehicles.filter(v => v.status === 'online' || v.hasAlarm);
  } else {
    // 低缩放级别，只显示有报警的车辆
    return vehicles.filter(v => v.hasAlarm);
  }
}

// 清理分片渲染资源
function cleanupMarkerRender() {
  stopChunkedMarkerRender();
  allMapVehicles = [];
  mapVehicles.value = [];
}

// 优化计算属性，使用缓存
const vehicleCount = computed(() => {
  let count = 0;
  const countVehicles = (nodes: VehicleNode[]) => {
    for (const node of nodes) {
      if (node.type === 'vehicle') count++;
      if (node.children) countVehicles(node.children);
    }
  };
  if (vehicleTreeData.value.length > 0) {
    countVehicles(vehicleTreeData.value);
  } else {
    count = mapVehicles.value.length;
  }
  return count;
});

// 缓存过滤结果，避免重复计算
const filteredVehicles = computed(() => {
  const vehicles = mapVehicles.value;
  return {
    online: vehicles.filter((v) => v.status === 'online'),
    offline: vehicles.filter((v) => v.status === 'offline'),
    alarm: vehicles.filter((v) => v.hasAlarm),
  };
});

const onlineCount = computed(() => filteredVehicles.value.online.length);
const offlineCount = computed(() => filteredVehicles.value.offline.length);
const alarmCount = computed(() => filteredVehicles.value.alarm.length);

// 转换数据格式以适配子组件
const mapVehiclesDisplay = computed(() => {
  return mapVehicles.value.map(v => ({
    ...v,
    x: v.mapX,
    y: v.mapY,
    name: v.license_plate,
    hasAlarm: v.alarm
  }));
});

const selectedVehicleDisplay = computed(() => {
  if (!selectedVehicle.value) return null;
  return {
    id: selectedVehicle.value.id,
    name: selectedVehicle.value.license_plate,
    status: selectedVehicle.value.status,
    speed: selectedVehicle.value.speed,
    direction: selectedVehicle.value.direction,
    mileage: selectedVehicle.value.mileage,
    loadWeight: selectedVehicle.value.loadWeight,
    totalWeight: selectedVehicle.value.totalWeight,
    location: selectedVehicle.value.location
  };
});

// 底部信息栏相关数据
const currentDate = ref('');
const serverIp = ref(localStorage.getItem('serverIp') || '127.0.0.1');
const serverPort = ref(localStorage.getItem('serverPort') || '9808');
const processedAlarmCount = ref(0);
const unprocessedAlarmCount = ref(0);

// 更新当前日期
function updateCurrentDate() {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  const hours = String(now.getHours()).padStart(2, '0');
  const minutes = String(now.getMinutes()).padStart(2, '0');
  const seconds = String(now.getSeconds()).padStart(2, '0');
  currentDate.value = `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
}

// 获取报警统计数据
async function updateAlarmStats() {
  try {
    const response = await api.get('/api/alerts/stats');
    unprocessedAlarmCount.value = response.data?.unprocessed || 0;
    processedAlarmCount.value = response.data?.processed || 0;
  } catch (error) {
    console.error('获取报警统计数据失败:', error);
    // 降级处理，使用本地计算
    unprocessedAlarmCount.value = mapVehicles.value.filter((v) => v.hasAlarm).length;
    processedAlarmCount.value = 0;
  }
}

async function buildVehicleTree() {
  try {
    // 从后端 API 获取车辆数据
    console.log('从后端获取车辆数据...');
    const response = await api.get('/api/vehicles');
    console.log('获取到车辆数据:', response);

    if (response && response.list) {
      const vehicles = response.list as BackendVehicle[];
      console.log('获取到车辆数据:', vehicles);

      // 构建车辆树
      const company1: VehicleNode = {
        id: 'company-1',
        name: '北京XX物流公司',
        type: 'company',
        children: [],
      };

      const group1: VehicleNode = {
        id: 'group-1',
        name: '车队一队',
        type: 'group',
        children: [],
      };

      const group2: VehicleNode = {
        id: 'group-2',
        name: '车队二队',
        type: 'group',
        children: [],
      };

      // 分配车辆到不同车队
      vehicles.forEach((v: BackendVehicle, index: number) => {
        const vehicleNode: VehicleNode = {
          id: `v-${v.vehicle_id}`,
          name: v.license_plate,
          type: 'vehicle',
          status: v.status === 1 ? 'online' : 'offline',
          hasAlarm: v.alarm || false,
          speed: v.speed || '0',
          direction: v.direction || '静止',
          mileage: v.mileage || '0',
          loadWeight: v.load_weight || '0',
          totalWeight: v.total_weight || '0',
          location: v.location || '未知位置',
          x: 30 + index * 10,
          y: 30 + index * 5,
        };

        if (index % 2 === 0) {
          group1.children!.push(vehicleNode);
        } else {
          group2.children!.push(vehicleNode);
        }
      });

      company1.children = [group1, group2];
      vehicleTreeData.value = [company1];

      // 转换为地图车辆格式 - 使用分片渲染
      const mapVehiclesData = vehicles.map((v: BackendVehicle, index: number) => ({
        id: v.vehicle_id,
        license_plate: v.license_plate,
        status: v.status === 1 ? 'online' : 'offline',
        alarm: v.alarm || false,
        speed: v.speed || '0',
        direction: v.direction || '静止',
        mileage: v.mileage || '0',
        loadWeight: v.load_weight || '0',
        totalWeight: v.total_weight || '0',
        location: v.location || '未知位置',
        mapX: 30 + index * 10,
        mapY: 30 + index * 5,
      }));
      
      // 使用分片渲染优化大量标记
      console.log(`[MapWindow] 开始分片渲染 ${mapVehiclesData.length} 个车辆标记`);
      startChunkedMarkerRender(mapVehiclesData);
    } else {
      console.warn('获取车辆数据失败，使用空数据');
      vehicleTreeData.value = [];
      cleanupMarkerRender();
    }
  } catch (error) {
    console.error('获取车辆数据失败:', error);
    // 使用空数据作为 fallback
    vehicleTreeData.value = [];
    cleanupMarkerRender();
  }
  console.log('buildVehicleTree 执行完成');
}

function handleNodeClick(data: VehicleNode) {
  if (data.type === 'vehicle') {
    selectVehicle(data);
  }
}

function selectVehicle(vehicle: MapVehicle | VehicleNode) {
  if ('mapX' in vehicle) {
    // MapVehicle 类型
    selectedVehicle.value = selectedVehicle.value?.id === vehicle.id ? null : vehicle;
  } else if ('x' in vehicle) {
    // VehicleNode 类型，转换为 MapVehicle
    const mapVehicle: MapVehicle = {
      id: parseInt(vehicle.id.replace('v-', '')),
      license_plate: vehicle.name,
      status: vehicle.status || 'offline',
      alarm: vehicle.hasAlarm || false,
      speed: vehicle.speed || '0',
      direction: vehicle.direction || '静止',
      mileage: vehicle.mileage || '0',
      loadWeight: vehicle.loadWeight || '0',
      totalWeight: vehicle.totalWeight || '0',
      location: vehicle.location || '未知位置',
      mapX: vehicle.x || 0,
      mapY: vehicle.y || 0
    };
    selectedVehicle.value = selectedVehicle.value?.id === mapVehicle.id ? null : mapVehicle;
  }
}

function handleMapTypeChange(type: string) {
  mapType.value = type;
  // 清除旧地图实例
  if (olMap) { olMap.dispose(); olMap = null; }
  if (vehicleLayer) { vehicleLayer = null; }
  if (type === 'local') { loadLocalMap(); }
  // 统一通过 initializeMap 渲染
  nextTick(() => {
    if (!mapRef.value) return;
    const container = mapRef.value;
    container.style.width = '100%';
    container.style.height = '100%';
    container.style.display = 'block';
    initializeMap(container);
  });
}

// 初始化天地图
function initTiandituMap() {
  // 检查DOM元素是否存在
  if (!mapRef.value) {
    console.warn('地图容器DOM元素不存在，跳过初始化');
    return;
  }

  // 确保地图容器有宽度和高度
  const mapContainer = mapRef.value;

  // 强制设置容器样式，确保它能获取到尺寸
  mapContainer.style.width = '100%';
  mapContainer.style.height = '100%';
  mapContainer.style.minWidth = '300px';
  mapContainer.style.minHeight = '300px';
  mapContainer.style.display = 'block';

  // 检查地图容器的宽度和高度
  const width = mapContainer.clientWidth;
  const height = mapContainer.clientHeight;

  console.log('地图容器初始尺寸:', width, 'x', height);

  if (width > 0 && height > 0) {
    // 容器尺寸正常，初始化地图
    initializeMap(mapContainer);
  } else {
    console.warn('地图容器宽度或高度为0，使用ResizeObserver监听尺寸变化');
    
    // 使用ResizeObserver监听容器尺寸变化
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0];
      const { width, height } = entry.contentRect;
      
      console.log('ResizeObserver捕获到尺寸变化:', width, 'x', height);
      
      if (width > 0 && height > 0) {
        console.log('地图容器尺寸已更新，开始初始化地图:', width, 'x', height);
        // 先断开观察，避免无限循环
        observer.disconnect();
        // 使用setTimeout避免在ResizeObserver回调中直接修改DOM
        setTimeout(() => {
          initializeMap(mapContainer);
        }, 0);
      }
    });
    
    // 开始观察容器尺寸变化
    observer.observe(mapContainer);
    
    // 设置超时，防止ResizeObserver一直不触发
    let retryCount = 0;
    const maxRetries = 20;
    const retryInterval = 500;
    
    const retryTimer = setInterval(() => {
      retryCount++;
      const currentWidth = mapContainer.clientWidth;
      const currentHeight = mapContainer.clientHeight;
      
      console.log(`重试初始化地图 (${retryCount}/${maxRetries}):`, currentWidth, 'x', currentHeight);
      
      if (currentWidth > 0 && currentHeight > 0) {
        clearInterval(retryTimer);
        observer.disconnect();
        console.log('重试成功，使用当前尺寸初始化地图:', currentWidth, 'x', currentHeight);
        initializeMap(mapContainer);
      } else if (retryCount >= maxRetries) {
        clearInterval(retryTimer);
        observer.disconnect();
        console.error('地图容器尺寸一直为0，放弃初始化');
      }
    }, retryInterval);
  }
}

// 实际初始化地图的函数
function initializeMap(mapContainer: HTMLElement) {
  try {
    let layers: any[] = [];
    const isLocal = mapType.value === 'local';

    if (isLocal) {
      // 本地地图
      layers.push(new TileLayer({
        source: new XYZ({
          url: '/api/map/tiles/{z}/{x}/{y}.png',
          maxZoom: 18,
          tileLoadFunction: (imageTile: any, src: string) => {
            fetch(src).then(r => r.ok ? r.blob() : Promise.reject()).then(blob => {
              imageTile.getImage().src = URL.createObjectURL(blob);
            }).catch(() => {
              const m = src.match(/tiles\/(\d+)\/(\d+)\/(\d+)/);
              imageTile.getImage().src = m
                ? `https://webrd01.is.autonavi.com/appmaptile?lang=zh_cn&size=1&scale=1&style=8&x=${m[2]}&y=${m[3]}&z=${m[1]}`
                : '';
            });
          }
        }),
        zIndex: 1
      }));
    } else {
      // 天地图矢量图层
      layers.push(new TileLayer({ source: new XYZ({ url: getTiandituVecUrl(), crossOrigin: 'anonymous', projection: 'EPSG:3857' }) }));
      // 天地图注记图层
      const labelLayer = new TileLayer({ source: new XYZ({ url: getTiandituCvaUrl(), crossOrigin: 'anonymous', projection: 'EPSG:3857' }) });
      labelLayer.set('isLabelLayer', true);
      layers.push(labelLayer);
    }

    // 上海坐标作为默认中心点 [经度, 纬度]
    let defaultCenter = [121.4737, 31.2304]; // 上海
    let defaultZoom = 10;

    // 如果有真实车辆数据，计算车辆中心点并自动适应
    if (mapVehicles.value.length > 0) {
      const vehiclesWithCoords = mapVehicles.value.filter(v => v.longitude && v.latitude);
      if (vehiclesWithCoords.length > 0) {
        // 计算所有车辆的中心点
        const avgLng = vehiclesWithCoords.reduce((sum, v) => sum + v.longitude, 0) / vehiclesWithCoords.length;
        const avgLat = vehiclesWithCoords.reduce((sum, v) => sum + v.latitude, 0) / vehiclesWithCoords.length;
        defaultCenter = [avgLng, avgLat];
        
        // 计算缩放级别：根据车辆分布范围自动调整
        const lngRange = Math.max(...vehiclesWithCoords.map(v => v.longitude)) - Math.min(...vehiclesWithCoords.map(v => v.longitude));
        const latRange = Math.max(...vehiclesWithCoords.map(v => v.latitude)) - Math.min(...vehiclesWithCoords.map(v => v.latitude));
        const maxRange = Math.max(lngRange, latRange);
        
        // 根据范围计算合适的缩放级别
        if (maxRange > 10) defaultZoom = 6;
        else if (maxRange > 5) defaultZoom = 7;
        else if (maxRange > 2) defaultZoom = 8;
        else if (maxRange > 1) defaultZoom = 9;
        else defaultZoom = 11;
        
        console.log(`根据 ${vehiclesWithCoords.length} 辆真实车辆位置自动适应地图: 中心=[${avgLng.toFixed(4)}, ${avgLat.toFixed(4)}], 缩放级别=${defaultZoom}`);
      }
    }

    // 计算地图中心点
    const centerPoint = fromLonLat(defaultCenter);

    // 创建地图实例
    olMap = new Map({
      target: mapContainer,
      layers,
      view: new View({
        center: centerPoint,
        zoom: defaultZoom,
        maxZoom: 18,
        minZoom: 2,
      }),
    });

    // 地图加载完成后添加车辆标记
    olMap.on('rendercomplete', () => {
      updateVehicleMarkers();
    });

    console.log(`地图初始化成功: 类型=${isLocal ? '本地' : '天地图'}, 中心点=[${defaultCenter[0]}, ${defaultCenter[1]}], 缩放级别=${defaultZoom}`);
  } catch (error) {
    console.error('天地图初始化失败:', error);
  }
}

// 更新车辆标记
function updateVehicleMarkers() {
  if (!olMap || mapType.value !== 'tianditu') return;

  // 移除旧的车辆图层
  if (vehicleLayer) {
    olMap.removeLayer(vehicleLayer);
  }

  // 创建新的矢量源
  const vectorSource = new VectorSource();

  // 添加车辆标记
  mapVehicles.value.forEach((vehicle, index) => {
    // 为模拟数据生成合理的经纬度
    const lng = 100 + (index % 10) * 2; // 100-118
    const lat = 30 + (index % 5) * 2; // 30-38

    const markerGeometry = new Point(fromLonLat([lng, lat]));
    const markerPoint = new Feature({
      geometry: markerGeometry,
      properties: {
        id: vehicle.id,
        name: vehicle.license_plate,
        status: vehicle.status,
        hasAlarm: vehicle.alarm,
      },
    });

    // 设置标记样式
    const markerStyle = new Style({
      image: new Circle({
        radius: 6,
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
  });

  // 创建车辆图层
  vehicleLayer = new VectorLayer({
    source: vectorSource,
    zIndex: 10,
  });

  olMap.addLayer(vehicleLayer);
}

async function loadLocalMap() {
  try {
    console.log('加载本地地图...');
    const resp = await api.get('/api/map/metadata');
    const data = resp?.data?.data;
    if (data) {
      localMapInfo.value = data;
      ElMessage.success(`已加载本地地图: ${data.name}`);
    } else {
      // 降级：使用内置元数据
      localMapInfo.value = {
        name: '2014年版中国地图', projection: 'WGS84',
        center: '104.195, 35.861', zoomLevel: 5,
        layers: [
          { id: 1, file: '省界.MAP', label: '省级边界', visible: true, minZoom: 2, maxZoom: 8 },
          { id: 2, file: '市界.MAP', label: '市级边界', visible: true, minZoom: 6, maxZoom: 10 },
          { id: 3, file: '县界.MAP', label: '县级边界', visible: true, minZoom: 8, maxZoom: 12 },
          { id: 4, file: '高速.MAP', label: '高速公路', visible: true, minZoom: 4, maxZoom: 14 },
          { id: 5, file: '国道.MAP', label: '国道', visible: true, minZoom: 5, maxZoom: 14 },
          { id: 6, file: '省道.MAP', label: '省道', visible: true, minZoom: 6, maxZoom: 14 },
          { id: 7, file: '县道.MAP', label: '县道', visible: true, minZoom: 7, maxZoom: 14 },
          { id: 8, file: '水系.MAP', label: '水系', visible: true, minZoom: 3, maxZoom: 12 },
          { id: 9, file: '绿地.MAP', label: '绿地', visible: true, minZoom: 8, maxZoom: 16 },
          { id: 10, file: '建成区界.MAP', label: '建成区边界', visible: true, minZoom: 10, maxZoom: 16 },
        ],
      };
      ElMessage.success('已加载本地地图(内置元数据)');
    }
  } catch (error) {
    console.error('加载本地地图失败:', error);
    ElMessage.warning('加载本地地图失败，使用默认视图');
  }
}

// 初始化本地地图（使用高德瓦片作为底图）
function initLocalMap() {
  if (!localMapRef.value) {
    console.warn('本地地图容器DOM元素不存在，跳过初始化');
    return;
  }

  // 确保容器有正确的尺寸
  const mapContainer = localMapRef.value;
  mapContainer.style.width = '100%';
  mapContainer.style.height = '100%';
  mapContainer.style.position = 'relative';

  // 本地地图：优先从后端 /api/map/tiles 加载，降级到高德瓦片
  const localSource = new XYZ({
    url: '/api/map/tiles/{z}/{x}/{y}.png',
    maxZoom: 18,
    tileLoadFunction: (imageTile: any, src: string) => {
      fetch(src).then(r => r.ok ? r.blob() : Promise.reject()).then(blob => {
        imageTile.getImage().src = URL.createObjectURL(blob);
      }).catch(() => {
        const m = src.match(/tiles\/(\d+)\/(\d+)\/(\d+)/);
        imageTile.getImage().src = m
          ? `https://webrd01.is.autonavi.com/appmaptile?lang=zh_cn&size=1&scale=1&style=8&x=${m[2]}&y=${m[3]}&z=${m[1]}`
          : '';
      });
    }
  });

  const baseLayer = new TileLayer({
    source: localSource,
  });

  // 创建车辆图层
  vehicleLayer = new VectorLayer({
    source: new VectorSource(),
    style: (feature: any) => {
      const status = feature.get('status');
      const color = getVehicleColor(status);
      return new Style({
        image: new Circle({
          radius: 6,
          fill: new Fill({ color }),
          stroke: new Stroke({ color: '#fff', width: 2 }),
        }),
      });
    },
  });

  // 从 GeoSet 元数据获取中心点和缩放级别
  const centerStr = localMapInfo.value?.center || '104.195, 35.861';
  const [lon, lat] = centerStr.split(',').map(Number);
  const zoom = localMapInfo.value?.zoomLevel || 5;

  // 初始化 OpenLayers 地图实例
  olMap = new Map({
    target: mapContainer,
    layers: [baseLayer, vehicleLayer],
    view: new View({
      center: fromLonLat([lon, lat]),
      zoom: Math.min(zoom, 10), // 限制最大初始缩放
      maxZoom: 18,
      minZoom: 2,
    }),
  });

  // 加载车辆数据
  loadVehicles();

  console.log('本地地图初始化完成（使用高德瓦片）');
}

// 防抖处理的刷新函数
const handleRefresh = debounce(async () => {
  isRefreshing.value = true;
  try {
    await buildVehicleTree();
    ElMessage.success('数据已刷新');
  } catch (error) {
    console.error('刷新数据失败:', error);
    ElMessage.error('刷新数据失败');
  } finally {
    isRefreshing.value = false;
  }
}, 300);

// 标签页切换函数
function handleTabChange(tabName: string) {
  activeTab.value = tabName;
  console.log('切换到标签页:', tabName);
}

// 列宽调整功能
function startResize(columnIndex: number, event: MouseEvent) {
  event.preventDefault();
  resizingColumn.value = columnIndex;
  startX.value = event.clientX;

  // 获取当前列的宽度
  const headerItems = document.querySelectorAll(`.active-tab-${activeTab.value} .header-item`);
  const column = headerItems[columnIndex];
  if (column) {
    startWidth.value = column.clientWidth;
  }

  // 添加全局鼠标事件监听器
  document.addEventListener('mousemove', resizeColumn);
  document.addEventListener('mouseup', stopResize);

  // 添加调整中状态
  if (column) {
    column.classList.add('resizing');
  }
}

// 节流处理的列宽调整函数
const resizeColumn = throttle((event: MouseEvent) => {
  if (resizingColumn.value === null) return;

  const deltaX = event.clientX - startX.value;
  const newWidth = startWidth.value + deltaX;

  // 最小宽度限制
  if (newWidth < 50) return;

  // 设置新宽度
  const headerItems = document.querySelectorAll(`.active-tab-${activeTab.value} .header-item`);
  const column = headerItems[resizingColumn.value];
  if (column) {
    (column as HTMLElement).style.width = `${newWidth}px`;
  }

  // 同时调整表格行中对应列的宽度
  const rowItems = document.querySelectorAll(
    `.active-tab-${activeTab.value} .row-item:nth-child(${resizingColumn.value + 1})`
  );
  rowItems.forEach((item) => {
    (item as HTMLElement).style.width = `${newWidth}px`;
  });
}, 16); // 约60fps

function stopResize() {
  // 移除调整中状态
  const headerItems = document.querySelectorAll(`.active-tab-${activeTab.value} .header-item`);
  headerItems.forEach((item) => {
    item.classList.remove('resizing');
  });

  // 移除全局鼠标事件监听器
  document.removeEventListener('mousemove', resizeColumn);
  document.removeEventListener('mouseup', stopResize);

  // 重置状态
  resizingColumn.value = null;
  startX.value = 0;
  startWidth.value = 0;
}

// 侧边栏宽度调整功能（由于侧边栏已拆分为独立组件，暂时禁用）
function startResizeSidebar(event: MouseEvent) {
  event.preventDefault();
  // 侧边栏已拆分为独立组件，resize功能需要通过组件间通信实现
  console.log('侧边栏resize功能已禁用，请通过组件属性调整宽度');
}

// 节流处理的侧边栏调整函数（由于侧边栏已拆分为独立组件，暂时禁用）
const resizeSidebar = throttle((event: MouseEvent) => {
  // 侧边栏已拆分为独立组件，resize功能需要通过组件间通信实现
}, 16);

function stopResizeSidebar() {
  // 侧边栏已拆分为独立组件，resize功能需要通过组件间通信实现
}

// 组件卸载时清除定时器和地图实例
onUnmounted(() => {
  // 清理日期更新定时器
  if (dateInterval) {
    clearInterval(dateInterval);
    dateInterval = null;
  }
  
  // 清理分片渲染定时器
  cleanupMarkerRender();
  
  // 清理 ResizeObserver
  if (resizeObserver) {
    resizeObserver.disconnect();
    resizeObserver = null;
  }
  
  // 清理 OpenLayers 地图实例
  if (olMap) {
    olMap.dispose();
    olMap = null;
  }
});

onMounted(async () => {
  console.log('MapWindow onMounted 开始执行');
  settingsStore.loadHomePageName();
  tiandituKey.value = getTiandituKey();
  gaodeKey.value = localStorage.getItem('gaodeKey') || '';

  try {
    console.log('开始获取车辆数据...');
    await buildVehicleTree();
    console.log('车辆数据获取完成');
  } catch (error) {
    console.error('初始化车辆数据失败:', error);
  }

  // 等待DOM更新完成后再初始化地图
  console.log('等待DOM更新...');
  await nextTick();
  console.log('DOM更新完成');

  console.log('开始初始化地图...');
  console.log('mapRef.value:', mapRef.value);
  console.log('localMapRef.value:', localMapRef.value);
  console.log('mapType.value:', mapType.value);

  // 统一初始化地图
  console.log('初始化地图...');
  handleMapTypeChange(mapType.value);

  // 初始化底部信息栏数据
  console.log('初始化底部信息栏数据...');
  updateCurrentDate();
  await updateAlarmStats();
  console.log('底部信息栏数据初始化完成');

  // 每秒更新日期时间
  console.log('设置日期时间更新定时器...');
  dateInterval = setInterval(updateCurrentDate, 1000);
  console.log('MapWindow onMounted 执行完成');
});
</script>

<style scoped>
.map-window {
  width: 100%;
  height: 100vh;
  background-color: #f5f7fa;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.top-header {
  background: #fff;
  border-bottom: 1px solid #dcdfe6;
  padding: 0 20px;
  height: 50px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.top-header h1 {
  font-size: 16px;
  font-weight: bold;
  color: #303133;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.main-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  padding: 12px;
  gap: 0;
  position: relative;
}

.sidebar-container {
  width: 280px;
  background: #fff;
  border: 1px solid #dcdfe6;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  transition: width 0.2s ease;
}

/* 侧边栏调整手柄 */
.resize-handle {
  width: 4px;
  background: #e4e7ed;
  cursor: col-resize;
  position: relative;
  margin: 0 4px;
  border-radius: 2px;
  transition: background 0.2s ease;
}

.resize-handle:hover {
  background: #409eff;
}

.resize-handle.resizing {
  background: #409eff;
  width: 4px;
}

.resize-handle::before {
  content: '';
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 100%;
  height: 40px;
  background: transparent;
  border-radius: 2px;
}

.resize-handle:hover::before {
  background: rgba(64, 158, 255, 0.1);
}

.map-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow: hidden;
  min-width: 300px;
}

.sidebar-header {
  padding: 12px 16px;
  border-bottom: 1px solid #dcdfe6;
  font-weight: bold;
  color: #303133;
  display: flex;
  align-items: center;
  gap: 8px;
  background-color: #fafafa;
}

.vehicle-tree {
  padding: 8px;
}

.tree-node-content {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}

.node-icon {
  font-size: 16px;
}

.node-icon.company {
  color: #409eff;
}
.node-icon.group {
  color: #67c23a;
}
.node-icon.vehicle {
  color: #e6a23c;
}

.node-label {
  flex: 1;
  font-size: 13px;
}

.alarm-icon {
  color: #f56c6c;
  font-size: 14px;
}

.map-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  background: #fff;
  padding: 10px 16px;
  border: 1px solid #dcdfe6;
}

.map-api-key {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #606266;
}

.map-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  position: relative;
}

.map {
  flex: 1;
  position: relative;
  background-color: #e8f4f8;
  overflow: hidden;
}

.map-grid {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image:
    linear-gradient(rgba(59, 130, 246, 0.08) 1px, transparent 1px),
    linear-gradient(90deg, rgba(59, 130, 246, 0.08) 1px, transparent 1px);
  background-size: 50px 50px;
}

.local-map-tiles {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.tile-row {
  display: flex;
  gap: 2px;
}

.map-tile {
  width: 120px;
  height: 80px;
  background: linear-gradient(135deg, #e8f4f8 0%, #d4e8d4 100%);
  border: 1px solid #c0d4c0;
  display: flex;
  justify-content: center;
  align-items: center;
  font-size: 12px;
  color: #606266;
}

/* 本地地图底图样式 */
.local-map-base {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 1;
}

.china-outline {
  position: absolute;
  top: 10%;
  left: 20%;
  width: 60%;
  height: 80%;
  background: linear-gradient(135deg, #e6f7ff 0%, #f0f9ff 100%);
  border: 2px solid #91d5ff;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(145, 213, 255, 0.3);
}

.province-labels {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 2;
}

.province-label {
  position: absolute;
  font-size: 12px;
  color: #1890ff;
  font-weight: 500;
  text-shadow: 1px 1px 2px rgba(255, 255, 255, 0.8);
  transform: translate(-50%, -50%);
  background: rgba(255, 255, 255, 0.7);
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid #d9ecff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.province-label:hover {
  background: rgba(255, 255, 255, 0.9);
  transform: translate(-50%, -50%) scale(1.1);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

/* 本地地图信息面板 */
.local-map-info {
  position: absolute;
  top: 10px;
  left: 10px;
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  padding: 12px;
  width: 280px;
  z-index: 20;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.map-info-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: bold;
  color: #303133;
  padding-bottom: 8px;
  border-bottom: 1px solid #ebeef5;
  margin-bottom: 8px;
}

.map-info-header .el-icon {
  color: #409eff;
}

.map-info-content {
  font-size: 12px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 6px;
}

.info-row .label {
  color: #909399;
}

.info-row .value {
  color: #303133;
  font-weight: 500;
}

.layer-list {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid #ebeef5;
}

.layer-title {
  font-weight: bold;
  color: #303133;
  margin-bottom: 8px;
}

.layer-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  border-bottom: 1px solid #f0f0f0;
}

.layer-item:last-child {
  border-bottom: none;
}

.layer-item .el-icon {
  color: #909399;
  font-size: 14px;
}

.layer-name {
  flex: 1;
  color: #303133;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 地图加载状态 */
.map-loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: #606266;
  font-size: 14px;
}

.loading-icon {
  font-size: 32px;
  color: #409eff;
  animation: rotate 1s linear infinite;
}

@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.map-marker {
  position: absolute;
  transform: translate(-50%, -50%);
  cursor: pointer;
  z-index: 10;
}

.marker-icon {
  width: 28px;
  height: 28px;
  border-radius: 50% 50% 50% 0;
  background-color: #67c23a;
  display: flex;
  justify-content: center;
  align-items: center;
  color: #fff;
  transform: rotate(-45deg);
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
}

.marker-icon.offline {
  background-color: #909399;
}
.marker-icon.alarm {
  background-color: #f56c6c;
}

.marker-label {
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-top: 4px;
  background: rgba(0, 0, 0, 0.75);
  color: #fff;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
  white-space: nowrap;
}

.data-toolbar {
  background: #f0f8ff;
  border: 1px solid #b8d4f1;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  min-height: 200px;
}

.toolbar-header {
  background: linear-gradient(to bottom, #e6f3ff, #d4e8ff);
  border-bottom: 1px solid #b8d4f1;
  padding: 8px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: bold;
  color: #1f3a5f;
}

.toolbar-title {
  font-size: 13px;
}

.toolbar-stats {
  display: flex;
  gap: 16px;
  font-size: 12px;
}

.toolbar-stats span {
  color: #336699;
}

.toolbar-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.data-table {
  flex: 1;
  background: #fff;
  border-bottom: 1px solid #b8d4f1;
  overflow: auto;
}

.table-header {
  display: flex;
  background: linear-gradient(to bottom, #f0f8ff, #e6f3ff);
  border-bottom: 1px solid #b8d4f1;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: bold;
  color: #1f3a5f;
  position: sticky;
  top: 0;
  z-index: 5;
}

/* 列宽调整功能 */
.header-item {
  position: relative;
  cursor: col-resize;
}

.header-item:hover::after {
  content: '';
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background: #409eff;
  cursor: col-resize;
}

.header-item.resizing::after {
  background: #409eff;
  width: 2px;
}

/* 允许表格行的单元格也支持调整 */
.row-item {
  position: relative;
}

.table-body {
  font-size: 12px;
}

.table-row {
  display: flex;
  padding: 6px 12px;
  border-bottom: 1px solid #f0f0f0;
  align-items: center;
}

/* 实时信息表格列宽 */
.active-tab-realtime .header-item:nth-child(1),
.active-tab-realtime .row-item:nth-child(1) {
  width: 60px;
}

.active-tab-realtime .header-item:nth-child(2),
.active-tab-realtime .row-item:nth-child(2) {
  width: 100px;
}

.active-tab-realtime .header-item:nth-child(3),
.active-tab-realtime .row-item:nth-child(3) {
  width: 180px;
}

.active-tab-realtime .header-item:nth-child(4),
.active-tab-realtime .row-item:nth-child(4) {
  width: 150px;
}

.active-tab-realtime .header-item:nth-child(5),
.active-tab-realtime .row-item:nth-child(5) {
  width: 80px;
}

.active-tab-realtime .header-item:nth-child(6),
.active-tab-realtime .row-item:nth-child(6) {
  width: 100px;
}

.active-tab-realtime .header-item:nth-child(7),
.active-tab-realtime .row-item:nth-child(7) {
  width: 100px;
}

.active-tab-realtime .header-item:nth-child(8),
.active-tab-realtime .row-item:nth-child(8) {
  width: 100px;
}

.active-tab-realtime .header-item:nth-child(9),
.active-tab-realtime .row-item:nth-child(9) {
  width: 60px;
}

/* 车辆报警表格列宽 */
.active-tab-alarm .header-item:nth-child(1),
.active-tab-alarm .row-item:nth-child(1) {
  width: 60px;
}

.active-tab-alarm .header-item:nth-child(2),
.active-tab-alarm .row-item:nth-child(2) {
  width: 100px;
}

.active-tab-alarm .header-item:nth-child(3),
.active-tab-alarm .row-item:nth-child(3) {
  width: 180px;
}

.active-tab-alarm .header-item:nth-child(4),
.active-tab-alarm .row-item:nth-child(4) {
  width: 120px;
}

.active-tab-alarm .header-item:nth-child(5),
.active-tab-alarm .row-item:nth-child(5) {
  width: 200px;
}

.active-tab-alarm .header-item:nth-child(6),
.active-tab-alarm .row-item:nth-child(6) {
  width: 100px;
}

/* 交互内容表格列宽 */
.active-tab-interaction .header-item:nth-child(1),
.active-tab-interaction .row-item:nth-child(1) {
  width: 60px;
}

.active-tab-interaction .header-item:nth-child(2),
.active-tab-interaction .row-item:nth-child(2) {
  width: 100px;
}

.active-tab-interaction .header-item:nth-child(3),
.active-tab-interaction .row-item:nth-child(3) {
  width: 200px;
}

.active-tab-interaction .header-item:nth-child(4),
.active-tab-interaction .row-item:nth-child(4) {
  width: 180px;
}

.active-tab-interaction .header-item:nth-child(5),
.active-tab-interaction .row-item:nth-child(5) {
  width: 120px;
}

.active-tab-interaction .header-item:nth-child(6),
.active-tab-interaction .row-item:nth-child(6) {
  width: 250px;
}

.table-row:hover {
  background: #f5faff;
}

.toolbar-tabs {
  background: #fff;
  border-top: 1px solid #b8d4f1;
}

.tab-content {
  padding: 12px;
  font-size: 12px;
  min-height: 80px;
}

.vehicle-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.vehicle-info .info-item {
  display: flex;
  gap: 16px;
}

.vehicle-info .label {
  color: #333;
}

.vehicle-info .value {
  color: #0066cc;
  font-weight: bold;
}

.alarm-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.alarm-item {
  padding: 6px 12px;
  background: #fff3f3;
  border: 1px solid #ffcccc;
  border-radius: 4px;
  display: flex;
  gap: 16px;
}

.alarm-vehicle {
  font-weight: bold;
  color: #cc0000;
}

.alarm-time {
  color: #666;
}

.alarm-type {
  color: #cc0000;
}

.detail-panel {
  height: 220px;
  background: #fff;
  border-top: 1px solid #dcdfe6;
  flex-shrink: 0;
}

.panel-header {
  padding: 10px 20px;
  border-bottom: 1px solid #dcdfe6;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: #fafafa;
}

.vehicle-title {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 15px;
  font-weight: bold;
  color: #303133;
}

.panel-content {
  padding: 16px 20px;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 12px;
}

.info-item {
  background: #fafafa;
  padding: 10px;
  border: 1px solid #ebeef5;
}

.info-item .label {
  display: block;
  font-size: 12px;
  color: #909399;
  margin-bottom: 4px;
}

.info-item .value {
  font-size: 13px;
  color: #303133;
  font-weight: 500;
}

@media (max-width: 1200px) {
  .info-grid {
    grid-template-columns: repeat(3, 1fr);
  }

  .toolbar-section:first-child {
    width: 100%;
  }

  .data-toolbar {
    flex-direction: column;
    height: auto;
  }
}

/* 底部信息栏样式 */
.bottom-info-bar {
  background: linear-gradient(to right, #f8f9fa, #e9ecef);
  border-top: 2px solid #dee2e6;
  padding: 10px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
}

.bottom-info-bar .info-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  padding: 4px 12px;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.bottom-info-bar .info-item:hover {
  background: rgba(255, 255, 255, 1);
  transform: translateY(-2px);
  box-shadow: 0 3px 6px rgba(0, 0, 0, 0.15);
}

.bottom-info-bar .info-label {
  font-weight: 600;
  color: #495057;
  white-space: nowrap;
}

.bottom-info-bar .info-value {
  font-weight: 500;
  color: #212529;
  white-space: nowrap;
}

.bottom-info-bar .info-value.online {
  color: #28a745;
  font-weight: 700;
}

.bottom-info-bar .info-value.offline {
  color: #dc3545;
  font-weight: 700;
}

.bottom-info-bar .info-value.processed {
  color: #17a2b8;
  font-weight: 700;
}

.bottom-info-bar .info-value.unprocessed {
  color: #ffc107;
  font-weight: 700;
}

/* 响应式调整 */
@media (max-width: 1400px) {
  .bottom-info-bar {
    flex-wrap: wrap;
    gap: 8px;
    padding: 8px 16px;
  }

  .bottom-info-bar .info-item {
    font-size: 12px;
    padding: 3px 10px;
  }
}

@media (max-width: 768px) {
  .bottom-info-bar {
    justify-content: center;
  }

  .bottom-info-bar .info-item {
    flex: 1;
    min-width: 120px;
    justify-content: center;
  }
}
</style>


