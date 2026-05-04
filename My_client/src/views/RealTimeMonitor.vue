<template>
  <div class="realtime-monitor">
    <!-- 头部工具栏 -->
    <div class="monitor-header">
      <h2>货物运输实时监管大屏</h2>

      <div class="header-stats">
        <div class="stat-item">
          <div class="stat-label">累计发货</div>
          <div class="stat-value">{{ stats.totalShipment }}吨</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">累计卸货</div>
          <div class="stat-value">{{ stats.totalUnload }}吨</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">在途车辆</div>
          <div class="stat-value">{{ stats.inTransit }}辆</div>
        </div>
        <div class="stat-item">
          <div class="stat-label">运力</div>
          <div class="stat-value">{{ stats.capacity }}吨</div>
        </div>
      </div>
    </div>

    <!-- 主体内容 -->
    <div class="monitor-content">
      <!-- 左侧控制工具栏 -->
      <div class="control-sidebar">
        <!-- 运输载具列表栏 -->
        <div class="control-section">
          <div class="section-header" @click="toggleVehiclesPanel">
            <h3>运输载具列表</h3>
            <el-icon v-if="vehiclesPanelOpen"><ArrowUp /></el-icon>
            <el-icon v-else><ArrowDown /></el-icon>
          </div>
          <div v-if="vehiclesPanelOpen" class="section-content">
            <div class="vehicle-list">
              <div
                v-for="vehicle in vehicles"
                :key="vehicle.id"
                class="vehicle-item"
                :class="{ selected: selectedVehicle?.id === vehicle.id }"
                @click="selectVehicle(vehicle)"
              >
                <div class="vehicle-icon">
                  <el-icon :color="getVehicleColor(vehicle.status)"><Van /></el-icon>
                </div>
                <div class="vehicle-info">
                  <div class="vehicle-name">{{ vehicle.licensePlate }}</div>
                  <div class="vehicle-status">{{ getStatusText(vehicle.status) }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 货物装卸节点 -->
        <div class="control-section">
          <div class="section-header" @click="toggleNodesPanel">
            <h3>货物装卸节点</h3>
            <el-icon v-if="nodesPanelOpen"><ArrowUp /></el-icon>
            <el-icon v-else><ArrowDown /></el-icon>
          </div>
          <div v-if="nodesPanelOpen" class="section-content">
            <el-tabs v-model="activeNodeTab">
              <el-tab-pane label="发货节点">
                <div class="node-list">
                  <div v-for="node in shippingNodes" :key="node.id" class="node-item shipping-node">
                    <div class="node-name">{{ node.name }}</div>
                    <div class="node-info">{{ node.address }}</div>
                    <div class="node-stats">今日发货: {{ node.dailyShipment }}吨</div>
                  </div>
                </div>
              </el-tab-pane>
              <el-tab-pane label="卸货节点">
                <div class="node-list">
                  <div v-for="node in unloadingNodes" :key="node.id" class="node-item unloading-node">
                    <div class="node-name">{{ node.name }}</div>
                    <div class="node-info">{{ node.address }}</div>
                    <div class="node-stats">今日卸货: {{ node.dailyUnload }}吨</div>
                  </div>
                </div>
              </el-tab-pane>
              <el-tab-pane label="其他装卸位置">
                <div class="node-list">
                  <div v-for="node in otherNodes" :key="node.id" class="node-item other-node">
                    <div class="node-name">{{ node.name }}</div>
                    <div class="node-info">{{ node.address }}</div>
                  </div>
                </div>
              </el-tab-pane>
            </el-tabs>
          </div>
        </div>
      </div>

      <!-- 右侧内容区域 -->
      <div class="content-section">
        <!-- 地图区域 -->
        <div class="map-section">
          <!-- 地图类型切换 -->
          <div class="map-type-switch">
            <el-radio-group v-model="mapType" size="small">
              <el-radio-button value="tianditu">天地图</el-radio-button>
              <el-radio-button value="local">本地地图</el-radio-button>
            </el-radio-group>
          </div>

          <!-- 地图容器 -->
          <div ref="mapContainer" class="map-container"></div>
          <div v-if="loading" class="map-loading">
            <el-icon class="is-loading"><Loading /></el-icon>
            <span>{{ mapType === 'tianditu' ? '天地图加载中...' : '本地地图加载中...' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { ArrowUp, ArrowDown, Van, Loading } from '@element-plus/icons-vue';
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
import { throttle, debounce, MemoryMonitor } from '@/utils/performance';
import { getTiandituVecUrl, getTiandituCvaUrl } from '@/utils/mapConfig';


// 面板控制
const vehiclesPanelOpen = ref(true);
const nodesPanelOpen = ref(true);
const activeNodeTab = ref('0');
const mapType = ref('tianditu');
const displayMode = ref('map');
const loading = ref(true);
const mapContainer = ref<HTMLElement | null>(null);
const mapInstance = ref<Map | null>(null);
const nodeLayer = ref<VectorLayer | null>(null);

// 后端车辆数据类型
interface BackendVehicle {
  vehicle_id: number;
  license_plate: string;
  status: number;
}

// 车辆类型定义
interface VehicleItem {
  id: number;
  licensePlate: string;
  status: number;
}

const selectedVehicle = ref<VehicleItem | null>(null);

// 统计数据
interface Stats {
  totalShipment: number;
  totalUnload: number;
  inTransit: number;
  capacity: number;
}

const stats = ref<Stats>({
  totalShipment: 0,
  totalUnload: 0,
  inTransit: 0,
  capacity: 0,
});

// 运输载具数据
const vehicles = ref<VehicleItem[]>([]);

// 节点类型定义
interface Node {
  id: number;
  name: string;
  type: string;
  latitude: number;
  longitude: number;
  status: number;
}

// 装卸节点数据
const shippingNodes = ref<Node[]>([]);
const unloadingNodes = ref<Node[]>([]);
const otherNodes = ref<Node[]>([]);

// 后端节点数据类型
interface BackendNode {
  id: number;
  name: string;
  type: string;
  latitude: number;
  longitude: number;
  status: number;
}

// 加载数据
const loadData = async () => {
  try {
    // 加载车辆数据
    console.log('开始加载车辆数据...');
    const vehicleResponse = await api.get('/api/vehicles');
    if (vehicleResponse && vehicleResponse.items) {
      vehicles.value = vehicleResponse.items.map((vehicle: BackendVehicle) => ({
        id: vehicle.vehicle_id,
        licensePlate: vehicle.license_plate,
        status: vehicle.status || 1,
      }));
      console.log('车辆数据加载完成:', vehicles.value.length, '辆');
    }

    // 加载车辆统计数据
    console.log('开始加载车辆统计数据...');
    const vehicleStatsResponse = await api.get('/api/statistics/vehicles');
    console.log('车辆统计数据加载完成:', vehicleStatsResponse);

    // 加载称重统计数据
    console.log('开始加载称重统计数据...');
    const weighingStatsResponse = await api.get('/api/statistics/weighing');
    console.log('称重统计数据加载完成:', weighingStatsResponse);

    // 加载装卸节点数据
    console.log('开始加载装卸节点数据...');
    const nodesResponse = await api.get('/api/location/places');
    console.log('装卸节点数据加载完成:', nodesResponse);

    // 处理统计数据
    if (weighingStatsResponse) {
      stats.value = {
        totalShipment: weighingStatsResponse.total_weight || 0,
        totalUnload: weighingStatsResponse.total_weight || 0,
        inTransit: vehicleStatsResponse.online_vehicles || 0,
        capacity: (vehicleStatsResponse.total_vehicles || 0) * 50,
      };
    }

    // 处理装卸节点数据
    if (nodesResponse && nodesResponse.items) {
      shippingNodes.value = nodesResponse.items
        .filter((node: BackendNode) => node.type === 'shipping')
        .map((node: BackendNode) => ({
          id: node.id,
          name: node.name,
          type: node.type,
          latitude: node.latitude,
          longitude: node.longitude,
          status: 1,
        }));
      unloadingNodes.value = nodesResponse.items
        .filter((node: BackendNode) => node.type === 'unloading')
        .map((node: BackendNode) => ({
          id: node.id,
          name: node.name,
          type: node.type,
          latitude: node.latitude,
          longitude: node.longitude,
          status: 1,
        }));
      otherNodes.value = nodesResponse.items
        .filter((node: BackendNode) => node.type !== 'shipping' && node.type !== 'unloading')
        .map((node: BackendNode) => ({
          id: node.id,
          name: node.name,
          type: node.type,
          latitude: node.latitude,
          longitude: node.longitude,
          status: 1,
        }));
    }
  } catch (error) {
    console.error('加载数据失败:', error);
    ElMessage.error('加载数据失败');

    // 保持数据为空，不使用默认数据
    vehicles.value = [];
    stats.value = {
      totalShipment: 0,
      totalUnload: 0,
      inTransit: 0,
      capacity: 0,
    };
    shippingNodes.value = [];
    unloadingNodes.value = [];
    otherNodes.value = [];
  }
};

// 面板控制方法
const toggleVehiclesPanel = () => {
  vehiclesPanelOpen.value = !vehiclesPanelOpen.value;
};

const toggleNodesPanel = () => {
  nodesPanelOpen.value = !nodesPanelOpen.value;
};

// 选择车辆
const selectVehicle = (vehicle: VehicleItem) => {
  selectedVehicle.value = vehicle;
};

// 工具函数

const _getStatusType = (status: number) => {
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

const getVehicleColor = (status: number) => {
  switch (status) {
    case 1:
      return '#67C23A';
    case 2:
      return '#909399';
    case 3:
      return '#F56C6C';
    default:
      return '#909399';
  }
};

// 初始化地图（含有限重试，避免无限循环）
const MAX_MAP_RETRIES = 50;
let mapRetryCount = 0;

const initMap = async () => {
  try {
    await nextTick();

    if (!mapContainer.value) {
      mapRetryCount++;
      if (mapRetryCount > MAX_MAP_RETRIES) {
        console.error(`地图容器重试超限(${MAX_MAP_RETRIES}次)，放弃初始化`);
        return;
      }
      console.error(`地图容器不存在(${mapRetryCount}/${MAX_MAP_RETRIES})，100ms后重试...`);
      setTimeout(() => {
        initMap();
      }, 100);
      return;
    }

    // 检查父容器尺寸
    const mapSection = mapContainer.value.parentElement;
    if (mapSection) {
      console.log('地图容器父元素尺寸:', mapSection.getBoundingClientRect());
      // 确保父容器有正确的尺寸
      mapSection.style.width = '100%';
      mapSection.style.height = '100%';
    }

    // 检查容器尺寸
    const checkContainerSize = () => {
      if (!mapContainer.value) return false;
      const rect = mapContainer.value.getBoundingClientRect();
      console.log('地图容器尺寸:', rect);
      return rect.width > 0 && rect.height > 0;
    };

    if (!checkContainerSize()) {
      mapRetryCount++;
      if (mapRetryCount > MAX_MAP_RETRIES) {
        console.error(`地图容器尺寸重试超限(${MAX_MAP_RETRIES}次)，使用固定尺寸继续`);
        mapContainer.value.style.width = '800px';
        mapContainer.value.style.height = '600px';
        // 继续执行，不返回
      } else {
        console.log(`地图容器尺寸为0(${mapRetryCount}/${MAX_MAP_RETRIES})，100ms后重试...`);
        mapContainer.value.style.width = '800px';
        mapContainer.value.style.height = '600px';
        setTimeout(() => {
          initMap();
        }, 100);
        return;
      }
    }

    // 重置重试计数（初始化成功）
    mapRetryCount = 0;

    // 确保地图容器有正确的尺寸
    mapContainer.value.style.width = '100%';
    mapContainer.value.style.height = '100%';
    mapContainer.value.innerHTML = '';

    let baseLayer: TileLayer;

    if (mapType.value === 'tianditu') {
      const vectorSource = new XYZ({
        url: getTiandituVecUrl(),
      });

      const vectorLayer = new TileLayer({
        source: vectorSource,
      });

      const labelSource = new XYZ({
        url: getTiandituCvaUrl(),
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
      drawNodes();
      loading.value = false;
    });
  } catch (error) {
    console.error('地图初始化失败:', error);
    loading.value = false;
    // 发生错误后，1000ms后重试
    setTimeout(() => {
      initMap();
    }, 1000);
  }
};

// 绘制装卸节点
const drawNodes = () => {
  if (!mapInstance.value) return;

  if (nodeLayer.value && mapInstance.value) {
    mapInstance.value.removeLayer(nodeLayer.value);
  }

  const vectorSource = new VectorSource();

  shippingNodes.value.forEach((node) => {
    if (!node.latitude || !node.longitude) return;

    const markerGeometry = new Point(fromLonLat([node.longitude, node.latitude]));
    const markerPoint = new Feature({
      geometry: markerGeometry,
      properties: { node: node, type: 'shipping' },
    });

    const markerStyle = new Style({
      image: new Circle({
        radius: 8,
        fill: new Fill({ color: '#409eff' }),
        stroke: new Stroke({ color: '#fff', width: 2 }),
      }),
    });

    markerPoint.setStyle(markerStyle);
    vectorSource.addFeature(markerPoint);
  });

  unloadingNodes.value.forEach((node) => {
    if (!node.latitude || !node.longitude) return;

    const markerGeometry = new Point(fromLonLat([node.longitude, node.latitude]));
    const markerPoint = new Feature({
      geometry: markerGeometry,
      properties: { node: node, type: 'unloading' },
    });

    const markerStyle = new Style({
      image: new Circle({
        radius: 8,
        fill: new Fill({ color: '#f56c6c' }),
        stroke: new Stroke({ color: '#fff', width: 2 }),
      }),
    });

    markerPoint.setStyle(markerStyle);
    vectorSource.addFeature(markerPoint);
  });

  otherNodes.value.forEach((node) => {
    if (!node.latitude || !node.longitude) return;

    const markerGeometry = new Point(fromLonLat([node.longitude, node.latitude]));
    const markerPoint = new Feature({
      geometry: markerGeometry,
      properties: { node: node, type: 'other' },
    });

    const markerStyle = new Style({
      image: new Circle({
        radius: 8,
        fill: new Fill({ color: '#909399' }),
        stroke: new Stroke({ color: '#fff', width: 2 }),
      }),
    });

    markerPoint.setStyle(markerStyle);
    vectorSource.addFeature(markerPoint);
  });

  nodeLayer.value = new VectorLayer({
    source: vectorSource,
    zIndex: 8,
  });

  if (nodeLayer.value && mapInstance.value) {
    mapInstance.value.addLayer(nodeLayer.value);
  }
};

// 定时器清理 - 防止内存泄漏
let refreshTimer: number | null = null;
let memoryMonitor: ReturnType<typeof MemoryMonitor.getInstance> | null = null;
const cleanupTimers = () => {
  if (refreshTimer !== null) {
    clearInterval(refreshTimer);
    refreshTimer = null;
  }
};

// 初始化内存监控
const initMemoryMonitor = () => {
  memoryMonitor = MemoryMonitor.getInstance();
  memoryMonitor.startMonitoring(10000); // 每10秒采样一次
};

// 生命周期
onMounted(async () => {
  await loadData();
  initMemoryMonitor();
  
  // 初始化地图
  await nextTick();
  initMap();
  
  // 添加窗口 resize 监听
  window.addEventListener('resize', handleResizeThrottled);
});

onUnmounted(() => {
  // 清理定时器
  cleanupTimers();
  
  // 停止内存监控
  if (memoryMonitor) {
    memoryMonitor.stopMonitoring();
    console.log('[RealTimeMonitor] 内存报告:', memoryMonitor.getReport());
  }
  
  // 移除窗口 resize 监听
  window.removeEventListener('resize', handleResizeThrottled);
  
  // 清理地图实例 - 防止内存泄漏
  if (mapInstance.value) {
    try {
      // 移除所有图层
      const layers = mapInstance.value.getLayers();
      if (layers) {
        layers.forEach((layer: any) => {
          mapInstance.value.removeLayer(layer);
        });
      }
      // 销毁地图实例
      mapInstance.value.dispose();
    } catch (e) {
      console.warn('[RealTimeMonitor] 清理地图实例时出错:', e);
    }
    mapInstance.value = null;
  }
  
  // 清理图层引用
  nodeLayer.value = null;
});

// 使用节流优化窗口 resize 处理
const handleResizeThrottled = throttle(() => {
  if (mapInstance.value) {
    mapInstance.value.updateSize();
  }
}, 100);

// 使用防抖优化数据加载
const loadDataDebounced = debounce(loadData, 500);

// 监听地图类型变化
watch(() => mapType.value, () => {
  loading.value = true;
  if (mapInstance.value) {
    mapInstance.value.dispose();
    mapInstance.value = null;
  }
  initMap();
});
</script>

<style scoped>
.realtime-monitor {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #f5f5f5;
}

.monitor-header {
  background: #fff;
  padding: 20px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.monitor-header h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.header-stats {
  display: flex;
  gap: 30px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 4px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 14px;
  color: #909399;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: #409eff;
}

.monitor-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}

/* 左侧控制工具栏 */
.control-sidebar {
  width: 320px;
  background: #fff;
  border-right: 1px solid #e4e7ed;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  min-height: 0;
}

.control-section {
  border-bottom: 1px solid #e4e7ed;
}

.section-header {
  padding: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  transition: background-color 0.3s;
}

.section-header:hover {
  background-color: #f8f9fa;
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.section-content {
  padding: 16px;
}

/* 运输载具列表 */
.vehicle-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 300px;
  overflow-y: auto;
}

.vehicle-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.3s;
}

.vehicle-item:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  border-color: #409eff;
}

.vehicle-item.selected {
  border-color: #409eff;
  box-shadow: 0 0 0 2px #409eff;
  background-color: #ecf5ff;
}

.vehicle-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
}

.vehicle-info {
  flex: 1;
  min-width: 0;
}

.vehicle-name {
  font-weight: 600;
  color: #303133;
  margin-bottom: 4px;
}

.vehicle-status {
  font-size: 12px;
  color: #909399;
}

/* 装卸节点列表 */
.node-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 300px;
  overflow-y: auto;
  margin-top: 10px;
}

.node-item {
  padding: 12px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  transition: all 0.3s;
}

.node-item:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.node-item.shipping-node {
  border-left: 4px solid #409eff;
}

.node-item.unloading-node {
  border-left: 4px solid #f56c6c;
}

.node-item.other-node {
  border-left: 4px solid #909399;
}

.node-name {
  font-weight: 600;
  color: #303133;
  margin-bottom: 4px;
}

.node-info {
  font-size: 12px;
  color: #606266;
  margin-bottom: 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.node-stats {
  font-size: 12px;
  color: #909399;
}

/* 右侧内容区域 */
.content-section {
  flex: 1;
  min-width: 0;
  min-height: 0;
  position: relative;
}

/* 右侧货运运输地图 */
.map-section {
  width: 100%;
  height: 100%;
  position: relative;
}

.map-type-switch {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 1000;
}

.map-container {
  width: 100%;
  height: 100%;
}

.map-loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #909399;
}

</style>


