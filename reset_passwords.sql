-- 用户密码重置脚本
-- 用于将数据库中的明文密码重置为Argon2哈希格式

-- 注意：这个脚本需要先生成Argon2哈希值
-- 建议使用后端的hash_password函数生成

-- 方法1: 直接更新特定用户的密码（需要手动生成哈希）
-- 生成密码哈希的步骤：
-- 1. 在Rust代码中调用：hash_password("your_new_password")
-- 2. 将生成的哈希值替换下面的占位符
-- 3. 执行此SQL脚本

-- 示例：重置管理员密码为 admin123
-- 下面的哈希值对应密码 "admin123"
-- 如果需要不同的密码，请使用Rust代码生成新的哈希

-- 查询当前用户密码格式
SELECT
    user_id,
    user_name,
    CASE
        WHEN password LIKE '$argon2id$%' THEN '已加密'
        ELSE '明文密码（需要修复）'
    END as password_status,
    LENGTH(password) as password_length
FROM users;

-- 显示需要更新的用户（明文密码的用户）
-- 用户组状态检查
WITH user_status AS (
    SELECT
        u.user_id,
        u.user_name,
        u.password,
        CASE
            WHEN u.password LIKE '$argon2id$%' THEN 'encrypted'
            ELSE 'plaintext'
        END as password_type,
        ug.group_name,
        ug.group_id,
        u.user_group_id
    FROM users u
    LEFT JOIN user_groups ug ON u.user_group_id = ug.group_id
)
SELECT * FROM user_status
WHERE password_type = 'plaintext';

-- 批量重置所有明文密码用户为默认密码
-- 默认密码：admin123（Argon2哈希）
-- 注意：这是临时的，用户登录后应该立即修改密码

-- 取消下面的注释以执行密码重置
-- UPDATE users
-- SET password = '$argon2id$v=19$m=19456,t=2,p=1$[生成的哈希值]',
--     update_time = CURRENT_TIMESTAMP
-- WHERE password NOT LIKE '$argon2id$%';

-- 验证密码是否已更新
-- SELECT user_id, user_name, password FROM users;

-- 创建密码重置历史表（可选）
CREATE TABLE IF NOT EXISTS password_reset_history (
    reset_id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(user_id),
    old_password VARCHAR(255),
    new_password VARCHAR(255) NOT NULL,
    reset_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reset_method VARCHAR(50) NOT NULL DEFAULT 'admin_script',
    reset_by VARCHAR(50) DEFAULT 'system'
);

-- 记录密码重置（示例）
-- INSERT INTO password_reset_history (user_id, old_password, new_password, reset_method)
-- SELECT user_id, password, new_password, 'admin_script'
-- FROM (
--     SELECT user_id, password, '$argon2id$...' as new_password
--     FROM users
--     WHERE password NOT LIKE '$argon2id$%'
-- ) updates;

-- 查看重置历史
-- SELECT * FROM password_reset_history ORDER BY reset_time DESC;

-- ============================================
-- 推荐做法：为每个用户生成唯一密码
-- ============================================

-- 为每个用户生成临时密码并记录
-- 临时密码格式: 用户名 + "2026!"
-- 例如: admin2026!, manager2026!

-- 需要先生成临时密码的哈希值，然后执行：

-- 为特定用户重置密码（示例）
-- UPDATE users
-- SET password = '对应密码的哈希值',
--     update_time = CURRENT_TIMESTAMP
-- WHERE user_name = 'admin';

-- 验证特定用户的密码格式
SELECT
    user_name,
    CASE
        WHEN password LIKE '$argon2id$%' THEN '✓ 加密'
        ELSE '✗ 明文'
    END as status
FROM users
ORDER BY user_id;

-- ============================================
-- 安全建议
-- ============================================

-- 1. 强制所有用户首次登录后修改密码
-- 2. 实施密码复杂度策略（最小8位，包含大小写字母、数字、特殊字符）
-- 3. 定期轮换密码（如90天）
-- 4. 实施密码历史记录（禁止使用最近5次密码）
-- 5. 记录所有密码重置操作

-- ============================================
-- 快速检查和修复
-- ============================================

-- 检查数据库连接
SELECT current_database(), current_user, version();

-- 检查用户表结构
\d users

-- 检查用户组表结构
\d user_groups

-- 检查索引
SELECT indexname, tablename FROM pg_indexes
WHERE tablename IN ('users', 'user_groups')
ORDER BY tablename, indexname;

-- 统计用户信息
SELECT
    ug.group_name,
    COUNT(u.user_id) as user_count,
    COUNT(CASE WHEN u.password LIKE '$argon2id$%' THEN 1 END) as encrypted_count,
    COUNT(CASE WHEN u.password NOT LIKE '$argon2id$%' THEN 1 END) as plaintext_count
FROM user_groups ug
LEFT JOIN users u ON ug.group_id = u.user_group_id
GROUP BY ug.group_id
ORDER BY ug.group_id;


