import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';
import Layout from '@/layout/MainLayout.vue';

// 路由守卫白名单
const whiteList = ['/login'];

// 从 cookie 中获取 token
const getTokenFromCookie = (name: string): string | null => {
  const cookieValue = document.cookie
    .split('; ') 
    .find(row => row.startsWith(`${name}=`))
    ?.split('=')[1];
  return cookieValue ? decodeURIComponent(cookieValue) : null;
};

// 检查是否已认证
const isAuthenticated = (): boolean => {
  // 检查 auth_check cookie
  const authedCookie = getTokenFromCookie('auth_check');
  if (authedCookie === '1') {
    return true;
  }
  
  // 检查 localStorage 中的 token
  const token = localStorage.getItem('access_token') || sessionStorage.getItem('access_token');
  return !!token;
};

const routes = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/Login.vue'),
  },
  {
    path: '/',
    component: Layout,
    redirect: '/home',
    children: [
      // 首页
      {
        path: 'home',
        name: 'Home',
        component: () => import('@/views/HomeScreen.vue'),
      },
      // 实时监控
      {
        path: 'map-window',
        name: 'MapWindow',
        component: () => import('@/views/MapWindow.vue'),
      },
      {
        path: 'realtime',
        name: 'RealTimeMonitor',
        component: () => import('@/views/RealTimeMonitor.vue'),
      },
      // 仪表盘
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/views/Dashboard.vue'),
      },
      {
        path: 'safety-dashboard',
        name: 'SafetyDashboard',
        component: () => import('@/views/monitoring/SafetyMonitorDashboard.vue'),
      },
      {
        path: 'global-dashboard',
        name: 'GlobalDashboard',
        component: () => import('@/views/monitoring/GlobalSafetyDashboard.vue'),
      },
      // 业务管理
      {
        path: 'business/orders',
        name: 'Orders',
        component: () => import('@/views/Order.vue'),
      },
      {
        path: 'business/vehicles',
        name: 'VehicleManage',
        component: () => import('@/views/VehicleManage.vue'),
      },
      {
        path: 'business/drivers',
        name: 'Driver',
        component: () => import('@/views/Driver.vue'),
      },
      {
        path: 'business/finance',
        name: 'Finance',
        component: () => import('@/views/Finance.vue'),
      },
      {
        path: 'business/logistics',
        name: 'Logistics',
        component: () => import('@/views/Logistics.vue'),
      },
      // 处警中心
      {
        path: 'alarm-center',
        name: 'AlarmCenter',
        component: () => import('@/views/AlarmCenter.vue'),
      },
      // 监管中心
      {
        path: 'supervision/track-playback',
        name: 'TrackPlayback',
        component: () => import('@/views/TrackPlayback.vue'),
      },
      // 数据报表
      {
        path: 'reports',
        name: 'Reports',
        component: () => import('@/views/Reports.vue'),
      },
      {
        path: 'reports/status',
        name: 'StatusReports',
        component: () => import('@/views/StatusQuery.vue'),
      },
      {
        path: 'history',
        name: 'HistoryData',
        component: () => import('@/views/HistoryData.vue'),
      },
      // 系统设置
      {
        path: 'settings',
        name: 'Settings',
        component: () => import('@/views/Settings.vue'),
      },
      {
        path: 'settings/disaster-recovery',
        name: 'DisasterRecovery',
        component: () => import('@/views/DisasterRecovery.vue'),
      },
      {
        path: 'settings/communication',
        name: 'CommunicationSettings',
        component: () => import('@/views/CommunicationSettings.vue'),
      },
      {
        path: 'settings/organization/general',
        name: 'OrganizationSettings',
        component: () => import('@/views/organization/OrganizationSettings.vue'),
      },
      {
        path: 'settings/location',
        name: 'LocationSettings',
        component: () => import('@/views/Location.vue'),
      },
      // 设备管理
      {
        path: 'settings/devices/terminal-tools',
        name: 'TerminalTools',
        component: () => import('@/views/device/TerminalTool.vue'),
      },
      {
        path: 'settings/devices/vehicles',
        name: 'VehicleManageSettings',
        component: () => import('@/views/VehicleManage.vue'),
      },
      {
        path: 'settings/organization/units',
        name: 'OrganizationUnitManage',
        component: () => import('@/views/organization/OrganizationUnitManage.vue'),
      },
      {
        path: 'settings/organization/departments',
        name: 'DepartmentManage',
        component: () => import('@/views/organization/DepartmentManage.vue'),
      },
      {
        path: 'settings/organization/roles',
        name: 'RoleManage',
        component: () => import('@/views/organization/RoleManage.vue'),
      },
      {
        path: 'settings/organization/users',
        name: 'UserManage',
        component: () => import('@/views/organization/UserManage.vue'),
      },
      {
        path: 'settings/organization/vehicle-teams',
        name: 'VehicleTeamManage',
        component: () => import('@/views/organization/VehicleTeamManage.vue'),
      },
      // 其他
      {
        path: 'help',
        name: 'Help',
        component: () => import('@/views/Help.vue'),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes: routes as RouteRecordRaw[],
});

// 路由守卫
router.beforeEach((to, _from, next) => {
  if (isAuthenticated()) {
    if (to.path === '/login') {
      // 已登录用户访问登录页，重定向到首页
      next({ path: '/home' });
    } else {
      next();
    }
  } else {
    if (whiteList.indexOf(to.path) !== -1) {
      // 在白名单中，直接访问
      next();
    } else {
      // 不在白名单中，重定向到登录页
      next(`/login?redirect=${to.path}`);
    }
  }
});

export default router;


