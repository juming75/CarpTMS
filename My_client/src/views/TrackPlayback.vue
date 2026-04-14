<template>
  <div class="track-playback-container">
    <!-- 页面标题 -->
    <el-page-header @back="$router.back()" content="轨迹回放"></el-page-header>
    
    <!-- 回放控制区 -->
    <el-card class="playback-controls">
      <template #header>
        <div class="card-header">
          <span>回放控制</span>
        </div>
      </template>
      
      <el-form :model="playbackForm" label-width="100px" size="small">
        <el-form-item label="选择车辆">
          <el-select v-model="playbackForm.vehicleId" placeholder="请选择车辆" style="width: 200px;">
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
            v-model="playbackForm.startTime"
            type="datetime"
            placeholder="选择开始时间"
            style="width: 200px;"
          ></el-date-picker>
        </el-form-item>
        
        <el-form-item label="结束时间">
          <el-date-picker
            v-model="playbackForm.endTime"
            type="datetime"
            placeholder="选择结束时间"
            style="width: 200px;"
          ></el-date-picker>
        </el-form-item>
        
        <el-form-item label="播放速度">
          <el-slider v-model="playbackForm.speed" :min="1" :max="10" :step="1"></el-slider>
        </el-form-item>
        
        <el-form-item>
          <el-button type="primary" @click="startPlayback" :disabled="!playbackForm.vehicleId || !playbackForm.startTime || !playbackForm.endTime">
            开始回放
          </el-button>
          <el-button @click="pausePlayback">暂停</el-button>
          <el-button @click="stopPlayback">停止</el-button>
        </el-form-item>
      </el-form>
    </el-card>
    
    <!-- 地图容器 -->
    <el-card class="map-container">
      <template #header>
        <div class="card-header">
          <span>轨迹地图</span>
          <div class="map-controls">
            <el-button size="small" @click="centerMap">居中地图</el-button>
            <el-button size="small" @click="clearMap">清空轨迹</el-button>
          </div>
        </div>
      </template>
      
      <div id="playback-map" class="map"></div>
    </el-card>
    
    <!-- 轨迹数据面板 -->
    <el-card class="track-data">
      <template #header>
        <div class="card-header">
          <span>轨迹数据</span>
        </div>
      </template>
      
      <el-table :data="trackData" style="width: 100%;" size="small">
        <el-table-column prop="time" label="时间" width="180"></el-table-column>
        <el-table-column prop="latitude" label="纬度" width="120"></el-table-column>
        <el-table-column prop="longitude" label="经度" width="120"></el-table-column>
        <el-table-column prop="speed" label="速度(km/h)" width="100"></el-table-column>
        <el-table-column prop="direction" label="方向" width="100"></el-table-column>
        <el-table-column prop="status" label="状态"></el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue';
import { ElMessage } from 'element-plus';
import api from '@/api';
import Map from 'ol/Map';
import View from 'ol/View';
import TileLayer from 'ol/layer/Tile';
import VectorLayer from 'ol/layer/Vector';
import VectorSource from 'ol/source/Vector';
import OSM from 'ol/source/OSM';
import XYZ from 'ol/source/XYZ';
import Feature from 'ol/Feature';
import Point from 'ol/geom/Point';
import LineString from 'ol/geom/LineString';
import { fromLonLat } from 'ol/proj';

import Style from 'ol/style/Style';
import Stroke from 'ol/style/Stroke';

// 车辆列表
const vehicles = ref<any[]>([]);

// 回放表单
const playbackForm = ref({
  vehicleId: '',
  startTime: '',
  endTime: '',
  speed: 5
});

// 地图实例
let map: any = null;
// 轨迹线实例
let polylineLayer: any = null;
// 标记点实例
let markerLayer: any = null;
// 回放定时器
let playbackTimer: number | null = null;
// 当前播放索引
let currentIndex = 0;
// 轨迹数据
const trackData = ref<any[]>([]);

