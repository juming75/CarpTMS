-- 数据库索引优化迁移文件
-- 创建缺失的索引以提升查询性能

-- 车辆表 - 司机ID索引
-- 用于按司机查询车辆列表
CREATE INDEX IF NOT EXISTS idx_vehicles_driver_id ON vehicles(driver_id);

-- 设备表 - 设备ID索引
-- 用于按设备ID快速查找设备
CREATE INDEX IF NOT EXISTS idx_devices_device_id ON devices(device_id);

-- 订单表 - 状态索引
-- 用于按状态筛选订单（如待处理、已完成等）
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(order_status);

-- 告警表 - 类型索引
-- 用于按告警类型统计和筛选
CREATE INDEX IF NOT EXISTS idx_alerts_alert_type ON alerts(alert_type);

-- 告警表 - 状态索引
-- 用于按状态筛选告警（未处理、已处理等）
CREATE INDEX IF NOT EXISTS idx_alerts_status ON alerts(status);

-- 告警表 - 创建时间索引（用于分页排序）
-- 倒序索引，优化 ORDER BY created_at DESC 查询
CREATE INDEX IF NOT EXISTS idx_alerts_created_at ON alerts(created_at DESC);

-- 可选：复合索引示例（根据实际查询需求启用）
-- CREATE INDEX IF NOT EXISTS idx_alerts_status_created_at ON alerts(status, created_at DESC);
-- CREATE INDEX IF NOT EXISTS idx_orders_status_created_at ON orders(order_status, create_time DESC);
