-- ============================================
-- 数据库优化脚本
-- 包含：分片策略、索引优化、分区表、读写分离配置
-- ============================================

-- ============================================
-- 1. 创建分区表 - 称重数据按时间分区
-- ============================================

-- 创建称重数据分区表（如果原表存在，需要先迁移数据）
CREATE TABLE IF NOT EXISTS weighing_data_partitioned (
    id BIGSERIAL,
    vehicle_id INTEGER NOT NULL,
    device_id VARCHAR(50) NOT NULL,
    weighing_time TIMESTAMP NOT NULL,
    gross_weight DOUBLE PRECISION NOT NULL,
    tare_weight DOUBLE PRECISION,
    net_weight DOUBLE PRECISION NOT NULL,
    axle_count INTEGER,
    speed DOUBLE PRECISION,
    lane_no INTEGER,
    site_id INTEGER,
    status INTEGER NOT NULL DEFAULT 1,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP,
    PRIMARY KEY (id, weighing_time)
) PARTITION BY RANGE (weighing_time);

-- 创建分区（按月分区）
-- 当前月份分区
CREATE TABLE IF NOT EXISTS weighing_data_y2026m03 PARTITION OF weighing_data_partitioned
    FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

-- 下一个月分区
CREATE TABLE IF NOT EXISTS weighing_data_y2026m04 PARTITION OF weighing_data_partitioned
    FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');

-- 未来几个月的分区
CREATE TABLE IF NOT EXISTS weighing_data_y2026m05 PARTITION OF weighing_data_partitioned
    FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');

CREATE TABLE IF NOT EXISTS weighing_data_y2026m06 PARTITION OF weighing_data_partitioned
    FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');

-- 创建自动分区函数
CREATE OR REPLACE FUNCTION create_weighing_data_partition()
RETURNS void AS $$
DECLARE
    partition_date DATE;
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
BEGIN
    -- 创建未来3个月的分区
    FOR i IN 1..3 LOOP
        partition_date := DATE_TRUNC('month', CURRENT_DATE + (i || ' months')::INTERVAL);
        partition_name := 'weighing_data_y' || TO_CHAR(partition_date, 'YYYY') || 'm' || TO_CHAR(partition_date, 'MM');
        start_date := partition_date;
        end_date := partition_date + INTERVAL '1 month';
        
        -- 检查分区是否已存在
        IF NOT EXISTS (
            SELECT 1 FROM pg_tables 
            WHERE tablename = partition_name
        ) THEN
            EXECUTE format(
                'CREATE TABLE IF NOT EXISTS %I PARTITION OF weighing_data_partitioned FOR VALUES FROM (%L) TO (%L)',
                partition_name, start_date, end_date
            );
            
            -- 为分区创建索引
            EXECUTE format(
                'CREATE INDEX IF NOT EXISTS idx_%s_vehicle_id ON %I(vehicle_id)',
                partition_name, partition_name
            );
            EXECUTE format(
                'CREATE INDEX IF NOT EXISTS idx_%s_weighing_time ON %I(weighing_time)',
                partition_name, partition_name
            );
            EXECUTE format(
                'CREATE INDEX IF NOT EXISTS idx_%s_device_id ON %I(device_id)',
                partition_name, partition_name
            );
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- 执行分区创建
SELECT create_weighing_data_partition();

-- ============================================
-- 2. 创建审计日志分区表（按时间分区）
-- ============================================

CREATE TABLE IF NOT EXISTS audit_logs_partitioned (
    id BIGSERIAL,
    user_id INTEGER,
    username VARCHAR(50),
    action VARCHAR(100) NOT NULL,
    resource VARCHAR(100) NOT NULL,
    resource_id TEXT,
    request_data TEXT,
    ip_address VARCHAR(50),
    user_agent TEXT,
    action_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    result SMALLINT NOT NULL DEFAULT 1,
    error_message TEXT,
    PRIMARY KEY (id, action_time)
) PARTITION BY RANGE (action_time);

-- 创建审计日志分区
CREATE TABLE IF NOT EXISTS audit_logs_y2026m03 PARTITION OF audit_logs_partitioned
    FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

CREATE TABLE IF NOT EXISTS audit_logs_y2026m04 PARTITION OF audit_logs_partitioned
    FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');

