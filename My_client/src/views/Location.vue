<template>
  <div class="location-management">
    <div class="location-header">
      <h2>位置管理</h2>
    </div>

    <div class="location-tabs">
      <el-tabs v-model="activeTab" type="card">
        <!-- 电子围栏 -->
        <el-tab-pane label="电子围栏" name="fence">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>电子围栏管理</span>
                  <el-button type="primary" size="small" @click="handleFenceAdd">
                    <el-icon><Plus /></el-icon> 新建围栏
                  </el-button>
                </div>
              </template>
              <div class="fence-list" v-loading="loading.fence">
                <el-table :data="fenceList" stripe row-key="fence_id">
                  <el-table-column prop="fence_id" label="ID" width="60" />
                  <el-table-column prop="fence_name" label="围栏名称" />
                  <el-table-column prop="fence_type" label="类型" width="100" />
                  <el-table-column prop="status" label="状态" width="80">
                    <template #default="scope">
                      <el-tag :type="scope.row.status === 'active' ? 'success' : 'info'" size="small">
                        {{ scope.row.status === 'active' ? '启用' : '禁用' }}
                      </el-tag>
                    </template>
                  </el-table-column>
                  <el-table-column prop="create_time" label="创建时间" width="180" />
                  <el-table-column label="操作" width="150">
                    <template #default="scope">
                      <el-button type="primary" text size="small" @click="handleFenceEdit(scope.row)">编辑</el-button>
                      <el-button type="danger" text size="small" @click="handleFenceDelete(scope.row)">删除</el-button>
                    </template>
                  </el-table-column>
                </el-table>
              </div>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- 位置编辑 -->
        <el-tab-pane label="位置编辑" name="position">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>位置编辑</span>
                  <el-button type="primary" size="small" @click="handlePositionAdd">
                    <el-icon><Plus /></el-icon> 新建位置
                  </el-button>
                </div>
              </template>
              <div class="position-list" v-loading="loading.position">
                <el-table :data="positionList" stripe row-key="position_id">
                  <el-table-column prop="position_id" label="ID" width="60" />
                  <el-table-column prop="place_name" label="位置名称" />
                  <el-table-column prop="latitude" label="纬度" width="120" />
                  <el-table-column prop="longitude" label="经度" width="120" />
                  <el-table-column prop="address" label="地址" />
                  <el-table-column label="操作" width="150">
                    <template #default="scope">
                      <el-button type="primary" text size="small" @click="handlePositionEdit(scope.row)"
                        >编辑</el-button
                      >
                      <el-button type="danger" text size="small" @click="handlePositionDelete(scope.row)"
                        >删除</el-button
                      >
                    </template>
                  </el-table-column>
                </el-table>
              </div>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- 地点编辑 -->
        <el-tab-pane label="地点编辑" name="place">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>地点编辑</span>
                  <el-button type="primary" size="small" @click="handlePlaceAdd">
                    <el-icon><Plus /></el-icon> 新建地点
                  </el-button>
                </div>
              </template>
              <div class="place-list" v-loading="loading.place">
                <el-table :data="placeList" stripe row-key="place_id">
                  <el-table-column prop="place_id" label="ID" width="60" />
                  <el-table-column prop="place_name" label="地点名称" />
                  <el-table-column prop="address" label="地址" />
                  <el-table-column prop="contact_person" label="联系人" width="100" />
                  <el-table-column prop="contact_phone" label="联系电话" width="120" />
                  <el-table-column label="操作" width="150">
                    <template #default="scope">
                      <el-button type="primary" text size="small" @click="handlePlaceEdit(scope.row)">编辑</el-button>
                      <el-button type="danger" text size="small" @click="handlePlaceDelete(scope.row)">删除</el-button>
                    </template>
                  </el-table-column>
                </el-table>
              </div>
            </el-card>
          </div>
        </el-tab-pane>

        <!-- 路线编辑 -->
        <el-tab-pane label="路线编辑" name="route">
          <div class="tab-content">
            <el-card shadow="hover">
              <template #header>
                <div class="card-header">
                  <span>路线编辑</span>
                  <el-button type="primary" size="small" @click="handleRouteAdd">
                    <el-icon><Plus /></el-icon> 新建路线
                  </el-button>
                </div>
              </template>
              <div class="route-list" v-loading="loading.route">
                <el-table :data="routeList" stripe row-key="route_id">
                  <el-table-column prop="route_id" label="ID" width="60" />
                  <el-table-column prop="route_name" label="路线名称" />
                  <el-table-column prop="start_point" label="起点" />
                  <el-table-column prop="end_point" label="终点" />
                  <el-table-column prop="distance" label="距离 (km)" width="100" />
                  <el-table-column label="操作" width="150">
                    <template #default="scope">
                      <el-button type="primary" text size="small" @click="handleRouteEdit(scope.row)">编辑</el-button>
                      <el-button type="danger" text size="small" @click="handleRouteDelete(scope.row)">删除</el-button>
                    </template>
                  </el-table-column>
                </el-table>
              </div>
            </el-card>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <!-- 对话框组件 -->
    <LocationDialog v-model:visible="dialog.visible" :type="dialog.type" :data="dialog.data" @save="handleDialogSave" />
  </div>
