-- ============================================
-- B端用户密码修复脚本（P0优先级）
-- 自动更新所有明文密码为Argon2哈希
-- ============================================

-- 说明：
-- 1. 执行前请先备份数据库
-- 2. 此脚本会更新所有使用明文密码的用户
-- 3. 默认密码策略：用户名 + "123"
-- 4. 用户首次登录后应立即修改密码

-- ============================================
-- 第一步：检查当前密码状态
-- ============================================

-- 显示所有用户的密码格式
SELECT
    user_id,
    user_name,
    ug.group_name,
    CASE
        WHEN u.password LIKE '$argon2id$%' THEN '✓ 已加密'
        ELSE '✗ 明文密码（需要修复）'
    END as password_status,
    LENGTH(u.password) as password_length,
    u.update_time
FROM users u
LEFT JOIN user_groups ug ON u.user_group_id = ug.group_id
ORDER BY u.user_id;

-- 统计密码格式
SELECT
    ug.group_name,
    COUNT(u.user_id) as total_users,
    SUM(CASE WHEN u.password LIKE '$argon2id$%' THEN 1 ELSE 0 END) as encrypted_count,
    SUM(CASE WHEN u.password NOT LIKE '$argon2id$%' THEN 1 ELSE 0 END) as plaintext_count
FROM user_groups ug
LEFT JOIN users u ON ug.group_id = u.user_group_id
GROUP BY ug.group_name
ORDER BY ug.group_name;

-- ============================================
-- 第二步：备份数据
-- ============================================

-- 创建备份表
DROP TABLE IF EXISTS users_backup_p0_fix;

CREATE TABLE users_backup_p0_fix AS
SELECT * FROM users;

SELECT '✓ 已创建备份表 users_backup_p0_fix' as status;

-- ============================================
-- 第三步：更新明文密码为Argon2哈希
-- ============================================

-- 更新默认管理员账户
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$Ne5P0SURb5RPBbGtEc6opw$fVIF2d6MrHLXn4t71m3KitDin3/8YTJ7ZZsYvEEXw7w',
    update_time = CURRENT_TIMESTAMP
WHERE user_name = 'admin' AND password NOT LIKE '$argon2id$%';

SELECT '✓ 已更新 admin 密码（密码: admin）' as status;

-- 更新管理员账户（admin123）
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$/hW8VoNhe8s+ug6LdV4cOA$bLGzC21Cay1+BTdctMPtMRPwWzCyD4CPsxms2Z9RGcM',
    update_time = CURRENT_TIMESTAMP
WHERE user_name = 'admin123' AND password NOT LIKE '$argon2id$%';

-- 更新经理账户
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$ZQkx7iF1Mj76x0ybhCoOBQ$X51qF28Nx+V+BIGE0YyIeX7MtHi/SCIFs8meEtidkYI',
    update_time = CURRENT_TIMESTAMP
WHERE user_name = 'manager' AND password NOT LIKE '$argon2id$%';

SELECT '✓ 已更新 manager 密码（密码: manager）' as status;

-- 更新经理账户（manager123）
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$NKpw/QJ26zAxe9BsLy1Jyg$HFPOUR/uj/2b6+RzcgF6Nd8RSkWdvFw82tNEwSDMUIM',
    update_time = CURRENT_TIMESTAMP
WHERE user_name = 'manager123' AND password NOT LIKE '$argon2id$%';

-- 更新普通用户账户（通用账户）
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$Akn4MJE5HDESPtRBAW4Mwg$9WaxjlOxWFpsUpeSFu88zc2Blrye8oNh9IZ/AXveIpY',
    update_time = CURRENT_TIMESTAMP
WHERE user_name = 'user' AND password NOT LIKE '$argon2id$%';

SELECT '✓ 已更新 user 密码（密码: user）' as status;

-- 批量更新其他所有明文密码用户为默认密码
-- 默认密码：123456（临时密码，用户登录后必须修改）
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$qMX22dfzWiTBmJrESrfE2w$Oc3yU3E8Nfn6sTOltygEhB1NJAKSuUjlq0STKWzhPmY',
    update_time = CURRENT_TIMESTAMP
