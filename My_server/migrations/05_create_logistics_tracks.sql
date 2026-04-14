-- 创建物流轨迹表
CREATE TABLE IF NOT EXISTS logistics_tracks (
    track_id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL,
    vehicle_id INTEGER NOT NULL,
    track_time TIMESTAMP NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    address TEXT,
    status SMALLINT NOT NULL DEFAULT 1,
    remark TEXT,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束
    FOREIGN KEY (order_id) REFERENCES orders(order_id),
    FOREIGN KEY (vehicle_id) REFERENCES vehicles(vehicle_id)
);

-- 为常用查询创建索引
CREATE INDEX IF NOT EXISTS idx_logistics_tracks_vehicle_id 
ON logistics_tracks(vehicle_id);

CREATE INDEX IF NOT EXISTS idx_logistics_tracks_order_id 
ON logistics_tracks(order_id);

CREATE INDEX IF NOT EXISTS idx_logistics_tracks_track_time 
ON logistics_tracks(track_time DESC);

CREATE INDEX IF NOT EXISTS idx_logistics_tracks_vehicle_time 
ON logistics_tracks(vehicle_id, track_time DESC);

-- 添加表注释
COMMENT ON TABLE logistics_tracks IS '物流轨迹表';
COMMENT ON COLUMN logistics_tracks.track_id IS '轨迹ID';
COMMENT ON COLUMN logistics_tracks.order_id IS '订单ID';
COMMENT ON COLUMN logistics_tracks.vehicle_id IS '车辆ID';
COMMENT ON COLUMN logistics_tracks.track_time IS '轨迹时间';
COMMENT ON COLUMN logistics_tracks.latitude IS '纬度';
COMMENT ON COLUMN logistics_tracks.longitude IS '经度';
COMMENT ON COLUMN logistics_tracks.address IS '地址';
COMMENT ON COLUMN logistics_tracks.status IS '状态';
COMMENT ON COLUMN logistics_tracks.remark IS '备注';
COMMENT ON COLUMN logistics_tracks.create_time IS '创建时间';

