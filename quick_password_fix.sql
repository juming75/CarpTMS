-- 快速密码检查和修复脚本

-- 第一步：检查当前密码状态
SELECT '=== 第一步：检查当前密码状态 ===' as info;
SELECT
    user_id,
    user_name,
    CASE
        WHEN password LIKE '$argon2id$%' THEN '✓ 已加密'
        ELSE '✗ 明文密码（需要修复）'
    END as password_status,
    LENGTH(password) as password_length
FROM users
ORDER BY user_id;

-- 第二步：统计
SELECT '=== 第二步：统计密码格式 ===' as info;
SELECT
    COUNT(*) as total_users,
    SUM(CASE WHEN password LIKE '$argon2id$%' THEN 1 ELSE 0 END) as encrypted_users,
    SUM(CASE WHEN password NOT LIKE '$argon2id$%' THEN 1 ELSE 0 END) as plaintext_users
FROM users;

-- 第三步：创建备份
SELECT '=== 第三步：创建备份 ===' as info;
DROP TABLE IF EXISTS users_backup_20260317;
CREATE TABLE users_backup_20260317 AS SELECT * FROM users;
SELECT '✓ 已创建备份表 users_backup_20260317' as status;

-- 第四步：更新明文密码
SELECT '=== 第四步：更新明文密码为Argon2哈希 ===' as info;

-- 更新 admin
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$Ne5P0SURb5RPBbGtEc6opw$fVIF2d6MrHLXn4t71m3KitDin3/8YTJ7ZZsYvEEXw7w',
    update_time = CURRENT_TIMESTAMP
WHERE user_name = 'admin' AND password NOT LIKE '$argon2id$%';
SELECT '✓ 已更新 admin 密码（默认密码: admin）' as status;

-- 更新 manager
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$ZQkx7iF1Mj76x0ybhCoOBQ$X51qF28Nx+V+BIGE0YyIeX7MtHi/SCIFs8meEtidkYI',
    update_time = CURRENT_TIMESTAMP
WHERE user_name = 'manager' AND password NOT LIKE '$argon2id$%';
SELECT '✓ 已更新 manager 密码（默认密码: manager）' as status;

-- 更新 user
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$Akn4MJE5HDESPtRBAW4Mwg$9WaxjlOxWFpsUpeSFu88zc2Blrye8oNh9IZ/AXveIpY',
    update_time = CURRENT_TIMESTAMP
WHERE user_name = 'user' AND password NOT LIKE '$argon2id$%';
SELECT '✓ 已更新 user 密码（默认密码: user）' as status;

-- 更新所有其他明文密码用户为临时密码 123456
UPDATE users
SET password = '$argon2id$v=19$m=19456,t=2,p=1$qMX22dfzWiTBmJrESrfE2w$Oc3yU3E8Nfn6sTOltygEhB1NJAKSuUjlq0STKWzhPmY',
    update_time = CURRENT_TIMESTAMP
WHERE password NOT LIKE '$argon2id$%';
SELECT '✓ 已批量更新其他所有明文密码用户（临时密码: 123456）' as status;

-- 第五步：验证修复结果
SELECT '=== 第五步：验证修复结果 ===' as info;
SELECT
    user_name,
    CASE
        WHEN password LIKE '$argon2id$%' THEN '✓ 已修复'
        ELSE '✗ 仍然需要修复'
    END as status,
    LENGTH(password) as hash_length
FROM users
ORDER BY user_name;

-- 第六步：检查是否还有明文密码
SELECT '=== 第六步：最终检查 ===' as info;
SELECT
    CASE
        WHEN COUNT(*) = 0 THEN '✓ 所有密码已成功修复'
        ELSE '⚠️ 还有 ' || COUNT(*) || ' 个用户需要手动修复'
    END as final_status
FROM users
WHERE password NOT LIKE '$argon2id$%';

-- 第七步：显示默认密码对照表
SELECT '=== 第七步：默认密码对照表 ===' as info;
SELECT
    user_name as 用户名,
    CASE user_name
        WHEN 'admin' THEN 'admin'
        WHEN 'manager' THEN 'manager'
        WHEN 'user' THEN 'user'
        ELSE '123456（临时密码，请立即修改）'
    END as 默认密码,
    '⚠️ 首次登录后请立即修改密码' as 注意事项
FROM users
WHERE password LIKE '$argon2id$%'
ORDER BY user_name;

SELECT '=== 修复完成！请使用上述密码登录系统 ===' as summary;


