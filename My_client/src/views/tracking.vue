<template>
  <div class="tracking-management">
    <div class="tracking-header">
      <h2>轨迹回放</h2>
    </div>

    <div class="tracking-tabs">
      <el-tabs v-model="activeTab" type="card">
        <!-- 轨迹回放 -->
        <el-tab-pane label="轨迹回放" name="track">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>轨迹回放管理</span>
                </div>
              </template>
              <div class="track-playback-section">
                <div class="playback-controls">
                  <el-form :inline="true" :model="playbackForm" class="playback-form">
                    <el-form-item label="车辆">
                      <el-select v-model="playbackForm.vehicleId" placeholder="选择车辆" clearable>
                        <el-option
                          v-for="vehicle in vehicles"
                          :key="vehicle.vehicle_id"
                          :label="vehicle.license_plate"
                          :value="vehicle.vehicle_id"
                        />
                      </el-select>
                    </el-form-item>
                    <el-form-item label="开始时间">
                      <el-date-picker
                        v-model="playbackForm.startTime"
                        type="datetime"
                        placeholder="选择开始时间"
                        clearable
                      />
                    </el-form-item>
                    <el-form-item label="结束时间">
                      <el-date-picker
                        v-model="playbackForm.endTime"
                        type="datetime"
                        placeholder="选择结束时间"
                        clearable
                      />
                    </el-form-item>
                    <el-form-item>
                      <el-button type="primary" @click="loadTrackData">
                        <el-icon><Search /></el-icon> 加载轨迹
                      </el-button>
                    </el-form-item>
                  </el-form>

                  <div class="playback-buttons">
                    <el-button-group>
                      <el-button
                        :icon="VideoPlay"
                        type="primary"
                        @click="startPlayback"
                        :disabled="!playbackData.length"
                      >
                        开始
                      </el-button>
                      <el-button :icon="VideoPause" @click="pausePlayback" :disabled="!isPlaying"> 暂停 </el-button>
                      <el-button :icon="Close" @click="stopPlayback" :disabled="!isPlaying && !isPaused">
                        停止
                      </el-button>
                    </el-button-group>

                    <el-button-group>
                      <el-button @click="showPointTrack" :type="trackType === 'point' ? 'primary' : ''">
                        点轨迹
                      </el-button>
                      <el-button @click="showMotionTrack" :type="trackType === 'motion' ? 'primary' : ''">
                        运动轨迹
                      </el-button>
                      <el-button @click="showAllTrack" :type="trackType === 'all' ? 'primary' : ''">
                        全部轨迹
                      </el-button>
                      <el-button type="danger" @click="clearTrack" :disabled="!playbackData.length">
                        清除轨迹
                      </el-button>
                    </el-button-group>

                    <el-form :inline="true" :model="playbackForm" class="speed-control">
                      <el-form-item label="速度">
                        <el-select v-model="playbackForm.speed" placeholder="播放速度">
                          <el-option label="0.5x" :value="0.5" />
                          <el-option label="1x" :value="1" />
                          <el-option label="2x" :value="2" />
                          <el-option label="5x" :value="5" />
                        </el-select>
                      </el-form-item>
                    </el-form>
                  </div>
                </div>

                <div class="playback-map" v-loading="loading.track">
                  <div ref="playbackMapContainer" class="map-container"></div>
                </div>

                <div class="playback-info">
                  <h4>轨迹信息</h4>
                  <el-table :data="playbackData" stripe size="small">
                    <el-table-column prop="time" label="时间" width="180" />
                    <el-table-column prop="latitude" label="纬度" width="120" />
                    <el-table-column prop="longitude" label="经度" width="120" />
                    <el-table-column prop="speed" label="速度 (km/h)" width="100" />
                    <el-table-column prop="direction" label="方向 (°)" width="100" />
                    <el-table-column prop="address" label="地址" />
                  </el-table>
                </div>
              </div>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- 载重分析 -->
        <el-tab-pane label="载重分析" name="loadAnalysis">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>载重分析</span>
                </div>
              </template>
              <div class="load-analysis-content">
                <div class="load-analysis-params">
                  <el-date-picker
                    v-model="loadAnalysisForm.date"
                    type="date"
                    placeholder="选择日期"
                    format="YYYY-MM-DD"
                    value-format="YYYY-MM-DD"
                    style="width: 150px; margin-right: 10px;"
                  />
                  <el-select
                    v-model="loadAnalysisForm.vehicleId"
                    placeholder="选择车辆"
                    style="width: 180px; margin-right: 10px;"
                  >
                    <el-option
                      v-for="vehicle in vehicles"
                      :key="vehicle.vehicle_id"
                      :label="vehicle.license_plate"
                      :value="vehicle.vehicle_id"
                    />
                  </el-select>
                  <el-button type="primary" @click="loadLoadAnalysisData" :loading="loading.loadAnalysis">
                    获取数据
                  </el-button>
                </div>
                <div class="load-analysis-chart">
                  <h3>载重分析曲线</h3>
                  <div class="chart-container" ref="loadAnalysisChartRef"></div>
                </div>
              </div>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- 装卸位置 -->
        <el-tab-pane label="装卸位置" name="loadingPosition">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>装卸位置</span>
                </div>
              </template>
              <div class="loading-position-content">
                <div class="loading-position-params">
                  <el-date-picker
                    v-model="loadingPositionForm.dateRange"
                    type="daterange"
                    range-separator="至"
                    start-placeholder="开始日期"
                    end-placeholder="结束日期"
                    format="YYYY-MM-DD"
                    value-format="YYYY-MM-DD"
                    style="width: 300px; margin-right: 10px;"
                  />
                  <el-select
                    v-model="loadingPositionForm.vehicleId"
                    placeholder="选择车辆"
                    style="width: 180px; margin-right: 10px;"
                  >
                    <el-option
                      v-for="vehicle in vehicles"
                      :key="vehicle.vehicle_id"
                      :label="vehicle.license_plate"
                      :value="vehicle.vehicle_id"
                    />
                  </el-select>
                  <el-button type="primary" @click="loadLoadingPositionData" :loading="loading.loadingPosition">
                    获取数据
                  </el-button>
                </div>
                <div class="loading-position-list" v-loading="loading.loadingPosition">
                  <el-table :data="loadingPositionList" stripe>
                    <el-table-column prop="id" label="ID" width="60" />
                    <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
                    <el-table-column prop="license_plate" label="车牌号" />
                    <el-table-column prop="type" label="类型" width="80">
                      <template #default="scope">
                        <el-tag :type="scope.row.type === 'loading' ? 'success' : 'warning'" size="small">
                          {{ scope.row.type === 'loading' ? '装货' : '卸货' }}
                        </el-tag>
                      </template>
                    </el-table-column>
                    <el-table-column prop="location" label="位置" />
                    <el-table-column prop="time" label="时间" width="180" />
                  </el-table>
                </div>
              </div>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- 装卸趟次 -->
        <el-tab-pane label="装卸趟次" name="loadingTrips">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>装卸趟次</span>
                </div>
              </template>
              <div class="loading-trips-content">
                <div class="loading-trips-params">
                  <el-date-picker
                    v-model="loadingTripsForm.dateRange"
                    type="daterange"
                    range-separator="至"
                    start-placeholder="开始日期"
                    end-placeholder="结束日期"
                    format="YYYY-MM-DD"
                    value-format="YYYY-MM-DD"
                    style="width: 300px; margin-right: 10px;"
                  />
                  <el-select
                    v-model="loadingTripsForm.vehicleId"
                    placeholder="选择车辆"
                    style="width: 180px; margin-right: 10px;"
                  >
                    <el-option
                      v-for="vehicle in vehicles"
                      :key="vehicle.vehicle_id"
                      :label="vehicle.license_plate"
                      :value="vehicle.vehicle_id"
                    />
                  </el-select>
                  <el-button type="primary" @click="loadLoadingTripsData" :loading="loading.loadingTrips">
                    获取数据
                  </el-button>
                </div>
                <div class="loading-trips-list" v-loading="loading.loadingTrips">
                  <el-table :data="loadingTripsList" stripe>
                    <el-table-column prop="trip_id" label="趟次ID" width="100" />
                    <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
                    <el-table-column prop="license_plate" label="车牌号" />
                    <el-table-column prop="start_time" label="开始时间" width="180" />
                    <el-table-column prop="end_time" label="结束时间" width="180" />
                    <el-table-column prop="loading_count" label="装货次数" width="100" />
                    <el-table-column prop="unloading_count" label="卸货次数" width="100" />
                  </el-table>
                </div>
              </div>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- 轨迹数据 -->
        <el-tab-pane label="轨迹数据" name="trackData">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>轨迹数据</span>
                </div>
              </template>
              <div class="track-data-content">
                <div class="track-data-params">
                  <el-date-picker
                    v-model="trackDataForm.dateRange"
                    type="daterange"
                    range-separator="至"
                    start-placeholder="开始日期"
                    end-placeholder="结束日期"
                    format="YYYY-MM-DD"
                    value-format="YYYY-MM-DD"
                    style="width: 300px; margin-right: 10px;"
                  />
                  <el-select
                    v-model="trackDataForm.vehicleId"
                    placeholder="选择车辆"
                    style="width: 180px; margin-right: 10px;"
                  >
                    <el-option
                      v-for="vehicle in vehicles"
                      :key="vehicle.vehicle_id"
                      :label="vehicle.license_plate"
                      :value="vehicle.vehicle_id"
                    />
                  </el-select>
                  <el-button type="primary" @click="loadTrackDataNew" :loading="loading.trackData">
                    获取数据
                  </el-button>
                </div>
                <div class="track-data-list" v-loading="loading.trackData">
                  <el-table :data="trackDataList" stripe>
                    <el-table-column prop="id" label="ID" width="60" />
                    <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
                    <el-table-column prop="latitude" label="纬度" width="120" />
                    <el-table-column prop="longitude" label="经度" width="120" />
                    <el-table-column prop="speed" label="速度" width="80" />
                    <el-table-column prop="timestamp" label="时间" width="180" />
                  </el-table>
                </div>
              </div>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- 运输作业 -->
        <el-tab-pane label="运输作业" name="transportJob">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>运输作业</span>
                </div>
              </template>
              <div class="transport-job-content">
                <div class="transport-job-params">
                  <el-date-picker
                    v-model="transportJobForm.dateRange"
                    type="daterange"
                    range-separator="至"
                    start-placeholder="开始日期"
                    end-placeholder="结束日期"
                    format="YYYY-MM-DD"
                    value-format="YYYY-MM-DD"
                    style="width: 300px; margin-right: 10px;"
                  />
                  <el-select
                    v-model="transportJobForm.vehicleId"
                    placeholder="选择车辆"
                    style="width: 180px; margin-right: 10px;"
                  >
                    <el-option
                      v-for="vehicle in vehicles"
                      :key="vehicle.vehicle_id"
                      :label="vehicle.license_plate"
                      :value="vehicle.vehicle_id"
                    />
                  </el-select>
                  <el-button type="primary" @click="loadTransportJobData" :loading="loading.transportJob">
                    获取数据
                  </el-button>
                </div>
                <div class="transport-job-list" v-loading="loading.transportJob">
                  <el-table :data="transportJobList" stripe>
                    <el-table-column prop="job_id" label="作业ID" width="100" />
                    <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
                    <el-table-column prop="license_plate" label="车牌号" />
                    <el-table-column prop="start_location" label="起始位置" />
                    <el-table-column prop="end_location" label="结束位置" />
                    <el-table-column prop="start_time" label="开始时间" width="180" />
                    <el-table-column prop="end_time" label="结束时间" width="180" />
                  </el-table>
                </div>
              </div>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- 告警事件 -->
        <el-tab-pane label="告警事件" name="alarmEvent">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>告警事件</span>
                </div>
              </template>
              <div class="alarm-event-content">
                <div class="alarm-event-params">
                  <el-date-picker
                    v-model="alarmEventForm.dateRange"
                    type="daterange"
                    range-separator="至"
                    start-placeholder="开始日期"
                    end-placeholder="结束日期"
                    format="YYYY-MM-DD"
                    value-format="YYYY-MM-DD"
                    style="width: 300px; margin-right: 10px;"
                  />
                  <el-select
                    v-model="alarmEventForm.vehicleId"
                    placeholder="选择车辆"
                    style="width: 180px; margin-right: 10px;"
                  >
                    <el-option
                      v-for="vehicle in vehicles"
                      :key="vehicle.vehicle_id"
                      :label="vehicle.license_plate"
                      :value="vehicle.vehicle_id"
                    />
                  </el-select>
                  <el-select
                    v-model="alarmEventForm.alarmType"
                    placeholder="告警类型"
                    style="width: 150px; margin-right: 10px;"
                  >
                    <el-option label="全部" value="" />
                    <el-option label="超速" value="speeding" />
                    <el-option label="超载" value="overload" />
                    <el-option label="偏离路线" value="deviation" />
                  </el-select>
                  <el-button type="primary" @click="loadAlarmEventData" :loading="loading.alarmEvent">
                    获取数据
                  </el-button>
                </div>
                <div class="alarm-event-list" v-loading="loading.alarmEvent">
                  <el-table :data="alarmEventList" stripe>
                    <el-table-column prop="alarm_id" label="告警ID" width="100" />
                    <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
                    <el-table-column prop="license_plate" label="车牌号" />
                    <el-table-column prop="alarm_type" label="告警类型" width="100" />
                    <el-table-column prop="alarm_message" label="告警信息" />
                    <el-table-column prop="alarm_time" label="告警时间" width="180" />
                    <el-table-column prop="status" label="状态" width="80">
                      <template #default="scope">
                        <el-tag :type="scope.row.status === 'resolved' ? 'success' : 'danger'" size="small">
                          {{ scope.row.status === 'resolved' ? '已处理' : '未处理' }}
                        </el-tag>
                      </template>
                    </el-table-column>
                  </el-table>
                </div>
              </div>
            </el-card>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { Search, VideoPlay, VideoPause, Close } from '@element-plus/icons-vue';
