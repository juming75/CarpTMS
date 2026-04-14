import { app, BrowserWindow, ipcMain, Menu, shell } from 'electron';
import path from 'path';
import { fileURLToPath } from 'url';
import Database from 'better-sqlite3';
import fs from 'fs';
import { exec } from 'child_process';
import * as net from 'net';

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
      return 0;
    }
  }

  migrate() {
    const currentVersion = this.getCurrentVersion();
    console.log(`Current database version: ${currentVersion}`);

    const pendingMigrations = this.migrations.filter((m) => m.version > currentVersion);

    if (pendingMigrations.length === 0) {
      console.log('Database is up to date');
      return;
    }

    console.log(`Found ${pendingMigrations.length} pending migrations`);

    this.db.exec('BEGIN TRANSACTION;');

    try {
      for (const migration of pendingMigrations) {
        console.log(`Applying migration version ${migration.version}...`);
        this.db.exec(migration.up);

        this.db.prepare('INSERT INTO migrations (version) VALUES (?)').run(migration.version);
        console.log(`Migration version ${migration.version} applied successfully`);
      }

      this.db.exec('COMMIT TRANSACTION;');
      console.log('All migrations applied successfully');
    } catch (error) {
      this.db.exec('ROLLBACK TRANSACTION;');
      console.error('Migration failed:', error);
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
    }
  }

  createBackup() {
    this.initBackupDir();

    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const backupPath = path.join(this.backupDir, `CarpTMS_backup_${timestamp}.db`);
    const dbPath = path.join(this.app.getPath('userData'), 'CarpTMS_local.db');

    try {
      fs.copyFileSync(dbPath, backupPath);
      console.log(`Backup created: ${backupPath}`);

      this.cleanupOldBackups();

      return {
        success: true,
        backupPath,
        message: '备份创建成功',
      };
    } catch (error) {
      console.error('Backup failed:', error);
      return {
        success: false,
        error: error.message,
        message: '备份创建失败',
      };
    }
  }

  restoreBackup(backupPath) {
    const dbPath = path.join(this.app.getPath('userData'), 'CarpTMS_local.db');

    try {
      if (this.db && this.db.open) {
        this.db.close();
      }

      fs.copyFileSync(backupPath, dbPath);

      this.db = new Database(dbPath);

      console.log(`Backup restored: ${backupPath}`);
      return {
        success: true,
        message: '备份恢复成功',
      };
    } catch (error) {
      console.error('Restore failed:', error);
      return {
        success: false,
        error: error.message,
        message: '备份恢复失败',
      };
    }
  }

  getBackupList() {
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

      return {
        success: true,
        backups: files,
      };
    } catch (error) {
      console.error('Failed to get backup list:', error);
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
          console.log(`Deleted old backup: ${backup.filePath}`);
        } catch (error) {
          console.error(`Failed to delete old backup ${backup.filePath}:`, error);
        }
      }
    }
  }

  deleteBackup(backupPath) {
    try {
      fs.unlinkSync(backupPath);
      console.log(`Deleted backup: ${backupPath}`);
      return {
        success: true,
        message: '备份删除成功',
      };
    } catch (error) {
      console.error(`Failed to delete backup ${backupPath}:`, error);
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
  } else {
    mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
  }

  mainWindow.on('closed', () => {
    mainWindow = null;
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: 'deny' };
  });
}

// 初始化数据库
function initDatabase() {
  try {
    const dbPath = path.join(app.getPath('userData'), 'CarpTMS_local.db');
    db = new Database(dbPath);

    const migration = new DatabaseMigration(db);
    migration.migrate();

    backupManager = new DatabaseBackup(db, app);

    console.log('Database initialized successfully');
  } catch (error) {
    console.error('Failed to initialize database:', error);
  }
}

