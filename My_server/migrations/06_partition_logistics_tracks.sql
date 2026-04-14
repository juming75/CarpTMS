-- 物流轨迹表分表实现
-- 1. 删除原表（如果存在）
DROP TABLE IF EXISTS logistics_tracks;

-- 2. 创建主表（分区表的模板）
CREATE TABLE logistics_tracks (
    track_id SERIAL,
    order_id INTEGER NOT NULL,
    vehicle_id INTEGER NOT NULL,
    track_time TIMESTAMP NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    address TEXT,
    status SMALLINT NOT NULL DEFAULT 1,
    remark TEXT,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束：主键必须包含分区键track_time
    PRIMARY KEY (track_id, track_time),
    FOREIGN KEY (order_id) REFERENCES orders(order_id),
    FOREIGN KEY (vehicle_id) REFERENCES vehicles(vehicle_id)
) PARTITION BY RANGE (track_time);

-- 3. 创建索引模板（会被所有分区继承）
CREATE INDEX IF NOT EXISTS idx_logistics_tracks_vehicle_id 
ON logistics_tracks(vehicle_id);

CREATE INDEX IF NOT EXISTS idx_logistics_tracks_order_id 
ON logistics_tracks(order_id);

CREATE INDEX IF NOT EXISTS idx_logistics_tracks_track_time 
ON logistics_tracks(track_time DESC);

CREATE INDEX IF NOT EXISTS idx_logistics_tracks_vehicle_time 
ON logistics_tracks(vehicle_id, track_time DESC);

-- 4. 创建分区函数和触发器
-- 4.1 创建函数用于自动创建分区
CREATE OR REPLACE FUNCTION create_logistics_tracks_partition()
RETURNS TRIGGER AS $$
DECLARE
    partition_name TEXT;
    partition_start TIMESTAMP;
    partition_end TIMESTAMP;
BEGIN
    -- 按月份分区，格式：logistics_tracks_202601
    partition_name := 'logistics_tracks_' || TO_CHAR(NEW.track_time, 'YYYYMM');
    partition_start := DATE_TRUNC('month', NEW.track_time);
    partition_end := partition_start + INTERVAL '1 month';
    
    -- 检查分区是否存在，不存在则创建
    IF NOT EXISTS (
        SELECT 1 FROM pg_class 
        WHERE relname = partition_name AND relkind = 'r'
    ) THEN
        -- 创建新分区
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %I PARTITION OF logistics_tracks
             FOR VALUES FROM (%L) TO (%L)',
            partition_name, partition_start, partition_end
        );
        
        -- 在分区上创建索引（PostgreSQL 11+ 会自动继承，但显式创建更清晰）
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_vehicle_id ON %I(vehicle_id)', partition_name, partition_name);
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_order_id ON %I(order_id)', partition_name, partition_name);
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_track_time ON %I(track_time DESC)', partition_name, partition_name);
        EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_vehicle_time ON %I(vehicle_id, track_time DESC)', partition_name, partition_name);
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 4.2 创建触发器，在插入数据前检查并创建分区
CREATE TRIGGER trg_create_logistics_tracks_partition
BEFORE INSERT ON logistics_tracks
FOR EACH ROW
EXECUTE FUNCTION create_logistics_tracks_partition();

-- 5. 创建历史数据迁移函数（可选，用于迁移已有数据）
CREATE OR REPLACE FUNCTION migrate_logistics_tracks_data()
RETURNS VOID AS $$
DECLARE
    month_record RECORD;
BEGIN
    -- 获取所有已存在的数据的月份范围
    FOR month_record IN 
        SELECT DISTINCT DATE_TRUNC('month', track_time) AS month_start
        FROM logistics_tracks
        ORDER BY month_start
    LOOP
        -- 手动调用分区创建函数
        PERFORM create_logistics_tracks_partition();
    END LOOP;
    
    RETURN;
END;
$$ LANGUAGE plpgsql;

-- 6. 为称重数据表创建类似的分区方案
-- 6.1 删除原表（如果存在）
DROP TABLE IF EXISTS weighing_data;

-- 6.2 创建主表
CREATE TABLE weighing_data (
    id SERIAL,
    vehicle_id INTEGER NOT NULL REFERENCES vehicles(vehicle_id),
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
    
    -- 约束：主键必须包含分区键weighing_time
    PRIMARY KEY (id, weighing_time)
) PARTITION BY RANGE (weighing_time);

