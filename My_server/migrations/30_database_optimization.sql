-- 数据库优化
-- 创建必要的表
-- 1. 确保用户表存在
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(20),
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    department_id INTEGER REFERENCES departments(id),
    organization_id INTEGER REFERENCES organizations(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 2. 确保部门表存在
CREATE TABLE IF NOT EXISTS departments (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    code VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 3. 确保组织表存在
CREATE TABLE IF NOT EXISTS organizations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    code VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 4. 确保系统设置表存在
CREATE TABLE IF NOT EXISTS system_settings (
    id SERIAL PRIMARY KEY,
    key VARCHAR(255) UNIQUE NOT NULL,
    value JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 5. 确保组织设置表存在
CREATE TABLE IF NOT EXISTS organization_settings (
    id SERIAL PRIMARY KEY,
    organization_id INTEGER REFERENCES organizations(id),
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(organization_id, key)
);

-- 6. 确保位置相关表存在
CREATE TABLE IF NOT EXISTS location_fences (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    coordinates JSONB NOT NULL,
    radius INTEGER,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS location_positions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    altitude DOUBLE PRECISION,
    accuracy DOUBLE PRECISION,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS location_places (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS location_routes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL,
    waypoints JSONB NOT NULL,
    distance DOUBLE PRECISION,
    duration INTEGER,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 7. 确保报警表存在
CREATE TABLE IF NOT EXISTS alerts (
    id SERIAL PRIMARY KEY,
    vehicle_id INTEGER REFERENCES vehicles(id),
    type VARCHAR(50) NOT NULL,
    message TEXT NOT NULL,
    severity VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    coordinates JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 8. 确保仪表盘相关表存在
CREATE TABLE IF NOT EXISTS statistics (
    id SERIAL PRIMARY KEY,
    type VARCHAR(50) NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    unit VARCHAR(20),
    category VARCHAR(50),
    period VARCHAR(20),
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB
);

CREATE TABLE IF NOT EXISTS dashboard_tasks (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    priority VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    due_date TIMESTAMP,
    assignee_id INTEGER REFERENCES users(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS dashboard_dynamics (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    type VARCHAR(50) NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB
);

-- 数据库索引优化
-- 1. 用户表索引
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_department_id ON users(department_id);
CREATE INDEX IF NOT EXISTS idx_users_organization_id ON users(organization_id);

-- 2. 部门表索引
CREATE INDEX IF NOT EXISTS idx_departments_code ON departments(code);

-- 3. 组织表索引
CREATE INDEX IF NOT EXISTS idx_organizations_code ON organizations(code);

-- 4. 系统设置表索引
CREATE INDEX IF NOT EXISTS idx_system_settings_key ON system_settings(key);

-- 5. 组织设置表索引
CREATE INDEX IF NOT EXISTS idx_organization_settings_organization_id ON organization_settings(organization_id);
CREATE INDEX IF NOT EXISTS idx_organization_settings_key ON organization_settings(key);

-- 6. 位置相关表索引
CREATE INDEX IF NOT EXISTS idx_location_fences_type ON location_fences(type);
CREATE INDEX IF NOT EXISTS idx_location_positions_coordinates ON location_positions USING gist (point(longitude, latitude));
CREATE INDEX IF NOT EXISTS idx_location_places_type ON location_places(type);
CREATE INDEX IF NOT EXISTS idx_location_places_coordinates ON location_places USING gist (point(longitude, latitude));
CREATE INDEX IF NOT EXISTS idx_location_routes_type ON location_routes(type);

-- 7. 报警表索引
CREATE INDEX IF NOT EXISTS idx_alerts_vehicle_id ON alerts(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_alerts_type ON alerts(type);
CREATE INDEX IF NOT EXISTS idx_alerts_severity ON alerts(severity);
CREATE INDEX IF NOT EXISTS idx_alerts_status ON alerts(status);
CREATE INDEX IF NOT EXISTS idx_alerts_created_at ON alerts(created_at);

-- 8. 仪表盘相关表索引
CREATE INDEX IF NOT EXISTS idx_statistics_type ON statistics(type);
CREATE INDEX IF NOT EXISTS idx_statistics_category ON statistics(category);
CREATE INDEX IF NOT EXISTS idx_statistics_period ON statistics(period);
CREATE INDEX IF NOT EXISTS idx_statistics_timestamp ON statistics(timestamp);
CREATE INDEX IF NOT EXISTS idx_dashboard_tasks_priority ON dashboard_tasks(priority);
CREATE INDEX IF NOT EXISTS idx_dashboard_tasks_status ON dashboard_tasks(status);
CREATE INDEX IF NOT EXISTS idx_dashboard_tasks_assignee_id ON dashboard_tasks(assignee_id);
CREATE INDEX IF NOT EXISTS idx_dashboard_tasks_due_date ON dashboard_tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_dashboard_dynamics_type ON dashboard_dynamics(type);
CREATE INDEX IF NOT EXISTS idx_dashboard_dynamics_timestamp ON dashboard_dynamics(timestamp);

-- 数据一致性优化
-- 1. 添加外键约束
-- 确保车辆表与订单表的外键约束
ALTER TABLE orders ADD CONSTRAINT IF NOT EXISTS fk_orders_vehicle_id FOREIGN KEY (vehicle_id) REFERENCES vehicles(id) ON DELETE SET NULL;

-- 确保报警表与车辆表的外键约束
ALTER TABLE alerts ADD CONSTRAINT IF NOT EXISTS fk_alerts_vehicle_id FOREIGN KEY (vehicle_id) REFERENCES vehicles(id) ON DELETE SET NULL;

-- 2. 添加触发器确保数据一致性
-- 为 users 表添加更新时间触发器
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 为所有需要更新时间的表添加触发器
DO $$
DECLARE
    table_name TEXT;
BEGIN
    FOR table_name IN ('users', 'departments', 'organizations', 'system_settings', 'organization_settings', 'location_fences', 'location_positions', 'location_places', 'location_routes', 'alerts', 'dashboard_tasks', 'dashboard_dynamics')
    LOOP
        EXECUTE format('CREATE TRIGGER update_%s_updated_at
        BEFORE UPDATE ON %s
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();', table_name, table_name);
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- 3. 插入默认数据
-- 插入默认部门
INSERT INTO departments (name, code, description) VALUES
('技术部', 'tech', '负责系统开发和维护'),
('运营部', 'ops', '负责系统运营和管理'),
('财务部', 'finance', '负责财务管理'),
('市场部', 'market', '负责市场推广')
ON CONFLICT (code) DO NOTHING;

-- 插入默认组织
INSERT INTO organizations (name, code, description) VALUES
('总公司', 'headquarters', '公司总部'),
('分公司', 'branch', '分公司')
ON CONFLICT (code) DO NOTHING;

-- 插入默认系统设置
INSERT INTO system_settings (key, value, description) VALUES
('system_name', '{"value": "CarpTMS"}', '系统名称'),
('system_version', '{"value": "1.0.0"}', '系统版本'),
('max_login_attempts', '{"value": 5}', '最大登录尝试次数'),
('token_expiration', '{"value": 3600}', 'Token过期时间（秒）')
ON CONFLICT (key) DO NOTHING;

-- 插入默认仪表盘数据
INSERT INTO statistics (type, value, unit, category, period) VALUES
('total_vehicles', 100, '辆', '车辆', 'day'),
('total_orders', 50, '单', '订单', 'day'),
('total_distance', 1000, '公里', '运输', 'day'),
('total_income', 50000, '元', '财务', 'day')
ON CONFLICT DO NOTHING;

-- 插入默认仪表盘任务
INSERT INTO dashboard_tasks (title, description, priority, status, due_date) VALUES
('系统维护', '定期系统维护', 'medium', 'pending', NOW() + INTERVAL '7 days'),
('数据备份', '定期数据备份', 'high', 'pending', NOW() + INTERVAL '1 day'),
('系统升级', '系统版本升级', 'medium', 'pending', NOW() + INTERVAL '30 days')
ON CONFLICT DO NOTHING;

-- 插入默认仪表盘动态
INSERT INTO dashboard_dynamics (title, description, type) VALUES
('系统启动', '系统正常启动', 'system'),
('用户登录', '管理员登录系统', 'user'),
('数据同步', '数据同步完成', 'data')
ON CONFLICT DO NOTHING;