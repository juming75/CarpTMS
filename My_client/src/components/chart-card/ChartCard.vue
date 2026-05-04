<template>
  <el-card shadow="hover" class="chart-card" :body-style="{ padding: '0' }">
    <template #header v-if="title">
      <div class="card-header">
        <span class="card-title">{{ title }}</span>
        <div class="card-actions" v-if="$slots.actions">
          <slot name="actions"></slot>
        </div>
      </div>
    </template>
    <div class="chart-container" ref="chartContainer">
      <div class="chart-content">
        <slot></slot>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface Props {
  title?: string;
  height?: string;
}

defineProps<Props>();

const chartContainer = ref<HTMLElement>();

defineExpose({
  chartContainer,
});
</script>

<style scoped>
.chart-card {
  display: flex;
  flex-direction: column;
  border: none;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: bold;
  color: #303133;
  padding: 0;
}

.card-title {
  font-size: 16px;
  font-weight: bold;
}

.card-actions {
  display: flex;
  gap: 10px;
}

.chart-container {
  flex: 1;
  overflow: hidden;
  min-height: v-bind('height');
  position: relative;
}

.chart-content {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: #fafafa;
}
</style>