-- 创建审计日志自动分区函数
CREATE OR REPLACE FUNCTION create_audit_logs_partition()
RETURNS void AS $$
DECLARE
    partition_date DATE;
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
BEGIN
    FOR i IN 1..3 LOOP
        partition_date := DATE_TRUNC('month', CURRENT_DATE + (i || ' months')::INTERVAL);
        partition_name := 'audit_logs_y' || TO_CHAR(partition_date, 'YYYY') || 'm' || TO_CHAR(partition_date, 'MM');
        start_date := partition_date;
        end_date := partition_date + INTERVAL '1 month';
        
        IF NOT EXISTS (
            SELECT 1 FROM pg_tables 
            WHERE tablename = partition_name
        ) THEN
            EXECUTE format(
                'CREATE TABLE IF NOT EXISTS %I PARTITION OF audit_logs_partitioned FOR VALUES FROM (%L) TO (%L)',
                partition_name, start_date, end_date
            );
            
            EXECUTE format(
                'CREATE INDEX IF NOT EXISTS idx_%s_user_id ON %I(user_id)',
                partition_name, partition_name
            );
            EXECUTE format(
                'CREATE INDEX IF NOT EXISTS idx_%s_action_time ON %I(action_time)',
                partition_name, partition_name
            );
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

SELECT create_audit_logs_partition();

-- ============================================
-- 3. 优化索引 - 添加复合索引和覆盖索引
-- ============================================

-- 称重数据表优化索引
-- 复合索引：车辆ID + 称重时间（最常用的查询组合）
CREATE INDEX IF NOT EXISTS idx_weighing_data_vehicle_time 
ON weighing_data(vehicle_id, weighing_time DESC);

-- 复合索引：设备ID + 称重时间
CREATE INDEX IF NOT EXISTS idx_weighing_data_device_time 
ON weighing_data(device_id, weighing_time DESC);

-- 覆盖索引：包含常用查询字段
CREATE INDEX IF NOT EXISTS idx_weighing_data_covering 
ON weighing_data(vehicle_id, weighing_time DESC) 
INCLUDE (gross_weight, tare_weight, net_weight, status);

-- 车辆表优化索引
-- 复合索引：车牌号 + 状态
CREATE INDEX IF NOT EXISTS idx_vehicles_plate_status 
ON vehicles(license_plate, status);

-- 复合索引：车组ID + 状态
CREATE INDEX IF NOT EXISTS idx_vehicles_group_status 
ON vehicles(group_id, status);

-- 覆盖索引：车辆基本信息查询
CREATE INDEX IF NOT EXISTS idx_vehicles_basic_info 
ON vehicles(vehicle_id) 
INCLUDE (vehicle_name, license_plate, vehicle_type, status);

-- 用户表优化索引
-- 复合索引：用户名 + 用户组ID
CREATE INDEX IF NOT EXISTS idx_users_name_group 
ON users(user_name, user_group_id);

-- 审计日志表优化索引
-- 复合索引：用户ID + 操作时间
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_time 
ON audit_logs(user_id, action_time DESC);

-- 复合索引：操作类型 + 时间
CREATE INDEX IF NOT EXISTS idx_audit_logs_action_time 
ON audit_logs(action, action_time DESC);

-- ============================================
-- 4. 创建读写分离视图和函数
-- ============================================

-- 创建只读视图（用于报表和统计查询）
CREATE OR REPLACE VIEW v_weighing_data_readonly AS
SELECT 
    w.*,
    v.license_plate,
    v.vehicle_name,
    v.vehicle_type
FROM weighing_data w
LEFT JOIN vehicles v ON w.vehicle_id = v.vehicle_id;

-- 创建车辆统计视图
CREATE OR REPLACE VIEW v_vehicle_statistics AS
SELECT 
    v.vehicle_id,
    v.license_plate,
    v.vehicle_name,
    COUNT(w.id) as weighing_count,
    AVG(w.net_weight) as avg_weight,
    MAX(w.weighing_time) as last_weighing_time,
    MIN(w.weighing_time) as first_weighing_time
