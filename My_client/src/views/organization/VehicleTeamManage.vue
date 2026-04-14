<template>
  <div class="vehicle-team-manage">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>车队管理</span>
          <el-button type="primary" size="small" @click="showAddTeamDialog = true">
            <el-icon><Plus /></el-icon> 添加车队
          </el-button>
        </div>
      </template>

      <!-- 车队列表 -->
      <el-table :data="vehicleTeams" style="margin-top: 20px" v-loading="loading">
        <el-table-column prop="group_id" label="车队 ID" width="80" />
        <el-table-column prop="group_name" label="车队名称" />
        <el-table-column prop="parent_name" label="上级车队" />
        <el-table-column prop="description" label="描述" />
        <el-table-column prop="vehicle_count" label="车辆数量" width="100" />
        <el-table-column prop="create_time" label="创建时间" width="180" />
        <el-table-column label="操作" width="150" fixed="right">
          <template #default="scope">
            <el-button size="small" @click="handleEdit(scope.row)">
              <el-icon><Edit /></el-icon> 编辑
            </el-button>
            <el-button size="small" type="danger" @click="handleDelete(scope.row)">
              <el-icon><Delete /></el-icon> 删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination" style="margin-top: 20px; text-align: right">
        <el-pagination
          v-model:current-page="pagination.currentPage"
          v-model:page-size="pagination.pageSize"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          :total="teamTotal"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- 添加/编辑车队对话框 -->
    <el-dialog v-model="showAddTeamDialog" :title="editingTeam ? '编辑车队' : '添加车队'" width="600px">
      <el-form ref="teamFormRef" :model="editingTeam" label-width="120px">
        <el-form-item label="车队名称" required>
          <el-input v-model="editingTeam.group_name" placeholder="请输入车队名称" />
        </el-form-item>
        <el-form-item label="上级车队">
          <el-select v-model="editingTeam.parent_id" placeholder="请选择上级车队" clearable>
            <el-option
              v-for="team in vehicleTeams"
              :key="team.group_id"
              :label="team.group_name"
              :value="team.group_id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingTeam.description" type="textarea" placeholder="请输入车队描述" :rows="3" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddTeamDialog = false">取消</el-button>
          <el-button type="primary" @click="handleSave">保存</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, reactive, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, Edit, Delete } from '@element-plus/icons-vue';
import api from '@/api';
import type { FormInstance } from 'element-plus';

// 定义车队类型
interface VehicleTeam {
  group_id: number;
  group_name: string;
  parent_id: number | null;
  parent_name: string | null;
  description: string | null;
  vehicle_count: number;
  create_time: string;
  update_time?: string;
}

// API响应类型
interface ApiResponse<T> {
  items?: T[];
  total?: number;
  [key: string]: unknown;
}

// 车队数据
const vehicleTeams = ref<VehicleTeam[]>([]);
const loading = ref(false);
const teamTotal = ref(0);

// 分页配置
const pagination = reactive({
  currentPage: 1,
  pageSize: 10,
});

// 添加/编辑对话框
const showAddTeamDialog = ref(false);
const teamFormRef = ref<FormInstance>();
const editingTeam = ref<VehicleTeam>({
  group_id: 0,
  group_name: '',
  parent_id: null,
  parent_name: '',
  description: '',
  vehicle_count: 0,
  create_time: '',
  update_time: '',
});

// 加载车队列表
const loadVehicleTeams = async () => {
  loading.value = true;
  try {
    const params = {
      page: pagination.currentPage,
      page_size: pagination.pageSize,
    };
    const response = await api.get('/api/vehicle-groups', { params });
    if (response && response.list) {
      vehicleTeams.value = response.list || [];
      teamTotal.value = response.total || 0;
    }
  } catch (error) {
    console.error('获取车队列表失败:', error);
    ElMessage.error('获取车队列表失败');
  } finally {
    loading.value = false;
  }
};

// 编辑车队
const handleEdit = (team: VehicleTeam) => {
  editingTeam.value = { ...team };
  showAddTeamDialog.value = true;
};

// 删除车队
const handleDelete = async (team: VehicleTeam) => {
  ElMessageBox.confirm('确定要删除该车队吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/vehicle-groups/${team.group_id}`);
        ElMessage.success('删除成功');
        loadVehicleTeams();
      } catch (error) {
        console.error('删除车队失败:', error);
        ElMessage.error('删除车队失败');
      }
    })
    .catch(() => {
      // 取消删除
    });
};

// 保存车队
const handleSave = async () => {
  try {
    const teamData = {
      group_name: editingTeam.value.group_name,
      parent_id: editingTeam.value.parent_id,
      description: editingTeam.value.description,
    };

    if (editingTeam.value.group_id) {
      // 更新现有车队
      await api.put(`/api/vehicle-groups/${editingTeam.value.group_id}`, teamData);
      ElMessage.success('更新成功');
    } else {
      // 添加新车队
      await api.post('/api/vehicle-groups', teamData);
      ElMessage.success('添加成功');
    }

    showAddTeamDialog.value = false;
    loadVehicleTeams();
  } catch (error) {
    console.error('保存车队失败:', error);
    ElMessage.error('保存车队失败');
  }
};

// 分页处理
const handleSizeChange = (size: number) => {
  pagination.pageSize = size;
  loadVehicleTeams();
};

const handleCurrentChange = (current: number) => {
  pagination.currentPage = current;
  loadVehicleTeams();
};

// 初始化数据
onMounted(async () => {
  await loadVehicleTeams();
});
</script>

<style scoped>
.vehicle-team-manage {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>


