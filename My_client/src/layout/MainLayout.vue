<template>
  <el-container class="main-layout">
    <!-- 顶部导航栏 -->
    <el-header class="top-header">
      <div class="header-content">
        <!-- Logo -->
        <div class="logo">
          <el-icon class="logo-icon"><Van /></el-icon>
          <span class="logo-text">CarpTMS</span>
        </div>

        <!-- 顶部一级菜单 -->
        <el-menu
          :default-active="currentRoute"
          class="top-menu"
          @select="handleMenuSelect"
          mode="horizontal"
          background-color="#fff"
          text-color="#303133"
          active-text-color="#409eff"
          :collapse-transition="false"
        >
          <!-- 首页 -->
          <el-menu-item index="/map-window">
            <el-icon><Monitor /></el-icon>
            <span>首页</span>
          </el-menu-item>

          <!-- 数据大屏 -->
          <el-sub-menu index="realtime-group">
            <template #title>
              <el-icon><Monitor /></el-icon>
              <span>数据大屏</span>
            </template>
            <el-menu-item index="/realtime">
              <span>货运运输数据大屏</span>
            </el-menu-item>
            <el-menu-item index="/safety-dashboard">
              <span>企业运营安全大屏</span>
            </el-menu-item>
            <el-menu-item index="/global-dashboard">
              <span>全域安全数据大屏</span>
            </el-menu-item>
            <el-menu-item index="/dashboard">
              <span>业务管理数据大屏</span>
            </el-menu-item>
          </el-sub-menu>

          <!-- 业务管理 -->
          <el-sub-menu index="business-group">
            <template #title>
              <el-icon><Operation /></el-icon>
              <span>业务管理</span>
            </template>
            <el-menu-item index="/business/orders">
              <span>订单管理</span>
            </el-menu-item>
            <el-menu-item index="/business/logistics">
              <span>物流跟踪</span>
            </el-menu-item>
            <el-menu-item index="/business/drivers">
              <span>司机管理</span>
            </el-menu-item>
            <el-menu-item index="/business/finance">
              <span>财务管理</span>
            </el-menu-item>
          </el-sub-menu>

          <!-- 处警中心 -->
          <el-menu-item index="/alarm-center">
            <el-icon><WarningFilled /></el-icon>
            <span>处警中心</span>
          </el-menu-item>

          <!-- 监管中心 -->
          <el-sub-menu index="supervision-group">
            <template #title>
              <el-icon><Monitor /></el-icon>
              <span>监管中心</span>
            </template>
            <el-menu-item index="/supervision/track-playback">
              <span>轨迹回放</span>
            </el-menu-item>
            <el-menu-item index="/supervision/video-center">
              <span>视频中心</span>
            </el-menu-item>
          </el-sub-menu>

          <!-- 数据报表 -->
          <el-sub-menu index="reports-group">
            <template #title>
              <el-icon><Document /></el-icon>
              <span>数据报表</span>
            </template>
            <el-menu-item index="/reports">
              <span>报表中心</span>
            </el-menu-item>
            <el-menu-item index="/reports/status">
              <span>状态查询</span>
            </el-menu-item>
          </el-sub-menu>

          <!-- 系统设置 -->
          <el-sub-menu index="settings-group">
            <template #title>
              <el-icon><Setting /></el-icon>
              <span>系统设置</span>
            </template>
            <!-- 位置管理（直接打开标签页，去掉子菜单） -->
            <el-menu-item index="/settings/location">
              <span>位置管理</span>
            </el-menu-item>
            <!-- 设备管理 -->
            <el-sub-menu index="/settings/devices">
              <template #title>
                <span>设备管理</span>
              </template>
              <el-menu-item index="/settings/devices/vehicles">
                <span>车辆管理</span>
              </el-menu-item>
              <el-menu-item index="/settings/devices/terminal-tools">
                <span>终端工具</span>
              </el-menu-item>
              <el-menu-item index="/settings/devices/calibration">
                <span>标定管理</span>
              </el-menu-item>
            </el-sub-menu>
            <!-- 组织机构 -->
            <el-sub-menu index="/settings/organization">
              <template #title>
                <span>组织机构</span>
              </template>
              <el-menu-item index="/settings/organization/users">
                <span>用户管理</span>
              </el-menu-item>
              <el-menu-item index="/settings/organization/roles">
                <span>角色管理</span>
              </el-menu-item>
              <el-menu-item index="/settings/organization/departments">
                <span>部门管理</span>
              </el-menu-item>
              <el-menu-item index="/settings/organization/vehicle-teams">
                <span>车队管理</span>
              </el-menu-item>
              <el-menu-item index="/settings/organization/units">
                <span>组织单位</span>
              </el-menu-item>
            </el-sub-menu>
            <!-- 灾备管理 -->
            <el-menu-item index="/settings/disaster-recovery">
              <span>灾备管理</span>
            </el-menu-item>
            <!-- 基础设置（带标签页） -->
            <el-menu-item index="/settings">
              <span>基础设置</span>
            </el-menu-item>
          </el-sub-menu>

          <!-- 帮助 -->
          <el-menu-item index="/help">
            <el-icon><Document /></el-icon>
            <span>帮助</span>
          </el-menu-item>
        </el-menu>

          <!-- 用户信息和操作 -->
        <div class="header-right">
          <el-tooltip content="刷新数据" placement="bottom">
            <el-button circle @click="handleRefresh">
              <el-icon><Refresh /></el-icon>
            </el-button>
          </el-tooltip>
          <el-dropdown>
            <div class="user-dropdown">
              <el-avatar :size="32" :src="userAvatar" />
              <span class="username">{{ username }}</span>
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item>个人信息</el-dropdown-item>
                <el-dropdown-item divided @click="handleUserMenuLogout">退出登录</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
    </el-header>

    <!-- 面包屑导航 -->
    <el-breadcrumb class="breadcrumb" separator-class="el-icon-arrow-right">
      <el-breadcrumb-item v-for="(item, index) in breadcrumbItems" :key="index">
        <router-link v-if="item.path" :to="item.path">{{ item.title }}</router-link>
        <span v-else>{{ item.title }}</span>
      </el-breadcrumb-item>
    </el-breadcrumb>

    <!-- 主内容区 -->
    <el-main class="main-content">
      <router-view />
    </el-main>

    <!-- 页脚信息（含 WebSocket 状态） -->
    <el-footer height="36px" class="layout-footer">
      <div class="footer-content">
        <span>© 2026 CarpTMS - 车联网运输管理系统 | 版本: 1.1.0</span>
        <span class="footer-divider">|</span>
        <!-- WebSocket 连接状态 -->
        <span class="ws-inline-status">
          <span class="ws-dot" :class="wsStatusClass" :title="wsStatusText">{{ wsStatusIcon }}</span>
          <span :class="['ws-label', wsStatusClass]">{{ wsStatusText }}</span>
        </span>
        <span class="footer-divider">|</span>
        <!-- 心跳状态 -->
        <span class="ws-hb-inline">
          <span>❤️ 心跳:</span>
          <span>↑{{ wsHbPings }}</span>/<span class="hb-success">↓{{ wsHbPongs }}</span>
          <span v-if="wsHbFailures > 0" class="hb-error">✕{{ wsHbFailures }}</span>
          <span :class="['hb-quality', wsHbQualityClass]">{{ wsHbQuality }}%</span>
        </span>
        <el-button text size="small" class="footer-toggle-btn" @click="toggleWsCollapse">
          {{ showWsDetails ? '收起' : '展开' }}
        </el-button>
      </div>

      <!-- 展开时的详细状态行 -->
      <div v-if="showWsDetails" class="footer-detail-row">
        <span>消息: <strong>{{ wsMessageCount }}</strong></span>
        <span class="detail-sep">|</span>
        <span>队列: <strong :class="{ 'text-danger': wsQueueLength > 0 }">{{ wsQueueLength }}</strong></span>
        <span class="detail-sep">|</span>
        <span>运行: <strong>{{ wsUptimeFormatted }}</strong></span>
        <span class="detail-sep">|</span>
        <span>重连: <strong>{{ wsReconnectAttempts }}</strong></span>
      </div>
    </el-footer>
  </el-container>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { ElMessage } from 'element-plus';
