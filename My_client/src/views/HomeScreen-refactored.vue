<template>
  <div class="home-screen">
    <!-- 页面标题 -->
  <div class="screen-header">
    <h1>运营安全监控大屏</h1>
    <div class="header-info">
      <el-button type="primary" @click="handleRefresh">
        <el-icon><Refresh /></el-icon>
        刷新数据
      </el-button>
    </div>
  </div>
    
    <!-- 统计卡片区域 -->
    <div class="stats-section">
      <el-row :gutter="20">
        <!-- 企业总数 -->
        <el-col :span="4">
          <StatCard
            label="企业总数"
            :value="statsData.companyCount"
            icon="OfficeBuilding"
          />
        </el-col>
        
        <!-- 车辆总数 -->
        <el-col :span="4">
          <StatCard
            label="车辆总数"
            :value="statsData.vehicleCount"
            icon="Van"
          />
        </el-col>
        
        <!-- 运输行业类别分布 -->
        <el-col :span="4">
          <StatCard
            label="运输行业类别分布"
            :value="statsData.industryDistribution"
            icon="PieChart"
          />
        </el-col>
        
        <!-- 主动安全车辆数 -->
        <el-col :span="4">
          <StatCard
            label="主动安全车辆数"
            :value="statsData.activeSafetyVehicles"
            unit="辆"
            icon="Shield"
          />
        </el-col>
        
        <!-- 今日报警数 -->
        <el-col :span="4">
          <StatCard
            label="今日报警数"
            :value="statsData.todayAlarms"
            unit="条"
            icon="Warning"
            :is-alarm="true"
          />
        </el-col>
        
        <!-- 今日查岗情况 -->
        <el-col :span="4">
          <StatCard
            label="今日查岗情况"
            :value="statsData.todayChecks"
            unit="次"
            icon="View"
          />
        </el-col>
      </el-row>
    </div>
    
    <!-- 中间图表区域 -->
    <div class="charts-section">
      <el-row :gutter="20">
        <!-- 车辆上线情况 -->
        <el-col :span="12">
          <ChartCard title="车辆上线情况" height="300px">
            <div class="online-status">
              <div class="status-item">
                <span class="status-label">服务器状态</span>
                <span class="status-value today">运行中</span>
              </div>
              <div class="status-item">
                <span class="status-label">服务器地址</span>
                <span class="status-value online">localhost:8082</span>
              </div>
              <div class="status-item">
                <span class="status-label">时间</span>
                <span class="status-value online">{{ currentTime }}</span>
              </div>
              <div class="status-item">
                <span class="status-label">实时在线</span>
                <span class="status-value online">{{ statsData.onlineStatus.realTime }}</span>
              </div>
              <div class="status-item">
                <span class="status-label">今日上线</span>
                <span class="status-value today">{{ statsData.onlineStatus.today }}</span>
              </div>
              <div class="status-item">
                <span class="status-label">3日离线</span>
                <span class="status-value offline">{{ statsData.onlineStatus.threeDaysOffline }}</span>
              </div>
              <div class="status-item">
                <span class="status-label">7日离线</span>
                <span class="status-value offline">{{ statsData.onlineStatus.sevenDaysOffline }}</span>
              </div>
              <div class="status-item">
                <span class="status-label">30日离线</span>
                <span class="status-value offline">{{ statsData.onlineStatus.thirtyDaysOffline }}</span>
              </div>
            </div>
          </ChartCard>
        </el-col>
        
        <!-- 实时风险等级分布 -->
        <el-col :span="12">
          <ChartCard title="实时风险等级分布" height="300px">
            <div class="risk-levels">
              <div class="risk-item high-risk">
                <div class="risk-label">高风险</div>
                <div class="risk-value">{{ statsData.riskLevels.high }}</div>
              </div>
              <div class="risk-item medium-risk">
                <div class="risk-label">中风险</div>
                <div class="risk-value">{{ statsData.riskLevels.medium }}</div>
              </div>
              <div class="risk-item low-risk">
                <div class="risk-label">低风险</div>
                <div class="risk-value">{{ statsData.riskLevels.low }}</div>
              </div>
            </div>
          </ChartCard>
        </el-col>
      </el-row>
    </div>
    
    <!-- 底部图表区域 -->
    <div class="bottom-charts-section">
      <el-row :gutter="20">
        <!-- 驾驶员异常分布 -->
        <el-col :span="8">
          <ChartCard title="驾驶员异常分布" height="300px">
            <EChartsBase
              :option="driverAbnormalChartOption"
              height="260px"
            />
          </ChartCard>
        </el-col>
        
        <!-- 七日安全对比 -->
        <el-col :span="8">
          <ChartCard title="七日安全对比" height="300px">
            <EChartsBase
              :option="sevenDaysCompareChartOption"
              height="260px"
            />
          </ChartCard>
        </el-col>
        
        <!-- 近30日风险趋势 -->
        <el-col :span="8">
          <ChartCard title="近30日风险趋势" height="300px">
            <EChartsBase
              :option="thirtyDaysTrendChartOption"
              height="260px"
            />
          </ChartCard>
        </el-col>
      </el-row>
    </div>
    
    <!-- 车辆运行状态表格 -->
    <div class="table-section">
      <ChartCard title="车辆运行状态" height="auto">
        <el-table :data="vehicleStatusData" stripe size="small">
          <el-table-column prop="companyName" label="企业名称"></el-table-column>
          <el-table-column prop="total" label="总数"></el-table-column>
          <el-table-column prop="online" label="上线数"></el-table-column>
          <el-table-column prop="onlineRate" label="上线率">
            <template #default="scope">
              <el-progress :percentage="scope.row.onlineRate" :stroke-width="8" :format="(percentage) => `${percentage}%`"></el-progress>
            </template>
          </el-table-column>
          <el-table-column prop="activeSafetyTotal" label="主动安全总数"></el-table-column>
          <el-table-column prop="activeSafetyOnline" label="主动安全上线数"></el-table-column>
          <el-table-column prop="activeSafetyRate" label="主动安全上线率">
            <template #default="scope">
              <el-progress :percentage="scope.row.activeSafetyRate" :stroke-width="8" :format="(percentage) => `${percentage}%`"></el-progress>
            </template>
          </el-table-column>
        </el-table>
      </ChartCard>
    </div>
    
    <!-- 行驶里程统计 -->
    <div class="mileage-section">
      <el-row :gutter="20">
        <el-col :span="8">
          <StatCard
            label="车辆安全行驶累计里程"
            :value="statsData.mileage.total"
            unit="km"
            :format-fn="formatMileage"
            icon="Odometer"
          />
        </el-col>
        <el-col :span="8">
          <StatCard
            label="本月安全行驶累计里程"
            :value="statsData.mileage.month"
            unit="km"
            :format-fn="formatMileage"
            icon="Odometer"
          />
        </el-col>
        <el-col :span="8">
          <StatCard
            label="本日安全行驶累计里程"
            :value="statsData.mileage.day"
            unit="km"
            :format-fn="formatMileage"
            icon="Odometer"
          />
        </el-col>
      </el-row>
    </div>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh } from '@element-plus/icons-vue'
