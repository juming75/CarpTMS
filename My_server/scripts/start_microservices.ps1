#!/usr/bin/env pwsh

# 微服务启动脚本 (PowerShell 版本)
# 用于启动和管理各个微服务

# 服务名称列表
$SERVICES = @("vehicle-service", "cargo-service", "trip-service", "billing-service", "user-service", "device-service", "finance-service", "location-service", "weighing-service", "alerts-service")

# 服务端口映射
$PORTS = @(8083, 8084, 8085, 8086, 8087, 8088, 8089, 8090, 8091, 8092)

# 日志目录
$LOG_DIR = "./logs"
if (!(Test-Path $LOG_DIR)) {
    New-Item -ItemType Directory -Path $LOG_DIR | Out-Null
}

# 构建 release 版本
function Build-Release {
    Write-Host "Building release version..."
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!"
        exit 1
    }
    Write-Host "Build successful!"
}

# 启动所有微服务
function Start-AllServices {
    Write-Host "Starting all microservices..."
    
    # 确保 release 版本已构建
    $releasePath = "./target/release/carptms_server.exe"
    if (!(Test-Path $releasePath)) {
        Write-Host "Release binary not found. Building..."
        Build-Release
    }
    
    for ($i = 0; $i -lt $SERVICES.Count; $i++) {
        $service = $SERVICES[$i]
        $port = $PORTS[$i]
        
        Write-Host "Starting $service on port $port..."
        
        # 设置环境变量
        $env:ARCHITECTURE_MODE = "micro_ddd"
        $env:SERVICE_NAME = $service
        $env:PORT = $port
        $env:CONFIG_FILE = "./config/micro_ddd.yaml"
        
        # 启动服务
        $logFile = "$LOG_DIR/${service}.log"
        $pidFile = "$LOG_DIR/${service}.pid"
        
        # 使用 Start-Process 启动服务
        $process = Start-Process -FilePath $releasePath -RedirectStandardOutput $logFile -RedirectStandardError $logFile -PassThru -WindowStyle Hidden
        
        # 保存进程ID
        $process.Id | Out-File -FilePath $pidFile
        
        Write-Host "$service started with PID $($process.Id)"
        Start-Sleep -Seconds 2
    }
    
    Write-Host "All microservices started successfully!"
    Write-Host ""
    Write-Host "Service endpoints:"
    for ($i = 0; $i -lt $SERVICES.Count; $i++) {
        Write-Host "  $($SERVICES[$i]): http://localhost:$($PORTS[$i])"
    }
}

# 停止所有微服务
function Stop-AllServices {
    Write-Host "Stopping all microservices..."
    
    foreach ($service in $SERVICES) {
        $pidFile = "$LOG_DIR/${service}.pid"
        if (Test-Path $pidFile) {
            $pid = Get-Content $pidFile
            Write-Host "Stopping $service with PID $pid..."
            try {
                Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
                Write-Host "$service stopped"
            } catch {
                Write-Host "$service already stopped"
            }
            Remove-Item $pidFile -ErrorAction SilentlyContinue
        } else {
            Write-Host "$service is not running"
        }
    }
    
    Write-Host "All microservices stopped!"
}

# 查看所有微服务状态
function Get-AllServiceStatus {
    Write-Host "Checking status of all microservices..."
    
    foreach ($service in $SERVICES) {
        $pidFile = "$LOG_DIR/${service}.pid"
        if (Test-Path $pidFile) {
            $pid = Get-Content $pidFile
            $process = Get-Process -Id $pid -ErrorAction SilentlyContinue
            if ($process) {
                Write-Host "$service`: RUNNING (PID: $pid)"
            } else {
                Write-Host "$service`: STOPPED (stale PID: $pid)"
                Remove-Item $pidFile -ErrorAction SilentlyContinue
            }
        } else {
            Write-Host "$service`: STOPPED"
        }
    }
}

# 查看微服务日志
function Get-ServiceLogs {
    param([string]$service)
    
    if ([string]::IsNullOrEmpty($service)) {
        Write-Host "Usage: Get-ServiceLogs -service <service-name>"
        return
    }
    
    $logFile = "$LOG_DIR/${service}.log"
    if (Test-Path $logFile) {
        Get-Content $logFile -Tail 50 -Wait
    } else {
        Write-Host "Log file not found for $service"
    }
}

# 重启微服务
function Restart-Service {
    param([string]$service)
    
    if ([string]::IsNullOrEmpty($service)) {
        Write-Host "Usage: Restart-Service -service <service-name>"
        return
    }
    
    # 查找服务索引
    $index = $SERVICES.IndexOf($service)
    if ($index -eq -1) {
        Write-Host "Service $service not found"
        return
    }
    
    # 停止服务
    $pidFile = "$LOG_DIR/${service}.pid"
    if (Test-Path $pidFile) {
        $pid = Get-Content $pidFile
        Write-Host "Stopping $service with PID $pid..."
        try {
            Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
            Write-Host "$service stopped"
        } catch {
            Write-Host "$service already stopped"
        }
        Remove-Item $pidFile -ErrorAction SilentlyContinue
    }
    
    # 启动服务
    $port = $PORTS[$index]
    Write-Host "Starting $service on port $port..."
    
    # 设置环境变量
    $env:ARCHITECTURE_MODE = "micro_ddd"
    $env:SERVICE_NAME = $service
    $env:PORT = $port
    $env:CONFIG_FILE = "./config/micro_ddd.yaml"
    
    # 启动服务
    $releasePath = "./target/release/carptms_server.exe"
    $logFile = "$LOG_DIR/${service}.log"
    $newPidFile = "$LOG_DIR/${service}.pid"
    
    $newProcess = Start-Process -FilePath $releasePath -RedirectStandardOutput $logFile -RedirectStandardError $logFile -PassThru -WindowStyle Hidden
    
    # 保存进程ID
    $newProcess.Id | Out-File -FilePath $newPidFile
    
    Write-Host "$service started with PID $($newProcess.Id)"
}

# 显示帮助信息
function Show-Help {
    Write-Host @"
Microservices Management Script

Usage: ./start_microservices.ps1 <action> [service]

Actions:
  build    - Build release version
  start    - Start all microservices
  stop     - Stop all microservices
  status   - Check status of all microservices
  logs     - View logs for a specific service
  restart  - Restart a specific service
  help     - Show this help message

Services:
  vehicle-service
  cargo-service
  trip-service
  billing-service
  user-service
  device-service
  finance-service
  location-service
  weighing-service
  alerts-service

Examples:
  ./start_microservices.ps1 build
  ./start_microservices.ps1 start
  ./start_microservices.ps1 status
  ./start_microservices.ps1 logs vehicle-service
  ./start_microservices.ps1 restart user-service
  ./start_microservices.ps1 stop
"@
}

# 主菜单
param(
    [string]$Action = "help",
    [string]$Service = ""
)

switch ($Action.ToLower()) {
    "build" { Build-Release }
    "start" { Start-AllServices }
    "stop" { Stop-AllServices }
    "status" { Get-AllServiceStatus }
    "logs" { Get-ServiceLogs -service $Service }
    "restart" { Restart-Service -service $Service }
    "help" { Show-Help }
    default {
        Write-Host "Unknown action: $Action"
        Show-Help
    }
}
