<template>
  <div class="crud-table">
    <!-- 表格头部 -->
    <div class="table-header" v-if="showHeader">
      <span>{{ title }}</span>
      <el-button type="primary" @click="$emit('add')" v-if="showAddButton">
        <el-icon><Plus /></el-icon>
        {{ addButtonText }}
      </el-button>
    </div>

    <!-- 表格 -->
    <el-table :data="data" stripe size="small" v-loading="loading">
      <!-- 自定义列 -->
      <template v-for="column in columns" :key="column.prop">
        <el-table-column
          :prop="column.prop"
          :label="column.label"
          :width="column.width"
          :align="column.align"
        >
          <template #default="scope">
            <slot :name="`column-${column.prop}`" :row="scope.row">
              {{ scope.row[column.prop as keyof T] }}
            </slot>
          </template>
        </el-table-column>
      </template>

      <!-- 操作列 -->
      <el-table-column label="操作" width="180" v-if="showActions">
        <template #default="scope">
          <slot name="actions" :row="scope.row">
            <el-button type="primary" text size="small" @click="$emit('view', scope.row)">
              <el-icon><View /></el-icon>
              查看
            </el-button>
            <el-button type="success" text size="small" @click="$emit('edit', scope.row)">
              <el-icon><Edit /></el-icon>
              编辑
            </el-button>
            <el-button type="danger" text size="small" @click="$emit('delete', scope.row)">
              <el-icon><Delete /></el-icon>
              删除
            </el-button>
          </slot>
        </template>
      </el-table-column>
    </el-table>

    <!-- 分页 -->
    <div class="pagination-container" v-if="showPagination">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="pageSizes"
        layout="total, sizes, prev, pager, next, jumper"
        :total="total"
        @size-change="$emit('size-change', $event)"
        @current-change="$emit('current-change', $event)"
      ></el-pagination>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref } from 'vue';
import { Plus, Edit, Delete, View } from '@element-plus/icons-vue';

// 泛型定义
interface Column {
  prop: string;
  label: string;
  width?: string | number;
  align?: 'left' | 'center' | 'right';
}

const props = defineProps<{
  // 数据类型
  data: any[];
  // 列定义
  columns: Column[];
  // 标题
  title?: string;
  // 加载状态
  loading?: boolean;
  // 是否显示头部
  showHeader?: boolean;
  // 是否显示添加按钮
  showAddButton?: boolean;
  // 添加按钮文本
  addButtonText?: string;
  // 是否显示操作列
  showActions?: boolean;
  // 是否显示分页
  showPagination?: boolean;
  // 当前页码
  currentPage?: number;
  // 每页大小
  pageSize?: number;
  // 每页大小选项
  pageSizes?: number[];
  // 总条数
  total?: number;
}>();

// 定义默认值
const currentPage = ref(props.currentPage || 1);
const pageSize = ref(props.pageSize || 10);

// 定义事件
const emit = defineEmits<{
  (e: 'add'): void;
  (e: 'view', row: any): void;
  (e: 'edit', row: any): void;
  (e: 'delete', row: any): void;
  (e: 'size-change', size: number): void;
  (e: 'current-change', page: number): void;
}>();
</script>

<style scoped>
.crud-table {
  width: 100%;
}

.table-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
  padding-bottom: 10px;
  border-bottom: 1px solid #e4e7ed;
}

.table-header span {
  font-size: 16px;
  font-weight: bold;
  color: #303133;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}
</style>