import { Van, Monitor, Document, Setting, Refresh, Operation, WarningFilled, Tools } from '@element-plus/icons-vue';
import { useAuthStore } from '@/stores/useAuthStore';
import { isRemoteOpsEnabled } from '@/utils/env';
import {
  getUnifiedCommunicationService,
} from '@/services/unifiedCommunicationService';

// 动态导入API
interface AuthApi {
  login: (username: string, password: string) => Promise<any>;
  logout: () => Promise<any>;
  getCurrentUser: (id: number) => Promise<any>;
}

let authApi: AuthApi | null = null;
async function importApi() {
  if (!authApi) {
    const module = await import('@/api');
    authApi = module.authApi;
  }
  return authApi;
}

const router = useRouter();
const route = useRoute();
const authStore = useAuthStore();

const username = ref('Admin');
const userAvatar = ref('');
const userId = ref(0);

// WebSocket 内联状态（页脚显示）
const showWsDetails = ref(false);
const wsStats = reactive({
  state: 'disconnected' as string,
  isConnected: false,
  reconnectAttempts: 0,
});
const wsHbPings = ref(0);
const wsHbPongs = ref(0);
const wsHbFailures = ref(0);
const wsHbQuality = ref('100');
const wsMessageCount = ref(0);
const wsUptime = ref(0); // 秒
const wsQueueLength = ref(0);
let wsUptimeTimer: number | null = null;