FROM vehicles v
LEFT JOIN weighing_data w ON v.vehicle_id = w.vehicle_id
GROUP BY v.vehicle_id, v.license_plate, v.vehicle_name;

-- 创建设备统计视图
CREATE OR REPLACE VIEW v_device_statistics AS
SELECT 
    d.device_id,
    d.device_name,
    COUNT(w.id) as weighing_count,
    MAX(w.weighing_time) as last_activity
FROM devices d
LEFT JOIN weighing_data w ON d.device_id = w.device_id
GROUP BY d.device_id, d.device_name;

-- ============================================
-- 5. 创建数据库连接池监控表
-- ============================================

CREATE TABLE IF NOT EXISTS db_pool_metrics (
    id SERIAL PRIMARY KEY,
    metric_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    pool_name VARCHAR(50) NOT NULL,
    active_connections INTEGER NOT NULL DEFAULT 0,
    idle_connections INTEGER NOT NULL DEFAULT 0,
    total_connections INTEGER NOT NULL DEFAULT 0,
    waiting_requests INTEGER NOT NULL DEFAULT 0,
    avg_wait_time_ms INTEGER NOT NULL DEFAULT 0,
    max_wait_time_ms INTEGER NOT NULL DEFAULT 0,
    connection_errors INTEGER NOT NULL DEFAULT 0,
    query_count INTEGER NOT NULL DEFAULT 0,
    slow_query_count INTEGER NOT NULL DEFAULT 0
);

-- 创建连接池监控索引
CREATE INDEX IF NOT EXISTS idx_pool_metrics_time ON db_pool_metrics(metric_time DESC);
CREATE INDEX IF NOT EXISTS idx_pool_metrics_name ON db_pool_metrics(pool_name);

-- 创建连接池监控函数
CREATE OR REPLACE FUNCTION record_pool_metrics(
    p_pool_name VARCHAR(50),
    p_active INTEGER,
    p_idle INTEGER,
    p_total INTEGER,
    p_waiting INTEGER,
    p_avg_wait INTEGER,
    p_max_wait INTEGER,
    p_errors INTEGER,
    p_queries INTEGER,
    p_slow_queries INTEGER
)
RETURNS void AS $$
BEGIN
    INSERT INTO db_pool_metrics (
        pool_name, active_connections, idle_connections, total_connections,
        waiting_requests, avg_wait_time_ms, max_wait_time_ms, connection_errors,
        query_count, slow_query_count
    ) VALUES (
        p_pool_name, p_active, p_idle, p_total, p_waiting, p_avg_wait, p_max_wait,
        p_errors, p_queries, p_slow_queries
    );
    
    -- 清理30天前的旧数据
    DELETE FROM db_pool_metrics WHERE metric_time < CURRENT_TIMESTAMP - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- 6. 创建数据库备份和恢复相关函数
-- ============================================

-- 创建备份信息表
CREATE TABLE IF NOT EXISTS db_backup_info (
    id SERIAL PRIMARY KEY,
    backup_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    backup_type VARCHAR(20) NOT NULL, -- 'full', 'incremental', 'schema'
    backup_file_path TEXT NOT NULL,
    backup_size_bytes BIGINT,
    tables_backed_up TEXT[],
    checksum VARCHAR(64),
    status VARCHAR(20) NOT NULL DEFAULT 'completed', -- 'running', 'completed', 'failed'
    error_message TEXT,
    retention_days INTEGER NOT NULL DEFAULT 30
);

-- 创建备份索引
CREATE INDEX IF NOT EXISTS idx_backup_info_time ON db_backup_info(backup_time DESC);
CREATE INDEX IF NOT EXISTS idx_backup_info_type ON db_backup_info(backup_type);

-- 创建备份函数
CREATE OR REPLACE FUNCTION record_backup(
    p_backup_type VARCHAR(20),
    p_file_path TEXT,
    p_size_bytes BIGINT,
    p_tables TEXT[],
    p_checksum VARCHAR(64),
    p_retention_days INTEGER DEFAULT 30
)
RETURNS INTEGER AS $$
DECLARE
    v_backup_id INTEGER;
