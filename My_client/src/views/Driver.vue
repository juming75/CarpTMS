﻿﻿﻿﻿﻿<template>
  <div class="driver-management">
    <div class="driver-header">
      <h2>司机管理</h2>
    </div>

    <!-- 主功能标签页 -->
    <el-tabs v-model="activeMainTab" type="card" class="main-tabs">
      <!-- 司机信息管理 -->
      <el-tab-pane label="司机信息管理" name="info">
        <div class="tab-content">
          <!-- 查询区域 -->
          <el-card class="query-card" shadow="hover" :body-style="{ padding: '20px' }">
            <el-form :model="driverQueryForm" :inline="true" label-position="right" label-width="80px" size="small">
              <el-form-item label="司机姓名">
                <el-input v-model="driverQueryForm.name" placeholder="请输入司机姓名" clearable></el-input>
              </el-form-item>
              <el-form-item label="手机号码">
                <el-input v-model="driverQueryForm.phone" placeholder="请输入手机号码" clearable></el-input>
              </el-form-item>
              <el-form-item label="状态">
                <el-select v-model="driverQueryForm.status" placeholder="请选择状态" clearable>
                  <el-option label="全部" value=""></el-option>
                  <el-option label="在职" value="active"></el-option>
                  <el-option label="离职" value="inactive"></el-option>
                </el-select>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="handleDriverQuery">
                  <el-icon><Search /></el-icon>
                  查询
                </el-button>
                <el-button @click="handleDriverReset">
                  <el-icon><Refresh /></el-icon>
                  重置
                </el-button>
                <el-button type="primary" @click="handleDriverAdd">
                  <el-icon><Plus /></el-icon>
                  新增司机
                </el-button>
              </el-form-item>
            </el-form>
          </el-card>

          <!-- 司机列表 -->
          <el-card class="driver-list-card" shadow="hover" style="margin-top: 20px">
            <div class="driver-list">
              <el-table :data="filteredDriverList" stripe size="small" v-loading="driverLoading">
                <el-table-column prop="id" label="ID" width="60"></el-table-column>
                <el-table-column prop="name" label="司机姓名"></el-table-column>
                <el-table-column prop="phone" label="手机号码" width="120"></el-table-column>
                <el-table-column prop="license" label="驾驶证号" width="200"></el-table-column>
                <el-table-column prop="licenseType" label="准驾车型" width="100"></el-table-column>
                <el-table-column prop="status" label="状态" width="80">
                  <template #default="scope">
                    <el-tag :type="scope.row.status === 'active' ? 'success' : 'info'" size="small">
                      {{ scope.row.status === 'active' ? '在职' : '离职' }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="joinDate" label="入职日期" width="120"></el-table-column>
                <el-table-column prop="lastLogin" label="最后登录" width="180"></el-table-column>
                <el-table-column label="操作" width="200">
                  <template #default="scope">
                    <el-button type="primary" text size="small" @click="handleDriverView(scope.row)">
                      <el-icon><View /></el-icon>
                      查看
                    </el-button>
                    <el-button type="success" text size="small" @click="handleDriverEdit(scope.row)">
                      <el-icon><Edit /></el-icon>
                      编辑
                    </el-button>
                    <el-button type="danger" text size="small" @click="handleDriverDelete(scope.row)">
                      <el-icon><Delete /></el-icon>
                      删除
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>

              <!-- 分页 -->
              <div class="pagination-container">
                <el-pagination
                  v-model:current-page="driverCurrentPage"
                  v-model:page-size="driverPageSize"
                  :page-sizes="[10, 20, 50, 100]"
                  layout="total, sizes, prev, pager, next, jumper"
                  :total="driverTotal"
                  @size-change="handleDriverSizeChange"
                  @current-change="handleDriverCurrentChange"
                ></el-pagination>
              </div>
            </div>
          </el-card>
        </div>
      </el-tab-pane>

      <!-- 资质审核 -->
      <el-tab-pane label="资质审核" name="qualification">
        <div class="tab-content">
          <!-- 查询区域 -->
          <el-card class="query-card" shadow="hover" :body-style="{ padding: '20px' }">
            <el-form :model="qualQueryForm" :inline="true" label-position="right" label-width="80px" size="small">
              <el-form-item label="司机姓名">
                <el-input v-model="qualQueryForm.name" placeholder="请输入司机姓名" clearable></el-input>
              </el-form-item>
              <el-form-item label="审核状态">
                <el-select v-model="qualQueryForm.status" placeholder="请选择审核状态" clearable>
                  <el-option label="全部" value=""></el-option>
                  <el-option label="待审核" value="pending"></el-option>
                  <el-option label="审核通过" value="approved"></el-option>
                  <el-option label="审核拒绝" value="rejected"></el-option>
                </el-select>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="handleQualQuery">
                  <el-icon><Search /></el-icon>
                  查询
                </el-button>
                <el-button @click="handleQualReset">
                  <el-icon><Refresh /></el-icon>
                  重置
                </el-button>
              </el-form-item>
            </el-form>
          </el-card>

          <!-- 资质列表 -->
          <el-card class="qual-list-card" shadow="hover" style="margin-top: 20px">
            <div class="qual-list">
              <el-table :data="filteredQualList" stripe size="small" v-loading="qualLoading">
                <el-table-column prop="id" label="ID" width="60"></el-table-column>
                <el-table-column prop="driverName" label="司机姓名"></el-table-column>
                <el-table-column prop="license" label="驾驶证号" width="200"></el-table-column>
                <el-table-column prop="licenseType" label="准驾车型" width="100"></el-table-column>
                <el-table-column prop="expiryDate" label="有效期至" width="120"></el-table-column>
                <el-table-column prop="status" label="审核状态" width="100">
                  <template #default="scope">
                    <el-tag :type="getStatusTagType(scope.row.status)" size="small">
                      {{ getStatusText(scope.row.status) }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="applyDate" label="申请日期" width="120"></el-table-column>
                <el-table-column label="操作" width="150">
                  <template #default="scope">
                    <el-button type="primary" text size="small" @click="handleQualView(scope.row)">
                      <el-icon><View /></el-icon>
                      查看
                    </el-button>
                    <el-button
                      v-if="scope.row.status === 'pending'"
                      type="success"
                      text
                      size="small"
                      @click="handleQualApprove(scope.row)"
                    >
                      <el-icon><Check /></el-icon>
                      通过
                    </el-button>
                    <el-button
                      v-if="scope.row.status === 'pending'"
                      type="danger"
                      text
                      size="small"
                      @click="handleQualReject(scope.row)"
                    >
                      <el-icon><Close /></el-icon>
                      拒绝
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </el-card>
        </div>
      </el-tab-pane>

      <!-- 排班管理 -->
      <el-tab-pane label="排班管理" name="schedule">
        <div class="tab-content">
          <!-- 查询区域 -->
          <el-card class="query-card" shadow="hover" :body-style="{ padding: '20px' }">
            <el-form :model="scheduleQueryForm" :inline="true" label-position="right" label-width="100px" size="small">
              <el-form-item label="司机姓名">
                <el-select v-model="scheduleQueryForm.driverId" placeholder="请选择司机">
                  <el-option label="全部" value=""></el-option>
                  <el-option
                    v-for="driver in driverList"
                    :key="driver.id"
                    :label="driver.name"
                    :value="driver.id"
                  ></el-option>
                </el-select>
              </el-form-item>
              <el-form-item label="排班日期">
                <el-date-picker
                  v-model="scheduleQueryForm.date"
                  type="date"
                  placeholder="请选择日期"
                  size="small"
                ></el-date-picker>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="handleScheduleQuery">
                  <el-icon><Search /></el-icon>
                  查询
                </el-button>
                <el-button @click="handleScheduleReset">
                  <el-icon><Refresh /></el-icon>
                  重置
                </el-button>
                <el-button type="primary" @click="handleScheduleAdd">
                  <el-icon><Plus /></el-icon>
                  新增排班
                </el-button>
              </el-form-item>
            </el-form>
          </el-card>

          <!-- 排班列表 -->
          <el-card class="schedule-list-card" shadow="hover" style="margin-top: 20px">
            <div class="schedule-list">
              <el-table :data="filteredScheduleList" stripe size="small" v-loading="scheduleLoading">
                <el-table-column prop="id" label="ID" width="60"></el-table-column>
                <el-table-column prop="driverName" label="司机姓名"></el-table-column>
                <el-table-column prop="vehicleLicense" label="车辆牌号" width="120"></el-table-column>
                <el-table-column prop="date" label="排班日期" width="120"></el-table-column>
                <el-table-column prop="shift" label="班次" width="100">
                  <template #default="scope">
                    <el-tag :type="scope.row.shift === 'morning' ? 'primary' : 'success'" size="small">
                      {{ scope.row.shift === 'morning' ? '早班' : '晚班' }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="startTime" label="开始时间" width="100"></el-table-column>
                <el-table-column prop="endTime" label="结束时间" width="100"></el-table-column>
                <el-table-column prop="status" label="状态" width="80">
                  <template #default="scope">
                    <el-tag :type="scope.row.status === 'active' ? 'success' : 'info'" size="small">
                      {{ scope.row.status === 'active' ? '有效' : '无效' }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column label="操作" width="150">
                  <template #default="scope">
                    <el-button type="primary" text size="small" @click="handleScheduleEdit(scope.row)">
                      <el-icon><Edit /></el-icon>
                      编辑
                    </el-button>
                    <el-button type="danger" text size="small" @click="handleScheduleDelete(scope.row)">
                      <el-icon><Delete /></el-icon>
                      删除
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </el-card>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- 司机详情对话框 -->
    <el-dialog v-model="driverDetailVisible" title="司机详情" width="600px">
      <div v-if="selectedDriver" class="driver-detail">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="基本信息">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-descriptions :column="1" size="small">
                  <el-descriptions-item label="司机姓名">{{ selectedDriver.name }}</el-descriptions-item>
                  <el-descriptions-item label="手机号码">{{ selectedDriver.phone }}</el-descriptions-item>
                  <el-descriptions-item label="性别">{{
                    selectedDriver.gender === 'male' ? '男' : '女'
                  }}</el-descriptions-item>
                  <el-descriptions-item label="年龄">{{ selectedDriver.age }}</el-descriptions-item>
                </el-descriptions>
              </el-col>
              <el-col :span="12">
                <el-descriptions :column="1" size="small">
                  <el-descriptions-item label="状态"
                    ><el-tag :type="selectedDriver.status === 'active' ? 'success' : 'info'">{{
                      selectedDriver.status === 'active' ? '在职' : '离职'
                    }}</el-tag></el-descriptions-item
                  >
                  <el-descriptions-item label="入职日期">{{ selectedDriver.joinDate }}</el-descriptions-item>
                  <el-descriptions-item label="最后登录">{{ selectedDriver.lastLogin }}</el-descriptions-item>
                </el-descriptions>
              </el-col>
            </el-row>
          </el-descriptions-item>
          <el-descriptions-item label="驾驶证信息">
            <el-row :gutter="20">
              <el-col :span="12">
                <el-descriptions :column="1" size="small">
                  <el-descriptions-item label="驾驶证号">{{ selectedDriver.license }}</el-descriptions-item>
                  <el-descriptions-item label="准驾车型">{{ selectedDriver.licenseType }}</el-descriptions-item>
                </el-descriptions>
              </el-col>
              <el-col :span="12">
                <el-descriptions :column="1" size="small">
                  <el-descriptions-item label="有效期至">{{ selectedDriver.licenseExpiry }}</el-descriptions-item>
                  <el-descriptions-item label="发证机关">{{ selectedDriver.licenseIssue }}</el-descriptions-item>
                </el-descriptions>
              </el-col>
            </el-row>
          </el-descriptions-item>
          <el-descriptions-item label="备注">
            {{ selectedDriver.remark || '无' }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="driverDetailVisible = false">关闭</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 司机编辑对话框 -->
    <el-dialog v-model="driverDialogVisible" :title="driverForm.id ? '编辑司机' : '新增司机'" width="600px">
      <el-form :model="driverForm" label-position="right" label-width="100px" size="small">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="司机姓名" prop="name">
              <el-input v-model="driverForm.name" placeholder="请输入司机姓名"></el-input>
            </el-form-item>
            <el-form-item label="手机号码" prop="phone">
              <el-input v-model="driverForm.phone" placeholder="请输入手机号码"></el-input>
            </el-form-item>
            <el-form-item label="性别" prop="gender">
              <el-select v-model="driverForm.gender" placeholder="请选择性别">
                <el-option label="男" value="male"></el-option>
                <el-option label="女" value="female"></el-option>
              </el-select>
            </el-form-item>
            <el-form-item label="年龄" prop="age">
              <el-input v-model="driverForm.age" type="number" placeholder="请输入年龄"></el-input>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="驾驶证号" prop="license">
              <el-input v-model="driverForm.license" placeholder="请输入驾驶证号"></el-input>
            </el-form-item>
            <el-form-item label="准驾车型" prop="licenseType">
              <el-input v-model="driverForm.licenseType" placeholder="请输入准驾车型"></el-input>
            </el-form-item>
            <el-form-item label="有效期至" prop="licenseExpiry">
              <el-date-picker
                v-model="driverForm.licenseExpiry"
                type="date"
                placeholder="请选择有效期至"
              ></el-date-picker>
            </el-form-item>
            <el-form-item label="状态" prop="status">
              <el-select v-model="driverForm.status" placeholder="请选择状态">
                <el-option label="在职" value="active"></el-option>
                <el-option label="离职" value="inactive"></el-option>
              </el-select>
            </el-form-item>
          </el-col>
        </el-row>
        <el-form-item label="备注" prop="remark">
          <el-input v-model="driverForm.remark" type="textarea" rows="3" placeholder="请输入备注信息"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="driverDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleDriverSave">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, computed, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Plus, Edit, Delete, Search, Refresh, View, Check, Close } from '@element-plus/icons-vue';
import api from '@/api';

// 主功能标签页
const activeMainTab = ref('info');

// 司机信息管理相关
const driverLoading = ref(false);
const driverQueryForm = ref({
  name: '',
  phone: '',
  status: '',
});

// 司机类型定义
interface DriverItem {
  id: number;
  name: string;
  phone: string;
  status: string;
  [key: string]: unknown;
}

// 资质类型定义
interface QualItem {
  id: number;
  driverName: string;
  status: string;
  [key: string]: unknown;
}

// 排班类型定义
interface ScheduleItem {
  id: number;
  driverId: string;
  date: string;
  [key: string]: unknown;
}

const driverList = ref<DriverItem[]>([]);
const driverTotal = ref(0);
const driverCurrentPage = ref(1);
const driverPageSize = ref(10);

// 计算过滤后的司机列表
const filteredDriverList = computed(() => {
  return driverList.value;
});

// 资质审核相关
const qualLoading = ref(false);
const qualQueryForm = ref({
  name: '',
  status: '',
});

const qualList = ref<QualItem[]>([]);

// 计算过滤后的资质列表
const filteredQualList = computed(() => {
  let result = [...qualList.value];

  // 按司机姓名过滤
  if (qualQueryForm.value.name) {
    result = result.filter((qual) => qual.driverName.includes(qualQueryForm.value.name));
  }

  // 按审核状态过滤
  if (qualQueryForm.value.status) {
    result = result.filter((qual) => qual.status === qualQueryForm.value.status);
  }

  return result;
});

// 排班管理相关
const scheduleLoading = ref(false);
const scheduleQueryForm = ref({
  driverId: '',
  date: '',
});

const scheduleList = ref<ScheduleItem[]>([]);

// 计算过滤后的排班列表
const filteredScheduleList = computed(() => {
  let result = [...scheduleList.value];

  // 按司机过滤
  if (scheduleQueryForm.value.driverId) {
    result = result.filter((schedule) => schedule.driverId === Number(scheduleQueryForm.value.driverId));
  }

  // 按日期过滤
  if (scheduleQueryForm.value.date) {
    const dateStr = new Date(scheduleQueryForm.value.date).toISOString().split('T')[0];
    result = result.filter((schedule) => schedule.date === dateStr);
  }

  return result;
});

// 对话框状态
const driverDetailVisible = ref(false);
const driverDialogVisible = ref(false);

// 选中的司机
const selectedDriver = ref<DriverItem | null>(null);

// 司机表单数据
const driverForm = ref({
  id: '',
  name: '',
  phone: '',
  gender: 'male',
  age: '',
  license: '',
  licenseType: '',
  licenseExpiry: '',
  licenseIssue: '',
  status: 'active',
  joinDate: new Date().toISOString().split('T')[0],
  lastLogin: new Date().toISOString().split('T')[0] + ' 00:00:00',
  remark: '',
});

// 获取审核状态文本
const getStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    pending: '待审核',
    approved: '审核通过',
    rejected: '审核拒绝',
  };
  return statusMap[status] || status;
};

// 获取审核状态标签类型
const getStatusTagType = (status: string) => {
  const typeMap: Record<string, string> = {
    pending: 'warning',
    approved: 'success',
    rejected: 'danger',
  };
  return typeMap[status] || 'info';
};

// 加载司机列表
const fetchDrivers = async () => {
  driverLoading.value = true;
  try {
    const params: Record<string, unknown> = {
      page: driverCurrentPage.value,
      page_size: driverPageSize.value,
    };
    if (driverQueryForm.value.name) {
      params.driver_name = driverQueryForm.value.name;
    }
    if (driverQueryForm.value.phone) {
      params.phone_number = driverQueryForm.value.phone;
    }
    if (driverQueryForm.value.status) {
      const statusMap: Record<string, number> = { active: 1, inactive: 0 };
      params.status = statusMap[driverQueryForm.value.status];
    }
    const response = await api.get('/api/drivers', { params }) as any;
    
    // 处理后端响应格式：{ code: 200, message: "...", data: { list: [...], total: number, ... } }
    let driverData = response;
    if (response && response.data) {
      driverData = response.data;
    }
    
    if (driverData && driverData.list) {
      driverList.value = driverData.list.map((item: any) => ({
        id: item.driver_id,
        name: item.driver_name,
        phone: item.phone_number || '',
        license: item.license_number || '',
        licenseType: item.license_type || '',
        licenseExpiry: item.license_expiry || '',
        licenseIssue: '',
        status: item.status === 1 ? 'active' : 'inactive',
        joinDate: item.hire_date ? new Date(item.hire_date).toLocaleDateString() : item.create_time ? new Date(item.create_time).toLocaleDateString() : '',
        lastLogin: item.update_time ? new Date(item.update_time).toLocaleString() : '',
        gender: 'male',
        age: '',
        remark: ''
      }));
      driverTotal.value = driverData.total || 0;
    } else {
      driverList.value = [];
      driverTotal.value = 0;
    }
  } catch (error) {
    console.error('获取司机列表失败:', error);
    ElMessage.error('获取司机列表失败');
  } finally {
    driverLoading.value = false;
  }
};

// 司机信息管理操作
const handleDriverQuery = () => {
  driverCurrentPage.value = 1;
  fetchDrivers();
};

const handleDriverReset = () => {
  driverQueryForm.value = {
    name: '',
    phone: '',
    status: '',
  };
  driverCurrentPage.value = 1;
  fetchDrivers();
};

const handleDriverSizeChange = (size: number) => {
  driverPageSize.value = size;
  driverCurrentPage.value = 1;
  fetchDrivers();
};

const handleDriverCurrentChange = (page: number) => {
  driverCurrentPage.value = page;
  fetchDrivers();
};

const handleDriverAdd = () => {
  driverForm.value = {
    id: '',
    name: '',
    phone: '',
    gender: 'male',
    age: '',
    license: '',
    licenseType: '',
    licenseExpiry: '',
    licenseIssue: '',
    status: 'active',
    joinDate: new Date().toISOString().split('T')[0],
    lastLogin: new Date().toISOString().split('T')[0] + ' 00:00:00',
    remark: '',
  };
  driverDialogVisible.value = true;
};

const handleDriverEdit = (row: DriverItem) => {
  driverForm.value = { ...row };
  driverDialogVisible.value = true;
};

const handleDriverView = async (row: DriverItem) => {
  try {
    const response = await api.get(`/api/drivers/${row.id}`) as any;
    
    // 处理后端响应格式：{ code: 200, message: "...", data: {...} }
    let driverData = response;
    if (response && response.data) {
      driverData = response.data;
    }
    
    if (driverData) {
      selectedDriver.value = {
        id: driverData.driver_id,
        name: driverData.driver_name,
        phone: driverData.phone_number || '',
        license: driverData.license_number || '',
        licenseType: driverData.license_type || '',
        licenseExpiry: driverData.license_expiry || '',
        licenseIssue: '',
        status: driverData.status === 1 ? 'active' : 'inactive',
        joinDate: driverData.hire_date ? new Date(driverData.hire_date).toLocaleDateString() : driverData.create_time ? new Date(driverData.create_time).toLocaleDateString() : '',
        lastLogin: driverData.update_time ? new Date(driverData.update_time).toLocaleString() : '',
        remark: driverData.remark || '',
        gender: 'male',
        age: ''
      };
      driverDetailVisible.value = true;
    }
  } catch (error) {
    console.error('获取司机详情失败:', error);
    ElMessage.error('获取司机详情失败');
  }
};

const handleDriverDelete = async (row: DriverItem) => {
  try {
    await api.delete(`/api/drivers/${row.id}`);
    ElMessage.success('司机删除成功');
    fetchDrivers();
  } catch (error) {
    console.error('删除司机失败:', error);
    ElMessage.error('删除司机失败');
  }
};

const handleDriverSave = async () => {
  try {
    // 状态映射：前端字符串 -> 后端数字
    const statusMap = {
      active: 1,
      inactive: 0,
    };

    const driverData = {
      driver_name: driverForm.value.name,
      license_number: driverForm.value.license,
      phone_number: driverForm.value.phone,
      email: '',
      status: statusMap[driverForm.value.status],
    };

    if (driverForm.value.id) {
      // 编辑
      await api.put(`/api/drivers/${driverForm.value.id}`, driverData);
      ElMessage.success('司机更新成功');
    } else {
      // 新增
      await api.post('/api/drivers', driverData);
      ElMessage.success('司机创建成功');
    }
    driverDialogVisible.value = false;
    fetchDrivers();
  } catch (error) {
    console.error('保存司机失败:', error);
    ElMessage.error('保存司机失败');
  }
};

// 资质审核操作
const handleQualQuery = () => {
  qualLoading.value = true;
  setTimeout(() => {
    qualLoading.value = false;
    ElMessage.success('查询完成');
  }, 500);
};

const handleQualReset = () => {
  qualQueryForm.value = {
    name: '',
    status: '',
  };
};

const handleQualView = (_row: QualItem) => {
  ElMessage.info('查看资质详情');
};

const handleQualApprove = (row: QualItem) => {
  row.status = 'approved';
  ElMessage.success('审核通过');
};

const handleQualReject = (row: QualItem) => {
  row.status = 'rejected';
  ElMessage.success('审核拒绝');
};

// 排班管理操作
const handleScheduleQuery = () => {
  scheduleLoading.value = true;
  setTimeout(() => {
    scheduleLoading.value = false;
    ElMessage.success('查询完成');
  }, 500);
};

const handleScheduleReset = () => {
  scheduleQueryForm.value = {
    driverId: '',
    date: '',
  };
};

const handleScheduleAdd = () => {
  ElMessage.info('新增排班');
};

const handleScheduleEdit = (_row: ScheduleItem) => {
  ElMessage.info('编辑排班');
};

const handleScheduleDelete = (row: ScheduleItem) => {
  scheduleList.value = scheduleList.value.filter((item) => item.id !== row.id);
  ElMessage.success('排班删除成功');
};

onMounted(() => {
  console.log('Driver 初始化完成');
  fetchDrivers();
});
</script>

<style scoped>
.driver-management {
  padding: 20px;
}

.driver-header {
  margin-bottom: 20px;
}

.driver-header h2 {
  margin: 0;
  font-size: 20px;
  color: #303133;
}

.tab-content {
  margin-top: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.query-card {
  margin-bottom: 20px;
}

.driver-list,
.qual-list,
.schedule-list {
  margin-top: 10px;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

.driver-detail {
  padding: 10px 0;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>


