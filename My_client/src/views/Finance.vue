﻿﻿﻿<template>
  <div class="finance-management">
    <div class="finance-header">
      <h2>财务管理</h2>
    </div>

    <!-- 主功能标签页 -->
    <el-tabs v-model="activeMainTab" type="card" class="main-tabs">
      <!-- 费用核算 -->
      <el-tab-pane label="费用核算" name="cost">
        <div class="tab-content">
          <!-- 查询区域 -->
          <el-card class="query-card" shadow="hover" :body-style="{ padding: '20px' }">
            <el-form :model="costQueryForm" :inline="true" label-position="right" label-width="100px" size="small">
              <el-form-item label="订单编号">
                <el-input v-model="costQueryForm.orderNo" placeholder="请输入订单编号" clearable></el-input>
              </el-form-item>
              <el-form-item label="费用类型">
                <el-select v-model="costQueryForm.costType" placeholder="请选择费用类型" clearable>
                  <el-option label="全部" value=""></el-option>
                  <el-option label="运输费" value="transport"></el-option>
                  <el-option label="保险费" value="insurance"></el-option>
                  <el-option label="燃油费" value="fuel"></el-option>
                  <el-option label="过路费" value="toll"></el-option>
                  <el-option label="其他" value="other"></el-option>
                </el-select>
              </el-form-item>
              <el-form-item label="核算状态">
                <el-select v-model="costQueryForm.status" placeholder="请选择核算状态" clearable>
                  <el-option label="全部" value=""></el-option>
                  <el-option label="待核算" value="pending"></el-option>
                  <el-option label="已核算" value="completed"></el-option>
                </el-select>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="handleCostQuery">
                  <el-icon><Search /></el-icon>
                  查询
                </el-button>
                <el-button @click="handleCostReset">
                  <el-icon><Refresh /></el-icon>
                  重置
                </el-button>
                <el-button type="primary" @click="handleCostAdd">
                  <el-icon><Plus /></el-icon>
                  新增费用
                </el-button>
              </el-form-item>
            </el-form>
          </el-card>

          <!-- 费用列表 -->
          <el-card class="cost-list-card" shadow="hover" style="margin-top: 20px">
            <div class="cost-list">
              <el-table :data="filteredCostList" stripe size="small" v-loading="costLoading">
                <el-table-column prop="id" label="ID" width="60"></el-table-column>
                <el-table-column prop="orderNo" label="订单编号"></el-table-column>
                <el-table-column prop="costType" label="费用类型" width="100">
                  <template #default="scope">
                    {{ getCostTypeText(scope.row.costType) }}
                  </template>
                </el-table-column>
                <el-table-column prop="amount" label="费用金额" width="120" align="right">
                  <template #default="scope"> ¥{{ scope.row.amount.toFixed(2) }} </template>
                </el-table-column>
                <el-table-column prop="status" label="核算状态" width="100">
                  <template #default="scope">
                    <el-tag :type="scope.row.status === 'completed' ? 'success' : 'warning'" size="small">
                      {{ scope.row.status === 'completed' ? '已核算' : '待核算' }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="costDate" label="费用日期" width="120"></el-table-column>
                <el-table-column prop="remark" label="备注"></el-table-column>
                <el-table-column label="操作" width="150">
                  <template #default="scope">
                    <el-button type="primary" text size="small" @click="handleCostEdit(scope.row)">
                      <el-icon><Edit /></el-icon>
                      编辑
                    </el-button>
                    <el-button type="danger" text size="small" @click="handleCostDelete(scope.row)">
                      <el-icon><Delete /></el-icon>
                      删除
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </el-card>
        </div>
      </el-tab-pane>

      <!-- 发票管理 -->
      <el-tab-pane label="发票管理" name="invoice">
        <div class="tab-content">
          <!-- 查询区域 -->
          <el-card class="query-card" shadow="hover" :body-style="{ padding: '20px' }">
            <el-form :model="invoiceQueryForm" :inline="true" label-position="right" label-width="100px" size="small">
              <el-form-item label="发票号码">
                <el-input v-model="invoiceQueryForm.invoiceNo" placeholder="请输入发票号码" clearable></el-input>
              </el-form-item>
              <el-form-item label="订单编号">
                <el-input v-model="invoiceQueryForm.orderNo" placeholder="请输入订单编号" clearable></el-input>
              </el-form-item>
              <el-form-item label="发票类型">
                <el-select v-model="invoiceQueryForm.invoiceType" placeholder="请选择发票类型" clearable>
                  <el-option label="全部" value=""></el-option>
                  <el-option label="增值税专用发票" value="special"></el-option>
                  <el-option label="增值税普通发票" value="normal"></el-option>
                </el-select>
              </el-form-item>
              <el-form-item label="开票状态">
                <el-select v-model="invoiceQueryForm.status" placeholder="请选择开票状态" clearable>
                  <el-option label="全部" value=""></el-option>
                  <el-option label="待开票" value="pending"></el-option>
                  <el-option label="已开票" value="issued"></el-option>
                  <el-option label="已作废" value="cancelled"></el-option>
                </el-select>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="handleInvoiceQuery">
                  <el-icon><Search /></el-icon>
                  查询
                </el-button>
                <el-button @click="handleInvoiceReset">
                  <el-icon><Refresh /></el-icon>
                  重置
                </el-button>
                <el-button type="primary" @click="handleInvoiceAdd">
                  <el-icon><Plus /></el-icon>
                  新增发票
                </el-button>
              </el-form-item>
            </el-form>
          </el-card>

          <!-- 发票列表 -->
          <el-card class="invoice-list-card" shadow="hover" style="margin-top: 20px">
            <div class="invoice-list">
              <el-table :data="filteredInvoiceList" stripe size="small" v-loading="invoiceLoading">
                <el-table-column prop="id" label="ID" width="60"></el-table-column>
                <el-table-column prop="invoiceNo" label="发票号码"></el-table-column>
                <el-table-column prop="orderNo" label="订单编号"></el-table-column>
                <el-table-column prop="invoiceType" label="发票类型" width="150">
                  <template #default="scope">
                    {{ scope.row.invoiceType === 'special' ? '增值税专用发票' : '增值税普通发票' }}
                  </template>
                </el-table-column>
                <el-table-column prop="amount" label="发票金额" width="120" align="right">
                  <template #default="scope"> ¥{{ scope.row.amount.toFixed(2) }} </template>
                </el-table-column>
                <el-table-column prop="status" label="开票状态" width="100">
                  <template #default="scope">
                    <el-tag :type="getStatusTagType(scope.row.status)" size="small">
                      {{ getInvoiceStatusText(scope.row.status) }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="invoiceDate" label="开票日期" width="120"></el-table-column>
                <el-table-column label="操作" width="180">
                  <template #default="scope">
                    <el-button type="primary" text size="small" @click="handleInvoiceView(scope.row)">
                      <el-icon><View /></el-icon>
                      查看
                    </el-button>
                    <el-button type="success" text size="small" @click="handleInvoiceEdit(scope.row)">
                      <el-icon><Edit /></el-icon>
                      编辑
                    </el-button>
                    <el-button type="danger" text size="small" @click="handleInvoiceDelete(scope.row)">
                      <el-icon><Delete /></el-icon>
                      删除
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </el-card>
        </div>
      </el-tab-pane>

      <!-- 收支统计 -->
      <el-tab-pane label="收支统计" name="statistics">
        <div class="tab-content">
          <!-- 统计查询 -->
          <el-card class="query-card" shadow="hover" :body-style="{ padding: '20px' }">
            <el-form :model="statsQueryForm" :inline="true" label-position="right" label-width="100px" size="small">
              <el-form-item label="统计周期">
                <el-select v-model="statsQueryForm.period" placeholder="请选择统计周期">
                  <el-option label="本月" value="month"></el-option>
                  <el-option label="本季度" value="quarter"></el-option>
                  <el-option label="本年" value="year"></el-option>
                  <el-option label="自定义" value="custom"></el-option>
                </el-select>
              </el-form-item>
              <el-form-item v-if="statsQueryForm.period === 'custom'" label="时间范围">
                <el-date-picker
                  v-model="statsQueryForm.dateRange"
                  type="daterange"
                  range-separator="至"
                  start-placeholder="开始日期"
                  end-placeholder="结束日期"
                  size="small"
                ></el-date-picker>
              </el-form-item>
              <el-form-item>
                <el-button type="primary" @click="handleStatsQuery">
                  <el-icon><Search /></el-icon>
                  查询统计
                </el-button>
                <el-button type="success" @click="handleStatsExport">
                  <el-icon><Download /></el-icon>
                  导出报表
                </el-button>
              </el-form-item>
            </el-form>
          </el-card>

          <!-- 统计概览 -->
          <div class="stats-overview" style="margin-top: 20px">
            <el-row :gutter="20">
              <el-col :span="6">
                <el-card shadow="hover" class="stats-card income-card">
                  <div class="stats-content">
                    <div class="stats-label">总收入</div>
                    <div class="stats-value">¥{{ statsData.totalIncome.toFixed(2) }}</div>
                    <div class="stats-change"><span class="change-positive">+12.5%</span> 较上月</div>
                  </div>
                </el-card>
              </el-col>
              <el-col :span="6">
                <el-card shadow="hover" class="stats-card expense-card">
                  <div class="stats-content">
                    <div class="stats-label">总支出</div>
                    <div class="stats-value">¥{{ statsData.totalExpense.toFixed(2) }}</div>
                    <div class="stats-change"><span class="change-negative">+8.2%</span> 较上月</div>
                  </div>
                </el-card>
              </el-col>
              <el-col :span="6">
                <el-card shadow="hover" class="stats-card profit-card">
                  <div class="stats-content">
                    <div class="stats-label">净利润</div>
                    <div class="stats-value">¥{{ statsData.netProfit.toFixed(2) }}</div>
                    <div class="stats-change"><span class="change-positive">+15.8%</span> 较上月</div>
                  </div>
                </el-card>
              </el-col>
              <el-col :span="6">
                <el-card shadow="hover" class="stats-card order-card">
                  <div class="stats-content">
                    <div class="stats-label">订单数量</div>
                    <div class="stats-value">{{ statsData.orderCount }}</div>
                    <div class="stats-change"><span class="change-positive">+23.1%</span> 较上月</div>
                  </div>
                </el-card>
              </el-col>
            </el-row>
          </div>

          <!-- 收支明细 -->
          <el-card class="stats-detail-card" shadow="hover" style="margin-top: 20px">
            <template #header>
              <div class="card-header">
                <span>收支明细</span>
              </div>
            </template>
            <div class="stats-detail">
              <el-table :data="statsDetailList" stripe size="small">
                <el-table-column prop="date" label="日期" width="120"></el-table-column>
                <el-table-column prop="type" label="类型" width="80">
                  <template #default="scope">
                    <el-tag :type="scope.row.type === 'income' ? 'success' : 'danger'" size="small">
                      {{ scope.row.type === 'income' ? '收入' : '支出' }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="category" label="分类" width="120"></el-table-column>
                <el-table-column prop="amount" label="金额" width="120" align="right">
                  <template #default="scope">
                    <span :class="scope.row.type === 'income' ? 'text-income' : 'text-expense'">
                      {{ scope.row.type === 'income' ? '+' : '-' }}¥{{ scope.row.amount.toFixed(2) }}
                    </span>
                  </template>
                </el-table-column>
                <el-table-column prop="description" label="描述"></el-table-column>
              </el-table>
            </div>
          </el-card>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- 费用编辑对话框 -->
    <el-dialog v-model="costDialogVisible" :title="costForm.id ? '编辑费用' : '新增费用'" width="600px">
      <el-form :model="costForm" label-position="right" label-width="100px" size="small">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="订单编号" prop="orderNo">
              <el-input v-model="costForm.orderNo" placeholder="请输入订单编号"></el-input>
            </el-form-item>
            <el-form-item label="费用类型" prop="costType">
              <el-select v-model="costForm.costType" placeholder="请选择费用类型">
                <el-option label="运输费" value="transport"></el-option>
                <el-option label="保险费" value="insurance"></el-option>
                <el-option label="燃油费" value="fuel"></el-option>
                <el-option label="过路费" value="toll"></el-option>
                <el-option label="其他" value="other"></el-option>
              </el-select>
            </el-form-item>
            <el-form-item label="费用金额" prop="amount">
              <el-input v-model="costForm.amount" type="number" placeholder="请输入费用金额"></el-input>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="费用日期" prop="costDate">
              <el-date-picker v-model="costForm.costDate" type="date" placeholder="请选择费用日期" value-format="YYYY-MM-DD"></el-date-picker>
            </el-form-item>
            <el-form-item label="核算状态" prop="status">
              <el-select v-model="costForm.status" placeholder="请选择核算状态">
                <el-option label="待核算" value="pending"></el-option>
                <el-option label="已核算" value="completed"></el-option>
              </el-select>
            </el-form-item>
          </el-col>
        </el-row>
        <el-form-item label="备注" prop="remark">
          <el-input v-model="costForm.remark" type="textarea" rows="3" placeholder="请输入备注信息"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="costDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleCostSave">确定</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 发票编辑对话框 -->
    <el-dialog v-model="invoiceDialogVisible" :title="invoiceForm.id ? '编辑发票' : '新增发票'" width="600px">
      <el-form :model="invoiceForm" label-position="right" label-width="100px" size="small">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-form-item label="发票号码" prop="invoiceNo">
              <el-input v-model="invoiceForm.invoiceNo" placeholder="请输入发票号码"></el-input>
            </el-form-item>
            <el-form-item label="订单编号" prop="orderNo">
              <el-input v-model="invoiceForm.orderNo" placeholder="请输入订单编号"></el-input>
            </el-form-item>
            <el-form-item label="发票类型" prop="invoiceType">
              <el-select v-model="invoiceForm.invoiceType" placeholder="请选择发票类型">
                <el-option label="增值税专用发票" value="special"></el-option>
                <el-option label="增值税普通发票" value="normal"></el-option>
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="发票金额" prop="amount">
              <el-input v-model="invoiceForm.amount" type="number" placeholder="请输入发票金额"></el-input>
            </el-form-item>
            <el-form-item label="开票日期" prop="invoiceDate">
              <el-date-picker
                v-model="invoiceForm.invoiceDate"
                type="date"
                placeholder="请选择开票日期"
                value-format="YYYY-MM-DD"
              ></el-date-picker>
            </el-form-item>
            <el-form-item label="开票状态" prop="status">
              <el-select v-model="invoiceForm.status" placeholder="请选择开票状态">
                <el-option label="待开票" value="pending"></el-option>
                <el-option label="已开票" value="issued"></el-option>
                <el-option label="已作废" value="cancelled"></el-option>
              </el-select>
            </el-form-item>
          </el-col>
        </el-row>
        <el-form-item label="备注" prop="remark">
          <el-input v-model="invoiceForm.remark" type="textarea" rows="3" placeholder="请输入备注信息"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="invoiceDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleInvoiceSave">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, computed, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Plus, Edit, Delete, Search, Refresh, View, Download } from '@element-plus/icons-vue';
import api from '@/api';

// 主功能标签页
const activeMainTab = ref('cost');

// 类型定义
interface CostItem {
  id: string;
  orderNo: string;
  costType: string;
  amount: number;
  status: string;
  costDate: string;
  remark: string;
}

interface InvoiceItem {
  id: string;
  invoiceNo: string;
  orderNo: string;
  invoiceType: string;
  amount: number;
  status: string;
  invoiceDate: string;
  remark: string;
}

interface StatsDetailItem {
  date: string;
  type: string;
  category: string;
  amount: number;
  description: string;
}

// 费用核算相关
const costLoading = ref(false);
const costQueryForm = ref({
  orderNo: '',
  costType: '',
  status: '',
});

const costList = ref<CostItem[]>([]);

// 计算过滤后的费用列表
const filteredCostList = computed(() => {
  let result = [...costList.value];

  // 按订单编号过滤
  if (costQueryForm.value.orderNo) {
    result = result.filter((cost) => cost.orderNo.includes(costQueryForm.value.orderNo));
  }

  // 按费用类型过滤
  if (costQueryForm.value.costType) {
    result = result.filter((cost) => cost.costType === costQueryForm.value.costType);
  }

  // 按核算状态过滤
  if (costQueryForm.value.status) {
    result = result.filter((cost) => cost.status === costQueryForm.value.status);
  }

  return result;
});

// 发票管理相关
const invoiceLoading = ref(false);
const invoiceQueryForm = ref({
  invoiceNo: '',
  orderNo: '',
  invoiceType: '',
  status: '',
});

const invoiceList = ref<InvoiceItem[]>([]);

// 计算过滤后的发票列表
const filteredInvoiceList = computed(() => {
  let result = [...invoiceList.value];

  // 按发票号码过滤
  if (invoiceQueryForm.value.invoiceNo) {
    result = result.filter((invoice) => invoice.invoiceNo.includes(invoiceQueryForm.value.invoiceNo));
  }

  // 按订单编号过滤
  if (invoiceQueryForm.value.orderNo) {
    result = result.filter((invoice) => invoice.orderNo.includes(invoiceQueryForm.value.orderNo));
  }

  // 按发票类型过滤
  if (invoiceQueryForm.value.invoiceType) {
    result = result.filter((invoice) => invoice.invoiceType === invoiceQueryForm.value.invoiceType);
  }

  // 按开票状态过滤
  if (invoiceQueryForm.value.status) {
    result = result.filter((invoice) => invoice.status === invoiceQueryForm.value.status);
  }

  return result;
});

// 收支统计相关
const statsQueryForm = ref({
  period: 'month',
  dateRange: [] as [string, string][],
});

const statsData = ref({
  totalIncome: 0,
  totalExpense: 0,
  netProfit: 0,
  orderCount: 0,
});

const statsDetailList = ref<StatsDetailItem[]>([]);

// 对话框状态
const costDialogVisible = ref(false);
const invoiceDialogVisible = ref(false);

// 表单数据
const costForm = ref({
  id: '',
  orderNo: '',
  costType: 'transport',
  amount: 0,
  status: 'pending',
  costDate: new Date().toISOString().split('T')[0],
  remark: '',
});

const invoiceForm = ref({
  id: '',
  invoiceNo: '',
  orderNo: '',
  invoiceType: 'special',
  amount: 0,
  status: 'pending',
  invoiceDate: new Date().toISOString().split('T')[0],
  remark: '',
});

// 获取费用类型文本
const getCostTypeText = (type: string) => {
  const typeMap: Record<string, string> = {
    transport: '运输费',
    insurance: '保险费',
    fuel: '燃油费',
    toll: '过路费',
    other: '其他',
  };
  return typeMap[type] || type;
};

// 获取发票状态文本
const getInvoiceStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    pending: '待开票',
    issued: '已开票',
    cancelled: '已作废',
  };
  return statusMap[status] || status;
};

