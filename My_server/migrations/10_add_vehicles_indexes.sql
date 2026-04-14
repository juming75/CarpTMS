-- 为车辆表添加索引，优化查询性能

-- 基本查询优化索引
CREATE INDEX IF NOT EXISTS idx_vehicles_status ON vehicles(status);
CREATE INDEX IF NOT EXISTS idx_vehicles_device_id ON vehicles(device_id);
CREATE INDEX IF NOT EXISTS idx_vehicles_vehicle_type ON vehicles(vehicle_type);
CREATE INDEX IF NOT EXISTS idx_vehicles_group_id ON vehicles(group_id);

-- 时间查询优化索引
CREATE INDEX IF NOT EXISTS idx_vehicles_create_time ON vehicles(create_time);
CREATE INDEX IF NOT EXISTS idx_vehicles_update_time ON vehicles(update_time);

-- 组合索引，优化常见查询场景
CREATE INDEX IF NOT EXISTS idx_vehicles_status_device_id ON vehicles(status, device_id);
CREATE INDEX IF NOT EXISTS idx_vehicles_group_id_status ON vehicles(group_id, status);

-- 针对排序和时间范围查询的索引
CREATE INDEX IF NOT EXISTS idx_vehicles_create_time_desc ON vehicles(create_time DESC);
CREATE INDEX IF NOT EXISTS idx_vehicles_update_time_desc ON vehicles(update_time DESC);

-- 针对统计查询的索引
CREATE INDEX IF NOT EXISTS idx_vehicles_operation_status ON vehicles(operation_status);
CREATE INDEX IF NOT EXISTS idx_vehicles_is_simulation ON vehicles(is_simulation);


