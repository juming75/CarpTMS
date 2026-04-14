@echo off
REM CarpTMS 启动脚本（包含 DeepSeek AI 服务）

cd /d "%~dp0"
echo 正在启动 CarpTMS 系统（包含 DeepSeek AI 服务）...
echo ======================================
echo 启动 Docker Compose 服务...
docker-compose -f docker-compose.yml up -d --build

if %errorlevel% equ 0 (
    echo Docker Compose 服务启动成功！
    echo 
    echo 服务访问地址：
    echo - 前端：http://localhost
    echo - 后端 API：http://localhost:8082
    echo - DeepSeek API：http://localhost:8000
    echo - Swagger 文档：http://localhost:8082/docs
    echo 
    echo 正在等待服务启动完成...
    timeout /t 10 /nobreak >nul
    
    echo 
    echo 服务状态检查：
    echo - 前端：
    curl -s http://localhost/health || echo 前端服务未就绪
    echo - 后端：
    curl -s http://localhost:8082/api/health || echo 后端服务未就绪
    echo - DeepSeek：
    curl -s http://localhost:8000/v1/models || echo DeepSeek 服务未就绪
    echo 
    echo 启动完成！
) else (
    echo 启动失败，请检查错误信息。
    pause
)


