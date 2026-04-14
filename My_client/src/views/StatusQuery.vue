/* global Blob */
<template>
  <div class="status-query">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>状态查询</span>
          <el-button type="primary" size="small" @click="searchData">查询</el-button>
          <el-button size="small" @click="exportData">导出数据</el-button>
        </div>
      </template>

      <!-- 查询条件 -->
      <el-collapse v-model="activeSearchPanel" class="mb-4">
        <el-collapse-item title="查询条件" name="search">
          <el-form :model="searchForm" label-width="120px" inline>
            <el-form-item label="车辆选择">
              <el-select v-model="searchForm.vehicleId" filterable placeholder="请选择车辆" style="width: 200px">
                <el-option
                  v-for="vehicle in vehicleList"
                  :key="vehicle.id"
                  :label="vehicle.license_plate"
                  :value="vehicle.id"
                />
              </el-select>
            </el-form-item>

            <el-form-item label="开始时间">
              <el-date-picker
                v-model="searchForm.startTime"
                type="datetime"
                placeholder="选择开始时间"
                style="width: 200px"
              />
            </el-form-item>

            <el-form-item label="结束时间">
              <el-date-picker
                v-model="searchForm.endTime"
                type="datetime"
                placeholder="选择结束时间"
                style="width: 200px"
              />
            </el-form-item>

            <el-form-item label="状态类型">
              <el-select v-model="searchForm.statusType" placeholder="请选择状态类型" style="width: 150px">
                <el-option label="ACC状态" value="accStatus" />
                <el-option label="车辆运行" value="carRun" />
                <el-option label="车辆停止" value="carStop" />
                <el-option label="车辆称重" value="carWeight" />
              </el-select>
            </el-form-item>
          </el-form>
        </el-collapse-item>
      </el-collapse>

      <!-- 状态数据表格 -->
      <el-tabs v-model="activeTab" type="border-card">
        <!-- 速度筛选 -->
        <el-tab-pane label="速度筛选" name="speedFilter">
          <el-table v-if="speedFilterList.length > 0" :data="speedFilterList" stripe style="width: 100%">
            <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
            <el-table-column prop="license_plate" label="车牌号" width="120" />
            <el-table-column prop="start_time" label="开始时间" width="180" />
            <el-table-column prop="end_time" label="结束时间" width="180" />
            <el-table-column prop="speed" label="速度" width="100" />
            <el-table-column prop="location" label="地点" min-width="200" />
            <el-table-column prop="duration" label="持续时间" width="120" />
          </el-table>
          <el-empty v-else description="暂无数据" />
        </el-tab-pane>

        <!-- 停车统计 -->
        <el-tab-pane label="停车统计" name="parkingStats">
          <el-table v-if="parkingStatsList.length > 0" :data="parkingStatsList" stripe style="width: 100%">
            <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
            <el-table-column prop="license_plate" label="车牌号" width="120" />
            <el-table-column prop="start_time" label="开始时间" width="180" />
            <el-table-column prop="end_time" label="结束时间" width="180" />
            <el-table-column prop="duration" label="停车时长" width="120" />
            <el-table-column prop="location" label="地点" min-width="200" />
            <el-table-column prop="mileage" label="里程" width="120" />
          </el-table>
          <el-empty v-else description="暂无数据" />
        </el-tab-pane>

        <!-- 在线统计 -->
        <el-tab-pane label="在线统计" name="onlineStats">
          <el-table v-if="onlineStatsList.length > 0" :data="onlineStatsList" stripe style="width: 100%">
            <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
            <el-table-column prop="license_plate" label="车牌号" width="120" />
            <el-table-column prop="last_time" label="最后在线时间" width="180" />
            <el-table-column prop="online_status" label="在线状态" width="100">
              <template #default="scope">
                <el-tag :type="scope.row.online_status ? 'success' : 'danger'">
                  {{ scope.row.online_status ? '在线' : '离线' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="acc_status" label="ACC状态" width="100">
              <template #default="scope">
                <el-tag :type="scope.row.acc_status ? 'success' : 'warning'">
                  {{ scope.row.acc_status ? '开启' : '关闭' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="location" label="当前位置" min-width="200" />
            <el-table-column prop="speed" label="当前速度" width="100" />
          </el-table>
          <el-empty v-else description="暂无数据" />
        </el-tab-pane>

        <!-- 里程统计 -->
        <el-tab-pane label="里程统计" name="mileageStats">
          <el-table v-if="mileageStatsList.length > 0" :data="mileageStatsList" stripe style="width: 100%">
            <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
            <el-table-column prop="license_plate" label="车牌号" width="120" />
            <el-table-column prop="date" label="日期" width="150" />
            <el-table-column prop="start_mileage" label="开始里程" width="120" />
            <el-table-column prop="end_mileage" label="结束里程" width="120" />
            <el-table-column prop="total_mileage" label="总里程" width="120" />
            <el-table-column prop="avg_speed" label="平均速度" width="120" />
            <el-table-column prop="max_speed" label="最大速度" width="120" />
          </el-table>
          <el-empty v-else description="暂无数据" />
        </el-tab-pane>

        <!-- 状态查询 -->
        <el-tab-pane label="状态查询" name="statusQuery">
          <el-table v-if="statusQueryList.length > 0" :data="statusQueryList" stripe style="width: 100%">
            <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
            <el-table-column prop="license_plate" label="车牌号" width="120" />
            <el-table-column prop="timestamp" label="时间" width="180" />
            <el-table-column prop="speed" label="速度" width="100" />
            <el-table-column prop="mileage" label="里程" width="120" />
            <el-table-column prop="acc_status" label="ACC状态" width="100">
              <template #default="scope">
                <el-tag :type="scope.row.acc_status ? 'success' : 'warning'">
                  {{ scope.row.acc_status ? '开启' : '关闭' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="direction" label="方向" width="100" />
            <el-table-column prop="location" label="位置" min-width="200" />
          </el-table>
          <el-empty v-else description="暂无数据" />
        </el-tab-pane>

        <!-- 报警记录 -->
        <el-tab-pane label="报警记录" name="alarmRecords">
          <el-table v-if="alarmRecordsList.length > 0" :data="alarmRecordsList" stripe style="width: 100%">
            <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
            <el-table-column prop="license_plate" label="车牌号" width="120" />
            <el-table-column prop="alarm_time" label="报警时间" width="180" />
            <el-table-column prop="alarm_type" label="报警类型" width="120" />
            <el-table-column prop="alarm_content" label="报警内容" width="200" />
            <el-table-column prop="speed" label="报警时速度" width="120" />
            <el-table-column prop="location" label="报警地点" min-width="200" />
            <el-table-column prop="handle_status" label="处理状态" width="120">
              <template #default="scope">
                <el-tag :type="scope.row.handle_status ? 'success' : 'warning'">
                  {{ scope.row.handle_status ? '已处理' : '未处理' }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
          <el-empty v-else description="暂无数据" />
        </el-tab-pane>

        <!-- 车辆资料 -->
        <el-tab-pane label="车辆资料" name="vehicleInfo">
          <el-table v-if="vehicleInfoList.length > 0" :data="vehicleInfoList" stripe style="width: 100%">
            <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
            <el-table-column prop="license_plate" label="车牌号" width="120" />
            <el-table-column prop="vehicle_type" label="车辆类型" width="120" />
            <el-table-column prop="device_id" label="设备ID" width="150" />
            <el-table-column prop="sim_no" label="SIM卡号" width="150" />
            <el-table-column prop="group_name" label="所属分组" width="120" />
            <el-table-column prop="color" label="车辆颜色" width="100" />
            <el-table-column prop="brand" label="车辆品牌" width="120" />
            <el-table-column prop="purchase_date" label="购买日期" width="150" />
            <el-table-column prop="install_date" label="安装日期" width="150" />
          </el-table>
          <el-empty v-else description="暂无数据" />
        </el-tab-pane>

        <!-- 事件日报表 -->
        <el-tab-pane label="事件日报表" name="dailyEventReport">
          <el-table v-if="dailyEventReportList.length > 0" :data="dailyEventReportList" stripe style="width: 100%">
            <el-table-column prop="vehicle_id" label="车辆ID" width="100" />
            <el-table-column prop="license_plate" label="车牌号" width="120" />
            <el-table-column prop="report_date" label="报表日期" width="150" />
            <el-table-column prop="start_time" label="启动时间" width="180" />
            <el-table-column prop="stop_time" label="停止时间" width="180" />
            <el-table-column prop="running_time" label="运行时长" width="120" />
            <el-table-column prop="running_mileage" label="运行里程" width="120" />
            <el-table-column prop="max_speed" label="最大速度" width="120" />
            <el-table-column prop="alarm_count" label="报警次数" width="120" />
          </el-table>
          <el-empty v-else description="暂无数据" />
        </el-tab-pane>

        <!-- 轨迹回放 -->
        <el-tab-pane label="轨迹回放" name="trackPlayback">
          <div style="height: 700px">
            <TrackPlayback />
          </div>
        </el-tab-pane>
      </el-tabs>

      <!-- 分页 -->
      <div class="pagination-container" v-if="total > 0">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          :total="total"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, reactive, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import TrackPlayback from '@/components/TrackPlayback.vue';
import api, { vehicleApi } from '@/api/index';
import type { VehicleStatusData } from '@/types/status';

// 车辆列表项类型
interface VehicleListItem {
  id: number;
  license_plate: string;
}

// 通用分页响应类型
interface GenericPaginatedResponse<T> {
  success?: boolean;
  data?: {
    items?: T[];
    total?: number;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

// 车辆列表
const vehicleList = ref<VehicleListItem[]>([]);

// 活动的查询面板
const activeSearchPanel = ref(['search']);

// 查询表单
const searchForm = reactive({
  vehicleId: '',
  startTime: new Date(new Date().getTime() - 15 * 24 * 60 * 60 * 1000),
  endTime: new Date(),
  statusType: 'accStatus',
});

// 选项卡
const activeTab = ref('accStatus');

// 分页
const currentPage = ref(1);
const pageSize = ref(10);
const total = ref(0);

// 速度过滤数据类型
interface SpeedFilterData {
  vehicleId: number;
  licensePlate: string;
  maxSpeed: number;
  avgSpeed: number;
  overSpeedCount: number;
}

// 停车统计数据类型
interface ParkingStatsData {
  vehicleId: number;
  licensePlate: string;
  totalParkingTime: number;
  parkingCount: number;
  longestParkingTime: number;
}

// 在线统计数据类型
interface OnlineStatsData {
  vehicleId: number;
  licensePlate: string;
  onlineTime: number;
  offlineTime: number;
  onlineRate: number;
}

// 里程统计数据类型
interface MileageStatsData {
  vehicleId: number;
  licensePlate: string;
  totalMileage: number;
  dailyMileage: number;
  avgMileage: number;
}

// 警报记录数据类型
interface AlarmRecordData {
  id: number;
  vehicleId: number;
  vehiclePlate: string;
  type: string;
  message: string;
  level: string;
  timestamp: string;
  isProcessed: boolean;
}

// 日常事件报告数据类型
interface DailyEventReportData {
  date: string;
  eventCount: number;
  alarmCount: number;
  resolvedCount: number;
}

// 状态数据列表
const speedFilterList = ref<SpeedFilterData[]>([]);
const parkingStatsList = ref<ParkingStatsData[]>([]);
const onlineStatsList = ref<OnlineStatsData[]>([]);
const mileageStatsList = ref<MileageStatsData[]>([]);
const statusQueryList = ref<VehicleStatusData[]>([]);
const alarmRecordsList = ref<AlarmRecordData[]>([]);
const vehicleInfoList = ref<VehicleStatusData[]>([]);
const dailyEventReportList = ref<DailyEventReportData[]>([]);
const trackPlaybackList = ref<VehicleStatusData[]>([]);

// 轨迹回放相关

// 加载车辆列表
const loadVehicleList = async () => {
  try {
    const response = await vehicleApi.getAll();
    vehicleList.value = response.items || [];
  } catch (error) {
    console.error('加载车辆列表失败:', error);
    ElMessage.error('加载车辆列表失败');
  }
};

// 搜索数据
const searchData = async () => {
  try {
    // 根据不同的查询类型调用不同的API
    switch (activeTab.value) {
      case 'speedFilter':
        const response1 = (await api.get('/api/reports/speed-filter', {
          params: {
            vehicleId: searchForm.vehicleId,
            startTime: searchForm.startTime,
            endTime: searchForm.endTime,
            page: currentPage.value,
            pageSize: pageSize.value,
          },
        })) as GenericPaginatedResponse<SpeedFilterData>;
        speedFilterList.value = response1.data?.items || [];
        total.value = response1.data?.total || 0;
        break;
      case 'parkingStats':
        const response2 = (await api.get('/api/reports/parking-stats', {
          params: {
            vehicleId: searchForm.vehicleId,
            startTime: searchForm.startTime,
            endTime: searchForm.endTime,
            page: currentPage.value,
            pageSize: pageSize.value,
          },
        })) as GenericPaginatedResponse<ParkingStatsData>;
        parkingStatsList.value = response2.data?.items || [];
        total.value = response2.data?.total || 0;
        break;
      case 'onlineStats':
        const response3 = (await api.get('/api/reports/online-stats', {
          params: {
            vehicleId: searchForm.vehicleId,
            startTime: searchForm.startTime,
            endTime: searchForm.endTime,
            page: currentPage.value,
            pageSize: pageSize.value,
          },
        })) as GenericPaginatedResponse<OnlineStatsData>;
        onlineStatsList.value = response3.data?.items || [];
        total.value = response3.data?.total || 0;
        break;
      case 'mileageStats':
        const response4 = (await api.get('/api/reports/mileage-stats', {
          params: {
            vehicleId: searchForm.vehicleId,
            startTime: searchForm.startTime,
            endTime: searchForm.endTime,
            page: currentPage.value,
            pageSize: pageSize.value,
          },
        })) as GenericPaginatedResponse<MileageStatsData>;
        mileageStatsList.value = response4.data?.items || [];
        total.value = response4.data?.total || 0;
        break;
      case 'statusQuery':
        const response5 = (await api.get('/api/reports/status-query', {
          params: {
            vehicleId: searchForm.vehicleId,
            startTime: searchForm.startTime,
            endTime: searchForm.endTime,
            page: currentPage.value,
            pageSize: pageSize.value,
          },
        })) as GenericPaginatedResponse<VehicleStatusData>;
        statusQueryList.value = response5.data?.items || [];
        total.value = response5.data?.total || 0;
        break;
      case 'alarmRecords':
        const response6 = (await api.get('/api/reports/alarm-records', {
          params: {
            vehicleId: searchForm.vehicleId,
            startTime: searchForm.startTime,
            endTime: searchForm.endTime,
            page: currentPage.value,
            pageSize: pageSize.value,
          },
        })) as GenericPaginatedResponse<AlarmRecordData>;
        alarmRecordsList.value = response6.data?.items || [];
        total.value = response6.data?.total || 0;
        break;
      case 'vehicleInfo':
        const response7 = (await api.get('/api/reports/vehicle-info', {
          params: {
            vehicleId: searchForm.vehicleId,
            page: currentPage.value,
            pageSize: pageSize.value,
          },
        })) as GenericPaginatedResponse<VehicleStatusData>;
        vehicleInfoList.value = response7.data?.items || [];
        total.value = response7.data?.total || 0;
        break;
      case 'dailyEventReport':
        const response8 = (await api.get('/api/reports/daily-event-report', {
          params: {
            vehicleId: searchForm.vehicleId,
            startTime: searchForm.startTime,
            endTime: searchForm.endTime,
            page: currentPage.value,
            pageSize: pageSize.value,
          },
        })) as GenericPaginatedResponse<DailyEventReportData>;
        dailyEventReportList.value = response8.data?.items || [];
        total.value = response8.data?.total || 0;
        break;
      case 'trackPlayback':
        const response9 = (await api.get('/api/reports/track-playback', {
          params: {
            vehicleId: searchForm.vehicleId,
            startTime: searchForm.startTime,
            endTime: searchForm.endTime,
            page: currentPage.value,
            pageSize: pageSize.value,
          },
        })) as GenericPaginatedResponse<VehicleStatusData>;
        trackPlaybackList.value = response9.data?.items || [];
        total.value = response9.data?.total || 0;
        break;
    }
  } catch (error) {
    console.error('查询失败:', error);
    ElMessage.error('查询失败');
  }
};

// 导出数据
const exportData = async () => {
  try {
    const response = (await api.get('/api/reports/export', {
      params: {
        reportType: activeTab.value,
        vehicleId: searchForm.vehicleId,
        startTime: searchForm.startTime,
        endTime: searchForm.endTime,
      },
      responseType: 'blob'
    })) as unknown;

    // 创建下载链接
    const responseData = (response as { data?: Blob }).data;
    if (!responseData) {
      throw new Error('导出数据为空');
    }
    const url = window.URL.createObjectURL(responseData);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${activeTab.value}_report.xlsx`;
    link.click();
    window.URL.revokeObjectURL(url);

    ElMessage.success('导出成功');
  } catch (error) {
    console.error('导出失败:', error);
    ElMessage.error('导出失败');
  }
};

// 分页处理
const handleSizeChange = (size: number) => {
  pageSize.value = size;
  searchData();
};

const handleCurrentChange = (current: number) => {
  currentPage.value = current;
  searchData();
};

// 组件挂载时初始化
onMounted(() => {
  loadVehicleList();
  searchData();
});
</script>

<style scoped>
.status-query {
  padding: 20px;
}

.pagination-container {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
}
</style>



