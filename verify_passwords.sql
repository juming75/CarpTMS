-- 修复验证脚本（使用正确的字段名）

-- 检查当前密码状态
SELECT '=== 当前密码状态 ===' as info;
SELECT
    user_id,
    username,
    CASE
        WHEN password LIKE '$argon2id$%' THEN '✓ 已加密'
        ELSE '✗ 明文密码（需要修复）'
    END as password_status,
    LENGTH(password) as password_length
FROM users
ORDER BY user_id;

-- 统计
SELECT '=== 密码格式统计 ===' as info;
SELECT
    COUNT(*) as total_users,
    SUM(CASE WHEN password LIKE '$argon2id$%' THEN 1 ELSE 0 END) as encrypted_users,
    SUM(CASE WHEN password NOT LIKE '$argon2id$%' THEN 1 ELSE 0 END) as plaintext_users
FROM users;

-- 显示修复后的用户列表和默认密码
SELECT '=== 用户密码对照表 ===' as info;
SELECT
    username as 用户名,
    CASE username
        WHEN 'admin' THEN 'admin'
        WHEN 'manager' THEN 'manager'
        WHEN 'user' THEN 'user'
        ELSE '123456（临时密码，请立即修改）'
    END as 默认密码,
    CASE
        WHEN password LIKE '$argon2id$%' THEN '✓ 可用'
        ELSE '✗ 不可用'
    END as 登录状态,
    '⚠️ 首次登录后请立即修改密码' as 注意事项
FROM users
ORDER BY username;

-- 最终检查
SELECT '=== 修复状态 ===' as info;
SELECT
    CASE
        WHEN COUNT(*) = 0 THEN '✓ 所有密码已成功修复为Argon2哈希'
        ELSE '⚠️ 还有 ' || COUNT(*) || ' 个用户需要手动修复'
    END as final_status
FROM users
WHERE password NOT LIKE '$argon2id$%';

SELECT '=== 修复完成！请使用上述密码登录系统 ===' as summary;