import api from '@/api';

// OpenLayers 地图相关导入
import { Map } from 'ol';
import { View } from 'ol';
import { Tile as TileLayer } from 'ol/layer';
import { XYZ } from 'ol/source';
import { Vector as VectorLayer } from 'ol/layer';
import { Vector as VectorSource } from 'ol/source';
import { fromLonLat } from 'ol/proj';
import { Point, LineString } from 'ol/geom';
import { Feature } from 'ol';
import { Style, Circle, Fill, Stroke, Text } from 'ol/style';

interface VehicleItem {
  vehicle_id: number;
  license_plate: string;
}

interface TrackPoint {
  time: string;
  latitude: number;
  longitude: number;
  speed: number;
  direction: number;
  address: string;
  license_plate?: string;
}

// 当前激活的标签页
const activeTab = ref('track');

// 加载状态
const loading = reactive({
  loadAnalysis: false,
  loadingPosition: false,
  loadingTrips: false,
  trackData: false,
  transportJob: false,
  alarmEvent: false,
  track: false,
});

// 车辆列表
const vehicles = ref<VehicleItem[]>([]);

// 载重分析表单
const loadAnalysisForm = reactive({
  date: null as string | null,
  vehicleId: null as number | null,
});

// 装卸位置表单
const loadingPositionForm = reactive({
  dateRange: null as [string, string] | null,
  vehicleId: null as number | null,
});

