-- CarpTMS 数据库修复脚本
-- 用于修复表结构和字段名不一致问题

-- ============================================================================
-- 1. 检查并创建 sensor_data_aggregated 表
-- ============================================================================

CREATE TABLE IF NOT EXISTS sensor_data_aggregated (
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
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_sensor_agg_vehicle_type ON sensor_data_aggregated(vehicle_id, sensor_type);
CREATE INDEX IF NOT EXISTS idx_sensor_agg_time ON sensor_data_aggregated(start_time, end_time);

-- ============================================================================
-- 2. 修复 users 表字段命名 (username -> user_name)
-- ============================================================================

-- 检查是否需要重命名字段
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'users' AND column_name = 'username') THEN
        ALTER TABLE users RENAME COLUMN username TO user_name;
    END IF;
END $$;

-- ============================================================================
-- 3. 修复 vehicles 表字段命名 (updated_at -> update_time)
-- ============================================================================

-- 检查是否需要重命名字段
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'vehicles' AND column_name = 'updated_at') THEN
        ALTER TABLE vehicles RENAME COLUMN updated_at TO update_time;
    END IF;
END $$;

-- ============================================================================
-- 4. 确保 sensor_data 表使用正确的字段名
-- ============================================================================

-- 检查 collect_time 字段是否存在
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                    WHERE table_name = 'sensor_data' AND column_name = 'collect_time') THEN
        -- 如果不存在，重命名 sensor_time 为 collect_time
        IF EXISTS (SELECT 1 FROM information_schema.columns 
                       WHERE table_name = 'sensor_data' AND column_name = 'sensor_time') THEN
            ALTER TABLE sensor_data RENAME COLUMN sensor_time TO collect_time;
        END IF;
    END IF;
END $$;

-- ============================================================================
-- 5. 创建视图以提供向后兼容性（可选）
-- ============================================================================

-- 如果需要保持旧字段名的兼容性，可以创建视图
CREATE OR REPLACE VIEW sensor_data_legacy AS
SELECT 
    id,
    vehicle_id,
    sensor_type,
    collect_time AS sensor_time,  -- 别名以支持旧代码
    sensor_value,
    unit,
    status,
    create_time
FROM sensor_data;

-- ============================================================================
-- 6. 验证修复
-- ============================================================================

-- 输出修复状态
SELECT 
    'sensor_data_aggregated 表存在' AS check_item,
    EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'sensor_data_aggregated') AS status;

SELECT 
    'users 表使用 user_name 字段' AS check_item,
    EXISTS(SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'user_name') AS status;

SELECT 
    'vehicles 表使用 update_time 字段' AS check_item,
    EXISTS(SELECT 1 FROM information_schema.columns WHERE table_name = 'vehicles' AND column_name = 'update_time') AS status;

SELECT 
    'sensor_data 表使用 collect_time 字段' AS check_item,
    EXISTS(SELECT 1 FROM information_schema.columns WHERE table_name = 'sensor_data' AND column_name = 'collect_time') AS status;

-- ============================================================================
-- 完成
-- ============================================================================

-- 输出修复完成消息
DO $$
BEGIN
    RAISE NOTICE '数据库修复脚本执行完成！';
    RAISE NOTICE '请检查上述验证结果，确保所有表和字段都正确。';
END $$;


