<template>
  <div class="organization-unit-manage">
    <div class="page-header">
      <h2>组织单位管理</h2>
      <div class="header-actions">
        <el-button type="primary" @click="handleAdd">
          <el-icon><Plus /></el-icon> 添加组织单位
        </el-button>
        <el-button @click="handleRefresh">
          <el-icon><Refresh /></el-icon> 刷新数据
        </el-button>
      </div>
    </div>

    <el-card class="content-card">
      <!-- 搜索和筛选 -->
      <div class="search-filter">
        <el-input
          v-model="searchQuery"
          placeholder="搜索组织单位名称"
          clearable
          size="default"
          class="search-input"
          @keyup.enter="handleSearch"
        >
          <template #append>
            <el-button @click="handleSearch">
              <el-icon><Search /></el-icon>
            </el-button>
          </template>
        </el-input>

        <el-select v-model="filterType" placeholder="筛选组织类型" clearable size="default" @change="handleSearch">
          <el-option label="企业" value="enterprise" />
          <el-option label="政府机关" value="government" />
          <el-option label="社会团体" value="social" />
          <el-option label="其他" value="other" />
        </el-select>
      </div>

      <!-- 组织单位表格 -->
      <el-table v-loading="loading" :data="organizationUnits" style="width: 100%" stripe border>
        <el-table-column type="index" label="序列号" width="80" align="center" />
        <el-table-column prop="unit_id" label="ID" width="120" align="center" />
        <el-table-column prop="name" label="组织名称" min-width="150" />
        <el-table-column prop="type" label="组织类型" width="120" align="center">
          <template #default="scope">
            <el-tag
              :type="
                scope.row.type === 'enterprise'
                  ? 'success'
                  : scope.row.type === 'government'
                    ? 'primary'
                    : scope.row.type === 'social'
                      ? 'warning'
                      : 'info'
              "
            >
              {{
                scope.row.type === 'enterprise'
                  ? '企业'
                  : scope.row.type === 'government'
                    ? '政府机关'
                    : scope.row.type === 'social'
                      ? '社会团体'
                      : '其他'
              }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="parent_id" label="上级组织ID" width="120" align="center" />
        <el-table-column prop="description" label="描述" min-width="200" />
        <el-table-column prop="contact_person" label="联系人" width="120" />
        <el-table-column prop="contact_phone" label="联系电话" width="150" />
        <el-table-column prop="create_time" label="创建时间" width="180" align="center">
          <template #default="scope">
            {{ formatDate(scope.row.create_time) }}
          </template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="100" align="center">
          <template #default="scope">
            <el-switch
              v-model="scope.row.status"
              active-value="active"
              inactive-value="inactive"
              @change="handleStatusChange(scope.row)"
            />
          </template>
        </el-table-column>
        <el-table-column label="操作" width="250" align="center">
          <template #default="scope">
            <el-button type="primary" size="small" @click="handleEdit(scope.row)" :disabled="!hasPermission('edit')">
              编辑
            </el-button>
            <el-button type="info" size="small" @click="handleSettings(scope.row)" :disabled="!hasPermission('edit')">
              设置
            </el-button>
            <el-button type="danger" size="small" @click="handleDelete(scope.row)" :disabled="!hasPermission('delete')">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="pagination.currentPage"
          v-model:page-size="pagination.pageSize"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          :total="pagination.total"
          @size-change="handleSizeChange"
          @current-change="handleCurrentChange"
        />
      </div>
    </el-card>

    <!-- 添加/编辑组织单位对话框 -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑组织单位' : '添加组织单位'" width="500px">
      <el-form ref="formRef" :model="formData" :rules="formRules" label-width="100px">
        <el-form-item label="ID号" prop="unit_id">
          <el-input v-model="formData.unit_id" placeholder="请输入ID号（支持字母和数字）" />
        </el-form-item>
        <el-form-item label="组织名称" prop="name">
          <el-input v-model="formData.name" placeholder="请输入组织名称" />
        </el-form-item>
        <el-form-item label="组织类型" prop="type">
          <el-select v-model="formData.type" placeholder="请选择组织类型">
            <el-option label="企业" value="enterprise" />
            <el-option label="政府机关" value="government" />
            <el-option label="社会团体" value="social" />
            <el-option label="其他" value="other" />
          </el-select>
        </el-form-item>
        <el-form-item label="上级组织ID">
          <el-input v-model="formData.parent_id" placeholder="请输入上级组织ID" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input v-model="formData.description" placeholder="请输入组织描述" type="textarea" :rows="3" />
        </el-form-item>
        <el-form-item label="联系人">
          <el-input v-model="formData.contact_person" placeholder="请输入联系人" />
        </el-form-item>
        <el-form-item label="联系电话">
          <el-input v-model="formData.contact_phone" placeholder="请输入联系电话" />
        </el-form-item>
      </el-form>
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleSubmit">
            {{ isEdit ? '更新' : '保存' }}
          </el-button>
        </div>
      </template>
    </el-dialog>

    <!-- 组织设置对话框 -->
    <el-dialog v-model="settingsDialogVisible" title="组织单位设置" width="600px">
      <el-form ref="settingsFormRef" :model="settingsFormData" :rules="settingsFormRules" label-width="120px">
        <el-form-item label="个性化登录设置" prop="loginUrl">
          <el-input v-model="settingsFormData.loginUrl" placeholder="输入组织单位的独立网址" />
        </el-form-item>
        <el-form-item label="个性化登录页面副标题" prop="loginSubtitle">
          <el-input v-model="settingsFormData.loginSubtitle" placeholder="用户自定义" />
        </el-form-item>
        <el-form-item label="个性化登录页面公司名" prop="loginCompanyName">
          <el-input v-model="settingsFormData.loginCompanyName" placeholder="用户自定义，显示组织单位的公司名" />
        </el-form-item>
        <el-form-item label="首页标题" prop="homeTitle">
          <el-input v-model="settingsFormData.homeTitle" placeholder="用户自定义" />
        </el-form-item>
        <el-form-item label="行业分类" prop="industry">
          <el-select v-model="settingsFormData.industry" placeholder="请选择行业分类">
            <el-option label="货运行业" value="freight" />
            <el-option label="客运行业" value="passenger" />
            <el-option label="物流行业" value="logistics" />
            <el-option label="其他行业" value="other" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="settingsDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleSettingsSubmit">保存设置</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, reactive, onMounted } from 'vue';
