<template>
  <div class="dashboard">
    <!-- 页面标题 -->
    <div class="dashboard-header">
      <h2>仪表盘</h2>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-cards">
      <el-row :gutter="20">
        <el-col :span="6">
          <el-card shadow="hover" class="stats-card income-card">
            <div class="stats-content">
              <div class="stats-label">今日订单</div>
              <div class="stats-value">{{ statsData.todayOrders || 0 }}</div>
              <div class="stats-change"><span class="change-positive">+12.5%</span> 较昨日</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card shadow="hover" class="stats-card expense-card">
            <div class="stats-content">
              <div class="stats-label">在途车辆</div>
              <div class="stats-value">{{ statsData.onlineVehicles || 0 }}</div>
              <div class="stats-change"><span class="change-positive">+5.2%</span> 较昨日</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card shadow="hover" class="stats-card profit-card">
            <div class="stats-content">
              <div class="stats-label">今日收入</div>
              <div class="stats-value">¥{{ statsData.todayIncome ? statsData.todayIncome.toFixed(2) : '0.00' }}</div>
              <div class="stats-change"><span class="change-positive">+18.8%</span> 较昨日</div>
            </div>
          </el-card>
        </el-col>
        <el-col :span="6">
          <el-card shadow="hover" class="stats-card order-card">
            <div class="stats-content">
              <div class="stats-label">待处理订单</div>
              <div class="stats-value">{{ statsData.pendingOrders || 0 }}</div>
              <div class="stats-change"><span class="change-negative">-3.5%</span> 较昨日</div>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 图表区域 -->
    <div class="charts-container">
      <el-row :gutter="20">
        <el-col :span="12">
          <el-card shadow="hover" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>订单趋势</span>
              </div>
            </template>
            <div class="chart-content">
              <el-empty description="订单趋势图表"></el-empty>
            </div>
          </el-card>
        </el-col>
        <el-col :span="12">
          <el-card shadow="hover" class="chart-card">
            <template #header>
              <div class="card-header">
                <span>收入统计</span>
              </div>
            </template>
            <div class="chart-content">
              <el-empty description="收入统计图表"></el-empty>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>

    <!-- 最新动态和待处理事项 -->
    <div class="bottom-section">
      <el-row :gutter="20">
        <el-col :span="12">
          <el-card shadow="hover" class="dynamic-card">
            <template #header>
              <div class="card-header">
                <span>最新动态</span>
              </div>
            </template>
            <div class="dynamic-content">
              <el-scrollbar height="300px">
                <el-timeline>
                  <el-timeline-item
                    v-for="item in recentDynamics"
                    :key="item.id"
                    :timestamp="item.time"
                    :type="item.type"
                  >
                    <div class="timeline-content">
                      <strong>{{ item.title }}</strong>
                      <p>{{ item.description }}</p>
                    </div>
                  </el-timeline-item>
                </el-timeline>
              </el-scrollbar>
            </div>
          </el-card>
        </el-col>
        <el-col :span="12">
          <el-card shadow="hover" class="pending-card">
            <template #header>
              <div class="card-header">
                <span>待处理事项</span>
              </div>
            </template>
            <div class="pending-content">
              <el-scrollbar height="300px">
                <el-table :data="pendingTasks" stripe size="small">
                  <el-table-column prop="id" label="ID" width="60"></el-table-column>
                  <el-table-column prop="title" label="事项名称"></el-table-column>
                  <el-table-column prop="priority" label="优先级" width="80">
                    <template #default="scope">
                      <el-tag
                        :type="
                          scope.row.priority === 'high'
                            ? 'danger'
                            : scope.row.priority === 'medium'
                              ? 'warning'
                              : 'success'
                        "
                        size="small"
                      >
                        {{ scope.row.priority === 'high' ? '高' : scope.row.priority === 'medium' ? '中' : '低' }}
                      </el-tag>
                    </template>
                  </el-table-column>
                  <el-table-column prop="deadline" label="截止时间" width="150"></el-table-column>
                  <el-table-column prop="status" label="状态" width="80">
                    <template #default="scope">
                      <el-tag :type="scope.row.status === 'pending' ? 'warning' : 'success'" size="small">
                        {{ scope.row.status === 'pending' ? '待处理' : '已完成' }}
                      </el-tag>
                    </template>
                  </el-table-column>
                </el-table>
              </el-scrollbar>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import api from '@/api';

