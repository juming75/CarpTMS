<template>
  <div class="alarm-center">
    <!-- 顶部标题和统计 -->
    <div class="header-section">
      <h1 class="center-title">处警中心 - 主动安全报警</h1>
      <div class="alarm-stats">
        <el-statistic title="未处理报警" :value="unhandledAlarms" class="stat-item">
          <template #suffix>
            <el-icon color="#ef4444"><WarningFilled /></el-icon>
          </template>
        </el-statistic>
        <el-statistic title="今日报警" :value="todayAlarms" class="stat-item">
          <template #suffix>
            <el-icon color="#f59e0b"><BellFilled /></el-icon>
          </template>
        </el-statistic>
        <el-statistic title="本月报警" :value="monthAlarms" class="stat-item">
          <template #suffix>
            <el-icon color="#3b82f6"><CalendarFilled /></el-icon>
          </template>
        </el-statistic>
        <el-statistic title="处理率" :value="handleRate" suffix="%" class="stat-item">
          <template #suffix>
            <el-icon color="#10b981"><CheckCircleFilled /></el-icon>
          </template>
        </el-statistic>
      </div>
    </div>

    <!-- 报警类型筛选 -->
    <el-card class="filter-card mb-24">
      <div class="filter-container">
        <el-select v-model="selectedAlarmType" placeholder="报警类型" multiple collapse-tags style="width: 200px">
          <el-option label="超速" value="overSpeed" />
          <el-option label="疲劳驾驶" value="fatigue" />
          <el-option label="超载" value="overload" />
          <el-option label="违规停车" value="illegalPark" />
          <el-option label="碰撞预警" value="collisionWarning" />
          <el-option label="车道偏离" value="laneDeparture" />
          <el-option label="其他" value="other" />
        </el-select>
        <el-select v-model="selectedAlarmLevel" placeholder="报警等级" style="width: 120px">
          <el-option label="全部" value="" />
          <el-option label="高危" value="high" />
          <el-option label="中危" value="medium" />
          <el-option label="低危" value="low" />
        </el-select>
        <el-select v-model="selectedStatus" placeholder="处理状态" style="width: 120px">
          <el-option label="全部" value="" />
          <el-option label="未处理" value="unhandled" />
          <el-option label="处理中" value="handling" />
          <el-option label="已处理" value="handled" />
        </el-select>
        <el-date-picker
          v-model="dateRange"
          type="daterange"
          range-separator="至"
          start-placeholder="开始日期"
          end-placeholder="结束日期"
          format="YYYY-MM-DD"
          value-format="YYYY-MM-DD"
          style="width: 300px"
        />
        <el-button type="primary" @click="handleSearch">搜索</el-button>
        <el-button @click="handleReset">重置</el-button>
        <el-button type="danger" @click="handleBatchProcess" :disabled="selectedAlarms.length === 0">
          批量处理
        </el-button>
      </div>
    </el-card>

    <!-- 主体内容区域 -->
    <el-row :gutter="20">
      <!-- 左侧：实时报警列表 -->
      <el-col :xs="24" :lg="16">
        <el-card shadow="hover" class="alarm-list-card">
          <template #header>
            <div class="card-header">
              <span>实时报警列表</span>
              <el-button type="primary" size="small" @click="handleRefresh">
                <el-icon><Refresh /></el-icon>
                刷新
              </el-button>
            </div>
          </template>
          <el-table
            :data="filteredAlarms"
            stripe
            style="width: 100%"
            @selection-change="handleSelectionChange"
            @row-dblclick="handleAlarmDblClick"
            row-key="id"
          >
            <el-table-column type="selection" width="55" />
            <el-table-column prop="id" label="报警ID" width="120" />
            <el-table-column prop="type" label="报警类型" width="120">
              <template #default="scope">
                <el-tag :type="getAlarmTypeColor(scope.row.type)" size="small">
                  {{ getAlarmTypeName(scope.row.type) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="level" label="报警等级" width="100">
              <template #default="scope">
                <el-tag
                  :type="scope.row.level === 'high' ? 'danger' : scope.row.level === 'medium' ? 'warning' : 'info'"
                  size="small"
                >
                  {{ scope.row.level === 'high' ? '高危' : scope.row.level === 'medium' ? '中危' : '低危' }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="vehicle" label="车辆信息" width="150">
              <template #default="scope">
                <div class="vehicle-info">
                  <div class="vehicle-plate">{{ scope.row.vehicle }}</div>
                  <div class="vehicle-driver">{{ scope.row.driver }}</div>
                </div>
              </template>
            </el-table-column>
            <el-table-column prop="location" label="位置" min-width="180" />
            <el-table-column prop="speed" label="速度(km/h)" width="100" />
            <el-table-column prop="time" label="报警时间" width="180" :formatter="formatTime" />
            <el-table-column prop="status" label="状态" width="100">
              <template #default="scope">
                <el-tag
                  :type="
                    scope.row.status === 'unhandled'
                      ? 'danger'
                      : scope.row.status === 'handling'
                        ? 'warning'
                        : 'success'
                  "
                  size="small"
                >
                  {{ getStatusText(scope.row.status) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="200" fixed="right">
              <template #default="scope">
                <el-button size="small" @click="handleViewDetail(scope.row)"> 详情 </el-button>
                <el-button
                  size="small"
                  :type="scope.row.status === 'unhandled' ? 'primary' : 'warning'"
                  @click="handleProcessAlarm(scope.row)"
                >
                  {{ scope.row.status === 'unhandled' ? '处理' : '重新处理' }}
                </el-button>
              </template>
            </el-table-column>
          </el-table>

          <!-- 分页 -->
          <div class="pagination-container">
            <el-pagination
              v-model:current-page="currentPage"
              v-model:page-size="pageSize"
              :page-sizes="[10, 20, 50, 100]"
              layout="total, sizes, prev, pager, next, jumper"
              :total="filteredAlarms.length"
              @size-change="handleSizeChange"
              @current-change="handleCurrentChange"
            />
          </div>
        </el-card>
      </el-col>

      <!-- 右侧：报警统计和详情 -->
      <el-col :xs="24" :lg="8">
        <!-- 报警趋势 -->
        <el-card shadow="hover" class="chart-card mb-24">
          <template #header>
            <span>报警趋势</span>
          </template>
          <div ref="alarmTrendRef" class="chart-container"></div>
        </el-card>

        <!-- 报警类型分布 -->
        <el-card shadow="hover" class="chart-card mb-24">
          <template #header>
            <span>报警类型分布</span>
          </template>
          <div ref="alarmTypeRef" class="chart-container"></div>
        </el-card>

        <!-- 快速处理面板 -->
        <el-card shadow="hover" class="quick-process-card">
          <template #header>
            <span>快速处理</span>
          </template>
          <div class="quick-process-content">
            <div class="process-item" v-for="item in quickProcessItems" :key="item.type">
              <div class="process-info">
                <div class="process-type">{{ getAlarmTypeName(item.type) }}</div>
                <div class="process-count">{{ item.count }} 条未处理</div>
              </div>
              <el-button type="primary" size="small" @click="handleQuickProcess(item.type)"> 处理全部 </el-button>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 报警详情对话框 -->
    <el-dialog v-model="showAlarmDetail" title="报警详情" width="800px">
      <div class="alarm-detail">
        <div class="detail-section">
          <h3 class="section-title">基本信息</h3>
          <el-descriptions :column="2" border>
            <el-descriptions-item label="报警ID">{{ selectedAlarm.id }}</el-descriptions-item>
            <el-descriptions-item label="报警类型">
              <el-tag :type="getAlarmTypeColor(selectedAlarm.type)">
                {{ getAlarmTypeName(selectedAlarm.type) }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="报警等级">
              <el-tag
                :type="
                  selectedAlarm.level === 'high' ? 'danger' : selectedAlarm.level === 'medium' ? 'warning' : 'info'
                "
              >
                {{ selectedAlarm.level === 'high' ? '高危' : selectedAlarm.level === 'medium' ? '中危' : '低危' }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="报警时间">{{ formatTime(selectedAlarm.time) }}</el-descriptions-item>
            <el-descriptions-item label="当前状态">
              <el-tag
                :type="
                  selectedAlarm.status === 'unhandled'
                    ? 'danger'
                    : selectedAlarm.status === 'handling'
                      ? 'warning'
                      : 'success'
                "
              >
                {{ getStatusText(selectedAlarm.status) }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="处理时间">{{
              selectedAlarm.handleTime ? formatTime(selectedAlarm.handleTime) : '未处理'
            }}</el-descriptions-item>
          </el-descriptions>
        </div>

        <div class="detail-section">
          <h3 class="section-title">车辆信息</h3>
          <el-descriptions :column="2" border>
            <el-descriptions-item label="车牌号">{{ selectedAlarm.vehicle }}</el-descriptions-item>
            <el-descriptions-item label="驾驶员">{{ selectedAlarm.driver }}</el-descriptions-item>
            <el-descriptions-item label="车辆类型">{{ selectedAlarm.vehicleType }}</el-descriptions-item>
            <el-descriptions-item label="所属公司">{{ selectedAlarm.company }}</el-descriptions-item>
            <el-descriptions-item label="当前速度">{{ selectedAlarm.speed }} km/h</el-descriptions-item>
            <el-descriptions-item label="GPS定位">{{ selectedAlarm.location }}</el-descriptions-item>
          </el-descriptions>
        </div>

        <div class="detail-section">
          <h3 class="section-title">报警详情</h3>
          <div class="alarm-content">
            {{ selectedAlarm.content }}
          </div>
          <div class="alarm-map" v-if="selectedAlarm.location">
            <h4 class="map-title">位置地图</h4>
            <div class="map-placeholder">
              <el-icon class="map-icon"><LocationFilled /></el-icon>
              <div class="map-text">地图加载中...</div>
              <div class="map-subtext">{{ selectedAlarm.location }}</div>
            </div>
          </div>
        </div>

        <div class="detail-section" v-if="selectedAlarm.status !== 'unhandled'">
          <h3 class="section-title">处理记录</h3>
          <el-timeline>
            <el-timeline-item
              v-for="record in selectedAlarm.handleRecords"
              :key="record.id"
              :timestamp="formatTime(record.time)"
              :type="record.type === 'start' ? 'warning' : record.type === 'process' ? 'primary' : 'success'"
            >
              <div class="timeline-content">
                <div class="record-operator">{{ record.operator }}</div>
                <div class="record-content">{{ record.content }}</div>
              </div>
            </el-timeline-item>
          </el-timeline>
        </div>
      </div>
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="showAlarmDetail = false">关闭</el-button>
          <el-button type="primary" @click="handleProcessAlarm(selectedAlarm)">
            {{ selectedAlarm.status === 'unhandled' ? '处理报警' : '重新处理' }}
          </el-button>
        </div>
      </template>
    </el-dialog>

    <!-- 处理报警对话框 -->
    <el-dialog v-model="showProcessDialog" title="处理报警" width="600px">
      <el-form :model="processForm" label-width="100px">
        <el-form-item label="处理结果" required>
          <el-select v-model="processForm.result" placeholder="请选择处理结果">
            <el-option label="已通知驾驶员" value="notified" />
            <el-option label="已派车处理" value="dispatched" />
            <el-option label="已解决" value="resolved" />
            <el-option label="误报" value="falseAlarm" />
            <el-option label="其他" value="other" />
          </el-select>
        </el-form-item>
        <el-form-item label="处理说明" required>
          <el-input
            v-model="processForm.description"
            type="textarea"
            :rows="4"
            placeholder="请输入处理说明"
            maxlength="200"
            show-word-limit
          />
        </el-form-item>
        <el-form-item label="处理人">
          <el-input v-model="processForm.operator" placeholder="请输入处理人" />
        </el-form-item>
      </el-form>
      <template #footer>
        <div class="dialog-footer">
          <el-button @click="showProcessDialog = false">取消</el-button>
          <el-button type="primary" @click="handleSubmitProcess">提交处理</el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck - WebSocket 类型定义不兼容
import { ref, reactive, computed, onMounted, onUnmounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
// 按需导入ECharts核心模块和需要的图表类型
import * as echarts from 'echarts/core';
import { PieChart, BarChart, LineChart } from 'echarts/charts';
import { TitleComponent, TooltipComponent, LegendComponent, GridComponent, DatasetComponent, TransformComponent } from 'echarts/components';
import { LabelLayout, UniversalTransition } from 'echarts/features';
import { CanvasRenderer } from 'echarts/renderers';

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
import { WarningFilled, Refresh } from '@element-plus/icons-vue';
import api from '@/api';

// 统计数据
const unhandledAlarms = ref(0);
const todayAlarms = ref(0);
const monthAlarms = ref(0);
const handleRate = ref(0);

// 报警类型映射
const alarmTypeMap: Record<string, string> = {
  overSpeed: '超速',
  fatigue: '疲劳驾驶',
  overload: '超载',
  illegalPark: '违规停车',
  collisionWarning: '碰撞预警',
  laneDeparture: '车道偏离',
  other: '其他',
};

// 报警类型颜色
const alarmTypeColorMap: Record<string, string> = {
  overSpeed: 'danger',
  fatigue: 'warning',
  overload: 'primary',
  illegalPark: 'info',
  collisionWarning: 'danger',
  laneDeparture: 'warning',
  other: 'info',
};

// 状态文本映射
const statusTextMap: Record<string, string> = {
  unhandled: '未处理',
  handling: '处理中',
  handled: '已处理',
};

// 报警列表数据
const alarms = ref<AlarmItem[]>([]);

// 筛选条件
const selectedAlarmType = ref<string[]>([]);
const selectedAlarmLevel = ref('');
const selectedStatus = ref('');
const dateRange = ref<[string, string] | null>(null);

// 分页
const currentPage = ref(1);
const pageSize = ref(10);

// 报警类型定义
interface AlarmItem {
  id: string;
  type: string;
  level: string;
  vehicle: string;
  driver: string;
  speed: number;
  location: string;
  content: string;
  status: string;
  time: string;
  handleRecords: unknown[];
}

// 选中的报警
const selectedAlarms = ref<AlarmItem[]>([]);
const selectedAlarm = ref<AlarmItem>({
  id: '',
  type: '',
  level: '',
  vehicle: '',
  driver: '',
  speed: 0,
  location: '',
  content: '',
  status: '',
  time: '',
  handleRecords: [],
});

// 对话框状态
const showAlarmDetail = ref(false);
const showProcessDialog = ref(false);

// 处理表单
const processForm = reactive({
  result: '',
  description: '',
  operator: '值班人员',
});

// 图表引用
const alarmTrendRef = ref<HTMLElement>();
const alarmTypeRef = ref<HTMLElement>();

// 图表实例
let alarmTrendChart: echarts.ECharts | null = null;
let alarmTypeChart: echarts.ECharts | null = null;

// 快速处理项
const quickProcessItems = ref<{ id: number; name: string; action: string }[]>([]);

// API响应类型
interface AlarmStatsResponse {
  data?: {
    unhandled?: number;
    today?: number;
    month?: number;
    handleRate?: number;
  };
}

interface AlarmListResponse {
  data?: AlarmItem[];
}

interface QuickProcessResponse {
  data?: unknown[];
}

interface TrendDataItem {
  count: number;
  time: string;
}

interface TrendResponse {
  data?: TrendDataItem[];
}

interface TypeResponse {
  data?: unknown[];
}

// 从API获取报警数据
const fetchAlarmData = async () => {
  try {
    // 获取报警统计数据
    const statsResponse = await api.get('/api/alerts/stats');
    if (statsResponse && statsResponse.data) {
      unhandledAlarms.value = statsResponse.data.unprocessed || 0;
      todayAlarms.value = 0;
      monthAlarms.value = 0;
      handleRate.value = 0;
    }

    // 获取报警列表
    const alarmsResponse = await api.get('/api/alerts');
    if (alarmsResponse && alarmsResponse.data && alarmsResponse.data.alerts) {
      alarms.value = alarmsResponse.data.alerts;
    }

    // 获取快速处理项
    const quickItemsResponse = await api.get('/api/alerts/quick-process');
    if (quickItemsResponse && quickItemsResponse.data) {
      quickProcessItems.value = quickItemsResponse.data;
    }

    // 获取报警趋势数据
    const trendResponse = await api.get('/api/alerts/trend');
    if (trendResponse && trendResponse.data) {
      updateAlarmTrendChart(trendResponse.data);
    }

    // 获取报警类型分布
    const typeResponse = await api.get('/api/alerts/types');
    if (typeResponse && typeResponse.data) {
      updateAlarmTypeChart(typeResponse.data);
    }

    ElMessage.success('报警数据加载成功');
  } catch (error) {
    console.error('获取报警数据失败:', error);
    ElMessage.error('获取报警数据失败，请检查网络连接');
  }
};

// 更新报警趋势图
const updateAlarmTrendChart = (data: TrendDataItem[]) => {
  if (!alarmTrendChart) return;

  const option = alarmTrendChart.getOption() as { series?: { data?: unknown[] }[]; xAxis?: { data?: unknown[] } };
  if (option && option.series) {
    option.series[0].data = data?.map((item) => item.count) || [];
    if (option.xAxis) {
      option.xAxis.data = data?.map((item) => item.time) || [];
    }
    alarmTrendChart.setOption(option);
  }
};

// 更新报警类型分布图
const updateAlarmTypeChart = (data: unknown[]) => {
  if (!alarmTypeChart) return;

  if (alarmTypeChart) {
    const option = alarmTypeChart.getOption() as { series?: { data?: unknown[] }[] };
    if (option && option.series && option.series[0]) {
      option.series[0].data = data;
      alarmTypeChart.setOption(option);
    }
  }
};

// 根据筛选条件过滤报警
const filteredAlarms = computed(() => {
  let filtered = [...alarms.value];

  // 按类型筛选
  if (selectedAlarmType.value.length > 0) {
    filtered = filtered.filter((alarm) => selectedAlarmType.value.includes(alarm.type));
  }

  // 按等级筛选
  if (selectedAlarmLevel.value) {
    filtered = filtered.filter((alarm) => alarm.level === selectedAlarmLevel.value);
  }

  // 按状态筛选
  if (selectedStatus.value) {
    filtered = filtered.filter((alarm) => alarm.status === selectedStatus.value);
  }

  // 按时间范围筛选
  if (dateRange.value) {
    const [start, end] = dateRange.value;
    filtered = filtered.filter((alarm) => {
      const alarmTime = new Date(alarm.time);
      return alarmTime >= new Date(start) && alarmTime <= new Date(end);
    });
  }

  // 排序：未处理的排在前面，按时间倒序
  filtered.sort((a, b) => {
    if (a.status !== b.status) {
      return a.status === 'unhandled' ? -1 : 1;
    }
    return new Date(b.time).getTime() - new Date(a.time).getTime();
  });

  return filtered;
});

// 初始化报警趋势图
const initAlarmTrendChart = () => {
  if (!alarmTrendRef.value) return;

  alarmTrendChart = echarts.init(alarmTrendRef.value);

  const option = {
    tooltip: {
      trigger: 'axis',
      formatter: '{b}: {c}次',
    },
    xAxis: {
      type: 'category',
      data: ['00:00', '03:00', '06:00', '09:00', '12:00', '15:00', '18:00', '21:00'],
    },
    yAxis: {
      type: 'value',
      name: '报警次数',
    },
    series: [
      {
        name: '报警次数',
        type: 'line',
        data: [0, 0, 0, 0, 0, 0, 0, 0],
        smooth: true,
        itemStyle: {
          color: '#ef4444',
        },
        lineStyle: {
          width: 3,
          color: '#ef4444',
        },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(239, 68, 68, 0.3)' },
            { offset: 1, color: 'rgba(239, 68, 68, 0.1)' },
          ]),
        },
      },
    ],
  };

  alarmTrendChart.setOption(option);
};

// 初始化报警类型分布
const initAlarmTypeChart = () => {
  if (!alarmTypeRef.value) return;

  alarmTypeChart = echarts.init(alarmTypeRef.value);

  const option = {
    tooltip: {
      trigger: 'item',
      formatter: '{b}: {c}次 ({d}%)',
    },
    legend: {
      orient: 'vertical',
      left: 10,
      top: 'center',
      textStyle: {
        fontSize: 12,
      },
    },
    series: [
      {
        name: '报警类型',
        type: 'pie',
        radius: ['40%', '70%'],
        center: ['70%', '50%'],
        data: [],
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)',
          },
        },
        label: {
          show: false,
        },
      },
    ],
  };

  alarmTypeChart.setOption(option);
};

// 获取报警类型名称
const getAlarmTypeName = (type: string) => {
  return alarmTypeMap[type] || type;
};

// 获取报警类型颜色
const getAlarmTypeColor = (type: string) => {
  return alarmTypeColorMap[type] || 'info';
};

// 获取状态文本
const getStatusText = (status: string) => {
  return statusTextMap[status] || status;
};

// 格式化时间
const formatTime = (time: string) => {
  if (!time) return '-';
  return new Date(time).toLocaleString();
};

// 处理选择变化
const handleSelectionChange = (selection: AlarmItem[]) => {
  selectedAlarms.value = selection;
};

// 处理查看详情
const handleViewDetail = (alarm: AlarmItem) => {
  selectedAlarm.value = { ...alarm };
  showAlarmDetail.value = true;
};

// 处理报警双击
const handleAlarmDblClick = (alarm: AlarmItem) => {
  handleViewDetail(alarm);
};

// 处理报警
const handleProcessAlarm = (alarm: AlarmItem) => {
  selectedAlarm.value = { ...alarm };
  processForm.result = '';
  processForm.description = '';
  showProcessDialog.value = true;
};

// 提交处理
const handleSubmitProcess = () => {
  if (!processForm.result || !processForm.description) {
    ElMessage.warning('请填写完整的处理信息');
    return;
  }

  // 更新报警状态
  const alarmIndex = alarms.value.findIndex((item) => item.id === selectedAlarm.value.id);
  if (alarmIndex > -1) {
    // 添加处理记录
    const newRecord = {
      id: Date.now(),
      type: 'process',
      operator: processForm.operator,
      content: `${processForm.result} - ${processForm.description}`,
      time: new Date().toISOString(),
    };

    // 如果是首次处理，添加开始处理记录
    if (alarms.value[alarmIndex].status === 'unhandled') {
      const startRecord = {
        id: Date.now() - 1,
        type: 'start',
        operator: '系统',
        content: '开始处理',
        time: new Date().toISOString(),
      };
      alarms.value[alarmIndex].handleRecords.push(startRecord);
    }

    alarms.value[alarmIndex].handleRecords.push(newRecord);
    alarms.value[alarmIndex].status = 'handled';
    alarms.value[alarmIndex].handleTime = new Date().toISOString();

    // 更新统计数据
    unhandledAlarms.value--;

    // 关闭对话框
    showProcessDialog.value = false;
    ElMessage.success('报警处理成功');

    // 更新快速处理项
    const quickIndex = quickProcessItems.value.findIndex((item) => item.type === selectedAlarm.value.type);
    if (quickIndex > -1 && quickProcessItems.value[quickIndex].count > 0) {
      quickProcessItems.value[quickIndex].count--;
    }
  }
};

// 快速处理
const handleQuickProcess = (type: string) => {
  ElMessageBox.confirm(`确定要处理所有${getAlarmTypeName(type)}类型的未处理报警吗？`, '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(() => {
      // 模拟处理
      let count = 0;
      alarms.value.forEach((alarm) => {
        if (alarm.type === type && alarm.status === 'unhandled') {
          alarm.status = 'handled';
          alarm.handleTime = new Date().toISOString();
          alarm.handleRecords.push(
            {
              id: Date.now() + count,
              type: 'start',
              operator: '系统',
              content: '开始处理',
              time: new Date().toISOString(),
            },
            {
              id: Date.now() + count + 1,
              type: 'process',
              operator: '值班人员',
              content: `快速处理 - ${getAlarmTypeName(type)}`,
              time: new Date().toISOString(),
            }
          );
          count++;
        }
      });

      // 更新统计数据
      unhandledAlarms.value -= count;

      // 更新快速处理项
      const quickIndex = quickProcessItems.value.findIndex((item) => item.type === type);
      if (quickIndex > -1) {
        quickProcessItems.value[quickIndex].count = 0;
      }

      ElMessage.success(`已处理${count}条报警`);
    })
    .catch(() => {
      // 取消处理
    });
};

// 批量处理
const handleBatchProcess = () => {
  if (selectedAlarms.value.length === 0) {
    ElMessage.warning('请选择要处理的报警');
    return;
  }

  ElMessageBox.confirm(`确定要处理选中的${selectedAlarms.value.length}条报警吗？`, '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(() => {
      // 模拟处理
      selectedAlarms.value.forEach((alarm) => {
        const alarmIndex = alarms.value.findIndex((item) => item.id === alarm.id);
        if (alarmIndex > -1) {
          alarms.value[alarmIndex].status = 'handled';
          alarms.value[alarmIndex].handleTime = new Date().toISOString();
          alarms.value[alarmIndex].handleRecords.push(
            {
              id: Date.now(),
              type: 'start',
              operator: '系统',
              content: '开始处理',
              time: new Date().toISOString(),
            },
            {
              id: Date.now() + 1,
              type: 'process',
              operator: '值班人员',
              content: '批量处理',
              time: new Date().toISOString(),
            }
          );
        }
      });

      // 更新统计数据
      unhandledAlarms.value -= selectedAlarms.value.length;

      // 清空选中
      selectedAlarms.value = [];

      ElMessage.success(`已处理${selectedAlarms.value.length}条报警`);
    })
    .catch(() => {
      // 取消处理
    });
};

// 刷新数据
const handleRefresh = () => {
  fetchAlarmData();
};

// 搜索
const handleSearch = () => {
  currentPage.value = 1;
  console.log('搜索条件:', {
    selectedAlarmType: selectedAlarmType.value,
    selectedAlarmLevel: selectedAlarmLevel.value,
    selectedStatus: selectedStatus.value,
    dateRange: dateRange.value,
  });
};

// 重置
const handleReset = () => {
  selectedAlarmType.value = [];
  selectedAlarmLevel.value = '';
  selectedStatus.value = '';
  dateRange.value = null;
  currentPage.value = 1;
};

// 处理分页大小变化
const handleSizeChange = (size: number) => {
  pageSize.value = size;
  currentPage.value = 1;
};

// 处理当前页变化
const handleCurrentChange = (page: number) => {
  currentPage.value = page;
};

// 窗口大小变化时调整图表
const handleResize = () => {
  alarmTrendChart?.resize();
  alarmTypeChart?.resize();
};

onMounted(() => {
  // 初始化图表
  initAlarmTrendChart();
  initAlarmTypeChart();

  // 从API获取数据
  fetchAlarmData();

  // 监听窗口大小变化
  window.addEventListener('resize', handleResize);
});

onUnmounted(() => {
  // 移除事件监听
  window.removeEventListener('resize', handleResize);

  // 销毁图表
  alarmTrendChart?.dispose();
  alarmTypeChart?.dispose();
});
</script>

<style scoped>
.alarm-center {
  padding: 20px;
  background-color: #f5f7fa;
  min-height: 100vh;
}

.header-section {
  margin-bottom: 24px;
}

.center-title {
  font-size: 28px;
  font-weight: bold;
  color: #1f2937;
  margin-bottom: 20px;
}

.alarm-stats {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.stat-item {
  flex: 1;
  min-width: 150px;
}

.filter-card {
  margin-bottom: 24px;
  transition: all 0.3s ease;
}

.filter-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.filter-container {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  align-items: center;
}

.mb-24 {
  margin-bottom: 24px;
}

.alarm-list-card {
  height: calc(100vh - 400px);
  display: flex;
  flex-direction: column;
  transition: all 0.3s ease;
}

.alarm-list-card:hover {
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.alarm-list-card .el-card__body {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.alarm-list-card .el-table {
  flex: 1;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header span {
  font-weight: bold;
  font-size: 16px;
}

.pagination-container {
  margin-top: 16px;
  display: flex;
  justify-content: center;
}

.chart-card {
  transition: all 0.3s ease;
}

.chart-card:hover {
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.chart-container {
  width: 100%;
  height: 250px;
}

.quick-process-card {
  transition: all 0.3s ease;
}

.quick-process-card:hover {
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
}

.quick-process-content {
  padding: 10px 0;
}

.process-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  margin-bottom: 12px;
  background-color: #f9fafb;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.process-item:hover {
  transform: translateX(5px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.process-info {
  flex: 1;
}

.process-type {
  font-weight: bold;
  color: #374151;
  margin-bottom: 4px;
}

.process-count {
  font-size: 12px;
  color: #6b7280;
}

.alarm-detail {
  padding: 10px 0;
}

.detail-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 16px;
  font-weight: bold;
  color: #374151;
  margin-bottom: 16px;
}

.vehicle-info {
  display: flex;
  flex-direction: column;
}

.vehicle-plate {
  font-weight: bold;
  color: #374151;
}

.vehicle-driver {
  font-size: 12px;
  color: #6b7280;
}

.alarm-content {
  padding: 16px;
  background-color: #f9fafb;
  border-radius: 8px;
  margin-bottom: 16px;
  line-height: 1.6;
}

.alarm-map {
  margin-top: 16px;
}

.map-title {
  font-size: 14px;
  font-weight: bold;
  color: #374151;
  margin-bottom: 12px;
}

.map-placeholder {
  width: 100%;
  height: 200px;
  background-color: #e5e7eb;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #6b7280;
}

.map-icon {
  font-size: 48px;
  margin-bottom: 12px;
  color: #3b82f6;
}

.map-text {
  font-size: 16px;
  font-weight: bold;
  margin-bottom: 8px;
}

.map-subtext {
  font-size: 12px;
}

.timeline-content {
  padding: 8px 0;
}

.record-operator {
  font-weight: bold;
  color: #374151;
  margin-bottom: 4px;
}

.record-content {
  color: #6b7280;
  line-height: 1.4;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>