import StatCard from '@/components/stat-card/StatCard.vue'
import ChartCard from '@/components/chart-card/ChartCard.vue'
import EChartsBase from '@/components/echarts-base/EChartsBase.vue'
import type { EChartsOption } from 'echarts'

// 统计数据
const statsData = ref({
  companyCount: 156,
  vehicleCount: 2845,
  industryDistribution: '5类',
  onlineStatus: {
    realTime: 2340,
    today: 2580,
    threeDaysOffline: 180,
    sevenDaysOffline: 245,
    thirtyDaysOffline: 380
  },
  activeSafetyVehicles: 2150,
  todayAlarms: 32,
  riskLevels: {
    high: 5,
    medium: 28,
    low: 242
  },
  todayChecks: 45,
  mileage: {
    total: 12567890,
    month: 456780,
    day: 12345
  }
})

// 车辆运行状态数据
const vehicleStatusData = ref([
  {
    companyName: '混凝土公司A',
    total: 120,
    online: 115,
    onlineRate: 96,
    activeSafetyTotal: 90,
    activeSafetyOnline: 88,
    activeSafetyRate: 98
  },
  {
    companyName: '物流公司B',
    total: 85,
    online: 78,
    onlineRate: 92,
    activeSafetyTotal: 60,
    activeSafetyOnline: 55,
    activeSafetyRate: 92
  },
  {
    companyName: '运输公司C',
    total: 95,
    online: 90,
    onlineRate: 95,
    activeSafetyTotal: 75,
    activeSafetyOnline: 72,
    activeSafetyRate: 96
  }
])

// 当前时间
const currentTime = ref('')

// 格式化里程
const formatMileage = (value: number) => {
  return new Intl.NumberFormat('zh-CN').format(value)
}

