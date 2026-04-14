#!/bin/bash

# 数据库备份脚本
# 实现完整备份与增量备份结合的策略

# 配置参数
DB_HOST="localhost"
DB_PORT="5432"
DB_NAME="tms"
DB_USER="postgres"
DB_PASSWORD="postgres"
BACKUP_DIR="/var/backups/tms_db"
FULL_BACKUP_DIR="$BACKUP_DIR/full"
INCREMENTAL_BACKUP_DIR="$BACKUP_DIR/incremental"
ARCHIVE_DIR="$BACKUP_DIR/archives"
RETENTION_DAYS=7
FULL_BACKUP_INTERVAL=1 # 每天一次完整备份

# 创建备份目录
mkdir -p $FULL_BACKUP_DIR $INCREMENTAL_BACKUP_DIR $ARCHIVE_DIR

# 设置PGPASSWORD环境变量，避免密码提示
export PGPASSWORD=$DB_PASSWORD

# 生成带时间戳的备份文件名
timestamp=$(date +"%Y%m%d_%H%M%S")
full_backup_file="$FULL_BACKUP_DIR/${DB_NAME}_full_${timestamp}.sql.gz"

# 检查是否需要执行完整备份
do_full_backup=false

# 如果当天没有完整备份，执行完整备份
if [ $(find $FULL_BACKUP_DIR -name "${DB_NAME}_full_$(date +"%Y%m%d")*.sql.gz" | wc -l) -eq 0 ]; then
    do_full_backup=true
fi

if $do_full_backup; then
    echo "执行完整备份..."
    # 执行完整备份
    pg_dump -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -Fp | gzip > $full_backup_file
    
    # 检查备份是否成功
    if [ $? -eq 0 ]; then
        echo "完整备份成功：$full_backup_file"
        
        # 创建一个标记文件，用于增量备份的起点
        ln -sf $full_backup_file $BACKUP_DIR/latest_full_backup
        
        # 清理旧的完整备份
        echo "清理 $RETENTION_DAYS 天前的完整备份..."
        find $FULL_BACKUP_DIR -name "${DB_NAME}_full_*.sql.gz" -mtime +$RETENTION_DAYS -delete
    else
        echo "完整备份失败！"
        exit 1
    fi
else
    echo "今天已执行完整备份，跳过完整备份"
    
    # 执行增量备份
    echo "执行增量备份..."
    incremental_backup_file="$INCREMENTAL_BACKUP_DIR/${DB_NAME}_incremental_${timestamp}.gz"
    
    # 使用pg_receivewal获取WAL日志（增量备份）
    pg_receivewal -h $DB_HOST -p $DB_PORT -U $DB_USER -D $INCREMENTAL_BACKUP_DIR --verbose --slot=backup_slot --no-loop
    
    # 检查增量备份是否成功
    if [ $? -eq 0 ]; then
        echo "增量备份成功"
    else
        echo "增量备份失败！"
        exit 1
    fi
fi

# 清理旧的增量备份，只保留最新完整备份之后的增量备份
if [ -f $BACKUP_DIR/latest_full_backup ]; then
    latest_full_backup=$(readlink -f $BACKUP_DIR/latest_full_backup)
    latest_full_time=$(stat -c %Y $latest_full_backup)
    
    echo "清理完整备份 $latest_full_backup 之前的增量备份..."
    find $INCREMENTAL_BACKUP_DIR -type f -mtime +$RETENTION_DAYS -o -mtime -$latest_full_time | xargs -r rm -f
fi

# 清理归档日志
echo "清理 $RETENTION_DAYS 天前的归档日志..."
find $ARCHIVE_DIR -name "*.backup" -o -name "0000000*" | grep -v "archive_status" | xargs -r stat -c "%Y %n" | awk -v retention=$RETENTION_DAYS '$1 < (systime() - retention*86400) {print $2}' | xargs -r rm -f

echo "备份任务完成！"

# 解除PGPASSWORD环境变量
unset PGPASSWORD


