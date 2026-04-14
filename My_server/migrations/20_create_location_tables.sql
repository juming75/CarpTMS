-- 创建电子围栏表
CREATE TABLE IF NOT EXISTS location_fences (
    fence_id SERIAL PRIMARY KEY,
    fence_name VARCHAR(100) NOT NULL,
    fence_type VARCHAR(50) NOT NULL, -- circle, polygon, rectangle
    center_latitude DECIMAL(10, 8),
    center_longitude DECIMAL(11, 8),
    radius DECIMAL(10, 2), -- 用于圆形围栏
    polygon_points JSONB, -- 用于多边形围栏，存储坐标点数组
    rectangle_bounds JSONB, -- 用于矩形围栏，存储东北和西南坐标
    status VARCHAR(20) DEFAULT 'active', -- active, inactive
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    create_user_id INTEGER,
    update_user_id INTEGER
);

-- 创建位置表
CREATE TABLE IF NOT EXISTS locations (
    location_id SERIAL PRIMARY KEY,
    location_name VARCHAR(100) NOT NULL,
    latitude DECIMAL(10, 8) NOT NULL,
    longitude DECIMAL(11, 8) NOT NULL,
    address VARCHAR(255),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    create_user_id INTEGER,
    update_user_id INTEGER
);

-- 创建地点表
CREATE TABLE IF NOT EXISTS places (
    place_id SERIAL PRIMARY KEY,
    place_name VARCHAR(100) NOT NULL,
    address VARCHAR(255) NOT NULL,
    contact_person VARCHAR(50),
    contact_phone VARCHAR(20),
    contact_email VARCHAR(100),
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    create_user_id INTEGER,
    update_user_id INTEGER
);

-- 创建路线表
CREATE TABLE IF NOT EXISTS routes (
    route_id SERIAL PRIMARY KEY,
    route_name VARCHAR(100) NOT NULL,
    start_point VARCHAR(255) NOT NULL,
    start_latitude DECIMAL(10, 8),
    start_longitude DECIMAL(11, 8),
    end_point VARCHAR(255) NOT NULL,
    end_latitude DECIMAL(10, 8),
    end_longitude DECIMAL(11, 8),
    waypoints JSONB, -- 途经点数组
    distance DECIMAL(10, 2), -- 距离（公里）
    estimated_duration INTEGER, -- 预计耗时（分钟）
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    create_user_id INTEGER,
    update_user_id INTEGER
);

-- 添加注释
COMMENT ON TABLE location_fences IS '电子围栏表';
COMMENT ON TABLE locations IS '位置表';
COMMENT ON TABLE places IS '地点表';
COMMENT ON TABLE routes IS '路线表';

-- 创建索引
CREATE INDEX idx_location_fences_status ON location_fences(status);
CREATE INDEX idx_locations_coordinates ON locations(latitude, longitude);
CREATE INDEX idx_places_coordinates ON places(latitude, longitude);
CREATE INDEX idx_routes_name ON routes(route_name);

-- 插入示例数据
INSERT INTO location_fences (fence_name, fence_type, status, description) VALUES
('仓库围栏', 'circle', 'active', '主仓库电子围栏'),
('厂区围栏', 'polygon', 'active', '厂区范围围栏')
ON CONFLICT DO NOTHING;

INSERT INTO locations (location_name, latitude, longitude, description) VALUES
('仓库 A', 39.9042, 116.4074, '主仓库'),
('仓库 B', 39.9142, 116.4174, '备用仓库'),
('卸货点', 39.9242, 116.4274, '主要卸货区域')
ON CONFLICT DO NOTHING;

INSERT INTO places (place_name, address, contact_person, contact_phone, description) VALUES
('总部', '北京市朝阳区', '张三', '13800138000', '公司总部'),
('分公司', '上海市浦东新区', '李四', '13900139000', '上海分公司'),
('配送中心', '广州市天河区', '王五', '13700137000', '华南配送中心')
ON CONFLICT DO NOTHING;

INSERT INTO routes (route_name, start_point, end_point, distance, description) VALUES
('路线 1', '仓库 A', '客户 A', 50.0, '主要配送路线'),
('路线 2', '仓库 B', '客户 B', 30.0, '短途配送路线'),
('路线 3', '仓库 A', '客户 C', 80.0, '长途配送路线')
ON CONFLICT DO NOTHING;


