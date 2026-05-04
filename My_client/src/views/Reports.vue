/* global Blob */
<template>
  <div class="reports">
    <!-- 显示子路由内容 -->
    <router-view v-slot="{ Component }">
      <keep-alive>
        <component :is="Component" />
      </keep-alive>
    </router-view>

    <!-- 当访问的是 /reports 或 /reports/status 时显示报表中心内容 -->
    <el-card v-if="$route.path === '/reports' || $route.path === '/reports/status'">
      <template #header>
        <div class="card-header">
          <span>报表中心</span>
          <el-button type="primary" size="small" @click="generateReport">生成报表</el-button>
          <el-dropdown @command="exportReport">
            <el-button size="small">
              导出报表
              <el-icon class="el-icon--right"><arrow-down /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="docx">导出DOCX</el-dropdown-item>
                <el-dropdown-item command="xlsx">导出XLSX</el-dropdown-item>
                <el-dropdown-item command="pdf">导出PDF</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </template>

      <!-- 报表类型选择 -->
      <el-row :gutter="20" class="mb-4">
        <el-col :span="8">
          <el-form-item label="报表类型" label-width="80px">
            <el-select
              v-model="reportForm.reportType"
              placeholder="请选择报表类型"
              style="width: 100%"
              @change="handleReportTypeChange"
            >
              <el-option v-for="type in reportTypes" :key="type.value" :label="type.label" :value="type.value" />
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="8">
          <el-form-item label="时间范围" label-width="80px">
            <el-select v-model="reportForm.timeRange" placeholder="请选择时间范围" style="width: 100%">
              <el-option label="今日" value="today" />
              <el-option label="本周" value="week" />
              <el-option label="本月" value="month" />
              <el-option label="自定义" value="custom" />
            </el-select>
          </el-form-item>
        </el-col>
        <el-col :span="8">
          <el-form-item label="车辆选择" label-width="80px">
            <el-select v-model="reportForm.vehicleIds" multiple filterable placeholder="请选择车辆" style="width: 100%">
              <el-option
                v-for="vehicle in vehicleList"
                :key="vehicle.id"
                :label="vehicle.licensePlate"
                :value="vehicle.id"
              />
            </el-select>
          </el-form-item>
        </el-col>
      </el-row>

      <!-- 自定义时间范围 -->
      <el-row :gutter="20" class="mb-4" v-if="reportForm.timeRange === 'custom'">
        <el-col :span="12">
          <el-form-item label="开始时间" label-width="80px">
            <el-date-picker
              v-model="reportForm.startTime"
              type="datetime"
              placeholder="选择开始时间"
              style="width: 100%"
            />
          </el-form-item>
        </el-col>
        <el-col :span="12">
          <el-form-item label="结束时间" label-width="80px">
            <el-date-picker
              v-model="reportForm.endTime"
              type="datetime"
              placeholder="选择结束时间"
              style="width: 100%"
            />
          </el-form-item>
        </el-col>
      </el-row>

      <!-- 报表数据展示 -->
      <el-tabs v-model="activeReportTab" type="border-card" class="report-tabs">
        <!-- 文件记录报表 -->
        <el-tab-pane label="文件记录报表" name="fileRecord">
          <el-table v-if="fileRecordData.length > 0" :data="fileRecordData" stripe style="width: 100%">
            <el-table-column prop="id" label="序号" width="80" />
            <el-table-column prop="fileName" label="文件名" width="180" />
            <el-table-column prop="fileType" label="文件类型" width="100" />
            <el-table-column prop="fileSize" label="文件大小" width="100" />
            <el-table-column prop="uploadTime" label="上传时间" width="180" />
            <el-table-column prop="operator" label="操作人" width="120" />
            <el-table-column prop="status" label="状态" width="100" />
            <el-table-column prop="remark" label="备注" min-width="150" />
          </el-table>
          <el-empty v-else description="暂无文件记录数据" />
        </el-tab-pane>

        <!-- 三超报表 -->
        <el-tab-pane label="三超报表" name="threeOver">
          <el-tabs type="card">
            <el-tab-pane label="超限明细">
              <el-table v-if="overLimitData.length > 0" :data="overLimitData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="vehicleNo" label="车牌号" width="120" />
                <el-table-column prop="driver" label="驾驶员" width="100" />
                <el-table-column prop="time" label="时间" width="180" />
                <el-table-column prop="location" label="位置" min-width="150" />
                <el-table-column prop="overType" label="超限类型" width="120" />
                <el-table-column prop="actualValue" label="实际值" width="120" />
                <el-table-column prop="limitValue" label="限制值" width="120" />
                <el-table-column prop="overPercentage" label="超限百分比" width="120" />
                <el-table-column prop="remark" label="备注" min-width="150" />
              </el-table>
              <el-empty v-else description="暂无超限数据" />
            </el-tab-pane>
            <el-tab-pane label="超载明细">
              <el-table v-if="overLoadData.length > 0" :data="overLoadData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="vehicleNo" label="车牌号" width="120" />
                <el-table-column prop="driver" label="驾驶员" width="100" />
                <el-table-column prop="time" label="时间" width="180" />
                <el-table-column prop="location" label="位置" min-width="150" />
                <el-table-column prop="actualWeight" label="实际重量(kg)" width="120" />
                <el-table-column prop="limitWeight" label="限制重量(kg)" width="120" />
                <el-table-column prop="overWeight" label="超载重量(kg)" width="120" />
                <el-table-column prop="overPercentage" label="超载百分比" width="120" />
                <el-table-column prop="remark" label="备注" min-width="150" />
              </el-table>
              <el-empty v-else description="暂无超载数据" />
            </el-tab-pane>
          </el-tabs>
        </el-tab-pane>

        <!-- 作业报表 -->
        <el-tab-pane label="作业报表" name="job">
          <el-tabs type="card">
            <el-tab-pane label="单车作业报表">
              <el-table v-if="singleVehicleJobData.length > 0" :data="singleVehicleJobData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="vehicleNo" label="车牌号" width="120" />
                <el-table-column prop="driver" label="驾驶员" width="100" />
                <el-table-column prop="jobCount" label="作业次数" width="100" />
                <el-table-column prop="totalWeight" label="总重量(t)" width="100" />
                <el-table-column prop="totalDistance" label="总距离(km)" width="120" />
                <el-table-column prop="totalTime" label="总时长" width="120" />
                <el-table-column prop="avgWeight" label="平均重量(t)" width="120" />
                <el-table-column prop="efficiency" label="效率" width="100" />
              </el-table>
              <el-empty v-else description="暂无单车作业数据" />
            </el-tab-pane>
            <el-tab-pane label="站点作业报表">
              <el-table v-if="siteJobData.length > 0" :data="siteJobData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="siteName" label="站点名称" width="150" />
                <el-table-column prop="jobCount" label="作业次数" width="100" />
                <el-table-column prop="totalWeight" label="总重量(t)" width="100" />
                <el-table-column prop="totalVehicles" label="总车辆数" width="120" />
                <el-table-column prop="avgWeight" label="平均重量(t)" width="120" />
                <el-table-column prop="efficiency" label="效率" width="100" />
              </el-table>
              <el-empty v-else description="暂无站点作业数据" />
            </el-tab-pane>
            <el-tab-pane label="装货作业报表">
              <el-table v-if="loadingJobData.length > 0" :data="loadingJobData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="vehicleNo" label="车牌号" width="120" />
                <el-table-column prop="driver" label="驾驶员" width="100" />
                <el-table-column prop="loadingTime" label="装货时间" width="180" />
                <el-table-column prop="siteName" label="站点名称" width="150" />
                <el-table-column prop="weight" label="重量(t)" width="100" />
                <el-table-column prop="duration" label="时长" width="100" />
                <el-table-column prop="remark" label="备注" min-width="150" />
              </el-table>
              <el-empty v-else description="暂无装货作业数据" />
            </el-tab-pane>
            <el-tab-pane label="卸货作业报表">
              <el-table v-if="unloadingJobData.length > 0" :data="unloadingJobData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="vehicleNo" label="车牌号" width="120" />
                <el-table-column prop="driver" label="驾驶员" width="100" />
                <el-table-column prop="unloadingTime" label="卸货时间" width="180" />
                <el-table-column prop="siteName" label="站点名称" width="150" />
                <el-table-column prop="weight" label="重量(t)" width="100" />
                <el-table-column prop="duration" label="时长" width="100" />
                <el-table-column prop="remark" label="备注" min-width="150" />
              </el-table>
              <el-empty v-else description="暂无卸货作业数据" />
            </el-tab-pane>
            <el-tab-pane label="运输作业报表">
              <el-table v-if="transportJobData.length > 0" :data="transportJobData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="vehicleNo" label="车牌号" width="120" />
                <el-table-column prop="driver" label="驾驶员" width="100" />
                <el-table-column prop="startTime" label="开始时间" width="180" />
                <el-table-column prop="endTime" label="结束时间" width="180" />
                <el-table-column prop="startSite" label="起点" width="150" />
                <el-table-column prop="endSite" label="终点" width="150" />
                <el-table-column prop="distance" label="距离(km)" width="120" />
                <el-table-column prop="weight" label="重量(t)" width="100" />
                <el-table-column prop="duration" label="时长" width="100" />
              </el-table>
              <el-empty v-else description="暂无运输作业数据" />
            </el-tab-pane>
            <el-tab-pane label="区域作业报表">
              <el-table v-if="areaJobData.length > 0" :data="areaJobData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="areaName" label="区域名称" width="150" />
                <el-table-column prop="jobCount" label="作业次数" width="100" />
                <el-table-column prop="totalWeight" label="总重量(t)" width="100" />
                <el-table-column prop="totalVehicles" label="总车辆数" width="120" />
                <el-table-column prop="avgWeight" label="平均重量(t)" width="120" />
                <el-table-column prop="efficiency" label="效率" width="100" />
              </el-table>
              <el-empty v-else description="暂无区域作业数据" />
            </el-tab-pane>
          </el-tabs>
        </el-tab-pane>

        <!-- 运行报表 -->
        <el-tab-pane label="运行报表" name="run">
          <el-tabs type="card">
            <el-tab-pane label="超速报表">
              <el-table v-if="speedingData.length > 0" :data="speedingData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="vehicleNo" label="车牌号" width="120" />
                <el-table-column prop="driver" label="驾驶员" width="100" />
                <el-table-column prop="time" label="时间" width="180" />
                <el-table-column prop="location" label="位置" min-width="150" />
                <el-table-column prop="speed" label="速度(km/h)" width="120" />
                <el-table-column prop="limitSpeed" label="限速(km/h)" width="120" />
                <el-table-column prop="overSpeed" label="超速(km/h)" width="120" />
                <el-table-column prop="duration" label="时长" width="100" />
              </el-table>
              <el-empty v-else description="暂无超速数据" />
            </el-tab-pane>
            <el-tab-pane label="车辆运行报表">
              <el-table v-if="vehicleRunData.length > 0" :data="vehicleRunData" stripe style="width: 100%">
                <el-table-column prop="id" label="序号" width="80" />
                <el-table-column prop="vehicleNo" label="车牌号" width="120" />
                <el-table-column prop="driver" label="驾驶员" width="100" />
                <el-table-column prop="startTime" label="开始时间" width="180" />
                <el-table-column prop="endTime" label="结束时间" width="180" />
                <el-table-column prop="runTime" label="运行时长" width="120" />
                <el-table-column prop="distance" label="行驶距离(km)" width="120" />
                <el-table-column prop="avgSpeed" label="平均速度(km/h)" width="120" />
                <el-table-column prop="maxSpeed" label="最大速度(km/h)" width="120" />
                <el-table-column prop="fuelConsumption" label="耗油量(L)" width="120" />
              </el-table>
              <el-empty v-else description="暂无车辆运行数据" />
            </el-tab-pane>
          </el-tabs>
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
import { ArrowDown } from '@element-plus/icons-vue';


