-- ====================================
-- CarpTMS 数据库迁移检查脚本
-- ====================================

-- 检查位置管理表是否存在
SELECT '检查位置管理表...' AS info;

-- 1. 检查电子围栏表
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'location_fences') THEN
        CREATE TABLE location_fences (
            fence_id SERIAL PRIMARY KEY,
            fence_name VARCHAR(100) NOT NULL,
            fence_type VARCHAR(50) NOT NULL,
            center_latitude DECIMAL(10, 8),
            center_longitude DECIMAL(11, 8),
            radius DECIMAL(10, 2),
            polygon_points JSONB,
            rectangle_bounds JSONB,
            status VARCHAR(20) DEFAULT 'active',
            description TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            create_user_id INTEGER,
            update_user_id INTEGER
        );
        RAISE NOTICE '创建表 location_fences 成功';
    ELSE
        RAISE NOTICE '表 location_fences 已存在';
    END IF;
END $$;

-- 2. 检查位置表
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'locations') THEN
        CREATE TABLE locations (
            location_id SERIAL PRIMARY KEY,
            location_name VARCHAR(100) NOT NULL,
            latitude DECIMAL(10, 8) NOT NULL,
            longitude DECIMAL(11, 8) NOT NULL,
            address VARCHAR(255),
            description TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            create_user_id INTEGER,
            update_user_id INTEGER
        );
        RAISE NOTICE '创建表 locations 成功';
    ELSE
        RAISE NOTICE '表 locations 已存在';
    END IF;
END $$;

-- 3. 检查地点表
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'places') THEN
        CREATE TABLE places (
            place_id SERIAL PRIMARY KEY,
            place_name VARCHAR(100) NOT NULL,
            address VARCHAR(255) NOT NULL,
            contact_person VARCHAR(50),
            contact_phone VARCHAR(20),
            contact_email VARCHAR(100),
            latitude DECIMAL(10, 8),
            longitude DECIMAL(11, 8),
            description TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            create_user_id INTEGER,
            update_user_id INTEGER
        );
        RAISE NOTICE '创建表 places 成功';
    ELSE
        RAISE NOTICE '表 places 已存在';
    END IF;
END $$;

-- 4. 检查路线表
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'routes') THEN
        CREATE TABLE routes (
            route_id SERIAL PRIMARY KEY,
            route_name VARCHAR(100) NOT NULL,
            start_point VARCHAR(255) NOT NULL,
            start_latitude DECIMAL(10, 8),
            start_longitude DECIMAL(11, 8),
            end_point VARCHAR(255) NOT NULL,
            end_latitude DECIMAL(10, 8),
            end_longitude DECIMAL(11, 8),
            waypoints JSONB,
            distance DECIMAL(10, 2),
            estimated_duration INTEGER,
            description TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            create_user_id INTEGER,
            update_user_id INTEGER
        );
        RAISE NOTICE '创建表 routes 成功';
    ELSE
        RAISE NOTICE '表 routes 已存在';
    END IF;
END $$;

-- 5. 为 users 表添加部门 ID 字段
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'department_id') THEN
        ALTER TABLE users ADD COLUMN department_id INTEGER;
        ALTER TABLE users ADD CONSTRAINT fk_users_department 
            FOREIGN KEY (department_id) REFERENCES departments(department_id) ON DELETE SET NULL;
        CREATE INDEX idx_users_department_id ON users(department_id);
        RAISE NOTICE '为 users 表添加 department_id 字段成功';
    ELSE
        RAISE NOTICE 'users.department_id 字段已存在';
    END IF;
END $$;

-- 6. 插入示例数据（如果不存在）
INSERT INTO location_fences (fence_name, fence_type, status, description) 
VALUES ('仓库围栏', 'circle', 'active', '主仓库电子围栏')
ON CONFLICT DO NOTHING;

INSERT INTO locations (location_name, latitude, longitude, description) 
VALUES ('仓库 A', 39.9042, 116.4074, '主仓库')
ON CONFLICT DO NOTHING;

INSERT INTO places (place_name, address, contact_person, contact_phone, description) 
VALUES ('总部', '北京市朝阳区', '张三', '13800138000', '公司总部')
ON CONFLICT DO NOTHING;

INSERT INTO routes (route_name, start_point, end_point, distance, description) 
VALUES ('路线 1', '仓库 A', '客户 A', 50.0, '主要配送路线')
ON CONFLICT DO NOTHING;

SELECT '迁移完成！' AS info;


