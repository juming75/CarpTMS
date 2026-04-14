-- 为 users 表添加部门 ID 字段
ALTER TABLE users 
ADD COLUMN IF NOT EXISTS department_id INTEGER,
ADD CONSTRAINT fk_users_department 
    FOREIGN KEY (department_id) 
    REFERENCES departments(department_id) 
    ON DELETE SET NULL;

-- 添加索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_users_department_id ON users(department_id);

-- 添加注释
COMMENT ON COLUMN users.department_id IS '用户所属部门 ID';


