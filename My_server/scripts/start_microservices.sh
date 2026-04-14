#!/bin/bash

# 微服务启动脚本
# 用于启动和管理各个微服务

# 服务名称列表
SERVICES=("vehicle-service" "cargo-service" "trip-service" "billing-service" "user-service" "device-service" "finance-service" "location-service" "weighing-service" "alerts-service")

# 服务端口映射
PORTS=(8083 8084 8085 8086 8087 8088 8089 8090 8091 8092)

# 日志目录
LOG_DIR="./logs"
mkdir -p $LOG_DIR

# 启动所有微服务
start_all() {
    echo "Starting all microservices..."
    
    for i in "${!SERVICES[@]}"; do
        service=${SERVICES[$i]}
        port=${PORTS[$i]}
        
        echo "Starting $service on port $port..."
        
        # 设置环境变量
        export ARCHITECTURE_MODE="micro_ddd"
        export SERVICE_NAME="$service"
        export PORT="$port"
        export CONFIG_FILE="./config/micro_ddd.yaml"
        
        # 启动服务
        nohup cargo run --bin carptms_server > "$LOG_DIR/${service}.log" 2>&1 &
        
        # 保存进程ID
        echo $! > "$LOG_DIR/${service}.pid"
        
        echo "$service started with PID $(cat "$LOG_DIR/${service}.pid")"
        sleep 2
    done
    
    echo "All microservices started successfully!"
}

# 停止所有微服务
stop_all() {
    echo "Stopping all microservices..."
    
    for service in "${SERVICES[@]}"; do
        if [ -f "$LOG_DIR/${service}.pid" ]; then
            pid=$(cat "$LOG_DIR/${service}.pid")
            echo "Stopping $service with PID $pid..."
            kill $pid 2>/dev/null
            rm "$LOG_DIR/${service}.pid"
            echo "$service stopped"
        else
            echo "$service is not running"
        fi
    done
    
    echo "All microservices stopped!"
}

# 查看所有微服务状态
status_all() {
    echo "Checking status of all microservices..."
    
    for service in "${SERVICES[@]}"; do
        if [ -f "$LOG_DIR/${service}.pid" ]; then
            pid=$(cat "$LOG_DIR/${service}.pid")
            if ps -p $pid > /dev/null; then
                echo "$service: RUNNING (PID: $pid)"
            else
                echo "$service: STOPPED (stale PID: $pid)"
                rm "$LOG_DIR/${service}.pid"
            fi
        else
            echo "$service: STOPPED"
        fi
    done
}

# 查看微服务日志
logs() {
    service=$1
    if [ -z "$service" ]; then
        echo "Usage: $0 logs <service-name>"
        exit 1
    fi
    
    if [ -f "$LOG_DIR/${service}.log" ]; then
        tail -f "$LOG_DIR/${service}.log"
    else
        echo "Log file not found for $service"
    fi
}

# 重启微服务
restart() {
    service=$1
    if [ -z "$service" ]; then
        echo "Usage: $0 restart <service-name>"
        exit 1
    fi
    
    # 停止服务
    if [ -f "$LOG_DIR/${service}.pid" ]; then
        pid=$(cat "$LOG_DIR/${service}.pid")
        echo "Stopping $service with PID $pid..."
        kill $pid 2>/dev/null
        rm "$LOG_DIR/${service}.pid"
        echo "$service stopped"
    fi
    
    # 启动服务
    index=-1
    for i in "${!SERVICES[@]}"; do
        if [ "${SERVICES[$i]}" = "$service" ]; then
            index=$i
            break
        fi
    done
    
    if [ $index -eq -1 ]; then
        echo "Service $service not found"
        exit 1
    fi
    
    port=${PORTS[$index]}
    echo "Starting $service on port $port..."
    
    # 设置环境变量
    export ARCHITECTURE_MODE="micro_ddd"
    export SERVICE_NAME="$service"
    export PORT="$port"
    export CONFIG_FILE="./config/micro_ddd.yaml"
    
    # 启动服务
    nohup cargo run --bin carptms_server > "$LOG_DIR/${service}.log" 2>&1 &
    
    # 保存进程ID
    echo $! > "$LOG_DIR/${service}.pid"
    
    echo "$service started with PID $(cat "$LOG_DIR/${service}.pid")"
}

# 主菜单
case "$1" in
    start)  start_all ;;
    stop)   stop_all ;;
    status) status_all ;;
    logs)   logs "$2" ;;
    restart) restart "$2" ;;
    *)
        echo "Usage: $0 {start|stop|status|logs <service>|restart <service>}"
        exit 1
        ;;
esac
