-- Truck Scale 协议适配层 - 数据库架构
-- 版本: 1.0
-- 日期: 2026-02-08

-- ============================================================
-- 车辆管理表
-- ============================================================

-- 车辆表（43个字段）
CREATE TABLE IF NOT EXISTS truck_scale_vehicles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id VARCHAR(50) UNIQUE NOT NULL,
    plate_no VARCHAR(20) NOT NULL,
    terminal_no VARCHAR(50),
    sim_no VARCHAR(20),
    engine_no VARCHAR(50),
    frame_no VARCHAR(50),
    owner_name VARCHAR(100),
    owner_tel VARCHAR(20),
    owner_address VARCHAR(255),
    vehicle_type VARCHAR(50),
    vehicle_color VARCHAR(50),
    vehicle_brand VARCHAR(100),
    vehicle_model VARCHAR(100),
    group_id VARCHAR(50) REFERENCES truck_scale_vehicle_groups(group_id),
    driver_name VARCHAR(100),
    driver_tel VARCHAR(20),
    driver_license VARCHAR(50),
    max_weight DECIMAL(10,2) DEFAULT 0,
    tare_weight DECIMAL(10,2) DEFAULT 0,
    rated_weight DECIMAL(10,2) DEFAULT 0,
    length DECIMAL(10,2) DEFAULT 0,
    width DECIMAL(10,2) DEFAULT 0,
    height DECIMAL(10,2) DEFAULT 0,
    fuel_type VARCHAR(50),
    manufacturer VARCHAR(100),
    manufacture_date DATE,
    registration_date DATE,
    insurance_expire_date DATE,
    annual_inspection_date DATE,
    remark TEXT,
    
    -- 系统字段
    status INTEGER DEFAULT 0,            -- 0=正常，1=删除（回收站）
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    create_by VARCHAR(50),
    update_by VARCHAR(50)
);

-- 车辆表索引
CREATE INDEX idx_truck_scale_vehicles_plate_no ON truck_scale_vehicles(plate_no);
CREATE INDEX idx_truck_scale_vehicles_group_id ON truck_scale_vehicles(group_id);
CREATE INDEX idx_truck_scale_vehicles_status ON truck_scale_vehicles(status);
CREATE INDEX idx_truck_scale_vehicles_create_time ON truck_scale_vehicles(create_time);

-- 车组表（5个字段）
CREATE TABLE IF NOT EXISTS truck_scale_vehicle_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id VARCHAR(50) UNIQUE NOT NULL,
    parent_id VARCHAR(50),
    group_name VARCHAR(100) NOT NULL,
    contact_people VARCHAR(100),
    contact_tel VARCHAR(20),
    
    -- 系统字段
    status INTEGER DEFAULT 0,            -- 0=正常，1=删除（回收站）
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    create_by VARCHAR(50),
    update_by VARCHAR(50)
);

-- 车组表索引
CREATE INDEX idx_truck_scale_vehicle_groups_parent_id ON truck_scale_vehicle_groups(parent_id);
CREATE INDEX idx_truck_scale_vehicle_groups_status ON truck_scale_vehicle_groups(status);

-- ============================================================
-- 用户管理表
-- ============================================================

-- 用户表（43个字段）
CREATE TABLE IF NOT EXISTS truck_scale_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(50) UNIQUE NOT NULL,
    user_name VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    real_name VARCHAR(100),
    user_type INTEGER DEFAULT 3,          -- 0=超级管理员，1=管理员，2=操作员，3=普通用户
    group_id VARCHAR(50) REFERENCES truck_scale_user_groups(group_id),
    company VARCHAR(100),
    department VARCHAR(100),
    tel VARCHAR(20),
    mobile VARCHAR(20),
    email VARCHAR(100),
    address VARCHAR(255),
    permission TEXT,
    veh_group_list TEXT,
    
    -- 账户状态
    status INTEGER DEFAULT 0,            -- 0=正常，1=禁用，2=锁定
    expiration_time TIMESTAMP,
    
    -- 其他字段
    title VARCHAR(50),
    id_card VARCHAR(20),
    id_card_expire_date DATE,
    education VARCHAR(50),
    birth_date DATE,
    gender INTEGER DEFAULT 0,            -- 0=男，1=女
    avatar VARCHAR(255),
    signature VARCHAR(255),
    
    -- 登录信息
    last_login_time TIMESTAMP,
    last_login_ip VARCHAR(50),
    login_count INTEGER DEFAULT 0,
    
    -- 系统字段
    remark TEXT,
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    create_by VARCHAR(50),
    update_by VARCHAR(50)
);

-- 用户表索引
CREATE INDEX idx_truck_scale_users_user_name ON truck_scale_users(user_name);
CREATE INDEX idx_truck_scale_users_group_id ON truck_scale_users(group_id);
CREATE INDEX idx_truck_scale_users_status ON truck_scale_users(status);
CREATE INDEX idx_truck_scale_users_user_type ON truck_scale_users(user_type);

-- 用户组表（4个字段）
CREATE TABLE IF NOT EXISTS truck_scale_user_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id VARCHAR(50) UNIQUE NOT NULL,
    group_name VARCHAR(100) NOT NULL,
    user_type INTEGER DEFAULT 3,
    permission TEXT,
    
    -- 系统字段
    status INTEGER DEFAULT 0,            -- 0=正常，1=删除（回收站）
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    create_by VARCHAR(50),
    update_by VARCHAR(50)
);