// 统计数据类型
interface StatsData {
  todayOrders: number;
  onlineVehicles: number;
  todayIncome: number;
  pendingOrders: number;
}

// 动态项类型
interface DynamicItem {
  id: string;
  title: string;
  description: string;
  time: string;
  type: string;
}

// 待处理事项类型
interface PendingTask {
  id: string;
  title: string;
  priority: 'high' | 'medium' | 'low';
  deadline: string;
  status: 'pending' | 'completed';
}

// 统计数据
const statsData = ref<StatsData>({
  todayOrders: 0,
  onlineVehicles: 0,
  todayIncome: 0,
  pendingOrders: 0,
});

// 最新动态
const recentDynamics = ref<DynamicItem[]>([]);

// 待处理事项
const pendingTasks = ref<PendingTask[]>([]);



// 从API获取仪表盘数据
const fetchDashboardData = async () => {
  try {
    // 获取统计数据 — 拦截器已解包，直接返回 { todayOrders, onlineVehicles, ... }
    const statsResponse = await api.get('/api/dashboard/stats') as any;
    if (statsResponse) {
      Object.assign(statsData.value, statsResponse);
    }

    // 获取最新动态 — 拦截器已解包，直接返回数组
    const dynamicsResponse = await api.get('/api/dashboard/dynamics') as any;
    if (dynamicsResponse) {
      recentDynamics.value = Array.isArray(dynamicsResponse) ? dynamicsResponse : [];
    }

    // 获取待处理事项 — 拦截器已解包，直接返回数组
    const tasksResponse = await api.get('/api/dashboard/tasks') as any;
    if (tasksResponse) {
      pendingTasks.value = Array.isArray(tasksResponse) ? tasksResponse : [];
    }

    console.log('仪表盘数据加载成功');
  } catch (error) {
    console.error('获取仪表盘数据失败:', error);
  }
};

onMounted(() => {
  // 初始化数据
  fetchDashboardData();
  console.log('Dashboard 初始化完成');
});
</script>

<style scoped>
.dashboard {
  padding: 20px;
  width: 100%;
  height: 100%;
  overflow-y: auto;
}

.dashboard-header {
  margin-bottom: 20px;
}

.dashboard-header h2 {
  margin: 0;
  font-size: 20px;
  color: #303133;
}

/* 统计卡片样式 */
.stats-cards {
  margin-bottom: 20px;
}

.stats-card {
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
}

.stats-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.12);
}

.income-card {
  border-left: 4px solid #67c23a;
}

.expense-card {
  border-left: 4px solid #f56c6c;
}

.profit-card {
  border-left: 4px solid #409eff;
}

.order-card {
  border-left: 4px solid #e6a23c;
}

.stats-content {
  text-align: center;
}

.stats-label {
  font-size: 14px;
  color: #606266;
  margin-bottom: 8px;
}

.stats-value {
  font-size: 24px;
  font-weight: bold;
  color: #303133;
  margin-bottom: 4px;
}

.change-positive {
  color: #67c23a;
  font-size: 12px;
}

.change-negative {
  color: #f56c6c;
  font-size: 12px;
}

/* 图表区域 */
.charts-container {
  margin-bottom: 20px;
}

.chart-card {
  height: 300px;
  display: flex;
  flex-direction: column;
}

.chart-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fafafa;
}

/* 底部区域 */
.bottom-section {
  margin-bottom: 20px;
}

.dynamic-card,
.pending-card {
  height: 400px;
  display: flex;
  flex-direction: column;
}

.dynamic-content,
.pending-content {
  flex: 1;
  overflow: hidden;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.timeline-content {
  font-size: 13px;
}

.timeline-content p {
  margin: 5px 0 0 0;
  color: #606266;
}
</style>


