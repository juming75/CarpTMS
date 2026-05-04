<template>
  <div class="settings-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>基础设置</span>
        </div>
      </template>

      <!-- 标签页 -->
      <el-tabs v-model="activeTab" @tab-click="handleTabClick" class="settings-tabs">
        <!-- 基础要素 -->
        <el-tab-pane label="基础要素" name="general">
          <SystemSettings />
          <CommunicationSettings />
          <OpenapiPlatform />
        </el-tab-pane>

        <!-- 服务监测 -->
        <el-tab-pane label="服务监测" name="monitor">
          <ServiceMonitor />
        </el-tab-pane>

        <!-- 组织设置 -->
        <el-tab-pane label="组织设置" name="organization">
          <OrganizationSettings />
        </el-tab-pane>

        <!-- 组织模板 -->
        <el-tab-pane label="组织模板" name="group-template">
          <GroupTemplateConfig />
        </el-tab-pane>

        <!-- 自动化运维 -->
        <el-tab-pane v-if="isRemoteOpsEnabled" label="自动化运维" name="ansible-ops">
          <router-view />
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import SystemSettings from './SystemSettings.vue';
import CommunicationSettings from './CommunicationSettings.vue';
import OpenapiPlatform from './OpenapiPlatform.vue';
import ServiceMonitor from './ServiceMonitor.vue';
import OrganizationSettings from '@/views/organization/OrganizationSettings.vue';
import GroupTemplateConfig from '@/views/organization/GroupTemplateConfig.vue';
import { isRemoteOpsEnabled } from '@/utils/env';

const route = useRoute();
const router = useRouter();

// 当前激活的标签页
const activeTab = ref('general');

const tabToRouteMap: Record<string, string> = {
  general: 'SystemSettings',
  monitor: 'ServiceMonitor',
  organization: 'OrgSettings',
  'group-template': 'GroupTemplateConfig',
  'ansible-ops': 'AnsibleOps',
};

const routeToTabMap: Record<string, string> = {
  SystemSettings: 'general',
  ServiceMonitor: 'monitor',
  OrgSettings: 'organization',
  GroupTemplateConfig: 'group-template',
  AnsibleOps: 'ansible-ops',
};

function handleTabClick(tab: any) {
  const tabName = tab.props.name;
  if (tabName && tabToRouteMap[tabName]) {
    router.push({ name: tabToRouteMap[tabName] });
  }
}

watch(
  () => route.name,
  (newName) => {
    if (newName && typeof newName === 'string' && routeToTabMap[newName]) {
      activeTab.value = routeToTabMap[newName];
    }
  },
  { immediate: true }
);

onMounted(() => {
  if (route.name && typeof route.name === 'string' && routeToTabMap[route.name]) {
    activeTab.value = routeToTabMap[route.name];
  }
});
</script>

<style scoped>
.settings-container {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.settings-tabs {
  margin-top: 10px;
}

:deep(.el-tabs__header) {
  margin-bottom: 20px;
}

:deep(.el-tabs__content) {
  padding-top: 10px;
}
</style>
