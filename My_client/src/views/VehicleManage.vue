<template>
  <div class="vehicle-manage">
    <el-card>
      <!-- 工具栏 -->
      <div class="toolbar">
        <el-input v-model="searchKeyword" placeholder="搜索车辆" clearable style="width: 200px" @input="handleSearch">
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-button type="primary" @click="handleAdd">
          <el-icon><Plus /></el-icon>
          添加车辆
        </el-button>
        <el-button type="danger" :disabled="selectedIds.length === 0" @click="handleBatchDelete">
          <el-icon><Delete /></el-icon>
          批量删除
        </el-button>
        <div style="flex: 1"></div>
        <el-button @click="handleRefresh">
          <el-icon><Refresh /></el-icon>
          刷新
        </el-button>
        <el-button type="success" @click="handleSync" :loading="syncing">
          <el-icon><Upload /></el-icon>
          同步数据
        </el-button>
      </div>

      <!-- 车辆列表 - 使用 VirtualTable -->
      <div v-loading="loading" class="table-container">
        <VirtualTable
          :data="vehicleList"
          :columns="columns"
          :height="500"
          :row-height="48"
          :row-key="'vehicle_id'"
          :show-pagination="false"
        >
          <template #column-selection="{ row }">
            <el-checkbox
              :checked="selectedIds.includes(row.vehicle_id)"
              @change="(e: Event) => handleRowSelection(row, e)"
            />
          </template>
          <template #column-sync_status="{ row }">
            <el-tag :type="row.is_synced ? 'success' : 'warning'" size="small">
              {{ row.is_synced ? '已同步' : '未同步' }}
            </el-tag>
          </template>
          <template #column-actions="{ row }">
            <el-button size="small" @click="handleEdit(row)">编辑</el-button>
            <el-button size="small" type="danger" @click="handleDelete(row)">删除</el-button>
          </template>
        </VirtualTable>
      </div>

      <!-- 分页 -->
      <div class="pagination">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          @current-change="handlePageChange"
          @size-change="handleSizeChange"
        />
      </div>
    </el-card>

    <!-- 添加/编辑对话框 -->
    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="900px" @close="handleDialogClose">
      <el-form ref="formRef" :model="formData" :rules="formRules" label-width="120px" size="default">
        <el-tabs v-model="activeTab" type="card">
          <!-- 基本资料（*必填） -->
          <el-tab-pane label="基本资料（*必填）" name="basic">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="车辆名称" prop="vehicle_name" required>
                  <el-input v-model="formData.vehicle_name" placeholder="请输入车辆名称" />
                </el-form-item>
                <el-form-item label="车牌号码" prop="license_plate" required>
                  <el-input v-model="formData.license_plate" placeholder="请输入车牌号码" />
                </el-form-item>
                <el-form-item label="车辆类型" prop="vehicle_type" required>
                  <el-input v-model="formData.vehicle_type" placeholder="请输入车辆类型" />
                </el-form-item>
                <el-form-item label="车辆颜色" prop="vehicle_color">
                  <el-input v-model="formData.vehicle_color" placeholder="请输入车辆颜色" />
                </el-form-item>
                <el-form-item label="车辆品牌" prop="vehicle_brand">
                  <el-input v-model="formData.vehicle_brand" placeholder="请输入车辆品牌" />
                </el-form-item>
                <el-form-item label="车辆型号" prop="vehicle_model">
                  <el-input v-model="formData.vehicle_model" placeholder="请输入车辆型号" />
                </el-form-item>
                <el-form-item label="发动机号" prop="engine_no">
                  <el-input v-model="formData.engine_no" placeholder="请输入发动机号" />
                </el-form-item>
                <el-form-item label="车架编号" prop="frame_no">
                  <el-input v-model="formData.frame_no" placeholder="请输入车架编号" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="注册日期" prop="register_date">
                  <el-date-picker
                    v-model="formData.register_date"
                    type="datetime"
                    placeholder="选择注册日期"
                    style="width: 100%"
                  />
                </el-form-item>
                <el-form-item label="年检日期" prop="inspection_date">
                  <el-date-picker
                    v-model="formData.inspection_date"
                    type="datetime"
                    placeholder="选择年检日期"
                    style="width: 100%"
                  />
                </el-form-item>
                <el-form-item label="保险日期" prop="insurance_date">
                  <el-date-picker
                    v-model="formData.insurance_date"
                    type="datetime"
                    placeholder="选择保险日期"
                    style="width: 100%"
                  />
                </el-form-item>
                <el-form-item label="座位数" prop="seating_capacity">
                  <el-input v-model.number="formData.seating_capacity" placeholder="请输入座位数" />
                </el-form-item>
                <el-form-item label="载重（吨）" prop="load_capacity">
                  <el-input v-model.number="formData.load_capacity" placeholder="请输入载重" />
                </el-form-item>
                <el-form-item label="车长（米）" prop="vehicle_length">
                  <el-input v-model.number="formData.vehicle_length" placeholder="请输入车长" />
                </el-form-item>
                <el-form-item label="车宽（米）" prop="vehicle_width">
                  <el-input v-model.number="formData.vehicle_width" placeholder="请输入车宽" />
                </el-form-item>
                <el-form-item label="车高（米）" prop="vehicle_height">
                  <el-input v-model.number="formData.vehicle_height" placeholder="请输入车高" />
                </el-form-item>
              </el-col>
            </el-row>
          </el-tab-pane>

          <!-- 终端信息 -->
          <el-tab-pane label="终端信息" name="terminal">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="终端编号" prop="device_id">
                  <el-input v-model="formData.device_id" placeholder="请输入终端编号" />
                </el-form-item>
                <el-form-item label="终端类型" prop="terminal_type">
                  <el-select
                    v-model="formData.terminal_type"
                    placeholder="请选择终端类型"
                    style="width: 100%"
                    filterable
                  >
                    <el-option label="GPRS_GB北斗型" :value="'GPRS_GB北斗型'" />
                  </el-select>
                </el-form-item>
                <el-form-item label="通信方式" prop="communication_type">
                  <el-select
                    v-model="formData.communication_type"
                    placeholder="请选择通信方式"
                    style="width: 100%"
                    filterable
                  >
                    <el-option label="TCP" :value="'TCP'" />
                  </el-select>
                </el-form-item>
                <el-form-item label="SIM卡号" prop="sim_card_no">
                  <el-input v-model="formData.sim_card_no" placeholder="请输入SIM卡号" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="安装日期" prop="install_date">
                  <el-date-picker
                    v-model="formData.install_date"
                    type="datetime"
                    placeholder="选择安装日期"
                    style="width: 100%"
                  />
                </el-form-item>
                <el-form-item label="安装地址" prop="install_address">
                  <el-input v-model="formData.install_address" placeholder="请输入安装地址" />
                </el-form-item>
                <el-form-item label="安装技师" prop="install_technician">
                  <el-input v-model="formData.install_technician" placeholder="请输入安装技师" />
                </el-form-item>
              </el-col>
            </el-row>
          </el-tab-pane>

          <!-- 车主信息 -->
          <el-tab-pane label="车主信息" name="owner">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="车主名称" prop="own_name">
                  <el-input v-model="formData.own_name" placeholder="请输入车主名称" />
                </el-form-item>
                <el-form-item label="联系电话" prop="own_phone">
                  <el-input v-model="formData.own_phone" placeholder="请输入联系电话" />
                </el-form-item>
                <el-form-item label="证件号码" prop="own_id_card">
                  <el-input v-model="formData.own_id_card" placeholder="请输入证件号码" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="联系地址" prop="own_address">
                  <el-input v-model="formData.own_address" placeholder="请输入联系地址" type="textarea" rows="3" />
                </el-form-item>
                <el-form-item label="电子邮箱" prop="own_email">
                  <el-input v-model="formData.own_email" placeholder="请输入电子邮箱" />
                </el-form-item>
              </el-col>
            </el-row>
          </el-tab-pane>

          <!-- 运营信息 -->
          <el-tab-pane label="运营信息" name="operation">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="隶属车队" prop="group_id" required>
                  <el-select v-model="formData.group_id" placeholder="请选择隶属车队" style="width: 100%">
                    <el-option label="默认车组" :value="1" />
                    <el-option label="中川油" :value="2" />
                  </el-select>
                </el-form-item>
                <el-form-item label="运营状态" prop="operation_status">
                  <el-select v-model="formData.operation_status" placeholder="请选择运营状态" style="width: 100%">
                    <el-option label="正常" :value="1" />
                    <el-option label="停运" :value="0" />
                  </el-select>
                </el-form-item>
                <el-form-item label="运营路线" prop="operation_route">
                  <el-input v-model="formData.operation_route" placeholder="请输入运营路线" />
                </el-form-item>
                <el-form-item label="运营区域" prop="operation_area">
                  <el-input v-model="formData.operation_area" placeholder="请输入运营区域" />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="运营公司" prop="operation_company">
                  <el-input v-model="formData.operation_company" placeholder="请输入运营公司" />
                </el-form-item>
                <el-form-item label="司机姓名" prop="driver_name">
                  <el-input v-model="formData.driver_name" placeholder="请输入司机姓名" />
                </el-form-item>
                <el-form-item label="司机电话" prop="driver_phone">
                  <el-input v-model="formData.driver_phone" placeholder="请输入司机电话" />
                </el-form-item>
                <el-form-item label="驾驶证号" prop="driver_license_no">
                  <el-input v-model="formData.driver_license_no" placeholder="请输入驾驶证号" />
                </el-form-item>
              </el-col>
            </el-row>
          </el-tab-pane>

          <!-- 财务信息 -->
          <el-tab-pane label="财务信息" name="finance">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="购车价格" prop="purchase_price">
                  <el-input v-model.number="formData.purchase_price" placeholder="请输入购车价格" />
                </el-form-item>
                <el-form-item label="年度费用" prop="annual_fee">
                  <el-input v-model.number="formData.annual_fee" placeholder="请输入年度费用" />
                </el-form-item>
                <el-form-item label="保险费用" prop="insurance_fee">
                  <el-input v-model.number="formData.insurance_fee" placeholder="请输入保险费用" />
                </el-form-item>
              </el-col>
            </el-row>
          </el-tab-pane>

          <!-- 其他信息 -->
          <el-tab-pane label="其他信息" name="other">
            <el-row :gutter="20">
              <el-col :span="24">
                <el-form-item label="备注" prop="remark">
                  <el-input v-model="formData.remark" placeholder="请输入备注信息" type="textarea" rows="3" />
                </el-form-item>
              </el-col>
            </el-row>
          </el-tab-pane>
        </el-tabs>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Search, Plus, Delete, Refresh, Upload } from '@element-plus/icons-vue';
