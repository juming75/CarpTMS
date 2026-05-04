# CarpTMS 生产部署指南

## 支持平台

| 平台 | 部署方式 | 脚本 |
|------|---------|------|
| Windows Server 2019/2022 | Windows Service + PowerShell | `deploy-windows-server.ps1` |
| 银河麒麟 V10 | systemd + Bash | `deploy-kylin.sh` |
| Linux (通用) | systemd/Docker | `docker-compose.prod.yml` |
| Kubernetes | Helm Chart | `deploy/helm/` |

---

## 快速开始

### 1. 生成安全密钥

```powershell
# Windows
.\generate-secrets.ps1 | Out-File secrets.env -Encoding UTF8

# 加载密钥到环境变量
Get-Content secrets.env | ForEach-Object {
    if($_ -match '^([A-Z_]+)=(.+)$') {
        [Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process')
    }
}
```

### 2. 部署到 Windows Server

```powershell
# 以管理员身份运行 PowerShell
.\deploy-windows-server.ps1 -Environment production -InstallServices

# 启动服务
Start-Service CarpTMS

# 查看日志
Get-Content C:\CarpTMS\logs\app.log -Tail 100 -Wait
```

### 3. 部署到银河麒麟

```bash
# 设置环境变量
export CARPTMS_DB_PASSWORD="your_secure_password"
export CARPTMS_JWT_SECRET="your_64+_char_random_string"
export CARPTMS_ENCRYPTION_KEY="your_64_hex_char_key"
# ... 其他密钥

# 执行部署
sudo chmod +x deploy-kylin.sh
sudo ./deploy-kylin.sh

# 启动服务
sudo systemctl start carptms

# 查看日志
sudo journalctl -u carptms -f
```

### 4. Docker Compose 部署

```bash
# 准备环境变量文件
cp My_server/.env.production .env
# 编辑 .env 填入实际值

# 启动全部服务
docker-compose -f docker-compose.prod.yml up -d

# 查看状态
docker-compose -f docker-compose.prod.yml ps
```

---

## 环境变量清单

### 必需变量（生产环境必须设置）

| 变量名 | 说明 | 格式要求 |
|--------|------|---------|
| `CARPTMS_DB_PASSWORD` | 数据库密码 | 16+字符 |
| `CARPTMS_REDIS_PASSWORD` | Redis密码 | 任意 |
| `CARPTMS_JWT_SECRET` | JWT签名密钥 | 64+字符 |
| `CARPTMS_JWT_REFRESH_SECRET` | JWT刷新密钥 | 64+字符 |
| `CARPTMS_ENCRYPTION_KEY` | AES加密密钥 | 64位hex |

### 可选变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `CARPTMS_DB_HOST` | localhost | 数据库主机 |
| `CARPTMS_DB_PORT` | 5432 | 数据库端口 |
| `CARPTMS_REDIS_HOST` | localhost | Redis主机 |
| `CARPTMS_PORT` | 8082 | 服务端口 |
| `CARPTMS_LOG_LEVEL` | info | 日志级别 |
| `CARPTMS_WORKER_THREADS` | 8 | 工作线程数 |

---

## 安全配置检查清单

- [ ] 所有密钥已替换为随机生成的强密钥
- [ ] `.env` 文件权限设置为仅管理员/所有者可读 (Windows: ACL, Linux: 600)
- [ ] 数据库使用独立用户，非超级用户
- [ ] Redis 已启用密码认证
- [ ] JWT 密钥长度 >= 64 字符
- [ ] 加密密钥为 64 位 hex (32字节)
- [ ] TLS/SSL 证书已配置（生产环境）
- [ ] 防火墙已开放必要端口 (8082, 5432, 6379)
- [ ] 日志已配置轮转，防止磁盘占满

---

## 故障排查

### Windows Server

```powershell
# 服务无法启动
Get-WinEvent -FilterHashtable @{LogName='Application'; ID=1000} -MaxEvents 10

# 查看详细日志
Get-Content C:\CarpTMS\logs\app.log -Tail 200

# 检查端口占用
Get-NetTCPConnection -LocalPort 8082
```

### 银河麒麟/Linux

```bash
# 服务无法启动
sudo journalctl -u carptms -n 100 --no-pager

# 检查端口监听
sudo ss -tlnp | grep 8082

# 检查文件权限
ls -la /opt/carptms/backend/.env
```

---

## 升级维护

### 热更新（不中断服务）

```powershell
# Windows
.\deploy-windows-server.ps1 -OnlyUpdate
Restart-Service CarpTMS

# 银河麒麟
sudo systemctl reload carptms
```

### 数据库迁移

```bash
cd My_server
cargo run --bin db_init -- migrate
```

---

## 监控告警

部署完成后，访问以下地址：

| 服务 | 地址 | 默认账号 |
|------|------|---------|
| Grafana | http://localhost:3000 | admin/admin |
| Prometheus | http://localhost:9090 | - |
| 应用健康检查 | http://localhost:8082/api/health | - |

---

*文档版本: 2.0.0 | 更新日期: 2026-05-01*
