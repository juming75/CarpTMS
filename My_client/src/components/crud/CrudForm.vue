<template>
  <el-form :model="form" :rules="rules" label-position="right" label-width="100px" size="small">
    <slot></slot>
  </el-form>
</template>

<script setup lang="ts">
const props = defineProps<{
  // 表单数据
  form: any;
  // 表单规则
  rules?: any;
}>();
void props; // 保留变量以符合 TypeScript
</script>

<style scoped>
/* 可以添加表单的样式 */
</style>