// 获取发票状态标签类型
const getStatusTagType = (status: string) => {
  const typeMap: Record<string, string> = {
    pending: 'warning',
    issued: 'success',
    cancelled: 'danger',
  };
  return typeMap[status] || 'info';
};

// API响应类型定义
interface CostApiResponse {
  items: Array<{
    cost_id: string;
    cost_type: string;
    amount: number;
    cost_date: string;
    description: string;
  }>;
}

interface InvoiceApiResponse {
  items: Array<{
    invoice_id: string;
    invoice_number: string;
    amount: number;
    invoice_date: string;
    description: string;
  }>;
}

interface StatsApiResponse {
  total_invoice: number;
  total_cost: number;
}

// 从后端获取费用数据
const fetchCosts = async () => {
  costLoading.value = true;
  try {
    const response = await api.get('/api/finance/costs', {
      params: costQueryForm.value,
    });
    if (response && response.data && response.data.list) {
      costList.value = response.data.list.map((item: any) => ({
        id: item.cost_id,
        orderNo: '',
        costType: item.cost_type,
        amount: item.amount,
        status: 'completed',
        costDate: item.cost_date,
        remark: item.description,
      }));
    }
  } catch (error) {
    console.error('获取费用列表失败:', error);
    ElMessage.error('获取费用列表失败');
  } finally {
    costLoading.value = false;
  }
};