// WebSocket 计算属性
const wsStatusClass = computed(() => {
  if (wsStats.isConnected) return 'ws-connected';
  if (wsStats.state === 'connecting' || wsStats.state === 'reconnecting') return 'ws-connecting';
  return 'ws-disconnected';
});

const wsStatusText = computed(() => {
  const m: Record<string, string> = { connected: '已连接', connecting: '连接中', reconnecting: '重连中', disconnected: '已断开', error: '错误' };
  return m[wsStats.state] || wsStats.state;
});

const wsStatusIcon = computed(() => ({ connected: '●', connecting: '◐', reconnecting: '◑', disconnected: '○', error: '✕' }[wsStats.state] || '○'));

const wsHbQualityClass = computed(() => {
  const q = parseFloat(wsHbQuality.value);
  return q >= 90 ? 'hb-q-good' : q >= 70 ? 'hb-q-warn' : 'hb-q-bad';
});

const wsUptimeFormatted = computed(() => {
  const s = wsUptime.value;
  if (s < 60) return `${s}秒`;
  if (s < 3600) return `${Math.floor(s / 60)}分${s % 60}秒`;
  return `${Math.floor(s / 3600)}时${Math.floor((s % 3600) / 60)}分`;
});

function toggleWsCollapse() { showWsDetails.value = !showWsDetails.value; }

function refreshWsStatus() {
  const svc = getUnifiedCommunicationService();
  if (!svc) return;
  const s = svc.getStats();
  Object.assign(wsStats, s);
}

function startWsMonitoring() {
  refreshWsStatus();
  const svc = getUnifiedCommunicationService();
  if (!svc) return;

  // 监听连接事件
  svc.on('connected', () => { refreshWsStatus(); });
  svc.on('disconnected', () => { refreshWsStatus(); });
  svc.on('error', () => { refreshWsStatus(); });

  // 监听消息计数
  svc.on('message', () => { wsMessageCount.value++; });

  // 运行时间计时器
  wsUptimeTimer = window.setInterval(() => {
    if (wsStats.isConnected) wsUptime.value++;
  }, 1000);

  // 定期同步心跳统计（从服务获取）
  setInterval(() => {
    const s = svc.getStats();
    if (s) Object.assign(wsStats, s);
  }, 5000);
}

// 面包屑导航数据
const breadcrumbItems = ref<{ title: string; path?: string }[]>([]);

const currentRoute = computed(() => {
  // 构建完整的路由路径（包括查询参数）用于菜单激活
  let fullPath = route.path;
  if (Object.keys(route.query).length > 0) {
    const params = new URLSearchParams(route.query as Record<string, string>);
    fullPath += `?${params.toString()}`;
  }
  return fullPath;
});

