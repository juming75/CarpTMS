-- 创建部门表
CREATE TABLE IF NOT EXISTS departments (
    department_id SERIAL PRIMARY KEY,
    department_name VARCHAR(100) NOT NULL,
    parent_department_id INTEGER REFERENCES departments(department_id),
    manager_id INTEGER REFERENCES users(user_id),
    phone VARCHAR(20),
    description TEXT,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

-- 插入默认部门数据
INSERT INTO departments (department_name, description) 
VALUES 
    ('总公司', '总公司'),
    ('技术部', '技术部门'),
    ('市场部', '市场部门'),
    ('财务部', '财务部门')
ON CONFLICT DO NOTHING;


