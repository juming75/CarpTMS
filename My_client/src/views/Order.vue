<template>
  <div class="order-management">
    <div class="order-header">
      <h2>订单管理</h2>
    </div>

    <!-- 查询区域 -->
    <el-card class="query-card" shadow="hover" :body-style="{ padding: '20px' }">
      <el-form :model="queryForm" :inline="true" label-position="right" label-width="80px" size="small">
        <el-form-item label="订单编号">
          <el-input v-model="queryForm.orderNo" placeholder="请输入订单编号" clearable></el-input>
        </el-form-item>
        <el-form-item label="客户名称">
          <el-input v-model="queryForm.customer" placeholder="请输入客户名称" clearable></el-input>
        </el-form-item>
        <el-form-item label="订单状态">
          <el-select v-model="queryForm.status" placeholder="请选择订单状态" clearable>
            <el-option label="全部" value=""></el-option>
            <el-option label="待处理" value="pending"></el-option>
            <el-option label="进行中" value="processing"></el-option>
            <el-option label="已完成" value="completed"></el-option>
            <el-option label="已取消" value="cancelled"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="创建时间">
          <el-date-picker
            v-model="queryForm.dateRange"
            type="daterange"
            range-separator="至"
            start-placeholder="开始日期"
            end-placeholder="结束日期"
            size="small"
          ></el-date-picker>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleQuery">
            <el-icon><Search /></el-icon>
            查询
          </el-button>
          <el-button @click="handleReset">
            <el-icon><Refresh /></el-icon>
            重置
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 订单列表 -->
    <el-card class="order-list-card" shadow="hover" style="margin-top: 20px">
      <template #header>
        <div class="card-header">
          <span>订单列表</span>
          <el-button type="primary" @click="handleOrderAdd">
            <el-icon><Plus /></el-icon>
            新建订单
          </el-button>
        </div>
      </template>
      <div class="order-list">
        <el-table :data="filteredOrderList" stripe size="small" v-loading="loading">
          <el-table-column prop="id" label="ID" width="60"></el-table-column>
          <el-table-column prop="orderNo" label="订单编号"></el-table-column>
          <el-table-column prop="customer" label="客户名称"></el-table-column>
          <el-table-column prop="contact" label="联系人" width="100"></el-table-column>
          <el-table-column prop="phone" label="联系电话" width="120"></el-table-column>
          <el-table-column prop="status" label="订单状态" width="100">
            <template #default="scope">
              <el-tag :type="getStatusTagType(scope.row.status)" size="small">
                {{ getStatusText(scope.row.status) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="createTime" label="创建时间" width="180"></el-table-column>
          <el-table-column prop="totalAmount" label="订单金额" width="120" align="right">
            <template #default="scope"> ¥{{ scope.row.totalAmount.toFixed(2) }} </template>
          </el-table-column>
          <el-table-column label="操作" width="180">
            <template #default="scope">
              <el-button type="primary" text size="small" @click="handleOrderView(scope.row)">
                <el-icon><View /></el-icon>
                查看
              </el-button>
              <el-button type="success" text size="small" @click="handleOrderEdit(scope.row)">
                <el-icon><Edit /></el-icon>
                编辑
              </el-button>
              <el-button type="danger" text size="small" @click="handleOrderDelete(scope.row)">
                <el-icon><Delete /></el-icon>
                删除
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
            :total="total"
            @size-change="handleSizeChange"
            @current-change="handleCurrentChange"
          ></el-pagination>
        </div>
      </div>
    </el-card>

    <!-- 订单详情对话框 -->
    <el-dialog v-model="orderDetailVisible" title="订单详情" width="800px">
      <div v-if="selectedOrder" class="order-detail">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="订单基本信息">
            <el-row>
              <el-col :span="12">
                <el-descriptions :column="1" size="small">
                  <el-descriptions-item label="订单编号">{{ selectedOrder.orderNo }}</el-descriptions-item>
                  <el-descriptions-item label="客户名称">{{ selectedOrder.customer }}</el-descriptions-item>
                  <el-descriptions-item label="联系人">{{ selectedOrder.contact }}</el-descriptions-item>
                  <el-descriptions-item label="联系电话">{{ selectedOrder.phone }}</el-descriptions-item>
                </el-descriptions>
              </el-col>
              <el-col :span="12">
                <el-descriptions :column="1" size="small">
                  <el-descriptions-item label="订单状态"
                    ><el-tag :type="getStatusTagType(selectedOrder.status)">{{
                      getStatusText(selectedOrder.status)
                    }}</el-tag></el-descriptions-item
                  >
                  <el-descriptions-item label="创建时间">{{ selectedOrder.createTime }}</el-descriptions-item>
                  <el-descriptions-item label="订单金额"
                    >¥{{ selectedOrder.totalAmount.toFixed(2) }}</el-descriptions-item
                  >
                  <el-descriptions-item label="支付状态"
                    ><el-tag :type="selectedOrder.paid ? 'success' : 'warning'">{{
                      selectedOrder.paid ? '已支付' : '未支付'
                    }}</el-tag></el-descriptions-item
                  >
                </el-descriptions>
              </el-col>
            </el-row>
          </el-descriptions-item>
          <el-descriptions-item label="订单内容">
            <el-table :data="selectedOrder.items" stripe size="small" style="margin-top: 10px">
              <el-table-column prop="name" label="商品名称"></el-table-column>
              <el-table-column prop="quantity" label="数量" width="80" align="center"></el-table-column>
              <el-table-column prop="unitPrice" label="单价" width="100" align="right">
                <template #default="scope">¥{{ scope.row.unitPrice.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column prop="total" label="小计" width="100" align="right">
                <template #default="scope">¥{{ scope.row.total.toFixed(2) }}</template>
              </el-table-column>
            </el-table>
          </el-descriptions-item>
          <el-descriptions-item label="物流信息" v-if="selectedOrder.logistics">
            <el-descriptions :column="1" size="small">
              <el-descriptions-item label="物流公司">{{ selectedOrder.logistics.company }}</el-descriptions-item>
              <el-descriptions-item label="物流单号">{{ selectedOrder.logistics.trackingNo }}</el-descriptions-item>
              <el-descriptions-item label="物流状态"
                ><el-tag type="info">{{ selectedOrder.logistics.status }}</el-tag></el-descriptions-item
              >
            </el-descriptions>
          </el-descriptions-item>
          <el-descriptions-item label="备注">
            {{ selectedOrder.remark || '无' }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="orderDetailVisible = false">关闭</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 订单编辑对话框 -->
    <el-dialog v-model="orderDialogVisible" :title="orderForm.id ? '编辑订单' : '新建订单'" width="600px">
      <el-form :model="orderForm" label-position="right" label-width="100px" size="small">
        <el-form-item label="订单编号" prop="orderNo">
          <el-input v-model="orderForm.orderNo" placeholder="请输入订单编号" :disabled="!!orderForm.id"></el-input>
        </el-form-item>
        <el-form-item label="客户名称" prop="customer">
          <el-input v-model="orderForm.customer" placeholder="请输入客户名称"></el-input>
        </el-form-item>
        <el-form-item label="联系人" prop="contact">
          <el-input v-model="orderForm.contact" placeholder="请输入联系人"></el-input>
        </el-form-item>
        <el-form-item label="联系电话" prop="phone">
          <el-input v-model="orderForm.phone" placeholder="请输入联系电话"></el-input>
        </el-form-item>
        <el-form-item label="订单状态" prop="status">
          <el-select v-model="orderForm.status" placeholder="请选择订单状态">
            <el-option label="待处理" value="pending"></el-option>
            <el-option label="进行中" value="processing"></el-option>
            <el-option label="已完成" value="completed"></el-option>
            <el-option label="已取消" value="cancelled"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="支付状态" prop="paid">
          <el-switch v-model="orderForm.paid" active-text="已支付" inactive-text="未支付"></el-switch>
        </el-form-item>
        <el-form-item label="备注" prop="remark">
          <el-input v-model="orderForm.remark" type="textarea" rows="3" placeholder="请输入备注信息"></el-input>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="orderDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="handleOrderSave">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Plus, Edit, Delete, Search, Refresh, View } from '@element-plus/icons-vue';
import api from '@/api';
import type { OrderQueryForm, Order, OrderForm, OrderStatus, StatusMap, StatusTagTypeMap } from '@/types/order';

// 加载状态
const loading = ref(false);

// 查询表单
const queryForm = ref<OrderQueryForm>({
  orderNo: '',
  customer: '',
  status: '',
  dateRange: [],
});

// 订单列表数据
const orderList = ref<Order[]>([]);

// 分页相关
const currentPage = ref(1);
const pageSize = ref(10);
const total = ref(0);

// 计算过滤后的订单列表 - 现在直接返回后端返回的数据
// 过滤逻辑已移至后端 API
const filteredOrderList = computed(() => {
  return orderList.value;
});

// 对话框状态
const orderDialogVisible = ref(false);
const orderDetailVisible = ref(false);

// 订单表单数据
const orderForm = ref<OrderForm>({
  id: 0,
  orderNo: '',
  customer: '',
  contact: '',
  phone: '',
  status: 'pending',
  paid: false,
  remark: '',
});

// 选中的订单
const selectedOrder = ref<Order | null>(null);

// 获取订单状态文本
const getStatusText = (status: OrderStatus) => {
  const statusMap: StatusMap = {
    pending: '待处理',
    processing: '进行中',
    completed: '已完成',
    cancelled: '已取消',
  };
  return statusMap[status] || status;
};

// 获取订单状态标签类型
const getStatusTagType = (status: OrderStatus) => {
  const typeMap: StatusTagTypeMap = {
    pending: 'warning',
    processing: 'primary',
    completed: 'success',
    cancelled: 'danger',
  };
  return typeMap[status] || 'info';
};

// 后端订单数据类型
// @ts-ignore
interface BackendOrder {
  order_id: number;
  order_no: string;
  customer_name: string;
  customer_phone: string;
  order_status: number;
  order_amount: number;
  create_time: string;
}

// 从后端获取订单数据
const fetchOrders = async () => {
  loading.value = true;
  try {
    // 状态映射：前端字符串 -> 后端数字
    const statusMap = {
      pending: 1,
      processing: 2,
      completed: 3,
      cancelled: 4,
    };

    const params = {
      page: currentPage.value,
      page_size: pageSize.value,
      order_no: queryForm.value.orderNo,
      customer_name: queryForm.value.customer,
      status: queryForm.value.status ? statusMap[queryForm.value.status as keyof typeof statusMap] : undefined,
    };
    const response = await api.get('/api/orders', { params });

    if (response && response.data && response.data.list) {
      orderList.value = (response.data.list || []).map((item: any) => ({
        id: item.order_id,
        orderNo: item.order_no,
        customer: item.customer_name,
        contact: item.customer_phone,
        phone: item.customer_phone,
        status:
          item.order_status === 1
            ? 'pending'
            : item.order_status === 2
              ? 'processing'
              : item.order_status === 3
                ? 'completed'
                : 'cancelled',
        totalAmount: item.order_amount,
        createTime: item.create_time,
        paid: false,
      }));
      // 使用后端返回的总数
      total.value = response.data.total || response.data.list.length || 0;
    }
  } catch (error) {
    console.error('获取订单列表失败:', error);
    ElMessage.error('获取订单列表失败');
  } finally {
    loading.value = false;
  }
};

// 查询操作
const handleQuery = () => {
  currentPage.value = 1;
  fetchOrders();
};

// 重置操作
const handleReset = () => {
  queryForm.value = {
    orderNo: '',
    customer: '',
    status: '',
    dateRange: [],
  };
  currentPage.value = 1;
  fetchOrders();
};

// 分页大小变化
const handleSizeChange = (size: number) => {
  pageSize.value = size;
  currentPage.value = 1;
  fetchOrders();
};

// 当前页变化
const handleCurrentChange = (page: number) => {
  currentPage.value = page;
  fetchOrders();
};

// 添加订单
const handleOrderAdd = () => {
  orderForm.value = {
    id: 0,
    orderNo: `ORD${new Date().getFullYear()}${String(new Date().getMonth() + 1).padStart(2, '0')}${String(new Date().getDate()).padStart(2, '0')}${String(orderList.value.length + 1).padStart(3, '0')}`,
    customer: '',
    contact: '',
    phone: '',
    status: 'pending',
    paid: false,
    remark: '',
  };
  orderDialogVisible.value = true;
};

// 编辑订单
const handleOrderEdit = async (row: Order) => {
  try {
    const response = await api.get(`/api/orders/${row.id}`);
    if (response && response.data) {
      orderForm.value = {
        id: response.data.order_id,
        orderNo: response.data.order_no,
        customer: response.data.customer_name,
        contact: response.data.customer_phone,
        phone: response.data.customer_phone,
        status: response.data.order_status === 1 ? 'pending' : response.data.order_status === 2 ? 'processing' : response.data.order_status === 3 ? 'completed' : 'cancelled',
        paid: false,
        remark: response.data.remark || ''
      };
      orderDialogVisible.value = true;
    }
  } catch (error) {
    console.error('获取订单详情失败:', error);
    ElMessage.error('获取订单详情失败');
  }
};

// 查看订单详情
const handleOrderView = async (row: Order) => {
  try {
    const response = await api.get(`/api/orders/${row.id}`);
    if (response && response.data) {
      selectedOrder.value = {
        id: response.data.order_id,
        orderNo: response.data.order_no,
        customer: response.data.customer_name,
        contact: response.data.customer_phone,
        phone: response.data.customer_phone,
        status: response.data.order_status === 1 ? 'pending' : response.data.order_status === 2 ? 'processing' : response.data.order_status === 3 ? 'completed' : 'cancelled',
        totalAmount: response.data.order_amount,
        createTime: response.data.create_time,
        paid: false,
        remark: response.data.remark || '',
        items: []
      };
      orderDetailVisible.value = true;
    }
  } catch (error) {
    console.error('获取订单详情失败:', error);
    ElMessage.error('获取订单详情失败');
  }
};

// 删除订单
const handleOrderDelete = async (row: Order) => {
  try {
    await api.delete(`/api/orders/${row.id}`);
    orderList.value = orderList.value.filter((item) => item.id !== row.id);
    ElMessage.success('订单删除成功');
  } catch (error) {
    console.error('删除订单失败:', error);
    ElMessage.error('删除订单失败');
  }
};

// 保存订单
const handleOrderSave = async () => {
  try {
    if (orderForm.value.id) {
      // 编辑
      await api.put(`/api/orders/${orderForm.value.id}`, orderForm.value);
      ElMessage.success('订单更新成功');
    } else {
      // 新增
      await api.post('/api/orders', orderForm.value);
      ElMessage.success('订单创建成功');
    }
    orderDialogVisible.value = false;
    fetchOrders();
  } catch (error) {
    console.error('保存订单失败:', error);
    ElMessage.error('保存订单失败');
  }
};

onMounted(() => {
  fetchOrders();
});
</script>

<style scoped>
.order-management {
  padding: 20px;
}

.order-header {
  margin-bottom: 20px;
}

.order-header h2 {
  margin: 0;
  font-size: 20px;
  color: #303133;
}

.query-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.order-list {
  margin-top: 10px;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

.order-detail {
  padding: 10px 0;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>