-- 用户组表索引
CREATE INDEX idx_truck_scale_user_groups_status ON truck_scale_user_groups(status);

-- ============================================================
-- 会话管理表
-- ============================================================

-- 会话表
CREATE TABLE IF NOT EXISTS truck_scale_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(100) UNIQUE NOT NULL,
    user_id VARCHAR(50) REFERENCES truck_scale_users(user_id),
    connection_id VARCHAR(100),
    login_time TIMESTAMP NOT NULL,
    last_heartbeat TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    logout_time TIMESTAMP,
    
    client_ip VARCHAR(50),
    client_version VARCHAR(50),
    
    status INTEGER DEFAULT 0,            -- 0=活跃，1=已登出，2=超时
    create_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 会话表索引
CREATE INDEX idx_truck_scale_sessions_user_id ON truck_scale_sessions(user_id);
CREATE INDEX idx_truck_scale_sessions_expires_at ON truck_scale_sessions(expires_at);
CREATE INDEX idx_truck_scale_sessions_status ON truck_scale_sessions(status);
CREATE INDEX idx_truck_scale_sessions_connection_id ON truck_scale_sessions(connection_id);

-- ============================================================
-- 回收站表（可选，用于软删除记录）
-- ============================================================

-- 车辆回收站表
CREATE TABLE IF NOT EXISTS truck_scale_vehicle_recycle_bin (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vehicle_id VARCHAR(50) NOT NULL,
    original_id UUID NOT NULL,
    plate_no VARCHAR(20),
    group_id VARCHAR(50),
    delete_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    delete_by VARCHAR(50),
    reason TEXT
);

-- 用户回收站表
CREATE TABLE IF NOT EXISTS truck_scale_user_recycle_bin (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(50) NOT NULL,
    original_id UUID NOT NULL,
    user_name VARCHAR(50),
    group_id VARCHAR(50),
    delete_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    delete_by VARCHAR(50),
    reason TEXT
);

-- ============================================================
-- 更新时间触发器
-- ============================================================

-- 车辆表更新时间触发器
CREATE OR REPLACE FUNCTION update_truck_scale_vehicles_update_time()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_time = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_truck_scale_vehicles_update_time
    BEFORE UPDATE ON truck_scale_vehicles
    FOR EACH ROW
    EXECUTE FUNCTION update_truck_scale_vehicles_update_time();

-- 车组表更新时间触发器
CREATE OR REPLACE FUNCTION update_truck_scale_vehicle_groups_update_time()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_time = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_truck_scale_vehicle_groups_update_time
    BEFORE UPDATE ON truck_scale_vehicle_groups
    FOR EACH ROW
    EXECUTE FUNCTION update_truck_scale_vehicle_groups_update_time();

-- 用户表更新时间触发器
CREATE OR REPLACE FUNCTION update_truck_scale_users_update_time()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_time = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_truck_scale_users_update_time
    BEFORE UPDATE ON truck_scale_users
    FOR EACH ROW
    EXECUTE FUNCTION update_truck_scale_users_update_time();

-- 用户组表更新时间触发器
CREATE OR REPLACE FUNCTION update_truck_scale_user_groups_update_time()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_time = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_truck_scale_user_groups_update_time
    BEFORE UPDATE ON truck_scale_user_groups
    FOR EACH ROW
    EXECUTE FUNCTION update_truck_scale_user_groups_update_time();

-- ============================================================
-- 初始化数据
-- ============================================================

-- 创建默认超级管理员用户
INSERT INTO truck_scale_users (
    user_id, user_name, password_hash, real_name, user_type, status
) VALUES (
    'admin',
    'admin',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYz7xH9QqUe', -- bcrypt hash of 'admin123'
    '超级管理员',
    0, -- SuperAdmin
    0
) ON CONFLICT (user_id) DO NOTHING;

-- 创建默认用户组
INSERT INTO truck_scale_user_groups (group_id, group_name, user_type, status)
VALUES 
    ('admin_group', '管理员组', 1, 0),
    ('operator_group', '操作员组', 2, 0),
    ('user_group', '用户组', 3, 0)
ON CONFLICT (group_id) DO NOTHING;

-- ============================================================
-- 授权
-- ============================================================

-- 授予必要的权限（根据实际数据库用户调整）
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO CarpTMS;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO CarpTMS;

COMMENT ON TABLE truck_scale_vehicles IS 'Truck Scale 车辆表';
COMMENT ON TABLE truck_scale_vehicle_groups IS 'Truck Scale 车组表';
COMMENT ON TABLE truck_scale_users IS 'Truck Scale 用户表';
COMMENT ON TABLE truck_scale_user_groups IS 'Truck Scale 用户组表';
COMMENT ON TABLE truck_scale_sessions IS 'Truck Scale 会话表';
COMMENT ON TABLE truck_scale_vehicle_recycle_bin IS 'Truck Scale 车辆回收站表';
COMMENT ON TABLE truck_scale_user_recycle_bin IS 'Truck Scale 用户回收站表';


