import { ElMessage } from 'element-plus';

interface ApiClient {
  get(url: string, config?: unknown): Promise<{ data: unknown }>;
  post(url: string, data?: unknown, config?: unknown): Promise<{ data: unknown }>;
  put(url: string, data?: unknown, config?: unknown): Promise<{ data: unknown }>;
  delete(url: string, config?: unknown): Promise<{ data: unknown }>;
}

interface LocalStorageService {
  getOfflineOperations(): OfflineOperation[];
  markOperationAsSynced(id: string): void;
  clearSyncedOperations(): void;
  saveLastSyncTime(time: number): void;
  getLastSyncTime(): number;
  saveVehicles(vehicles: unknown[]): void;
}

interface OfflineOperation {
  id: string;
  type: string;
  entity: string;
  entityId?: string | number;
  data: unknown;
  synced: boolean;
}

const _OfflineOperationTypeValues = {
  CREATE: 'CREATE',
  UPDATE: 'UPDATE',
  DELETE: 'DELETE',
} as const;

// 动态导入localStorage服务
let localStorageServiceCache: LocalStorageService | null = null;

async function importLocalStorageService(): Promise<{
  localStorageService: LocalStorageService;
  OfflineOperation: typeof _OfflineOperationTypeValues;
  OfflineOperationType: typeof _OfflineOperationTypeValues;
}> {
  if (!localStorageServiceCache) {
    const module = await import('./localStorage');
    localStorageServiceCache = module.localStorageService as LocalStorageService;
  }
  return {
    localStorageService: localStorageServiceCache,
    OfflineOperation: _OfflineOperationTypeValues,
    OfflineOperationType: _OfflineOperationTypeValues,
  };
}

// 动态导入API
let apiCache: ApiClient | null = null;
async function importApi(): Promise<ApiClient> {
  if (!apiCache) {
    const module = await import('@/api');
    apiCache = module.default as ApiClient;
  }
  return apiCache;
}

// 同步服务类
export class SyncService {
  private isSyncing = false;
  private syncInterval: number | null = null;

  // 初始化同步服务
  async initialize(): Promise<void> {
    console.log('[同步服务] 初始化同步服务');

    // 监听网络状态变化
    this.setupNetworkListeners();

    // 启动定期同步检查
    this.startPeriodicSync();

    // 检查是否有未同步的操作
    this.checkPendingSync();

    // 初始化时从服务器同步数据
    await this.syncFromServer();
  }

  // 设置网络状态监听器
  private setupNetworkListeners(): void {
    // 监听网络恢复事件
    window.addEventListener('online', () => {
      console.log('[同步服务] 网络已恢复，开始同步离线操作');
      this.syncOfflineOperations();
    });

    // 监听网络断开事件
    window.addEventListener('offline', () => {
      console.log('[同步服务] 网络已断开，暂停同步');
      this.pauseSync();
    });
  }

  // 启动定期同步检查
  private startPeriodicSync(): void {
    // 每30秒检查一次是否有未同步的操作，并从服务器同步数据
    this.syncInterval = window.setInterval(async () => {
      await this.checkPendingSync();
      // 定期从服务器同步数据
      await this.syncFromServer();
    }, 30000);

    console.log('[同步服务] 定期同步检查已启动，间隔30秒');
  }

  // 暂停同步
  private pauseSync(): void {
    if (this.syncInterval) {
      clearInterval(this.syncInterval);
      this.syncInterval = null;
      console.log('[同步服务] 同步已暂停');
    }
  }

  // 检查是否有未同步的操作
  private async checkPendingSync(): Promise<void> {
    const { localStorageService: lsService } = await importLocalStorageService();
    const operations = lsService.getOfflineOperations();
    const pendingOperations = operations.filter((op: { synced: boolean }) => !op.synced);

    // 检查网络状态
    const isOnline = typeof window !== 'undefined' && typeof window.navigator !== 'undefined' && window.navigator.onLine;

    if (pendingOperations.length > 0 && isOnline) {
      console.log('[同步服务] 发现', pendingOperations.length, '个未同步的操作');
      await this.syncOfflineOperations();
    }
  }