</template>

<script setup lang="ts">
/* global fetch */
import { ref, reactive, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus } from '@element-plus/icons-vue';
import api from '@/api';
import LocationDialog from './components/LocationDialog.vue';

interface FenceItem {
  fence_id: number;
  fence_name: string;
  fence_type: string;
  status: string;
  create_time: string;
}

interface PositionItem {
  position_id: number;
  place_name: string;
  latitude: number;
  longitude: number;
  address: string;
  description: string;
  create_time: string;
  update_time: string | null;
}

interface PlaceItem {
  place_id: number;
  place_name: string;
  address: string;
  contact_person: string;
  contact_phone: string;
}

interface RouteItem {
  route_id: number;
  route_name: string;
  start_point: string;
  end_point: string;
  waypoints: any;
  distance: number;
  estimated_duration: number;
  description: string;
  create_time: string;
  update_time: string | null;
}

// 当前激活的标签页
const activeTab = ref('fence');

// 从 URL 参数中获取 tab 值
const route = useRoute();

// 加载状态
const loading = reactive({
  fence: false,
  position: false,
  place: false,
  route: false,
});

// 数据列表
const fenceList = ref<FenceItem[]>([]);
const positionList = ref<PositionItem[]>([]);
const placeList = ref<PlaceItem[]>([]);
const routeList = ref<RouteItem[]>([]);

// 对话框状态
const dialog = reactive({
  visible: false,
  type: '', // fence, position, place, route
  data: null as any,
});

// API 调用函数
const fetchFences = async () => {
  loading.fence = true;
  try {
    const response = await api.get('/api/location/fences') as any;
    console.log('获取围栏列表响应:', response);
    if (response && response.list) {
      fenceList.value = response.list;
    } else if (response && response.data && response.data.list) {
      fenceList.value = response.data.list;
    } else {
      console.error('获取围栏列表响应格式错误:', response);
      ElMessage.error('获取围栏列表响应格式错误');
    }
  } catch (error) {
    console.error('获取围栏列表失败:', error);
    ElMessage.error('获取围栏列表失败');
  } finally {
    loading.fence = false;
  }
};

const fetchPositions = async () => {
  loading.position = true;
  try {
    const response = await api.get('/api/location/positions') as any;
    console.log('获取位置列表响应:', response);
    if (response && response.list) {
      positionList.value = response.list;
    } else if (response && response.data && response.data.list) {
      positionList.value = response.data.list;
    } else {
      console.error('获取位置列表响应格式错误:', response);
      ElMessage.error('获取位置列表响应格式错误');
    }
  } catch (error) {
    console.error('获取位置列表失败:', error);
    ElMessage.error('获取位置列表失败');
  } finally {
    loading.position = false;
  }
};

const fetchPlaces = async () => {
  loading.place = true;
  try {
    const response = await api.get('/api/location/places') as any;
    console.log('获取地点列表响应:', response);
    if (response && response.list) {
      placeList.value = response.list;
    } else if (response && response.data && response.data.list) {
      placeList.value = response.data.list;
    } else {
      console.error('获取地点列表响应格式错误:', response);
      ElMessage.error('获取地点列表响应格式错误');
    }
  } catch (error) {
    console.error('获取地点列表失败:', error);
    ElMessage.error('获取地点列表失败');
  } finally {
    loading.place = false;
  }
};

const fetchRoutes = async () => {
  loading.route = true;
  try {
    const response = await api.get('/api/location/routes') as any;
    console.log('获取路线列表响应:', response);
    if (response && response.list) {
      routeList.value = response.list;
    } else if (response && response.data && response.data.list) {
      routeList.value = response.data.list;
    } else {
      console.error('获取路线列表响应格式错误:', response);
      ElMessage.error('获取路线列表响应格式错误');
    }
  } catch (error) {
    console.error('获取路线列表失败:', error);
    ElMessage.error('获取路线列表失败');
  } finally {
    loading.route = false;
  }
};

// 处理函数
const handleFenceAdd = () => {
  dialog.type = 'fence';
  dialog.data = null;
  dialog.visible = true;
};

