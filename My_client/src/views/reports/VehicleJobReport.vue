<template>
  <div class="vehicle-job-report">
    <el-card shadow="hover">
      <template #header>
        <div class="card-header">
          <span>������ҵ����</span>
        </div>
      </template>
      <div class="report-content">
        <div class="search-params">
          <el-date-picker
            v-model="searchForm.dateRange"
            type="daterange"
            range-separator="��"
            start-placeholder="��ʼ����"
            end-placeholder="��������"
            format="YYYY-MM-DD"
            value-format="YYYY-MM-DD"
            style="width: 300px; margin-right: 10px;"
          />
          <el-select
            v-model="searchForm.vehicleId"
            placeholder="ѡ����"
            style="width: 180px; margin-right: 10px;"
          >
            <el-option label="ȫ��" value="" />
            <el-option
              v-for="vehicle in vehicles"
              :key="vehicle.vehicle_id"
              :label="vehicle.license_plate"
              :value="vehicle.vehicle_id"
            />
          </el-select>
          <el-button type="primary" @click="loadReportData" :loading="loading">
            ��ѯ
          </el-button>
          <el-button @click="exportReport">
            ����
          </el-button>
        </div>
        <div class="report-table">
          <el-table :data="reportData" style="width: 100%">
            <el-table-column type="index" label="���" width="80" />
            <el-table-column prop="vehicleId" label="����ID" width="120" />
            <el-table-column prop="licensePlate" label="���ƺ�" />
            <el-table-column prop="jobCount" label="��ҵ����" />
            <el-table-column prop="totalDistance" label="�����(km)" />
            <el-table-column prop="totalTime" label="��ʱ��(h)" />
            <el-table-column prop="totalWeight" label="������(kg)" />
            <el-table-column prop="averageSpeed" label="ƽ���ٶ�(km/h)" />
            <el-table-column prop="fuelConsumption" label="�ͺ�(L)" />
            <el-table-column prop="efficiency" label="��ҵЧ��" />
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

// ������������
interface VehicleJobReportItem {
  vehicleId: number;
  licensePlate: string;
  jobCount: number;
  totalDistance: number;
  totalTime: number;
  totalWeight: number;
  averageSpeed: number;
  fuelConsumption: number;
  efficiency: number;
}

// ��������
const searchForm = ref({
  dateRange: [] as string[],
  vehicleId: ''
});

// �����б�
const vehicles = ref<VehicleItem[]>([]);

// ��������
const reportData = ref<VehicleJobReportItem[]>([]);

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
    console.error('���س�������ʧ��:', error);
    ElMessage.error('���س�������ʧ��');
  }
};

// ���ر�������
const loadReportData = async () => {
  loading.value = true;
  try {
    // ������ѯ����
    const params = new URLSearchParams();
    if (searchForm.value.dateRange && searchForm.value.dateRange.length === 2) {
      params.append('start_date', searchForm.value.dateRange[0]);
      params.append('end_date', searchForm.value.dateRange[1]);
    }
    if (searchForm.value.vehicleId) {
      params.append('vehicle_id', searchForm.value.vehicleId);
    }
    
    // ������ʵAPI��ȡ��������
    const response = await fetch(`/api/reports/vehicle-job?${params.toString()}`, {
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
    ElMessage.success('���ر������ݳɹ�');
  } catch (error) {
    console.error('���ر�������ʧ��:', error);
    ElMessage.error('���ر�������ʧ��');
  } finally {
    loading.value = false;
  }
};

// ��������
const exportReport = () => {
  ElMessage.info('�����������ܿ�����');
};

// �������ʱ��������
onMounted(() => {
  loadVehicles();
});
</script>

<style scoped>
.vehicle-job-report {
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

