/**
 * 统一通信服务使用示例
 *
 * 这个文件展示了如何使用统一通信服务 (unifiedCommunicationService)
 * 来自动选择TCP或WebSocket协议进行通信
 */

// import { initUnifiedCommunicationService, initForNewServer, initForLegacyServer } from './services/unifiedCommunicationService'
// import type { CommunicationMessage } from './services/unifiedCommunicationService'

// ============================================
// 示例 1: 使用新服务器（自动选择协议）
// ============================================
async function example1_AutoProtocol() {
  console.log('=== 示例 1: 自动选择协议 ===');

  // 初始化统一通信服务 - 自动选择TCP或WebSocket
  // const newService = initUnifiedCommunicationService({
  //   host: 'localhost',
  //   port: 8082,
  //   protocol: 'auto',  // 自动选择最优协议
  //   wsPath: '/ws',
  //   reconnectInterval: 3000,
  //   maxReconnectAttempts: 5,
  //   heartbeatInterval: 30
  // })

  // try {
  //   // 连接到服务器
  //   const connected = await newService.connect()
  //   console.log('连接结果:', connected)
  //   console.log('当前使用的协议:', newService.getCurrentProtocol())

  //   // 发送登录消息
  //   await newService.send({
  //     type: 'command',
  //     command: 'login',
  //     timestamp: Date.now(),
  //     payload: {
  //       username: 'admin',
  //       password: 'password'
  //     }
  //   })

  //   // 监听消息
  //   newService.on('message', (message: CommunicationMessage) => {
  //     console.log('收到消息:', message)
  //   })

  //   // 监听连接状态变化
  //   newService.on('connected', (data) => {
  //     console.log('已连接:', data)
  //   })

  //   newService.on('disconnected', () => {
  //     console.log('已断开连接')
  //   })

  //   newService.on('error', (data) => {
  //     console.error('连接错误:', data)
  //   })

  //   // 获取统计信息
  //   console.log('统计信息:', newService.getStats())

  //   // 切换协议（可选）
  //   // await newService.switchProtocol('websocket')

  // } catch (error) {
  //   console.error('错误:', error)
  // }

  console.log('统一通信服务未实现');
}

// ============================================
// 示例 2: 使用便捷函数初始化新服务器
// ============================================
async function example2_NewServerHelper() {
  console.log('=== 示例 2: 使用便捷函数 ===');

  // 使用便捷函数初始化新服务器
  // const newService = initForNewServer('localhost', 8082, 'auto')

  // // 连接
  // const connected = await newService.connect()
  // console.log('连接结果:', connected)

  // // 发送消息
  // await newService.send({
  //   type: 'command',
  //   command: 'get_vehicles',
  //   timestamp: Date.now(),
  //   payload: {}
  // })

  // // 断开连接
  // await newService.disconnect()

  console.log('统一通信服务未实现');
}

// ============================================
// 示例 3: 连接旧服务器（仅TCP）
// ============================================
async function example3_LegacyServer() {
  console.log('=== 示例 3: 连接旧服务器 ===');

  // 使用便捷函数初始化旧服务器（仅支持TCP）
  // const legacyService = initForLegacyServer('203.170.59.153', 9808)

  // // 连接
  // const connected = await legacyService.connect()
  // console.log('连接结果:', connected)

  // // 发送消息
  // await legacyService.send({
  //   type: 'command',
  //   command: 'GET_VEHICLES',
  //   timestamp: Date.now(),
  //   payload: {}
  // })

  // // 断开连接
  // await legacyService.disconnect()

  console.log('统一通信服务未实现');
}

// ============================================
// 示例 4: 协议切换
// ============================================
async function example4_ProtocolSwitch() {
  console.log('=== 示例 4: 协议切换 ===');

  // const service = initForNewServer('localhost', 8082, 'websocket')

  // // 连接（使用WebSocket）
  // await service.connect()
  // console.log('初始协议:', service.getCurrentProtocol())

  // // 切换到TCP
  // await service.switchProtocol('tcp')
  // console.log('切换后协议:', service.getCurrentProtocol())

  // // 切换回WebSocket
  // await service.switchProtocol('websocket')
  // console.log('最终协议:', service.getCurrentProtocol())

  console.log('统一通信服务未实现');
}

