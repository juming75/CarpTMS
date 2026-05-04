<template>
  <el-card class="performance-monitor">
    <template #header>
      <span>性能监控</span>
    </template>
    <el-tabs type="card">
      <el-tab-pane label="概览">
        <div class="stats-grid">
          <div class="stat-card">
            <div class="stat-icon memory">
              <el-icon><Monitor /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ memoryUsage }}%</div>
              <div class="stat-label">内存使用率</div>
            </div>
            <el-progress :percentage="memoryUsage" :color="memoryColor" :show-text="false" />
          </div>

          <div class="stat-card">
            <div class="stat-icon api">
              <el-icon><Connection /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ apiSummary.avgDuration }}ms</div>
              <div class="stat-label">平均API响应</div>
            </div>
          </div>

          <div class="stat-card">
            <div class="stat-icon success">
              <el-icon><CircleCheck /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ apiSummary.successRate }}%</div>
              <div class="stat-label">API成功率</div>
            </div>
          </div>

          <div class="stat-card">
            <div class="stat-icon count">
              <el-icon><DataAnalysis /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ apiSummary.count }}</div>
              <div class="stat-label">API调用次数</div>
            </div>
          </div>
        </div>
      </el-tab-pane>

      <el-tab-pane label="API日志">
        <el-table :data="apiMetrics" size="small" border>
          <el-table-column prop="timestamp" label="时间" width="180">
            <template #default="scope">{{ formatTime(scope.row.timestamp) }}</template>
          </el-table-column>
          <el-table-column prop="method" label="方法" width="80">
            <template #default="scope">
              <el-tag :type="getMethodType(scope.row.method)">{{ scope.row.method }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="url" label="URL" min-width="300" />
          <el-table-column prop="duration" label="耗时(ms)" width="100" align="right">
            <template #default="scope">
              <span :class="{ 'slow-api': scope.row.duration > 1000 }">{{ scope.row.duration }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="status" label="状态" width="80">
            <template #default="scope">
              <el-tag :type="scope.row.success ? 'success' : 'danger'">{{ scope.row.status }}</el-tag>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <el-tab-pane label="内存监控">
        <div class="chart-container">
          <div class="chart-header">
            <span>内存使用趋势</span>
          </div>
          <div class="memory-chart">
            <div class="chart-bars">
              <div
                v-for="(metric, index) in memoryMetrics.slice(0, 20).reverse()"
                :key="metric.id"
                class="chart-bar"
                :style="{ height: `${(metric.value / 500) * 100}%` }"
              >
                <div class="bar-tooltip">{{ metric.value }} MB</div>
              </div>
            </div>
            <div class="chart-labels">
              <span>20s前</span>
              <span>现在</span>
            </div>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Monitor, Connection, CircleCheck, DataAnalysis } from '@element-plus/icons-vue';
import { performanceMonitor, type APIMetric, type PerformanceMetric } from '@/utils/performanceMonitor';

const apiMetrics = ref<APIMetric[]>([]);
const memoryMetrics = ref<PerformanceMetric[]>([]);
const memoryUsage = ref(0);

const apiSummary = computed(() => {
  return performanceMonitor.getAPISummary();
});

const memoryColor = computed(() => {
  if (memoryUsage.value > 80) return '#f56c6c';
  if (memoryUsage.value > 60) return '#e6a23c';
  return '#67c23a';
});

const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
};

const getMethodType = (method: string): string => {
  const types: Record<string, string> = {
    GET: 'success',
    POST: 'primary',
    PUT: 'warning',
    DELETE: 'danger',
  };
  return types[method] || 'info';
};

const updateMetrics = (): void => {
  apiMetrics.value = performanceMonitor.getAPIMetrics();
  memoryMetrics.value = performanceMonitor.getMemoryMetrics();
  memoryUsage.value = performanceMonitor.getCurrentMemoryUsage();
};

let unsubscribe: () => void;

onMounted(() => {
  unsubscribe = performanceMonitor.subscribe(updateMetrics);
  updateMetrics();
});

onUnmounted(() => {
  if (unsubscribe) {
    unsubscribe();
  }
});
</script>

<style scoped>
.performance-monitor {
  height: 100%;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 20px;
}

.stat-card {
  background: #fafafa;
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
}

.stat-icon.memory {
  background: #fef08a;
  color: #854d0e;
}

.stat-icon.api {
  background: #bfdbfe;
  color: #1d4ed8;
}

.stat-icon.success {
  background: #bbf7d0;
  color: #166534;
}

.stat-icon.count {
  background: #ddd6fe;
  color: #5b21b6;
}

.stat-info {
  display: flex;
  flex-direction: column;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  color: #303133;
}

.stat-label {
  font-size: 12px;
  color: #909399;
}

.slow-api {
  color: #f56c6c;
  font-weight: bold;
}

.chart-container {
  padding: 16px;
  background: #fafafa;
  border-radius: 8px;
}

.chart-header {
  margin-bottom: 16px;
  font-weight: bold;
  color: #303133;
}

.memory-chart {
  display: flex;
  flex-direction: column;
  height: 200px;
}

.chart-bars {
  flex: 1;
  display: flex;
  align-items: flex-end;
  gap: 4px;
  border-bottom: 1px solid #e4e7ed;
  padding-bottom: 8px;
}

.chart-bar {
  flex: 1;
  background: linear-gradient(to top, #67c23a, #85ce61);
  border-radius: 4px 4px 0 0;
  position: relative;
  transition: height 0.3s ease;
}

.chart-bar:hover {
  background: linear-gradient(to top, #409eff, #67b8ff);
}

.bar-tooltip {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  background: #303133;
  color: #fff;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.2s;
}

.chart-bar:hover .bar-tooltip {
  opacity: 1;
}

.chart-labels {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
  font-size: 12px;
  color: #909399;
}
</style>