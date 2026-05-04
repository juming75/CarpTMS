# CarpTMS 部署配置

> **⚠️ 安全提示**: 本文档不包含任何敏感信息。所有服务器凭证必须通过环境变量或安全的密钥管理系统注入。

## 前置条件

部署前请确保已准备好以下环境变量：

```bash
# 服务器连接
export SERVER_IP=<服务器IP地址>
export SERVER_USER=<服务器用户名>
export SERVER_PASS=<服务器密码>  # 建议使用 SSH 密钥认证

# 数据库连接
export DB_HOST=<数据库服务器IP>
export DB_PORT=<数据库端口, 默认5432>
export DB_USER=<数据库用户名>
export DB_PASSWORD=<数据库密码>
export DB_NAME=<数据库名称>

# 应用配置
export JWT_SECRET=<JWT密钥, 至少64字符>
export ENCRYPTION_KEY=<加密密钥, 32字节十六进制>
```

## 部署步骤

### 1. 前置准备

在系统服务器上创建部署目录：
```powershell
# 连接到服务器 (建议使用 SSH 密钥)
ssh $SERVER_USER@$SERVER_IP

# 创建目录
mkdir C:\CarpTMS
mkdir C:\CarpTMS\www
mkdir C:\CarpTMS\backend
mkdir C:\CarpTMS\logs
```

### 2. 配置环境变量

在服务器上创建 `.env` 文件：
```bash
# 数据库配置
DATABASE_URL=postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME

# JWT 配置
JWT_SECRET=$JWT_SECRET
JWT_EXPIRATION_HOURS=24

# 加密配置
ENCRYPTION_KEY=$ENCRYPTION_KEY

# 服务配置
RUST_LOG=info
SERVER_PORT=8082
```

### 3. 部署前端

将前端构建产物上传到系统服务器：
```powershell
# 在本机执行
scp -r My_client/dist/* $SERVER_USER@$SERVER_IP:C:\CarpTMS\www\
```

### 4. 部署后端

将后端可执行文件上传到系统服务器：
```powershell
# 编译后端（如尚未编译）
cd My_server
cargo build --release

# 上传可执行文件
scp target/release/carptms_server.exe $SERVER_USER@$SERVER_IP:C:\CarpTMS\backend\

# 上传配置文件
scp .env $SERVER_USER@$SERVER_IP:C:\CarpTMS\backend\
```

### 5. 配置 Windows 服务（可选）

在服务器上配置为 Windows 服务自动运行。

---

## 快速部署脚本

### deploy.bat - Windows 部署脚本

```batch
@echo off
setlocal

:: 从环境变量读取配置
set SERVER_IP=%CARPTMS_SERVER_IP%
set USER=%CARPTMS_SERVER_USER%
set PASSWORD=%CARPTMS_SERVER_PASS%

echo ========================================
echo CarpTMS 部署脚本
echo ========================================

echo.
echo [1/4] 编译后端...
cd My_server
cargo build --release

echo.
echo [2/4] 构建前端...
cd ..\My_client
npm run build

echo.
echo [3/4] 上传前端到服务器...
pscp -r -pw %PASSWORD% My_client\dist\* %USER%@%SERVER_IP%:\inetpub\wwwroot\carptms\

echo.
echo [4/4] 上传后端到服务器...
pscp -pw %PASSWORD% My_server\target\release\carptms_server.exe %USER%@%SERVER_IP%:C:\CarpTMS\backend\

echo.
echo ========================================
echo 部署完成!
echo ========================================
pause
```

> **注意**: 请勿在脚本中硬编码密码，使用环境变量或安全的密钥管理工具。

---

## 验证部署

部署完成后访问：
- 前端: `http://<SERVER_IP>/carptms/`
- 后端 API: `http://<SERVER_IP>:8082/api/health`

---

## 数据库配置

确保数据库服务器上已安装 PostgreSQL，并创建数据库：

```sql
-- 在数据库服务器上执行
CREATE DATABASE carptms_db;
CREATE USER carptms WITH PASSWORD :password;  -- 从环境变量获取
GRANT ALL PRIVILEGES ON DATABASE carptms_db TO carptms;
```

修改后端 `.env` 文件中的数据库连接：
```
DATABASE_URL=postgres://carptms:<password>@<db_host>:5432/carptms_db
```

---

## 安全建议

1. **使用 SSH 密钥认证**: 避免在脚本中存储明文密码
2. **启用 TLS/SSL**: 生产环境务必使用 HTTPS
3. **定期轮换密钥**: 建议每 90 天轮换一次 JWT 密钥
4. **启用审计日志**: 记录所有敏感操作
5. **配置防火墙**: 仅允许必要的端口访问

---

## AI 模型配置（可选）

### 概述

CarpTMS 支持本地 AI 推理功能，需要下载并放置 GGUF 格式的模型文件。由于模型文件较大（1GB - 6GB），未包含在源代码仓库中，需要用户自行下载。

### 支持的模型

| 模型名称 | 推荐用途 | 大小 | 下载地址 |
|----------|----------|------|----------|
| Qwen2.5-1.5B-Q4_K_M.gguf | 轻量级推理，适合边缘部署 | ~940MB | [HuggingFace](https://huggingface.co/models?search=Qwen2.5-1.5B-Q4_K_M.gguf) |
| Qwen3.5-9B-Q4_K_M.gguf | 高质量推理，适合服务器部署 | ~5.3GB | [HuggingFace](https://huggingface.co/models?search=Qwen3.5-9B-Q4_K_M.gguf) |

### 放置位置

下载模型文件后，将其放置在以下目录：

```
My_server/
└── models/
    ├── Qwen2.5-1.5B-Q4_K_M.gguf    # 轻量级模型（可选）
    └── Qwen3.5-9B-Q4_K_M.gguf      # 大型模型（可选）
```

### 配置启用

在 `.env` 文件中配置 AI 模型路径：

```bash
# AI 模型配置
AI_MODEL_PATH=./models/Qwen3.5-9B-Q4_K_M.gguf
AI_ENABLED=true
AI_THREADS=4
```

### 注意事项

1. **模型可选**：AI 功能为可选功能，不安装模型不影响核心业务功能
2. **硬件要求**：
   - 1.5B 模型：建议至少 4GB 内存
   - 9B 模型：建议至少 16GB 内存
3. **下载建议**：使用支持断点续传的下载工具
4. **模型格式**：仅支持 GGUF 格式的量化模型

### 验证 AI 功能

启动服务后，访问以下接口验证 AI 功能：

```bash
# 测试 AI 摘要功能
curl -X POST http://localhost:8082/api/ai/summarize \
  -H "Content-Type: application/json" \
  -d '{"text": "这是一段测试文本"}'
```

> **提示**: 如果不打算使用 AI 功能，可以跳过此步骤，系统会自动禁用 AI 相关功能。
