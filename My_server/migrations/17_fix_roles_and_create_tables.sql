-- 修复user_groups表结构以匹配roles.rs
-- 添加role_id和role_name字段的别名
ALTER TABLE user_groups 
ADD COLUMN IF NOT EXISTS role_id INTEGER GENERATED ALWAYS AS (group_id) STORED,
ADD COLUMN IF NOT EXISTS role_name VARCHAR(100) GENERATED ALWAYS AS (group_name) STORED;

-- 确保vehicle_groups表有完整的结构
ALTER TABLE vehicle_groups 
ADD COLUMN IF NOT EXISTS create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN IF NOT EXISTS update_time TIMESTAMP;

-- 确保departments表有完整的结构
ALTER TABLE departments 
ADD COLUMN IF NOT EXISTS create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
ADD COLUMN IF NOT EXISTS update_time TIMESTAMP;

-- 插入默认角色数据（如果不存在）
INSERT INTO user_groups (group_name, description) 
VALUES 
    ('管理员', '系统管理员角色'),
    ('普通用户', '普通用户角色'),
    ('经理', '部门经理角色')
ON CONFLICT DO NOTHING;

-- 插入默认车辆分组数据（如果不存在）
INSERT INTO vehicle_groups (group_name, description) 
VALUES 
    ('默认车队', '系统默认车队'),
    ('第一车队', '第一运输车队'),
    ('第二车队', '第二运输车队')
ON CONFLICT DO NOTHING;

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_user_groups_role_name ON user_groups(role_name);
CREATE INDEX IF NOT EXISTS idx_vehicle_groups_group_name ON vehicle_groups(group_name);
CREATE INDEX IF NOT EXISTS idx_departments_department_name ON departments(department_name);


