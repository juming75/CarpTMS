<template>
  <div class="event-query">
    <el-breadcrumb separator="/" class="breadcrumb">
      <el-breadcrumb-item><span>�������</span></el-breadcrumb-item>
      <el-breadcrumb-item><span>�¼���ѯ</span></el-breadcrumb-item>
    </el-breadcrumb>

    <el-card shadow="hover" class="card">
      <template #header>
        <div class="card-header">
          <span>�¼���ѯ����</span>
        </div>
      </template>

      <div class="card-content">
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
            v-model="searchForm.eventType"
            placeholder="�¼�����"
            style="width: 150px; margin-right: 10px;"
          >
            <el-option label="ȫ��" value="" />
            <el-option label="�澯�¼�" value="alarm" />
            <el-option label="��ҵ�¼�" value="job" />
            <el-option label="λ���¼�" value="location" />
            <el-option label="ϵͳ�¼�" value="system" />
          </el-select>
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
          <el-input
            v-model="searchForm.keyword"
            placeholder="�ؼ�������"
            style="width: 200px; margin-right: 10px;"
          />
          <el-button type="primary" @click="loadEventQueryData" :loading="loading">
            ��ѯ
          </el-button>
        </div>

        <el-table :data="eventQueryData" style="width: 100%" class="events-table">
          <el-table-column type="index" label="���" width="80" />
          <el-table-column prop="eventId" label="�¼�ID" width="120" />
          <el-table-column prop="eventType" label="�¼�����" />
          <el-table-column prop="vehiclePlate" label="���ƺ�" />
          <el-table-column prop="eventTime" label="�¼�ʱ��" width="180" />
          <el-table-column prop="eventContent" label="�¼�����" />
          <el-table-column prop="status" label="״̬" />
          <el-table-column prop="operator" label="������" />
          <el-table-column label="����" width="120">
            <template #default="scope">
              <el-button size="small" type="primary" @click="viewEventDetail(scope.row)">
                �鿴
              </el-button>
            </template>
          </el-table-column>
        </el-table>

        <div class="pagination">
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

// �¼���������
interface EventItem {
  eventId: string;
  eventType: string;
  vehiclePlate: string;
  eventTime: string;
  eventContent: string;
  status: string;
  operator: string;
}

// ��������
const searchForm = {
  dateRange: [new Date(Date.now() - 30 * 24 * 60 * 60 * 1000), new Date()] as [Date, Date],
  eventType: '',
  vehicleId: '',
  keyword: '',
};

// ����
const vehicles = ref<VehicleItem[]>([]);
const eventQueryData = ref<EventItem[]>([]);
const loading = ref(false);

// ��ҳ
const currentPage = ref(1);
const pageSize = ref(10);
const total = ref(100);

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

// �����¼���ѯ����
const loadEventQueryData = async () => {
  loading.value = true;
  try {
    // ������ѯ����
    const params: Record<string, any> = {};
    if (searchForm.dateRange) {
      params.start_date = searchForm.dateRange[0] instanceof Date ? searchForm.dateRange[0].toISOString().split('T')[0] : searchForm.dateRange[0];
      params.end_date = searchForm.dateRange[1] instanceof Date ? searchForm.dateRange[1].toISOString().split('T')[0] : searchForm.dateRange[1];
    }
    if (searchForm.eventType) params.event_type = searchForm.eventType;
    if (searchForm.vehicleId) params.vehicle_id = searchForm.vehicleId;
    if (searchForm.keyword) params.keyword = searchForm.keyword;
    params.page = currentPage.value;
    params.page_size = pageSize.value;
    
    // ������ʵAPI��ȡ�¼���ѯ����
    const response = await api.get('/api/events', { params }) as any;
    eventQueryData.value = response.data?.items || response.items || [];
    total.value = response.data?.total || response.total || 0;
    ElMessage.success('�����¼���ѯ���ݳɹ�');
  } catch (error) {
    console.error('�����¼���ѯ����ʧ��:', error);
    ElMessage.error('�����¼���ѯ����ʧ��');
  } finally {
    loading.value = false;
  }
};

// �鿴�¼�����
const viewEventDetail = (row: EventItem) => {
  ElMessage.info(`�鿴�¼�����: ${row.eventId}`);
};

// ��ҳ����
const handleSizeChange = (size: number) => {
  pageSize.value = size;
  loadEventQueryData();
};

const handleCurrentChange = (current: number) => {
  currentPage.value = current;
  loadEventQueryData();
};

// �������ʱ��������
onMounted(() => {
  loadVehicles();
  loadEventQueryData();
});
</script>

<style scoped>
.event-query {
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

.events-table {
  max-height: 500px;
  overflow-y: auto;
  margin-bottom: 20px;
}

.pagination {
  display: flex;
  justify-content: flex-end;
  margin-top: 20px;
}
</style>

