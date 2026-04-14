/**
 * CarpTMS Electron 主进程（增强版）
 * 集成了详细的日志记录和错误处理
 */
import { app, BrowserWindow, ipcMain, Menu, shell } from 'electron';
import path from 'path';
import { fileURLToPath } from 'url';
import Database from 'better-sqlite3';
import fs from 'fs';
import { exec } from 'child_process';
import * as net from 'net';
import logger from './logger.js';

// ES模块中模拟__dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// 全局变量
let mainWindow = null;
let db = null;

// 全局TCP客户端变量
let tcpSocket = null;
let tcpConnected = false;
let tcpHost = '';
let tcpPort = 0;
let tcpMessageQueue = [];

// 错误处理：全局未捕获异常
process.on('uncaughtException', (error) => {
  logger.fatal(
    '未捕获的异常',
    {
      error: error.message,
      stack: error.stack,
    },
    error
  );
});

// 错误处理：未处理的 Promise rejection
process.on('unhandledRejection', (reason, promise) => {
  logger.fatal('未处理的 Promise rejection', {
    reason: String(reason),
  });
});

// 数据库迁移管理
class DatabaseMigration {
  constructor(db) {
    this.db = db;
    this.migrations = [
      {
        version: 1,
        up: `
          CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version INTEGER NOT NULL,
            applied_at TEXT DEFAULT (datetime('now'))
          );
          CREATE TABLE IF NOT EXISTS local_vehicles (
            vehicle_id INTEGER PRIMARY KEY,
            vehicle_name TEXT NOT NULL,
            device_id TEXT,
            own_no TEXT,
            own_name TEXT,
            own_phone TEXT,
            group_id INTEGER,
            is_synced INTEGER DEFAULT 0,
            server_id INTEGER,
            version INTEGER DEFAULT 0,
            last_modified TEXT,
            sync_version INTEGER,
            sync_conflict INTEGER DEFAULT 0,
            conflict_reason TEXT,
            create_time TEXT,
            update_time TEXT
          );
          CREATE TABLE IF NOT EXISTS local_weighing_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            vehicle_id INTEGER NOT NULL,
            device_id TEXT NOT NULL,
            weighing_time TEXT NOT NULL,
            gross_weight REAL NOT NULL,
            tare_weight REAL,
            net_weight REAL NOT NULL,
            axle_count INTEGER,
            speed REAL,
            lane_no INTEGER,
            site_id INTEGER,
            status INTEGER DEFAULT 0,
            is_synced INTEGER DEFAULT 0,
            server_id INTEGER,
            version INTEGER DEFAULT 0,
            last_modified TEXT,
            sync_version INTEGER,
            sync_conflict INTEGER DEFAULT 0,
            conflict_reason TEXT,
            sync_time TEXT,
            create_time TEXT DEFAULT (datetime('now'))
          );
          CREATE TABLE IF NOT EXISTS sync_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sync_type TEXT NOT NULL,
            record_count INTEGER,
            status INTEGER DEFAULT 0,
            error_message TEXT,
            conflict_count INTEGER DEFAULT 0,
            sync_time TEXT DEFAULT (datetime('now'))
          );
          CREATE INDEX IF NOT EXISTS idx_local_weighing_vehicle ON local_weighing_data(vehicle_id);
          CREATE INDEX IF NOT EXISTS idx_local_weighing_time ON local_weighing_data(weighing_time);
        `,
      },
    ];
  }

  getCurrentVersion() {
    try {
      const result = this.db.prepare('SELECT version FROM migrations ORDER BY version DESC LIMIT 1').get();
      return result ? result.version : 0;
    } catch (error) {
      logger.error('获取当前数据库版本失败', { error: error.message }, error);
      return 0;
    }
  }