import { ElMessage, ElMessageBox, FormInstance } from 'element-plus';
import { Plus, Refresh, Search } from '@element-plus/icons-vue';
import api from '@/api';

// 组织单位类型定义
interface OrganizationUnit {
  unit_id: string;
  name: string;
  type: string;
  parent_id: string;
  description: string;
  contact_person: string;
  contact_phone: string;
  create_time: string;
  status: string;
  [key: string]: unknown;
}

// API响应类型
interface ApiResponse<T> {
  items?: T[];
  total?: number;
  [key: string]: unknown;
}

// 状态管理
const loading = ref(false);
const searchQuery = ref('');
const filterType = ref('');
const dialogVisible = ref(false);
const settingsDialogVisible = ref(false);
const isEdit = ref(false);
const formRef = ref<FormInstance>();
const settingsFormRef = ref<FormInstance>();
const currentOrganizationUnit = ref<OrganizationUnit | null>(null);

// 分页配置
const pagination = reactive({
  currentPage: 1,
  pageSize: 20,
  total: 0,
});

// 组织单位数据
const organizationUnits = ref<OrganizationUnit[]>([]);

// 表单数据
const formData = reactive({
  unit_id: '',
  name: '',
  type: 'enterprise',
  parent_id: '',
  description: '',
  contact_person: '',
  contact_phone: '',
  create_time: '',
  status: 'active',
});

// 表单验证规则
const formRules = {
  unit_id: [
    { required: true, message: '请输入ID号', trigger: 'blur' },
    { pattern: /^[a-zA-Z0-9]+$/, message: 'ID号只能包含字母和数字', trigger: 'blur' },
  ],
  name: [
    { required: true, message: '请输入组织名称', trigger: 'blur' },
    { min: 2, max: 50, message: '组织名称长度在 2 到 50 个字符', trigger: 'blur' },
  ],
  type: [{ required: true, message: '请选择组织类型', trigger: 'change' }],
};

// 组织设置表单数据
const settingsFormData = reactive({
  loginUrl: '',
  loginSubtitle: '',
  loginCompanyName: '',
  homeTitle: '',
  industry: 'freight', // 默认货运行业
});

// 组织设置表单验证规则
const settingsFormRules = {
  loginUrl: [{ required: true, message: '请输入组织单位的独立网址', trigger: 'blur' }],
  loginSubtitle: [{ required: true, message: '请输入个性化登录页面副标题', trigger: 'blur' }],
  loginCompanyName: [{ required: true, message: '请输入个性化登录页面公司名', trigger: 'blur' }],
  homeTitle: [{ required: true, message: '请输入首页标题', trigger: 'blur' }],
  industry: [{ required: true, message: '请选择行业分类', trigger: 'change' }],
};

// 格式化日期
const formatDate = (dateString: string) => {
  const date = new Date(dateString);
  return date.toLocaleString('zh-CN');
};

// 权限检查
const hasPermission = (_permission: string) => {
  // 这里可以根据用户权限进行判断
  return true;
};

