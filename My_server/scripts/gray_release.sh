#!/bin/bash

# 灰度发布脚本
# 用于在蓝绿部署环境之间平滑切换流量

# 配置参数
NGINX_CONTAINER="tms_nginx"
BLUE_SERVICE="tms_web_blue"
GREEN_SERVICE="tms_web_green"
NGINX_CONF="nginx.conf"
LOG_FILE="gray_release.log"

# 日志函数
log() {
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo "[$timestamp] $1" | tee -a $LOG_FILE
}

# 检查服务健康状态
check_service_health() {
    local service_name=$1
    local port=$2
    log "检查服务 $service_name 的健康状态..."
    
    local max_attempts=3
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        local health_status=$(docker exec $NGINX_CONTAINER curl -s -o /dev/null -w "%{http_code}" http://$service_name:$port/health)
        
        if [ "$health_status" == "200" ]; then
            log "服务 $service_name 健康状态正常"
            return 0
        else
            log "服务 $service_name 健康检查失败，状态码: $health_status (尝试 $attempt/$max_attempts)"
            attempt=$((attempt + 1))
            sleep 5
        fi
    done
    
    log "服务 $service_name 健康检查失败，达到最大尝试次数"
    return 1
}

# 切换到蓝色环境
switch_to_blue() {
    log "开始切换到蓝色环境..."
    
    # 检查蓝色环境健康状态
    if ! check_service_health $BLUE_SERVICE 8080; then
        log "蓝色环境健康检查失败，切换中止"
        return 1
    fi
    
    # 更新Nginx配置
    log "更新Nginx配置，将流量切换到蓝色环境..."
    cat > $NGINX_CONF << EOF
# 负载均衡器配置
upstream tms_backend {
    # 蓝色环境 - 活跃
    server $BLUE_SERVICE:8080 max_fails=3 fail_timeout=30s;
    # 绿色环境 - 权重为0，不接收流量
    server $GREEN_SERVICE:8080 max_fails=3 fail_timeout=30s weight=0;
}

server {
    listen 80;
    server_name localhost;

    # 健康检查端点
    location /health {
        proxy_pass http://tms_backend/health;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # API请求
    location /api {
        proxy_pass http://tms_backend/api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 60s;
    }

    # 其他请求
    location / {
        proxy_pass http://tms_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # 静态文件缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        proxy_pass http://tms_backend;
        proxy_cache_valid 200 30d;
        add_header Cache-Control "public, max-age=2592000";
    }

    # 错误页面
    error_page 404 /404.html;
    error_page 500 502 503 504 /50x.html;
    location = /50x.html {
        root /usr/share/nginx/html;
    }

    # 蓝绿部署切换端点
    location /switch {
        allow 127.0.0.1;
        deny all;
        return 200 'Blue-Green deployment switch endpoint';
    }
}
EOF
    
    # 重新加载Nginx配置
    log "重新加载Nginx配置..."
    docker cp $NGINX_CONF $NGINX_CONTAINER:/etc/nginx/conf.d/default.conf
    docker exec $NGINX_CONTAINER nginx -s reload
    
    if [ $? -eq 0 ]; then
        log "Nginx配置重新加载成功"
    else
        log "Nginx配置重新加载失败"
        return 1
    fi
    
    # 验证切换结果
    log "验证切换结果..."
    sleep 5
    local health_status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health)
    
    if [ "$health_status" == "200" ]; then
        log "切换到蓝色环境成功，服务健康状态正常"
        return 0
    else
        log "切换到蓝色环境失败，健康检查状态码: $health_status"
        return 1
    fi
}

# 切换到绿色环境
switch_to_green() {
    log "开始切换到绿色环境..."
    
    # 检查绿色环境健康状态
    if ! check_service_health $GREEN_SERVICE 8080; then
        log "绿色环境健康检查失败，切换中止"
        return 1
    fi
    
    # 更新Nginx配置
    log "更新Nginx配置，将流量切换到绿色环境..."
    cat > $NGINX_CONF << EOF
# 负载均衡器配置
upstream tms_backend {
    # 蓝色环境 - 权重为0，不接收流量
    server $BLUE_SERVICE:8080 max_fails=3 fail_timeout=30s weight=0;
    # 绿色环境 - 活跃
    server $GREEN_SERVICE:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name localhost;

    # 健康检查端点
    location /health {
        proxy_pass http://tms_backend/health;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # API请求
    location /api {
        proxy_pass http://tms_backend/api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 60s;
    }

    # 其他请求
    location / {
        proxy_pass http://tms_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # 静态文件缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        proxy_pass http://tms_backend;
        proxy_cache_valid 200 30d;
        add_header Cache-Control "public, max-age=2592000";
    }

    # 错误页面
    error_page 404 /404.html;
    error_page 500 502 503 504 /50x.html;
    location = /50x.html {
        root /usr/share/nginx/html;
    }

    # 蓝绿部署切换端点
    location /switch {
        allow 127.0.0.1;
        deny all;
        return 200 'Blue-Green deployment switch endpoint';
    }
}
EOF
    
    # 重新加载Nginx配置
    log "重新加载Nginx配置..."
    docker cp $NGINX_CONF $NGINX_CONTAINER:/etc/nginx/conf.d/default.conf
    docker exec $NGINX_CONTAINER nginx -s reload
    
    if [ $? -eq 0 ]; then
        log "Nginx配置重新加载成功"
    else
        log "Nginx配置重新加载失败"
        return 1
    fi
    
    # 验证切换结果
    log "验证切换结果..."
    sleep 5
    local health_status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health)
    
    if [ "$health_status" == "200" ]; then
        log "切换到绿色环境成功，服务健康状态正常"
        return 0
    else
        log "切换到绿色环境失败，健康检查状态码: $health_status"
        return 1
    fi
}

# 灰度发布 - 逐步切换流量
gray_release() {
    local target_env=$1
    local steps=5
    local step_weight=20
    
    log "开始灰度发布到 $target_env 环境..."
    
    # 检查目标环境健康状态
    if [ "$target_env" == "blue" ]; then
        if ! check_service_health $BLUE_SERVICE 8080; then
            log "蓝色环境健康检查失败，灰度发布中止"
            return 1
        fi
    else
        if ! check_service_health $GREEN_SERVICE 8080; then
            log "绿色环境健康检查失败，灰度发布中止"
            return 1
        fi
    fi
    
    # 逐步增加目标环境的权重
    for ((i=1; i<=$steps; i++)); do
        local target_weight=$((i * step_weight))
        local source_weight=$((100 - target_weight))
        
        log "灰度发布步骤 $i/$steps: 目标环境权重 $target_weight%, 源环境权重 $source_weight%"
        
        # 更新Nginx配置
        if [ "$target_env" == "blue" ]; then
            cat > $NGINX_CONF << EOF
# 负载均衡器配置
upstream tms_backend {
    # 蓝色环境 - 目标环境
    server $BLUE_SERVICE:8080 max_fails=3 fail_timeout=30s weight=$target_weight;
    # 绿色环境 - 源环境
    server $GREEN_SERVICE:8080 max_fails=3 fail_timeout=30s weight=$source_weight;
}

server {
    listen 80;
    server_name localhost;

    # 健康检查端点
    location /health {
        proxy_pass http://tms_backend/health;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # API请求
    location /api {
        proxy_pass http://tms_backend/api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 60s;
    }

    # 其他请求
    location / {
        proxy_pass http://tms_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # 静态文件缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        proxy_pass http://tms_backend;
        proxy_cache_valid 200 30d;
        add_header Cache-Control "public, max-age=2592000";
    }

    # 错误页面
    error_page 404 /404.html;
    error_page 500 502 503 504 /50x.html;
    location = /50x.html {
        root /usr/share/nginx/html;
    }

    # 蓝绿部署切换端点
    location /switch {
        allow 127.0.0.1;
        deny all;
        return 200 'Blue-Green deployment switch endpoint';
    }
}
EOF
        else
            cat > $NGINX_CONF << EOF
# 负载均衡器配置
upstream tms_backend {
    # 蓝色环境 - 源环境
    server $BLUE_SERVICE:8080 max_fails=3 fail_timeout=30s weight=$source_weight;
    # 绿色环境 - 目标环境
    server $GREEN_SERVICE:8080 max_fails=3 fail_timeout=30s weight=$target_weight;
}

server {
    listen 80;
    server_name localhost;

    # 健康检查端点
    location /health {
        proxy_pass http://tms_backend/health;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # API请求
    location /api {
        proxy_pass http://tms_backend/api;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 60s;
    }

    # 其他请求
    location / {
        proxy_pass http://tms_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # 静态文件缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        proxy_pass http://tms_backend;
        proxy_cache_valid 200 30d;
        add_header Cache-Control "public, max-age=2592000";
    }

    # 错误页面
    error_page 404 /404.html;
    error_page 500 502 503 504 /50x.html;
    location = /50x.html {
        root /usr/share/nginx/html;
    }

    # 蓝绿部署切换端点
    location /switch {
        allow 127.0.0.1;
        deny all;
        return 200 'Blue-Green deployment switch endpoint';
    }
}
EOF
        fi
        
        # 重新加载Nginx配置
        docker cp $NGINX_CONF $NGINX_CONTAINER:/etc/nginx/conf.d/default.conf
        docker exec $NGINX_CONTAINER nginx -s reload
        
        if [ $? -eq 0 ]; then
            log "Nginx配置重新加载成功"
        else
            log "Nginx配置重新加载失败"
            return 1
        fi
        
        # 验证服务健康状态
        sleep 10
        local health_status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health)
        
        if [ "$health_status" != "200" ]; then
            log "服务健康检查失败，状态码: $health_status，灰度发布中止"
            # 回滚到源环境
            if [ "$target_env" == "blue" ]; then
                switch_to_green
            else
                switch_to_blue
            fi
            return 1
        else
            log "服务健康状态正常，继续灰度发布..."
        fi
    done
    
    log "灰度发布到 $target_env 环境成功完成"
    return 0
}

# 显示帮助信息
show_help() {
    echo "灰度发布脚本使用说明:"
    echo ""
    echo "用法: $0 [命令]"
    echo ""
    echo "命令:"
    echo "  switch-blue     - 切换到蓝色环境"
    echo "  switch-green    - 切换到绿色环境"
    echo "  gray-blue       - 灰度发布到蓝色环境"
    echo "  gray-green      - 灰度发布到绿色环境"
    echo "  status          - 查看当前部署状态"
    echo "  help            - 显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 switch-blue    # 直接切换到蓝色环境"
    echo "  $0 gray-green     # 灰度发布到绿色环境"
}

# 查看当前部署状态
check_status() {
    log "查看当前部署状态..."
    
    # 检查Nginx配置
    local nginx_config=$(docker exec $NGINX_CONTAINER cat /etc/nginx/conf.d/default.conf 2>/dev/null)
    
    if [ -z "$nginx_config" ]; then
        log "无法获取Nginx配置"
        return 1
    fi
    
    # 检查服务状态
    local blue_status=$(docker inspect --format='{{.State.Status}}' $BLUE_SERVICE 2>/dev/null || echo "unknown")
    local green_status=$(docker inspect --format='{{.State.Status}}' $GREEN_SERVICE 2>/dev/null || echo "unknown")
    local nginx_status=$(docker inspect --format='{{.State.Status}}' $NGINX_CONTAINER 2>/dev/null || echo "unknown")
    
    log "服务状态:"
    log "  蓝色环境 ($BLUE_SERVICE): $blue_status"
    log "  绿色环境 ($GREEN_SERVICE): $green_status"
    log "  Nginx负载均衡器 ($NGINX_CONTAINER): $nginx_status"
    
    # 检查流量分配
    if echo "$nginx_config" | grep -q "weight=0.*$BLUE_SERVICE"; then
        log "当前流量分配: 绿色环境活跃，蓝色环境备用"
    elif echo "$nginx_config" | grep -q "weight=0.*$GREEN_SERVICE"; then
        log "当前流量分配: 蓝色环境活跃，绿色环境备用"
    else
        log "当前流量分配: 灰度发布中，流量在两个环境之间分配"
    fi
    
    # 检查服务健康状态
    log "服务健康状态:"
    local blue_health=$(docker exec $NGINX_CONTAINER curl -s -o /dev/null -w "%{http_code}" http://$BLUE_SERVICE:8080/health 2>/dev/null || echo "unknown")
    local green_health=$(docker exec $NGINX_CONTAINER curl -s -o /dev/null -w "%{http_code}" http://$GREEN_SERVICE:8080/health 2>/dev/null || echo "unknown")
    local public_health=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health 2>/dev/null || echo "unknown")
    
    log "  蓝色环境健康状态: $blue_health"
    log "  绿色环境健康状态: $green_health"
    log "  公共端点健康状态: $public_health"
    
    return 0
}

# 主函数
main() {
    case "$1" in
        switch-blue)
            switch_to_blue
            ;;
        switch-green)
            switch_to_green
            ;;
        gray-blue)
            gray_release "blue"
            ;;
        gray-green)
            gray_release "green"
            ;;
        status)
            check_status
            ;;
        help|
        *)
            show_help
            ;;
    esac
}

# 执行主函数
main "$1"

