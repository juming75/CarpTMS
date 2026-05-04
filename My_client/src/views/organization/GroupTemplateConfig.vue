<template>
  <div class="group-template-config">
    <div class="page-header">
      <h2>组织模板配置</h2>
      <div class="header-actions">
        <el-button @click="handleRefresh">
          <el-icon><Refresh /></el-icon> 刷新数据
        </el-button>
      </div>
    </div>

    <el-row :gutter="20">
      <!-- 左侧：模板列表 -->
      <el-col :span="8">
        <el-card class="template-list-card">
          <template #header>
            <div class="card-header">
              <span>模板列表</span>
              <el-button type="primary" size="small" @click="handleAddTemplate">
                <el-icon><Plus /></el-icon> 新增模板
              </el-button>
            </div>
          </template>

          <div class="template-list">
            <div
              v-for="template in templates"
              :key="template.template_id"
              class="template-item"
              :class="{ active: selectedTemplate?.template_id === template.template_id }"
              @click="handleSelectTemplate(template)"
            >
              <div class="template-info">
                <div class="template-name">{{ template.template_name }}</div>
                <div class="template-desc">{{ template.description }}</div>
              </div>
              <div class="template-actions">
                <el-button size="small" @click.stop="handleEditTemplate(template)">
                  <el-icon><Edit /></el-icon>
                </el-button>
                <el-button size="small" type="danger" @click.stop="handleDeleteTemplate(template)">
                  <el-icon><Delete /></el-icon>
                </el-button>
              </div>
            </div>
            <el-empty v-if="templates.length === 0" description="暂无模板" />
          </div>
        </el-card>
      </el-col>

      <!-- 右侧：模板配置 -->
      <el-col :span="16">
        <el-card v-if="selectedTemplate" class="template-config-card">
          <template #header>
            <div class="card-header">
              <span>模板配置：{{ selectedTemplate.template_name }}</span>
              <el-button type="primary" size="small" @click="handleSaveConfig">
                <el-icon><Check /></el-icon> 保存配置
              </el-button>
            </div>
          </template>

          <!-- 基础配置 -->
          <el-card class="config-section">
            <template #header>
              <span>基础配置</span>
            </template>
            <el-form :model="baseConfig" label-width="120px">
              <el-form-item label="模板名称">
                <el-input v-model="baseConfig.template_name" placeholder="请输入模板名称" />
              </el-form-item>
              <el-form-item label="模板描述">
                <el-input v-model="baseConfig.description" type="textarea" :rows="3" placeholder="请输入模板描述" />
              </el-form-item>
              <el-form-item label="行业分类">
                <el-select v-model="baseConfig.industry" placeholder="请选择行业分类">
                  <el-option label="货运行业" value="freight" />
                  <el-option label="客运行业" value="passenger" />
                  <el-option label="物流行业" value="logistics" />
                  <el-option label="其他行业" value="other" />
                </el-select>
              </el-form-item>
            </el-form>
          </el-card>

          <!-- 车队层级配置 -->
          <el-card class="config-section">
            <template #header>
              <div class="card-header">
                <span>车队层级配置</span>
                <el-button size="small" @click="handleAddLevel">
                  <el-icon><Plus /></el-icon> 添加层级
                </el-button>
              </div>
            </template>
            <el-table :data="teamLevels" style="width: 100%">
              <el-table-column prop="level" label="层级" width="80" />
              <el-table-column prop="name" label="名称" />
              <el-table-column prop="description" label="描述" />
              <el-table-column label="操作" width="100">
                <template #default="scope">
                  <el-button size="small" type="danger" @click="handleDeleteLevel(scope.$index)">
                    <el-icon><Delete /></el-icon>
                  </el-button>
                </template>
              </el-table-column>
            </el-table>
          </el-card>

          <!-- 角色权限配置 -->
          <el-card class="config-section">
            <template #header>
              <div class="card-header">
                <span>角色权限配置</span>
                <el-button size="small" @click="handleAddRole">
                  <el-icon><Plus /></el-icon> 添加角色
                </el-button>
              </div>
            </template>
            <el-table :data="rolePermissions" style="width: 100%">
              <el-table-column prop="role_name" label="角色名称" />
              <el-table-column prop="permissions" label="权限" />
              <el-table-column label="操作" width="100">
                <template #default="scope">
                  <el-button size="small" type="danger" @click="handleDeleteRole(scope.$index)">
                    <el-icon><Delete /></el-icon>
                  </el-button>
                </template>
              </el-table-column>
            </el-table>
          </el-card>

          <!-- 应用到组织 -->
          <el-card class="config-section">
            <template #header>
              <span>应用到组织</span>
            </template>
            <el-form label-width="120px">
              <el-form-item label="选择组织">
                <el-select v-model="selectedOrgs" multiple placeholder="请选择要应用模板的组织" style="width: 100%">
                  <el-option
                    v-for="org in organizations"
                    :key="org.unit_id"
                    :label="org.name"
                    :value="org.unit_id"
                  />
                </el-select>
              </el-form-item>
              <el-form-item>
                <el-button type="success" @click="handleApplyToOrgs">
                  <el-icon><Upload /></el-icon> 应用到选中组织
                </el-button>
              </el-form-item>
            </el-form>
          </el-card>
        </el-card>
        <el-empty v-else description="请选择一个模板进行配置" />
      </el-col>
    </el-row>

    <!-- 新增/编辑模板对话框 -->
    <el-dialog v-model="templateDialogVisible" :title="isEditing ? '编辑模板' : '新增模板'" width="500px">
      <el-form ref="templateFormRef" :model="templateForm" label-width="100px">
        <el-form-item label="模板名称" required>
          <el-input v-model="templateForm.template_name" placeholder="请输入模板名称" />
        </el-form-item>
        <el-form-item label="模板描述">
          <el-input v-model="templateForm.description" type="textarea" :rows="3" placeholder="请输入模板描述" />
        </el-form-item>
        <el-form-item label="行业分类">
          <el-select v-model="templateForm.industry" placeholder="请选择行业分类" style="width: 100%">
            <el-option label="货运行业" value="freight" />
            <el-option label="客运行业" value="passenger" />
            <el-option label="物流行业" value="logistics" />
            <el-option label="其他行业" value="other" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="templateDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleTemplateSubmit">保存</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, reactive, onMounted } from 'vue';
