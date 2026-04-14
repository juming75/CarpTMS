<template>
  <div class="map-container">
    <div ref="mapContainer" class="map-wrapper"></div>
    <div v-if="loading" class="map-loading">
      <el-icon class="is-loading"><loading /></el-icon>
      <span>地图加载中...</span>
    </div>

    <!-- 地图控制工具栏 -->
    <div class="map-controls">
      <el-button-group>
        <el-tooltip content="放大">
          <el-button :icon="ZoomIn" @click="zoomIn" />
        </el-tooltip>
        <el-tooltip content="缩小">
          <el-button :icon="ZoomOut" @click="zoomOut" />
        </el-tooltip>
        <el-tooltip content="重置视图">
          <el-button :icon="Refresh" @click="resetView" />
        </el-tooltip>
      </el-button-group>

      <el-button-group>
        <el-tooltip content="显示/隐藏车辆">
          <el-button :icon="Van" @click="toggleVehicles" />
        </el-tooltip>
        <el-tooltip content="显示/隐藏轨迹">
          <el-button :icon="Position" @click="toggleTracks" />
        </el-tooltip>
        <el-tooltip content="显示/隐藏区域">
          <el-button :icon="Location" @click="toggleZones" />
        </el-tooltip>
      </el-button-group>

      <el-button-group>
        <el-tooltip content="地图模式">
          <el-select v-model="mapType" placeholder="选择地图" @change="changeMapType">
            <el-option label="标准地图" value="normal" />
            <el-option label="卫星地图" value="satellite" />
            <el-option label="地形地图" value="terrain" />
          </el-select>
        </el-tooltip>
      </el-button-group>
    </div>

    <!-- 车辆信息弹窗 -->
    <el-dialog v-model="vehicleDialogVisible" title="车辆信息" width="500px" :close-on-click-modal="false">
      <div v-if="selectedVehicle" class="vehicle-info">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="车牌号">
            {{ selectedVehicle.licensePlate }}
          </el-descriptions-item>
          <el-descriptions-item label="车辆类型">
            {{ selectedVehicle.vehicleType }}
          </el-descriptions-item>
          <el-descriptions-item label="驾驶员">
            {{ selectedVehicle.driverName || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusType(selectedVehicle.status)">
              {{ getStatusText(selectedVehicle.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="速度"> {{ selectedVehicle.speed || 0 }} km/h </el-descriptions-item>
          <el-descriptions-item label="方向"> {{ selectedVehicle.direction || 0 }}° </el-descriptions-item>
          <el-descriptions-item label="经度">
            {{ selectedVehicle.longitude }}
          </el-descriptions-item>
          <el-descriptions-item label="纬度">
            {{ selectedVehicle.latitude }}
          </el-descriptions-item>
          <el-descriptions-item label="GPS时间">
            {{ formatDateTime(selectedVehicle.gpsTime) }}
          </el-descriptions-item>
          <el-descriptions-item label="更新时间">
            {{ formatDateTime(selectedVehicle.updateTime) }}
          </el-descriptions-item>
        </el-descriptions>

        <div class="vehicle-actions">
          <el-button type="primary" @click="viewTrack">查看轨迹</el-button>
          <el-button @click="closeDialog">关闭</el-button>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck - OpenLayers 类型定义不兼容，忽略类型检查
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { ZoomIn, ZoomOut, Refresh, Van, Position, Location } from '@element-plus/icons-vue';
// import type { Vehicle } from '@/types/vehicle' // 暂时注释，模块不存在

// OpenLayers 地图相关导入
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

// 类型定义
interface Vehicle {
  licensePlate: string;
  vehicleType: string;
  driverName?: string;
  status: number;
  speed?: number;
  direction?: number;
  longitude: number;
  latitude: number;
  gpsTime: string;
  updateTime: string;
  track?: Array<{ longitude: number; latitude: number; time: string }>;
}

interface MapNode {
  longitude: number;
  latitude: number;
  name?: string;
  type?: string;
}

// Props
interface Props {
  vehicles?: Vehicle[];
  shippingNodes?: MapNode[];
  unloadingNodes?: MapNode[];
  otherNodes?: MapNode[];
  showVehicles?: boolean;
  showTracks?: boolean;
  showZones?: boolean;
  center?: [number, number];
  zoom?: number;
}

const props = withDefaults(defineProps<Props>(), {
  vehicles: () => [],
  shippingNodes: () => [],
  unloadingNodes: () => [],
  otherNodes: () => [],
  showVehicles: true,
  showTracks: false,
  showZones: false,
  center: () => [116.404, 39.915],
  zoom: 12,
});

// Emits
const emit = defineEmits<{
  vehicleClick: [vehicle: Vehicle];
  mapReady: [];
  'update:showVehicles': [value: boolean];
  'update:showTracks': [value: boolean];
  'update:showZones': [value: boolean];
  viewTrack: [vehicle: Vehicle];
}>();

// Refs
const mapContainer = ref<HTMLElement | null>(null);
const loading = ref(true);
const mapType = ref('normal');
const mapInstance = ref<Map | null>(null);
const vehicleLayer = ref<VectorLayer | null>(null);
const trackLayer = ref<VectorLayer | null>(null);
const zoneLayer = ref<VectorLayer | null>(null);
const nodeLayer = ref<VectorLayer | null>(null);
const markers = ref<Feature[]>([]);
const vehicleDialogVisible = ref(false);
const selectedVehicle = ref<Vehicle | null>(null);

// 初始化地图
const initMap = () => {
  console.log('开始初始化地图...');

  if (!mapContainer.value) {
    console.error('地图容器不存在');
    loading.value = false;
    return;
  }

  // 确保地图容器有正确的尺寸
  const checkContainerSize = () => {
    if (!mapContainer.value) return false;

    const rect = mapContainer.value.getBoundingClientRect();
    console.log('Map container size:', rect.width, 'x', rect.height);

    if (rect.width > 0 && rect.height > 0) {
      return true;
    }
    return false;
  };

  // 检查容器尺寸，如果尺寸为0，等待一段时间后重试
  if (!checkContainerSize()) {
    console.warn('Map container has 0 size, waiting for resize...');
    setTimeout(() => {
      initMap();
    }, 100);
    return;
  }

  try {
    // 从localStorage获取天地图API Key
    const tiandituKey = localStorage.getItem('tiandituKey') || '34d8cf060f7e8ac09be79b9261d65274';
    console.log('使用天地图API Key:', tiandituKey);

    // 清空地图容器
    mapContainer.value.innerHTML = '';

    // 创建天地图矢量图层
    const vectorSource = new XYZ({
      url: `https://t0.tianditu.gov.cn/vec_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=vec&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${tiandituKey}`,
      crossOrigin: 'anonymous',
    });

    const vectorLayer = new TileLayer({
      source: vectorSource,
    });

    // 创建天地图矢量注记图层
    const labelSource = new XYZ({
      url: `https://t0.tianditu.gov.cn/cva_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=cva&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${tiandituKey}`,
      crossOrigin: 'anonymous',
    });

    const labelLayer = new TileLayer({
      source: labelSource,
    });

    // 计算地图中心点（中国中心位置）
    const centerPoint = fromLonLat([104.195, 35.861]);
    console.log('地图中心点:', centerPoint);

    // 创建地图实例
    console.log('创建地图实例...');
    const map = new Map({
      target: mapContainer.value,
      layers: [vectorLayer, labelLayer],
      view: new View({
        center: centerPoint,
        zoom: 5,
        maxZoom: 18,
        minZoom: 2,
      }),
    });

    console.log('地图实例创建成功');
    mapInstance.value = map;

    // 地图加载完成后添加节点
    map.on('rendercomplete', () => {
      console.log('地图渲染完成');
      drawNodes();
      loading.value = false;
      emit('mapReady');
    });
  } catch (error) {
    console.error('天地图初始化失败:', error);
    // 显示错误提示
    if (mapContainer.value) {
      mapContainer.value.innerHTML = `
        <div style="width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;">
          <strong>天地图初始化失败</strong>
          <br>
          <small>请检查网络连接或API Key配置</small>
          <br>
          <small>错误信息: ${error.message}</small>
        </div>
      `;
    }
    loading.value = false;
  }
};

// 获取地图样式

const _getMapStyle = () => {
  switch (mapType.value) {
    case 'satellite':
      return 'amap://styles/whitesmoke?style=6';
    case 'terrain':
      return 'amap://styles/darkblue?style=8';
    default:
      return 'normal';
  }
};

// 创建车辆标记
const createVehicleMarkers = () => {
  if (!mapInstance.value || !props.showVehicles) return;

  // 移除旧的车辆图层
  if (vehicleLayer.value) {
    mapInstance.value.removeLayer(vehicleLayer.value);
  }

  // 创建新的矢量源
  const vectorSource = new VectorSource();

  props.vehicles.forEach((vehicle) => {
    if (!vehicle.latitude || !vehicle.longitude) return;

    try {
      const markerGeometry = new Point(fromLonLat([vehicle.longitude, vehicle.latitude]));
      const markerPoint = new Feature({
        geometry: markerGeometry,
        properties: {
          vehicle: vehicle,
        },
      });

      // 设置标记样式
      const markerStyle = new Style({
        image: new Circle({
          radius: 8,
          fill: new Fill({
            color: getVehicleColor(vehicle.status),
          }),
          stroke: new Stroke({
            color: '#fff',
            width: 2,
          }),
        }),
      });

      markerPoint.setStyle(markerStyle);
      vectorSource.addFeature(markerPoint);
      markers.value.push(markerPoint);
    } catch (error) {
      console.error('创建车辆标记失败:', error);
    }
  });

  // 创建车辆图层
  vehicleLayer.value = new VectorLayer({
    source: vectorSource,
    zIndex: 10,
  });

  mapInstance.value.addLayer(vehicleLayer.value);

  // 添加点击事件
  mapInstance.value.on('click', (event: { pixel: number[] }) => {
    const features = mapInstance.value?.getFeaturesAtPixel(event.pixel);
    if (features && features.length > 0) {
      const feature = features[0];
      const vehicle = feature.get('vehicle') as Vehicle;
      if (vehicle) {
        selectedVehicle.value = vehicle;
        vehicleDialogVisible.value = true;
        emit('vehicleClick', vehicle);
      }
    }
  });
};

// 获取车辆颜色
const getVehicleColor = (status: number) => {
  const statusColors: Record<number, string> = {
    1: '#67C23A', // 在线
    2: '#E6A23C', // 离线
    3: '#F56C6C', // 报警
    4: '#909399', // 未知
  };
  return statusColors[status] || '#909399';
};

// 绘制轨迹
const drawTracks = () => {
  if (!mapInstance.value || !props.showTracks) return;

  // 移除旧的轨迹图层
  if (trackLayer.value) {
    mapInstance.value.removeLayer(trackLayer.value);
  }

  // 创建新的矢量源
  const vectorSource = new VectorSource();

  // 为每个车辆绘制轨迹
  props.vehicles.forEach((vehicle) => {
    if (!vehicle.track || vehicle.track.length < 2) return;

    try {
      // 这里可以添加轨迹绘制逻辑
      // 例如使用 LineString 来绘制轨迹
    } catch (error) {
      console.error('绘制轨迹失败:', error);
    }
  });

  // 创建轨迹图层
  trackLayer.value = new VectorLayer({
    source: vectorSource,
    zIndex: 5,
  });

  mapInstance.value.addLayer(trackLayer.value);
};

// 绘制区域
const drawZones = () => {
  if (!mapInstance.value || !props.showZones) return;

  // 移除旧的区域图层
  if (zoneLayer.value) {
    mapInstance.value.removeLayer(zoneLayer.value);
  }

  // 创建新的矢量源
  const vectorSource = new VectorSource();

  // 示例：绘制禁行区域
  const zones = [{ center: [116.404, 39.915], radius: 500, color: '#F56C6C' }];

  zones.forEach((_zone) => {
    try {
      // 这里可以添加区域绘制逻辑
      // 例如使用 Circle 来绘制区域
    } catch (error) {
      console.error('绘制区域失败:', error);
    }
  });

  // 创建区域图层
  zoneLayer.value = new VectorLayer({
    source: vectorSource,
    zIndex: 3,
  });

  mapInstance.value.addLayer(zoneLayer.value);
};

// 绘制装卸节点
const drawNodes = () => {
  if (!mapInstance.value) return;

  // 移除旧的节点图层
  if (nodeLayer.value) {
    mapInstance.value.removeLayer(nodeLayer.value);
  }

  // 创建新的矢量源
  const vectorSource = new VectorSource();

  // 绘制发货节点（蓝点）
  props.shippingNodes.forEach((node) => {
    if (!node.latitude || !node.longitude) return;

    try {
      const markerGeometry = new Point(fromLonLat([node.longitude, node.latitude]));
      const markerPoint = new Feature({
        geometry: markerGeometry,
        properties: {
          node: node,
          type: 'shipping',
        },
      });

      // 设置标记样式（蓝点）
      const markerStyle = new Style({
        image: new Circle({
          radius: 8,
          fill: new Fill({ color: '#409eff' }),
          stroke: new Stroke({ color: '#fff', width: 2 }),
        }),
      });

      markerPoint.setStyle(markerStyle);
      vectorSource.addFeature(markerPoint);
    } catch (error) {
      console.error('绘制发货节点失败:', error);
    }
  });

  // 绘制卸货节点（鲜红色）
  props.unloadingNodes.forEach((node) => {
    if (!node.latitude || !node.longitude) return;

    try {
      const markerGeometry = new Point(fromLonLat([node.longitude, node.latitude]));
      const markerPoint = new Feature({
        geometry: markerGeometry,
        properties: {
          node: node,
          type: 'unloading',
        },
      });

      // 设置标记样式（鲜红色）
      const markerStyle = new Style({
        image: new Circle({
          radius: 8,
          fill: new Fill({ color: '#f56c6c' }),
          stroke: new Stroke({ color: '#fff', width: 2 }),
        }),
      });

      markerPoint.setStyle(markerStyle);
      vectorSource.addFeature(markerPoint);
    } catch (error) {
      console.error('绘制卸货节点失败:', error);
    }
  });

  // 绘制其他装卸位置（紫色）
  props.otherNodes.forEach((node) => {
    if (!node.latitude || !node.longitude) return;

    try {
      const markerGeometry = new Point(fromLonLat([node.longitude, node.latitude]));
      const markerPoint = new Feature({
        geometry: markerGeometry,
        properties: {
          node: node,
          type: 'other',
        },
      });

      // 设置标记样式（紫色）
      const markerStyle = new Style({
        image: new Circle({
          radius: 8,
          fill: new Fill({ color: '#909399' }),
          stroke: new Stroke({ color: '#fff', width: 2 }),
        }),
      });

      markerPoint.setStyle(markerStyle);
      vectorSource.addFeature(markerPoint);
    } catch (error) {
      console.error('绘制其他装卸位置失败:', error);
    }
  });

  // 绘制节点之间的虚线连接
  if (props.shippingNodes.length > 0 && props.unloadingNodes.length > 0) {
    // 这里可以添加节点之间的连接逻辑
    // 例如使用 LineString 来绘制虚线连接
  }

  // 创建节点图层
  nodeLayer.value = new VectorLayer({
    source: vectorSource,
    zIndex: 8,
  });

  mapInstance.value.addLayer(nodeLayer.value);
};

// 地图控制
const zoomIn = () => {
  const view = mapInstance.value?.getView();
  if (view) {
    const currentZoom = view.getZoom();
    if (currentZoom) {
      view.setZoom(currentZoom + 1);
    }
  }
};

const zoomOut = () => {
  const view = mapInstance.value?.getView();
  if (view) {
    const currentZoom = view.getZoom();
    if (currentZoom) {
      view.setZoom(currentZoom - 1);
    }
  }
};

const resetView = () => {
  const view = mapInstance.value?.getView();
  if (view) {
    view.setCenter(fromLonLat(props.center));
    view.setZoom(props.zoom);
  }
};

const changeMapType = (_type: string) => {
  // 这里可以添加地图类型切换逻辑
  // 例如切换不同的底图图层
};

const toggleVehicles = () => {
  emit('update:showVehicles', !props.showVehicles);
};

const toggleTracks = () => {
  emit('update:showTracks', !props.showTracks);
};

const toggleZones = () => {
  emit('update:showZones', !props.showZones);
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

const formatDateTime = (datetime: string) => {
  if (!datetime) return '-';
  return new Date(datetime).toLocaleString('zh-CN');
};

const viewTrack = () => {
  if (selectedVehicle.value) {
    emit('viewTrack', selectedVehicle.value);
  }
};

const closeDialog = () => {
  vehicleDialogVisible.value = false;
  selectedVehicle.value = null;
};

// 监听 props 变化
watch(
  () => props.vehicles,
  () => {
    createVehicleMarkers();
  },
  { deep: true }
);

watch(
  () => props.shippingNodes,
  () => {
    drawNodes();
  },
  { deep: true }
);

watch(
  () => props.unloadingNodes,
  () => {
    drawNodes();
  },
  { deep: true }
);

watch(
  () => props.otherNodes,
  () => {
    drawNodes();
  },
  { deep: true }
);

watch(
  () => props.showVehicles,
  (newVal) => {
    if (newVal) {
      createVehicleMarkers();
    } else if (vehicleLayer.value) {
      mapInstance.value?.removeLayer(vehicleLayer.value);
      vehicleLayer.value = null;
    }
  }
);

watch(
  () => props.showTracks,
  (newVal) => {
    if (newVal) {
      drawTracks();
    } else if (trackLayer.value) {
      mapInstance.value?.removeLayer(trackLayer.value);
      trackLayer.value = null;
    }
  }
);

watch(
  () => props.showZones,
  (newVal) => {
    if (newVal) {
      drawZones();
    } else if (zoneLayer.value) {
      mapInstance.value?.removeLayer(zoneLayer.value);
      zoneLayer.value = null;
    }
  }
);

// 生命周期
onMounted(() => {
  // 初始化地图
  initMap();
});

onUnmounted(() => {
  // 清理地图实例
  if (vehicleLayer.value) {
    mapInstance.value?.removeLayer(vehicleLayer.value);
  }
  if (trackLayer.value) {
    mapInstance.value?.removeLayer(trackLayer.value);
  }
  if (zoneLayer.value) {
    mapInstance.value?.removeLayer(zoneLayer.value);
  }
  if (nodeLayer.value) {
    mapInstance.value?.removeLayer(nodeLayer.value);
  }
  mapInstance.value?.dispose();
});
</script>

<style scoped>
.map-container {
  position: relative;
  width: 100%;
  height: 100%;
}

.map-wrapper {
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

.map-controls {
  position: absolute;
  top: 10px;
  right: 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  z-index: 1000;
}

.vehicle-info {
  padding: 0 10px;
}

.vehicle-actions {
  display: flex;
  justify-content: center;
  gap: 10px;
  margin-top: 20px;
}
</style>


