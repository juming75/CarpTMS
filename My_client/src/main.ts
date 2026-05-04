import { createApp } from 'vue';
import { createPinia } from 'pinia';
import ElementPlus from 'element-plus';
import 'element-plus/dist/index.css';
import zhCn from 'element-plus/dist/locale/zh-cn.mjs';
import * as ElementPlusIconsVue from '@element-plus/icons-vue';

import App from './App.vue';
import router from './router';
import { syncService } from './services/syncService';
import { initializeApp } from './services/appInitializer';
import { initPerformanceMonitoring } from './utils/performanceMonitor';
import logger from './utils/logger';
import { backendMonitor } from './services/backendMonitor';

const app = createApp(App);

// 判断是否为 ResizeObserver 无害错误
function isResizeObserverLoopError(err: unknown): boolean {
  if (err instanceof Error) {
    const msg = err.message?.toLowerCase() || '';
    return msg.includes('resizeobserver') && msg.includes('loop completed with undelivered notifications');
  }
  if (typeof err === 'string') {
    return err.toLowerCase().includes('resizeobserver');
  }
  return false;
}

app.config.errorHandler = (err, instance, info) => {
  if (isResizeObserverLoopError(err)) return;
  logger.error('Vue Runtime Error:', err);
  logger.error('Component Name:', instance?.$options?.name || instance?.$options?.__name || 'Unknown');
  logger.error('Current Route:', window.location.hash || router.currentRoute.value.path);
  logger.error('Error Info:', info);
};

window.addEventListener('unhandledrejection', (event) => {
  if (isResizeObserverLoopError(event.reason)) {
    event.preventDefault();
    return;
  }
  logger.error('Unhandled Promise Rejection:', event.reason);
});

window.addEventListener('error', (event) => {
  if (isResizeObserverLoopError(event.error || event.message)) {
    event.preventDefault();
    return;
  }
  logger.error('Global JavaScript Error:', event.error || event.message);
  logger.error('Location:', event.filename, 'Line:', event.lineno, 'Col:', event.colno);
});

for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component);
}

app.use(createPinia());
app.use(router);
app.use(ElementPlus, {
  locale: zhCn,
});

initializeApp(app)
  .then(() => {
    initPerformanceMonitoring();
  })
  .catch((error) => {
    console.error('应用初始化失败:', error);
  });

export { syncService };