// 加载组织单位列表
const loadOrganizationUnits = async () => {
  loading.value = true;
  try {
    const params = {
      page: pagination.currentPage,
      page_size: pagination.pageSize,
      name: searchQuery.value,
      type: filterType.value,
    };
    const response = await api.get('/api/organizations', { params });
    organizationUnits.value = response.list || [];
    pagination.total = response.total || 0;
  } catch (error) {
    console.error('获取组织单位列表失败:', error);
    ElMessage.error('获取组织单位列表失败');
  } finally {
    loading.value = false;
  }
};

// 搜索和筛选
const handleSearch = () => {
  pagination.currentPage = 1;
  loadOrganizationUnits();
};

// 刷新数据
const handleRefresh = () => {
  loadOrganizationUnits();
};

// 添加组织单位
const handleAdd = () => {
  isEdit.value = false;
  // 重置表单
  Object.assign(formData, {
    unit_id: '',
    name: '',
    type: 'enterprise',
    parent_id: 0,
    description: '',
    contact_person: '',
    contact_phone: '',
    create_time: '',
    status: 'active',
  });
  dialogVisible.value = true;
};

// 编辑组织单位
const handleEdit = (row: OrganizationUnit) => {
  isEdit.value = true;
  // 填充表单数据
  Object.assign(formData, { ...row });
  dialogVisible.value = true;
};

// 删除组织单位
const handleDelete = async (row: OrganizationUnit) => {
  ElMessageBox.confirm(`确定要删除组织单位 "${row.name}" 吗？`, '删除确认', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/organizations/${row.unit_id}`);
        ElMessage.success('删除成功');
        loadOrganizationUnits();
      } catch (error) {
        console.error('删除组织单位失败:', error);
        ElMessage.error('删除组织单位失败');
      }
    })
    .catch(() => {
      // 取消删除
    });
};

// 状态变更
const handleStatusChange = async (row: OrganizationUnit) => {
  try {
    await api.put(`/api/organizations/${row.unit_id}/status`, { status: row.status });
    ElMessage.success(`组织单位 ${row.name} 状态已更新为 ${row.status === 'active' ? '激活' : '禁用'}`);
  } catch (error) {
    console.error('更新组织单位状态失败:', error);
    ElMessage.error('更新组织单位状态失败');
  }
};

// 表单提交
const handleSubmit = async () => {
  if (!formRef.value) return;

  await formRef.value.validate(async (valid) => {
    if (valid) {
      try {
        if (isEdit.value) {
          // 编辑逻辑
          await api.put(`/api/organizations/${formData.unit_id}`, formData);
          ElMessage.success('编辑成功');
        } else {
          // 添加逻辑
          await api.post('/api/organizations', formData);
          ElMessage.success('添加成功');
        }
        dialogVisible.value = false;
        loadOrganizationUnits();
      } catch (error) {
        console.error('保存组织单位失败:', error);
        ElMessage.error('保存组织单位失败');
      }
    }
  });
};

// 组织设置
const handleSettings = async (row: OrganizationUnit) => {
  currentOrganizationUnit.value = row;
  // 重置设置表单
  Object.assign(settingsFormData, {
    loginUrl: '',
    loginSubtitle: '',
    loginCompanyName: '',
    homeTitle: '',
    industry: 'freight', // 默认货运行业
  });
  // 打开设置对话框
  settingsFormData.unit_id = row.unit_id;
  settingsFormData.name = row.name;
  settingsDialogVisible.value = true;
  try {
    const resp = await api.get(`/api/organizations/${row.unit_id}/settings/unit_settings`);
    const saved = (resp as any).unit_settings || {};
    Object.assign(settingsFormData, saved);
  } catch { }
};

// 保存组织设置
const handleSettingsSubmit = async () => {
  if (!settingsFormRef.value) return;

  await settingsFormRef.value.validate(async (valid) => {
    if (valid) {
      try {
        // 这里可以调用 API 保存组织设置
        const orgId = settingsFormData.unit_id || selectedOrganization.value;
        await api.put(`/api/organizations/${orgId}/settings/unit_settings`, settingsFormData);
        ElMessage.success('组织设置保存成功');
        ElMessage.success('组织设置保存成功');
        settingsDialogVisible.value = false;
      } catch (error) {
        console.error('保存组织设置失败:', error);
        ElMessage.error('保存组织设置失败');
      }
    }
  });
};

// 分页事件处理
const handleSizeChange = (size: number) => {
  pagination.pageSize = size;
  loadOrganizationUnits();
};

const handleCurrentChange = (page: number) => {
  pagination.currentPage = page;
  loadOrganizationUnits();
};

// 组件挂载时初始化数据
onMounted(() => {
  loadOrganizationUnits();
});
</script>

<style scoped>
.organization-unit-manage {
  width: 100%;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: bold;
  color: #303133;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.content-card {
  margin-bottom: 20px;
}

.search-filter {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.search-input {
  width: 300px;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>


