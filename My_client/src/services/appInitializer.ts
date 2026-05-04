import type { App } from 'vue';
import { syncService } from './syncService';
import { initForNewServer, initForLegacyServer } from './unifiedCommunicationService';
import { wrapAsync, wrapSync, getUserMessage } from './errorHandler';
import monitoringService from './monitoring';
import { useServiceStore } from '@/stores/useServiceStore';
import logger from '@/utils/logger';

interface ServerConfig {
  hostname: string;
  port: number;
  isDev: boolean;
}

function getServerConfig(): ServerConfig {
  const isDev = import.meta.env.DEV;
  const hostname = isDev ? 'localhost' : window.location.hostname;
  const port = isDev
    ? 8082
    : window.location.port
    ? parseInt(window.location.port)
    : window.location.protocol === 'https:'
    ? 443
    : 80;
  return { hostname, port, isDev };
}

export function initializeServerConfig(): void {
  const { hostname, port, isDev } = getServerConfig();
  const serverIp = hostname;
  const serverPort = String(port);

  localStorage.setItem('serverIp', serverIp);
  localStorage.setItem('serverPort', serverPort);
  localStorage.setItem('newServerIp', serverIp);
  localStorage.setItem('newServerPort', serverPort);

  if (isDev) {
    logger.info('服务器配置已初始化:', { mode: '开发环境', serverIp, serverPort });
  }
}

async function initializeCoreServices(): Promise<void> {
  const serviceStore = useServiceStore();

  const newServerResult = await wrapAsync(async () => {
    const { hostname, port } = getServerConfig();
    const newService = initForNewServer(hostname, port, 'auto');
    await newService.connect();
    await wrapAsync(async () => {
      await newService.send({ type: 'command', command: 'ping', payload: {} });
      return true;
    }, '新服务器消息测试');
    newService.on('message', () => {});
    serviceStore.setNewServerService(newService);
    return newService;
  }, '初始化统一通信服务（新服务器）');

  if (!newServerResult.success) {
    logger.warn('统一通信服务（新服务器）初始化失败:', getUserMessage(newServerResult.error));
  }

  const legacyServerResult = await wrapAsync(async () => {
    const { hostname, port } = getServerConfig();
    const legacyServerIp = localStorage.getItem('serverIp') || hostname;
    const legacyServerPort = parseInt(localStorage.getItem('serverPort') || String(port), 10);
    const legacyService = initForLegacyServer(legacyServerIp, legacyServerPort);
    await legacyService.connect();
    legacyService.on('message', () => {});
    serviceStore.setLegacyServerService(legacyService);
    return legacyService;
  }, '初始化统一通信服务（旧服务器）');

  if (!legacyServerResult.success) {
    logger.warn('统一通信服务（旧服务器）初始化失败:', getUserMessage(legacyServerResult.error));
  }

  const syncInitResult = await wrapAsync(async () => {
    await syncService.initialize();
    return true;
  }, '初始化同步服务');

  if (syncInitResult.success) {
    serviceStore.setSyncService(syncService);
    await wrapAsync(async () => {
      await syncService.triggerSync();
      return true;
    }, '手动触发同步测试');
  } else {
    logger.warn('同步服务初始化失败:', getUserMessage(syncInitResult.error));
  }
}

export function setupResourceCleanup(): void {
  window.addEventListener('beforeunload', () => {
    syncService.destroy();
    monitoringService.cleanup();
  });
}

export async function exposeServicesToStore(): Promise<void> {
  const serviceStore = useServiceStore();
  serviceStore.setSyncService(syncService);
  serviceStore.setMonitoringService(monitoringService);
}

export async function initializeApp(app: App): Promise<{ mountedApp: unknown; syncService: typeof syncService }> {
  const result = await wrapAsync(async () => {
    wrapSync(() => {
      initializeServerConfig();
      return true;
    }, '初始化服务器配置');

    monitoringService.init();
    const mountedApp = app.mount('#app');

    await wrapAsync(async () => {
      await initializeCoreServices();
      return true;
    }, '初始化核心服务');

    setupResourceCleanup();
    await exposeServicesToStore();

    return { mountedApp, syncService };
  }, '应用初始化');

  if (!result.success) {
    logger.error('应用初始化失败:', getUserMessage(result.error));
    const mountedApp = app.mount('#app');
    return { mountedApp, syncService };
  }

  return result.data;
}
