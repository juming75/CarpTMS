<template>
  <div class="vehicle-jobs">
    <el-breadcrumb separator="/" class="breadcrumb">
      <el-breadcrumb-item><span>�������</span></el-breadcrumb-item>
      <el-breadcrumb-item><span>������ҵ</span></el-breadcrumb-item>
    </el-breadcrumb>

    <el-card shadow="hover" class="card">
      <template #header>
        <div class="card-header">
          <span>������ҵ����</span>
        </div>
      </template>

      <div class="card-content">
        <div class="search-params">
          <el-date-picker
            v-model="searchForm.date"
            type="date"
            placeholder="ѡ������"
            format="YYYY-MM-DD"
            value-format="YYYY-MM-DD"
            style="width: 150px; margin-right: 10px;"
          />
          <el-select
            v-model="searchForm.vehicleId"
            placeholder="ѡ����"
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
            v-model="searchForm.status"
            placeholder="��ҵ״̬"
            style="width: 120px; margin-right: 10px;"
          >
            <el-option label="ȫ��" value="" />
            <el-option label="������" value="in_progress" />
            <el-option label="�����" value="completed" />
            <el-option label="��ȡ��" value="cancelled" />
          </el-select>
          <el-button type="primary" @click="loadVehicleJobsData" :loading="loading">
            ��ѯ
          </el-button>
        </div>

        <el-table :data="vehicleJobsData" style="width: 100%" class="jobs-table">
          <el-table-column type="index" label="���" width="80" />
          <el-table-column prop="fleet" label="����" />
          <el-table-column prop="licensePlate" label="���ƺ�" />
          <el-table-column prop="vehicleType" label="����" />
          <el-table-column prop="date" label="����" width="120" />
          <el-table-column prop="cargoType" label="�������" />
          <el-table-column prop="loadWeight" label="װ����(kg)" />
          <el-table-column prop="unloadWeight" label="ж����(kg)" />
          <el-table-column prop="tripCount" label="����" width="80" />
          <el-table-column prop="measurement" label="װ�ؼ���" />
          <el-table-column prop="distance" label="���(km)" />
          <el-table-column prop="duration" label="ʱ��(Сʱ)" width="100" />
          <el-table-column label="����" width="120">
            <template #default="scope">
              <el-button size="small" type="primary" @click="viewJobDetail(scope.row)">
                �鿴
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import api from '@/api';

// ��������
interface VehicleItem {
  vehicle_id: number;
  vehicle_name: string;
  license_plate: string;
  vehicle_type: string;
  status: number;
}

// ������ҵ��������
interface VehicleJobItem {
  jobId: string;
  fleet: string;
  licensePlate: string;
  vehicleType: string;
  date: string;
  cargoType: string;
  loadWeight: number;
  unloadWeight: number;
  tripCount: number;
  measurement: string;
  distance: number;
  duration: number;
  status: string;
}

// ��������
const searchForm = {
  date: new Date().toISOString().split('T')[0],
  vehicleId: null as number | null,
  status: '',
};

// ����
const vehicles = ref<VehicleItem[]>([]);
const vehicleJobsData = ref<VehicleJobItem[]>([]);
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

// ���س�����ҵ����
const loadVehicleJobsData = async () => {
  loading.value = true;
  try {
    // ������ѯ����
    const params: Record<string, any> = {};
    if (searchForm.date) params.date = searchForm.date;
    if (searchForm.vehicleId) params.vehicle_id = searchForm.vehicleId;
    if (searchForm.status) params.status = searchForm.status;
    
    // ������ʵAPI��ȡ������ҵ����
    const response = await api.get('/api/jobs', { params }) as any;
    vehicleJobsData.value = response.data?.items || response.items || [];
    ElMessage.success('���س�����ҵ���ݳɹ�');
  } catch (error) {
    console.error('���س�����ҵ����ʧ��:', error);
    ElMessage.error('���س�����ҵ����ʧ��');
  } finally {
    loading.value = false;
  }
};

// �鿴��ҵ����
const viewJobDetail = (row: VehicleJobItem) => {
  ElMessage.info(`�鿴��ҵ����: ${row.jobId}`);
};

// �������ʱ��������
onMounted(() => {
  loadVehicles();
  loadVehicleJobsData();
});
</script>

<style scoped>
.vehicle-jobs {
  padding: 20px;
}

.breadcrumb {
  margin-bottom: 20px;
}

.card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 16px;
  font-weight: bold;
  color: #303133;
}

.card-content {
  padding: 20px;
}

.search-params {
  margin-bottom: 20px;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}

.jobs-table {
  max-height: 600px;
  overflow-y: auto;
}
</style>