  // 同步离线操作到后端
  async syncOfflineOperations(): Promise<void> {
    if (this.isSyncing) {
      console.log('[同步服务] 同步已在进行中，跳过');
      return;
    }

    try {
      this.isSyncing = true;
      console.log('[同步服务] 开始同步离线操作');

      // 检查是否有token
      const token = localStorage.getItem('token');
      if (!token) {
        // SECURITY
        this.isSyncing = false;
        return;
      }

      // 动态导入localStorage服务
      const { localStorageService: lsService } = await importLocalStorageService();

      // 获取未同步的操作
      const operations = lsService.getOfflineOperations();
      const pendingOperations = operations.filter((op: { synced: boolean }) => !op.synced);

      if (pendingOperations.length === 0) {
        console.log('[同步服务] 没有未同步的操作');
        this.isSyncing = false;
        return;
      }

      console.log('[同步服务] 准备同步', pendingOperations.length, '个操作');

      // 按时间顺序同步操作
      let successCount = 0;
      let failCount = 0;

      for (const operation of pendingOperations) {
        try {
          await this.syncSingleOperation(operation);
          successCount++;

          // 标记操作为已同步
          lsService.markOperationAsSynced(operation.id);
          console.log('[同步服务] 操作同步成功:', operation.id, operation.type, operation.entity);
        } catch (error) {
          failCount++;
          console.error('[同步服务] 操作同步失败:', operation.id, error);
        }
      }

      // 清理已同步的操作
      lsService.clearSyncedOperations();

      // 保存最后同步时间
      lsService.saveLastSyncTime(Date.now());

      console.log('[同步服务] 同步完成:', {
        total: pendingOperations.length,
        success: successCount,
        fail: failCount,
      });

      // 显示同步结果
      if (successCount > 0) {
        ElMessage.success(`同步成功 ${successCount} 个操作`);
      }

      if (failCount > 0) {
        ElMessage.warning(`同步失败 ${failCount} 个操作，请检查网络连接后重试`);
      }
    } catch (error) {
      console.error('[同步服务] 同步过程中发生错误:', error);
      ElMessage.error('同步失败，请检查网络连接后重试');
    } finally {
      this.isSyncing = false;
    }
  }

  // 同步单个操作
  private async syncSingleOperation(operation: { id: string; type: string; entity: string; entityId?: string | number; data: unknown }): Promise<void> {
    console.log('[同步服务] 同步操作:', operation.id, operation.type, operation.entity);

    switch (operation.entity) {
      case 'vehicle':
        await this.syncVehicleOperation(operation);
        break;
      case 'device':
        await this.syncDeviceOperation(operation);
        break;
      case 'user':
        await this.syncUserOperation(operation);
        break;
      default:
        console.warn('[同步服务] 未知实体类型，跳过同步:', operation.entity);
    }
  }

  // 同步车辆操作
  private async syncVehicleOperation(operation: { type: string; entityId?: string | number; data: unknown }): Promise<void> {
    const { OfflineOperationType: OfflineOpType } = await importLocalStorageService();
    const apiClient = await importApi();

    switch (operation.type) {
      case OfflineOpType.CREATE:
        await apiClient.post('/api/vehicles', operation.data);
        console.log('[同步服务] 同步创建车辆成功');
        break;

      case OfflineOpType.UPDATE:
        if (operation.entityId) {
          await apiClient.put(`/api/vehicles/${operation.entityId}`, operation.data);
          console.log('[同步服务] 同步更新车辆成功:', operation.entityId);
        }
        break;

      case OfflineOpType.DELETE:
        if (operation.entityId) {
          await apiClient.delete(`/api/vehicles/${operation.entityId}`);
          console.log('[同步服务] 同步删除车辆成功:', operation.entityId);
        }
        break;
    }
  }

  // 同步设备操作
  private async syncDeviceOperation(operation: { type: string; entityId?: string | number; data: unknown }): Promise<void> {
    const { OfflineOperationType: OfflineOpType } = await importLocalStorageService();
    const apiClient = await importApi();

    switch (operation.type) {
      case OfflineOpType.CREATE:
        await apiClient.post('/api/devices', operation.data);
        break;
      case OfflineOpType.UPDATE:
        if (operation.entityId) {
          await apiClient.put(`/api/devices/${operation.entityId}`, operation.data);
        }
        break;
      case OfflineOpType.DELETE:
        if (operation.entityId) {
          await apiClient.delete(`/api/devices/${operation.entityId}`);
        }
        break;
    }
  }

  // 同步用户操作
  private async syncUserOperation(operation: { type: string; entityId?: string | number; data: unknown }): Promise<void> {
    const { OfflineOperationType: OfflineOpType } = await importLocalStorageService();
    const apiClient = await importApi();

    switch (operation.type) {
      case OfflineOpType.CREATE:
        await apiClient.post('/api/users', operation.data);
        break;
      case OfflineOpType.UPDATE:
        if (operation.entityId) {
          await apiClient.put(`/api/users/${operation.entityId}`, operation.data);
        }
        break;
      case OfflineOpType.DELETE:
        if (operation.entityId) {
          await apiClient.delete(`/api/users/${operation.entityId}`);
        }
        break;
    }
  }

  // 手动触发同步
  async triggerSync(): Promise<void> {
    console.log('[同步服务] 手动触发同步');
    await this.syncOfflineOperations();
    // 同步服务器数据到客户端
    await this.syncFromServer();
  }

