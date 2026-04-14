-- 优化查询性能的索引

-- 为车辆表添加索引，优化模糊搜索和组合查询

-- 优化车辆名称和车牌号的模糊搜索
CREATE INDEX IF NOT EXISTS idx_vehicles_vehicle_name ON vehicles(vehicle_name);
CREATE INDEX IF NOT EXISTS idx_vehicles_license_plate ON vehicles(license_plate);

-- 优化常见的组合查询场景
CREATE INDEX IF NOT EXISTS idx_vehicles_vehicle_type_status ON vehicles(vehicle_type, status);
CREATE INDEX IF NOT EXISTS idx_vehicles_status_is_simulation ON vehicles(status, is_simulation);
CREATE INDEX IF NOT EXISTS idx_vehicles_vehicle_name_license_plate ON vehicles(vehicle_name, license_plate);

-- 优化按vehicle_id排序的查询
CREATE INDEX IF NOT EXISTS idx_vehicles_vehicle_id_desc ON vehicles(vehicle_id DESC);

-- 为称重数据表添加索引，优化关联查询

-- 优化与车辆表的关联查询
CREATE INDEX IF NOT EXISTS idx_weighing_data_vehicle_id_weighing_time ON weighing_data(vehicle_id, weighing_time DESC);

-- 优化时间范围查询
CREATE INDEX IF NOT EXISTS idx_weighing_data_weighing_time_range ON weighing_data(weighing_time);

-- 优化多条件组合查询
CREATE INDEX IF NOT EXISTS idx_weighing_data_vehicle_id_time_status ON weighing_data(vehicle_id, weighing_time DESC, status);
CREATE INDEX IF NOT EXISTS idx_weighing_data_site_id_vehicle_id_time ON weighing_data(site_id, vehicle_id, weighing_time DESC);

-- 为用户表添加索引，优化认证和查询
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active);

-- 为订单表添加索引，优化查询
CREATE INDEX IF NOT EXISTS idx_orders_order_id ON orders(order_id);
CREATE INDEX IF NOT EXISTS idx_orders_customer_id ON orders(customer_id);
CREATE INDEX IF NOT EXISTS idx_orders_order_status ON orders(order_status);
CREATE INDEX IF NOT EXISTS idx_orders_create_time ON orders(create_time);
CREATE INDEX IF NOT EXISTS idx_orders_customer_id_status ON orders(customer_id, order_status);

-- 为设备表添加索引，优化查询
CREATE INDEX IF NOT EXISTS idx_devices_device_id ON devices(device_id);
CREATE INDEX IF NOT EXISTS idx_devices_device_type ON devices(device_type);
CREATE INDEX IF NOT EXISTS idx_devices_status ON devices(status);
CREATE INDEX IF NOT EXISTS idx_devices_vehicle_id ON devices(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_devices_device_type_status ON devices(device_type, status);