BEGIN
    INSERT INTO db_backup_info (
        backup_type, backup_file_path, backup_size_bytes, tables_backed_up,
        checksum, status, retention_days
    ) VALUES (
        p_backup_type, p_file_path, p_size_bytes, p_tables, p_checksum,
        'completed', p_retention_days
    ) RETURNING id INTO v_backup_id;
    
    RETURN v_backup_id;
END;
$$ LANGUAGE plpgsql;

-- 创建清理过期备份函数
CREATE OR REPLACE FUNCTION cleanup_expired_backups()
RETURNS TABLE (deleted_count INTEGER) AS $$
DECLARE
    v_count INTEGER;
BEGIN
    DELETE FROM db_backup_info 
    WHERE backup_time < CURRENT_TIMESTAMP - (retention_days || ' days')::INTERVAL
    RETURNING id INTO v_count;
    
    RETURN QUERY SELECT v_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- 7. 创建分区维护任务
-- ============================================

-- 创建分区维护日志表
CREATE TABLE IF NOT EXISTS partition_maintenance_log (
    id SERIAL PRIMARY KEY,
    maintenance_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    table_name VARCHAR(100) NOT NULL,
    operation VARCHAR(50) NOT NULL,
    partition_name VARCHAR(100),
    status VARCHAR(20) NOT NULL,
    message TEXT
);

-- 创建自动分区维护函数
CREATE OR REPLACE FUNCTION maintain_partitions()
RETURNS void AS $$
BEGIN
    -- 创建称重数据新分区
    PERFORM create_weighing_data_partition();
    
    -- 创建审计日志新分区
    PERFORM create_audit_logs_partition();
    
    -- 记录维护日志
    INSERT INTO partition_maintenance_log (table_name, operation, status, message)
    VALUES ('weighing_data_partitioned', 'auto_create', 'success', 'Auto-created new partitions');
    
    INSERT INTO partition_maintenance_log (table_name, operation, status, message)
    VALUES ('audit_logs_partitioned', 'auto_create', 'success', 'Auto-created new partitions');
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- 8. 创建性能监控视图
-- ============================================

-- 创建表大小监控视图
CREATE OR REPLACE VIEW v_table_sizes AS
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as total_size,
    pg_total_relation_size(schemaname||'.'||tablename) as total_size_bytes,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) as table_size,
    pg_size_pretty(pg_indexes_size(schemaname||'.'||tablename)) as indexes_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- 创建索引使用统计视图
CREATE OR REPLACE VIEW v_index_usage AS
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan as index_scans,
    idx_tup_read as tuples_read,
    idx_tup_fetch as tuples_fetched,
    pg_size_pretty(pg_relation_size(indexrelid)) as index_size
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan DESC;

-- ============================================
-- 9. 初始化数据迁移（如果需要）
-- ============================================

-- 迁移称重数据到分区表（可选，根据数据量决定）
-- INSERT INTO weighing_data_partitioned 
-- SELECT * FROM weighing_data 
-- WHERE weighing_time >= '2026-03-01';

-- ============================================
-- 10. 创建定时任务（需要pg_cron扩展）
-- ============================================

-- 安装pg_cron扩展（如果可用）
-- CREATE EXTENSION IF NOT EXISTS pg_cron;

-- 创建定时分区维护任务（每小时执行一次）
-- SELECT cron.schedule('partition-maintenance', '0 * * * *', 'SELECT maintain_partitions()');

-- 创建定时清理任务（每天凌晨2点执行）
-- SELECT cron.schedule('cleanup-expired-backups', '0 2 * * *', 'SELECT cleanup_expired_backups()');

-- ============================================
-- 完成提示
-- ============================================

DO $$
BEGIN
    RAISE NOTICE '数据库优化脚本执行完成！';
    RAISE NOTICE '已创建的分区表：';
    RAISE NOTICE '  - weighing_data_partitioned（按月分区）';
    RAISE NOTICE '  - audit_logs_partitioned（按月分区）';
    RAISE NOTICE '已创建的优化索引：';
    RAISE NOTICE '  - 复合索引和覆盖索引';
    RAISE NOTICE '已创建的监控表：';
    RAISE NOTICE '  - db_pool_metrics（连接池监控）';
    RAISE NOTICE '  - db_backup_info（备份信息）';
    RAISE NOTICE '  - partition_maintenance_log（分区维护日志）';
END;
$$;


