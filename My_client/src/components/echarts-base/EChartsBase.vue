<template>
  <div ref="chartRef" class="echarts-container" :style="{ height: height }"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue';
// 按需导入ECharts核心模块和需要的图表类型
import * as echarts from 'echarts/core';
import { PieChart, BarChart, LineChart } from 'echarts/charts';
import { TitleComponent, TooltipComponent, LegendComponent, GridComponent, DatasetComponent, TransformComponent } from 'echarts/components';
import { LabelLayout, UniversalTransition } from 'echarts/features';
import { CanvasRenderer } from 'echarts/renderers';
import type { EChartsOption } from 'echarts';

// 注册必要的组件
echarts.use([
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DatasetComponent,
  TransformComponent,
  PieChart,
  BarChart,
  LineChart,
  LabelLayout,
  UniversalTransition,
  CanvasRenderer,
]);

interface Props {
  option: EChartsOption;
  height?: string;
  autoResize?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  height: '400px',
  autoResize: true,
});

const chartRef = ref<HTMLElement>();
let chartInstance: echarts.ECharts | null = null;

// 初始化图表
const initChart = () => {
  if (!chartRef.value) return;

  chartInstance = echarts.init(chartRef.value);
  chartInstance.setOption(props.option);

  // 自适应
  if (props.autoResize) {
    window.addEventListener('resize', handleResize);
  }
};

// 更新图表
const updateChart = (option: EChartsOption) => {
  if (chartInstance) {
    chartInstance.setOption(option, true);
  }
};

// 自适应调整
const handleResize = () => {
  if (chartInstance) {
    chartInstance.resize();
  }
};

// 监听 option 变化
watch(
  () => props.option,
  (newOption) => {
    if (chartInstance) {
      chartInstance.setOption(newOption, true);
    }
  },
  { deep: true }
);

onMounted(() => {
  nextTick(() => {
    initChart();
  });
});

onUnmounted(() => {
  if (chartInstance) {
    chartInstance.dispose();
    chartInstance = null;
  }
  if (props.autoResize) {
    window.removeEventListener('resize', handleResize);
  }
});

// 暴露图表实例
defineExpose({
  chartInstance,
  updateChart,
});
</script>

<style scoped>
.echarts-container {
  width: 100%;
  height: 100%;
}
</style>


