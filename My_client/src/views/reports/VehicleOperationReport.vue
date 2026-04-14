<template>

  <div class="vehicle-operation-report">

    <el-card>

      <template #header>

        <div class="card-header">

          <span>车辆运行报表</span>

        </div>

      </template>

      <div class="report-content">

        <el-form :model="searchForm" label-width="100px" class="search-form">

          <el-form-item label="日期范围">

            <el-date-picker

              v-model="searchForm.dateRange"

              type="daterange"

              range-separator="至"

              start-placeholder="开始日期"

              end-placeholder="结束日期"

              style="width: 300px"

            />

          </el-form-item>

          <el-form-item label="车辆ID">

            <el-select v-model="searchForm.vehicleId" placeholder="请选择车辆" style="width: 200px">

              <el-option label="所有车辆" value="" />

              <el-option v-for="vehicle in vehicles" :key="vehicle.id" :label="vehicle.licensePlate" :value="vehicle.id" />

            </el-select>

          </el-form-item>

          <el-form-item label="运行状态">

            <el-select v-model="searchForm.status" placeholder="请选择运行状态" style="width: 200px">

              <el-option label="所有状态" value="" />

              <el-option label="运行中" value="running" />

              <el-option label="停止" value="stopped" />

              <el-option label="故障" value="fault" />

              <el-option label="离线" value="offline" />

            </el-select>

          </el-form-item>

          <el-form-item>

            <el-button type="primary" @click="search">查询</el-button>

            <el-button @click="reset">重置</el-button>

            <el-button type="success" @click="exportReport">导出报表</el-button>

          </el-form-item>

        </el-form>



        <el-table :data="vehicleOperationData" style="width: 100%" border>

          <el-table-column prop="id" label="记录ID" width="100" />

          <el-table-column prop="vehicleId" label="车辆ID" width="100" />

          <el-table-column prop="licensePlate" label="车牌号" width="120" />

          <el-table-column prop="startTime" label="开始时间" width="180" />

          <el-table-column prop="endTime" label="结束时间" width="180" />

          <el-table-column prop="status" label="运行状态" width="100" />

          <el-table-column prop="mileage" label="行驶里程(km)" width="120" />

          <el-table-column prop="duration" label="运行时长(h)" width="120" />

          <el-table-column prop="fuelConsumption" label="油耗(L)" width="100" />

          <el-table-column prop="averageSpeed" label="平均速度(km/h)" width="120" />

          <el-table-column label="操作" width="100" fixed="right">

            <template #default="scope">

              <el-button type="primary" size="small" @click="viewDetail(scope.row)">详情</el-button>

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
// @ts-nocheck

import { ref, onMounted } from 'vue';

import { ElMessage } from 'element-plus';

import api from '@/api';



// 搜索表单

const searchForm = {

  dateRange: [] as [Date, Date] | null,

  vehicleId: '',

  status: '',

};



// 车辆列表

const vehicles = ref([]);



// 表格数据

const vehicleOperationData = ref([]);

const total = ref(0);

const currentPage = ref(1);

const pageSize = ref(10);



// 加载车辆列表

const loadVehicles = async () => {

  try {

    const response = await api.get('/api/vehicles') as any;

    vehicles.value = response?.items || [];

  } catch (error) {

    console.error('加载车辆列表失败:', error);

    ElMessage.error('加载车辆列表失败');

  }

};



// 加载车辆运行数据

const loadVehicleOperationData = async () => {

  try {

    // 构建查询参数

    const params = {

      page: currentPage.value,

      page_size: pageSize.value,

    };

    

    if (searchForm.dateRange) {

      params.start_date = searchForm.dateRange[0] instanceof Date ? searchForm.dateRange[0].toISOString().split('T')[0] : searchForm.dateRange[0];

      params.end_date = searchForm.dateRange[1] instanceof Date ? searchForm.dateRange[1].toISOString().split('T')[0] : searchForm.dateRange[1];

    }

    

    if (searchForm.vehicleId) {

      params.vehicle_id = searchForm.vehicleId;

    }

    

    if (searchForm.status) {

      params.status = searchForm.status;

    }

    

    // 调用API获取车辆运行数据

    const response = await api.get('/api/reports/vehicle-operation', { params }) as any;

    vehicleOperationData.value = response?.items || [];

    total.value = response?.total || 0;

  } catch (error) {

    console.error('加载车辆运行数据失败:', error);

    ElMessage.error('加载车辆运行数据失败');

  }

};



// 搜索

const search = () => {

  currentPage.value = 1;

  loadVehicleOperationData();

};



// 重置

const reset = () => {

  searchForm.dateRange = null;

  searchForm.vehicleId = '';

  searchForm.status = '';

  currentPage.value = 1;

  loadVehicleOperationData();

};



// 导出报表

const exportReport = async () => {

  try {

    // 构建查询参数

    const params = {};

    

    if (searchForm.dateRange) {

      params.start_date = searchForm.dateRange[0] instanceof Date ? searchForm.dateRange[0].toISOString().split('T')[0] : searchForm.dateRange[0];

      params.end_date = searchForm.dateRange[1] instanceof Date ? searchForm.dateRange[1].toISOString().split('T')[0] : searchForm.dateRange[1];

    }

    

    if (searchForm.vehicleId) {

      params.vehicle_id = searchForm.vehicleId;

    }

    

    if (searchForm.status) {

      params.status = searchForm.status;

    }

    

    // 调用API导出报表

    const response = await api.get('/api/reports/vehicle-operation/export', { 

      params,

      responseType: 'blob'

    }) as any;

    

    // 处理导出文件

    const blob = new Blob([response], { type: 'application/vnd.ms-excel' });

    const url = URL.createObjectURL(blob);

    const link = document.createElement('a');

    link.href = url;

    link.download = `车辆运行报表_${new Date().toISOString().split('T')[0]}.xlsx`;

    link.click();

    URL.revokeObjectURL(url);

    

    ElMessage.success('导出报表成功');

  } catch (error) {

    console.error('导出报表失败:', error);

    ElMessage.error('导出报表失败');

  }

};



// 查看详情

const viewDetail = (row) => {

  console.log('查看车辆运行详情:', row);

  // 这里可以实现查看详情功能

  ElMessage.info('查看详情功能开发中');

};



// 分页处理

const handleSizeChange = (size) => {

  pageSize.value = size;

  loadVehicleOperationData();

};



const handleCurrentChange = (current) => {

  currentPage.value = current;

  loadVehicleOperationData();

};



// 初始化

onMounted(() => {

  loadVehicles();

  loadVehicleOperationData();

});

</script>



<style scoped>

.vehicle-operation-report {

  padding: 20px;

}



.card-header {

  display: flex;

  justify-content: space-between;

  align-items: center;

}



.search-form {

  margin-bottom: 20px;

  padding: 20px;

  background-color: #f9f9f9;

  border-radius: 4px;

}



.report-content {

  min-height: 500px;

}



.pagination {

  margin-top: 20px;

  display: flex;

  justify-content: flex-end;

}

</style>



