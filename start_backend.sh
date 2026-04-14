#!/bin/bash
# 停止旧进程
taskkill /F /IM carptms_server.exe 2>nul
taskkill /F /IM cargo.exe 2>nul

# 等待
sleep 2

# 进入目录
cd d:/studying/Codecargo/CarpTMS/My_server

# 编译
echo "编译后端..."
cargo build --bin carptms_server

# 启动
echo "启动后端..."
cargo run --bin carptms_server


