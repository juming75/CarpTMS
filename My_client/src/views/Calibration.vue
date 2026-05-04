<template>
  <div class="calibration-container">
    <h2>标定管理</h2>

    <!-- 搜索和工具栏 -->
    <el-form :inline="true" :model="queryForm" class="search-form">
      <el-form-item label="传感器号">
        <el-input v-model="queryForm.sensor_no" placeholder="请输入传感器号" clearable />
      </el-form-item>
      <el-form-item label="车牌号">
        <el-input v-model="queryForm.plate_no" placeholder="请输入车牌号" clearable />
      </el-form-item>
      <el-form-item>
        <el-button type="primary" @click="handleSearch">查询</el-button>
        <el-button @click="handleReset">重置</el-button>
        <el-button type="success" @click="handleAdd">新增标定</el-button>
      </el-form-item>
    </el-form>

    <!-- 数据表格 -->
    <el-table :data="tableData" v-loading="loading" border stripe style="width: 100%">
      <el-table-column prop="sensor_no" label="传感器号" width="120" />
      <el-table-column prop="vehicle_id" label="车辆ID" width="80" />
      <el-table-column prop="plate_no" label="车牌号" width="120" />
      <el-table-column prop="sensor_side" label="传感器位置" width="100" />
      <el-table-column prop="sensor_group" label="传感器组" width="100" />
      <el-table-column prop="self_weight" label="自重(kg)" width="100" />
      <el-table-column prop="is_calibrated" label="标定状态" width="100">
        <template #default="{ row }">
          <el-tag :type="row.is_calibrated ? 'success' : 'warning'">
            {{ row.is_calibrated ? '已标定' : '未标定' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="rated_total_weight" label="额定总重(kg)" width="120" />
      <el-table-column prop="r2_score" label="R²评分" width="100" />
      <el-table-column prop="create_time" label="创建时间" width="180">
        <template #default="{ row }">
          {{ formatDateTime(row.create_time) }}
        </template>
      </el-table-column>
      <el-table-column label="操作" width="200" fixed="right">
        <template #default="{ row }">
          <el-button type="primary" size="small" @click="handleView(row)">详情</el-button>
          <el-button type="warning" size="small" @click="handleEdit(row)">编辑</el-button>
          <el-button type="danger" size="small" @click="handleDelete(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 分页 -->
    <el-pagination
      v-model:current-page="pagination.page"
      v-model:page-size="pagination.page_size"
      :total="pagination.total"
      :page-sizes="[10, 20, 50, 100]"
      layout="total, sizes, prev, pager, next, jumper"
      @size-change="fetchData"
      @current-change="fetchData"
      style="margin-top: 20px; justify-content: flex-end"
    />

    <!-- 新增/编辑对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="600px"
      @close="resetForm"
    >
      <el-form :model="formData" :rules="formRules" ref="formRef" label-width="120px">
        <el-form-item label="传感器号" prop="sensor_no">
          <el-input v-model="formData.sensor_no" placeholder="请输入传感器号" />
        </el-form-item>
        <el-form-item label="车辆ID" prop="vehicle_id">
          <el-input-number v-model="formData.vehicle_id" :min="0" />
        </el-form-item>
        <el-form-item label="车牌号" prop="plate_no">
          <el-input v-model="formData.plate_no" placeholder="请输入车牌号" />
        </el-form-item>
        <el-form-item label="传感器位置" prop="sensor_side">
          <el-select v-model="formData.sensor_side" placeholder="请选择">
            <el-option label="左侧" value="left" />
            <el-option label="右侧" value="right" />
          </el-select>
        </el-form-item>
        <el-form-item label="传感器组" prop="sensor_group">
          <el-input-number v-model="formData.sensor_group" :min="1" />
        </el-form-item>
        <el-form-item label="自重(kg)" prop="self_weight">
          <el-input-number v-model="formData.self_weight" :precision="2" :min="0" />
        </el-form-item>
        <el-form-item label="额定总重(kg)" prop="rated_total_weight">
          <el-input-number v-model="formData.rated_total_weight" :precision="2" :min="0" />
        </el-form-item>
        <el-form-item label="已标定" prop="is_calibrated">
          <el-switch v-model="formData.is_calibrated" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit" :loading="submitting">确定</el-button>
      </template>
    </el-dialog>

    <!-- 详情对话框 -->
    <el-dialog v-model="detailVisible" title="标定详情" width="700px">
      <el-descriptions :column="2" border v-if="detailData">
        <el-descriptions-item label="传感器号">{{ detailData.sensor_no }}</el-descriptions-item>
        <el-descriptions-item label="车辆ID">{{ detailData.vehicle_id }}</el-descriptions-item>
        <el-descriptions-item label="车牌号">{{ detailData.plate_no }}</el-descriptions-item>
        <el-descriptions-item label="传感器位置">{{ detailData.sensor_side }}</el-descriptions-item>
        <el-descriptions-item label="传感器组">{{ detailData.sensor_group }}</el-descriptions-item>
        <el-descriptions-item label="自重(kg)">{{ detailData.self_weight }}</el-descriptions-item>
        <el-descriptions-item label="额定总重(kg)">{{ detailData.rated_total_weight }}</el-descriptions-item>
        <el-descriptions-item label="标定状态">
          <el-tag :type="detailData.is_calibrated ? 'success' : 'warning'">
            {{ detailData.is_calibrated ? '已标定' : '未标定' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="R²评分">{{ detailData.r2_score }}</el-descriptions-item>
        <el-descriptions-item label="RMSE">{{ detailData.rmse }}</el-descriptions-item>
        <el-descriptions-item label="最大误差">{{ detailData.max_error }}</el-descriptions-item>
        <el-descriptions-item label="标定点数">{{ detailData.point_count }}</el-descriptions-item>
        <el-descriptions-item label="创建时间" :span="2">{{ formatDateTime(detailData.create_time) }}</el-descriptions-item>
        <el-descriptions-item label="更新时间" :span="2">{{ detailData.update_time ? formatDateTime(detailData.update_time) : '-' }}</el-descriptions-item>
      </el-descriptions>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import api from '@/api';

interface Calibration {
  id: number;
  sensor_no: number;
  vehicle_id: number;
  plate_no: string;
  sensor_side: string;
  sensor_group: number;
  self_weight: number;
  polynomial_json: string;
  linear_segments_json: string;
  is_calibrated: boolean;
  create_time: string;
  update_time: string;
  calibration_points: any;
  pa_raw: number;
  axle_number: number;
  is_left_wheel: boolean;
  turn_point: number;
  polynomial_order: number;
  polynomial_coefs_json: string;
  r2_score: number;
  rmse: number;
  max_error: number;
  point_count: number;
  rated_total_weight: number;
  tare_weight: number;
}

const tableData = ref<Calibration[]>([]);
const loading = ref(false);
const dialogVisible = ref(false);
const detailVisible = ref(false);
const submitting = ref(false);
const dialogTitle = ref('新增标定');
const formRef = ref();
const detailData = ref<Calibration | null>(null);

const queryForm = reactive({
  sensor_no: '',
  plate_no: '',
});

const pagination = reactive({
  page: 1,
  page_size: 20,
  total: 0,
});

const formData = reactive({
  id: 0,
  sensor_no: 0,
  vehicle_id: 0,
  plate_no: '',
  sensor_side: 'left',
  sensor_group: 1,
  self_weight: 0,
  polynomial_json: '',
  linear_segments_json: '',
  is_calibrated: false,
  calibration_points: [],
  pa_raw: 0,
  axle_number: 1,
  is_left_wheel: true,
  turn_point: 50000,
  polynomial_order: 2,
  r2_score: 0,
  rmse: 0,
  max_error: 0,
  point_count: 0,
  rated_total_weight: 0,
  tare_weight: 0,
});

const formRules = {
  sensor_no: [{ required: true, message: '请输入传感器号', trigger: 'blur' }],
  plate_no: [{ required: true, message: '请输入车牌号', trigger: 'blur' }],
};

const formatDateTime = (datetime: string) => {
  if (!datetime) return '-';
  return new Date(datetime).toLocaleString('zh-CN');
};

async function fetchData() {
  loading.value = true;
  try {
    const params: any = {
      page: pagination.page,
      page_size: pagination.page_size,
    };
    if (queryForm.sensor_no) params.sensor_no = parseInt(queryForm.sensor_no);
    if (queryForm.plate_no) params.plate_no = queryForm.plate_no;

    const res = await api.get('/api/weight-calibrations', { params });
    const data = res?.data ?? res;
    tableData.value = data?.list || [];
    pagination.total = data?.total || 0;
  } catch (error: any) {
    console.error('获取标定数据失败:', error);
    if (error.response?.status === 401) {
      ElMessage.error('请先登录');
    } else {
      ElMessage.error('获取标定数据失败');
    }
  } finally {
    loading.value = false;
  }
}

function handleSearch() {
  pagination.page = 1;
  fetchData();
}

function handleReset() {
  queryForm.sensor_no = '';
  queryForm.plate_no = '';
  pagination.page = 1;
  fetchData();
}

function handleAdd() {
  dialogTitle.value = '新增标定';
  formData.id = 0;
  formData.sensor_no = 0;
  formData.vehicle_id = 0;
  formData.plate_no = '';
  formData.sensor_side = 'left';
  formData.sensor_group = 1;
  formData.self_weight = 0;
  formData.polynomial_json = '';
  formData.linear_segments_json = '';
  formData.is_calibrated = false;
  formData.calibration_points = [];
  formData.pa_raw = 0;
  formData.axle_number = 1;
  formData.is_left_wheel = true;
  formData.turn_point = 50000;
  formData.polynomial_order = 2;
  formData.r2_score = 0;
  formData.rmse = 0;
  formData.max_error = 0;
  formData.point_count = 0;
  formData.rated_total_weight = 0;
  formData.tare_weight = 0;
  dialogVisible.value = true;
}

function handleEdit(row: Calibration) {
  dialogTitle.value = '编辑标定';
  formData.id = row.id;
  formData.sensor_no = row.sensor_no;
  formData.vehicle_id = row.vehicle_id;
  formData.plate_no = row.plate_no;
  formData.sensor_side = row.sensor_side;
  formData.sensor_group = row.sensor_group || 1;
  formData.self_weight = row.self_weight || 0;
  formData.polynomial_json = row.polynomial_json || '';
  formData.linear_segments_json = row.linear_segments_json || '';
  formData.is_calibrated = row.is_calibrated;
  formData.calibration_points = row.calibration_points || [];
  formData.pa_raw = row.pa_raw || 0;
  formData.axle_number = row.axle_number || 1;
  formData.is_left_wheel = row.is_left_wheel ?? true;
  formData.turn_point = row.turn_point || 50000;
  formData.polynomial_order = row.polynomial_order || 2;
  formData.r2_score = row.r2_score || 0;
  formData.rmse = row.rmse || 0;
  formData.max_error = row.max_error || 0;
  formData.point_count = row.point_count || 0;
  formData.rated_total_weight = row.rated_total_weight || 0;
  formData.tare_weight = row.tare_weight || 0;
  dialogVisible.value = true;
}

function handleView(row: Calibration) {
  detailData.value = row;
  detailVisible.value = true;
}

async function handleDelete(row: Calibration) {
  try {
    await ElMessageBox.confirm('确定要删除该标定数据吗？', '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    });

    await api.delete(`/api/weight-calibrations/${row.id}`);
    ElMessage.success('删除成功');
    fetchData();
  } catch (error: any) {
    if (error !== 'cancel') {
      console.error('删除失败:', error);
      ElMessage.error('删除失败');
    }
  }
}

function resetForm() {
  formRef.value?.resetFields();
}

async function handleSubmit() {
  try {
    await formRef.value?.validate();
    submitting.value = true;

    const data = {
      sensor_no: formData.sensor_no,
      vehicle_id: formData.vehicle_id,
      plate_no: formData.plate_no,
      sensor_side: formData.sensor_side,
      sensor_group: formData.sensor_group,
      self_weight: formData.self_weight,
      polynomial_json: formData.polynomial_json || '{}',
      linear_segments_json: formData.linear_segments_json || null,
      is_calibrated: formData.is_calibrated,
      calibration_points: formData.calibration_points,
      pa_raw: formData.pa_raw,
      axle_number: formData.axle_number,
      is_left_wheel: formData.is_left_wheel,
      turn_point: formData.turn_point,
      polynomial_order: formData.polynomial_order,
      r2_score: formData.r2_score,
      rmse: formData.rmse,
      max_error: formData.max_error,
      point_count: formData.point_count,
      rated_total_weight: formData.rated_total_weight,
      tare_weight: formData.tare_weight,
    };

    if (formData.id) {
      await api.put(`/api/weight-calibrations/${formData.id}`, data);
      ElMessage.success('更新成功');
    } else {
      await api.post('/api/weight-calibrations', data);
      ElMessage.success('创建成功');
    }

    dialogVisible.value = false;
    fetchData();
  } catch (error: any) {
    if (error.response?.status === 401) {
      ElMessage.error('请先登录');
    } else if (error.response?.status === 400) {
      ElMessage.error(error.response.data?.message || '请求参数错误');
    } else {
      ElMessage.error(formData.id ? '更新失败' : '创建失败');
    }
  } finally {
    submitting.value = false;
  }
}

onMounted(() => {
  fetchData();
});
</script>

<style scoped>
.calibration-container {
  padding: 20px;
  background: white;
  border-radius: 8px;
  min-height: 400px;
}

h2 {
  margin-bottom: 16px;
  color: #303133;
}

.search-form {
  margin-bottom: 20px;
}
</style>
