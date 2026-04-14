#!/bin/bash

# 主数据库初始化脚本
set -e

# 等待PostgreSQL启动
sleep 5

# 创建复制用户
psql -U postgres -d tms -c "CREATE USER $POSTGRES_REPLICATION_USER REPLICATION LOGIN ENCRYPTED PASSWORD '$POSTGRES_REPLICATION_PASSWORD';"

# 配置pg_hba.conf允许复制连接
echo "host replication $POSTGRES_REPLICATION_USER 0.0.0.0/0 md5" >> /var/lib/postgresql/data/pg_hba.conf

# 重新加载配置
pg_ctl reload -D /var/lib/postgresql/data

echo "主数据库复制配置完成！"


