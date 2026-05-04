<template>
  <div class="crud-table">
    <div class="table-header" v-if="showHeader">
      <span>{{ title }}</span>
      <el-button type="primary" @click="emit('add')" v-if="showAddButton">
        <el-icon><Plus /></el-icon>
        {{ addButtonText }}
      </el-button>
    </div>

    <el-table
      :data="data"
      stripe
      size="small"
      v-loading="loading"
      :height="tableHeight"
      :max-height="maxTableHeight"
    >
      <template v-for="column in columns" :key="String(column.prop)">
        <el-table-column
          :prop="String(column.prop)"
          :label="column.label"
          :width="column.width"
          :min-width="column.minWidth"
          :align="column.align || 'left'"
          :fixed="column.fixed"
          :sortable="column.sortable"
        >
          <template #default="scope">
            <slot :name="`column-${String(column.prop)}`" :row="scope.row" :column="column">
              {{ getCellValue(scope.row, column.prop) }}
            </slot>
          </template>
        </el-table-column>
      </template>

      <el-table-column label="操作" :width="actionsWidth" :fixed="actionsFixed" v-if="showActions">
        <template #default="scope">
          <slot name="actions" :row="scope.row">
            <el-button type="primary" text size="small" @click="emit('view', scope.row)">
              <el-icon><View /></el-icon>
              查看
            </el-button>
            <el-button type="success" text size="small" @click="emit('edit', scope.row)">
              <el-icon><Edit /></el-icon>
              编辑
            </el-button>
            <el-button type="danger" text size="small" @click="emit('delete', scope.row)">
              <el-icon><Delete /></el-icon>
              删除
            </el-button>
          </slot>
        </template>
      </el-table-column>
    </el-table>

    <div class="pagination-container" v-if="showPagination">
      <el-pagination
        v-model:current-page="internalCurrentPage"
        v-model:page-size="internalPageSize"
        :page-sizes="internalPageSizes"
        layout="total, sizes, prev, pager, next, jumper"
        :total="total"
        @size-change="handleSizeChange"
        @current-change="handleCurrentChange"
      />
    </div>
  </div>
</template>

<script setup lang="ts" generic="T extends Record<string, unknown>">
import { ref, computed, watch } from 'vue';
import { Plus, Edit, Delete, View } from '@element-plus/icons-vue';

export interface TableColumn<T = Record<string, unknown>> {
  prop: keyof T | string;
  label: string;
  width?: string | number;
  minWidth?: string | number;
  align?: 'left' | 'center' | 'right';
  fixed?: 'left' | 'right';
  sortable?: boolean | 'custom';
  formatter?: (row: T, column: TableColumn<T>, cellValue: unknown, rowIndex: number) => string;
}

export interface CrudTableProps<T extends Record<string, unknown>> {
  data: T[];
  columns: TableColumn<T>[];
  title?: string;
  loading?: boolean;
  showHeader?: boolean;
  showAddButton?: boolean;
  addButtonText?: string;
  showActions?: boolean;
  actionsWidth?: string | number;
  actionsFixed?: 'left' | 'right';
  showPagination?: boolean;
  currentPage?: number;
  pageSize?: number;
  pageSizes?: number[];
  total?: number;
  tableHeight?: string | number;
  maxTableHeight?: string | number;
}

const props = withDefaults(defineProps<CrudTableProps<T>>(), {
  title: '',
  loading: false,
  showHeader: true,
  showAddButton: true,
  addButtonText: '新增',
  showActions: true,
  actionsWidth: 180,
  actionsFixed: 'right',
  showPagination: true,
  currentPage: 1,
  pageSize: 20,
  pageSizes: () => [10, 20, 50, 100],
  total: 0,
  tableHeight: undefined,
  maxTableHeight: 600,
});

const emit = defineEmits<{
  (e: 'add'): void;
  (e: 'view', row: T): void;
  (e: 'edit', row: T): void;
  (e: 'delete', row: T): void;
  (e: 'size-change', size: number): void;
  (e: 'current-change', page: number): void;
  (e: 'update:currentPage', page: number): void;
  (e: 'update:pageSize', size: number): void;
}>();

const internalCurrentPage = ref(props.currentPage);
const internalPageSize = ref(props.pageSize);
const internalPageSizes = ref(props.pageSizes || [10, 20, 50, 100]);

watch(() => props.currentPage, (val) => {
  internalCurrentPage.value = val;
});

watch(() => props.pageSize, (val) => {
  internalPageSize.value = val;
});

const getCellValue = (row: T, prop: keyof T | string): unknown => {
  if (!prop) return undefined;
  const value = row[prop as keyof T];
  return value !== undefined && value !== null ? String(value) : '';
};

const handleSizeChange = (size: number) => {
  internalPageSize.value = size;
  emit('update:pageSize', size);
  emit('size-change', size);
};

const handleCurrentChange = (page: number) => {
  internalCurrentPage.value = page;
  emit('update:currentPage', page);
  emit('current-change', page);
};
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
