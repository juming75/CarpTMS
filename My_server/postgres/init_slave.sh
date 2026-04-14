#!/bin/bash

# 从数据库初始化脚本
set -e

# 等待主数据库完全启动
sleep 10

# 检查数据目录是否为空
if [ "$(ls -A /var/lib/postgresql/data)" ]; then
    echo "数据目录不为空，跳过初始化..."
    exit 0
fi

# 从主数据库获取基础备份
echo "正在从主数据库获取基础备份..."
pghba_conf_backup="/var/lib/postgresql/data/pg_hba.conf"
pg_basebackup -h $POSTGRES_MASTER_HOST -U $POSTGRES_REPLICATION_USER -D /var/lib/postgresql/data -Fp -Xs -R -P

# 配置从数据库参数
echo "配置从数据库参数..."
echo "hot_standby = on" >> /var/lib/postgresql/data/postgresql.conf
echo "primary_conninfo = 'host=$POSTGRES_MASTER_HOST port=5432 user=$POSTGRES_REPLICATION_USER password=$POSTGRES_REPLICATION_PASSWORD'" >> /var/lib/postgresql/data/postgresql.conf

# 创建standby.signal文件，标记为从库
echo "创建standby.signal文件..."
touch /var/lib/postgresql/data/standby.signal

echo "从数据库复制配置完成！"


