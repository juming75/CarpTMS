/**
 * 服务器配置工具
 * 用于在浏览器中设置服务器地址和端口
 */

// 配置选项
const serverConfigs = {
  local: {
    ip: '127.0.0.1',
    port: '8081',
    name: '本地服务器'
  },
  remote: {
    ip: '203.170.59.153',
    port: '9808',
    name: '远程服务器'
  }
}

/**
 * 设置服务器配置
 * @param {string} configType - 配置类型: 'local' 或 'remote'
 */
function setServerConfig(configType = 'local') {
  const config = serverConfigs[configType]
  
  if (!config) {
    console.error(`无效的配置类型: ${configType}`)
    return false
  }
  
  // 设置配置
  localStorage.setItem('serverIp', config.ip)
  localStorage.setItem('serverPort', config.port)
  
  console.log(`✅ 已切换到 ${config.name}`)
  console.log('服务器IP:', config.ip)
  console.log('服务器端口:', config.port)
  
  return true
}

/**
 * 获取当前服务器配置
 */
function getServerConfig() {
  return {
    ip: localStorage.getItem('serverIp'),
    port: localStorage.getItem('serverPort')
  }
}

/**
 * 显示当前配置
 */
function showCurrentConfig() {
  const config = getServerConfig()
  console.log('📋 当前服务器配置:')
  console.log('  服务器IP:', config.ip || '未设置')
  console.log('  服务器端口:', config.port || '未设置')
}

/**
 * 切换到本地服务器
 */
function useLocalServer() {
  setServerConfig('local')
}

/**
 * 切换到远程服务器
 */
function useRemoteServer() {
  setServerConfig('remote')
}

/**
 * 清除服务器配置
 */
function clearServerConfig() {
  localStorage.removeItem('serverIp')
  localStorage.removeItem('serverPort')
  console.log('🗑️ 服务器配置已清除')
}

/**
 * 重新加载页面
 */
function reloadPage() {
  console.log('🔄 页面即将刷新...')
  setTimeout(() => {
    location.reload()
  }, 500)
}

// 将函数暴露到全局
window.serverConfig = {
  set: setServerConfig,
  get: getServerConfig,
  show: showCurrentConfig,
  useLocal: useLocalServer,
  useRemote: useRemoteServer,
  clear: clearServerConfig,
  reload: reloadPage
}

// 页面加载时显示当前配置
console.log('🚀 服务器配置工具已加载')
showCurrentConfig()

console.log('\n💡 使用方法:')
console.log('  切换到本地服务器: serverConfig.useLocal()')
console.log('  切换到远程服务器: serverConfig.useRemote()')
console.log('  显示当前配置: serverConfig.show()')
console.log('  清除配置: serverConfig.clear()')
console.log('  刷新页面: serverConfig.reload()')


