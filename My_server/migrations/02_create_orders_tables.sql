-- 创建订单表
CREATE TABLE IF NOT EXISTS orders (
    order_id SERIAL PRIMARY KEY,
    order_no VARCHAR(50) NOT NULL UNIQUE,
    vehicle_id INTEGER NOT NULL,
    driver_id INTEGER,
    customer_name VARCHAR(100) NOT NULL,
    customer_phone VARCHAR(20) NOT NULL,
    origin TEXT NOT NULL,
    destination TEXT NOT NULL,
    cargo_type VARCHAR(100) NOT NULL,
    cargo_weight NUMERIC(10, 2) NOT NULL,
    cargo_volume NUMERIC(10, 2) NOT NULL,
    cargo_count INTEGER NOT NULL,
    order_amount NUMERIC(12, 2) NOT NULL,
    order_status SMALLINT NOT NULL DEFAULT 1,
    departure_time TIMESTAMP,
    arrival_time TIMESTAMP,
    remark TEXT,
    create_user_id INTEGER NOT NULL,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP,
    
    -- 约束
    FOREIGN KEY (vehicle_id) REFERENCES vehicles(vehicle_id)
);

-- 创建订单项表
CREATE TABLE IF NOT EXISTS order_items (
    item_id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL,
    item_name VARCHAR(100) NOT NULL,
    item_description TEXT,
    quantity INTEGER NOT NULL,
    unit_price NUMERIC(10, 2) NOT NULL,
    total_price NUMERIC(10, 2) NOT NULL,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP,
    
    -- 约束
    FOREIGN KEY (order_id) REFERENCES orders(order_id) ON DELETE CASCADE
);

-- 为常用查询创建索引
CREATE INDEX IF NOT EXISTS idx_orders_vehicle_id 
ON orders(vehicle_id);

CREATE INDEX IF NOT EXISTS idx_orders_order_no 
ON orders(order_no);

CREATE INDEX IF NOT EXISTS idx_orders_customer_name 
ON orders(customer_name);

CREATE INDEX IF NOT EXISTS idx_order_items_order_id 
ON order_items(order_id);

-- 添加表注释
COMMENT ON TABLE orders IS '订单表';
COMMENT ON COLUMN orders.order_id IS '订单ID';
COMMENT ON COLUMN orders.order_no IS '订单编号';
COMMENT ON COLUMN orders.vehicle_id IS '车辆ID';
COMMENT ON COLUMN orders.driver_id IS '司机ID';
COMMENT ON COLUMN orders.customer_name IS '客户名称';
COMMENT ON COLUMN orders.customer_phone IS '客户电话';
COMMENT ON COLUMN orders.origin IS '出发地';
COMMENT ON COLUMN orders.destination IS '目的地';
COMMENT ON COLUMN orders.cargo_type IS '货物类型';
COMMENT ON COLUMN orders.cargo_weight IS '货物重量';
COMMENT ON COLUMN orders.cargo_volume IS '货物体积';
COMMENT ON COLUMN orders.cargo_count IS '货物数量';
COMMENT ON COLUMN orders.order_amount IS '订单金额';
COMMENT ON COLUMN orders.order_status IS '订单状态';
COMMENT ON COLUMN orders.departure_time IS '出发时间';
COMMENT ON COLUMN orders.arrival_time IS '到达时间';
COMMENT ON COLUMN orders.remark IS '备注';
COMMENT ON COLUMN orders.create_user_id IS '创建用户ID';
COMMENT ON COLUMN orders.create_time IS '创建时间';
COMMENT ON COLUMN orders.update_time IS '更新时间';

COMMENT ON TABLE order_items IS '订单项表';
COMMENT ON COLUMN order_items.item_id IS '订单项ID';
COMMENT ON COLUMN order_items.order_id IS '订单ID';
COMMENT ON COLUMN order_items.item_name IS '商品名称';
COMMENT ON COLUMN order_items.item_description IS '商品描述';
COMMENT ON COLUMN order_items.quantity IS '数量';
COMMENT ON COLUMN order_items.unit_price IS '单价';
COMMENT ON COLUMN order_items.total_price IS '总价';
COMMENT ON COLUMN order_items.create_time IS '创建时间';
COMMENT ON COLUMN order_items.update_time IS '更新时间';