// 计算面包屑导航
const updateBreadcrumb = () => {
  const path = route.path;
  const items: { title: string; path?: string }[] = [];
  
  // 首页
  items.push({ title: '首页', path: '/home' });
  
  // 根据路径生成面包屑
  if (path.startsWith('/business/')) {
    items.push({ title: '业务管理', path: '/business' });
    if (path.includes('/business/orders')) {
      items.push({ title: '订单管理' });
    } else if (path.includes('/business/logistics')) {
      items.push({ title: '物流跟踪' });
    } else if (path.includes('/business/drivers')) {
      items.push({ title: '司机管理' });
    } else if (path.includes('/business/finance')) {
      items.push({ title: '财务管理' });
    }
  } else if (path.startsWith('/settings/')) {
    // 判断是否是基础设置页面（SettingsContainer）的子路由
    const settingsChildRoutes = ['SystemSettings', 'ServiceMonitor', 'OrgSettings', 'GroupTemplateConfig', 'AnsibleOps'];
    const isSettingsContainerChild = settingsChildRoutes.includes(route.name as string);

    if (isSettingsContainerChild || path === '/settings') {
      // 基础设置页面及其标签页
      items.push({ title: '系统设置', path: '/settings' });
      items.push({ title: '基础设置' });
      if (route.name === 'ServiceMonitor') {
        items.push({ title: '服务监测' });
      } else if (route.name === 'OrgSettings') {
        items.push({ title: '组织设置' });
      } else if (route.name === 'GroupTemplateConfig') {
        items.push({ title: '组织模板' });
      } else if (route.name === 'AnsibleOps') {
        items.push({ title: '自动化运维' });
      }
    } else {
      // 其他系统设置子页面
      items.push({ title: '系统设置', path: '/settings' });
      if (path.includes('/settings/location')) {
        items.push({ title: '位置管理', path: '/settings/location' });
      } else if (path.includes('/settings/devices')) {
        items.push({ title: '设备管理', path: '/settings/devices' });
        if (path.includes('/settings/devices/vehicles')) {
          items.push({ title: '车辆管理' });
        } else if (path.includes('/settings/devices/terminal-tools')) {
          items.push({ title: '终端工具' });
        }
      } else if (path.includes('/settings/organization')) {
        items.push({ title: '组织机构', path: '/settings/organization' });
        if (path.includes('/settings/organization/users')) {
          items.push({ title: '用户管理' });
        } else if (path.includes('/settings/organization/roles')) {
          items.push({ title: '角色管理' });
        } else if (path.includes('/settings/organization/departments')) {
          items.push({ title: '部门管理' });
        } else if (path.includes('/settings/organization/vehicle-teams')) {
          items.push({ title: '车队管理' });
        } else if (path.includes('/settings/organization/units')) {
          items.push({ title: '组织单位' });
        }
      } else if (path.includes('/settings/disaster-recovery')) {
        items.push({ title: '灾备管理' });
      }
    }
  } else if (path.startsWith('/supervision/')) {
    items.push({ title: '监管中心', path: '/supervision' });
    if (path.includes('/supervision/track-playback')) {
      items.push({ title: '轨迹回放' });
    } else if (path.includes('/supervision/video-center')) {
      items.push({ title: '视频中心' });
    }
  } else if (path.startsWith('/reports/')) {
    items.push({ title: '数据报表', path: '/reports' });
    if (path.includes('/reports/status')) {
      items.push({ title: '状态查询' });
    }
  } else if (path === '/alarm-center') {
    items.push({ title: '处警中心' });
  } else if (path === '/realtime') {
    items.push({ title: '数据大屏', path: '/realtime-group' });
    items.push({ title: '货运运输数据大屏' });
  } else if (path === '/safety-dashboard') {
    items.push({ title: '数据大屏', path: '/realtime-group' });
    items.push({ title: '企业运营安全大屏' });
  } else if (path === '/global-dashboard') {
    items.push({ title: '数据大屏', path: '/realtime-group' });
    items.push({ title: '全域安全数据大屏' });
  } else if (path === '/dashboard') {
    items.push({ title: '数据大屏', path: '/realtime-group' });
    items.push({ title: '业务管理数据大屏' });
  } else if (path === '/help') {
    items.push({ title: '帮助' });
  }
  
  breadcrumbItems.value = items;
};