// 更新当前时间
const updateCurrentTime = () => {
  const now = new Date()
  currentTime.value = now.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

// 驾驶员异常分布图表
const driverAbnormalChartOption = computed<EChartsOption>(() => ({
  tooltip: {
    trigger: 'item'
  },
  legend: {
    orient: 'vertical',
    right: 10,
    top: 'center'
  },
  series: [
    {
      name: '驾驶员异常分布',
      type: 'pie',
      radius: ['40%', '70%'],
      avoidLabelOverlap: false,
      itemStyle: {
        borderRadius: 10,
        borderColor: '#fff',
        borderWidth: 2
      },
      label: {
        show: false,
        position: 'center'
      },
      emphasis: {
        label: {
          show: true,
          fontSize: 20,
          fontWeight: 'bold'
        }
      },
      labelLine: {
        show: false
      },
      data: [
        { value: 45, name: '疲劳驾驶' },
        { value: 28, name: '超速' },
        { value: 18, name: '分心驾驶' },
        { value: 12, name: '其他' }
      ]
    }
  ]
}))

// 七日安全对比图表
const sevenDaysCompareChartOption = computed<EChartsOption>(() => ({
  tooltip: {
    trigger: 'axis'
  },
  legend: {
    data: ['报警数', '查岗数']
  },
  xAxis: {
    type: 'category',
    data: ['周一', '周二', '周三', '周四', '周五', '周六', '周日']
  },
  yAxis: {
    type: 'value'
  },
  series: [
    {
      name: '报警数',
      type: 'line',
      data: [28, 32, 25, 30, 35, 28, 32],
      smooth: true,
      itemStyle: { color: '#f5576c' }
    },
    {
      name: '查岗数',
      type: 'line',
      data: [42, 45, 40, 48, 50, 45, 45],
      smooth: true,
      itemStyle: { color: '#4facfe' }
    }
  ]
}))

// 近30日风险趋势图表
const thirtyDaysTrendChartOption = computed<EChartsOption>(() => ({
  tooltip: {
    trigger: 'axis'
  },
  xAxis: {
    type: 'category',
    data: Array.from({ length: 30 }, (_, i) => `${i + 1}日`)
  },
  yAxis: {
    type: 'value'
  },
  series: [
    {
      name: '高风险',
      type: 'bar',
      stack: 'total',
      data: Array.from({ length: 30 }, () => Math.floor(Math.random() * 5)),
      itemStyle: { color: '#f5576c' }
    },
    {
      name: '中风险',
      type: 'bar',
      stack: 'total',
      data: Array.from({ length: 30 }, () => Math.floor(Math.random() * 20)),
      itemStyle: { color: '#4facfe' }
    },
    {
      name: '低风险',
      type: 'bar',
      stack: 'total',
      data: Array.from({ length: 30 }, () => Math.floor(Math.random() * 100)),
      itemStyle: { color: '#43e97b' }
    }
  ]
}))

// 刷新数据
const handleRefresh = async () => {
  ElMessage.success('数据已刷新')
  // TODO: 调用 API 获取最新数据
}

let timeTimer: any

onMounted(() => {
  updateCurrentTime()
  timeTimer = setInterval(updateCurrentTime, 1000)
})

onUnmounted(() => {
  if (timeTimer) {
    clearInterval(timeTimer)
  }
})
</script>

<style scoped>
.home-screen {
  padding: 20px;
  width: 100%;
  height: 100%;
  overflow-y: auto;
  background-color: #f0f2f5;
}

.screen-header {
  margin-bottom: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.screen-header h1 {
  margin: 0;
  font-size: 24px;
  color: #303133;
  font-weight: bold;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 20px;
}

.current-time {
  font-size: 16px;
  color: #606266;
  font-family: 'Courier New', monospace;
}

/* 统计卡片区域 */
.stats-section {
  margin-bottom: 20px;
}

/* 图表区域 */
.charts-section {
  margin-bottom: 20px;
}

.bottom-charts-section {
  margin-bottom: 20px;
}

/* 车辆上线情况样式 */
.online-status {
  display: flex;
  flex-direction: column;
  gap: 15px;
  width: 100%;
  padding: 10px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.status-item:hover {
  transform: translateX(5px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.status-label {
  font-size: 14px;
  color: #606266;
  font-weight: 500;
}

.status-value {
  font-size: 20px;
  font-weight: bold;
  padding: 5px 15px;
  border-radius: 20px;
  color: white;
}

.status-value.online {
  background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
}

.status-value.today {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
}

.status-value.offline {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

/* 风险等级样式 */
.risk-levels {
  display: flex;
  justify-content: space-around;
  width: 100%;
  padding: 20px;
}

.risk-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 120px;
  height: 120px;
  border-radius: 50%;
  color: white;
  font-weight: bold;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s ease;
}

.risk-item:hover {
  transform: scale(1.1);
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
}

.high-risk {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.medium-risk {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
}

.low-risk {
  background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
}

.risk-label {
  font-size: 14px;
  margin-bottom: 5px;
  opacity: 0.9;
}

.risk-value {
  font-size: 32px;
  font-weight: bold;
}

/* 表格区域 */
.table-section {
  margin-bottom: 20px;
}

/* 里程统计区域 */
.mileage-section {
  margin-bottom: 20px;
}
</style>
