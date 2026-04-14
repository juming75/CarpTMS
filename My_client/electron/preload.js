const { contextBridge, ipcRenderer } = require('electron');

// 向渲染进程暴露 API
contextBridge.exposeInMainWorld('electronAPI', {
  // 车辆管理
  getLocalVehicles: () => ipcRenderer.invoke('get-local-vehicles'),
  saveLocalVehicle: (vehicle) => ipcRenderer.invoke('save-local-vehicle', vehicle),
  deleteLocalVehicle: (vehicleId) => ipcRenderer.invoke('delete-local-vehicle', vehicleId),

  // 称重数据
  getLocalWeighingData: (limit) => ipcRenderer.invoke('get-local-weighing-data', limit),
  saveLocalWeighingData: (data) => ipcRenderer.invoke('save-local-weighing-data', data),

  // 数据同步
  getUnsyncedData: () => ipcRenderer.invoke('get-unsynced-data'),
  markAsSynced: (data) => ipcRenderer.invoke('mark-as-synced', data),
  logSync: (log) => ipcRenderer.invoke('log-sync', log),

  // 数据库信息
  getDbInfo: () => ipcRenderer.invoke('get-db-info'),

  // 系统信息
  getAppVersion: () => ipcRenderer.invoke('get-app-version'),
  getElectronVersion: () => ipcRenderer.invoke('get-electron-version'),

  // 窗口控制
  minimizeWindow: () => ipcRenderer.send('window-minimize'),
  maximizeWindow: () => ipcRenderer.send('window-maximize'),
  closeWindow: () => ipcRenderer.send('window-close'),

  // TCP客户端操作
  tcpInit: (host, port) => ipcRenderer.invoke('tcp-init', host, port),
  tcpConnect: () => ipcRenderer.invoke('tcp-connect'),
  tcpSend: (data) => ipcRenderer.invoke('tcp-send', data),
  tcpIsConnected: () => ipcRenderer.invoke('tcp-is-connected'),
  tcpDisconnect: () => ipcRenderer.invoke('tcp-disconnect'),

  // 事件监听
  on: (channel, callback) => {
    const validChannels = ['sync-progress', 'sync-complete', 'sync-error'];
    if (validChannels.includes(channel)) {
      ipcRenderer.on(channel, callback);
    }
  },
  removeListener: (channel, callback) => {
    ipcRenderer.removeListener(channel, callback);
  },
});


