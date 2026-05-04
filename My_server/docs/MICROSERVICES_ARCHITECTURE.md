# CarpTMS 微服务架构技术文档

## 目录

1. [架构概述](#架构概述)
2. [服务划分](#服务划分)
3. [服务间通信](#服务间通信)
4. [数据管理](#数据管理)
5. [部署架构](#部署架构)
6. [监控与运维](#监控与运维)
7. [安全设计](#安全设计)
8. [性能优化](#性能优化)

## 架构概述

### 架构演进

CarpTMS 从单体架构演进为微服务架构，主要驱动力包括：

- **业务复杂度增长**：随着业务功能增加，单体代码库变得难以维护
- **团队规模扩大**：需要支持多团队并行开发
- **可扩展性需求**：不同服务有不同的扩展需求
- **技术栈灵活性**：允许各服务采用最适合的技术栈

### 架构原则

1. **单一职责**：每个服务只负责一个明确的业务领域
2. **独立部署**：服务可以独立构建、测试和部署
3. **故障隔离**：单个服务故障不影响整个系统
4. **数据隔离**：每个服务管理自己的数据存储
5. **API优先**：服务间通过明确定义的API通信

## 服务划分

### 核心服务

| 服务名称 | 职责 | 端口 | 技术栈 |
|---------|------|------|--------|
| vehicle-service | 车辆管理 | 8083 | Rust + Actix Web |
| cargo-service | 货物管理 | 8084 | Rust + Actix Web |
| trip-service | 行程管理 | 8085 | Rust + Actix Web |
| billing-service | 计费管理 | 8086 | Rust + Actix Web |
| user-service | 用户管理 | 8087 | Rust + Actix Web |
| device-service | 设备管理 | 8088 | Rust + Actix Web |
| finance-service | 财务管理 | 8089 | Rust + Actix Web |
| location-service | 位置服务 | 8090 | Rust + Actix Web |
| weighing-service | 称重服务 | 8091 | Rust + Actix Web |
| alerts-service | 告警服务 | 8092 | Rust + Actix Web |

### 服务依赖图

```
                    ┌─────────────────┐
                    │   API Gateway   │
                    │     (Nginx)     │
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
   ┌────┴────┐         ┌────┴────┐         ┌────┴────┐
   │  User   │         │ Vehicle │         │  Cargo  │
   │ Service │         │ Service │         │ Service │
   └────┬────┘         └────┬────┘         └────┬────┘
        │                    │                    │
        └────────────────────┼────────────────────┘
                             │
                    ┌────────┴────────┐
                    │  Shared Services │
                    │  (Auth, Config)  │
                    └─────────────────┘
```

## 服务间通信

### 同步通信

**REST API**
- 用于实时请求-响应场景
- 使用 JSON 格式
- 支持 HTTP/2

**gRPC** (规划中)
- 用于高性能内部通信
- 使用 Protocol Buffers
- 支持双向流

### 异步通信

**消息队列**
- RabbitMQ / Kafka
- 用于事件驱动架构
- 服务解耦

**事件类型**
- `VehicleCreated` - 车辆创建
- `OrderCreated` - 订单创建
- `WeighingCompleted` - 称重完成
- `AlertGenerated` - 告警生成

### 服务发现

使用 Consul / Eureka 进行服务注册和发现：

```yaml
# 服务注册示例
service:
  name: vehicle-service
  port: 8083
  health-check:
    path: /health
    interval: 30s
```

## 数据管理

### 数据库策略

**每个服务一个数据库**

| 服务 | 数据库 | 用途 |
|------|--------|------|
| vehicle-service | PostgreSQL | 车辆数据 |
| user-service | PostgreSQL | 用户数据 |
| location-service | PostgreSQL + PostGIS | 位置数据 |
| device-service | PostgreSQL | 设备数据 |

**缓存策略**

- Redis Cluster 作为分布式缓存
- 本地缓存用于热点数据
- 缓存更新策略：Cache-Aside

### 数据一致性

**Saga 模式**
- 用于分布式事务
- 补偿事务处理失败

**事件溯源**
- 关键业务事件持久化
- 支持状态重建

## 部署架构

### 容器化

**Docker 镜像**

```dockerfile
# 多阶段构建
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/carptms_server /usr/local/bin/
EXPOSE 8080
CMD ["carptms_server"]
```

**Docker Compose**

```yaml
# 微服务部署
version: '3.8'
services:
  vehicle-service:
    image: carptms/vehicle-service:latest
    ports:
      - "8083:8083"
    environment:
      - DATABASE_URL=postgres://...
```

### Kubernetes 部署 (规划中)

```yaml
# Deployment 示例
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vehicle-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: vehicle-service
  template:
    metadata:
      labels:
        app: vehicle-service
    spec:
      containers:
      - name: vehicle-service
        image: carptms/vehicle-service:latest
        ports:
        - containerPort: 8083
```

### CI/CD 流程

1. **代码提交** -> GitHub Actions
2. **自动测试** -> 单元测试 + 集成测试
3. **构建镜像** -> Docker Build
4. **推送镜像** -> Docker Registry
5. **部署到测试环境** -> Staging
6. **性能测试** -> K6 Load Test
7. **部署到生产环境** -> Production

## 监控与运维

### 监控体系

**指标收集**
- Prometheus - 指标存储
- Grafana - 可视化
- 自定义业务指标

**日志管理**
- ELK Stack (Elasticsearch, Logstash, Kibana)
- 结构化日志 (JSON)
- 分布式追踪 (Jaeger)

**告警机制**
- Alertmanager
- 分级告警 (Warning, Critical)
- 多渠道通知 (Email, Slack, SMS)

### 关键指标

**系统指标**
- CPU 使用率
- 内存使用率
- 磁盘 I/O
- 网络流量

**应用指标**
- 请求延迟 (P50, P95, P99)
- 错误率
- 吞吐量 (RPS)
- 并发连接数

**业务指标**
- 订单量
- 活跃用户数
- 车辆在线率
- 设备故障率

### 运维工具

```bash
# 查看服务状态
./scripts/start_microservices.ps1 status

# 查看日志
./scripts/start_microservices.ps1 logs vehicle-service

# 重启服务
./scripts/start_microservices.ps1 restart vehicle-service
```

## 安全设计

### 认证授权

**JWT Token**
- Access Token (15分钟)
- Refresh Token (7天)
- RSA256 签名

**权限控制**
- RBAC (Role-Based Access Control)
- 资源级别权限
- API 网关统一鉴权

### 网络安全

**TLS/SSL**
- 全链路 HTTPS
- 证书自动续期
- HSTS 配置

**网络隔离**
- 服务网格 (Service Mesh)
- 零信任网络
- 微分段

### 数据安全

**加密存储**
- 数据库字段加密
- 密钥管理服务 (KMS)
- 敏感数据脱敏

**传输加密**
- mTLS 服务间通信
- 证书轮换

## 性能优化

### 缓存策略

**多级缓存**

```
Client -> CDN -> Nginx Cache -> Application Cache -> Redis -> Database
```

**缓存更新**
- 主动更新
- 被动失效
- 定时刷新

### 数据库优化

**读写分离**
- 主库写操作
- 从库读操作
- 自动故障切换

**分库分表**
- 按时间分表
- 按地域分库
- 分布式 ID

### 异步处理

**消息队列**
- 削峰填谷
- 任务异步化
- 事件驱动

**批量处理**
- 批量写入
- 定时任务
- 数据同步

## 故障处理

### 熔断降级

**熔断器模式**
- 失败率阈值：50%
- 熔断时间：30秒
- 半开试探：5个请求

**降级策略**
- 返回默认值
- 使用缓存数据
- 简化功能

### 重试机制

**指数退避**
- 初始间隔：100ms
- 最大间隔：10s
- 最大重试：3次

### 限流策略

**令牌桶算法**
- 容量：1000
- 速率：100/秒
- 突发：200

## 扩展性设计

### 水平扩展

**无状态服务**
- 会话外置 (Redis)
- 配置中心
- 共享存储

**自动扩缩容**
- CPU 使用率 > 70%：扩容
- CPU 使用率 < 30%：缩容
- 最小副本数：2
- 最大副本数：10

### 数据扩展

**分片策略**
- 哈希分片
- 范围分片
- 列表分片

## 开发规范

### API 设计

**RESTful 规范**
- 资源命名：/api/vehicles
- HTTP 方法：GET, POST, PUT, DELETE
- 状态码：200, 201, 400, 401, 403, 404, 500

**版本控制**
- URL 版本：/api/v1/vehicles
- Header 版本：Accept: application/vnd.api.v1+json

### 代码规范

**Rust 规范**
- 遵循 Rustfmt
- Clippy 检查
- 文档注释

**DDD 实践**
- 领域实体
- 应用服务
- 仓储模式

## 迁移指南

### 从单体到微服务

**阶段一：准备**
1. 代码模块化
2. API 网关搭建
3. 监控体系建立

**阶段二：拆分**
1. 识别边界上下文
2. 逐个服务拆分
3. 双写数据同步

**阶段三：优化**
1. 性能调优
2. 故障演练
3. 文档完善

## 附录

### 参考文档

- [The Twelve-Factor App](https://12factor.net/)
- [Microservices Patterns](https://microservices.io/)
- [Domain-Driven Design](https://dddcommunity.org/)

### 工具链

- **开发**：Rust, VS Code, Git
- **构建**：Cargo, Docker, GitHub Actions
- **部署**：Docker Compose, Kubernetes
- **监控**：Prometheus, Grafana, ELK
- **测试**：K6, JMeter, Postman

### 联系方式

- 技术支持：support@carptms.com
- 文档反馈：docs@carptms.com