// 装卸趟次表单
const loadingTripsForm = reactive({
  dateRange: null as [string, string] | null,
  vehicleId: null as number | null,
});

// 轨迹数据表单
const trackDataForm = reactive({
  dateRange: null as [string, string] | null,
  vehicleId: null as number | null,
});

// 运输作业表单
const transportJobForm = reactive({
  dateRange: null as [string, string] | null,
  vehicleId: null as number | null,
});

// 告警事件表单
const alarmEventForm = reactive({
  dateRange: null as [string, string] | null,
  vehicleId: null as number | null,
  alarmType: '',
});

// 轨迹回放相关
const playbackForm = reactive({
  vehicleId: null as number | null,
  startTime: null as Date | null,
  endTime: null as Date | null,
  speed: 1,
});
const playbackData = ref<TrackPoint[]>([]);
const playbackMapContainer = ref<HTMLElement | null>(null);
const playbackMap = ref<Map | null>(null);
const trackLayer = ref<VectorLayer | null>(null);
const vehicleMarker = ref<Feature | null>(null);
const isPlaying = ref(false);
const isPaused = ref(false);
const currentPlaybackIndex = ref(0);
const playbackTimer = ref<number | null>(null);
const trackType = ref('all'); // point: 点轨迹, motion: 运动轨迹, all: 全部轨迹

