-- CarpTMS 数据库索引优化脚本
-- 执行方式: psql -U postgres -d CarpTMS -f create_indexes.sql

\echo '=========================================='
\echo 'CarpTMS 数据库索引创建'
\echo '=========================================='

-- ==========================================
-- GPS轨迹表索引
-- ==========================================
\echo ''
\echo '创建GPS轨迹表索引...'

-- 车辆ID和时间复合索引（最常用）
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_gps_track_vehicle_time
ON gps_track_data(vehicle_id, gps_time DESC);

-- 时间索引（用于时间范围查询）
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_gps_track_time
ON gps_track_data(gps_time DESC);

-- 空间索引（用于地理位置查询，需要PostGIS）
-- 如果启用了PostGIS，取消下面的注释
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_gps_track_spatial
-- ON gps_track_data USING GIST(location);

\echo '✓ GPS轨迹表索引创建完成'

-- ==========================================
-- 传感器数据表索引
-- ==========================================
\echo ''
\echo '创建传感器数据表索引...'

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sensor_vehicle_time
ON sensor_data(vehicle_id, collect_time DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sensor_time
ON sensor_data(collect_time DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sensor_type
ON sensor_data(sensor_type);

\echo '✓ 传感器数据表索引创建完成'

-- ==========================================
-- 报警记录表索引
-- ==========================================
\echo ''
\echo '创建报警记录表索引...'

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_alarm_vehicle_time
ON alarm_records(vehicle_id, alarm_time DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_alarm_time
ON alarm_records(alarm_time DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_alarm_level
ON alarm_records(alarm_type);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_alarm_handled
ON alarm_records(handle_status);

\echo '✓ 报警记录表索引创建完成'

-- ==========================================
-- 称重数据表索引
-- ==========================================
\echo ''
\echo '创建称重数据表索引...'

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_weighing_vehicle_time
ON weighing_records(vehicle_id, weigh_time DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_weighing_time
ON weighing_records(weigh_time DESC);

\echo '✓ 称重数据表索引创建完成'

-- ==========================================
-- 车辆表索引
-- ==========================================
\echo ''
\echo '创建车辆表索引...'

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_vehicle_status
ON vehicles(status);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_vehicle_group
ON vehicles(group_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_vehicle_plate
ON vehicles(plate_no);

\echo '✓ 车辆表索引创建完成'

-- ==========================================
-- 用户表索引
-- ==========================================
\echo ''
\echo '创建用户表索引...'

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_status
ON users(status);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_mobile
ON users(mobile);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_type
ON users(user_type);

\echo '✓ 用户表索引创建完成'

-- ==========================================
-- 设备表索引
-- ==========================================
\echo ''
\echo '创建设备表索引...'

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_device_status
ON devices(status);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_device_vehicle
ON devices(vehicle_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_device_type
ON devices(device_type);

\echo '✓ 设备表索引创建完成'

-- ==========================================
-- 分析表统计信息
-- ==========================================
\echo ''
\echo '分析表统计信息...'

ANALYZE vehicles;
ANALYZE gps_track_data;
ANALYZE sensor_data;
ANALYZE alarm_records;
ANALYZE weighing_records;
ANALYZE users;
ANALYZE devices;

\echo '✓ 表统计信息分析完成'

-- ==========================================
-- 显示索引信息
-- ==========================================
\echo ''
\echo '=========================================='
\echo '已创建的索引列表'
\echo '=========================================='
\echo ''

SELECT
    schemaname,
    tablename,
    indexname,
    indexdef
FROM pg_indexes
WHERE tablename IN (
    'vehicles', 'gps_track_data', 'sensor_data',
    'alarm_records', 'weighing_records', 'users', 'devices'
)
ORDER BY tablename, indexname;

\echo ''
\echo '=========================================='
\echo '索引创建完成！'
\echo '=========================================='