import type { Vehicle, VehicleStatus } from '@/types/vehicle';
import type { FormInstance } from 'element-plus';
import VirtualTable, { type VirtualTableColumn } from '@/components/crud/VirtualTable.vue';

// 通用车辆API响应类型
interface VehicleApiResponse {
  success?: boolean;
  data?: unknown;
  items?: Vehicle[];
  total?: number;
  [key: string]: unknown;
}

// 动态导入vehicleApi
async function importVehicleApi() {
  const { vehicleApi } = await import('@/api');
  return vehicleApi;
}

const loading = ref(false);
const syncing = ref(false);
const dialogVisible = ref(false);
const searchKeyword = ref('');
const vehicleList = ref<Vehicle[]>([]);
const selectedIds = ref<number[]>([]);
const currentPage = ref(1);
const pageSize = ref(20);
const total = ref(0);
const activeTab = ref('basic');

// VirtualTable 列配置
const columns = ref<VirtualTableColumn<Vehicle>[]>([
  { prop: 'selection', label: '', width: 55, align: 'center' },
  { prop: 'vehicle_id', label: 'ID', width: 80, align: 'center' },
  { prop: 'vehicle_name', label: '车辆名称', minWidth: 150 },
  { prop: 'device_id', label: '设备ID', width: 150 },
  { prop: 'license_plate', label: '车牌号', width: 120 },
  { prop: 'own_name', label: '车主', width: 120 },
  { prop: 'own_phone', label: '联系电话', width: 130 },
  { prop: 'sync_status', label: '同步状态', width: 100, align: 'center' },
  { prop: 'actions', label: '操作', width: 200 },
]);

