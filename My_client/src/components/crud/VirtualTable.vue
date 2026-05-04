<template>
  <div class="virtual-table" ref="containerRef" :style="{ height: `${height}px` }">
    <div class="virtual-table-header" :style="{ width: `${totalWidth}px` }">
      <div
        v-for="column in columns"
        :key="String(column.prop)"
        class="virtual-table-cell header-cell"
        :style="getColumnStyle(column)"
      >
        {{ column.label }}
      </div>
    </div>

    <div
      class="virtual-table-body"
      ref="bodyRef"
      :style="{ height: `${bodyHeight}px`, top: `${offsetY}px` }"
      @scroll="handleScroll"
    >
      <div :style="{ height: `${totalHeight}px`, width: `${totalWidth}px` }">
        <div
          v-for="row in visibleRows"
          :key="getRowKey(row, rowIndex)"
          class="virtual-table-row"
          :style="{ height: `${rowHeight}px` }"
        >
          <div
            v-for="column in columns"
            :key="String(column.prop)"
            class="virtual-table-cell"
            :style="getColumnStyle(column)"
          >
            <slot :name="`column-${String(column.prop)}`" :row="row" :column="column">
              {{ getCellValue(row, column.prop) }}
            </slot>
          </div>
        </div>
      </div>
    </div>

    <div class="virtual-table-footer" v-if="showPagination">
      <el-pagination
        v-model:current-page="internalCurrentPage"
        v-model:page-size="internalPageSize"
        :page-sizes="pageSizes"
        :total="total"
        layout="total, sizes, prev, pager, next"
        @size-change="handleSizeChange"
        @current-change="handleCurrentChange"
      />
    </div>
  </div>
</template>

<script setup lang="ts" generic="T extends Record<string, unknown>">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';

export interface VirtualTableColumn<T = Record<string, unknown>> {
  prop: keyof T | string;
  label: string;
  width?: number;
  minWidth?: number;
  flex?: number;
  align?: 'left' | 'center' | 'right';
}

export interface VirtualTableProps<T extends Record<string, unknown>> {
  data: T[];
  columns: VirtualTableColumn<T>[];
  rowHeight?: number;
  height?: number;
  showPagination?: boolean;
  currentPage?: number;
  pageSize?: number;
  pageSizes?: number[];
  total?: number;
  rowKey?: keyof T | ((row: T) => string | number);
}

const props = withDefaults(defineProps<VirtualTableProps<T>>(), {
  rowHeight: 48,
  height: 600,
  showPagination: true,
  currentPage: 1,
  pageSize: 100,
  pageSizes: () => [50, 100, 200, 500],
  total: 0,
  rowKey: 'id',
});

const emit = defineEmits<{
  (e: 'scroll', event: Event): void;
  (e: 'size-change', size: number): void;
  (e: 'current-change', page: number): void;
  (e: 'update:currentPage', page: number): void;
  (e: 'update:pageSize', size: number): void;
}>();

const containerRef = ref<HTMLElement | null>(null);
const bodyRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const containerHeight = ref(props.height);

const internalCurrentPage = ref(props.currentPage);
const internalPageSize = ref(props.pageSize);

watch(() => props.currentPage, (val) => {
  internalCurrentPage.value = val;
});

watch(() => props.pageSize, (val) => {
  internalPageSize.value = val;
});

const bufferSize = 5;

const totalHeight = computed(() => props.data.length * props.rowHeight);

const totalWidth = computed(() => {
  let width = 0;
  props.columns.forEach((col) => {
    if (col.width) {
      width += Number(col.width);
    } else {
      width += col.minWidth || 100;
    }
  });
  return width;
});

const bodyHeight = computed(() => containerHeight.value - 48);

const startIndex = computed(() => {
  const start = Math.floor(scrollTop.value / props.rowHeight) - bufferSize;
  return Math.max(0, start);
});

const endIndex = computed(() => {
  const visibleCount = Math.ceil(bodyHeight.value / props.rowHeight);
  const end = startIndex.value + visibleCount + bufferSize * 2;
  return Math.min(props.data.length, end);
});

const visibleRows = computed(() => {
  return props.data.slice(startIndex.value, endIndex.value);
});

const rowIndex = computed(() => {
  return Math.floor(scrollTop.value / props.rowHeight);
});

const offsetY = computed(() => {
  return startIndex.value * props.rowHeight;
});

const getRowKey = (row: T, _index: number): string | number => {
  if (typeof props.rowKey === 'function') {
    return props.rowKey(row);
  }
  return row[props.rowKey] as string | number;
};

const getCellValue = (row: T, prop: keyof T | string): unknown => {
  if (!prop) return undefined;
  const value = row[prop as keyof T];
  return value !== undefined && value !== null ? String(value) : '';
};

const getColumnStyle = (column: VirtualTableColumn<T>) => {
  const style: Record<string, string> = {
    textAlign: column.align || 'left',
  };

  if (column.width) {
    style.width = `${column.width}px`;
    style.minWidth = `${column.width}px`;
    style.flex = '0 0 auto';
  } else if (column.minWidth) {
    style.minWidth = `${column.minWidth}px`;
    style.flex = '1';
  } else if (column.flex) {
    style.flex = String(column.flex);
  }

  return style;
};

const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement;
  scrollTop.value = target.scrollTop;
  emit('scroll', event);
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

  if (bodyRef.value) {
    bodyRef.value.scrollTop = 0;
    scrollTop.value = 0;
  }
};

const resizeObserver = ref<ResizeObserver | null>(null);

onMounted(() => {
  if (containerRef.value) {
    resizeObserver.value = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerHeight.value = entry.contentRect.height;
      }
    });
    resizeObserver.value.observe(containerRef.value);
  }
});

onUnmounted(() => {
  if (resizeObserver.value) {
    resizeObserver.value.disconnect();
  }
});

defineExpose({
  scrollTo: (top: number) => {
    if (bodyRef.value) {
      bodyRef.value.scrollTop = top;
    }
  },
  getScrollTop: () => scrollTop.value,
});
</script>

<style scoped>
.virtual-table {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  background: #fff;
}

.virtual-table-header {
  display: flex;
  background: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  flex-shrink: 0;
}

.header-cell {
  font-weight: bold;
  color: #303133;
  background: #f5f7fa;
}

.virtual-table-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
}

.virtual-table-row {
  display: flex;
  border-bottom: 1px solid #ebeef5;
  transition: background-color 0.2s;
}

.virtual-table-row:hover {
  background-color: #f5f7fa;
}

.virtual-table-cell {
  padding: 12px 10px;
  font-size: 14px;
  color: #606266;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: flex;
  align-items: center;
}

.virtual-table-footer {
  padding: 10px;
  border-top: 1px solid #e4e7ed;
  display: flex;
  justify-content: flex-end;
}

/* 滚动条样式 */
.virtual-table-body::-webkit-scrollbar {
  width: 8px;
}

.virtual-table-body::-webkit-scrollbar-track {
  background: #f1f1f1;
}

.virtual-table-body::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 4px;
}

.virtual-table-body::-webkit-scrollbar-thumb:hover {
  background: #a1a1a1;
}
</style>