  // 从服务器同步数据到客户端
  private async syncFromServer(): Promise<void> {
    try {
      console.log('[同步服务] 开始从服务器同步数据');

      // 检查是否有token
      const token = localStorage.getItem('token');
      if (!token) {
        // SECURITY
        return;
      }

      // 导入API服务
      const { vehicleApi, authApi } = await import('@/api');

      // 同步用户信息（单独的try-catch，不影响其他同步）
      if (localStorage.getItem('userInfo')) {
        try {
          const userInfoStr = localStorage.getItem('userInfo')!;
          const userInfo = JSON.parse(userInfoStr);
          if (userInfo.user_id) {
            console.log('[同步服务] 开始同步用户信息:', userInfo.user_id);
            try {
              const userData = await authApi.getCurrentUser(userInfo.user_id);
              if (userData) {
                localStorage.setItem('userInfo', JSON.stringify(userData));
                const username = typeof userData === 'object' && userData !== null && 'username' in userData ? userData.username : 'Unknown';
                console.log('[同步服务] 用户信息同步成功:', username);
                ElMessage.success(`用户信息同步成功: ${username}`);
              }
            } catch (userError) {
              console.error('[同步服务] 同步用户信息失败:', userError);
            }
          }
        } catch (parseError) {
          console.error('[同步服务] 解析用户信息失败:', parseError);
        }
      }

      // 同步车辆资料（单独的try-catch，确保即使其他同步失败也能执行）
      try {
        console.log('[同步服务] 直接使用vehicleApi获取车辆数据');
        const vehiclesData = await vehicleApi.getAll();
        console.log('[同步服务] vehicleApi获取车辆数据:', vehiclesData);

        // 增强错误处理：检查响应格式
        if (vehiclesData) {
          // 处理不同的数据格式
          let vehicles: unknown[] = [];
          if ('success' in (vehiclesData as object) && 'data' in (vehiclesData as object)) {
            const data = vehiclesData as unknown;
            vehicles = (data as { items?: unknown[] })?.items || (data as unknown[]) || [];
          } else if (Array.isArray(vehiclesData)) {
            // 直接返回数组的情况
            vehicles = vehiclesData;
          } else if ('data' in (vehiclesData as object) && Array.isArray((vehiclesData as unknown as { data: unknown }).data)) {
            // 只有data字段的情况
            vehicles = (vehiclesData as unknown as { data: unknown[] }).data;
          } else if ('items' in (vehiclesData as object) && Array.isArray((vehiclesData as unknown as { items: unknown }).items)) {
            // 直接返回分页对象的情况（如 { items: [], page: 1, page_size: 20, pages: 0, total: 0 }）
            vehicles = (vehiclesData as unknown as { items: unknown[] }).items;
          }

          // 验证数据是否为真实数据（包含必要字段）
          if (Array.isArray(vehicles)) {
            // 保存车辆数据到本地存储
            const { localStorageService } = await import('@/services/localStorage');
            localStorageService.saveVehicles(vehicles as any);
            console.log('[同步服务] 车辆资料同步成功，共', vehicles.length, '条记录');

            // 显示同步成功的消息
            ElMessage.success(`从服务器同步成功 ${vehicles.length} 条车辆资料`);
          } else {
            console.warn('[同步服务] 获取到的车辆数据格式不正确或为空:', vehiclesData);
          }
        } else {
          console.warn('[同步服务] 没有获取到车辆数据:', vehiclesData);
        }
      } catch (error) {
        console.error('[同步服务] 同步车辆资料失败:', error);
      }

      console.log('[同步服务] 从服务器同步数据完成');
    } catch (error) {
      console.error('[同步服务] 从服务器同步数据时发生错误:', error);
    }
  }

  // 获取同步状态
  async getSyncStatus(): Promise<{
    pendingOperations: number;
    lastSyncTime: number;
    isSyncing: boolean;
  }> {
    const { localStorageService: lsService } = await importLocalStorageService();
    const operations = lsService.getOfflineOperations();
    const pendingOperations = operations.filter((op: { synced: boolean }) => !op.synced).length;
    const lastSyncTime = lsService.getLastSyncTime();

    return {
      pendingOperations,
      lastSyncTime,
      isSyncing: this.isSyncing,
    };
  }

  // 清理所有同步状态
  async clearSyncStatus(): Promise<void> {
    const { localStorageService: lsService } = await importLocalStorageService();
    lsService.clearSyncedOperations();
    console.log('[同步服务] 同步状态已清理');
  }

  // 销毁同步服务
  destroy(): void {
    this.pauseSync();
    console.log('[同步服务] 已销毁');
  }
}

// 创建同步服务实例
export const syncService = new SyncService();