const formRef = ref<FormInstance>();
const formData = reactive<Partial<Vehicle>>({
  // 基本资料
  vehicle_name: '',
  license_plate: '',
  vehicle_type: '',
  vehicle_color: '',
  vehicle_brand: '',
  vehicle_model: '',
  engine_no: '',
  frame_no: '',
  register_date: new Date().toISOString().slice(0, 16),
  inspection_date: new Date(new Date().setFullYear(new Date().getFullYear() + 1)).toISOString().slice(0, 16),
  insurance_date: new Date(new Date().setFullYear(new Date().getFullYear() + 1)).toISOString().slice(0, 16),
  seating_capacity: 2,
  load_capacity: 0,
  vehicle_length: 0,
  vehicle_width: 0,
  vehicle_height: 0,

  // 终端信息
  device_id: '',
  terminal_type: 'GPRS_GB北斗型',
  communication_type: 'TCP',
  sim_card_no: '',
  install_date: new Date().toISOString().slice(0, 16),
  install_address: '',
  install_technician: '',

  // 车主信息
  own_name: '',
  own_phone: '',
  own_id_card: '',
  own_address: '',
  own_email: '',

  // 运营信息
  group_id: 1,
  operation_status: 1,
  operation_route: '',
  operation_area: '',
  operation_company: '',
  driver_name: '',
  driver_phone: '',
  driver_license_no: '',

  // 财务信息
  purchase_price: 0,
  annual_fee: 0,
  insurance_fee: 0,

  // 其他信息
  remark: '',
  status: 'idle' as VehicleStatus,
  create_user_id: 1,
});

