-- 为称重数据表添加索引，优化查询性能

-- 基本查询优化索引
CREATE INDEX IF NOT EXISTS idx_weighing_data_vehicle_id ON weighing_data(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_weighing_data_device_id ON weighing_data(device_id);
CREATE INDEX IF NOT EXISTS idx_weighing_data_status ON weighing_data(status);

-- 时间查询优化索引
CREATE INDEX IF NOT EXISTS idx_weighing_data_weighing_time ON weighing_data(weighing_time);
CREATE INDEX IF NOT EXISTS idx_weighing_data_create_time ON weighing_data(create_time);
CREATE INDEX IF NOT EXISTS idx_weighing_data_update_time ON weighing_data(update_time);

-- 组合索引，优化常见查询场景
CREATE INDEX IF NOT EXISTS idx_weighing_data_vehicle_id_time ON weighing_data(vehicle_id, weighing_time DESC);
CREATE INDEX IF NOT EXISTS idx_weighing_data_device_id_time ON weighing_data(device_id, weighing_time DESC);
CREATE INDEX IF NOT EXISTS idx_weighing_data_status_time ON weighing_data(status, weighing_time DESC);
CREATE INDEX IF NOT EXISTS idx_weighing_data_site_id_time ON weighing_data(site_id, weighing_time DESC);

-- 针对排序和统计查询的索引
CREATE INDEX IF NOT EXISTS idx_weighing_data_weighing_time_desc ON weighing_data(weighing_time DESC);

-- 针对多条件查询的组合索引
CREATE INDEX IF NOT EXISTS idx_weighing_data_lane_no_time ON weighing_data(lane_no, weighing_time DESC);
CREATE INDEX IF NOT EXISTS idx_weighing_data_axle_count_time ON weighing_data(axle_count, weighing_time DESC);


