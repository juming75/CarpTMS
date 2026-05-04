# CarpTMS 车联网运输管理系统

[![Apache 2.0 License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/Rust-1.94-blue.svg)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

## 项目简介

CarpTMS 是一套**完全开放**的车联网为主的低空经济运营平台。设计目标为支持 **5万台** 危化与普货车辆监管和调度，预留了 **500-1000架无人机** 的管理代码。

本项目采用 Apache License 2.0 开源，欢迎大家二次开发并维护好自己的客户，在这个变动的世界里找到一条自己的活路。

## 功能特点

### 核心功能
- **车辆实时监控**：支持JT1078、GB28181等多种视频流协议
- **多协议支持**：HTTP-FLV、HLS、WebRTC、RTMP等流媒体输出
- **高效存储**：支持历史轨迹回放和视频录像存储
- **数据分析**：内置AI模块，支持运输数据分析

### 技术特性
- **高性能**：基于Rust异步运行时，设计支持大规模并发
- **信创适配**：完美适配ARM/龙芯/申威等国产信创平台
- **AI推理**：内置HuggingFace Candle本地AI推理能力
- **容器化部署**：支持Docker/Kubernetes一键部署

## 技术栈

### 后端技术
| 技术 | 用途 |
|------|------|
| Rust 1.94 | 系统编程语言 |
| Actix-web 4 | Web框架 |
| Tokio | 异步运行时 |
| SQLx | 数据库访问 |
| Redis | 缓存层 |
| PostgreSQL | 主数据库 |

### 前端技术
| 技术 | 用途 |
|------|------|
| Vue 3 | 前端框架 |
| TypeScript | 类型安全 |
| Pinia | 状态管理 |
| Element Plus | UI组件库 |

### 基础设施
| 技术 | 用途 |
|------|------|
| Docker | 容器化 |
| Kubernetes | 编排 |
| Nginx | 反向代理 |
| Prometheus | 监控 |
| Grafana | 可视化 |

## 快速开始

### 环境要求

- Rust 1.94+
- PostgreSQL 14+
- Redis 6+
- Node.js 18+ (前端)

### 后端编译

```bash
cd My_server

# 安装依赖
cargo build

# 开发模式
cargo run

# 生产模式（CPU AI）
cargo build --release

# GPU加速模式
cargo build --release --features ai-gpu
```

### 前端部署

```bash
cd My_client

# 安装依赖
npm install

# 开发模式
npm run dev

# 生产构建
npm run build
```

### Docker部署

```bash
# 使用docker-compose一键启动
docker-compose up -d

# 仅启动后端
docker-compose up -d carptms-server
```

## 项目结构

```
CarpTMS/
├── My_server/           # 后端服务 (Rust)
│   ├── src/
│   │   ├── bin/         # 可执行入口
│   │   ├── video/       # 视频流处理
│   │   ├── domain/      # 领域模型
│   │   ├── bff/         # 前端适配层
│   │   └── ...
│   ├── migrations/      # 数据库迁移
│   └── Cargo.toml
│
├── My_client/           # 前端应用 (Vue)
│   ├── src/
│   │   ├── components/  # Vue组件
│   │   ├── views/       # 页面视图
│   │   └── ...
│   └── package.json
│
├── scripts/             # 部署脚本
├── k8s/                 # Kubernetes配置
├── docs/                # 文档
└── LICENSE              # Apache 2.0许可证
```

## 数据库初始化

```bash
# 1. 创建数据库
psql -U postgres -c "CREATE DATABASE carptms;"

# 2. 运行迁移
cd My_server
psql -U postgres -d carptms -f migrations/20260101000001_create_vehicles_table.sql
psql -U postgres -d carptms -f migrations/20260101000002_create_orders_tables.sql
# ... 其他迁移文件

# 3. 初始化基础数据
psql -U postgres -d carptms -f ../create_database.sql
```

## 配置说明

### 后端配置 (config.yaml)

```yaml
database:
  host: "localhost"
  port: 5432
  username: "postgres"
  password: "your_password"
  name: "carptms"

redis:
  host: "localhost"
  port: 6379

app:
  host: "0.0.0.0"
  port: 8080
```

### 环境变量

```bash
# 数据库配置
DATABASE_URL=postgres://postgres:password@localhost:5432/carptms

# Redis配置
REDIS_URL=redis://localhost:6379

# JWT密钥
JWT_SECRET=your-secret-key

# 日志级别
RUST_LOG=info
```

## API文档

启动服务后访问：`http://localhost:8080/swagger-ui/`

## 性能基准

| 指标 | 数值 |
|------|------|
| 最大车辆接入 | 50,000+ |
| 并发视频流 | 10,000+ |
| 消息吞吐量 | 100,000 msg/s |
| 平均响应时间 | < 50ms |

## 社区与支持

- **问题反馈**：[GitHub Issues](https://github.com/your-repo/CarpTMS/issues)
- **技术讨论**：[GitHub Discussions](https://github.com/your-repo/CarpTMS/discussions)

## 贡献指南

欢迎提交 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 开源许可证

本项目基于 **Apache License 2.0** 开源 - 详见 [LICENSE](LICENSE) 文件

```
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

## 版本历史

### v1.1.0 (2026-05-04)
- ✨ 完整生产就绪版本
- 🐛 添加首次登录密码修改功能
- 🐛 完善标定管理功能
- 🐛 集成Qwen3.5,强化了大规模车辆排班调度的内在能力
- 📦 优化数据库连接配置
- 📝 完善部署文档

### v1.0.0 (历史版本)
- 🎉 基础功能版本

---

**CarpTMS - 让运输更智能**