import { ElMessage, ElMessageBox, FormInstance } from 'element-plus';
import { Refresh, Plus, Edit, Delete, Check, Upload } from '@element-plus/icons-vue';
import api from '@/api';

// 模板类型定义
interface GroupTemplate {
  template_id: number;
  template_name: string;
  description: string;
  industry: string;
  create_time: string;
  update_time?: string;
}

// 组织类型定义
interface Organization {
  unit_id: number;
  name: string;
}

// 状态管理
const loading = ref(false);
const templates = ref<GroupTemplate[]>([]);
const selectedTemplate = ref<GroupTemplate | null>(null);
const organizations = ref<Organization[]>([]);
const selectedOrgs = ref<number[]>([]);

// 模板配置
const baseConfig = reactive({
  template_name: '',
  description: '',
  industry: '',
});

// 车队层级配置
const teamLevels = ref([
  { level: 1, name: '总公司', description: '最高层级' },
  { level: 2, name: '分公司', description: '分公司层级' },
  { level: 3, name: '车队', description: '车队层级' },
]);

// 角色权限配置
const rolePermissions = ref([
  { role_name: '管理员', permissions: '全部权限' },
  { role_name: '调度员', permissions: '调度管理' },
  { role_name: '驾驶员', permissions: '查看权限' },
]);

// 对话框控制
const templateDialogVisible = ref(false);
const isEditing = ref(false);
const templateFormRef = ref<FormInstance>();
const templateForm = reactive({
  template_name: '',
  description: '',
  industry: 'freight',
});

// 加载模板列表
const loadTemplates = async () => {
  loading.value = true;
  try {
    const response = await api.get('/api/group-templates');
    if (response && response.list) {
      templates.value = response.list;
    }
  } catch (error) {
    console.error('获取模板列表失败:', error);
    ElMessage.error('获取模板列表失败');
  } finally {
    loading.value = false;
  }
};

// 加载组织列表
const loadOrganizations = async () => {
  try {
    const response: any = await api.get('/api/organizations', { params: { page: 1, page_size: 100 } });
    // 处理响应格式：{ code: 200, data: { list: [...] } }
    if (response && response.data) {
      const data = response.data.data || response.data;
      if (data && data.list) {
        organizations.value = data.list.map((org: any) => ({
          unit_id: org.unit_id,
          name: org.name,
        }));
      } else if (Array.isArray(data)) {
        organizations.value = data.map((org: any) => ({
          unit_id: org.unit_id,
          name: org.name,
        }));
      }
    }
  } catch (error) {
    console.error('获取组织列表失败:', error);
    ElMessage.error('获取组织列表失败');
  }
};

