<template>
  <div class="department-manage">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>部门管理</span>
          <el-button type="primary" size="small" @click="showAddDepartmentDialog = true">
            <el-icon><Plus /></el-icon> 添加部门
          </el-button>
        </div>
      </template>

      <!-- 部门列表 -->
      <el-table :data="departments" style="margin-top: 20px" v-loading="loading">
        <el-table-column prop="department_id" label="部门ID" width="80" />
        <el-table-column prop="department_name" label="部门名称" />
        <el-table-column prop="parent_department_name" label="上级部门" />
        <el-table-column prop="manager_name" label="部门经理" />
        <el-table-column prop="phone" label="联系电话" />
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
          :total="departmentTotal"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- 添加/编辑部门对话框 -->
    <el-dialog v-model="showAddDepartmentDialog" :title="editingDepartment ? '编辑部门' : '添加部门'" width="600px">
      <el-form ref="departmentFormRef" :model="editingDepartment" label-width="120px">
        <el-form-item label="部门名称" required>
          <el-input v-model="editingDepartment.department_name" placeholder="请输入部门名称" />
        </el-form-item>
        <el-form-item label="上级部门">
          <el-select v-model="editingDepartment.parent_department_id" placeholder="请选择上级部门">
            <el-option label="无" value="" />
            <el-option
              v-for="dept in departments"
              :key="dept.department_id"
              :label="dept.department_name"
              :value="dept.department_id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="部门经理">
          <el-select v-model="editingDepartment.manager_id" placeholder="请选择部门经理">
            <el-option label="无" value="" />
            <el-option v-for="user in users" :key="user.user_id" :label="user.real_name" :value="user.user_id" />
          </el-select>
        </el-form-item>
        <el-form-item label="联系电话">
          <el-input v-model="editingDepartment.phone" placeholder="请输入联系电话" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingDepartment.description" type="textarea" placeholder="请输入部门描述" :rows="3" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddDepartmentDialog = false">取消</el-button>
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

// 定义部门类型
interface Department {
  department_id: number;
  department_name: string;
  parent_department_id: string | number;
  parent_department_name: string;
  manager_id: string | number;
  manager_name: string;
  phone: string;
  description: string;
  create_time: string;
}

// 定义用户类型
interface User {
  user_id: number;
  real_name: string;
}



// 用户数据
const users = ref<User[]>([
  { user_id: 1, real_name: '管理员' },
  { user_id: 2, real_name: '用户1' },
]);

// 部门数据
const departments = ref<Department[]>([]);
const departmentTotal = ref(0);
const loading = ref(false);

// 分页配置
const pagination = reactive({
  currentPage: 1,
  pageSize: 10,
});

// 添加/编辑对话框
const showAddDepartmentDialog = ref(false);
const departmentFormRef = ref<FormInstance>();
const editingDepartment = ref<Department>({
  department_id: 0,
  department_name: '',
  parent_department_id: '',
  parent_department_name: '',
  manager_id: '',
  manager_name: '',
  phone: '',
  description: '',
  create_time: '',
});

// 加载部门列表
const fetchDepartments = async () => {
  loading.value = true;
  try {
    const params = {
      page: pagination.currentPage,
      page_size: pagination.pageSize,
    };
    const apiResponse = await api.get('/api/departments', { params });
    if (apiResponse && apiResponse.list) {
      departments.value = apiResponse.list || [];
      departmentTotal.value = apiResponse.total || 0;
    }
  } catch (error) {
    console.error('获取部门列表失败:', error);
    ElMessage.error('获取部门列表失败');
  } finally {
    loading.value = false;
  }
};

// 编辑部门
const handleEdit = (department: Department) => {
  editingDepartment.value = { ...department };
  showAddDepartmentDialog.value = true;
};

// 删除部门
const handleDelete = async (department: Department) => {
  ElMessageBox.confirm('确定要删除该部门吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/departments/${department.department_id}`);
        ElMessage.success('删除成功');
        fetchDepartments();
      } catch (error) {
        console.error('删除部门失败:', error);
        ElMessage.error('删除部门失败');
      }
    })
    .catch(() => {
      // 取消删除
    });
};

// 保存部门
const handleSave = async () => {
  try {
    if (editingDepartment.value.department_id) {
      // 更新现有部门
      await api.put(`/api/departments/${editingDepartment.value.department_id}`, editingDepartment.value);
      ElMessage.success('更新成功');
    } else {
      // 添加新部门
      await api.post('/api/departments', editingDepartment.value);
      ElMessage.success('添加成功');
    }
    showAddDepartmentDialog.value = false;
    fetchDepartments();
  } catch (error) {
    console.error('保存部门失败:', error);
    ElMessage.error('保存部门失败');
  }
};

// 分页处理
const handleSizeChange = (size: number) => {
  pagination.pageSize = size;
  fetchDepartments();
};

const handleCurrentChange = (current: number) => {
  pagination.currentPage = current;
  fetchDepartments();
};

// 初始化时加载部门列表
onMounted(() => {
  fetchDepartments();
});
</script>

<style scoped>
.department-manage {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>