-- 6.3 创建索引模板
CREATE INDEX IF NOT EXISTS idx_weighing_data_vehicle_id ON weighing_data(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_weighing_data_device_id ON weighing_data(device_id);
CREATE INDEX IF NOT EXISTS idx_weighing_data_weighing_time ON weighing_data(weighing_time DESC);
CREATE INDEX IF NOT EXISTS idx_weighing_data_status ON weighing_data(status);

-- 6.4 创建分区函数和触发器
CREATE OR REPLACE FUNCTION create_weighing_data_partition()
RETURNS TRIGGER AS $$
DECLARE
    partition_name TEXT;
    partition_start TIMESTAMP;
    partition_end TIMESTAMP;
BEGIN
    -- 按月份分区，格式：weighing_data_202601
    partition_name := 'weighing_data_' || TO_CHAR(NEW.weighing_time, 'YYYYMM');
    partition_start := DATE_TRUNC('month', NEW.weighing_time);
    partition_end := partition_start + INTERVAL '1 month';
    
    -- 检查分区是否存在，不存在则创建
    IF NOT EXISTS (
        SELECT 1 FROM pg_class 
        WHERE relname = partition_name AND relkind = 'r'
    ) THEN
        -- 创建新分区
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
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_create_weighing_data_partition
BEFORE INSERT ON weighing_data
FOR EACH ROW
EXECUTE FUNCTION create_weighing_data_partition();

-- 7. 为审计日志表创建分区方案
-- 7.1 删除原表（如果存在）
DROP TABLE IF EXISTS audit_logs;

-- 7.2 创建主表
CREATE TABLE audit_logs (
    id SERIAL,
    user_id INTEGER REFERENCES users(user_id),
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
    
    -- 约束：主键必须包含分区键action_time
    PRIMARY KEY (id, action_time)
) PARTITION BY RANGE (action_time);

-- 7.3 创建索引模板
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action_time ON audit_logs(action_time DESC);

-- 7.4 创建分区函数和触发器
CREATE OR REPLACE FUNCTION create_audit_logs_partition()
RETURNS TRIGGER AS $$
DECLARE
    partition_name TEXT;
    partition_start TIMESTAMP;
    partition_end TIMESTAMP;
BEGIN
    -- 按月份分区，格式：audit_logs_202601
    partition_name := 'audit_logs_' || TO_CHAR(NEW.action_time, 'YYYYMM');
    partition_start := DATE_TRUNC('month', NEW.action_time);
    partition_end := partition_start + INTERVAL '1 month';
    
    -- 检查分区是否存在，不存在则创建
    IF NOT EXISTS (
        SELECT 1 FROM pg_class 
        WHERE relname = partition_name AND relkind = 'r'
    ) THEN
        -- 创建新分区
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
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_create_audit_logs_partition
BEFORE INSERT ON audit_logs
FOR EACH ROW
EXECUTE FUNCTION create_audit_logs_partition();

-- 8. 添加分区管理函数，用于定期维护分区
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
    
    -- 可选：删除超过1年的旧分区（根据业务需求调整）
    old_partition_start := current_month - INTERVAL '12 months';
    
    -- 处理logistics_tracks旧分区
    old_partition := 'logistics_tracks_' || TO_CHAR(old_partition_start, 'YYYYMM');
    IF EXISTS (
        SELECT 1 FROM pg_class 
        WHERE relname = old_partition AND relkind = 'r'
    ) THEN
        EXECUTE format('DROP TABLE IF EXISTS %I', old_partition);
    END IF;
    
    -- 处理weighing_data旧分区
    old_partition := 'weighing_data_' || TO_CHAR(old_partition_start, 'YYYYMM');
    IF EXISTS (
        SELECT 1 FROM pg_class 
        WHERE relname = old_partition AND relkind = 'r'
    ) THEN
        EXECUTE format('DROP TABLE IF EXISTS %I', old_partition);
    END IF;
    
    -- 处理audit_logs旧分区
    old_partition := 'audit_logs_' || TO_CHAR(old_partition_start, 'YYYYMM');
    IF EXISTS (
        SELECT 1 FROM pg_class 
        WHERE relname = old_partition AND relkind = 'r'
    ) THEN
        EXECUTE format('DROP TABLE IF EXISTS %I', old_partition);
    END IF;
    
    RETURN;
END;
$$ LANGUAGE plpgsql;