// 从后端获取发票数据
const fetchInvoices = async () => {
  invoiceLoading.value = true;
  try {
    const response = await api.get('/api/finance/invoices', {
      params: invoiceQueryForm.value,
    });
    if (response && response.data && response.data.list) {
      invoiceList.value = response.data.list.map((item: any) => ({
        id: item.invoice_id,
        invoiceNo: item.invoice_number,
        orderNo: '',
        invoiceType: 'special',
        amount: item.amount,
        status: 'issued',
        invoiceDate: item.invoice_date,
        remark: item.description,
      }));
    }
  } catch (error) {
    console.error('获取发票列表失败:', error);
    ElMessage.error('获取发票列表失败');
  } finally {
    invoiceLoading.value = false;
  }
};

// 从后端获取统计数据
const fetchStats = async () => {
  try {
    const response = await api.get('/api/finance/statistics', {
      params: statsQueryForm.value,
    });
    if (response && response.data) {
      statsData.value = {
        totalIncome: response.data.total_invoice || 0,
        totalExpense: response.data.total_cost || 0,
        netProfit: (response.data.total_invoice || 0) - (response.data.total_cost || 0),
        orderCount: 0,
      };
      statsDetailList.value = [];
    }
  } catch (error) {
    console.error('获取统计数据失败:', error);
    ElMessage.error('获取统计数据失败');
  }
};

