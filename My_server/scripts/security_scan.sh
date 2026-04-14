#!/bin/bash

# 安全扫描脚本

set -e

echo "Starting security scan..."

# 1. 检查系统更新
echo "\n1. Checking for system updates..."
if [ -f "/etc/alpine-release" ]; then
    apk update && apk list --upgradable
elif [ -f "/etc/debian_version" ]; then
    apt update && apt list --upgradable
elif [ -f "/etc/redhat-release" ]; then
    yum check-update
fi

# 2. 检查开放端口
echo "\n2. Checking open ports..."
netstat -tulpn || ss -tulpn

# 3. 检查系统负载
echo "\n3. Checking system load..."
uname -a
uptime

# 4. 检查Docker容器状态
echo "\n4. Checking Docker container status..."
if command -v docker &> /dev/null; then
    docker ps -a
fi

# 5. 检查日志中的错误
echo "\n5. Checking for errors in logs..."
if [ -d "/var/log" ]; then
    find /var/log -name "*.log" -exec grep -l "ERROR" {} \; 2>/dev/null | head -10
fi

# 6. 检查敏感文件权限
echo "\n6. Checking permissions on sensitive files..."
sensitive_files=("/etc/passwd" "/etc/shadow" "/etc/ssh/sshd_config" "/etc/nginx/nginx.conf" "/etc/nginx/ssl/key.pem")
for file in "${sensitive_files[@]}"; do
    if [ -f "$file" ]; then
        echo "$file: $(ls -la $file)"
    fi
done

# 7. 检查环境变量中的敏感信息
echo "\n7. Checking environment variables for sensitive information..."
env | grep -E "(PASSWORD|SECRET|KEY|TOKEN)" | head -20

# 8. 检查网络连接
echo "\n8. Checking network connections..."
netstat -an | grep ESTABLISHED || ss -an | grep ESTABLISHED

# 9. 检查磁盘使用情况
echo "\n9. Checking disk usage..."
df -h

# 10. 检查内存使用情况
echo "\n10. Checking memory usage..."
free -m

echo "\nSecurity scan completed!"


