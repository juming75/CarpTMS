<template>
  <div class="unloading-job-report">
    <el-card shadow="hover">
      <template #header>
        <div class="card-header">
          <span>卸载作业报表</span>
        </div>
      </template>
      <div class="report-content">
        <div class="search-params">
          <el-date-picker
            v-model="searchForm.dateRange"
            type="daterange"
            range-separator="至"
            start-placeholder="开始日期"
            end-placeholder="结束日期"
            format="YYYY-MM-DD"
            value-format="YYYY-MM-DD"
            style="width: 300px; margin-right: 10px;"
          />
          <el-select
            v-model="searchForm.vehicleId"
            placeholder="选择车辆"
            style="width: 180px; margin-right: 10px;"
          >
            <el-option label="全部" value="" />
            <el-option
              v-for="vehicle in vehicles"
              :key="vehicle.vehicle_id"
              :label="vehicle.license_plate"
              :value="vehicle.vehicle_id"
            />
          </el-select>
          <el-select
            v-model="searchForm.locationId"
            placeholder="选择卸载地点"
            style="width: 180px; margin-right: 10px;"
          >
            <el-option label="全部" value="" />
            <el-option
              v-for="location in locations"
              :key="location.id"
              :label="location.name"
              :value="location.id"
            />
          </el-select>
          <el-button type="primary" @click="loadReportData" :loading="loading">
            查询
          </el-button>
          <el-button @click="exportReport">
            导出
          </el-button>
        </div>
        <div class="report-table">
          <el-table :data="reportData" style="width: 100%">
            <el-table-column type="index" label="序号" width="80" />
            <el-table-column prop="jobId" label="作业ID" width="120" />
            <el-table-column prop="vehicleId" label="车辆ID" width="120" />
            <el-table-column prop="licensePlate" label="车牌号" />
            <el-table-column prop="driver" label="司机" />
            <el-table-column prop="locationId" label="地点ID" width="120" />
            <el-table-column prop="locationName" label="卸载地点" />
            <el-table-column prop="time" label="卸载时间" width="180" />
            <el-table-column prop="weight" label="卸载重量(kg)" />
            <el-table-column prop="duration" label="卸载时长(min)" />
            <el-table-column prop="status" label="状态" />
          </el-table>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage, ElDatePicker, ElSelect, ElOption, ElButton, ElTable, ElTableColumn, ElCard } from 'element-plus';
import api from '@/api';

// ��������
interface VehicleItem {
  vehicle_id: number;
  license_plate: string;
}

// �ص�����
interface LocationItem {
  id: number;
  name: string;
}

// ������������
interface UnloadingJobReportItem {
  jobId: number;
  vehicleId: number;
  licensePlate: string;
  driver: string;
  locationId: number;
  locationName: string;
  time: string;
  weight: number;
  duration: number;
  status: string;
}

// ��������
const searchForm = ref({
  dateRange: [] as string[],
  vehicleId: '',
  locationId: ''
});

// �����б�
const vehicles = ref<VehicleItem[]>([]);

// �ص��б�
const locations = ref<LocationItem[]>([]);

// ��������
const reportData = ref<UnloadingJobReportItem[]>([]);

// ����״̬
const loading = ref(false);

// ���س�������
const loadVehicles = async () => {
  try {
    const response = await api.get('/api/vehicles') as any;
    if (response && response.items) {
      vehicles.value = response.items || [];
    }
  } catch (error) {
    console.error('加载车辆列表失败:', error);
    ElMessage.error('加载车辆列表失败');
  }
};

// ���صص�����
const loadLocations = async () => {
  try {
    // ������ʵAPI��ȡ�ص�����
    const response = await fetch('/api/location/places', {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${localStorage.getItem('access_token') || sessionStorage.getItem('access_token')}`
      }
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const data = await response.json();
    locations.value = data?.items || [];
  } catch (error) {
    console.error('加载地点列表失败:', error);
    ElMessage.error('加载地点列表失败');
  }
};

// 加载报表数据
const loadReportData = async () => {
  loading.value = true;
  try {
    // 构建查询参数
    const params = new URLSearchParams();
    if (searchForm.value.dateRange && searchForm.value.dateRange.length === 2) {
      params.append('start_date', searchForm.value.dateRange[0]);
      params.append('end_date', searchForm.value.dateRange[1]);
    }
    if (searchForm.value.vehicleId) {
      params.append('vehicle_id', searchForm.value.vehicleId);
    }
    if (searchForm.value.locationId) {
      params.append('location_id', searchForm.value.locationId);
    }
    
    // 调用实际API获取报表数据
    const response = await fetch(`/api/reports/unloading-job?${params.toString()}`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${localStorage.getItem('access_token') || sessionStorage.getItem('access_token')}`
      }
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const data = await response.json();
    reportData.value = data?.items || [];
    ElMessage.success('加载报表数据成功');
  } catch (error) {
    console.error('加载报表数据失败:', error);
    ElMessage.error('加载报表数据失败');
  } finally {
    loading.value = false;
  }
};

// 导出报表
const exportReport = () => {
  ElMessage.info('导出功能开发中');
};

// 组件挂载时加载数据
onMounted(() => {
  loadVehicles();
  loadLocations();
});
</script>

<style scoped>
.unloading-job-report {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.search-params {
  margin-bottom: 20px;
  display: flex;
  align-items: center;
}

.report-table {
  margin-top: 20px;
}
</style>

