<template>
  <div class="user-manage">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>用户管理</span>
          <el-button type="primary" size="small" @click="showAddUserDialog = true">
            <el-icon><Plus /></el-icon> 添加用户
          </el-button>
        </div>
      </template>

      <!-- 搜索栏 -->
      <div class="search-bar">
        <el-input
          v-model="searchForm.keyword"
          placeholder="输入用户名或手机号搜索"
          style="width: 300px; margin-right: 10px"
        >
          <template #append>
            <el-button @click="handleSearch"
              ><el-icon><Search /></el-icon
            ></el-button>
          </template>
        </el-input>
      </div>

      <!-- 用户列表 -->
      <el-table :data="filteredUsers" style="margin-top: 20px">
        <el-table-column prop="id" label="用户 ID" width="80" />
        <el-table-column prop="username" label="用户名" />
        <el-table-column prop="full_name" label="真实姓名" />
        <el-table-column prop="phone_number" label="手机号" />
        <el-table-column prop="email" label="邮箱" />
        <el-table-column prop="user_group_name" label="角色" />
        <el-table-column prop="department_name" label="部门" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="scope">
            <el-tag :type="scope.row.status === 1 ? 'success' : 'danger'">
              {{ scope.row.status === 1 ? '启用' : '禁用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="create_time" label="创建时间" width="180" />
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="scope">
            <el-button size="small" @click="handleEdit(scope.row)">
              <el-icon><Edit /></el-icon> 编辑
            </el-button>
            <el-button size="small" type="danger" @click="handleDelete(scope.row)">
              <el-icon><Delete /></el-icon> 删除
            </el-button>
            <el-button size="small" @click="toggleStatus(scope.row)">
              <el-icon><SwitchButton /></el-icon> {{ scope.row.status === 1 ? '禁用' : '启用' }}
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
          :total="filteredUsers.length"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- 添加/编辑用户对话框 -->
    <el-dialog v-model="showAddUserDialog" :title="editingUser ? '编辑用户' : '添加用户'" width="600px">
      <el-form ref="userFormRef" :model="editingUser" label-width="120px">
        <el-form-item label="用户名" required>
          <el-input v-model="editingUser.username" placeholder="请输入用户名" />
        </el-form-item>
        <el-form-item label="真实姓名" required>
          <el-input v-model="editingUser.full_name" placeholder="请输入真实姓名" />
        </el-form-item>
        <el-form-item label="密码" :required="!editingUser.id">
          <el-input v-model="editingUser.password" type="password" placeholder="请输入密码" />
        </el-form-item>
        <el-form-item label="手机号">
          <el-input v-model="editingUser.phone_number" placeholder="请输入手机号" />
        </el-form-item>
        <el-form-item label="邮箱">
          <el-input v-model="editingUser.email" placeholder="请输入邮箱" />
        </el-form-item>
        <el-form-item label="角色" required>
          <el-select v-model="editingUser.user_group_id" placeholder="请选择角色">
            <el-option v-for="role in roles" :key="role.role_id" :label="role.role_name" :value="role.role_id" />
          </el-select>
        </el-form-item>
        <el-form-item label="部门">
          <el-select v-model="editingUser.department_id" placeholder="请选择部门" clearable>
            <el-option
              v-for="dept in departments"
              :key="dept.department_id"
              :label="dept.department_name"
              :value="dept.department_id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="状态">
          <el-switch v-model="editingUser.status" :active-value="1" :inactive-value="0" />
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddUserDialog = false">取消</el-button>
          <el-button type="primary" @click="handleSave">保存</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Search, Plus, Delete, Edit } from '@element-plus/icons-vue';
import api from '@/api';
import type { FormInstance } from 'element-plus';

// 用户数据
interface UserItem {
  id: number;
  username: string;
  full_name: string;
  phone_number: string;
  email: string;
  user_group_id: number;
  department_id: number | null;
  status: number;
  create_time: string;
  password?: string;
}

const users = ref<UserItem[]>([]);

// 角色数据
interface RoleItem {
  role_id: number;
  role_name: string;
}

const roles = ref<RoleItem[]>([]);

// 部门数据
interface DepartmentItem {
  department_id: number;
  department_name: string;
}

const departments = ref<DepartmentItem[]>([]);

// 从后端获取角色数据
const fetchRoles = async () => {
  try {
    const response: any = await api.get('/api/roles');
    if (response) {
      roles.value = (response || []).map((role: { role_id: number; role_name: string }) => ({
        role_id: role.role_id,
        role_name: role.role_name,
      })) || [];
    }
  } catch (error) {
    console.error('获取角色列表失败:', error);
  }
};

// 从后端获取部门数据
const fetchDepartments = async () => {
  try {
    const response = await api.get('/api/departments') as any;
    if (response && response.list) {
      departments.value = response.list.map((dept: any) => ({
        department_id: dept.department_id,
        department_name: dept.department_name,
      }));
    }
  } catch (error) {
    console.error('获取部门列表失败:', error);
  }
};

// 搜索表单
const searchForm = reactive({
  keyword: '',
});

// 分页配置
const pagination = reactive({
  currentPage: 1,
  pageSize: 10,
});

// 添加/编辑对话框
const showAddUserDialog = ref(false);
const userFormRef = ref<FormInstance>();
const editingUser = ref<UserItem>({
  id: 0,
  username: '',
  full_name: '',
  password: '',
  phone_number: '',
  email: '',
  user_group_id: 0,
  department_id: null,
  status: 1,
  create_time: '',
});

// 筛选后的用户列表
const filteredUsers = computed(() => {
  if (!searchForm.keyword) {
    return users.value;
  }
  const keyword = searchForm.keyword.toLowerCase();
  return users.value.filter(
    (user) =>
      user.username.toLowerCase().includes(keyword) ||
      user.full_name.toLowerCase().includes(keyword) ||
      (user.phone_number && user.phone_number.includes(keyword))
  );
});

// 搜索
const handleSearch = () => {
  pagination.currentPage = 1;
};

// 编辑用户
const handleEdit = (user: UserItem) => {
  editingUser.value = {
    ...user,
    password: '', // 编辑时不显示密码
  };
  showAddUserDialog.value = true;
};

// 从后端获取用户数据
const fetchUsers = async () => {
  try {
    const response = (await api.get('/api/users')) as any;
    if (response && response.list) {
      users.value = response.list;
    }
  } catch (error) {
    console.error('获取用户列表失败:', error);
    ElMessage.error('获取用户列表失败');
  }
};

// 保存用户
const handleSave = async () => {
  try {
    const userData: Record<string, unknown> = {
      username: editingUser.value.username,
      full_name: editingUser.value.full_name,
      phone_number: editingUser.value.phone_number,
      email: editingUser.value.email,
      user_group_id: editingUser.value.user_group_id,
      status: editingUser.value.status,
    };

    // 如果选择了部门，添加部门 ID
    if (editingUser.value.department_id) {
      userData.department_id = editingUser.value.department_id;
    }

    // 如果是新用户或填写了密码，则包含密码
    if (!editingUser.value.id || editingUser.value.password) {
      userData.password = editingUser.value.password;
    }

    if (editingUser.value.id) {
      // 更新现有用户
      await api.put(`/api/users/${editingUser.value.id}`, userData) as unknown;
      ElMessage.success('更新成功');
    } else {
      // 添加新用户
      await api.post('/api/users', userData) as unknown;
      ElMessage.success('添加成功');
    }

    // 重新获取用户列表
    await fetchUsers();
    showAddUserDialog.value = false;
  } catch (error) {
    console.error('保存用户失败:', error);
    ElMessage.error('保存用户失败');
  }
};

// 删除用户
const handleDelete = async (user: UserItem) => {
  ElMessageBox.confirm('确定要删除该用户吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/users/${user.id}`) as unknown;
        ElMessage.success('删除成功');
        // 重新获取用户列表
        await fetchUsers();
      } catch (error) {
        console.error('删除用户失败:', error);
        ElMessage.error('删除用户失败');
      }
    })
    .catch(() => {
      // 取消删除
    });
};

// 切换用户状态
const toggleStatus = async (user: UserItem) => {
  try {
    const newStatus = user.status === 1 ? 0 : 1;
    await api.put(`/api/users/${user.id}`, { status: newStatus }) as unknown;
    user.status = newStatus;
    ElMessage.success(`用户已${newStatus === 1 ? '启用' : '禁用'}`);
  } catch (error) {
    console.error('切换用户状态失败:', error);
    ElMessage.error('切换用户状态失败');
  }
};

// 分页处理
const handleSizeChange = (size: number) => {
  pagination.pageSize = size;
};

const handleCurrentChange = (current: number) => {
  pagination.currentPage = current;
};

// 组件加载时获取数据
onMounted(async () => {
  await fetchUsers();
  await fetchRoles();
  await fetchDepartments();
});
</script>

<style scoped>
.user-manage {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.search-bar {
  margin-bottom: 20px;
  display: flex;
  align-items: center;
}
</style>