// 数据列表
const loadAnalysisData = ref<any[]>([]);
const loadingPositionList = ref<any[]>([]);
const loadingTripsList = ref<any[]>([]);
const trackDataList = ref<any[]>([]);
const transportJobList = ref<any[]>([]);
const alarmEventList = ref<any[]>([]);

// 图表引用
const loadAnalysisChartRef = ref<HTMLElement | null>(null);

// 加载车辆列表
const loadVehicles = async () => {
  try {
    const response = await api.get('/api/vehicles') as any;
    vehicles.value = response.items || [];
  } catch (error) {
    console.error('获取车辆列表失败:', error);
    ElMessage.error('获取车辆列表失败');
  }
};

// 载重分析数据加载
const loadLoadAnalysisData = async () => {
  if (!loadAnalysisForm.date || !loadAnalysisForm.vehicleId) {
    ElMessage.warning('请选择日期和车辆');
    return;
  }
  
  loading.loadAnalysis = true;
  try {
    // 模拟数据
    loadAnalysisData.value = Array.from({ length: 24 }, (_, i) => ({
      time: `${i}:00`,
      weight: Math.random() * 5 + 10,
    }));
    ElMessage.success('获取数据成功');
  } catch (error) {
    console.error('获取载重分析数据失败:', error);
    ElMessage.error('获取数据失败');
  } finally {
    loading.loadAnalysis = false;
  }
};

