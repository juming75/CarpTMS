-- 冷热数据分离实现
-- 1. 为轨迹数据创建归档表
CREATE TABLE IF NOT EXISTS logistics_tracks_archive (
    LIKE logistics_tracks INCLUDING ALL,
    archive_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2. 为称重数据创建归档表
CREATE TABLE IF NOT EXISTS weighing_data_archive (
    LIKE weighing_data INCLUDING ALL,
    archive_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3. 为审计日志创建归档表
CREATE TABLE IF NOT EXISTS audit_logs_archive (
    LIKE audit_logs INCLUDING ALL,
    archive_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 4. 创建归档数据的索引
CREATE INDEX IF NOT EXISTS idx_logistics_tracks_archive_vehicle_id ON logistics_tracks_archive(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_logistics_tracks_archive_order_id ON logistics_tracks_archive(order_id);
CREATE INDEX IF NOT EXISTS idx_logistics_tracks_archive_track_time ON logistics_tracks_archive(track_time DESC);

CREATE INDEX IF NOT EXISTS idx_weighing_data_archive_vehicle_id ON weighing_data_archive(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_weighing_data_archive_device_id ON weighing_data_archive(device_id);
CREATE INDEX IF NOT EXISTS idx_weighing_data_archive_weighing_time ON weighing_data_archive(weighing_time DESC);

CREATE INDEX IF NOT EXISTS idx_audit_logs_archive_user_id ON audit_logs_archive(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_archive_action ON audit_logs_archive(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_archive_action_time ON audit_logs_archive(action_time DESC);

-- 5. 创建数据归档函数
CREATE OR REPLACE FUNCTION archive_old_partitions()
RETURNS VOID AS $$
DECLARE
    current_month TIMESTAMP;
    hot_data_months INTEGER := 3; -- 热数据保留3个月
    old_partition_start TIMESTAMP;
    old_partition TEXT;
    archive_table TEXT;
    main_table TEXT;
BEGIN
    -- 获取当前月份
    current_month := DATE_TRUNC('month', CURRENT_TIMESTAMP);
    
    -- 计算旧分区开始时间（超过3个月的数据）
    old_partition_start := current_month - INTERVAL '1 month' * hot_data_months;
    
    -- 处理logistics_tracks
    main_table := 'logistics_tracks';
    archive_table := 'logistics_tracks_archive';
    old_partition := main_table || '_' || TO_CHAR(old_partition_start, 'YYYYMM');
    
    IF EXISTS (
        SELECT 1 FROM pg_class 
        WHERE relname = old_partition AND relkind = 'r'
    ) THEN
        -- 1. 将旧分区数据插入到归档表
        EXECUTE format(
            'INSERT INTO %I SELECT *, CURRENT_TIMESTAMP FROM %I',
            archive_table, old_partition
        );
        
        -- 2. 删除旧分区
        EXECUTE format('DROP TABLE IF EXISTS %I', old_partition);
        
        RAISE NOTICE 'Archived partition % to %', old_partition, archive_table;
    END IF;
    
    -- 处理weighing_data
    main_table := 'weighing_data';
    archive_table := 'weighing_data_archive';
    old_partition := main_table || '_' || TO_CHAR(old_partition_start, 'YYYYMM');
    
    IF EXISTS (
        SELECT 1 FROM pg_class 
        WHERE relname = old_partition AND relkind = 'r'
    ) THEN
        -- 1. 将旧分区数据插入到归档表
        EXECUTE format(
            'INSERT INTO %I SELECT *, CURRENT_TIMESTAMP FROM %I',
            archive_table, old_partition
        );
        
        -- 2. 删除旧分区
        EXECUTE format('DROP TABLE IF EXISTS %I', old_partition);
        
        RAISE NOTICE 'Archived partition % to %', old_partition, archive_table;
    END IF;
    
    -- 处理audit_logs
    main_table := 'audit_logs';
    archive_table := 'audit_logs_archive';
    old_partition := main_table || '_' || TO_CHAR(old_partition_start, 'YYYYMM');
    
    IF EXISTS (
        SELECT 1 FROM pg_class 
        WHERE relname = old_partition AND relkind = 'r'
    ) THEN
        -- 1. 将旧分区数据插入到归档表
        EXECUTE format(
            'INSERT INTO %I SELECT *, CURRENT_TIMESTAMP FROM %I',
            archive_table, old_partition
        );
        
        -- 2. 删除旧分区
        EXECUTE format('DROP TABLE IF EXISTS %I', old_partition);
        
        RAISE NOTICE 'Archived partition % to %', old_partition, archive_table;
    END IF;
    
    RETURN;
END;
$$ LANGUAGE plpgsql;

-- 6. 修改分区管理函数，添加冷数据归档
CREATE OR REPLACE FUNCTION manage_partitions()
RETURNS VOID AS $$
DECLARE
    current_month TIMESTAMP;
    partition_name TEXT;
    partition_start TIMESTAMP;
    partition_end TIMESTAMP;
    old_partition TEXT;
    old_partition_start TIMESTAMP;
BEGIN
    -- 获取当前月份
    current_month := DATE_TRUNC('month', CURRENT_TIMESTAMP);
    
    -- 预创建未来3个月的分区
    FOR i IN 0..2 LOOP
        partition_start := current_month + INTERVAL '1 month' * i;
        partition_end := partition_start + INTERVAL '1 month';
        
        -- 处理logistics_tracks
        partition_name := 'logistics_tracks_' || TO_CHAR(partition_start, 'YYYYMM');
        IF NOT EXISTS (
            SELECT 1 FROM pg_class 
            WHERE relname = partition_name AND relkind = 'r'
        ) THEN
            EXECUTE format(
                'CREATE TABLE IF NOT EXISTS %I PARTITION OF logistics_tracks
                 FOR VALUES FROM (%L) TO (%L)',
                partition_name, partition_start, partition_end
            );
            -- 创建索引
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_vehicle_id ON %I(vehicle_id)', partition_name, partition_name);
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_order_id ON %I(order_id)', partition_name, partition_name);
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_track_time ON %I(track_time DESC)', partition_name, partition_name);
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_vehicle_time ON %I(vehicle_id, track_time DESC)', partition_name, partition_name);
        END IF;
        
        -- 处理weighing_data
        partition_name := 'weighing_data_' || TO_CHAR(partition_start, 'YYYYMM');
        IF NOT EXISTS (
            SELECT 1 FROM pg_class 
            WHERE relname = partition_name AND relkind = 'r'
        ) THEN
            EXECUTE format(
                'CREATE TABLE IF NOT EXISTS %I PARTITION OF weighing_data
                 FOR VALUES FROM (%L) TO (%L)',
                partition_name, partition_start, partition_end
            );
            -- 创建索引
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_vehicle_id ON %I(vehicle_id)', partition_name, partition_name);
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_device_id ON %I(device_id)', partition_name, partition_name);
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_weighing_time ON %I(weighing_time DESC)', partition_name, partition_name);
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_status ON %I(status)', partition_name, partition_name);
        END IF;
        
        -- 处理audit_logs
        partition_name := 'audit_logs_' || TO_CHAR(partition_start, 'YYYYMM');
        IF NOT EXISTS (
            SELECT 1 FROM pg_class 
            WHERE relname = partition_name AND relkind = 'r'
        ) THEN
            EXECUTE format(
                'CREATE TABLE IF NOT EXISTS %I PARTITION OF audit_logs
                 FOR VALUES FROM (%L) TO (%L)',
                partition_name, partition_start, partition_end
            );
            -- 创建索引
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_user_id ON %I(user_id)', partition_name, partition_name);
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_action ON %I(action)', partition_name, partition_name);
            EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_action_time ON %I(action_time DESC)', partition_name, partition_name);
        END IF;
    END LOOP;
    
    -- 调用归档函数，处理超过3个月的旧分区
    PERFORM archive_old_partitions();
    
    RETURN;
END;
$$ LANGUAGE plpgsql;

-- 7. 创建冷热数据联合查询视图
-- 轨迹数据联合视图
CREATE OR REPLACE VIEW logistics_tracks_all AS
SELECT 
    track_id, order_id, vehicle_id, track_time, latitude, longitude, 
    address, status, remark, create_time,
    FALSE AS is_archived
FROM logistics_tracks
UNION ALL
SELECT 
    track_id, order_id, vehicle_id, track_time, latitude, longitude, 
    address, status, remark, create_time,
    TRUE AS is_archived
FROM logistics_tracks_archive;

-- 称重数据联合视图
CREATE OR REPLACE VIEW weighing_data_all AS
SELECT 
    id, vehicle_id, device_id, weighing_time, gross_weight, tare_weight, 
    net_weight, axle_count, speed, lane_no, site_id, status, 
    create_time, update_time,
    FALSE AS is_archived
FROM weighing_data
UNION ALL
SELECT 
    id, vehicle_id, device_id, weighing_time, gross_weight, tare_weight, 
    net_weight, axle_count, speed, lane_no, site_id, status, 
    create_time, update_time,
    TRUE AS is_archived
FROM weighing_data_archive;

-- 审计日志联合视图
CREATE OR REPLACE VIEW audit_logs_all AS
SELECT 
    id, user_id, username, action, resource, resource_id, 
    request_data, ip_address, user_agent, action_time, result, error_message,
    FALSE AS is_archived
FROM audit_logs
UNION ALL
SELECT 
    id, user_id, username, action, resource, resource_id, 
    request_data, ip_address, user_agent, action_time, result, error_message,
    TRUE AS is_archived
FROM audit_logs_archive;

-- 8. 创建定期执行分区管理的定时任务
-- 注意：PostgreSQL 10+ 需要安装 pg_cron 扩展才能使用定时任务
-- 安装命令：CREATE EXTENSION IF NOT EXISTS pg_cron;

-- 每天凌晨2点执行分区管理和冷数据归档
-- SELECT cron.schedule('0 2 * * *', 'SELECT manage_partitions();');

-- 9. 创建冷热数据查询函数，优化查询性能
CREATE OR REPLACE FUNCTION get_vehicle_tracks(
    p_vehicle_id INTEGER,
    p_start_time TIMESTAMP,
    p_end_time TIMESTAMP
)
RETURNS TABLE (
    track_id INTEGER,
    order_id INTEGER,
    vehicle_id INTEGER,
    track_time TIMESTAMP,
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    address TEXT,
    status SMALLINT,
    remark TEXT,
    create_time TIMESTAMP,
    is_archived BOOLEAN
)
AS $$
BEGIN
    -- 优先查询热数据
    RETURN QUERY
    SELECT 
        t.track_id, t.order_id, t.vehicle_id, t.track_time, t.latitude, t.longitude, 
        t.address, t.status, t.remark, t.create_time, FALSE AS is_archived
    FROM logistics_tracks t
    WHERE t.vehicle_id = p_vehicle_id
      AND t.track_time BETWEEN p_start_time AND p_end_time
    UNION ALL
    -- 仅当需要时查询冷数据
    SELECT 
        a.track_id, a.order_id, a.vehicle_id, a.track_time, a.latitude, a.longitude, 
        a.address, a.status, a.remark, a.create_time, TRUE AS is_archived
    FROM logistics_tracks_archive a
    WHERE a.vehicle_id = p_vehicle_id
      AND a.track_time BETWEEN p_start_time AND p_end_time;
END;
$$ LANGUAGE plpgsql;

-- 10. 创建称重数据查询函数
CREATE OR REPLACE FUNCTION get_vehicle_weighing_data(
    p_vehicle_id INTEGER,
    p_start_time TIMESTAMP,
    p_end_time TIMESTAMP
)
RETURNS TABLE (
    id INTEGER,
    vehicle_id INTEGER,
    device_id VARCHAR(50),
    weighing_time TIMESTAMP,
    gross_weight DOUBLE PRECISION,
    tare_weight DOUBLE PRECISION,
    net_weight DOUBLE PRECISION,
    axle_count INTEGER,
    speed DOUBLE PRECISION,
    lane_no INTEGER,
    site_id INTEGER,
    status INTEGER,
    create_time TIMESTAMP,
    update_time TIMESTAMP,
    is_archived BOOLEAN
)
AS $$
BEGIN
    -- 优先查询热数据
    RETURN QUERY
    SELECT 
        w.id, w.vehicle_id, w.device_id, w.weighing_time, w.gross_weight, w.tare_weight, 
        w.net_weight, w.axle_count, w.speed, w.lane_no, w.site_id, w.status, 
        w.create_time, w.update_time, FALSE AS is_archived
    FROM weighing_data w
    WHERE w.vehicle_id = p_vehicle_id
      AND w.weighing_time BETWEEN p_start_time AND p_end_time
    UNION ALL
    -- 仅当需要时查询冷数据
    SELECT 
        a.id, a.vehicle_id, a.device_id, a.weighing_time, a.gross_weight, a.tare_weight, 
        a.net_weight, a.axle_count, a.speed, a.lane_no, a.site_id, a.status, 
        a.create_time, a.update_time, TRUE AS is_archived
    FROM weighing_data_archive a
    WHERE a.vehicle_id = p_vehicle_id
      AND a.weighing_time BETWEEN p_start_time AND p_end_time;
END;
$$ LANGUAGE plpgsql;