WHERE password NOT LIKE '$argon2id$%';

SELECT '✓ 已批量更新其他所有明文密码用户（密码: 123456）' as status;

-- ============================================
-- 第四步：验证更新结果
-- ============================================

-- 检查是否还有明文密码用户
SELECT
    COUNT(*) as remaining_plaintext_users
FROM users
WHERE password NOT LIKE '$argon2id$%';

-- 如果结果为0，说明所有密码已修复
-- 如果结果>0，需要手动检查未更新的用户

-- 显示更新后的所有用户
SELECT
    user_id,
    user_name,
    ug.group_name,
    CASE
        WHEN u.password LIKE '$argon2id$%' THEN '✓ 已加密'
        ELSE '✗ 仍然是明文'
    END as password_status,
    LENGTH(u.password) as password_length,
    u.update_time
FROM users u
LEFT JOIN user_groups ug ON u.user_group_id = ug.group_id
ORDER BY u.user_id;

-- ============================================
-- 第五步：密码对照表（供用户参考）
-- ============================================

-- 显示用户名和对应的新密码
SELECT
    u.user_name as 用户名,
    ug.group_name as 用户组,
    CASE u.user_name
        WHEN 'admin' THEN 'admin'
        WHEN 'manager' THEN 'manager'
        WHEN 'user' THEN 'user'
        ELSE '123456（临时密码）'
    END as 默认密码,
    '⚠️ 首次登录后请立即修改密码' as 注意事项
FROM users u
LEFT JOIN user_groups ug ON u.user_group_id = ug.group_id
WHERE u.password LIKE '$argon2id$%'
ORDER BY u.user_name;

-- ============================================
-- 第六步：创建密码重置历史记录
-- ============================================

-- 创建历史记录表
DROP TABLE IF EXISTS password_reset_history_p0;

CREATE TABLE password_reset_history_p0 (
    reset_id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(user_id),
    user_name VARCHAR(50),
    old_password VARCHAR(255),
    reset_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reset_method VARCHAR(50) NOT NULL DEFAULT 'p0_fix_script',
    reset_by VARCHAR(50) DEFAULT 'system'
);

-- 记录密码重置（从备份表获取）
INSERT INTO password_reset_history_p0 (user_id, user_name, old_password)
SELECT
    b.user_id,
    b.user_name,
    b.password
FROM users_backup_p0_fix b
LEFT JOIN users u ON b.user_id = u.user_id
WHERE u.password != b.password OR b.password NOT LIKE '$argon2id$%';

SELECT '✓ 已创建密码重置历史记录' as status;

-- ============================================
-- 第七步：清理和总结
-- ============================================

-- 统计信息
SELECT
    '修复完成' as status,
    (SELECT COUNT(*) FROM users) as total_users,
    (SELECT COUNT(*) FROM users WHERE password LIKE '$argon2id$%') as encrypted_users,
    (SELECT COUNT(*) FROM password_reset_history_p0) as reset_count;

-- 显示备份表信息
SELECT
    'users_backup_p0_fix' as 备份表名,
    (SELECT COUNT(*) FROM users_backup_p0_fix) as 备份记录数;

-- 显示恢复命令（如果需要回滚）
SELECT
    '如需回滚，请执行以下命令:' as 提示;

SELECT
    'DROP TABLE users;' as step1,
    'ALTER TABLE users_backup_p0_fix RENAME TO users;' as step2,
    'DROP TABLE password_reset_history_p0;' as step3;

-- ============================================
-- 使用说明
-- ============================================

-- 1. 首次登录使用以下密码：
--    admin -> admin
--    manager -> manager
--    user -> user
--    其他用户 -> 123456（临时密码）
--
-- 2. 首次登录后必须立即修改密码
--
-- 3. 如遇到问题，可以查看备份表：
--    SELECT * FROM users_backup_p0_fix;
--
-- 4. 如需回滚，执行恢复命令
--
-- 5. 建议定期清理备份表：
--    DROP TABLE users_backup_p0_fix;
--    DROP TABLE password_reset_history_p0;

-- ============================================
-- 脚本执行完毕
-- ============================================


