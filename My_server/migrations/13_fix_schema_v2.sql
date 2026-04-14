-- CarpTMS 数据库修复脚本 v2
-- 修复表结构不匹配问题

-- ============================================================================
-- 1. 修复 sensor_data 表 - 添加缺失的 status 字段
-- ============================================================================

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'sensor_data' AND column_name = 'status') THEN
        ALTER TABLE sensor_data ADD COLUMN status INTEGER DEFAULT 1;
        RAISE NOTICE 'Added status column to sensor_data table';
    END IF;
END $$;

-- ============================================================================
-- 2. 修复 sensor_data_aggregated 表 - 添加缺失的 updated_at 字段
-- ============================================================================

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables 
                   WHERE table_name = 'sensor_data_aggregated') THEN
        CREATE TABLE sensor_data_aggregated (
            id SERIAL PRIMARY KEY,
            vehicle_id INTEGER NOT NULL REFERENCES vehicles(vehicle_id),
            sensor_type VARCHAR(50) NOT NULL,
            start_time TIMESTAMP NOT NULL,
            end_time TIMESTAMP NOT NULL,
            count INTEGER NOT NULL,
            min_value NUMERIC(20, 4),
            max_value NUMERIC(20, 4),
            avg_value NUMERIC(20, 4),
            sum_value NUMERIC(20, 4),
            unit VARCHAR(50),
            create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        RAISE NOTICE 'Created sensor_data_aggregated table';
    ELSE
        IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                       WHERE table_name = 'sensor_data_aggregated' AND column_name = 'updated_at') THEN
            ALTER TABLE sensor_data_aggregated ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;
            RAISE NOTICE 'Added updated_at column to sensor_data_aggregated table';
        END IF;
    END IF;
END $$;

-- ============================================================================
-- 3. 修复 vehicles 表 - 添加 update_time 字段（向后兼容 updated_at）
-- ============================================================================

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name = 'vehicles' AND column_name = 'updated_at') THEN
        ALTER TABLE vehicles RENAME COLUMN updated_at TO update_time;
        RAISE NOTICE 'Renamed updated_at to update_time in vehicles table';
    ELSIF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                        WHERE table_name = 'vehicles' AND column_name = 'update_time') THEN
        ALTER TABLE vehicles ADD COLUMN update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP;
        RAISE NOTICE 'Added update_time column to vehicles table';
    END IF;
END $$;

-- ============================================================================
-- 4. 修复 users 表 - 添加 user_name 字段（向后兼容 username）
-- ============================================================================

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name = 'users' AND column_name = 'username') THEN
        ALTER TABLE users RENAME COLUMN username TO user_name;
        RAISE NOTICE 'Renamed username to user_name in users table';
    ELSIF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                        WHERE table_name = 'users' AND column_name = 'user_name') THEN
        ALTER TABLE users ADD COLUMN user_name VARCHAR(50);
        RAISE NOTICE 'Added user_name column to users table';
    END IF;
END $$;

-- ============================================================================
-- 5. 创建索引以优化查询性能
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_sensor_data_vehicle_time ON sensor_data(vehicle_id, collect_time);
CREATE INDEX IF NOT EXISTS idx_sensor_data_type_time ON sensor_data(sensor_type, collect_time);
CREATE INDEX IF NOT EXISTS idx_sensor_agg_vehicle_type ON sensor_data_aggregated(vehicle_id, sensor_type);
CREATE INDEX IF NOT EXISTS idx_sensor_agg_time ON sensor_data_aggregated(start_time, end_time);

-- ============================================================================
-- 6. 验证修复结果
-- ============================================================================

SELECT 
    'sensor_data.status 字段存在' AS check_item,
    EXISTS(SELECT 1 FROM information_schema.columns WHERE table_name = 'sensor_data' AND column_name = 'status') AS status;

SELECT 
    'sensor_data_aggregated 表存在' AS check_item,
    EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'sensor_data_aggregated') AS status;

SELECT 
    'sensor_data_aggregated.updated_at 字段存在' AS check_item,
    EXISTS(SELECT 1 FROM information_schema.columns WHERE table_name = 'sensor_data_aggregated' AND column_name = 'updated_at') AS status;

SELECT 
    'vehicles.update_time 字段存在' AS check_item,
    EXISTS(SELECT 1 FROM information_schema.columns WHERE table_name = 'vehicles' AND column_name = 'update_time') AS status;

SELECT 
    'users.user_name 字段存在' AS check_item,
    EXISTS(SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'user_name') AS status;

-- ============================================================================
-- 完成
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE '数据库修复脚本执行完成！';
END $$;


