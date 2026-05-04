<template>
  <div class="track-playback-player">
    <!-- 控制面板 -->
    <div class="control-panel">
      <el-card>
        <template #header>
          <div class="panel-header">
            <span>轨迹回放</span>
            <div class="panel-actions">
              <el-button
                size="small"
                @click="handleStart"
                :disabled="trackData.length === 0 || isPlaying"
              >
                <el-icon><VideoPlay /></el-icon>
                开始回放
              </el-button>
              <el-button
                size="small"
                @click="handlePause"
                :disabled="!isPlaying"
              >
                <el-icon><VideoPause /></el-icon>
                暂停
              </el-button>
              <el-button
                size="small"
                @click="handleStop"
              >
                <el-icon><VideoCamera /></el-icon>
                停止
              </el-button>
              <el-button
                size="small"
                @click="handleCenter"
              >
                <el-icon><Aim /></el-icon>
                居中
              </el-button>
              <el-button
                size="small"
                @click="handleClear"
              >
                <el-icon><Delete /></el-icon>
                清空
              </el-button>
              <span class="speed-label">速度:</span>
              <el-slider
                v-model="playbackSpeed"
                :min="1"
                :max="10"
                :step="1"
                class="speed-slider"
              />
            </div>
          </div>
        </template>

        <!-- 查询表单 -->
        <el-form :model="queryForm" label-width="100px" inline size="small">
          <el-form-item label="车辆选择">
            <el-select
              v-model="queryForm.vehicleId"
              filterable
              placeholder="请选择车辆"
              style="width: 180px"
              clearable
            >
              <el-option
                v-for="vehicle in vehicles"
                :key="vehicle.id"
                :label="vehicle.license_plate"
                :value="vehicle.id"
              />
            </el-select>
          </el-form-item>
          <el-form-item label="开始时间">
            <el-date-picker
              v-model="queryForm.startTime"
              type="datetime"
              placeholder="选择开始时间"
              style="width: 180px"
            />
          </el-form-item>
          <el-form-item label="结束时间">
            <el-date-picker
              v-model="queryForm.endTime"
              type="datetime"
              placeholder="选择结束时间"
              style="width: 180px"
            />
          </el-form-item>
          <el-form-item>
            <el-button type="primary" @click="handleQuery">查询</el-button>
            <el-button @click="handleReset">重置</el-button>
          </el-form-item>
        </el-form>

        <!-- 状态信息 -->
        <div class="status-info">
          <el-tag>
            轨迹点数: <strong>{{ trackData.length }}</strong>
          </el-tag>
          <el-tag v-if="trackData.length > 0">
            当前位置: <strong>{{ currentIndex + 1 }}/{{ trackData.length }}</strong>
          </el-tag>
          <el-tag v-if="isPlaying" type="success">
            播放中
          </el-tag>
          <el-tag v-else type="info">
            已暂停
          </el-tag>
        </div>
      </el-card>
    </div>

    <!-- 地图容器 -->
    <div class="map-container">
      <!-- 地图工具栏 -->
      <div class="map-toolbar">
        <el-radio-group v-model="mapType" size="small" @change="handleMapTypeChange">
          <el-radio-button value="tianditu">天地图</el-radio-button>
          <el-radio-button value="local">本地地图</el-radio-button>
        </el-radio-group>
      </div>
      <div id="player-map" class="map"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, nextTick } from 'vue';
import { ElMessage } from 'element-plus';
import {
  VideoPlay,
  VideoPause,
  VideoCamera,
  Aim,
  Delete
} from '@element-plus/icons-vue';
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
import Icon from 'ol/style/Icon';
import { getTiandituVecUrl, getTiandituCvaUrl } from '@/utils/mapConfig';
import api from '@/api';

// ================ 状态定义 ================
const queryForm = reactive({
  vehicleId: '',
  startTime: '',
  endTime: ''
});

const vehicles = ref<any[]>([]);
const trackData = ref<any[]>([]);
const playbackSpeed = ref(5);
const isPlaying = ref(false);
const currentIndex = ref(0);

// ================ 地图相关 ================
let map: any = null;
let polylineLayer: any = null;
let markerLayer: any = null;
let playbackTimer: number | null = null;
const mapType = ref('tianditu');