// 后端车辆数据类型
interface BackendVehicle {
  vehicle_id: number;
  license_plate: string;
}

// API响应类型
interface ApiResponse<T = unknown> {
  code: number;
  message: string;
  data?: T;
}

// 分页响应类型
interface PaginatedResponse<T = unknown> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  pages?: number;
}

// 报表数据响应类型
type ReportDataResponse = ApiResponse<PaginatedResponse<unknown>>;

// 车辆API类型
interface VehicleApi {
  getAll: (params?: Record<string, unknown>) => Promise<ApiResponse<PaginatedResponse<BackendVehicle>>>;
}

// 文件记录报表数据
interface FileRecordData {
  id: number;
  fileName: string;
  fileType: string;
  fileSize: number;
  uploadTime: string;
  [key: string]: unknown;
}

// 车辆运行报表数据
interface CarRunData {
  id: number;
  vehicleId: number;
  licensePlate: string;
  runTime: number;
  runDistance: number;
  avgSpeed: number;
  [key: string]: unknown;
}

// 车辆停车报表数据
interface CarStopData {
  id: number;
  vehicleId: number;
  licensePlate: string;
  stopTime: number;
  stopCount: number;
  [key: string]: unknown;
}

// 车辆称重报表数据
interface CarWeightData {
  id: number;
  vehicleId: number;
  licensePlate: string;
  weightTime: string;
  grossWeight: number;
  tareWeight: number;
  netWeight: number;
  [key: string]: unknown;
}

