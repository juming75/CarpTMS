#!/bin/bash

# 检查是否存在证书文件
if [ ! -f "/etc/nginx/ssl/cert.pem" ] || [ ! -f "/etc/nginx/ssl/key.pem" ]; then
    echo "Generating self-signed SSL certificates..."
    
    # 生成自签名证书
    openssl req -x509 -newkey rsa:4096 \
        -keyout /etc/nginx/ssl/key.pem \
        -out /etc/nginx/ssl/cert.pem \
        -days 365 \
        -nodes \
        -subj "/CN=localhost"
    
    echo "SSL certificates generated successfully!"
else
    echo "SSL certificates already exist, skipping generation."
fi

# 设置证书权限
chmod 600 /etc/nginx/ssl/key.pem
chmod 644 /etc/nginx/ssl/cert.pem

echo "SSL certificate setup completed."