// ================ 初始化 ================
const initMap = () => {
  let baseLayer: any;

  if (mapType.value === 'tianditu') {
    // 底图图层 - 矢量底图
    const vecLayer = new TileLayer({
      source: new XYZ({
        url: getTiandituVecUrl(),
        projection: 'EPSG:3857',
        attributions: '天地图'
      })
    });

    // 注记图层
    const cvaLayer = new TileLayer({
      source: new XYZ({
        url: getTiandituCvaUrl(),
        projection: 'EPSG:3857'
      })
    });

    baseLayer = vecLayer;
    baseLayer.set('labelLayer', cvaLayer);
  } else {
    // 本地地图 - 使用高德瓦片服务（与 RealTimeMonitor 保持一致）
    baseLayer = new TileLayer({
      source: new XYZ({
        url: 'https://webrd01.is.autonavi.com/appmaptile?lang=zh_cn&size=1&scale=1&style=8&x={x}&y={y}&z={z}',
        crossOrigin: 'anonymous',
      })
    });
  }

  // 轨迹线图层
  polylineLayer = new VectorLayer({
    source: new VectorSource(),
    style: new Style({
      stroke: new Stroke({
        color: '#409EFF',
        width: 3
      })
    })
  });

  // 标记点图层
  markerLayer = new VectorLayer({
    source: new VectorSource(),
    style: new Style({
      image: new Circle({
        radius: 6,
        fill: new Fill({ color: '#F56C6C' }),
        stroke: new Stroke({
          color: '#fff',
          width: 2
        })
      })
    })
  });

  const layers = [baseLayer, polylineLayer, markerLayer];
  if (baseLayer.get('labelLayer')) {
    layers.splice(1, 0, baseLayer.get('labelLayer'));
  }

  map = new Map({
    target: 'player-map',
    layers: layers,
    view: new View({
      center: fromLonLat([116.3974, 39.9093]), // 北京
      zoom: 12,
      minZoom: 3,
      maxZoom: 18
    })
  });
};

const handleMapTypeChange = (type: string) => {
  if (map) {
    map.setTarget(undefined);
    map = null;
  }
  mapType.value = type;
  initMap();
  if (trackData.value.length > 0) {
    drawTrack();
  }
};

const loadVehicles = async () => {
  try {
    const response = await api.get('/api/vehicles');
    vehicles.value = response.data || [];
  } catch (error) {
    console.error('加载车辆列表失败:', error);
  }
};

// ================ 查询处理 ================
const handleQuery = async () => {
  if (!queryForm.vehicleId) {
    ElMessage.warning('请选择车辆');
    return;
  }

  try {
    const response = await api.get('/api/track/query', {
      params: {
        vehicleId: queryForm.vehicleId,
        startTime: queryForm.startTime,
        endTime: queryForm.endTime
      }
    });

    trackData.value = response.data || [];
    
    if (trackData.value.length === 0) {
      ElMessage.warning('未找到轨迹数据');
      return;
    }

    // 绘制轨迹
    drawTrack();
    
    // 居中地图
    handleCenter();
    
    ElMessage.success(`查询成功，共 ${trackData.value.length} 个点位`);
  } catch (error) {
    console.error('查询轨迹失败:', error);
    ElMessage.error('查询轨迹失败');
  }
};

const handleReset = () => {
  queryForm.vehicleId = '';
  queryForm.startTime = '';
  queryForm.endTime = '';
  trackData.value = [];
  currentIndex.value = 0;
  isPlaying.value = false;
  if (playbackTimer) {
    clearInterval(playbackTimer);
    playbackTimer = null;
  }
  clearMap();
};