// 超速报表数据
interface OverSpeedData {
  id: number;
  vehicleId: number;
  licensePlate: string;
  speedTime: string;
  maxSpeed: number;
  [key: string]: unknown;
}

// 油料报表数据
interface OilReportData {
  id: number;
  vehicleId: number;
  licensePlate: string;
  oilTime: string;
  oilConsumption: number;
  [key: string]: unknown;
}

// 导出API响应类型
interface ExportResponse {
  success: boolean;
  data: unknown;
  message?: string;
}

// 报表API类型
interface ReportApi {
  getSalesReport: (params?: Record<string, unknown>) => Promise<ApiResponse<unknown>>;
  getLogisticsReport: (params?: Record<string, unknown>) => Promise<ApiResponse<unknown>>;
  getVehicleReport: (params?: Record<string, unknown>) => Promise<ApiResponse<unknown>>;
  getFinanceReport: (params?: Record<string, unknown>) => Promise<ApiResponse<unknown>>;
  exportReport: (params?: Record<string, unknown>) => Promise<ApiResponse<unknown>>;
  export: (params?: Record<string, unknown>) => Promise<ExportResponse>;
  getData: (params?: Record<string, unknown>) => Promise<ReportDataResponse>;
}

// 动态导入API
let vehicleApi: VehicleApi | null = null;
let reportApi: ReportApi | null = null;
async function importApi() {
  if (!vehicleApi) {
    const module = await import('@/api');
    vehicleApi = module.vehicleApi;
    reportApi = module.reportApi;
  }
  return { vehicleApi, reportApi };
}