// 装卸位置数据加载
const loadLoadingPositionData = async () => {
  if (!loadingPositionForm.dateRange || !loadingPositionForm.vehicleId) {
    ElMessage.warning('请选择时间范围和车辆');
    return;
  }
  
  loading.loadingPosition = true;
  try {
    // 模拟数据
    loadingPositionList.value = Array.from({ length: 10 }, (_, i) => ({
      id: i + 1,
      vehicle_id: loadingPositionForm.vehicleId,
      license_plate: vehicles.value.find(v => v.vehicle_id === loadingPositionForm.vehicleId)?.license_plate || '',
      type: i % 2 === 0 ? 'loading' : 'unloading',
      location: `位置${i + 1}`,
      time: new Date().toISOString(),
    }));
    ElMessage.success('获取数据成功');
  } catch (error) {
    console.error('获取装卸位置数据失败:', error);
    ElMessage.error('获取数据失败');
  } finally {
    loading.loadingPosition = false;
  }
};

// 装卸趟次数据加载
const loadLoadingTripsData = async () => {
  if (!loadingTripsForm.dateRange || !loadingTripsForm.vehicleId) {
    ElMessage.warning('请选择时间范围和车辆');
    return;
  }
  
  loading.loadingTrips = true;
  try {
    // 模拟数据
    loadingTripsList.value = Array.from({ length: 5 }, (_, i) => ({
      trip_id: i + 1,
      vehicle_id: loadingTripsForm.vehicleId,
      license_plate: vehicles.value.find(v => v.vehicle_id === loadingTripsForm.vehicleId)?.license_plate || '',
      start_time: new Date().toISOString(),
      end_time: new Date().toISOString(),
      loading_count: Math.floor(Math.random() * 3) + 1,
      unloading_count: Math.floor(Math.random() * 3) + 1,
    }));
    ElMessage.success('获取数据成功');
  } catch (error) {
    console.error('获取装卸趟次数据失败:', error);
    ElMessage.error('获取数据失败');
  } finally {
    loading.loadingTrips = false;
  }
};