// 初始化地图
const initMap = async () => {
  try {
    // 确保地图容器已经渲染
    await nextTick();
    
    // 检查地图容器是否存在
    const mapContainer = document.getElementById('playback-map');
    if (!mapContainer) {
      throw new Error('地图容器不存在');
    }
    
    // 强制设置容器样式，确保它能获取到尺寸
    mapContainer.style.width = '100%';
    mapContainer.style.height = '500px';
    mapContainer.style.minWidth = '300px';
    mapContainer.style.minHeight = '300px';
    mapContainer.style.display = 'block';
    
    // 检查地图容器的宽度和高度
    const width = mapContainer.clientWidth;
    const height = mapContainer.clientHeight;
    
    console.log('轨迹回放地图容器初始尺寸:', width, 'x', height);
    
    if (width > 0 && height > 0) {
      // 容器尺寸正常，初始化地图
      console.log('容器尺寸正常，开始初始化地图');
      createMapInstance(mapContainer);
    } else {
      console.warn('轨迹回放地图容器宽度或高度为0，使用ResizeObserver监听尺寸变化');
      
      // 使用ResizeObserver监听容器尺寸变化
      const observer = new ResizeObserver((entries) => {
        const entry = entries[0];
        const { width, height } = entry.contentRect;
        
        console.log('ResizeObserver捕获到尺寸变化:', width, 'x', height);
        
        if (width > 0 && height > 0) {
          console.log('轨迹回放地图容器尺寸已更新，开始初始化地图:', width, 'x', height);
          // 先断开观察，避免无限循环
          observer.disconnect();
          // 使用setTimeout避免在ResizeObserver回调中直接修改DOM
          setTimeout(() => {
            createMapInstance(mapContainer);
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
        
        console.log(`重试初始化轨迹回放地图 (${retryCount}/${maxRetries}):`, currentWidth, 'x', currentHeight);
        
        if (currentWidth > 0 && currentHeight > 0) {
          clearInterval(retryTimer);
          observer.disconnect();
          console.log('重试成功，使用当前尺寸初始化地图:', currentWidth, 'x', currentHeight);
          createMapInstance(mapContainer);
        } else if (retryCount >= maxRetries) {
          clearInterval(retryTimer);
          observer.disconnect();
          console.error('轨迹回放地图容器尺寸一直为0，放弃初始化');
          // 显示错误提示
          if (mapContainer) {
            mapContainer.innerHTML = '';
            const errorDiv = document.createElement('div');
            errorDiv.style.cssText = 'width: 100%; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;';
            errorDiv.innerHTML = '<strong>地图容器初始化失败</strong><br><small>容器尺寸为0，请检查页面布局</small>';
            mapContainer.appendChild(errorDiv);
          }
        }
      }, retryInterval);
    }
  } catch (error) {
    console.error('轨迹回放地图初始化失败:', error);
    ElMessage.error('轨迹回放地图初始化失败');
  }
};

// 实际创建地图实例的函数
const createMapInstance = (mapContainer: HTMLElement) => {
  try {
    // 清空地图容器
    mapContainer.innerHTML = '';
    
    // 确保容器有正确的样式
    mapContainer.style.width = '100%';
    mapContainer.style.height = '500px';
    mapContainer.style.position = 'relative';
    mapContainer.style.overflow = 'hidden';
    
    // 显示加载状态
    const loadingDiv = document.createElement('div');
    loadingDiv.id = 'map-loading';
    loadingDiv.style.cssText = 'position: absolute; top: 0; left: 0; width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: rgba(255, 255, 255, 0.8); z-index: 1000;';
    loadingDiv.innerHTML = '<div style="text-align: center;"><div style="font-size: 24px; color: #409eff; margin-bottom: 10px;">加载地图中...</div><div style="width: 40px; height: 40px; border: 4px solid #f3f3f3; border-top: 4px solid #409eff; border-radius: 50%; animation: spin 1s linear infinite; margin: 0 auto;"></div></div>';
    mapContainer.appendChild(loadingDiv);
    
    // 强制使用天地图，确保地图能够加载
    const mapType = 'tianditu';
    console.log('轨迹回放地图类型:', mapType);
    
    // 准备地图图层
    let layers: any[] = [];
    
    if (mapType === 'tianditu') {
      // 天地图实现
      console.log('开始初始化天地图');
      const tiandituKey = '34d8cf060f7e8ac09be79b9261d65274'; // 浏览器端密钥
      
      // 创建天地图矢量图层
      const vectorSource = new XYZ({
        url: `https://t0.tianditu.gov.cn/vec_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=vec&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${tiandituKey}`,
        crossOrigin: 'anonymous',
        projection: 'EPSG:3857'
      });
      
      const vectorLayer = new TileLayer({
        source: vectorSource,
        zIndex: 1
      });
      
      // 创建天地图矢量注记图层
      const labelSource = new XYZ({
        url: `https://t0.tianditu.gov.cn/cva_w/wmts?service=WMTS&request=GetTile&version=1.0.0&LAYER=cva&tileMatrixSet=w&TileMatrix={z}&TileRow={y}&TileCol={x}&style=default&format=tiles&tk=${tiandituKey}`,
        crossOrigin: 'anonymous',
        projection: 'EPSG:3857'
      });
      
      const labelLayer = new TileLayer({
        source: labelSource,
        zIndex: 2
      });
      
      layers = [vectorLayer, labelLayer];
      console.log('使用天地图图层');
      console.log('天地图矢量图层:', vectorSource);
      console.log('天地图注记图层:', labelSource);
    } else if (mapType === 'local') {
      // 本地地图实现
      console.log('开始初始化本地地图');
      const localSource = new XYZ({
        url: '/map/{z}/{x}/{y}.png',
        projection: 'EPSG:3857',
        tileGrid: {
          origin: [-20037508.3428, 20037508.3428],
          resolutions: [
            156543.03392804103,
            78271.51696402051,
            39135.758482010255,
            19567.879241005127,
            9783.939620502563,
            4891.969810251282,
            2445.984905125641,
            1222.9924525628205,
            611.4962262814102,
            305.7481131407051,
            152.87405657035256,
            76.43702828517628,
            38.21851414258814,
            19.10925707129407,
            9.554628535647034,
            4.777314267823517,
            2.3886571339117583,
            1.1943285669558792,
            0.5971642834779396,
            0.2985821417389698
          ],
          tileSize: 256
        }
      });
      
      const localLayer = new TileLayer({
        source: localSource,
      });
      
      layers = [localLayer];
      console.log('使用本地地图图层');
    } else {
      // 默认使用OSM地图
      console.log('开始初始化OSM地图');
      const osmLayer = new TileLayer({
        source: new OSM({
          url: 'https://{a-c}.tile.openstreetmap.org/{z}/{x}/{y}.png'
        })
      });
      
      layers = [osmLayer];
      console.log('使用OSM地图图层');
    }
    
    // 创建地图实例
    map = new Map({
      target: mapContainer,
      layers: layers,
      view: new View({
        center: fromLonLat([116.397428, 39.90923]),
        zoom: 12,
        minZoom: 2,
        maxZoom: 18
      })
    });
    
    // 地图加载完成后移除加载状态
    map.on('rendercomplete', () => {
      setTimeout(() => {
        const loadingElement = document.getElementById('map-loading');
        if (loadingElement) {
          loadingElement.remove();
        }
        console.log('轨迹回放地图渲染完成');
      }, 500);
    });
    
    console.log('轨迹回放地图初始化成功');
  } catch (error) {
    console.error('创建地图实例失败:', error);
    // 显示错误提示
    if (mapContainer) {
      mapContainer.innerHTML = '';
      const errorDiv = document.createElement('div');
      errorDiv.style.cssText = 'width: 100%; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; background: #f0f9ff; color: #64b5f6;';
      errorDiv.innerHTML = '<strong>地图初始化失败</strong><br><small>请检查网络连接或OpenLayers库是否正确加载</small>';
      mapContainer.appendChild(errorDiv);
    }
  }
};

// 加载车辆列表
const loadVehicles = async () => {
  try {
    console.log('开始获取车辆列表');
    const response = await api.get('/api/vehicles');
    console.log('车辆列表响应:', response);
    
    // 处理不同的响应格式
    if (response) {
      if (response.list) {
        vehicles.value = response.list;
      } else if (response.items) {
        vehicles.value = response.items;
      } else if (Array.isArray(response)) {
        vehicles.value = response;
      } else {
        vehicles.value = [];
      }
    } else {
      vehicles.value = [];
    }
    
    console.log('车辆列表加载成功，共', vehicles.value.length, '辆');
  } catch (error) {
    console.error('获取车辆列表失败:', error);
    ElMessage.error('获取车辆列表失败');
  }
};

// 加载轨迹数据
const loadTrackData = async () => {
  if (!playbackForm.value.vehicleId || !playbackForm.value.startTime || !playbackForm.value.endTime) {
    ElMessage.warning('请选择车辆和时间范围');
    return;
  }
  
  try {
    // 转换时间格式为ISO字符串
    const startTime = new Date(playbackForm.value.startTime).toISOString();
    const endTime = new Date(playbackForm.value.endTime).toISOString();
    
    console.log('开始加载轨迹数据，参数:', {
      vehicle_id: playbackForm.value.vehicleId,
      start_time: startTime,
      end_time: endTime
    });
    
    const response = await api.get('/api/tracks', {
      params: {
        vehicle_id: playbackForm.value.vehicleId,
        start_time: startTime,
        end_time: endTime,
        page: 1,
        page_size: 1000
      }
    });
    
    console.log('轨迹数据响应:', response);
    
    // 处理响应数据
    let trackList: any[] = [];
    
    if (response && response.list) {
      // 直接返回分页格式
      trackList = response.list;
    } else if (response && Array.isArray(response)) {
      // 直接返回数组
      trackList = response;
    } else {
      console.warn('未获取到轨迹数据:', response);
      ElMessage.warning('未获取到轨迹数据');
      return;
    }
    
    // 转换轨迹数据格式，确保包含longitude和latitude字段
    const formattedTracks = trackList.map(item => ({
      time: item.track_time || item.time,
      latitude: item.latitude,
      longitude: item.longitude,
      speed: item.speed || 0,
      direction: item.direction || '未知',
      status: item.status_text || item.status || '正常'
    }));
    
    trackData.value = formattedTracks;
    drawTrack(formattedTracks);
    console.log('轨迹数据加载成功，共', formattedTracks.length, '条记录');
  } catch (error) {
    console.error('获取轨迹数据失败:', error);
    ElMessage.error('获取轨迹数据失败');
  }
};

// 绘制轨迹
const drawTrack = (data: any[]) => {
  if (!map) {
    console.error('地图实例不存在，无法绘制轨迹');
    ElMessage.error('地图未初始化，无法绘制轨迹');
    return;
  }
  
  // 清空之前的轨迹
  if (polylineLayer) {
    map.removeLayer(polylineLayer);
  }
  
  if (markerLayer) {
    map.removeLayer(markerLayer);
  }
  
  if (data.length === 0) {
    ElMessage.warning('轨迹数据为空');
    return;
  }
  
  try {
    // 提取经纬度坐标并转换为 OpenLayers 坐标
    const coordinates = data.map(item => fromLonLat([item.longitude, item.latitude]));
    
    // 创建轨迹线特征
    const lineFeature = new Feature({
      geometry: new LineString(coordinates)
    });
    
    // 创建轨迹线样式
    const lineStyle = new Style({
      stroke: new Stroke({
        color: '#3366FF',
        width: 4
      })
    });
    
    // 创建轨迹线源和层
    const lineSource = new VectorSource();
    lineSource.addFeature(lineFeature);
    polylineLayer = new VectorLayer({
      source: lineSource,
      style: lineStyle
    });
    
    // 创建标记点特征
    const pointFeature = new Feature({
      geometry: new Point(coordinates[0])
    });
    
    // 创建标记点源和层
    const pointSource = new VectorSource();
    pointSource.addFeature(pointFeature);
    markerLayer = new VectorLayer({
      source: pointSource
    });
    
    // 添加层到地图
    map.addLayer(polylineLayer);
    map.addLayer(markerLayer);
    
    // 调整地图视野
    const lineExtent = lineFeature.getGeometry()?.getExtent();
    if (lineExtent) {
      map.getView().fit(lineExtent, { padding: [50, 50, 50, 50] });
    }
    
    console.log('轨迹绘制成功');
  } catch (error) {
    console.error('绘制轨迹失败:', error);
    ElMessage.error('绘制轨迹失败');
  }
};

// 开始回放
const startPlayback = async () => {
  // 先加载轨迹数据
  await loadTrackData();
  
  if (trackData.value.length === 0) {
    ElMessage.warning('没有轨迹数据可供回放');
    return;
  }
  
  if (!map) {
    console.error('地图实例不存在，无法开始回放');
    ElMessage.error('地图未初始化，无法开始回放');
    return;
  }
  
  // 停止之前的回放
  if (playbackTimer) {
    clearInterval(playbackTimer);
  }
  
  currentIndex = 0;
  
  // 开始回放
  playbackTimer = window.setInterval(() => {
    if (currentIndex < trackData.value.length) {
      const point = trackData.value[currentIndex];
      const coordinate = fromLonLat([point.longitude, point.latitude]);
      
      // 更新标记点位置
      if (markerLayer) {
        const source = markerLayer.getSource();
        if (source) {
          const features = source.getFeatures();
          if (features.length > 0) {
            const feature = features[0];
            feature.setGeometry(new Point(coordinate));
          }
        }
      }
      
      // 更新地图中心
      try {
        map.getView().setCenter(coordinate);
      } catch (error) {
        console.error('更新地图中心失败:', error);
      }
      
      currentIndex++;
    } else {
      // 回放结束
      if (playbackTimer) {
        clearInterval(playbackTimer);
        playbackTimer = null;
      }
      ElMessage.success('轨迹回放完成');
    }
  }, 1000 / playbackForm.value.speed);
};

// 暂停回放
const pausePlayback = () => {
  if (playbackTimer) {
    clearInterval(playbackTimer);
    playbackTimer = null;
    ElMessage.info('轨迹回放已暂停');
  }
};

// 停止回放
const stopPlayback = () => {
  if (playbackTimer) {
    clearInterval(playbackTimer);
    playbackTimer = null;
  }
  
  currentIndex = 0;
  
  if (trackData.value.length > 0 && map) {
    const firstPoint = trackData.value[0];
    if (markerLayer) {
      // 处理标记点位置
    }
    try {
      map.setCenter([firstPoint.longitude, firstPoint.latitude]);
    } catch (error) {
      console.error('设置地图中心失败:', error);
    }
  }
  
  ElMessage.info('轨迹回放已停止');
};

// 居中地图
const centerMap = () => {
  if (!map) {
    console.error('地图实例不存在，无法居中地图');
    ElMessage.error('地图未初始化，无法居中地图');
    return;
  }
  
  try {
    if (trackData.value.length > 0) {
      const firstPoint = trackData.value[0];
      map.setCenter([firstPoint.longitude, firstPoint.latitude]);
    } else {
      map.setCenter([116.397428, 39.90923]);
    }
  } catch (error) {
    console.error('居中地图失败:', error);
    ElMessage.error('居中地图失败');
  }
};

// 清空地图
const clearMap = () => {
  if (map) {
    if (polylineLayer) {
      map.removeLayer(polylineLayer);
      polylineLayer = null;
    }
    
    if (markerLayer) {
      map.removeLayer(markerLayer);
      markerLayer = null;
    }
  }
  
  trackData.value = [];
  currentIndex = 0;
  
  if (playbackTimer) {
    clearInterval(playbackTimer);
    playbackTimer = null;
  }
  
  ElMessage.info('轨迹已清空');
};

// 组件挂载时
onMounted(() => {
  initMap();
  loadVehicles();
});

// 组件卸载时
onUnmounted(() => {
  // 清理回放定时器
  if (playbackTimer) {
    clearInterval(playbackTimer);
    playbackTimer = null;
  }
  
  // 清理地图实例
  if (map) {
    map.setTarget(null);
    map.dispose();
    map = null;
  }
});
</script>

<style scoped>
.track-playback-container {
  padding: 20px;
}

.playback-controls {
  margin-bottom: 20px;
}

.map-container {
  margin-bottom: 20px;
}

.map {
  height: 500px;
  width: 100%;
}

.track-data {
  margin-top: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.map-controls {
  display: flex;
  gap: 10px;
}
</style>