const formRules = {
  // 基本资料
  vehicle_name: [{ required: true, message: '请输入车辆名称', trigger: 'blur' }],
  license_plate: [
    { required: true, message: '请输入车牌号码', trigger: 'blur' },
    {
      pattern:
        /^[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼使领A-Z]{1}[A-Z]{1}[A-Z0-9]{4}[A-Z0-9挂学警港澳]{1}$/,
      message: '请输入正确的车牌号格式',
      trigger: 'blur',
    },
  ],
  vehicle_type: [{ required: true, message: '请输入车辆类型', trigger: 'blur' }],
  group_id: [{ required: true, message: '请选择隶属车队', trigger: 'change' }],
  own_name: [{ required: true, message: '请输入车主姓名', trigger: 'blur' }],
  own_phone: [
    { required: true, message: '请输入联系电话', trigger: 'blur' },
    { pattern: /^1[3-9]\d{9}$/, message: '请输入正确的手机号码格式', trigger: 'blur' },
  ],
  own_id_card: [
    {
      pattern: /^[1-9]\d{5}(18|19|20)\d{2}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])\d{3}[\dXx]$/,
      message: '请输入正确的身份证号码格式',
      trigger: 'blur',
    },
  ],
  own_email: [{ pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/, message: '请输入正确的邮箱格式', trigger: 'blur' }],
  sim_card_no: [{ pattern: /^\d{11}$/, message: '请输入正确的SIM卡号格式', trigger: 'blur' }],
};

const dialogTitle = computed(() => {
  return formData.vehicle_id ? '编辑车辆' : '添加车辆';
});

// 加载车辆列表
const loadVehicles = async () => {
  loading.value = true;
  try {
    // 动态导入并调用后端 API 获取车辆列表
    const vehicleApi = await importVehicleApi();
    
    const params: Record<string, string | number | boolean> = {
      page: currentPage.value,
      page_size: pageSize.value,
    };
    
    if (searchKeyword.value) {
      params.vehicle_name = searchKeyword.value;
    }
    
    const response = (await vehicleApi.getAll(params)) as unknown as VehicleApiResponse;
    
    let vehicleData = response;
    if (response && response.data) {
      vehicleData = response.data as VehicleApiResponse;
    }
    
    // 检查data是否是分页响应格式
    if (vehicleData && ('list' in vehicleData || 'items' in vehicleData)) {
      // 分页响应格式
      const data = vehicleData as { list?: Vehicle[]; items?: Vehicle[]; total?: number };
      vehicleList.value = data.list || data.items || [];
      total.value = data.total || 0;
    } else {
      // 直接返回数据数组
      const directResponse = vehicleData as unknown as Vehicle[];
      vehicleList.value = Array.isArray(directResponse) ? directResponse : [];
      total.value = directResponse.length || 0;
    }
  } catch (error) {
    console.error('加载车辆列表失败:', error);
    // 显示错误信息
    ElMessage.error('加载车辆列表失败');
  } finally {
    loading.value = false;
  }
};

// 搜索
const handleSearch = () => {
  // 实现搜索逻辑
  console.log('搜索:', searchKeyword.value);
  currentPage.value = 1;
  loadVehicles();
};

// 添加
const handleAdd = () => {
  activeTab.value = 'basic';
  Object.assign(formData, {
    // 基本资料
    vehicle_name: '',
    license_plate: '',
    vehicle_type: '',
    vehicle_color: '',
    vehicle_brand: '',
    vehicle_model: '',
    engine_no: '',
    frame_no: '',
    register_date: new Date().toISOString().slice(0, 16),
    inspection_date: new Date(new Date().setFullYear(new Date().getFullYear() + 1)).toISOString().slice(0, 16),
    insurance_date: new Date(new Date().setFullYear(new Date().getFullYear() + 1)).toISOString().slice(0, 16),
    seating_capacity: 2,
    load_capacity: 0,
    vehicle_length: 0,
    vehicle_width: 0,
    vehicle_height: 0,

    // 终端信息
    device_id: '',
    terminal_type: 'GPRS_GB北斗型',
    communication_type: 'TCP',
    sim_card_no: '',
    install_date: new Date().toISOString().slice(0, 16),
    install_address: '',
    install_technician: '',

    // 车主信息
    own_name: '',
    own_phone: '',
    own_id_card: '',
    own_address: '',
    own_email: '',

    // 运营信息
    group_id: 1,
    operation_status: 1,
    operation_route: '',
    operation_area: '',
    operation_company: '',
    driver_name: '',
    driver_phone: '',
    driver_license_no: '',

    // 财务信息
    purchase_price: 0,
    annual_fee: 0,
    insurance_fee: 0,

    // 其他信息
    remark: '',
    status: 'idle' as VehicleStatus,
    create_user_id: 1,
  });
  dialogVisible.value = true;
};

