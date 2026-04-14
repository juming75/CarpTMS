<template>
  <div class="area-statistics-report">
    <el-card shadow="hover">
      <template #header>
        <div class="card-header">
          <span>鍖哄煙缁熻鎶ヨ〃</span>
        </div>
      </template>
      <div class="report-content">
        <div class="search-params">
          <el-date-picker
            v-model="searchForm.dateRange"
            type="daterange"
            range-separator="鑷?
            start-placeholder="寮€濮嬫棩鏈?
            end-placeholder="缁撴潫鏃ユ湡"
            format="YYYY-MM-DD"
            value-format="YYYY-MM-DD"
            style="width: 300px; margin-right: 10px;"
          />
          <el-select
            v-model="searchForm.areaId"
            placeholder="閫夋嫨鍖哄煙"
            style="width: 180px; margin-right: 10px;"
          >
            <el-option label="鍏ㄩ儴" value="" />
            <el-option
              v-for="area in areas"
              :key="area.id"
              :label="area.name"
              :value="area.id"
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
            <el-table-column prop="areaId" label="鍖哄煙ID" width="120" />
            <el-table-column prop="areaName" label="鍖哄煙鍚嶇О" />
            <el-table-column prop="vehicleCount" label="杞﹁締鏁? />
            <el-table-column prop="jobCount" label="浣滀笟娆℃暟" />
            <el-table-column prop="totalDistance" label="鎬婚噷绋?km)" />
            <el-table-column prop="totalTime" label="鎬绘椂闂?h)" />
            <el-table-column prop="totalWeight" label="鎬昏浇閲?kg)" />
            <el-table-column prop="averageSpeed" label="骞冲潎閫熷害(km/h)" />
            <el-table-column prop="efficiency" label="浣滀笟鏁堢巼" />
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

// 鍖哄煙绫诲瀷
interface AreaItem {
  id: number;
  name: string;
}

// 鎶ヨ〃鏁版嵁绫诲瀷
interface AreaStatisticsReportItem {
  areaId: number;
  areaName: string;
  vehicleCount: number;
  jobCount: number;
  totalDistance: number;
  totalTime: number;
  totalWeight: number;
  averageSpeed: number;
  efficiency: number;
}

// 鎼滅储琛ㄥ崟
const searchForm = ref({
  dateRange: [] as string[],
  areaId: ''
});

// 鍖哄煙鍒楄〃
const areas = ref<AreaItem[]>([]);

// 鎶ヨ〃鏁版嵁
const reportData = ref<AreaStatisticsReportItem[]>([]);

// 鍔犺浇鐘舵€?
const loading = ref(false);

// 鍔犺浇鍖哄煙鏁版嵁
const loadAreas = async () => {
  try {
    // 璋冪敤鐪熷疄API鑾峰彇鍖哄煙鏁版嵁
    const response = await fetch('/api/areas', {
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
    areas.value = data?.items || [];
  } catch (error) {
    console.error('鍔犺浇鍖哄煙鏁版嵁澶辫触:', error);
    ElMessage.error('鍔犺浇鍖哄煙鏁版嵁澶辫触');
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
    if (searchForm.value.areaId) {
      params.append('area_id', searchForm.value.areaId);
    }
    
    // 璋冪敤鐪熷疄API鑾峰彇鎶ヨ〃鏁版嵁
    const response = await fetch(`/api/reports/area-statistics?${params.toString()}`, {
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

// 缁勪欢鎸傝浇鏃跺姞杞芥暟鎹?
onMounted(() => {
  loadAreas();
});
</script>

<style scoped>
.area-statistics-report {
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