// 费用核算操作
const handleCostQuery = () => {
  fetchCosts();
};

const handleCostReset = () => {
  costQueryForm.value = {
    orderNo: '',
    costType: '',
    status: '',
  };
  fetchCosts();
};

const handleCostAdd = () => {
  costForm.value = {
    id: '',
    orderNo: '',
    costType: 'transport',
    amount: 0,
    status: 'pending',
    costDate: new Date().toISOString().split('T')[0],
    remark: '',
  };
  costDialogVisible.value = true;
};

const handleCostEdit = (row: CostItem) => {
  costForm.value = {
    ...row,
    costDate: row.costDate || new Date().toISOString().split('T')[0],
  };
  costDialogVisible.value = true;
};

const handleCostDelete = async (row: CostItem) => {
  try {
    await api.delete(`/api/finance/costs/${row.id}`);
    costList.value = costList.value.filter((item) => item.id !== row.id);
    ElMessage.success('费用删除成功');
  } catch (error) {
    console.error('删除费用失败:', error);
    ElMessage.error('删除费用失败');
  }
};

const handleCostSave = async () => {
  try {
    if (costForm.value.id) {
      // 编辑
      await api.put(`/api/finance/costs/${costForm.value.id}`, costForm.value);
      ElMessage.success('费用更新成功');
    } else {
      // 新增
      await api.post('/api/finance/costs', costForm.value);
      ElMessage.success('费用创建成功');
    }
    costDialogVisible.value = false;
    fetchCosts();
  } catch (error) {
    console.error('保存费用失败:', error);
    ElMessage.error('保存费用失败');
  }
};