// 车辆列表项类型
interface VehicleListItem {
  id: number;
  licensePlate: string;
}

// 车辆列表
const vehicleList = ref<VehicleListItem[]>([]);

// 加载车辆列表
const loadVehicleList = async () => {
  try {
    // 动态导入API
    const { vehicleApi } = await importApi();

    // 从后端获取车辆列表
    const response = await vehicleApi.getAll();
    if (response && response.data && response.data.list) {
      vehicleList.value = response.data.list.map((vehicle: BackendVehicle) => ({
        id: vehicle.vehicle_id,
        licensePlate: vehicle.license_plate,
      }));
    }
  } catch (error) {
    console.error('加载车辆列表失败:', error);
    ElMessage.error('加载车辆列表失败');
  }
};

// 报表类型选项
const reportTypes = [
  { label: '文件记录报表', value: 'fileRecord' },
  { label: '三超报表', value: 'threeOver' },
  { label: '作业报表', value: 'job' },
  { label: '运行报表', value: 'run' },
];

// 活动的报表类型
const activeReportTab = ref('fileRecord');

// 报表表单
const reportForm = reactive({
  reportType: 'fileRecord',
  timeRange: 'month',
  vehicleIds: [],
  startTime: new Date(new Date().getFullYear(), new Date().getMonth(), 1),
  endTime: new Date(),
});

