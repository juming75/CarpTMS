import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'

import App from './App.vue'
import router from './router'
import { syncService } from './services/syncService'
import { initForNewServer, initForLegacyServer, type CommunicationMessage } from './services/unifiedCommunicationService'
import { wrapAsync, wrapSync, getUserMessage } from './services/errorHandler'
import monitoringService from './services/monitoring'

const app = createApp(App)

for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.use(createPinia())
app.use(router)
app.use(ElementPlus, {
  locale: zhCn
})

function initializeServerConfig() {
  console.log('0. 设置服务器配置...')
  const serverIp = '127.0.0.1'
  const serverPort = '8082'

  console.log('设置服务器配置...')
  localStorage.setItem('serverIp', serverIp)
  localStorage.setItem('serverPort', serverPort)
  localStorage.setItem('newServerIp', serverIp)
  localStorage.setItem('newServerPort', serverPort)

  console.log('✅ 服务器配置已更新（本地服务器）:', {
    serverIp: localStorage.getItem('serverIp'),
    serverPort: localStorage.getItem('serverPort'),
    newServerIp: localStorage.getItem('newServerIp'),
    newServerPort: localStorage.getItem('newServerPort')
  })
}

async function initializeCoreServices() {
  console.log('1. 开始初始化统一通信服务（新服务器）...')
  const newServerResult = await wrapAsync(async () => {
    const newService = initForNewServer('localhost', 8082, 'auto')
    await newService.connect()

    const pingResult = await wrapAsync(async () => {
      await newService.send({
        type: 'command',
        command: 'ping',
        payload: {}
      })
      return true
    }, '新服务器消息测试')

    if (pingResult.success) {
      console.log('✅ 统一通信服务（新服务器）连接成功')
    } else {
      console.warn('新服务器消息测试失败，可能服务未启动:', getUserMessage(pingResult.error))
    }

    newService.on('message', (data: unknown) => {
      const message = data as CommunicationMessage
      console.log('[统一通信服务] 收到消息:', message)
    })

    ;(window as any).$newServerService = newService
    console.log('✅ 新服务器服务已暴露到全局 (window.$newServerService)')
    return newService
  }, '初始化统一通信服务（新服务器）')

  if (!newServerResult.success) {
    console.warn('统一通信服务（新服务器）初始化失败:', getUserMessage(newServerResult.error))
    console.log('  这不影响应用运行，将继续使用旧服务')
  }

  console.log('2. 开始初始化统一通信服务（旧服务器）...')
  const legacyServerResult = await wrapAsync(async () => {
    const legacyServerIp = localStorage.getItem('serverIp') || '127.0.0.1'
    const legacyServerPort = parseInt(localStorage.getItem('serverPort') || '8082')

    const legacyService = initForLegacyServer(legacyServerIp, legacyServerPort)
    await legacyService.connect()

    legacyService.on('message', (message: any) => {
      console.log('[统一通信服务-旧服务器] 收到消息:', message)
    })

    ;(window as any).$legacyServerService = legacyService
    console.log('✅ 旧服务器服务已暴露到全局 (window.$legacyServerService)')
    return legacyService
  }, '初始化统一通信服务（旧服务器）')

  if (!legacyServerResult.success) {
    console.warn('统一通信服务（旧服务器）初始化失败:', getUserMessage(legacyServerResult.error))
  }

  console.log('3. 开始初始化同步服务...')
  const syncInitResult = await wrapAsync(async () => {
    await syncService.initialize()
    console.log('同步服务初始化完成')
    return true
  }, '初始化同步服务')

  if (syncInitResult.success) {
    console.log('4. 手动触发同步测试...')
    const syncTriggerResult = await wrapAsync(async () => {
      await syncService.triggerSync()
      console.log('同步测试完成')
      return true
    }, '手动触发同步测试')

    if (!syncTriggerResult.success) {
      console.warn('同步测试失败:', getUserMessage(syncTriggerResult.error))
    }
  } else {
    console.warn('同步服务初始化失败:', getUserMessage(syncInitResult.error))
  }
}

function initializeApplication() {
  console.log('3. 初始化监控服务...')
  monitoringService.init()
  console.log('监控服务初始化完成')
  
  console.log('4. 挂载应用...')
  const mountedApp = app.mount('#app')
  console.log('应用挂载成功')
  return mountedApp
}

function setupResourceCleanup() {
  window.addEventListener('beforeunload', async () => {
    console.log('应用关闭，清理资源...')
    syncService.destroy()
    monitoringService.cleanup()
    console.log('资源清理完成')
  })
}

async function exposeServicesToGlobal() {
  console.log('5. 暴露服务到全局...')
  ;(window as any).$sync = syncService
  ;(window as any).$monitoring = monitoringService

  console.log('全局服务:')
  console.log('  - $sync: 同步服务')
  console.log('  - $monitoring: 监控服务')
  console.log('  - $newServerService: 新服务器服务（统一通信）')
  console.log('  - $legacyServerService: 旧服务器服务（统一通信）')
  console.log('服务暴露完成')
}

async function initializeApp() {
  const result = await wrapAsync(async () => {
    console.log('=== 应用初始化开始 ===')

    wrapSync(() => {
      initializeServerConfig()
      return true
    }, '初始化服务器配置')

    const mountedApp = initializeApplication()
    console.log('✅ 应用已挂载')

    await wrapAsync(async () => {
      await initializeCoreServices()
      return true
    }, '初始化核心服务')

    setupResourceCleanup()
    await exposeServicesToGlobal()

    console.log('=== 应用初始化完成 ===')

    return { mountedApp, syncService }
  }, '应用初始化')

  if (!result.success) {
    console.error('应用初始化失败:', getUserMessage(result.error))
    const mountedApp = app.mount('#app')
    return { mountedApp, syncService }
  }

  return result.data
}

// 使用 types/window.d.ts 中定义的 Window 接口扩展

initializeApp()

export { syncService }
