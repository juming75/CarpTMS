-- 添加订单表索引
CREATE INDEX IF NOT EXISTS idx_orders_order_id ON orders(order_id);
CREATE INDEX IF NOT EXISTS idx_orders_order_no ON orders(order_no);
CREATE INDEX IF NOT EXISTS idx_orders_vehicle_id ON orders(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_orders_customer_name ON orders(customer_name);
CREATE INDEX IF NOT EXISTS idx_orders_order_status ON orders(order_status);
CREATE INDEX IF NOT EXISTS idx_orders_create_time ON orders(create_time);

-- 添加订单项索引
CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_item_id ON order_items(item_id);

-- 添加用户表索引
CREATE INDEX IF NOT EXISTS idx_users_user_id ON users(user_id);
CREATE INDEX IF NOT EXISTS idx_users_user_name ON users(user_name);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- 添加车辆表索引
CREATE INDEX IF NOT EXISTS idx_vehicles_vehicle_id ON vehicles(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_vehicles_license_plate ON vehicles(license_plate);
CREATE INDEX IF NOT EXISTS idx_vehicles_vehicle_name ON vehicles(vehicle_name);

-- 添加设备表索引
CREATE INDEX IF NOT EXISTS idx_devices_device_id ON devices(device_id);
CREATE INDEX IF NOT EXISTS idx_devices_device_type ON devices(device_type);

-- 添加称重数据表索引
CREATE INDEX IF NOT EXISTS idx_weighing_data_id ON weighing_data(id);
CREATE INDEX IF NOT EXISTS idx_weighing_data_vehicle_id ON weighing_data(vehicle_id);
CREATE INDEX IF NOT EXISTS idx_weighing_data_create_time ON weighing_data(create_time);
CREATE INDEX IF NOT EXISTS idx_weighing_data_status ON weighing_data(status);

-- 添加角色表索引
CREATE INDEX IF NOT EXISTS idx_roles_role_id ON roles(role_id);
CREATE INDEX IF NOT EXISTS idx_roles_role_name ON roles(role_name);

-- 添加权限表索引
CREATE INDEX IF NOT EXISTS idx_permissions_permission_id ON permissions(permission_id);
CREATE INDEX IF NOT EXISTS idx_permissions_role_id ON permissions(role_id);