// 发票管理操作
const handleInvoiceQuery = () => {
  fetchInvoices();
};

const handleInvoiceReset = () => {
  invoiceQueryForm.value = {
    invoiceNo: '',
    orderNo: '',
    invoiceType: '',
    status: '',
  };
  fetchInvoices();
};

const handleInvoiceAdd = () => {
  invoiceForm.value = {
    id: '',
    invoiceNo: '',
    orderNo: '',
    invoiceType: 'special',
    amount: 0,
    status: 'pending',
    invoiceDate: new Date().toISOString().split('T')[0],
    remark: '',
  };
  invoiceDialogVisible.value = true;
};

const handleInvoiceEdit = (row: InvoiceItem) => {
  invoiceForm.value = {
    ...row,
    invoiceDate: row.invoiceDate || new Date().toISOString().split('T')[0],
  };
  invoiceDialogVisible.value = true;
};

const handleInvoiceView = async (row: InvoiceItem) => {
  try {
    const response = await api.get(`/api/finance/invoices/${row.id}`);
    if (response) {
      // 可以在这里处理发票详情显示
      ElMessage.info('查看发票详情');
    }
  } catch (error) {
    console.error('获取发票详情失败:', error);
    ElMessage.error('获取发票详情失败');
  }
};

const handleInvoiceDelete = async (row: InvoiceItem) => {
  try {
    await api.delete(`/api/finance/invoices/${row.id}`);
    invoiceList.value = invoiceList.value.filter((item) => item.id !== row.id);
    ElMessage.success('发票删除成功');
  } catch (error) {
    console.error('删除发票失败:', error);
    ElMessage.error('删除发票失败');
  }
};

