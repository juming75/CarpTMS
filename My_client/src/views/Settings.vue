<template>
  <div class="settings">
    <router-view v-slot="{ Component }">
      <template v-if="!Component">
        <SystemSettings />
        <CommunicationSettings />
        <ServiceMonitor />
        <OpenapiPlatform />
      </template>
      <component :is="Component" v-else />
    </router-view>
  </div>
</template>

<script setup lang="ts">
import SystemSettings from '@/components/settings/SystemSettings.vue';
import CommunicationSettings from '@/components/settings/CommunicationSettings.vue';
import ServiceMonitor from '@/components/settings/ServiceMonitor.vue';
import OpenapiPlatform from '@/components/settings/OpenapiPlatform.vue';
</script>

<style scoped>
.settings {
  padding: 20px;
  background-color: #f5f7fa;
  min-height: 100vh;
}
</style>