// ============================================
// 示例 5: 完整的应用集成
// ============================================
class CarpTMSApp {
  // private service: ReturnType<typeof initUnifiedCommunicationService>

  constructor() {
    // 初始化通信服务
    // this.service = initUnifiedCommunicationService({
    //   host: import.meta.env.VITE_SERVER_HOST || 'localhost',
    //   port: parseInt(import.meta.env.VITE_SERVER_PORT || '8082'),
    //   protocol: 'auto',
    //   wsPath: '/ws',
    //   reconnectInterval: 3000,
    //   maxReconnectAttempts: 5,
    //   heartbeatInterval: 30
    // })

    // this.setupEventHandlers()
    console.log('统一通信服务未实现');
  }

  // private setupEventHandlers() {
  //   // 监听连接状态
  //   this.service.on('connected', () => {
  //     console.log('应用: 服务器已连接')
  //     // 可以在这里显示连接状态UI
  //   })

  //   this.service.on('disconnected', () => {
  //     console.warn('应用: 服务器已断开')
  //     // 可以在这里显示离线状态UI
  //   })

  //   this.service.on('error', (error) => {
  //     console.error('应用: 连接错误', error)
  //     // 可以在这里显示错误提示
  //   })

  //   // 监听服务器推送的消息
  //   this.service.on('message', (message: CommunicationMessage) => {
  //     this.handleServerMessage(message)
  //   })
  // }

  // private handleServerMessage(message: CommunicationMessage) {
  //   switch (message.type) {
  //     case 'vehicle_update':
  //       console.log('车辆更新:', message.payload)
  //       // 更新车辆列表UI
  //       break

  //     case 'alert':
  //       console.log('告警:', message.payload)
  //       // 显示告警通知
  //       break

  //     case 'device_status':
  //       console.log('设备状态:', message.payload)
  //       // 更新设备状态
  //       break

  //     default:
  //       console.log('未知消息类型:', message.type)
  //   }
  // }

  async login(_username: string, _password: string) {
    try {
      // const response = await this.service.send({
      //   type: 'command',
      //   command: 'login',
      //   timestamp: Date.now(),
      //   payload: { username, password }
      // })

      // console.log('登录响应:', response)
      // return response
      throw new Error('统一通信服务未实现');
    } catch (error) {
      console.error('登录失败:', error);
      throw error;
    }
  }

  async getVehicles() {
    try {
      // const response = await this.service.send({
      //   type: 'command',
      //   command: 'get_vehicles',
      //   timestamp: Date.now(),
      //   payload: {}
      // })

      // console.log('车辆列表:', response)
      // return response
      throw new Error('统一通信服务未实现');
    } catch (error) {
      console.error('获取车辆列表失败:', error);
      throw error;
    }
  }

  async sendCommand(_vehicleId: number, _command: string, _params: unknown) {
    try {
      // const response = await this.service.send({
      //   type: 'command',
      //   command: 'vehicle_command',
      //   timestamp: Date.now(),
      //   payload: {
      //     vehicle_id: vehicleId,
      //     command,
      //     params
      //   }
      // })

      // console.log('命令响应:', response)
      // return response
      throw new Error('统一通信服务未实现');
    } catch (error) {
      console.error('发送命令失败:', error);
      throw error;
    }
  }

  getConnectionStats() {
    // return this.service.getStats()
    throw new Error('统一通信服务未实现');
  }

  async disconnect() {
    // await this.service.disconnect()
    throw new Error('统一通信服务未实现');
  }
}

// ============================================
// 使用示例
// ============================================

// 如果在浏览器控制台运行：
// example1_AutoProtocol()
// example2_NewServerHelper()
// example3_LegacyServer()
// example4_ProtocolSwitch()

// 在应用中使用：
// const app = new CarpTMSApp()
// await app.login('admin', 'password')
// await app.getVehicles()
// await app.sendCommand(1, 'start_engine', {})

export { example1_AutoProtocol, example2_NewServerHelper, example3_LegacyServer, example4_ProtocolSwitch, CarpTMSApp };


