# CarpTMS Client

动态车载称重系统 - Electron + Vue 3 桌面客户端

## 技术栈

- **前端框架**: Vue 3 + TypeScript
- **桌面框架**: Electron
- **UI组件库**: Element Plus
- **状态管理**: Pinia
- **路由**: Vue Router
- **图表**: ECharts
- **本地数据库**: SQLite (better-sqlite3)
- **HTTP客户端**: Axios

## 项目结构

```
My_client/
├── electron/              # Electron 主进程
│   ├── main.js          # 主进程入口
│   └── preload.js       # 预加载脚本
├── src/                 # Vue 3 源代码
│   ├── api/            # API 调用封装
│   ├── components/      # 通用组件
│   ├── layout/         # 布局组件
│   ├── services/       # 本地服务（SQLite）
│   ├── types/          # TypeScript 类型
│   ├── views/          # 页面视图
│   ├── router/         # 路由配置
│   ├── App.vue         # 根组件
│   └── main.ts         # 应用入口
├── package.json
├── vite.config.ts
├── tsconfig.json
└── start.bat          # Windows 启动脚本
```

## 快速开始

### 1. 安装依赖

```bash
npm install
```

### 2. 开发运行

**Windows:**

```bash
start.bat
```

**或者手动运行:**

```bash
npm run electron:dev
```

### 3. 打包发布

```bash
npm run electron:build
```

打包后的文件位于 `dist_electron/` 目录。

## 功能模块

- [x] 用户登录
- [x] 仪表盘（数据统计、图表）
- [x] 车辆管理（CRUD、批量操作）
- [ ] 实时监控（开发中）
- [ ] 历史数据查询（开发中）
- [ ] 报表中心（开发中）
- [ ] 系统设置（开发中）
- [ ] 数据同步（开发中）

## API 通信

前端通过 Axios 与后端 `My_server` (Rust + Actix) 通信：

```typescript
// API 基础地址: http://localhost:8080
import { vehicleApi } from '@/api';

// 获取车辆列表
const vehicles = await vehicleApi.getAll();

// 创建车辆
const vehicle = await vehicleApi.create({
  vehicle_name: '测试车辆',
  device_id: 'DEV001',
  own_no: '京A12345',
  group_id: 1,
});
```

## 本地数据缓存

使用 SQLite 本地数据库实现离线功能：

```typescript
import { localVehicleService } from '@/services/localDB';

// 获取本地车辆
const vehicles = await localVehicleService.getAll();

// 保存到本地
await localVehicleService.save(vehicle);
```

## 数据同步

本地数据与服务器同步机制：

1. 客户端将数据保存到本地 SQLite
2. 定期/手动触发同步到服务器
3. 同步成功后更新本地数据状态

## 配置说明

### Electron Builder 配置

- **应用ID**: `com.CarpTMS.client`
- **应用名称**: `CarpTMS Client`
- **输出目录**: `dist_electron/`
- **Windows 目标**: NSIS 安装包 + 便携版

### Vite 配置

- **开发服务器端口**: 5173
- **API 代理**: `/api` -> `http://localhost:8080`
- **打包输出**: `dist/`

## 开发注意事项

### 1. Electron 安全性

- 使用 `contextIsolation: true` 隔离上下文
- 通过 `preload.js` 安全地暴露 API
- 禁用 `nodeIntegration`

### 2. SQLite 使用

- 只在 Electron 环境中使用
- 渲染进程通过 IPC 与主进程通信
- 使用 `better-sqlite3` 同步 API

### 3. 路由模式

使用 `createWebHashHistory` 避免路由问题：

```typescript
const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
```

## 常见问题

### 1. 依赖安装失败

```bash
# 清除缓存重新安装
npm cache clean --force
rm -rf node_modules
npm install
```

### 2. Electron 启动失败

确保安装了 Node.js 和 npm：

```bash
node --version
npm --version
```

### 3. SQLite 数据库错误

确保 `better-sqlite3` 正确编译：

```bash
npm rebuild better-sqlite3
```

## 后续开发计划

- [ ] 实现实时 WebSocket 监控
- [ ] 完善数据同步机制
- [ ] 添加地图定位功能
- [ ] 优化报表生成和导出
- [ ] 实现离线模式
- [ ] 添加系统托盘功能

## 版本信息

- **当前版本**: 1.0.0
- **Electron 版本**: 33.2.0
- **Node.js 要求**: >= 18.0.0

---

**技术支持**: CarpTMS 开发团队


