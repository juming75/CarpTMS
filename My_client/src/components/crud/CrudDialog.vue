<template>
  <el-dialog v-model="visible" :title="title" :width="width">
    <slot></slot>
    <template #footer>
      <span class="dialog-footer">
        <el-button @click="$emit('cancel')">取消</el-button>
        <el-button type="primary" @click="$emit('confirm')" v-if="showConfirmButton">{{ confirmButtonText }}</el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
// @ts-nocheck
const props = defineProps<{
  // 对话框可见性
  visible: boolean;
  // 对话框标题
  title: string;
  // 对话框宽度
  width?: string;
  // 是否显示确认按钮
  showConfirmButton?: boolean;
  // 确认按钮文本
  confirmButtonText?: string;
}>();

// 定义事件
const emit = defineEmits<{
  (e: 'cancel'): void;
  (e: 'confirm'): void;
}>();
</script>

<style scoped>
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>