// 分页
const currentPage = ref(1);
const pageSize = ref(10);
const total = ref(0);

// 三超报表数据
interface OverLimitData {
  id: number;
  vehicleNo: string;
  driver: string;
  time: string;
  location: string;
  overType: string;
  actualValue: number;
  limitValue: number;
  overPercentage: number;
  remark: string;
}

interface OverLoadData {
  id: number;
  vehicleNo: string;
  driver: string;
  time: string;
  location: string;
  actualWeight: number;
  limitWeight: number;
  overWeight: number;
  overPercentage: number;
  remark: string;
}

// 作业报表数据
interface SingleVehicleJobData {
  id: number;
  vehicleNo: string;
  driver: string;
  jobCount: number;
  totalWeight: number;
  totalDistance: number;
  totalTime: string;
  avgWeight: number;
  efficiency: string;
}

interface SiteJobData {
  id: number;
  siteName: string;
  jobCount: number;
  totalWeight: number;
  totalVehicles: number;
  avgWeight: number;
  efficiency: string;
}

interface LoadingJobData {
  id: number;
  vehicleNo: string;
  driver: string;
  loadingTime: string;
  siteName: string;
  weight: number;
  duration: string;
  remark: string;
}

interface UnloadingJobData {
  id: number;
  vehicleNo: string;
  driver: string;
  unloadingTime: string;
  siteName: string;
  weight: number;
  duration: string;
  remark: string;
}

interface TransportJobData {
  id: number;
  vehicleNo: string;
  driver: string;
  startTime: string;
  endTime: string;
  startSite: string;
  endSite: string;
  distance: number;
  weight: number;
  duration: string;
}

interface AreaJobData {
  id: number;
  areaName: string;
  jobCount: number;
  totalWeight: number;
  totalVehicles: number;
  avgWeight: number;
  efficiency: string;
}

// 运行报表数据
interface SpeedingData {
  id: number;
  vehicleNo: string;
  driver: string;
  time: string;
  location: string;
  speed: number;
  limitSpeed: number;
  overSpeed: number;
  duration: string;
}

interface VehicleRunData {
  id: number;
  vehicleNo: string;
  driver: string;
  startTime: string;
  endTime: string;
  runTime: string;
  distance: number;
  avgSpeed: number;
  maxSpeed: number;
  fuelConsumption: number;
}

// 报表数据
const fileRecordData = ref<FileRecordData[]>([]);
const overLimitData = ref<OverLimitData[]>([]);
const overLoadData = ref<OverLoadData[]>([]);
const singleVehicleJobData = ref<SingleVehicleJobData[]>([]);
const siteJobData = ref<SiteJobData[]>([]);
const loadingJobData = ref<LoadingJobData[]>([]);
const unloadingJobData = ref<UnloadingJobData[]>([]);
const transportJobData = ref<TransportJobData[]>([]);
const areaJobData = ref<AreaJobData[]>([]);
const speedingData = ref<SpeedingData[]>([]);
const vehicleRunData = ref<VehicleRunData[]>([]);

// 报表类型变化处理
const handleReportTypeChange = (type: string) => {
  activeReportTab.value = type;
};