const handleFenceEdit = (row: FenceItem) => {
  dialog.type = 'fence';
  dialog.data = { ...row };
  dialog.visible = true;
};

const handleFenceDelete = async (row: FenceItem) => {
  ElMessageBox.confirm('确定要删除该围栏吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/location/fences/${row.fence_id}`);
        ElMessage.success('删除成功');
        fetchFences();
      } catch (error) {
        console.error('删除围栏失败:', error);
        ElMessage.error('删除围栏失败');
      }
    })
    .catch(() => {});
};

const handlePositionAdd = () => {
  dialog.type = 'position';
  dialog.data = null;
  dialog.visible = true;
};

const handlePositionEdit = (row: PositionItem) => {
  dialog.type = 'position';
  dialog.data = { ...row };
  dialog.visible = true;
};

const handlePositionDelete = async (row: PositionItem) => {
  ElMessageBox.confirm('确定要删除该位置吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/location/positions/${row.position_id}`);
        ElMessage.success('删除成功');
        fetchPositions();
      } catch (error) {
        console.error('删除位置失败:', error);
        ElMessage.error('删除位置失败');
      }
    })
    .catch(() => {});
};

const handlePlaceAdd = () => {
  dialog.type = 'place';
  dialog.data = null;
  dialog.visible = true;
};

const handlePlaceEdit = (row: PlaceItem) => {
  dialog.type = 'place';
  dialog.data = { ...row };
  dialog.visible = true;
};

const handlePlaceDelete = async (row: PlaceItem) => {
  ElMessageBox.confirm('确定要删除该地点吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/location/places/${row.place_id}`);
        ElMessage.success('删除成功');
        fetchPlaces();
      } catch (error) {
        console.error('删除地点失败:', error);
        ElMessage.error('删除地点失败');
      }
    })
    .catch(() => {});
};

const handleRouteAdd = () => {
  dialog.type = 'route';
  dialog.data = null;
  dialog.visible = true;
};

const handleRouteEdit = (row: RouteItem) => {
  dialog.type = 'route';
  dialog.data = { ...row };
  dialog.visible = true;
};

const handleRouteDelete = async (row: RouteItem) => {
  ElMessageBox.confirm('确定要删除该路线吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.delete(`/api/location/routes/${row.route_id}`);
        ElMessage.success('删除成功');
        fetchRoutes();
      } catch (error) {
        console.error('删除路线失败:', error);
        ElMessage.error('删除路线失败');
      }
    })
    .catch(() => {});
};

const handleDialogSave = async (type: string, data: any) => {
  try {
    if (type === 'fence') {
      if (data.fence_id) {
        await api.put(`/api/location/fences/${data.fence_id}`, data);
        ElMessage.success('更新成功');
      } else {
        await api.post('/api/location/fences', data);
        ElMessage.success('创建成功');
      }
      fetchFences();
    } else if (type === 'position') {
      if (data.position_id) {
        await api.put(`/api/location/positions/${data.position_id}`, data);
        ElMessage.success('更新成功');
      } else {
        await api.post('/api/location/positions', data);
        ElMessage.success('创建成功');
      }
      fetchPositions();
    } else if (type === 'place') {
      if (data.place_id) {
        await api.put(`/api/location/places/${data.place_id}`, data);
        ElMessage.success('更新成功');
      } else {
        await api.post('/api/location/places', data);
        ElMessage.success('创建成功');
      }
      fetchPlaces();
    } else if (type === 'route') {
      if (data.route_id) {
        await api.put(`/api/location/routes/${data.route_id}`, data);
        ElMessage.success('更新成功');
      } else {
        await api.post('/api/location/routes', data);
        ElMessage.success('创建成功');
      }
      fetchRoutes();
    }
    dialog.visible = false;
  } catch (error) {
    console.error('保存失败:', error);
    ElMessage.error('保存失败');
  }
};

// 初始化
onMounted(() => {
  // 从 URL 参数中获取 tab 值
  if (route.query.tab) {
    activeTab.value = route.query.tab as string;
  }
  fetchFences();
  fetchPositions();
  fetchPlaces();
  fetchRoutes();
});
</script>

<style scoped>
.location-management {
  padding: 20px;
}

.location-header h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.location-tabs {
  margin-top: 20px;
}

.tab-content {
  min-height: 400px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.map-settings-form {
  max-width: 600px;
  padding: 20px;
}

.form-tip {
  margin-top: 8px;
  font-size: 12px;
}

.map-test-section {
  margin-top: 20px;
}

.map-test-section h4 {
  margin-bottom: 16px;
  color: #303133;
}

.test-buttons {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.test-result {
  margin-top: 16px;
}
</style>


