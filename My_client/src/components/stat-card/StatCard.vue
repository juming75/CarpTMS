<template>
  <el-card :class="['stat-card', { 'alarm-card': isAlarm }]" shadow="hover" :body-style="{ padding: '20px' }">
    <div class="stat-content">
      <div class="stat-icon" v-if="icon">
        <el-icon :size="32"><component :is="icon" /></el-icon>
      </div>
      <div class="stat-info">
        <div class="stat-label">{{ label }}</div>
        <div class="stat-value">{{ formattedValue }}</div>
        <div class="stat-unit" v-if="unit">{{ unit }}</div>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue';

// 定义 props
interface Props {
  label: string; // 标签
  value: number | string; // 值
  unit?: string; // 单位
  icon?: unknown; // 图标组件
  isAlarm?: boolean; // 是否为报警样式
  formatFn?: (value: number | string) => string; // 格式化函数
}

const props = withDefaults(defineProps<Props>(), {
  unit: '',
  isAlarm: false,
});

// 格式化数值
const formattedValue = computed(() => {
  if (props.formatFn) {
    return props.formatFn(props.value);
  }
  return props.value;
});
</script>

<style scoped>
.stat-card {
  transition: all 0.3s ease;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stat-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.12);
}

.alarm-card {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 20px;
  width: 100%;
}

.stat-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 50px;
  height: 50px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 50%;
}

.stat-info {
  flex: 1;
  text-align: center;
}

.stat-label {
  font-size: 14px;
  opacity: 0.9;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 28px;
  font-weight: bold;
  line-height: 1.2;
}

.stat-unit {
  font-size: 14px;
  opacity: 0.8;
  margin-top: 4px;
}
</style>


