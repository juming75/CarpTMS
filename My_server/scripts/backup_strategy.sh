#!/bin/bash

# 数据备份策略脚本
# 支持全量备份和增量备份结合

# 配置参数
BACKUP_DIR="/var/backups/tms"
DB_HOST="localhost"
DB_PORT="5432"
DB_USER="postgres"
DB_PASSWORD="postgres"
DB_NAME="tms"

# 备份保留天数
FULL_BACKUP_RETENTION_DAYS=7
INCREMENTAL_BACKUP_RETENTION_DAYS=3

# 全量备份频率（天）
FULL_BACKUP_INTERVAL=1

# 创建备份目录
mkdir -p $BACKUP_DIR/full
mkdir -p $BACKUP_DIR/incremental

# 设置PGPASSWORD环境变量
export PGPASSWORD=$DB_PASSWORD

# 获取当前日期时间
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# 检查是否需要执行全量备份
LAST_FULL_BACKUP=$(find $BACKUP_DIR/full -name "*_full.sql.gz" -type f | sort -r | head -1)

if [ -z "$LAST_FULL_BACKUP" ] || [ $(($(date +%s) - $(stat -c %Y "$LAST_FULL_BACKUP" 2>/dev/null || echo 0))) -gt $((FULL_BACKUP_INTERVAL * 24 * 3600)) ]; then
    echo "执行全量备份..."
    FULL_BACKUP_FILE="$BACKUP_DIR/full/${DB_NAME}_full_${TIMESTAMP}.sql.gz"
    
    # 执行全量备份
    pg_dump -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME | gzip > $FULL_BACKUP_FILE
    
    if [ $? -eq 0 ]; then
        echo "全量备份成功：$FULL_BACKUP_FILE"
        
        # 更新全量备份时间戳
        echo $TIMESTAMP > $BACKUP_DIR/last_full_backup.txt
        
        # 清理旧的全量备份
        find $BACKUP_DIR/full -name "*_full.sql.gz" -type f -mtime +$FULL_BACKUP_RETENTION_DAYS -delete
    else
        echo "全量备份失败！"
        exit 1
    fi
else
    echo "执行增量备份..."
    INCREMENTAL_BACKUP_FILE="$BACKUP_DIR/incremental/${DB_NAME}_incremental_${TIMESTAMP}.sql.gz"
    
    # 获取上次全量备份的WAL位置
    if [ -f $BACKUP_DIR/last_wal_position.txt ]; then
        LAST_WAL_POS=$(cat $BACKUP_DIR/last_wal_position.txt)
    else
        LAST_WAL_POS="0/00000000"
    fi
    
    # 执行增量备份（基于WAL日志）
    pg_receivewal -h $DB_HOST -p $DB_PORT -U $DB_USER -D $BACKUP_DIR/incremental -v --start-lsn=$LAST_WAL_POS -f "${DB_NAME}_wal_%f_%t" --max-wal-size=1GB
    
    if [ $? -eq 0 ]; then
        echo "增量备份成功"
        
        # 更新WAL位置
        pg_controldata -D /var/lib/postgresql/data | grep "Latest checkpoint location" | awk '{print $4}' > $BACKUP_DIR/last_wal_position.txt
        
        # 清理旧的增量备份
        find $BACKUP_DIR/incremental -name "*_incremental.sql.gz" -type f -mtime +$INCREMENTAL_BACKUP_RETENTION_DAYS -delete
        find $BACKUP_DIR/incremental -name "*_wal_*" -type f -mtime +$INCREMENTAL_BACKUP_RETENTION_DAYS -delete
    else
        echo "增量备份失败！"
        exit 1
    fi
fi

# 解除PGPASSWORD环境变量
unset PGPASSWORD

echo "备份任务完成！"