  migrate() {
    const requestId = logger.generateRequestId();
    const currentVersion = this.getCurrentVersion();
    logger.info('开始数据库迁移', { requestId, currentVersion });

    const pendingMigrations = this.migrations.filter((m) => m.version > currentVersion);

    if (pendingMigrations.length === 0) {
      logger.info('数据库已是最新版本', { requestId });
      return;
    }

    logger.info(`发现 ${pendingMigrations.length} 个待执行的迁移`, { requestId });

    this.db.exec('BEGIN TRANSACTION;');

    try {
      for (const migration of pendingMigrations) {
        logger.info(`执行迁移版本 ${migration.version}`, { requestId });
        this.db.exec(migration.up);

        this.db.prepare('INSERT INTO migrations (version) VALUES (?)').run(migration.version);
        logger.info(`迁移版本 ${migration.version} 执行成功`, { requestId });
      }

      this.db.exec('COMMIT TRANSACTION;');
      logger.info('所有迁移执行成功', { requestId });
    } catch (error) {
      this.db.exec('ROLLBACK TRANSACTION;');
      logger.error('迁移失败', { requestId, error: error.message }, error);
      throw error;
    }
  }
}

// 数据备份管理
class DatabaseBackup {
  constructor(db, app) {
    this.db = db;
    this.app = app;
    this.backupDir = path.join(app.getPath('documents'), 'CarpTMS', 'Backups');
    this.maxBackups = 10;
  }

  initBackupDir() {
    if (!fs.existsSync(this.backupDir)) {
      fs.mkdirSync(this.backupDir, { recursive: true });
      logger.logDatabase('创建备份目录', this.backupDir);
    }
  }

  createBackup() {
    const requestId = logger.generateRequestId();
    this.initBackupDir();

    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const backupPath = path.join(this.backupDir, `CarpTMS_backup_${timestamp}.db`);
    const dbPath = path.join(this.app.getPath('userData'), 'CarpTMS_local.db');

    try {
      fs.copyFileSync(dbPath, backupPath);
      logger.info('备份创建成功', { requestId, backupPath });

      this.cleanupOldBackups();

      return {
        success: true,
        backupPath,
        message: '备份创建成功',
      };
    } catch (error) {
      logger.error('备份创建失败', { requestId, error: error.message }, error);
      return {
        success: false,
        error: error.message,
        message: '备份创建失败',
      };
    }
  }

  restoreBackup(backupPath) {
    const requestId = logger.generateRequestId();
    const dbPath = path.join(this.app.getPath('userData'), 'CarpTMS_local.db');

    try {
      if (this.db && this.db.open) {
        this.db.close();
      }

      fs.copyFileSync(backupPath, dbPath);

      this.db = new Database(dbPath);

      logger.info('备份恢复成功', { requestId, backupPath });
      return {
        success: true,
        message: '备份恢复成功',
      };
    } catch (error) {
      logger.error('备份恢复失败', { requestId, error: error.message }, error);
      return {
        success: false,
        error: error.message,
        message: '备份恢复失败',
      };
    }
  }

  getBackupList() {
    const requestId = logger.generateRequestId();
    this.initBackupDir();

    try {
      const files = fs
        .readdirSync(this.backupDir)
        .filter((file) => file.endsWith('.db'))
        .map((file) => {
          const filePath = path.join(this.backupDir, file);
          const stats = fs.statSync(filePath);
          return {
            fileName: file,
            filePath,
            size: stats.size,
            createdAt: stats.birthtime.toISOString(),
            modifiedAt: stats.mtime.toISOString(),
          };
        })
        .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());

      logger.info('获取备份列表成功', { requestId, count: files.length });
      return {
        success: true,
        backups: files,
      };
    } catch (error) {
      logger.error('获取备份列表失败', { requestId, error: error.message }, error);
      return {
        success: false,
        error: error.message,
        backups: [],
      };
    }
  }

  cleanupOldBackups() {
    const backups = this.getBackupList();
    if (backups.success && backups.backups.length > this.maxBackups) {
      const backupsToDelete = backups.backups.slice(this.maxBackups);

      for (const backup of backupsToDelete) {
        try {
          fs.unlinkSync(backup.filePath);
          logger.info('删除旧备份文件', { fileName: backup.fileName });
        } catch (error) {
          logger.error('删除旧备份失败', { fileName: backup.fileName, error: error.message });
        }
      }
    }
  }

  deleteBackup(backupPath) {
    const requestId = logger.generateRequestId();
    try {
      fs.unlinkSync(backupPath);
      logger.info('备份删除成功', { requestId, backupPath });
      return {
        success: true,
        message: '备份删除成功',
      };
    } catch (error) {
      logger.error('备份删除失败', { requestId, error: error.message }, error);
      return {
        success: false,
        error: error.message,
        message: '备份删除失败',
      };
    }
  }
}

