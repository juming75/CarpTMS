-- 创建组织单位表
CREATE TABLE IF NOT EXISTS organizations (
    unit_id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    type VARCHAR(50) NOT NULL,
    parent_id INTEGER,
    description TEXT,
    contact_person VARCHAR(100),
    contact_phone VARCHAR(20),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_organizations_name ON organizations(name);
CREATE INDEX IF NOT EXISTS idx_organizations_type ON organizations(type);
CREATE INDEX IF NOT EXISTS idx_organizations_status ON organizations(status);

-- 插入默认数据
INSERT INTO organizations (unit_id, name, type, parent_id, description, contact_person, contact_phone, status) 
VALUES 
    ('ORG001', '总公司', 'enterprise', NULL, '公司总部', '张总', '13800138000', 'active'),
    ('ORG002', '技术部', 'enterprise', NULL, '技术部门', '李经理', '13900139000', 'active'),
    ('ORG003', '市场部', 'enterprise', NULL, '市场部门', '王经理', '13700137000', 'active'),
    ('ORG004', '财务部', 'enterprise', NULL, '财务部门', '赵经理', '13600136000', 'active')
ON CONFLICT DO NOTHING;