// 轨迹数据加载
const loadTrackDataNew = async () => {
  if (!trackDataForm.dateRange || !trackDataForm.vehicleId) {
    ElMessage.warning('请选择时间范围和车辆');
    return;
  }
  
  loading.trackData = true;
  try {
    // 模拟数据
    trackDataList.value = Array.from({ length: 20 }, (_, i) => ({
      id: i + 1,
      vehicle_id: trackDataForm.vehicleId,
      latitude: 39.9 + Math.random() * 0.1,
      longitude: 116.4 + Math.random() * 0.1,
      speed: Math.random() * 60,
      timestamp: new Date().toISOString(),
    }));
    ElMessage.success('获取数据成功');
  } catch (error) {
    console.error('获取轨迹数据失败:', error);
    ElMessage.error('获取数据失败');
  } finally {
    loading.trackData = false;
  }
};

// 运输作业数据加载
const loadTransportJobData = async () => {
  if (!transportJobForm.dateRange || !transportJobForm.vehicleId) {
    ElMessage.warning('请选择时间范围和车辆');
    return;
  }
  
  loading.transportJob = true;
  try {
    // 模拟数据
    transportJobList.value = Array.from({ length: 8 }, (_, i) => ({
      job_id: i + 1,
      vehicle_id: transportJobForm.vehicleId,
      license_plate: vehicles.value.find(v => v.vehicle_id === transportJobForm.vehicleId)?.license_plate || '',
      start_location: `起点${i + 1}`,
      end_location: `终点${i + 1}`,
      start_time: new Date().toISOString(),
      end_time: new Date().toISOString(),
    }));
    ElMessage.success('获取数据成功');
  } catch (error) {
    console.error('获取运输作业数据失败:', error);
    ElMessage.error('获取数据失败');
  } finally {
    loading.transportJob = false;
  }
};

// 告警事件数据加载
const loadAlarmEventData = async () => {
  if (!alarmEventForm.dateRange || !alarmEventForm.vehicleId) {
    ElMessage.warning('请选择时间范围和车辆');
    return;
  }
  
  loading.alarmEvent = true;
  try {
    // 模拟数据
    const alarmTypes = ['speeding', 'overload', 'deviation'];
    alarmEventList.value = Array.from({ length: 12 }, (_, i) => ({
      alarm_id: i + 1,
      vehicle_id: alarmEventForm.vehicleId,
      license_plate: vehicles.value.find(v => v.vehicle_id === alarmEventForm.vehicleId)?.license_plate || '',
      alarm_type: alarmTypes[i % 3],
      alarm_message: `告警信息${i + 1}`,
      alarm_time: new Date().toISOString(),
      status: i % 2 === 0 ? 'resolved' : 'unresolved',
    }));
    ElMessage.success('获取数据成功');
  } catch (error) {
    console.error('获取告警事件数据失败:', error);
    ElMessage.error('获取数据失败');
  } finally {
    loading.alarmEvent = false;
  }
};

// 轨迹回放相关方法
const initPlaybackMap = () => {
  if (!playbackMapContainer.value) return;

  try {
    // 从localStorage获取天地图API Key
    const tiandituKey = localStorage.getItem('tiandituKey') || '34d8cf060f7e8ac09be79b9261d65274';

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

    // 创建地图实例
    const map = new Map({
      target: playbackMapContainer.value,
      layers: [vectorLayer, labelLayer],
      view: new View({
        center: fromLonLat([116.404, 39.915]),
        zoom: 12,
        maxZoom: 18,
        minZoom: 2,
      }),
    });

    playbackMap.value = map;
  } catch (error) {
    console.error('轨迹回放地图初始化失败:', error);
    ElMessage.error('轨迹回放地图初始化失败');
  }
};

const loadTrackData = async () => {
  if (!playbackForm.vehicleId || !playbackForm.startTime || !playbackForm.endTime) {
    ElMessage.warning('请选择车辆和时间范围');
    return;
  }

  loading.track = true;
  try {
    // 模拟数据
    playbackData.value = Array.from({ length: 50 }, (_, i) => ({
      time: new Date(Date.now() - (50 - i) * 60000).toISOString(),
      latitude: 39.9 + Math.random() * 0.1,
      longitude: 116.4 + Math.random() * 0.1,
      speed: Math.random() * 60,
      direction: Math.random() * 360,
      address: `地址${i + 1}`,
      license_plate: vehicles.value.find(v => v.vehicle_id === playbackForm.vehicleId)?.license_plate || '',
    }));

    if (playbackData.value.length > 0) {
      drawTrack();
      ElMessage.success(`加载成功，共 ${playbackData.value.length} 条轨迹数据`);
    } else {
      ElMessage.info('未找到轨迹数据');
    }
  } catch (error) {
    console.error('加载轨迹数据失败:', error);
    ElMessage.error('加载轨迹数据失败');
  } finally {
    loading.track = false;
  }
};

