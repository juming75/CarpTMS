-- 启用PostGIS扩展
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS postgis_topology;

-- 为logistics_tracks表添加地理空间列
ALTER TABLE IF EXISTS logistics_tracks 
ADD COLUMN IF NOT EXISTS geom GEOMETRY(Point, 4326);

-- 更新已存在数据的geom列
UPDATE logistics_tracks 
SET geom = ST_SetSRID(ST_MakePoint(longitude, latitude), 4326)
WHERE geom IS NULL;

-- 创建触发器，自动更新geom列
CREATE OR REPLACE FUNCTION update_logistics_tracks_geom()
RETURNS TRIGGER AS $$
BEGIN
    NEW.geom = ST_SetSRID(ST_MakePoint(NEW.longitude, NEW.latitude), 4326);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_logistics_tracks_geom
BEFORE INSERT OR UPDATE ON logistics_tracks
FOR EACH ROW
WHEN (NEW.longitude IS NOT NULL AND NEW.latitude IS NOT NULL)
EXECUTE FUNCTION update_logistics_tracks_geom();

-- 创建空间索引，优化地理空间查询
CREATE INDEX IF NOT EXISTS idx_logistics_tracks_geom 
ON logistics_tracks USING GIST (geom);

-- 创建车辆实时位置表，用于高效存储和查询当前车辆位置
CREATE TABLE IF NOT EXISTS vehicle_realtime_locations (
    vehicle_id INTEGER PRIMARY KEY,
    longitude DOUBLE PRECISION NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    geom GEOMETRY(Point, 4326) NOT NULL,
    speed NUMERIC(8, 2),
    direction SMALLINT,
    altitude NUMERIC(8, 2),
    accuracy NUMERIC(8, 2),
    status SMALLINT NOT NULL DEFAULT 1,
    update_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束
    FOREIGN KEY (vehicle_id) REFERENCES vehicles(vehicle_id)
);

-- 创建车辆实时位置的空间索引
CREATE INDEX IF NOT EXISTS idx_vehicle_realtime_locations_geom 
ON vehicle_realtime_locations USING GIST (geom);

-- 创建车辆实时位置的更新时间索引
CREATE INDEX IF NOT EXISTS idx_vehicle_realtime_locations_update_time 
ON vehicle_realtime_locations(update_time DESC);

-- 创建轨迹线表，用于存储完整轨迹线段，减少轨迹回放时的计算量
CREATE TABLE IF NOT EXISTS logistics_track_lines (
    line_id SERIAL PRIMARY KEY,
    vehicle_id INTEGER NOT NULL,
    order_id INTEGER NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP NOT NULL,
    geom GEOMETRY(LineString, 4326) NOT NULL,
    point_count INTEGER NOT NULL,
    distance NUMERIC(12, 2), -- 轨迹长度（米）
    status SMALLINT NOT NULL DEFAULT 1,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束
    FOREIGN KEY (vehicle_id) REFERENCES vehicles(vehicle_id),
    FOREIGN KEY (order_id) REFERENCES orders(order_id)
);

-- 创建轨迹线的空间索引
CREATE INDEX IF NOT EXISTS idx_logistics_track_lines_geom 
ON logistics_track_lines USING GIST (geom);

-- 创建轨迹线的车辆和时间索引
CREATE INDEX IF NOT EXISTS idx_logistics_track_lines_vehicle_time 
ON logistics_track_lines(vehicle_id, start_time DESC, end_time DESC);

-- 添加表注释
COMMENT ON TABLE vehicle_realtime_locations IS '车辆实时位置表，用于高效存储和查询当前车辆位置';
COMMENT ON TABLE logistics_track_lines IS '轨迹线表，用于存储完整轨迹线段，优化轨迹回放性能';
COMMENT ON COLUMN logistics_tracks.geom IS '轨迹点的地理空间数据';
COMMENT ON COLUMN vehicle_realtime_locations.geom IS '车辆当前位置的地理空间数据';
COMMENT ON COLUMN logistics_track_lines.geom IS '完整轨迹线的地理空间数据';