// 全局备份实例
let backupManager = null;

// 创建主窗口
function createWindow() {
  const requestId = logger.generateRequestId();
  logger.info('创建主窗口', { requestId });

  try {
    mainWindow = new BrowserWindow({
      width: 1400,
      height: 900,
      minWidth: 1200,
      minHeight: 700,
      frame: true,
      title: 'CarpTMS - 动态车载称重系统',
      icon: path.join(__dirname, '../build/icon.png'),
      webPreferences: {
        preload: path.join(__dirname, 'preload.js'),
        nodeIntegration: false,
        contextIsolation: true,
        webSecurity: false,
        sandbox: false,
      },
    });

    if (process.env.NODE_ENV === 'development') {
      mainWindow.loadURL('http://localhost:5173');
      mainWindow.webContents.openDevTools();
      logger.info('开发模式：加载开发服务器');
    } else {
      mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
      logger.info('生产模式：加载打包文件');
    }

    mainWindow.on('closed', () => {
      logger.info('主窗口关闭');
      mainWindow = null;
    });

    mainWindow.webContents.setWindowOpenHandler(({ url }) => {
      logger.info('打开外部链接', { url });
      shell.openExternal(url);
      return { action: 'deny' };
    });

    logger.info('主窗口创建成功', { requestId });
  } catch (error) {
    logger.error('创建主窗口失败', { requestId, error: error.message }, error);
    throw error;
  }
}

// 初始化数据库
function initDatabase() {
  const requestId = logger.generateRequestId();
  logger.info('初始化数据库', { requestId });

  try {
    const dbPath = path.join(app.getPath('userData'), 'CarpTMS_local.db');
    db = new Database(dbPath);

    logger.logDatabase('数据库连接成功', dbPath);

    const migration = new DatabaseMigration(db);
    migration.migrate();

    backupManager = new DatabaseBackup(db, app);

    logger.info('数据库初始化成功', { requestId });
  } catch (error) {
    logger.error('数据库初始化失败', { requestId, error: error.message }, error);
    throw error;
  }
}

// TCP连接函数
function connectTcp(host, port) {
  const requestId = logger.generateRequestId();
  return new Promise((resolve) => {
    logger.logTcpConnection('连接中', { requestId, host, port });

    // 关闭现有连接
    if (tcpSocket) {
      tcpSocket.destroy();
      tcpSocket = null;
    }

    // 创建新连接
    tcpSocket = new net.Socket();
    tcpHost = host;
    tcpPort = port;

    // 设置超时
    const timeout = setTimeout(() => {
      logger.error('TCP连接超时', { requestId, host, port });
      tcpConnected = false;
      if (tcpSocket) {
        tcpSocket.destroy();
        tcpSocket = null;
      }
      resolve(false);
    }, 10000);

    // 连接成功
    tcpSocket.connect(port, host, () => {
      clearTimeout(timeout);
      logger.logTcpConnection('连接成功', { requestId, host, port });
      tcpConnected = true;
      resolve(true);
    });

    // 连接关闭
    tcpSocket.on('close', () => {
      clearTimeout(timeout);
      logger.logTcpConnection('连接关闭', { requestId, host, port });
      tcpConnected = false;
    });

    // 连接错误
    tcpSocket.on('error', (error) => {
      clearTimeout(timeout);
      logger.error('TCP连接错误', { requestId, host, port, error: error.message }, error);
      tcpConnected = false;
      resolve(false);
    });

    // 接收数据
    tcpSocket.on('data', (data) => {
      logger.debug('TCP接收数据', { requestId, bytes: data.length });
      try {
        const jsonStr = data.toString('utf8');
        const response = JSON.parse(jsonStr);
        logger.debug('TCP接收到响应', { requestId, response });

        // 处理消息队列
        if (tcpMessageQueue.length > 0) {
          const { resolve } = tcpMessageQueue.shift();
          resolve(response);
        }
      } catch (error) {
        logger.error('TCP数据解析失败', { requestId, error: error.message }, error);
        if (tcpMessageQueue.length > 0) {
          const { reject } = tcpMessageQueue.shift();
          reject(new Error('数据解析失败'));
        }
      }
    });
  });
}

