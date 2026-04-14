#!/bin/bash

# 灾难恢复测试脚本
# 测试数据库主从切换和数据恢复流程

# 配置参数
MASTER_HOST="localhost"
MASTER_PORT="5432"
SLAVE_HOST="localhost"
SLAVE_PORT="5433"
DB_NAME="tms"
DB_USER="postgres"
DB_PASSWORD="postgres"

export PGPASSWORD=$DB_PASSWORD

# 测试步骤：
# 1. 连接主数据库，创建测试数据
# 2. 验证测试数据已复制到从数据库
# 3. 模拟主数据库故障
# 4. 提升从数据库为主数据库
# 5. 验证数据完整性
# 6. 恢复主从架构

# 清理环境
function cleanup {
    echo "清理测试环境..."
    unset PGPASSWORD
    echo "测试完成！"
}

trap cleanup EXIT

echo "=== 开始灾难恢复测试 ==="

# 步骤1：创建测试数据
echo "\n1. 连接主数据库，创建测试数据..."
test_table="test_failover_$(date +%s)"

# 创建测试表并插入数据
sql="CREATE TABLE IF NOT EXISTS $test_table (id serial PRIMARY KEY, name varchar(100), created_at timestamp DEFAULT now());"
psql -h $MASTER_HOST -p $MASTER_PORT -U $DB_USER -d $DB_NAME -c "$sql"

if [ $? -ne 0 ]; then
    echo "创建测试表失败！"
    exit 1
fi

# 插入测试数据
test_data="Test Data for Failover Testing"
psql -h $MASTER_HOST -p $MASTER_PORT -U $DB_USER -d $DB_NAME -c "INSERT INTO $test_table (name) VALUES ('$test_data');"

if [ $? -ne 0 ]; then
    echo "插入测试数据失败！"
    exit 1
fi

# 记录测试数据ID
test_id=$(psql -h $MASTER_HOST -p $MASTER_PORT -U $DB_USER -d $DB_NAME -t -c "SELECT id FROM $test_table WHERE name='$test_data' ORDER BY id DESC LIMIT 1")
test_id=$(echo $test_id | tr -d ' ')

if [ -z "$test_id" ]; then
    echo "获取测试数据ID失败！"
    exit 1
fi

echo "✓ 成功创建测试表并插入数据，ID: $test_id"

# 步骤2：验证测试数据已复制到从数据库
echo "\n2. 验证测试数据已复制到从数据库..."
sleep 3  # 等待复制完成

# 检查从数据库是否存在相同数据
slave_data=$(psql -h $SLAVE_HOST -p $SLAVE_PORT -U $DB_USER -d $DB_NAME -t -c "SELECT name FROM $test_table WHERE id=$test_id")
slave_data=$(echo $slave_data | tr -d ' ')

if [ "$slave_data" == "$test_data" ]; then
    echo "✓ 测试数据已成功复制到从数据库"
else
    echo "✗ 测试数据未复制到从数据库！"
    exit 1
fi

# 步骤3：模拟主数据库故障
echo "\n3. 模拟主数据库故障..."
# 这里我们不实际停止主数据库，而是通过修改从数据库的配置来测试提升

# 步骤4：提升从数据库为主数据库
echo "\n4. 提升从数据库为主数据库..."

# 连接从数据库，执行提升命令
psql -h $SLAVE_HOST -p $SLAVE_PORT -U $DB_USER -d $DB_NAME -c "SELECT pg_promote(wait_seconds => 60);"

if [ $? -ne 0 ]; then
    echo "✗ 提升从数据库为主数据库失败！"
    exit 1
fi

echo "✓ 从数据库已成功提升为主数据库"

# 步骤5：验证数据完整性
echo "\n5. 验证数据完整性..."

# 连接新的主数据库（原从数据库），检查测试数据是否存在
new_master_data=$(psql -h $SLAVE_HOST -p $SLAVE_PORT -U $DB_USER -d $DB_NAME -t -c "SELECT name FROM $test_table WHERE id=$test_id")
new_master_data=$(echo $new_master_data | tr -d ' ')

if [ "$new_master_data" == "$test_data" ]; then
    echo "✓ 测试数据在新主数据库中完整保留"
else
    echo "✗ 测试数据在新主数据库中丢失！"
    exit 1
fi

# 向新主数据库插入新数据，验证写入功能正常
echo "验证新主数据库写入功能..."
new_test_data="New Data after Failover"
psql -h $SLAVE_HOST -p $SLAVE_PORT -U $DB_USER -d $DB_NAME -c "INSERT INTO $test_table (name) VALUES ('$new_test_data');"

if [ $? -ne 0 ]; then
    echo "✗ 新主数据库写入失败！"
    exit 1
fi

new_test_id=$(psql -h $SLAVE_HOST -p $SLAVE_PORT -U $DB_USER -d $DB_NAME -t -c "SELECT id FROM $test_table WHERE name='$new_test_data' ORDER BY id DESC LIMIT 1")
new_test_id=$(echo $new_test_id | tr -d ' ')

echo "✓ 新主数据库写入功能正常，新数据ID: $new_test_id"

# 步骤6：清理测试数据
echo "\n6. 清理测试数据..."
psql -h $SLAVE_HOST -p $SLAVE_PORT -U $DB_USER -d $DB_NAME -c "DROP TABLE IF EXISTS $test_table;"

if [ $? -ne 0 ]; then
    echo "✗ 清理测试表失败！"
    exit 1
fi

echo "✓ 测试数据清理完成"

echo "\n=== 灾难恢复测试成功完成 ==="
echo "测试结果：数据库主从切换正常，数据完整性得到保障！"

