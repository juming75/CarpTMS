<template>
  <div class="fuel-consumption-report">
    <el-card shadow="hover">
      <template #header>
        <div class="card-header">
          <span>娌硅€楁姤锟?/span>
        </div>
      </template>
      <div class="report-content">
        <div class="search-params">
          <el-date-picker
            v-model="searchForm.dateRange"
            type="daterange"
            range-separator="锟?
            start-placeholder="寮€濮嬫棩锟?
            end-placeholder="缁撴潫鏃ユ湡"
            format="YYYY-MM-DD"
            value-format="YYYY-MM-DD"
            style="width: 300px; margin-right: 10px;"
          />
          <el-select
            v-model="searchForm.vehicleId"
            placeholder="閫夋嫨杞﹁締"
            style="width: 180px; margin-right: 10px;"
          >
            <el-option label="鍏ㄩ儴" value="" />
            <el-option
              v-for="vehicle in vehicles"
              :key="vehicle.vehicle_id"
              :label="vehicle.license_plate"
              :value="vehicle.vehicle_id"
            />
          </el-select>
          <el-button type="primary" @click="loadReportData" :loading="loading">
            鏌ヨ
          </el-button>
          <el-button @click="exportReport">
            瀵煎嚭
          </el-button>
        </div>
        <div class="report-table">
          <el-table :data="reportData" style="width: 100%">
            <el-table-column type="index" label="搴忓彿" width="80" />
            <el-table-column prop="vehicleId" label="杞﹁締ID" width="120" />
            <el-table-column prop="licensePlate" label="杞︾墝锟? />
            <el-table-column prop="driver" label="鍙告満" />
            <el-table-column prop="startTime" label="寮€濮嬫椂锟? width="180" />
            <el-table-column prop="endTime" label="缁撴潫鏃堕棿" width="180" />
            <el-table-column prop="distance" label="琛岄┒閲岀▼(km)" />
            <el-table-column prop="fuelConsumption" label="娌癸拷?L)" />
            <el-table-column prop="fuelEfficiency" label="鐕冩补鏁堢巼(L/100km)" />
            <el-table-column prop="cost" label="鐕冩补鎴愭湰(锟?" />
          </el-table>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted } from 'vue';
import { ElMessage, ElDatePicker, ElSelect, ElOption, ElButton, ElTable, ElTableColumn, ElCard } from 'element-plus';
import api from '@/api';

// 杞﹁締绫诲瀷
interface VehicleItem {
  vehicle_id: number;
  license_plate: string;
}

// 鎶ヨ〃鏁版嵁绫诲瀷
interface FuelConsumptionReportItem {
  vehicleId: number;
  licensePlate: string;
  driver: string;
  startTime: string;
  endTime: string;
  distance: number;
  fuelConsumption: number;
  fuelEfficiency: number;
  cost: number;
}

// 鎼滅储琛ㄥ崟
const searchForm = ref({
  dateRange: [] as string[],
  vehicleId: ''
});

// 杞﹁締鍒楄〃
const vehicles = ref<VehicleItem[]>([]);

// 鎶ヨ〃鏁版嵁
const reportData = ref<FuelConsumptionReportItem[]>([]);

// 鍔犺浇鐘讹拷?
const loading = ref(false);

// 鍔犺浇杞﹁締鏁版嵁
const loadVehicles = async () => {
  try {
    const response = await api.get('/api/vehicles') as any;
    if (response && response.items) {
      vehicles.value = response.items || [];
    }
  } catch (error) {
    console.error('鍔犺浇杞﹁締鏁版嵁澶辫触:', error);
    ElMessage.error('鍔犺浇杞﹁締鏁版嵁澶辫触');
  }
};

// 鍔犺浇鎶ヨ〃鏁版嵁
const loadReportData = async () => {
  loading.value = true;
  try {
    // 鏋勫缓鏌ヨ鍙傛暟
    const params = new URLSearchParams();
    if (searchForm.value.dateRange && searchForm.value.dateRange.length === 2) {
      params.append('start_date', searchForm.value.dateRange[0]);
      params.append('end_date', searchForm.value.dateRange[1]);
    }
    if (searchForm.value.vehicleId) {
      params.append('vehicle_id', searchForm.value.vehicleId);
    }
    
    // 璋冪敤鐪熷疄API鑾峰彇鎶ヨ〃鏁版嵁
    const response = await fetch(`/api/reports/fuel-consumption?${params.toString()}`, {
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
    ElMessage.success('鍔犺浇鎶ヨ〃鏁版嵁鎴愬姛');
  } catch (error) {
    console.error('鍔犺浇鎶ヨ〃鏁版嵁澶辫触:', error);
    ElMessage.error('鍔犺浇鎶ヨ〃鏁版嵁澶辫触');
  } finally {
    loading.value = false;
  }
};

// 瀵煎嚭鎶ヨ〃
const exportReport = () => {
  ElMessage.info('瀵煎嚭鎶ヨ〃鍔熻兘寮€鍙戜腑');
};

// 缁勪欢鎸傝浇鏃跺姞杞芥暟锟?
onMounted(() => {
  loadVehicles();
});
</script>

<style scoped>
.fuel-consumption-report {
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