// 编辑
const handleEdit = (row: Vehicle) => {
  activeTab.value = 'basic';
  Object.assign(formData, row);
  // 确保日期字段格式正确
  if (row.register_date) formData.register_date = row.register_date;
  if (row.inspection_date) formData.inspection_date = row.inspection_date;
  if (row.insurance_date) formData.insurance_date = row.insurance_date;
  if (row.install_date) formData.install_date = row.install_date;
  dialogVisible.value = true;
};

// 删除
const handleDelete = async (row: Vehicle) => {
  try {
    await ElMessageBox.confirm('确定要删除该车辆吗？', '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    });

    // 动态导入并调用后端 API 删除车辆
    if (row.vehicle_id !== undefined) {
      const vehicleApi = await importVehicleApi();
      await vehicleApi.delete(row.vehicle_id as number) as unknown;
      ElMessage.success('删除成功');
      loadVehicles();
    } else {
      ElMessage.error('车辆ID无效');
    }
  } catch (error: unknown) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败');
    }
  }
};

// 批量删除
const handleBatchDelete = async () => {
  try {
    await ElMessageBox.confirm(`确定要删除选中的 ${selectedIds.value.length} 个车辆吗？`, '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    });

    // 动态导入并调用后端 API 批量删除车辆
    const vehicleApi = await importVehicleApi();
    for (const id of selectedIds.value) {
      await vehicleApi.delete(id) as unknown;
    }
    ElMessage.success('批量删除成功');
    loadVehicles();
  } catch (error: unknown) {
    if (error !== 'cancel') {
      ElMessage.error('批量删除失败');
    }
  }
};

// 同步
const handleSync = async () => {
  syncing.value = true;
  try {
    // 调用后端 API 同步数据
    await loadVehicles();
    ElMessage.success('数据同步成功');
  } catch {
    ElMessage.error('数据同步失败');
  } finally {
    syncing.value = false;
  }
};

// 提交表单
const handleSubmit = async () => {
  if (!formRef.value) return;

  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return;

    try {
      // 动态导入vehicleApi
      const vehicleApi = await importVehicleApi();
      if (formData.vehicle_id) {
        // 调用后端 API 更新车辆
        await vehicleApi.update(formData.vehicle_id as number, formData as Vehicle) as unknown;
        ElMessage.success('更新成功');
      } else {
        // 调用后端 API 添加车辆
        await vehicleApi.create(formData as Vehicle) as unknown;
        ElMessage.success('添加成功');
      }
      dialogVisible.value = false;
      loadVehicles();
    } catch {
      ElMessage.error('操作失败');
    }
  });
};

// 刷新
const handleRefresh = () => {
  loadVehicles();
};

// 行选择处理
const handleRowSelection = (row: Vehicle, event: Event) => {
  const checkbox = event.target as HTMLInputElement;
  const id = row.vehicle_id;
  if (id === undefined) return;
  
  if (checkbox.checked) {
    if (!selectedIds.value.includes(id)) {
      selectedIds.value.push(id);
    }
  } else {
    selectedIds.value = selectedIds.value.filter((selectedId) => selectedId !== id);
  }
};

// 全选处理
const handleSelectAll = (event: Event) => {
  const checkbox = event.target as HTMLInputElement;
  if (checkbox.checked) {
    selectedIds.value = vehicleList.value
      .map((item) => item.vehicle_id)
      .filter((id): id is number => id !== undefined);
  } else {
    selectedIds.value = [];
  }
};

// 选择变化（兼容旧的表格事件）
const handleSelectionChange = (selection: Vehicle[]) => {
  selectedIds.value = selection.map((item) => item.vehicle_id).filter((id): id is number => id !== undefined);
};

// 分页
const handlePageChange = (page: number) => {
  currentPage.value = page;
  loadVehicles();
};

const handleSizeChange = (size: number) => {
  pageSize.value = size;
  currentPage.value = 1;
  loadVehicles();
};

const handleDialogClose = () => {
  formRef.value?.resetFields();
};

onMounted(() => {
  loadVehicles();
});
</script>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.table-container {
  min-height: 500px;
}

.pagination {
  display: flex;
  justify-content: flex-end;
  margin-top: 20px;
}
</style>