// 生成报表
const generateReport = async () => {
  try {
    // 动态导入API
    const { reportApi } = await importApi();

    // 从后端获取报表数据
    const response = await reportApi.getData({
      reportType: activeReportTab.value,
      timeRange: reportForm.timeRange,
      startTime: reportForm.startTime,
      endTime: reportForm.endTime,
      vehicleIds: reportForm.vehicleIds,
      page: currentPage.value,
      pageSize: pageSize.value,
    });

    if (response) {
      const paginatedData = response as PaginatedResponse<unknown>;
      if (paginatedData.items) {
        // 更新报表数据
        switch (activeReportTab.value) {
          case 'fileRecord':
            fileRecordData.value = (paginatedData.items as FileRecordData[]) || [];
            break;
          case 'threeOver':
            // 三超报表包含多个子报表，这里暂时统一处理
            overLimitData.value = (paginatedData.items as OverLimitData[]) || [];
            overLoadData.value = (paginatedData.items as OverLoadData[]) || [];
            break;
          case 'job':
            // 作业报表包含多个子报表，这里暂时统一处理
            singleVehicleJobData.value = (paginatedData.items as SingleVehicleJobData[]) || [];
            siteJobData.value = (paginatedData.items as SiteJobData[]) || [];
            loadingJobData.value = (paginatedData.items as LoadingJobData[]) || [];
            unloadingJobData.value = (paginatedData.items as UnloadingJobData[]) || [];
            transportJobData.value = (paginatedData.items as TransportJobData[]) || [];
            areaJobData.value = (paginatedData.items as AreaJobData[]) || [];
            break;
          case 'run':
            // 运行报表包含多个子报表，这里暂时统一处理
            speedingData.value = (paginatedData.items as SpeedingData[]) || [];
            vehicleRunData.value = (paginatedData.items as VehicleRunData[]) || [];
            break;
        }

        // 更新总数
        total.value = paginatedData.total || 0;
      }
    } else {
      ElMessage.error('获取报表数据失败');
    }
  } catch (error) {
    console.error('生成报表失败:', error);
    ElMessage.error('生成报表失败');
  }
};

// 导出报表
const exportReport = async (format: string) => {
  try {
    // 动态导入API
    const { reportApi } = await importApi();

    // 构建导出参数
    const exportParams = {
      reportType: activeReportTab.value,
      timeRange: reportForm.timeRange,
      startTime: reportForm.startTime,
      endTime: reportForm.endTime,
      vehicleIds: reportForm.vehicleIds,
      format: format,
    };

    // 调用导出API
    const response = await reportApi.export(exportParams);

    if (response.success) {
      // 创建下载链接
      const blob = new Blob([response.data], { type: getMimeType(format) });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;

      // 生成文件名
      let reportName = '';
      switch (activeReportTab.value) {
        case 'fileRecord':
          reportName = '文件记录报表';
          break;
        case 'threeOver':
          reportName = '三超报表';
          break;
        case 'job':
          reportName = '作业报表';
          break;
        case 'run':
          reportName = '运行报表';
          break;
        default:
          reportName = '综合报表';
      }

      link.download = `${reportName}.${format}`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);

      ElMessage.success('报表导出成功');
    } else {
      ElMessage.error(response.message || '导出报表失败');
    }
  } catch (error) {
    console.error('导出报表失败:', error);
    ElMessage.error('导出报表失败');
  }
};

// 获取文件MIME类型
const getMimeType = (format: string): string => {
  switch (format) {
    case 'docx':
      return 'application/vnd.openxmlformats-officedocument.wordprocessingml.document';
    case 'xlsx':
      return 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet';
    case 'pdf':
      return 'application/pdf';
    default:
      return 'application/octet-stream';
  }
};

// 分页处理
const handleSizeChange = (size: number) => {
  pageSize.value = size;
  generateReport();
};

const handleCurrentChange = (current: number) => {
  currentPage.value = current;
  generateReport();
};

// 组件挂载时初始化
onMounted(async () => {
  // 加载车辆列表
  await loadVehicleList();
  // 生成报表
  await generateReport();
});
</script>

<style scoped>
.reports {
  padding: 20px;
}

.reports-content {
  min-height: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.report-tabs {
  min-height: 400px;
}

.pagination-container {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
}
</style>