const handleInvoiceSave = async () => {
  try {
    if (invoiceForm.value.id) {
      // 编辑
      await api.put(`/api/finance/invoices/${invoiceForm.value.id}`, invoiceForm.value);
      ElMessage.success('发票更新成功');
    } else {
      // 新增
      await api.post('/api/finance/invoices', invoiceForm.value);
      ElMessage.success('发票创建成功');
    }
    invoiceDialogVisible.value = false;
    fetchInvoices();
  } catch (error) {
    console.error('保存发票失败:', error);
    ElMessage.error('保存发票失败');
  }
};

// 收支统计操作
const handleStatsQuery = () => {
  fetchStats();
};

const handleStatsExport = async () => {
  try {
    const response = await api.get('/api/finance/export', {
      params: statsQueryForm.value,
      responseType: 'blob',
    }) as { data: Blob };
    // 处理导出文件
    const blob = new Blob([response.data], { type: 'application/vnd.ms-excel' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `finance-stats-${new Date().toISOString().split('T')[0]}.xlsx`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    ElMessage.success('报表导出成功');
  } catch (error) {
    console.error('导出报表失败:', error);
    ElMessage.error('导出报表失败');
  }
};

onMounted(() => {
  fetchCosts();
  fetchInvoices();
  fetchStats();
});
</script>

<style scoped>
.finance-management {
  padding: 20px;
}

.finance-header {
  margin-bottom: 20px;
}

.finance-header h2 {
  margin: 0;
  font-size: 20px;
  color: #303133;
}

.tab-content {
  margin-top: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.query-card {
  margin-bottom: 20px;
}

.cost-list,
.invoice-list {
  margin-top: 10px;
}

/* 统计卡片样式 */
.stats-overview {
  margin-bottom: 20px;
}

.stats-card {
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
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

/* 收支明细 */
.text-income {
  color: #67c23a;
  font-weight: bold;
}

.text-expense {
  color: #f56c6c;
  font-weight: bold;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>