// 选择模板
const handleSelectTemplate = (template: GroupTemplate) => {
  selectedTemplate.value = template;
  baseConfig.template_name = template.template_name;
  baseConfig.description = template.description;
  baseConfig.industry = template.industry;
};

// 新增模板
const handleAddTemplate = () => {
  isEditing.value = false;
  templateForm.template_name = '';
  templateForm.description = '';
  templateForm.industry = 'freight';
  templateDialogVisible.value = true;
};

// 编辑模板
const handleEditTemplate = (template: GroupTemplate) => {
  isEditing.value = true;
  templateForm.template_name = template.template_name;
  templateForm.description = template.description;
  templateForm.industry = template.industry;
  templateDialogVisible.value = true;
};

// 删除模板
const handleDeleteTemplate = async (template: GroupTemplate) => {
  ElMessageBox.confirm('确定要删除该模板吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/group-templates/${template.template_id}`);
        ElMessage.success('删除成功');
        loadTemplates();
      } catch (error) {
        console.error('删除模板失败:', error);
        ElMessage.error('删除模板失败');
      }
    })
    .catch(() => {});
};

// 保存模板
const handleTemplateSubmit = async () => {
  try {
    const templateData = {
      template_name: templateForm.template_name,
      description: templateForm.description,
      industry: templateForm.industry,
    };

    if (isEditing.value && selectedTemplate.value) {
      await api.put(`/api/group-templates/${selectedTemplate.value.template_id}`, templateData);
      ElMessage.success('更新成功');
    } else {
      await api.post('/api/group-templates', templateData);
      ElMessage.success('添加成功');
    }

    templateDialogVisible.value = false;
    loadTemplates();
  } catch (error) {
    console.error('保存模板失败:', error);
    ElMessage.error('保存模板失败');
  }
};

// 保存配置
const handleSaveConfig = async () => {
  if (!selectedTemplate.value) {
    ElMessage.warning('请先选择一个模板');
    return;
  }

  try {
    const configData = {
      template_id: selectedTemplate.value.template_id,
      base_config: baseConfig,
      team_levels: teamLevels.value,
      role_permissions: rolePermissions.value,
    };

    await api.put(`/api/group-templates/${selectedTemplate.value.template_id}/config`, configData);
    ElMessage.success('配置保存成功');
  } catch (error) {
    console.error('保存配置失败:', error);
    ElMessage.error('保存配置失败');
  }
};

// 添加层级
const handleAddLevel = () => {
  const newLevel = {
    level: teamLevels.value.length + 1,
    name: '',
    description: '',
  };
  teamLevels.value.push(newLevel);
};

// 删除层级
const handleDeleteLevel = (index: number) => {
  teamLevels.value.splice(index, 1);
};

// 添加角色
const handleAddRole = () => {
  const newRole = {
    role_name: '',
    permissions: '',
  };
  rolePermissions.value.push(newRole);
};

// 删除角色
const handleDeleteRole = (index: number) => {
  rolePermissions.value.splice(index, 1);
};

// 应用到组织
const handleApplyToOrgs = async () => {
  if (!selectedTemplate.value) {
    ElMessage.warning('请先选择一个模板');
    return;
  }

  if (selectedOrgs.value.length === 0) {
    ElMessage.warning('请选择要应用的组织');
    return;
  }

  try {
    await api.post(`/api/group-templates/${selectedTemplate.value.template_id}/apply`, {
      organization_ids: selectedOrgs.value,
    });
    ElMessage.success('应用成功');
  } catch (error) {
    console.error('应用到组织失败:', error);
    ElMessage.error('应用到组织失败');
  }
};

// 刷新数据
const handleRefresh = () => {
  loadTemplates();
};

// 组件挂载时初始化数据
onMounted(async () => {
  await loadTemplates();
  await loadOrganizations();
});
</script>

<style scoped>
.group-template-config {
  padding: 20px;
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

.template-list-card,
.template-config-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header span {
  font-weight: bold;
}

.template-list {
  max-height: 600px;
  overflow-y: auto;
}

.template-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  margin-bottom: 10px;
  cursor: pointer;
  transition: all 0.3s;
}

.template-item:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  border-color: #409eff;
}

.template-item.active {
  border-color: #409eff;
  box-shadow: 0 0 0 2px #409eff;
  background-color: #ecf5ff;
}

.template-info {
  flex: 1;
}

.template-name {
  font-weight: 600;
  color: #303133;
  margin-bottom: 4px;
}

.template-desc {
  font-size: 12px;
  color: #909399;
}

.template-actions {
  display: flex;
  gap: 5px;
}

.config-section {
  margin-bottom: 20px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
