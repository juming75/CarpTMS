<template>
  <div class="transport-statistics">
    <el-breadcrumb separator="/" class="breadcrumb">
      <el-breadcrumb-item><span>�������</span></el-breadcrumb-item>
      <el-breadcrumb-item><span>����ͳ��</span></el-breadcrumb-item>
    </el-breadcrumb>

    <el-card shadow="hover" class="card">
      <template #header>
        <div class="card-header">
          <span>����ͳ�Ʒ���</span>
        </div>
      </template>

      <div class="card-content">
        <div class="search-params">
          <el-select
            v-model="searchForm.shiftType"
            placeholder="�������"
            style="width: 120px; margin-right: 10px;"
          >
            <el-option label="ȫ��" value="" />
            <el-option label="���" value="morning" />
            <el-option label="�а�" value="afternoon" />
            <el-option label="����" value="night" />
          </el-select>
          <el-select
            v-model="searchForm.cargoType"
            placeholder="�������"
            style="width: 120px; margin-right: 10px;"
          >
            <el-option label="ȫ��" value="" />
            <el-option label="������" value="concrete" />
            <el-option label="�ֲ�" value="steel" />
            <el-option label="ľ��" value="wood" />
            <el-option label="����" value="other" />
          </el-select>
          <el-select
            v-model="searchForm.dateType"
            placeholder="ָ������"
            style="width: 120px; margin-right: 10px;"
          >
            <el-option label="����" value="today" />
            <el-option label="����" value="yesterday" />
            <el-option label="����" value="thisWeek" />
            <el-option label="����" value="thisMonth" />
            <el-option label="�Զ���" value="custom" />
          </el-select>
          <el-date-picker
            v-model="searchForm.date"
            type="date"
            placeholder="ѡ������"
            style="width: 180px; margin-right: 10px;"
          />
          <el-input
            v-model="searchForm.route"
            placeholder="��·"
            style="width: 180px; margin-right: 10px;"
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
          <el-select
            v-model="searchForm.groupBy"
            placeholder="ͳ��ά��"
            style="width: 120px; margin-right: 10px;"
          >
            <el-option label="����" value="day" />
            <el-option label="����" value="week" />
            <el-option label="����" value="month" />
          </el-select>
          <el-button type="primary" @click="loadTransportStatisticsData" :loading="loading">
            ��ѯ
          </el-button>
        </div>

        <div class="statistics-table">
          <h3>����ͳ����ϸ</h3>
          <el-table :data="transportStatisticsData" style="width: 100%">
            <el-table-column type="index" label="���" width="80" />
            <el-table-column prop="fleet" label="����" />
            <el-table-column prop="licensePlate" label="���ƺ�" />
            <el-table-column prop="cargoType" label="�������" />
            <el-table-column prop="route" label="������·" />
            <el-table-column prop="date" label="����" width="120" />
            <el-table-column prop="shift" label="���" width="80" />
            <el-table-column prop="status" label="״̬" />
            <el-table-column prop="completion" label="��ɶ�" width="100" />
            <el-table-column prop="loadWeight" label="װ����(kg)" />
            <el-table-column prop="unloadWeight" label="ж����(kg)" />
            <el-table-column label="����" width="120">
              <template #default="scope">
                <el-button size="small" type="primary" @click="viewStatisticsDetail(scope.row)">
                  �鿴
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
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

// ����ͳ����������
interface TransportStatisticsItem {
  fleet: string;
  licensePlate: string;
  cargoType: string;
  route: string;
  date: string;
  shift: string;
  status: string;
  completion: number;
  loadWeight: number;
  unloadWeight: number;
}

// ��������
const searchForm = {
  date: new Date().toISOString().split('T')[0],
  vehicleId: '' as string | number | null,
  groupBy: 'day',
  shiftType: '',
  cargoType: '',
  route: '',
  dateType: '' as string,
};