const drawTrack = () => {
  if (!playbackMap.value || playbackData.value.length === 0) return;

  // 清除之前的轨迹图层
  if (trackLayer.value) {
    playbackMap.value.removeLayer(trackLayer.value as unknown as VectorLayer);
  }

  // 创建轨迹点
  const trackPoints = playbackData.value.map((point: TrackPoint) => {
    return [point.longitude, point.latitude];
  });

  // 创建矢量源
  const vectorSource = new VectorSource();

  // 根据轨迹类型绘制
  if (trackType.value === 'motion' || trackType.value === 'all') {
    // 创建轨迹线
    const lineString = new LineString(trackPoints.map((point: number[]) => fromLonLat(point)));
    const lineFeature = new Feature({
      geometry: lineString,
    });

    // 设置轨迹线样式
    lineFeature.setStyle(
      new Style({
        stroke: new Stroke({
          color: '#409eff',
          width: 2,
        }),
      })
    );

    vectorSource.addFeature(lineFeature);
  }

  if (trackType.value === 'point' || trackType.value === 'all') {
    // 创建轨迹点
    playbackData.value.forEach((point: TrackPoint, index: number) => {
      const pointGeometry = new Point(fromLonLat([point.longitude, point.latitude]));
      const pointFeature = new Feature({
        geometry: pointGeometry,
        properties: { ...point, index },
      });

      // 设置轨迹点样式
      pointFeature.setStyle(
        new Style({
          image: new Circle({
            radius: 4,
            fill: new Fill({ color: '#67C23A' }),
            stroke: new Stroke({ color: '#fff', width: 1 }),
          }),
          text: new Text({
            text: (index + 1).toString(),
            offsetY: -10,
            fill: new Fill({ color: '#303133' }),
            font: '10px sans-serif',
          }),
        })
      );

      vectorSource.addFeature(pointFeature);
    });
  }

  // 创建轨迹图层
  trackLayer.value = new VectorLayer({
    source: vectorSource,
    zIndex: 5,
  });

  playbackMap.value.addLayer(trackLayer.value as unknown as VectorLayer);

  // 调整地图视图以显示整个轨迹
  if (trackPoints.length > 0) {
    const firstPoint = fromLonLat(trackPoints[0]);
    const lastPoint = fromLonLat(trackPoints[trackPoints.length - 1]);
    const extent = [
      Math.min(firstPoint[0], lastPoint[0]) - 0.01,
      Math.min(firstPoint[1], lastPoint[1]) - 0.01,
      Math.max(firstPoint[0], lastPoint[0]) + 0.01,
      Math.max(firstPoint[1], lastPoint[1]) + 0.01,
    ];

    playbackMap.value.getView().fit(extent, {
      padding: [50, 50, 50, 50],
      duration: 1000,
    });
  }
};

const startPlayback = () => {
  if (playbackData.value.length === 0) return;

  isPlaying.value = true;
  isPaused.value = false;

  const playNextPoint = () => {
    if (currentPlaybackIndex.value >= playbackData.value.length) {
      stopPlayback();
      return;
    }

    const currentPoint = playbackData.value[currentPlaybackIndex.value];
    updateVehiclePosition(currentPoint);

    currentPlaybackIndex.value++;

    // 根据速度设置延迟
    const delay = 1000 / playbackForm.speed;
    playbackTimer.value = window.setTimeout(playNextPoint, delay);
  };

  playNextPoint();
};

const pausePlayback = () => {
  if (playbackTimer.value) {
    clearTimeout(playbackTimer.value);
    playbackTimer.value = null;
  }
  isPlaying.value = false;
  isPaused.value = true;
};

const stopPlayback = () => {
  if (playbackTimer.value) {
    clearTimeout(playbackTimer.value);
    playbackTimer.value = null;
  }
  isPlaying.value = false;
  isPaused.value = false;
  currentPlaybackIndex.value = 0;

  if (playbackData.value.length > 0) {
    updateVehiclePosition(playbackData.value[0]);
  }
};

