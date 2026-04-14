-- 数据库验证脚本
-- 用途: 验证数据库表结构和数据完整性

-- 设置输出格式
\x on
\echo

-- 1. 检查所有表
echo '========== 1. 数据库表列表 =========='
\dt

-- 2. 检查vehicles表结构
echo ''
echo '========== 2. Vehicles表结构 =========='
\d vehicles

-- 3. 统计vehicles表字段数量
echo ''
echo '========== 3. Vehicles表字段统计 =========='
SELECT
    COUNT(*) as total_fields,
    COUNT(*) FILTER (WHERE data_type LIKE 'varchar%') as varchar_fields,
    COUNT(*) FILTER (WHERE data_type LIKE 'integer%') as integer_fields,
    COUNT(*) FILTER (WHERE data_type LIKE 'timestamp%') as timestamp_fields,
    COUNT(*) FILTER (WHERE data_type LIKE 'numeric%') as numeric_fields,
    COUNT(*) FILTER (WHERE is_nullable = 'YES') as nullable_fields,
    COUNT(*) FILTER (WHERE is_nullable = 'NO') as required_fields
FROM information_schema.columns
WHERE table_name = 'vehicles'
  AND table_schema = 'public';

-- 4. 检查关键约束
echo ''
echo '========== 4. Vehicles表约束 =========='
SELECT
    constraint_name,
    constraint_type
FROM information_schema.table_constraints
WHERE table_name = 'vehicles'
  AND table_schema = 'public';

-- 5. 检查关键索引
echo ''
echo '========== 5. Vehicles表索引 =========='
SELECT
    indexname,
    indexdef
FROM pg_indexes
WHERE tablename = 'vehicles'
  AND schemaname = 'public';

-- 6. 检查数据量
echo ''
echo '========== 6. 数据统计 =========='
SELECT
    'veh_groups' as table_name,
    COUNT(*) as row_count
FROM veh_groups
UNION ALL
SELECT
    'vehicles' as table_name,
    COUNT(*) as row_count
FROM vehicles
UNION ALL
SELECT
    'user_groups' as table_name,
    COUNT(*) as row_count
FROM user_groups
UNION ALL
SELECT
    'users' as table_name,
    COUNT(*) as row_count
FROM users
UNION ALL
SELECT
    'weighing_data' as table_name,
    COUNT(*) as row_count
FROM weighing_data;

-- 7. 检查示例数据
echo ''
echo '========== 7. 示例车辆数据 =========='
\x off
SELECT
    vehicle_id,
    vehicle_name,
    license_plate,
    vehicle_type,
    vehicle_color,
    device_id,
    group_id,
    status,
    is_simulation
FROM vehicles
LIMIT 5;

-- 8. 检查字段完整性（是否所有必要字段都存在）
echo ''
\x on
echo '========== 8. 关键字段检查 =========='
WITH required_fields AS (
    SELECT unnest(ARRAY[
        'vehicle_id', 'vehicle_name', 'license_plate', 'vehicle_type',
        'vehicle_color', 'vehicle_brand', 'vehicle_model', 'engine_no',
        'frame_no', 'register_date', 'inspection_date', 'insurance_date',
        'seating_capacity', 'load_capacity', 'vehicle_length',
        'vehicle_width', 'vehicle_height', 'device_id', 'terminal_type',
        'communication_type', 'sim_card_no', 'install_date', 'install_address',
        'install_technician', 'own_no', 'own_name', 'own_phone',
        'own_id_card', 'own_address', 'own_email', 'group_id',
        'operation_status', 'operation_route', 'operation_area',
        'operation_company', 'driver_name', 'driver_phone',
        'driver_license_no', 'purchase_price', 'annual_fee',
        'insurance_fee', 'is_simulation', 'simulation_source',
        'remark', 'status', 'create_time', 'update_time',
        'create_user_id', 'update_user_id'
    ]) AS field_name
)
SELECT
    rf.field_name,
    CASE
        WHEN c.column_name IS NOT NULL THEN '✓ EXISTS'
        ELSE '✗ MISSING'
    END AS status
FROM required_fields rf
LEFT JOIN information_schema.columns c
    ON c.column_name = rf.field_name
    AND c.table_name = 'vehicles'
    AND c.table_schema = 'public'
ORDER BY rf.field_name;

-- 9. 数据库版本信息
echo ''
echo '========== 9. 数据库版本信息 =========='
\x off
SELECT
    version() AS postgresql_version,
    current_database() AS current_database,
    current_user AS current_user,
    NOW() AS check_time;

-- 完成
echo ''
\x on
echo '========== 验证完成 =========='
echo ''
echo '如果所有关键字段都显示 "✓ EXISTS"，则数据库结构正确！'