// TCP发送数据函数
function sendTcpData(data) {
  const requestId = logger.generateRequestId();
  return new Promise((resolve, reject) => {
    logger.debug('TCP发送数据', { requestId, data });

    if (!tcpConnected || !tcpSocket) {
      logger.warn('TCP未连接，尝试重连', { requestId });
      tcpMessageQueue.push({ requestId, data, resolve, reject });

      connectTcp(tcpHost, tcpPort).then((connected) => {
        if (!connected) {
          logger.error('TCP重连失败', { requestId });
          reject(new Error('无法连接到服务器'));
        }
      });
      return;
    }

    try {
      const jsonData = JSON.stringify(data);
      tcpSocket.write(jsonData);
      logger.debug('TCP数据已发送', { requestId, size: jsonData.length });

      tcpMessageQueue.push({ requestId, data, resolve, reject });

      setTimeout(() => {
        if (tcpMessageQueue.length > 0) {
          const { reject } = tcpMessageQueue.shift();
          logger.error('TCP发送超时', { requestId });
          reject(new Error('发送超时'));
        }
      }, 30000);
    } catch (error) {
      logger.error('TCP发送数据失败', { requestId, error: error.message }, error);
      reject(error);
    }
  });
}

// 设置 IPC 处理器
function setupIpcHandlers() {
  const requestId = logger.generateRequestId();
  logger.info('设置IPC处理器', { requestId });

  // ====== TCP客户端操作 ======
  ipcMain.handle('tcp-init', async (event, host, port) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('tcp-init', { requestId: reqId, host, port });

    try {
      logger.info('初始化TCP客户端', { requestId: reqId, host, port });
      tcpHost = host;
      tcpPort = port;

      const result = { success: true };
      logger.logIpcResponse('tcp-init', true, { requestId: reqId });
      return result;
    } catch (error) {
      logger.error('初始化TCP客户端失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('tcp-init', false, { requestId: reqId, error: error.message });
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-connect', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('tcp-connect', { requestId: reqId });

    try {
      logger.info('TCP客户端连接中...', { requestId: reqId });
      const connected = await connectTcp(tcpHost, tcpPort);

      logger.logIpcResponse('tcp-connect', true, { requestId: reqId, connected });
      return { success: true, connected };
    } catch (error) {
      logger.error('TCP客户端连接失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('tcp-connect', false, { requestId: reqId, error: error.message });
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-send', async (event, data) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('tcp-send', { requestId: reqId });

    try {
      logger.debug('TCP客户端发送数据', { requestId: reqId, data });
      const response = await sendTcpData(data);

      logger.logIpcResponse('tcp-send', true, { requestId: reqId });
      return { success: true, response };
    } catch (error) {
      logger.error('TCP客户端发送数据失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('tcp-send', false, { requestId: reqId, error: error.message });
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-is-connected', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('tcp-is-connected', { requestId: reqId });

    try {
      const result = { success: true, connected: tcpConnected };
      logger.logIpcResponse('tcp-is-connected', true, { requestId: reqId, connected: tcpConnected });
      return result;
    } catch (error) {
      logger.error('检查TCP连接状态失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('tcp-is-connected', false, { requestId: reqId, error: error.message });
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-disconnect', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('tcp-disconnect', { requestId: reqId });

    try {
      if (tcpSocket) {
        tcpSocket.destroy();
        tcpSocket = null;
        tcpConnected = false;
        logger.logTcpConnection('已断开', { requestId: reqId });
      }

      logger.logIpcResponse('tcp-disconnect', true, { requestId: reqId });
      return { success: true };
    } catch (error) {
      logger.error('TCP客户端断开连接失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('tcp-disconnect', false, { requestId: reqId, error: error.message });
      return { success: false, error: error.message };
    }
  });

  // ====== 车辆管理 ======
  ipcMain.handle('get-local-vehicles', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('get-local-vehicles', { requestId: reqId });

    try {
      const stmt = db.prepare('SELECT * FROM local_vehicles ORDER BY create_time DESC');
      const result = stmt.all();
      logger.logDatabase('查询本地车辆', 'local_vehicles', { requestId: reqId, count: result.length });
      logger.logIpcResponse('get-local-vehicles', true, { requestId: reqId, count: result.length });
      return result;
    } catch (error) {
      logger.error('获取本地车辆失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('get-local-vehicles', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  ipcMain.handle('save-local-vehicle', async (event, vehicle) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('save-local-vehicle', { requestId: reqId, vehicleId: vehicle.vehicle_id });

    try {
      let version = 0;
      let sync_conflict = 0;
      let conflict_reason = null;

      const existingStmt = db.prepare(
        'SELECT version, sync_version, server_id FROM local_vehicles WHERE vehicle_id = ?'
      );
      const existing = existingStmt.get(vehicle.vehicle_id);

      if (existing) {
        version = existing.version + 1;

        if (existing.sync_version && existing.sync_version > existing.version) {
          sync_conflict = 1;
          conflict_reason = '服务器版本高于本地版本';
          logger.warn('检测到版本冲突', { requestId: reqId, vehicleId: vehicle.vehicle_id });
        }
      }

      const stmt = db.prepare(`
        INSERT OR REPLACE INTO local_vehicles 
        (vehicle_id, vehicle_name, device_id, own_no, own_name, own_phone, group_id, is_synced, server_id, 
         version, last_modified, sync_version, sync_conflict, conflict_reason, create_time, update_time)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), ?, ?, ?, datetime('now'), datetime('now'))
      `);
      const result = stmt.run(
        vehicle.vehicle_id,
        vehicle.vehicle_name,
        vehicle.device_id || null,
        vehicle.own_no || null,
        vehicle.own_name || null,
        vehicle.own_phone || null,
        vehicle.group_id,
        0,
        vehicle.server_id || null,
        version,
        vehicle.sync_version || null,
        sync_conflict,
        conflict_reason
      );

      logger.logDatabase('保存本地车辆', 'local_vehicles', {
        requestId: reqId,
        vehicleId: vehicle.vehicle_id,
        changes: result.changes,
      });
      logger.logIpcResponse('save-local-vehicle', true, { requestId: reqId });
      return result;
    } catch (error) {
      logger.error('保存本地车辆失败', { requestId: reqId, vehicle, error: error.message }, error);
      logger.logIpcResponse('save-local-vehicle', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  ipcMain.handle('delete-local-vehicle', async (event, vehicleId) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('delete-local-vehicle', { requestId: reqId, vehicleId });

    try {
      const stmt = db.prepare('DELETE FROM local_vehicles WHERE vehicle_id = ?');
      const result = stmt.run(vehicleId);

      logger.logDatabase('删除本地车辆', 'local_vehicles', { requestId: reqId, vehicleId, changes: result.changes });
      logger.logIpcResponse('delete-local-vehicle', true, { requestId: reqId });
      return result;
    } catch (error) {
      logger.error('删除本地车辆失败', { requestId: reqId, vehicleId, error: error.message }, error);
      logger.logIpcResponse('delete-local-vehicle', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  // ====== 称重数据 ======
  ipcMain.handle('get-local-weighing-data', async (event, limit = 100) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('get-local-weighing-data', { requestId: reqId, limit });

    try {
      const stmt = db.prepare(`
        SELECT * FROM local_weighing_data 
        ORDER BY create_time DESC 
        LIMIT ?
      `);
      const result = stmt.all(limit);

      logger.logDatabase('查询称重数据', 'local_weighing_data', { requestId: reqId, count: result.length });
      logger.logIpcResponse('get-local-weighing-data', true, { requestId: reqId, count: result.length });
      return result;
    } catch (error) {
      logger.error('获取称重数据失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('get-local-weighing-data', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  ipcMain.handle('save-local-weighing-data', async (event, data) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('save-local-weighing-data', { requestId: reqId, dataId: data.id });

    try {
      let version = 0;
      let sync_conflict = 0;
      let conflict_reason = null;

      if (data.id) {
        const existingStmt = db.prepare('SELECT version, sync_version FROM local_weighing_data WHERE id = ?');
        const existing = existingStmt.get(data.id);
        if (existing) {
          version = existing.version + 1;

          if (existing.sync_version && existing.sync_version > existing.version) {
            sync_conflict = 1;
            conflict_reason = '服务器版本高于本地版本';
            logger.warn('检测到版本冲突', { requestId: reqId, dataId: data.id });
          }
        }

        const stmt = db.prepare(`
          UPDATE local_weighing_data 
          SET vehicle_id = ?, device_id = ?, weighing_time = ?, gross_weight = ?, tare_weight = ?, 
              net_weight = ?, axle_count = ?, speed = ?, lane_no = ?, site_id = ?, status = ?, 
              version = ?, last_modified = datetime('now'), sync_conflict = ?, conflict_reason = ?
          WHERE id = ?
        `);
        const result = stmt.run(
          data.vehicle_id,
          data.device_id,
          data.weighing_time,
          data.gross_weight,
          data.tare_weight || null,
          data.net_weight,
          data.axle_count || null,
          data.speed || null,
          data.lane_no || null,
          data.site_id || null,
          data.status || 0,
          version,
          sync_conflict,
          conflict_reason,
          data.id
        );

        logger.logDatabase('更新称重数据', 'local_weighing_data', {
          requestId: reqId,
          dataId: data.id,
          changes: result.changes,
        });
        logger.logIpcResponse('save-local-weighing-data', true, { requestId: reqId });
        return result;
      } else {
        const stmt = db.prepare(`
          INSERT INTO local_weighing_data 
          (vehicle_id, device_id, weighing_time, gross_weight, tare_weight, net_weight, 
           axle_count, speed, lane_no, site_id, status, version, last_modified)
          VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
        `);
        const result = stmt.run(
          data.vehicle_id,
          data.device_id,
          data.weighing_time,
          data.gross_weight,
          data.tare_weight || null,
          data.net_weight,
          data.axle_count || null,
          data.speed || null,
          data.lane_no || null,
          data.site_id || null,
          data.status || 0,
          version
        );

        logger.logDatabase('插入称重数据', 'local_weighing_data', {
          requestId: reqId,
          insertId: result.lastInsertRowid,
        });
        logger.logIpcResponse('save-local-weighing-data', true, { requestId: reqId });
        return result;
      }
    } catch (error) {
      logger.error('保存称重数据失败', { requestId: reqId, data, error: error.message }, error);
      logger.logIpcResponse('save-local-weighing-data', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  // ====== 数据同步 ======
  ipcMain.handle('get-unsynced-data', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('get-unsynced-data', { requestId: reqId });

    try {
      const vehiclesStmt = db.prepare('SELECT * FROM local_vehicles WHERE is_synced = 0');
      const weighingStmt = db.prepare('SELECT * FROM local_weighing_data WHERE is_synced = 0');

      const result = {
        vehicles: vehiclesStmt.all(),
        weighingData: weighingStmt.all(),
      };

      logger.info('获取未同步数据', {
        requestId: reqId,
        vehicleCount: result.vehicles.length,
        weighingCount: result.weighingData.length,
      });
      logger.logIpcResponse('get-unsynced-data', true, { requestId: reqId });
      return result;
    } catch (error) {
      logger.error('获取未同步数据失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('get-unsynced-data', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  ipcMain.handle('mark-as-synced', async (event, { type, ids, serverIds, serverVersions }) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('mark-as-synced', { requestId: reqId, type, count: ids.length });

    try {
      let conflictCount = 0;

      if (type === 'vehicles') {
        const updateStmt = db.prepare(`
          UPDATE local_vehicles 
          SET 
            is_synced = CASE 
              WHEN sync_version IS NULL OR ? > sync_version THEN 1 
              ELSE 0 
            END,
            server_id = ?,
            sync_version = ?,
            sync_conflict = CASE 
              WHEN sync_version IS NOT NULL AND ? <= sync_version THEN 1 
              ELSE 0 
            END,
            conflict_reason = CASE 
              WHEN sync_version IS NOT NULL AND ? <= sync_version THEN '本地版本已经高于或等于服务器版本，可能存在并发修改' 
              ELSE NULL 
            END
          WHERE vehicle_id = ?
        `);

        const selectStmt = db.prepare('SELECT vehicle_id, version FROM local_vehicles WHERE vehicle_id = ?');

        for (let i = 0; i < ids.length; i++) {
          const id = ids[i];
          const serverId = serverIds[i];
          const serverVersion = serverVersions ? serverVersions[i] : 0;

          const result = selectStmt.get(id);
          if (result && serverVersion <= result.version) {
            conflictCount++;
          }

          updateStmt.run(serverVersion, serverId, serverVersion, serverVersion, serverVersion, id);
        }

        logger.logDatabase('标记车辆为已同步', 'local_vehicles', {
          requestId: reqId,
          count: ids.length,
          conflictCount,
        });
      } else if (type === 'weighing') {
        const updateStmt = db.prepare(`
          UPDATE local_weighing_data 
          SET 
            is_synced = CASE 
              WHEN sync_version IS NULL OR ? > sync_version THEN 1 
              ELSE 0 
            END,
            server_id = ?, 
            sync_version = ?,
            sync_conflict = CASE 
              WHEN sync_version IS NOT NULL AND ? <= sync_version THEN 1 
              ELSE 0 
            END,
            conflict_reason = CASE 
              WHEN sync_version IS NOT NULL AND ? <= sync_version THEN '本地版本已经高于或等于服务器版本，可能存在并发修改' 
              ELSE NULL 
            END
          WHERE id = ?
        `);

        for (let i = 0; i < ids.length; i++) {
          const id = ids[i];
          const serverId = serverIds[i];
          const serverVersion = serverVersions ? serverVersions[i] : 0;

          updateStmt.run(serverVersion, serverId, serverVersion, serverVersion, serverVersion, id);
        }

        logger.logDatabase('标记称重数据为已同步', 'local_weighing_data', { requestId: reqId, count: ids.length });
      }

      logger.logIpcResponse('mark-as-synced', true, { requestId: reqId, conflictCount });
      return { success: true, conflictCount };
    } catch (error) {
      logger.error('标记为已同步失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('mark-as-synced', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  // ====== 同步日志 ======
  ipcMain.handle('log-sync', async (event, log) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('log-sync', { requestId: reqId, syncType: log.syncType });

    try {
      const stmt = db.prepare(`
        INSERT INTO sync_logs (sync_type, record_count, status, error_message, conflict_count)
        VALUES (?, ?, ?, ?, ?)
      `);
      const result = stmt.run(
        log.syncType,
        log.recordCount,
        log.status,
        log.errorMessage || null,
        log.conflictCount || 0
      );

      logger.logDatabase('记录同步日志', 'sync_logs', { requestId: reqId, insertId: result.lastInsertRowid });
      logger.logIpcResponse('log-sync', true, { requestId: reqId });
      return result;
    } catch (error) {
      logger.error('记录同步日志失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('log-sync', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  // ====== 数据库信息 ======
  ipcMain.handle('get-db-info', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('get-db-info', { requestId: reqId });

    try {
      const vehicleCount = db.prepare('SELECT COUNT(*) as count FROM local_vehicles').get().count;
      const weighingCount = db.prepare('SELECT COUNT(*) as count FROM local_weighing_data').get().count;

      const result = {
        path: db.open ? 'connected' : 'disconnected',
        vehicleCount,
        weighingCount,
      };

      logger.info('获取数据库信息', { requestId: reqId, vehicleCount, weighingCount });
      logger.logIpcResponse('get-db-info', true, { requestId: reqId });
      return result;
    } catch (error) {
      logger.error('获取数据库信息失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('get-db-info', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  // ====== 系统信息 ======
  ipcMain.handle('get-app-version', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('get-app-version', { requestId: reqId });

    try {
      const version = app.getVersion();
      logger.logIpcResponse('get-app-version', true, { requestId: reqId, version });
      return version;
    } catch (error) {
      logger.error('获取应用版本失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('get-app-version', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  ipcMain.handle('get-electron-version', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('get-electron-version', { requestId: reqId });

    try {
      const version = process.versions.electron;
      logger.logIpcResponse('get-electron-version', true, { requestId: reqId, version });
      return version;
    } catch (error) {
      logger.error('获取Electron版本失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('get-electron-version', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  // ====== 数据备份 ======
  ipcMain.handle('db-backup-create', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('db-backup-create', { requestId: reqId });

    try {
      if (!backupManager) {
        backupManager = new DatabaseBackup(db, app);
      }
      const result = backupManager.createBackup();
      logger.logIpcResponse('db-backup-create', result.success, { requestId: reqId });
      return result;
    } catch (error) {
      logger.error('创建备份失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('db-backup-create', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  ipcMain.handle('db-backup-restore', async (event, backupPath) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('db-backup-restore', { requestId: reqId, backupPath });

    try {
      if (!backupManager) {
        backupManager = new DatabaseBackup(db, app);
      }
      const result = backupManager.restoreBackup(backupPath);

      if (result.success) {
        initDatabase();
      }

      logger.logIpcResponse('db-backup-restore', result.success, { requestId: reqId });
      return result;
    } catch (error) {
      logger.error('恢复备份失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('db-backup-restore', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  ipcMain.handle('db-backup-list', async () => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('db-backup-list', { requestId: reqId });

    try {
      if (!backupManager) {
        backupManager = new DatabaseBackup(db, app);
      }
      const result = backupManager.getBackupList();
      logger.logIpcResponse('db-backup-list', result.success, { requestId: reqId, count: result.backups.length });
      return result;
    } catch (error) {
      logger.error('获取备份列表失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('db-backup-list', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  ipcMain.handle('db-backup-delete', async (event, backupPath) => {
    const reqId = logger.generateRequestId();
    logger.logIpcRequest('db-backup-delete', { requestId: reqId, backupPath });

    try {
      if (!backupManager) {
        backupManager = new DatabaseBackup(db, app);
      }
      const result = backupManager.deleteBackup(backupPath);
      logger.logIpcResponse('db-backup-delete', result.success, { requestId: reqId });
      return result;
    } catch (error) {
      logger.error('删除备份失败', { requestId: reqId, error: error.message }, error);
      logger.logIpcResponse('db-backup-delete', false, { requestId: reqId, error: error.message });
      throw error;
    }
  });

  logger.info('IPC处理器设置完成', { requestId });
}

// App 生命周期
app.whenReady().then(() => {
  logger.info('应用就绪', { version: app.getVersion(), platform: process.platform });

  initDatabase();
  createWindow();
  setupIpcHandlers();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      logger.info('激活事件：重新创建窗口');
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  logger.info('所有窗口关闭');

  if (process.platform !== 'darwin') {
    if (db) {
      db.close();
      logger.info('数据库连接已关闭');
    }
    app.quit();
  }
});

app.on('before-quit', () => {
  logger.info('应用即将退出');

  // 清理TCP连接
  if (tcpSocket) {
    tcpSocket.destroy();
    logger.logTcpConnection('应用退出时断开', { host: tcpHost, port: tcpPort });
  }

  // 关闭数据库
  if (db) {
    db.close();
  }
});

// 创建菜单
function createMenu() {
  const template = [
    {
      label: '文件',
      submenu: [
        {
          label: '刷新',
          accelerator: 'CmdOrCtrl+R',
          click: () => {
            logger.info('用户请求刷新应用');
            mainWindow?.reload();
          },
        },
        {
          label: '开发者工具',
          accelerator: 'CmdOrCtrl+Shift+I',
          click: () => {
            logger.info('用户请求打开开发者工具');
            mainWindow?.webContents.toggleDevTools();
          },
        },
        { type: 'separator' },
        {
          label: '退出',
          accelerator: 'CmdOrCtrl+Q',
          click: () => {
            logger.info('用户请求退出应用');
            app.quit();
          },
        },
      ],
    },
    {
      label: '帮助',
      submenu: [
        {
          label: '关于 CarpTMS',
          click: () => {
            logger.info('用户打开关于窗口');
            const aboutWindow = new BrowserWindow({
              width: 400,
              height: 300,
              parent: mainWindow,
              modal: true,
              frame: false,
              webPreferences: {
                preload: path.join(__dirname, 'preload.js'),
                nodeIntegration: false,
                contextIsolation: true,
              },
            });
            aboutWindow.loadFile(path.join(__dirname, '../about.html'));
          },
        },
      ],
    },
  ];

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
  logger.info('应用菜单创建完成');
}