// ================ 地图绘制 ================
const drawTrack = () => {
  if (!polylineLayer) return;

  const source = polylineLayer.getSource();
  source.clear();

  if (trackData.value.length < 2) return;

  // 绘制轨迹线
  const coordinates = trackData.value.map(point => 
    fromLonLat([point.longitude, point.latitude])
  );

  const lineFeature = new Feature({
    geometry: new LineString(coordinates)
  });

  source.addFeature(lineFeature);

  // 绘制起点标记
  const startMarker = new Feature({
    geometry: new Point(coordinates[0])
  });
  startMarker.setStyle(new Style({
    image: new Circle({
      radius: 8,
      fill: new Fill({ color: '#67C23A' }),
      stroke: new Stroke({ color: '#fff', width: 2 })
    })
  }));
  source.addFeature(startMarker);

  // 绘制终点标记
  const endMarker = new Feature({
    geometry: new Point(coordinates[coordinates.length - 1])
  });
  endMarker.setStyle(new Style({
    image: new Circle({
      radius: 8,
      fill: new Fill({ color: '#F56C6C' }),
      stroke: new Stroke({ color: '#fff', width: 2 })
    })
  }));
  source.addFeature(endMarker);
};

const updateMarker = () => {
  if (!markerLayer || trackData.value.length === 0) return;

  const source = markerLayer.getSource();
  source.clear();

  const point = trackData.value[currentIndex.value];
  const coordinate = fromLonLat([point.longitude, point.latitude]);

  const marker = new Feature({
    geometry: new Point(coordinate)
  });

  marker.setStyle(new Style({
    image: new Circle({
      radius: 7,
      fill: new Fill({ color: '#409EFF' }),
      stroke: new Stroke({ color: '#fff', width: 3 })
    })
  }));

  source.addFeature(marker);

  // 居中到当前位置
  map.getView().animate({
    center: coordinate,
    duration: 300
  });
};

const clearMap = () => {
  if (polylineLayer) {
    polylineLayer.getSource().clear();
  }
  if (markerLayer) {
    markerLayer.getSource().clear();
  }
};

// ================ 回放控制 ================
const handleStart = () => {
  if (trackData.value.length === 0) {
    ElMessage.warning('请先查询轨迹数据');
    return;
  }

  isPlaying.value = true;

  const interval = 1000 / playbackSpeed.value;

  playbackTimer = window.setInterval(() => {
    currentIndex.value++;

    if (currentIndex.value >= trackData.value.length) {
      handleStop();
      ElMessage.success('轨迹回放完成');
      return;
    }

    updateMarker();
  }, interval);
};

const handlePause = () => {
  isPlaying.value = false;
  if (playbackTimer) {
    clearInterval(playbackTimer);
    playbackTimer = null;
  }
};

const handleStop = () => {
  isPlaying.value = false;
  currentIndex.value = 0;
  if (playbackTimer) {
    clearInterval(playbackTimer);
    playbackTimer = null;
  }
  clearMap();
  if (trackData.value.length > 0) {
    drawTrack();
  }
};

const handleCenter = () => {
  if (trackData.value.length === 0) return;

  const coordinates = trackData.value.map(point => 
    fromLonLat([point.longitude, point.latitude])
  );

  const lineString = new LineString(coordinates);
  const extent = lineString.getExtent();

  map.getView().fit(extent, {
    padding: [50, 50, 50, 50],
    duration: 500
  });
};

const handleClear = () => {
  trackData.value = [];
  currentIndex.value = 0;
  isPlaying.value = false;
  if (playbackTimer) {
    clearInterval(playbackTimer);
    playbackTimer = null;
  }
  clearMap();
};

// ================ 生命周期 ================
onMounted(async () => {
  await nextTick();
  initMap();
  await loadVehicles();
});

onUnmounted(() => {
  if (playbackTimer) {
    clearInterval(playbackTimer);
  }
  if (map) {
    map.setTarget(undefined);
    map = null;
  }
});
</script>

<style scoped>
.track-playback-player {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 16px;
}

.control-panel {
  flex-shrink: 0;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.panel-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.speed-label {
  margin-left: 16px;
  margin-right: 8px;
  color: #606266;
  font-size: 14px;
}

.speed-slider {
  width: 120px;
  display: inline-block;
  vertical-align: middle;
}

.status-info {
  display: flex;
  gap: 12px;
  margin-top: 16px;
}

.map-container {
  flex: 1;
  min-height: 0;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
  position: relative;
}

.map-toolbar {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 10;
  background: rgba(255, 255, 255, 0.95);
  padding: 6px 12px;
  border-radius: 4px;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.map {
  width: 100%;
  height: 100%;
}
</style>
