-- ====================================================================
-- CarpTMS 用户表字段名标准化迁移
-- 将字段名统一为 user_id 和 user_name
-- ====================================================================

-- 步骤1: 创建备份表
SELECT '=== 步骤1: 创建备份表 ===' as info;
DROP TABLE IF EXISTS users_backup_before_migration_20260317;
CREATE TABLE users_backup_before_migration_20260317 AS
SELECT * FROM users;
SELECT '✓ 已创建备份表: users_backup_before_migration_20260317' as status;

-- 步骤2: 查看当前字段
SELECT '=== 步骤2: 当前字段结构 ===' as info;
SELECT column_name, data_type, character_maximum_length
FROM information_schema.columns
WHERE table_name = 'users'
ORDER BY ordinal_position;

-- 步骤3: 重命名字段
SELECT '=== 步骤3: 重命名字段 ===' as info;

-- 添加新字段
ALTER TABLE users ADD COLUMN IF NOT EXISTS user_id_new INTEGER;
ALTER TABLE users ADD COLUMN IF NOT EXISTS user_name_new VARCHAR(50);

-- 复制数据
UPDATE users SET user_id_new = id;
UPDATE users SET user_name_new = username;

-- 删除旧字段（添加新字段后再删除）
ALTER TABLE users RENAME COLUMN id TO id_old;
ALTER TABLE users RENAME COLUMN username TO username_old;
ALTER TABLE users RENAME COLUMN user_id_new TO id;
ALTER TABLE users RENAME COLUMN user_name_new TO username;

-- 删除旧字段（可选，建议测试确认后再执行）
-- ALTER TABLE users DROP COLUMN IF EXISTS id_old;
-- ALTER TABLE users DROP COLUMN IF EXISTS username_old;

SELECT '✓ 字段重命名完成' as status;

-- 步骤4: 更新约束和索引
SELECT '=== 步骤4: 更新约束 ===' as info;

-- 重新设置主键
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_pkey;
ALTER TABLE users ADD PRIMARY KEY (id);

-- 更新外键约束（如果存在）
-- ALTER TABLE other_table DROP CONSTRAINT IF EXISTS other_table_user_id_fkey;
-- ALTER TABLE other_table ADD CONSTRAINT other_table_user_id_fkey
--     FOREIGN KEY (user_id) REFERENCES users(id);

SELECT '✓ 约束更新完成' as status;

-- 步骤5: 验证迁移结果
SELECT '=== 步骤5: 验证迁移结果 ===' as info;

SELECT id, username, password, user_group_id, status
FROM users
LIMIT 5;

-- 步骤6: 显示迁移统计
SELECT '=== 步骤6: 迁移统计 ===' as info;
SELECT
    COUNT(*) as total_users,
    COUNT(CASE WHEN id IS NOT NULL THEN 1 END) as has_id,
    COUNT(CASE WHEN username IS NOT NULL THEN 1 END) as has_username,
    COUNT(CASE WHEN password IS NOT NULL THEN 1 END) as has_password
FROM users;

SELECT '=== 迁移完成！请验证数据正确性 ===' as summary;

-- 步骤7: 回滚脚本（如需回滚，请执行以下语句）
/*
-- 回滚脚本
BEGIN;

-- 恢复旧字段名
ALTER TABLE users RENAME COLUMN id TO id_temp;
ALTER TABLE users RENAME COLUMN username TO username_temp;
ALTER TABLE users RENAME COLUMN id_old TO id;
ALTER TABLE users RENAME COLUMN username_old TO username;

-- 删除临时字段
ALTER TABLE users DROP COLUMN IF EXISTS id_temp;
ALTER TABLE users DROP COLUMN IF EXISTS username_temp;

-- 恢复主键
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_pkey;
ALTER TABLE users ADD PRIMARY KEY (id);

COMMIT;
*/

-- 步骤8: 清理脚本（确认迁移成功后执行）
/*
-- 清理脚本：删除旧字段
BEGIN;

ALTER TABLE users DROP COLUMN IF EXISTS id_old;
ALTER TABLE users DROP COLUMN IF EXISTS username_old;

COMMIT;

SELECT '✓ 旧字段已清理' as status;
*/