// TCP连接函数
function connectTcp(host, port) {
  return new Promise((resolve) => {
    console.log('TCP连接中:', { host, port });

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
      console.error('TCP连接超时');
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
      console.log('TCP连接成功');
      tcpConnected = true;
      resolve(true);
    });

    // 连接关闭
    tcpSocket.on('close', () => {
      clearTimeout(timeout);
      console.log('TCP连接关闭');
      tcpConnected = false;
    });

    // 连接错误
    tcpSocket.on('error', (error) => {
      clearTimeout(timeout);
      console.error('TCP连接错误:', error);
      tcpConnected = false;
      resolve(false);
    });

    // 接收数据
    tcpSocket.on('data', (data) => {
      console.log('TCP接收数据:', data.length, 'bytes');
      // 简单处理：直接解析为JSON
      try {
        const jsonStr = data.toString('utf8');
        const response = JSON.parse(jsonStr);
        console.log('TCP接收到响应:', response);

        // 处理消息队列
        if (tcpMessageQueue.length > 0) {
          const { resolve } = tcpMessageQueue.shift();
          resolve(response);
        }
      } catch (error) {
        console.error('TCP数据解析失败:', error);
        // 处理消息队列
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
  return new Promise((resolve, reject) => {
    console.log('TCP发送数据:', data);

    if (!tcpConnected || !tcpSocket) {
      console.log('TCP未连接，尝试重连...');
      tcpMessageQueue.push({ data, resolve, reject });

      connectTcp(tcpHost, tcpPort).then((connected) => {
        if (!connected) {
          console.error('TCP重连失败');
          reject(new Error('无法连接到服务器'));
        }
      });
      return;
    }

    try {
      // 简单处理：发送JSON字符串
      const jsonData = JSON.stringify(data);
      tcpSocket.write(jsonData);

      // 加入消息队列等待响应
      tcpMessageQueue.push({ data, resolve, reject });

      // 设置响应超时
      setTimeout(() => {
        if (tcpMessageQueue.length > 0) {
          const { reject } = tcpMessageQueue.shift();
          reject(new Error('发送超时'));
        }
      }, 30000);
    } catch (error) {
      console.error('TCP发送数据失败:', error);
      reject(error);
    }
  });
}

// 设置 IPC 处理器
function setupIpcHandlers() {
  // ====== TCP客户端操作 ======
  ipcMain.handle('tcp-init', async (event, host, port) => {
    try {
      console.log('初始化TCP客户端:', { host, port });
      tcpHost = host;
      tcpPort = port;
      return { success: true };
    } catch (error) {
      console.error('初始化TCP客户端失败:', error);
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-connect', async () => {
    try {
      console.log('TCP客户端连接中...');
      const connected = await connectTcp(tcpHost, tcpPort);
      return { success: true, connected };
    } catch (error) {
      console.error('TCP客户端连接失败:', error);
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-send', async (event, data) => {
    try {
      console.log('TCP客户端发送数据:', data);
      const response = await sendTcpData(data);
      return { success: true, response };
    } catch (error) {
      console.error('TCP客户端发送数据失败:', error);
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-is-connected', async () => {
    try {
      return { success: true, connected: tcpConnected };
    } catch (error) {
      console.error('检查TCP连接状态失败:', error);
      return { success: false, error: error.message };
    }
  });

  ipcMain.handle('tcp-disconnect', async () => {
    try {
      if (tcpSocket) {
        tcpSocket.destroy();
        tcpSocket = null;
        tcpConnected = false;
      }
      return { success: true };
    } catch (error) {
      console.error('TCP客户端断开连接失败:', error);
      return { success: false, error: error.message };
    }
  });

  // ====== 车辆管理 ======
  ipcMain.handle('get-local-vehicles', async () => {
    const stmt = db.prepare('SELECT * FROM local_vehicles ORDER BY create_time DESC');
    return stmt.all();
  });

  ipcMain.handle('save-local-vehicle', async (event, vehicle) => {
    let version = 0;
    let sync_conflict = 0;
    let conflict_reason = null;

    const existingStmt = db.prepare('SELECT version, sync_version, server_id FROM local_vehicles WHERE vehicle_id = ?');
    const existing = existingStmt.get(vehicle.vehicle_id);

    if (existing) {
      version = existing.version + 1;

      if (existing.sync_version && existing.sync_version > existing.version) {
        sync_conflict = 1;
        conflict_reason = '服务器版本高于本地版本';
      }
    }

    const stmt = db.prepare(`
      INSERT OR REPLACE INTO local_vehicles 
      (vehicle_id, vehicle_name, device_id, own_no, own_name, own_phone, group_id, is_synced, server_id, 
       version, last_modified, sync_version, sync_conflict, conflict_reason, create_time, update_time)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), ?, ?, ?, datetime('now'), datetime('now'))
    `);
    return stmt.run(
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
  });

  ipcMain.handle('delete-local-vehicle', async (event, vehicleId) => {
    const stmt = db.prepare('DELETE FROM local_vehicles WHERE vehicle_id = ?');
    return stmt.run(vehicleId);
  });

  // ====== 称重数据 ======
  ipcMain.handle('get-local-weighing-data', async (event, limit = 100) => {
    const stmt = db.prepare(`
      SELECT * FROM local_weighing_data 
      ORDER BY create_time DESC 
      LIMIT ?
    `);
    return stmt.all(limit);
  });

  ipcMain.handle('save-local-weighing-data', async (event, data) => {
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
        }
      }

      const stmt = db.prepare(`
        UPDATE local_weighing_data 
        SET vehicle_id = ?, device_id = ?, weighing_time = ?, gross_weight = ?, tare_weight = ?, 
            net_weight = ?, axle_count = ?, speed = ?, lane_no = ?, site_id = ?, status = ?, 
            version = ?, last_modified = datetime('now'), sync_conflict = ?, conflict_reason = ?
        WHERE id = ?
      `);
      return stmt.run(
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
    } else {
      const stmt = db.prepare(`
        INSERT INTO local_weighing_data 
        (vehicle_id, device_id, weighing_time, gross_weight, tare_weight, net_weight, 
         axle_count, speed, lane_no, site_id, status, version, last_modified)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
      `);
      return stmt.run(
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
    }
  });

  // ====== 数据同步 ======
  ipcMain.handle('get-unsynced-data', async () => {
    const vehiclesStmt = db.prepare('SELECT * FROM local_vehicles WHERE is_synced = 0');
    const weighingStmt = db.prepare('SELECT * FROM local_weighing_data WHERE is_synced = 0');

    return {
      vehicles: vehiclesStmt.all(),
      weighingData: weighingStmt.all(),
    };
  });

  ipcMain.handle('mark-as-synced', async (event, { type, ids, serverIds, serverVersions }) => {
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
    }

    return { success: true, conflictCount };
  });

  // ====== 同步日志 ======
  ipcMain.handle('log-sync', async (event, log) => {
    const stmt = db.prepare(`
      INSERT INTO sync_logs (sync_type, record_count, status, error_message, conflict_count)
      VALUES (?, ?, ?, ?, ?)
    `);
    return stmt.run(log.syncType, log.recordCount, log.status, log.errorMessage || null, log.conflictCount || 0);
  });

  // ====== 数据库信息 ======
  ipcMain.handle('get-db-info', async () => {
    const vehicleCount = db.prepare('SELECT COUNT(*) as count FROM local_vehicles').get().count;
    const weighingCount = db.prepare('SELECT COUNT(*) as count FROM local_weighing_data').get().count;

    return {
      path: db.open ? 'connected' : 'disconnected',
      vehicleCount,
      weighingCount,
    };
  });

  // ====== 系统信息 ======
  ipcMain.handle('get-app-version', async () => {
    return app.getVersion();
  });

  ipcMain.handle('get-electron-version', async () => {
    return process.versions.electron;
  });

  // ====== 数据备份 ======
  ipcMain.handle('db-backup-create', async () => {
    if (!backupManager) {
      backupManager = new DatabaseBackup(db, app);
    }
    return backupManager.createBackup();
  });

  ipcMain.handle('db-backup-restore', async (event, backupPath) => {
    if (!backupManager) {
      backupManager = new DatabaseBackup(db, app);
    }
    const result = backupManager.restoreBackup(backupPath);

    if (result.success) {
      initDatabase();
    }

    return result;
  });

  ipcMain.handle('db-backup-list', async () => {
    if (!backupManager) {
      backupManager = new DatabaseBackup(db, app);
    }
    return backupManager.getBackupList();
  });

  ipcMain.handle('db-backup-delete', async (event, backupPath) => {
    if (!backupManager) {
      backupManager = new DatabaseBackup(db, app);
    }
    return backupManager.deleteBackup(backupPath);
  });
}

// App 生命周期
app.whenReady().then(() => {
  initDatabase();
  createWindow();
  setupIpcHandlers();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    if (db) {
      db.close();
    }
    app.quit();
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
            mainWindow?.reload();
          },
        },
        {
          label: '开发者工具',
          accelerator: 'CmdOrCtrl+Shift+I',
          click: () => {
            mainWindow?.webContents.toggleDevTools();
          },
        },
        { type: 'separator' },
        {
          label: '退出',
          accelerator: 'CmdOrCtrl+Q',
          click: () => {
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
}


