<template>

  <div class="organization-settings">

    <div class="page-header">

      <h2>组织个性化设置</h2>

      <div class="header-actions">

        <el-button @click="handleRefresh">

          <el-icon><Refresh /></el-icon> 刷新数据

        </el-button>

      </div>

    </div>



    <el-card class="content-card">

      <!-- 组织选择 -->

      <div class="org-selector">

        <el-select

          v-model="selectedOrganization"

          placeholder="选择组织"

          style="width: 300px"

          @change="handleOrganizationChange"

        >

          <el-option v-for="org in organizations" :key="org.unit_id" :label="org.name" :value="org.unit_id" />

        </el-select>

      </div>



      <!-- 品牌设置 -->

      <el-card class="settings-card">

        <template #header>

          <div class="card-header">

            <span>品牌设置</span>

            <el-button type="primary" size="small" @click="saveBrandSettings"> 保存设置 </el-button>

          </div>

        </template>



        <el-form ref="brandFormRef" :model="brandSettings" label-width="100px">

          <el-form-item label="公司名称">

            <el-input v-model="brandSettings.company_name" placeholder="请输入公司名称" />

          </el-form-item>

          <el-form-item label="副标题">

            <el-input v-model="brandSettings.subtitle" placeholder="请输入副标题" />

          </el-form-item>

          <el-form-item label="登录地址">

            <el-input v-model="brandSettings.login_url" placeholder="请输入登录地址" />

          </el-form-item>

          <el-form-item label="Logo">

            <el-input v-model="brandSettings.logo" placeholder="请输入Logo URL" />

          </el-form-item>

          <el-form-item label="Favicon">

            <el-input v-model="brandSettings.favicon" placeholder="请输入Favicon URL" />

          </el-form-item>

        </el-form>

      </el-card>



      <!-- 主题设置 -->

      <el-card class="settings-card">

        <template #header>

          <div class="card-header">

            <span>主题设置</span>

            <el-button type="primary" size="small" @click="saveThemeSettings"> 保存设置 </el-button>

          </div>

        </template>



        <el-form ref="themeFormRef" :model="themeSettings" label-width="100px">

          <el-form-item label="主色调">

            <el-color-picker v-model="themeSettings.primary_color" />

          </el-form-item>

          <el-form-item label="次要色调">

            <el-color-picker v-model="themeSettings.secondary_color" />

          </el-form-item>

          <el-form-item label="字体">

            <el-input v-model="themeSettings.font_family" placeholder="请输入字体" />

          </el-form-item>

          <el-form-item label="布局">

            <el-select v-model="themeSettings.layout" placeholder="请选择布局">

              <el-option label="默认" value="default" />

              <el-option label="紧凑" value="compact" />

              <el-option label="舒适" value="comfortable" />

            </el-select>

          </el-form-item>

        </el-form>

      </el-card>

    </el-card>

  </div>

</template>



<script setup lang="ts">

import { ref, reactive, onMounted } from 'vue';

import { ElMessage, FormInstance } from 'element-plus';

import { Refresh } from '@element-plus/icons-vue';

import api from '@/api';



// 组织类型定义

interface Organization {

  unit_id: string;

  name: string;

  [key: string]: unknown;

}



// 状态管理

const loading = ref(false);

const selectedOrganization = ref('');

const organizations = ref<Organization[]>([]);

const brandFormRef = ref<FormInstance>();

const themeFormRef = ref<FormInstance>();



// 品牌设置

const brandSettings = reactive({

  company_name: 'CarpTMS',

  subtitle: '智慧运输管理系统',

  login_url: '/login',

  logo: '',

  favicon: '',

});



// 主题设置

const themeSettings = reactive({

  primary_color: '#409eff',

  secondary_color: '#67C23A',

  font_family: 'Arial, sans-serif',

  layout: 'default',

});



// 加载组织列表

const loadOrganizations = async () => {

  try {

    const response = await api.get('/api/organizations', { params: { page: 1, page_size: 100 } }) as any;

    organizations.value = response.list || [];

  } catch (error) {

    console.error('获取组织列表失败:', error);

    ElMessage.error('获取组织列表失败');

  }

};



// 加载组织设置

const loadOrganizationSettings = async (organizationId: string) => {

  loading.value = true;

  try {

    // 加载品牌设置

    const brandResponse = await api.get(`/api/organizations/${organizationId}/settings/brand`) as any;

    if (brandResponse) {

      Object.assign(brandSettings, brandResponse);

    }



    // 加载主题设置

    const themeResponse = await api.get(`/api/organizations/${organizationId}/settings/theme`) as any;

    if (themeResponse) {

      Object.assign(themeSettings, themeResponse);

    }

  } catch (error) {

    console.error('获取组织设置失败:', error);

    ElMessage.error('获取组织设置失败');

  } finally {

    loading.value = false;

  }

};



// 保存品牌设置

const saveBrandSettings = async () => {

  try {

    await api.put(`/api/organizations/${selectedOrganization.value}/settings/brand`, brandSettings) as any;

    ElMessage.success('品牌设置保存成功');

  } catch (error) {

    console.error('保存品牌设置失败:', error);

    ElMessage.error('保存品牌设置失败');

  }

};



// 保存主题设置

const saveThemeSettings = async () => {

  try {

    await api.put(`/api/organizations/${selectedOrganization.value}/settings/theme`, themeSettings) as any;

    ElMessage.success('主题设置保存成功');

  } catch (error) {

    console.error('保存主题设置失败:', error);

    ElMessage.error('保存主题设置失败');

  }

};



// 组织变更处理

const handleOrganizationChange = (organizationId: string) => {

  loadOrganizationSettings(organizationId);

};



// 刷新数据

const handleRefresh = () => {

  loadOrganizationSettings(selectedOrganization.value);

};



// 组件挂载时初始化数据

onMounted(async () => {

  await loadOrganizations();

  if (organizations.value.length > 0) {

    selectedOrganization.value = String(organizations.value[0].unit_id);

    loadOrganizationSettings(selectedOrganization.value);

  }

});

</script>



<style scoped>

.organization-settings {

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



.org-selector {

  margin-bottom: 20px;

}



.settings-card {

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

</style>