const handleMenuSelect = async (index: string) => {
  console.log('菜单点击，准备跳转到:', index);
  console.log('当前路由:', route.path);

  // 分离路径和查询参数
  let targetPath = index;
  let query: Record<string, string> = {};
  
  if (index.includes('?')) {
    const [path, queryString] = index.split('?');
    targetPath = path;
    if (queryString) {
      const params = new URLSearchParams(queryString);
      for (const [key, value] of params) {
        query[key] = value;
      }
    }
  }

  // 检查是否是同一页面
  const isSamePage = targetPath === route.path;
  
  // 如果是同一页面但查询参数不同，仍需跳转
  if (isSamePage) {
    const currentQueryString = new URLSearchParams(route.query as Record<string, string>).toString();
    const newQueryString = new URLSearchParams(query).toString();
    if (currentQueryString === newQueryString) {
      console.log('已经在当前页面，无需跳转');
      return;
    }
  }

  try {
    // 构建路由参数
    const routeConfig = Object.keys(query).length > 0 
      ? { path: targetPath, query }
      : targetPath;

    // 使用push进行路由跳转
    await router.push(routeConfig);
    console.log('路由跳转成功:', index);
  } catch (error: unknown) {
    console.error('路由跳转失败:', error);
    // 忽略重复导航错误
    if (error && typeof error === 'object' && 'name' in error && error.name === 'NavigationDuplicated') {
      console.log('重复导航，忽略');
      return;
    }
    // 其他错误显示提示
    if (error && typeof error === 'object' && 'message' in error) {
      ElMessage.error(`跳转失败: ${String(error.message)}`);
    }
  }
};

const fetchCurrentUser = async () => {
  // 从localStorage获取用户ID和token
  const userInfoStr = localStorage.getItem('userInfo');
  let parsedUserId = 0;

  if (userInfoStr) {
    try {
      const userInfo = JSON.parse(userInfoStr);
      console.log('从localStorage获取的用户信息:', userInfo);
      parsedUserId = userInfo.user_id || 1;
      userId.value = parsedUserId;
      username.value = userInfo.username;
    } catch (e) {
      console.error('Failed to parse user info:', e);
      // 如果解析失败，设置默认用户ID
      parsedUserId = 1;
      userId.value = parsedUserId;
    }
  } else {
    // 如果没有userInfo，设置默认用户ID
    parsedUserId = 1;
    userId.value = parsedUserId;
  }

  // 调用API获取用户信息，无论userId是否大于0
  try {
    const authApi = await importApi();
    console.log('调用authApi.getCurrentUser，用户ID:', parsedUserId);
    const response = await authApi.getCurrentUser(parsedUserId);
    console.log('获取用户信息响应:', response);
    const userData = response.data?.data || response;
    if (userData) {
      // 直接使用返回的userData，因为authApi.getCurrentUser现在直接返回data部分
      username.value = userData.username;
      // 保存用户信息到localStorage
      localStorage.setItem('userInfo', JSON.stringify(userData));
      console.log('保存用户信息:', userData);
    }
  } catch (e) {
    console.error('Failed to fetch user info:', e);
  }
};



const handleLogout = async () => {
  try {
    // 调用 API 登出
    const authApi = await importApi();
    await authApi.logout();
  } catch (error) {
    // 忽略登出 API 错误，因为即使 API 失败，我们也需要清除本地状态
    console.warn('登出 API 调用失败，继续清除本地状态:', error);
  }
  
  // 调用 store 中的 logout 函数
  authStore.logout();
  
  // 清除本地存储中的相关数据
  localStorage.removeItem('token');
  localStorage.removeItem('userInfo');
  localStorage.removeItem('access_token');
  localStorage.removeItem('refresh_token');
  localStorage.removeItem('userId');
  
  // 清除 sessionStorage 中的相关数据
  sessionStorage.removeItem('access_token');
  sessionStorage.removeItem('refresh_token');
  
  // 清除自动登录标志，防止立即重新登录
  localStorage.setItem('autoLogin', 'false');
  
  // 清除 cookie
  document.cookie = 'access_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC';
  document.cookie = 'refresh_token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC';
  document.cookie = 'auth_check=; path=/; expires=Thu, 01 Jan 1970 00:00:00 UTC';
  
  ElMessage.success('退出登录成功');
  router.push('/login');
};

const handleRefresh = () => {
  ElMessage.success('数据已刷新');
};

// 监听路由变化，更新面包屑
watch(() => route.path, () => {
  updateBreadcrumb();
}, { immediate: true });