// ����
const vehicles = ref<VehicleItem[]>([]);
const transportStatisticsData = ref<TransportStatisticsItem[]>([]);
const loading = ref(false);

// ͼ������
const volumeChartRef = ref<HTMLElement | null>(null);
const distanceChartRef = ref<HTMLElement | null>(null);
const timeChartRef = ref<HTMLElement | null>(null);

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

// ��������ͳ������
const loadTransportStatisticsData = async () => {
  loading.value = true;
  try {
    // ������ѯ����
    const params: Record<string, any> = {};
    if (searchForm.date) params.date = searchForm.date;
    if (searchForm.vehicleId) params.vehicle_id = searchForm.vehicleId;
    if (searchForm.groupBy) params.group_by = searchForm.groupBy;
    if (searchForm.shiftType) params.shift_type = searchForm.shiftType;
    if (searchForm.cargoType) params.cargo_type = searchForm.cargoType;
    if (searchForm.route) params.route = searchForm.route;
    
    // ������ʵAPI��ȡ����ͳ������
    const response = await api.get('/api/statistics/transport', { params }) as any;
    transportStatisticsData.value = response.data?.items || response.items || [];
    ElMessage.success('��������ͳ�����ݳɹ�');
  } catch (error) {
    console.error('��������ͳ������ʧ��:', error);
    ElMessage.error('��������ͳ������ʧ��');
  } finally {
    loading.value = false;
  }
};

// ����ͼ��
// 查看统计详情
const viewStatisticsDetail = (_row: TransportStatisticsItem) => {
  ElMessage.info('查看统计详情');
};

// Reserved for future chart rendering
const drawCharts = () => {
  // reserved for future use with ECharts charting library
  // �������ʹ��ECharts������ͼ�������ͼ��
  // ����û������ͼ���⣬����ֻ���򵥵�ռλ
  if (volumeChartRef.value) {
    volumeChartRef.value.innerHTML = `
      <div style="height: 300px; display: flex; align-items: center; justify-content: center; border: 1px solid #e4e7ed; border-radius: 4px;">
        <div>
          <h4>������ͳ��ͼ��</h4>
          <p>����: ${transportStatisticsData.value.map(item => item.loadWeight).join(', ')}</p>
        </div>
      </div>
    `;
  }
  
  if (distanceChartRef.value) {
    distanceChartRef.value.innerHTML = `
      <div style="height: 300px; display: flex; align-items: center; justify-content: center; border: 1px solid #e4e7ed; border-radius: 4px;">
        <div>
          <h4>�������ͳ��ͼ��</h4>
          <p>����: ${transportStatisticsData.value.map(item => item.route).join(', ')}</p>
        </div>
      </div>
    `;
  }
  
  if (timeChartRef.value) {
    timeChartRef.value.innerHTML = `
      <div style="height: 300px; display: flex; align-items: center; justify-content: center; border: 1px solid #e4e7ed; border-radius: 4px;">
        <div>
          <h4>����ʱ��ͳ��ͼ��</h4>
          <p>����: ${transportStatisticsData.value.length} ����¼</p>
        </div>
      </div>
    `;
  }
};

// �������ʱ��������
onMounted(() => {
  loadVehicles();
  loadTransportStatisticsData();
  void drawCharts;
});
</script>

<style scoped>
.transport-statistics {
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

.statistics-charts {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.chart-item {
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  padding: 15px;
  background-color: #f9f9f9;
}

.chart-item h3 {
  margin-top: 0;
  margin-bottom: 15px;
  font-size: 14px;
  font-weight: bold;
  color: #303133;
}

.chart-container {
  height: 300px;
}

.statistics-table {
  margin-top: 30px;
}

.statistics-table h3 {
  margin-top: 0;
  margin-bottom: 15px;
  font-size: 14px;
  font-weight: bold;
  color: #303133;
}

.statistics-table .el-table {
  max-height: 400px;
  overflow-y: auto;
}
</style>

