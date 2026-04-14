-- 数据库查询性能优化

-- 1. 优化订单查询：添加按时间范围查询的复合索引
CREATE INDEX IF NOT EXISTS idx_orders_create_time 
ON orders(create_time DESC);

CREATE INDEX IF NOT EXISTS idx_orders_vehicle_time 
ON orders(vehicle_id, create_time DESC);

-- 2. 优化物流轨迹查询：添加更全面的复合索引
CREATE INDEX IF NOT EXISTS idx_logistics_tracks_vehicle_time_order 
ON logistics_tracks(vehicle_id, track_time DESC, order_id);

CREATE INDEX IF NOT EXISTS idx_logistics_tracks_order_time 
ON logistics_tracks(order_id, track_time DESC);

-- 3. 优化车辆查询：添加常用筛选条件的复合索引
CREATE INDEX IF NOT EXISTS idx_vehicles_status_type 
ON vehicles(status, vehicle_type);

CREATE INDEX IF NOT EXISTS idx_vehicles_simulation_status 
ON vehicles(is_simulation, status);

-- 4. 优化设备数据查询：添加设备ID和时间的复合索引
CREATE INDEX IF NOT EXISTS idx_device_data_device_time 
ON device_data(device_id, "timestamp" DESC);

-- 5. 优化称重数据查询：添加更全面的复合索引
CREATE INDEX IF NOT EXISTS idx_weighing_data_device_vehicle_time 
ON weighing_data(device_id, vehicle_id, weighing_time DESC);

-- 6. 优化用户查询：添加用户状态索引
CREATE INDEX IF NOT EXISTS idx_users_status 
ON users(status);

-- 7. 优化设备查询：添加设备状态和类型的复合索引
CREATE INDEX IF NOT EXISTS idx_devices_status_type 
ON devices(status, device_type);

-- 8. 优化报表数据查询：添加更全面的复合索引
CREATE INDEX IF NOT EXISTS idx_report_data_template_period_time 
ON report_data(template_id, period_type, period_value, report_time DESC);

-- 9. 运行VACUUM ANALYZE更新统计信息
-- 注意：生产环境中应在低峰期手动运行
-- VACUUM ANALYZE;