onMounted(async () => {
  // 只有在已登录状态下才调用fetchCurrentUser()函数
  const token = localStorage.getItem('access_token');
  if (token) {
    await fetchCurrentUser();
  }
  // 初始化面包屑
  updateBreadcrumb();
  // 启动 WebSocket 页脚状态监控
  startWsMonitoring();
});

onBeforeUnmount(() => {
  if (wsUptimeTimer) clearInterval(wsUptimeTimer);
});

// 用户下拉菜单的登出处理
const handleUserMenuLogout = () => {
  handleLogout();
};
</script>

<style scoped>
.main-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

/* 顶部导航栏 */
.top-header {
  background: #fff;
  box-shadow: 0 1px 4px rgba(0, 21, 41, 0.08);
  padding: 0;
  height: auto;
  line-height: 60px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  height: 60px;
}

/* Logo */
.logo {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: bold;
  font-size: 18px;
  color: #303133;
}

.logo-icon {
  font-size: 24px;
  color: #409eff;
}

.logo-text {
  font-size: 18px;
  font-weight: bold;
  color: #303133;
}

/* 顶部菜单 */
.top-menu {
  flex: 1;
  margin: 0 20px;
  border: none;
  background: transparent;
  box-shadow: none;
  z-index: 1000;
  position: relative;
}

.top-menu .el-menu-item {
  border-radius: 4px;
  margin: 0 4px;
}

.top-menu .el-menu-item:hover {
  background: rgba(64, 158, 255, 0.1);
}

.top-menu .el-menu-item.is-active {
  background: rgba(64, 158, 255, 0.1);
}

/* 顶部菜单子菜单 */
.top-menu .el-sub-menu .el-menu-item {
  min-width: 180px;
  margin: 0;
}

/* 确保子菜单显示在最上层 */
.top-menu .el-sub-menu .el-menu {
  z-index: 2000;
  position: relative;
}

/* 右侧用户信息和操作 */
.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.server-info {
  color: #606266;
  font-size: 14px;
  margin-right: 16px;
}

.user-dropdown {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.username {
  color: #303133;
  font-size: 14px;
}

/* 主内容区 */
.breadcrumb {
  padding: 10px 24px;
  background: white;
  border-bottom: 1px solid #ebeef5;
  margin: 0;
}

.main-content {
  background: #f0f2f5;
  padding: 20px;
  overflow-y: auto;
  flex: 1;
  width: 100%;
}

/* 页脚（含内联 WebSocket 状态） */
.layout-footer {
  background: #fff;
  border-top: 1px solid #e4e7ed;
  padding: 0 24px;
  display: flex;
  flex-direction: column;
  font-size: 12px;
  color: #909399;
}

.footer-content {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
}

.footer-divider {
  color: #dcdfe6;
  user-select: none;
}

/* WebSocket 内联连接状态 */
.ws-inline-status {
  display: flex;
  align-items: center;
  gap: 3px;
}
.ws-dot { font-size: 11px; }
.ws-connected { color: #67c23a; }
.ws-connecting { color: #e6a23c; }
.ws-disconnected { color: #c0c4cc; }
.ws-label { font-weight: 500; }

/* 心跳内联 */
.ws-hb-inline {
  display: flex;
  align-items: center;
  gap: 2px;
  font-size: 11px;
}
.hb-success { color: #67c23a; font-weight: 600; }
.hb-error { color: #f56c6c; font-weight: 600; }
.hb-quality {
  padding: 0 4px;
  border-radius: 8px;
  font-weight: 600;
  font-size: 10px;
}
.hb-q-good { color: #67c23a; background: #f0f9eb; }
.hb-q-warn { color: #e6a23c; background: #fdf6ec; }
.hb-q-bad  { color: #f56c6c; background: #fef0f0; }

/* 展开/收起按钮 */
.footer-toggle-btn {
  font-size: 10px !important;
  padding: 1px 6px !important;
  color: #909399 !important;
}
.footer-toggle-btn:hover {
  color: #409eff !important;
}

/* 展开时的详情行 */
.footer-detail-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 4px 0;
  border-top: 1px dashed #ebeef5;
  font-size: 11px;
  color: #909399;
}
.detail-sep { color: #dcdfe6; }
.text-danger { color: #f56c6c; }
</style>


