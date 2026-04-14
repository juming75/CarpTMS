@echo off
chcp 65001 >nul
title CarpTMS 停止服务脚本
setlocal EnableDelayedExpansion

echo ============================================
echo    CarpTMS 停止服务脚本
echo ============================================
echo.

set "PROJECT_DIR=D:\studying\Codecargo\CarpTMS"
set "PID_DIR=%PROJECT_DIR%\pids"

echo [1/3] 停止后端服务...
if exist "%PID_DIR%\backend.pid" (
    set /p BACKEND_PID=<"%PID_DIR%\backend.pid"
    echo   - 找到后端服务 (PID: !BACKEND_PID!)
    taskkill /PID !BACKEND_PID! /F >nul 2>&1
    if !errorlevel! equ 0 (
        echo     已停止
    ) else (
        echo     进程不存在或已停止
    )
    del "%PID_DIR%\backend.pid" >nul 2>&1
) else (
    echo   - 未找到 PID 文件，尝试查找进程...
    taskkill /IM carptms_server.exe /F >nul 2>&1
    if !errorlevel! equ 0 (
        echo     已停止
    ) else (
        echo     未找到运行中的后端服务
    )
)

echo.
echo [2/3] 停止前端服务...
if exist "%PID_DIR%\frontend.pid" (
    set /p FRONTEND_PID=<"%PID_DIR%\frontend.pid"
    echo   - 找到前端服务 (PID: !FRONTEND_PID!)
    taskkill /PID !FRONTEND_PID! /F >nul 2>&1
    if !errorlevel! equ 0 (
        echo     已停止
    ) else (
        echo     进程不存在或已停止
    )
    del "%PID_DIR%\frontend.pid" >nul 2>&1
) else (
    echo   - 未找到 PID 文件，尝试查找进程...
    taskkill /IM node.exe /FI "WINDOWTITLE eq CarpTMS-Frontend*" /F >nul 2>&1
    if !errorlevel! equ 0 (
        echo     已停止
    ) else (
        echo     未找到运行中的前端服务
    )
)

echo.
echo [3/3] 清理...
taskkill /IM serve.exe /F >nul 2>&1

echo.
echo ============================================
echo    所有服务已停止
echo ============================================
echo.
pause