const updateVehiclePosition = (point: TrackPoint) => {
  if (!playbackMap.value) return;

  // 移除旧的车辆标记
  if (vehicleMarker.value) {
    const source = trackLayer.value?.getSource();
    if (source) {
      source.removeFeature(vehicleMarker.value);
    }
  }

  // 创建新的车辆标记
  const markerGeometry = new Point(fromLonLat([point.longitude, point.latitude]));
  const marker = new Feature({
    geometry: markerGeometry,
    properties: point,
  });

  // 设置车辆标记样式
  marker.setStyle(
    new Style({
      image: new Circle({
        radius: 8,
        fill: new Fill({ color: '#67C23A' }),
        stroke: new Stroke({ color: '#fff', width: 2 }),
      }),
      text: new Text({
        text: point.license_plate || '车辆',
        offsetY: -15,
        fill: new Fill({ color: '#303133' }),
      }),
    })
  );

  // 添加车辆标记到图层
  const source = trackLayer.value?.getSource();
  if (source) {
    source.addFeature(marker);
    vehicleMarker.value = marker;
  }

  // 移动地图视图到当前位置
  playbackMap.value.getView().setCenter(fromLonLat([point.longitude, point.latitude]));
  playbackMap.value.getView().setZoom(15);
};

// 轨迹类型切换方法
const showPointTrack = () => {
  trackType.value = 'point';
  if (playbackData.value.length > 0) {
    drawTrack();
  }
};

const showMotionTrack = () => {
  trackType.value = 'motion';
  if (playbackData.value.length > 0) {
    drawTrack();
  }
};

const showAllTrack = () => {
  trackType.value = 'all';
  if (playbackData.value.length > 0) {
    drawTrack();
  }
};

const clearTrack = () => {
  if (playbackMap.value && trackLayer.value) {
    playbackMap.value.removeLayer(trackLayer.value as unknown as VectorLayer);
    trackLayer.value = null;
    playbackData.value = [];
    ElMessage.success('轨迹已清除');
  }
};

// 初始化
onMounted(() => {
  loadVehicles();

  // 延迟初始化地图，确保DOM已经渲染
  setTimeout(() => {
    initPlaybackMap();
  }, 100);
});

// 清理资源
onUnmounted(() => {
  if (playbackTimer.value) {
    clearTimeout(playbackTimer.value);
  }
  if (playbackMap.value) {
    playbackMap.value.dispose();
  }
});

// 监听标签页切换
watch(activeTab, (newTab) => {
  if (newTab === 'track' && !playbackMap.value) {
    setTimeout(() => {
      initPlaybackMap();
    }, 100);
  }
});
</script>

<style scoped>
.tracking-management {
  padding: 20px;
}

.tracking-header h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.tracking-tabs {
  margin-top: 20px;
}

.tab-content {
  min-height: 400px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.load-analysis-params,
.loading-position-params,
.loading-trips-params,
.track-data-params,
.transport-job-params,
.alarm-event-params {
  margin-bottom: 20px;
  padding: 15px;
  background: #f5f7fa;
  border-radius: 4px;
}

.load-analysis-chart {
  margin-top: 20px;
}

.chart-container {
  height: 400px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  background: #fff;
}

.loading-position-list,
.loading-trips-list,
.track-data-list,
.transport-job-list,
.alarm-event-list {
  margin-top: 15px;
}

/* 轨迹回放样式 */
.track-playback-section {
  padding: 20px;
}

.playback-controls {
  margin-bottom: 20px;
}

.playback-form {
  margin-bottom: 15px;
}

.playback-buttons {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-top: 10px;
}

.speed-control {
  margin-left: auto;
}

.playback-map {
  height: 500px;
  margin-bottom: 20px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  overflow: hidden;
}

.map-container {
  width: 100%;
  height: 100%;
}

.playback-info {
  margin-top: 20px;
}

.playback-info h4 {
  margin-bottom: 12px;
  color: #303133;
}

.playback-info .el-table {
  max-height: 300px;
  overflow: auto;
}
</style>