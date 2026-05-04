import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router';
import Layout from '@/layout/MainLayout.vue';
import { isRemoteOpsEnabled } from '@/utils/env';
import { isAuthenticated, redirectToLogin } from '@/services/authService';

function isLoginPage(): boolean {
  return window.location.hash === '#/login' || window.location.hash.startsWith('#/login?');
}

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
      {
        path: 'home',
        name: 'Home',
        component: () => import('@/views/HomeScreen.vue'),
      },
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
      {
        path: 'alarm-center',
        name: 'AlarmCenter',
        component: () => import('@/views/AlarmCenter.vue'),
      },
      {
        path: 'supervision/track-playback',
        name: 'TrackPlayback',
        component: () => import('@/views/TrackPlayback.vue'),
      },
      {
        path: 'supervision/video-center',
        name: 'VideoCenter',
        component: () => import('@/views/VideoCenter.vue'),
      },
      {
        path: 'supervision/load-analysis',
        name: 'LoadAnalysis',
        component: () => import('@/views/LoadAnalysis.vue'),
      },
      {
        path: 'supervision/calibration',
        name: 'Calibration',
        component: () => import('@/views/Calibration.vue'),
      },
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
      {
        path: 'settings',
        name: 'Settings',
        component: () => import('@/components/settings/SettingsContainer.vue'),
        redirect: { name: 'SystemSettings' },
        children: [
          {
            path: 'general',
            name: 'SystemSettings',
            component: () => import('@/components/settings/SystemSettings.vue'),
          },
          {
            path: 'monitor',
            name: 'ServiceMonitor',
            component: () => import('@/components/settings/ServiceMonitor.vue'),
          },
          ...(isRemoteOpsEnabled() ? [{
            path: 'ansible-ops',
            name: 'AnsibleOps',
            component: () => import('@/views/ansible-ops/AnsibleOps.vue'),
          }] : []),
          {
            path: 'organization',
            name: 'OrgSettings',
            component: () => import('@/views/organization/OrganizationSettings.vue'),
          },
          {
            path: 'group-template',
            name: 'GroupTemplateConfig',
            component: () => import('@/views/organization/GroupTemplateConfig.vue'),
          },
        ],
      },
      {
        path: 'settings/location',
        name: 'LocationSettings',
        component: () => import('@/views/Location.vue'),
      },
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
        path: 'settings/devices/calibration',
        name: 'CalibrationSettings',
        component: () => import('@/views/Calibration.vue'),
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
      {
        path: 'settings/disaster-recovery',
        name: 'DisasterRecovery',
        component: () => import('@/views/DisasterRecovery.vue'),
      },
      {
        path: 'unified-dispatch',
        name: 'UnifiedDispatch',
        component: () => import('@/views/UnifiedDispatch.vue'),
      },
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

router.beforeEach((to, _from, next) => {
  if (isAuthenticated()) {
    if (to.path === '/login') {
      next({ path: '/home' });
    } else {
      next();
    }
    return;
  }

  if (to.path === '/login') {
    next();
    return;
  }

  // 未认证且非登录页，重定向到登录页（使用 next 不刷新页面）
  next({
    path: '/login',
    query: { redirect: to.fullPath },
  });
});

export default router;
