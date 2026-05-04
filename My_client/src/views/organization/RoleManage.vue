<template>
  <div class="role-manage">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>角色管理</span>
          <el-button type="primary" size="small" @click="showAddRoleDialog = true">
            <el-icon><Plus /></el-icon> 添加角色
          </el-button>
        </div>
      </template>

      <!-- 角色列表 -->
      <el-table :data="roles" style="margin-top: 20px">
        <el-table-column prop="role_id" label="角色ID" width="80" />
        <el-table-column prop="role_name" label="角色名称" />
        <el-table-column prop="description" label="描述" min-width="200" />
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
          :total="roles.length"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- 添加/编辑角色对话框 -->
    <el-dialog v-model="showAddRoleDialog" :title="editingRole ? '编辑角色' : '添加角色'" width="600px">
      <el-form ref="roleFormRef" :model="editingRole" label-width="120px">
        <el-form-item label="角色名称" required>
          <el-input v-model="editingRole.role_name" placeholder="请输入角色名称" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="editingRole.description" type="textarea" placeholder="请输入角色描述" :rows="3" />
        </el-form-item>

        <!-- 权限设置 -->
        <el-form-item label="权限设置">
          <div class="permission-setting">
            <el-checkbox-group v-model="editingRole.permissions">
              <el-checkbox label="dashboard">仪表盘</el-checkbox>
              <el-checkbox label="real_time_monitor">实时监控</el-checkbox>
              <el-checkbox label="history_data">历史数据</el-checkbox>
              <el-checkbox label="vehicle_manage">车辆管理</el-checkbox>
              <el-checkbox label="driver_manage">司机管理</el-checkbox>
              <el-checkbox label="order_manage">订单管理</el-checkbox>
              <el-checkbox label="finance_manage">财务管理</el-checkbox>
              <el-checkbox label="reports">数据报表</el-checkbox>
              <el-checkbox label="system_settings">系统设置</el-checkbox>
              <el-checkbox label="organization_manage">组织管理</el-checkbox>
            </el-checkbox-group>
          </div>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddRoleDialog = false">取消</el-button>
          <el-button type="primary" @click="handleSave">保存</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, Edit, Delete } from '@element-plus/icons-vue';
import api from '@/api';
import type { FormInstance } from 'element-plus';

// 角色类型定义
interface Role {
  role_id: number;
  role_name: string;
  description: string;
  permissions: string[];
  create_time: string;
}

// API响应数据类型
interface ApiRoleData {
  role_id: number;
  role_name: string;
  description?: string;
  created_at: string;
  [key: string]: unknown;
}

// 角色数据
const roles = ref<Role[]>([]);

// 分页配置
const pagination = reactive({
  currentPage: 1,
  pageSize: 10,
});

// 添加/编辑对话框
const showAddRoleDialog = ref(false);
const roleFormRef = ref<FormInstance>();
const editingRole = ref<Role>({
  role_id: 0,
  role_name: '',
  description: '',
  permissions: [],
  create_time: '',
});

// 编辑角色
const handleEdit = (role: Role) => {
  editingRole.value = { ...role, permissions: [...role.permissions] };
  showAddRoleDialog.value = true;
};

// 从后端获取角色数据
const fetchRoles = async () => {
  try {
    const response: any = await api.get('/api/roles');
    let roleList: any[] = [];
    if (Array.isArray(response)) {
      roleList = response;
    } else if (response && Array.isArray(response.list)) {
      roleList = response.list;
    } else if (response && Array.isArray(response.items)) {
      roleList = response.items;
    } else if (response && Array.isArray(response.data)) {
      roleList = response.data;
    }
    roles.value = roleList.map((role: any) => ({
      role_id: role.role_id,
      role_name: role.role_name,
      description: role.description || '',
      permissions: [],
      create_time: role.created_at || role.create_time || '',
    }));
  } catch (error) {
    console.error('获取角色列表失败:', error);
    ElMessage.error('获取角色列表失败');
  }
};

// 保存角色
const handleSave = async () => {
  try {
    const roleData = {
      role_name: editingRole.value.role_name,
      description: editingRole.value.description || null,
    };

    if (editingRole.value.role_id) {
      // 更新现有角色
      await api.put(`/api/roles/${editingRole.value.role_id}`, roleData);
      ElMessage.success('更新成功');
    } else {
      // 添加新角色
      await api.post('/api/roles', roleData);
      ElMessage.success('添加成功');
    }

    // 重新获取角色列表
    await fetchRoles();
    showAddRoleDialog.value = false;
  } catch (error) {
    console.error('保存角色失败:', error);
    ElMessage.error('保存角色失败');
  }
};

// 删除角色
const handleDelete = async (role: Role) => {
  ElMessageBox.confirm('确定要删除该角色吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/roles/${role.role_id}`);
        ElMessage.success('删除成功');
        // 重新获取角色列表
        await fetchRoles();
      } catch (error) {
        console.error('删除角色失败:', error);
        ElMessage.error('删除角色失败');
      }
    })
    .catch(() => {
      // 取消删除
    });
};

// 组件加载时获取角色数据
onMounted(() => {
  fetchRoles();
});

// 分页处理
const handleSizeChange = (size: number) => {
  pagination.pageSize = size;
};

const handleCurrentChange = (current: number) => {
  pagination.currentPage = current;
};
</script>

<style scoped>
.role-manage {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.permission-setting {
  display: flex;
  flex-wrap: wrap;
  gap: 15px;
}

.permission-setting .el-checkbox {
  margin-right: 20px;
  margin-bottom: 10px;
}
</style>


