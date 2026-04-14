/**
 * 兼容层 V2 集成示例
 * 展示如何将新版兼容层集成到现有应用中
 */

// import { initApiCompatibilityLayerV2, getApiCompatibilityLayerV2 } from '@/services/apiCompatibilityLayerV2'
import { logService } from '@/services/logService';

/**
 * 初始化新版兼容层 V2
 *
 * 在 main.ts 中调用此函数替代原有的 initializeApiCompatibilityLayer
 */
export async function initializeApiCompatibilityLayerV2(): Promise<boolean> {
  try {
    const requestId = logService.generateRequestId();

    logService.info('[兼容层V2] 开始初始化', { requestId });

    // 初始化兼容层 V2（自动检测协议版本）
    // const layer = await initApiCompatibilityLayerV2(serverIp, serverPort, 'auto')

    // if (!layer) {
    //   logService.error('[兼容层V2] 初始化失败：无法创建实例', { requestId })
    //   return false
    // }

    // // 获取连接状态
    // const status = layer.getConnectionStatus()
    // logService.info('[兼容层V2] 初始化完成', {
    //   requestId,
    //   status
    // })

    // // 测试连接：尝试登录
    // try {
    //   logService.info('[兼容层V2] 测试登录连接...', { requestId })
    //   const loginResult = await layer.handleLogin('ED', '888888')

    //   if (loginResult.success) {
    //     logService.info('[兼容层V2] 登录测试成功', {
    //       requestId,
    //       userId: loginResult.userId
    //     })
    //   } else {
    //     logService.warn('[兼容层V2] 登录测试失败', {
    //       requestId,
    //       error: loginResult.error
    //     })
    //   }
    // } catch (error) {
    //   logService.warn('[兼容层V2] 登录测试失败（可能服务器未响应）', {
    //     requestId,
    //     error: error instanceof Error ? error.message : String(error)
    //   })
    // }

    // // 将兼容层实例暴露到全局（便于调试）
    // ;(window as any).$compatibilityLayerV2 = layer

    // logService.info('[兼容层V2] 已暴露到全局变量 window.$compatibilityLayerV2', { requestId })

    logService.warn('[兼容层V2] 未实现，跳过初始化', { requestId });
    return false;
  } catch (error) {
    logService.error(
      '[兼容层V2] 初始化失败',
      {
        error: error instanceof Error ? error.message : String(error),
      },
      error instanceof Error ? error : new Error(String(error))
    );
    return false;
  }
}

/**
 * 车辆服务 - 使用兼容层 V2
 */
export class VehicleServiceV2 {
  /**
   * 获取车辆列表
   */
  static async getVehicles(): Promise<unknown> {
    // const layer = getApiCompatibilityLayerV2()
    // if (!layer) {
    //   throw new Error('兼容层 V2 未初始化')
    // }

    try {
      logService.info('[车辆服务V2] 获取车辆列表');
      // const result = await layer.handleVehicleRequest('GET')
      // logService.info('[车辆服务V2] 获取车辆列表成功', {
      //   count: result.items?.length || 0
      // })
      // return result
      throw new Error('兼容层 V2 未实现');
    } catch (error) {
      logService.error('[车辆服务V2] 获取车辆列表失败', { error });
      throw error;
    }
  }

  /**
   * 获取单个车辆
   */
  static async getVehicle(vehicleId: number): Promise<unknown> {
    // const layer = getApiCompatibilityLayerV2()
    // if (!layer) {
    //   throw new Error('兼容层 V2 未初始化')
    // }

    try {
      logService.info('[车辆服务V2] 获取单个车辆', { vehicleId });
      // const result = await layer.handleVehicleRequest('GET', vehicleId)
      // logService.info('[车辆服务V2] 获取单个车辆成功')
      // return result
      throw new Error('兼容层 V2 未实现');
    } catch (error) {
      logService.error('[车辆服务V2] 获取单个车辆失败', { vehicleId, error });
      throw error;
    }
  }

  /**
   * 创建车辆
   */
  static async createVehicle(vehicleData: Record<string, unknown>): Promise<unknown> {
    // const layer = getApiCompatibilityLayerV2()
    // if (!layer) {
    //   throw new Error('兼容层 V2 未初始化')
    // }

    try {
      logService.info('[车辆服务V2] 创建车辆', { vehicleName: vehicleData.vehicle_name });
      // const result = await layer.handleVehicleRequest('POST', null, vehicleData)
      // logService.info('[车辆服务V2] 创建车辆成功')
      // return result
      throw new Error('兼容层 V2 未实现');
    } catch (error) {
      logService.error('[车辆服务V2] 创建车辆失败', { vehicleData, error });
      throw error;
    }
  }
}

/**
 * 用户服务 - 使用兼容层 V2
 */
export class UserServiceV2 {
  /**
   * 用户登录
   */
  static async login(username: string, _password: string): Promise<unknown> {
    // const layer = getApiCompatibilityLayerV2()
    // if (!layer) {
    //   throw new Error('兼容层 V2 未初始化')
    // }

    try {
      logService.info('[用户服务V2] 用户登录', { username });
      // const result = await layer.handleLogin(username, password)

      // if (result.success) {
      //   // 保存用户信息到 localStorage
      //   localStorage.setItem('userId', result.userId || '')
      //   localStorage.setItem('username', username)
      //   localStorage.setItem('loginTime', Date.now().toString())

      //   logService.info('[用户服务V2] 登录成功', {
      //     userId: result.userId
      //   })
      // } else {
      //   logService.warn('[用户服务V2] 登录失败', {
      //     error: result.error
      //   })
      // }

      // return result
      throw new Error('兼容层 V2 未实现');
    } catch (error) {
      logService.error('[用户服务V2] 登录失败', { username, error });
      throw error;
    }
  }

  /**
   * 用户登出
   */
  static logout(): void {
    logService.info('[用户服务V2] 用户登出');

    // 清除用户信息
    localStorage.removeItem('userId');
    localStorage.removeItem('username');
    localStorage.removeItem('loginTime');
    localStorage.removeItem('token');

    logService.info('[用户服务V2] 登出成功');
  }
}

/**
 * 主应用集成函数
 * 在 main.ts 中替换原有的兼容层初始化代码
 */
export async function setupCompatibilityLayerV2(): Promise<void> {
  console.log('=== 开始初始化兼容层 V2 ===');

  // 初始化兼容层 V2
  const success = await initializeApiCompatibilityLayerV2();

  if (success) {
    console.log('✅ 兼容层 V2 初始化成功');

    // 可以在这里添加额外的初始化逻辑
    // 例如：预加载车辆数据、初始化 WebSocket 等

    console.log('✅ 兼容层 V2 已就绪');
  } else {
    console.warn('⚠️ 兼容层 V2 初始化失败，将使用降级模式');
    // 可以在这里回退到旧版兼容层或降级服务
  }

  console.log('=== 兼容层 V2 初始化完成 ===');
}

// 导出全局函数，便于在浏览器控制台测试
(window as unknown as Record<string, unknown>).testCompatibilityLayerV2 = async () => {
  try {
    console.log('🧪 开始测试兼容层 V2');

    // 测试登录
    const loginResult = await UserServiceV2.login('ED', '888888');
    console.log('✅ 登录测试:', loginResult);

    // 测试获取车辆列表
    const vehicles = await VehicleServiceV2.getVehicles();
    console.log('✅ 车辆列表测试:', vehicles);

    console.log('🎉 所有测试通过！');

    return {
      success: true,
      login: loginResult,
      vehicles,
    };
  } catch (error) {
    console.error('❌ 测试失败:', error);
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
};


