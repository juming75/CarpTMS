-- 创建用户组表
CREATE TABLE IF NOT EXISTS user_groups (
    group_id SERIAL PRIMARY KEY,
    group_name VARCHAR(100) NOT NULL,
    description TEXT,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    user_id SERIAL PRIMARY KEY,
    user_name VARCHAR(50) NOT NULL UNIQUE,
    password VARCHAR(100) NOT NULL,
    email TEXT,
    phone VARCHAR(20),
    group_id INTEGER,
    user_group_id INTEGER NOT NULL REFERENCES user_groups(group_id),
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

-- 创建车组表
CREATE TABLE IF NOT EXISTS vehicle_groups (
    group_id SERIAL PRIMARY KEY,
    group_name VARCHAR(100) NOT NULL,
    parent_id INTEGER REFERENCES vehicle_groups(group_id),
    description TEXT,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

-- 创建设备表
CREATE TABLE IF NOT EXISTS devices (
    device_id VARCHAR(50) PRIMARY KEY,
    device_name VARCHAR(100) NOT NULL,
    device_type VARCHAR(50) NOT NULL,
    device_model VARCHAR(50) NOT NULL,
    manufacturer VARCHAR(100) NOT NULL,
    serial_number VARCHAR(100) NOT NULL,
    communication_type VARCHAR(50) NOT NULL,
    sim_card_no VARCHAR(20),
    ip_address VARCHAR(50),
    port INTEGER,
    mac_address VARCHAR(50),
    install_date TIMESTAMP,
    install_address TEXT,
    install_technician VARCHAR(100),
    status SMALLINT NOT NULL DEFAULT 1,
    remark TEXT,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP,
    create_user_id INTEGER NOT NULL DEFAULT 1,
    update_user_id INTEGER
);

-- 创建称重数据表
CREATE TABLE IF NOT EXISTS weighing_data (
    id SERIAL PRIMARY KEY,
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
    update_time TIMESTAMP
);

-- 创建同步日志表
CREATE TABLE IF NOT EXISTS sync_logs (
    id SERIAL PRIMARY KEY,
    table_name VARCHAR(50) NOT NULL,
    sync_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    total_count INTEGER NOT NULL,
    success_count INTEGER NOT NULL,
    fail_count INTEGER NOT NULL,
    sync_type VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL
);

-- 创建审计日志表
CREATE TABLE IF NOT EXISTS audit_logs (
    id SERIAL PRIMARY KEY,
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
    error_message TEXT
);

-- 插入默认用户组
INSERT INTO user_groups (group_name, description) 
VALUES 
    ('管理员组', '系统管理员组'),
    ('普通用户组', '普通用户组')
ON CONFLICT DO NOTHING;

-- 插入默认管理员用户
INSERT INTO users (user_name, password, group_id, user_group_id) 
VALUES 
    ('admin', '$2a$12$UQK2T0n39V5OYh1O8eT9YOh3v9QjJj9YOh3v9QjJj9YOh3v9QjJj', 1, 1)
ON CONFLICT DO NOTHING;

-- 插入默认车辆分组
INSERT INTO vehicle_groups (group_name, description) 
VALUES 
    ('默认分组', '系统默认车辆分组'),
    ('企业A', '企业A车辆分组，存放浙牌车辆'),
    ('企业B', '企业B车辆分组，存放浙牌和川牌车辆')
ON CONFLICT DO NOTHING;

-- 创建设备数据表
CREATE TABLE IF NOT EXISTS device_data (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(100) NOT NULL,
    command VARCHAR(100) NOT NULL,
    params JSONB NOT NULL DEFAULT '{}',
    raw_data BYTEA,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 创建报表模板表
CREATE TABLE IF NOT EXISTS report_templates (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    type SMALLINT NOT NULL,
    config JSONB NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT false,
    create_user_id INTEGER NOT NULL REFERENCES users(user_id),
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

-- 创建报表数据表
CREATE TABLE IF NOT EXISTS report_data (
    id SERIAL PRIMARY KEY,
    template_id INTEGER NOT NULL REFERENCES report_templates(id),
    report_time TIMESTAMP NOT NULL,
    period_type SMALLINT NOT NULL,
    period_value VARCHAR(50) NOT NULL,
    data JSONB NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

