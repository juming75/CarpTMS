-- 标定系统 DDD 改造：支持 service 层多项式拟合计算
-- 
-- 变更说明：
-- 1. calibration_points JSONB：存储该传感器的所有标定点，供 service 计算系数
-- 2. pa_raw：原始 AD 值（100 倍）
-- 3. axle_number：轴号（1-3）
-- 4. is_left_wheel：左右侧
-- 5. turn_point：转折点 AD 值（默认 50000）
-- 6. polynomial_order：多项式阶数（1=线性，2=二阶，3=三阶）
-- 7. 新增 weight_calculation 表：车辆称重记录（6 个传感器求和）

-- sensor_calibration 表新增字段
ALTER TABLE sensor_calibration
    ADD COLUMN IF NOT EXISTS calibration_points JSONB  DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS pa_raw              BIGINT DEFAULT 0,
    ADD COLUMN IF NOT EXISTS axle_number         SMALLINT DEFAULT 1,
    ADD COLUMN IF NOT EXISTS is_left_wheel      BOOLEAN DEFAULT true,
    ADD COLUMN IF NOT EXISTS turn_point          BIGINT DEFAULT 50000,
    ADD COLUMN IF NOT EXISTS polynomial_order    SMALLINT DEFAULT 2,
    ADD COLUMN IF NOT EXISTS polynomial_coefs_json TEXT,
    ADD COLUMN IF NOT EXISTS r2_score            NUMERIC(10,4) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS rmse               NUMERIC(10,2) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS max_error           NUMERIC(10,2) DEFAULT 0,
    ADD COLUMN IF NOT EXISTS point_count         INTEGER DEFAULT 0,
    ADD COLUMN IF NOT EXISTS rated_total_weight  NUMERIC(10,2) DEFAULT 32000,
    ADD COLUMN IF NOT EXISTS tare_weight         NUMERIC(10,2) DEFAULT 12000;

-- 为 sensor_calibration 新增字段添加注释
COMMENT ON COLUMN sensor_calibration.calibration_points    IS '标定点 JSONB 数组 [{pa_raw, actual_weight, temperature, load_percentage, is_manual}]';
COMMENT ON COLUMN sensor_calibration.pa_raw                 IS 'Pa 原始值（AD 值）';
COMMENT ON COLUMN sensor_calibration.axle_number            IS '轴号（1-6）';
COMMENT ON COLUMN sensor_calibration.is_left_wheel          IS '是否左侧轮';
COMMENT ON COLUMN sensor_calibration.turn_point             IS '分段转折点 AD 值（默认 50000）';
COMMENT ON COLUMN sensor_calibration.polynomial_order       IS '多项式阶数：1=线性，2=二阶，3=三阶';
COMMENT ON COLUMN sensor_calibration.polynomial_coefs_json  IS '计算后的多项式系数 JSON';
COMMENT ON COLUMN sensor_calibration.r2_score               IS 'R² 决定系数';
COMMENT ON COLUMN sensor_calibration.rmse                  IS '均方根误差 RMSE';
COMMENT ON COLUMN sensor_calibration.max_error             IS '最大误差';
COMMENT ON COLUMN sensor_calibration.point_count            IS '拟合使用的标定点数量';
COMMENT ON COLUMN sensor_calibration.rated_total_weight     IS '额定总重 kg';
COMMENT ON COLUMN sensor_calibration.tare_weight            IS '空车自重 kg';

-- 车辆标定总表：一辆车一条记录
CREATE TABLE IF NOT EXISTS vehicle_calibration_sheet (
    id                   SERIAL PRIMARY KEY,
    plate_no             VARCHAR(20)  NOT NULL UNIQUE,
    vehicle_id           INTEGER     NOT NULL,
    axle_count           SMALLINT    NOT NULL DEFAULT 3,
    sensor_count         SMALLINT    NOT NULL DEFAULT 6,
    rated_total_weight   NUMERIC(10,2) NOT NULL DEFAULT 32000,
    tare_weight          NUMERIC(10,2) NOT NULL DEFAULT 12000,
    axle_coefficients    JSONB      DEFAULT '{}',
    is_completed         BOOLEAN     NOT NULL DEFAULT false,
    create_time          TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time          TIMESTAMP
);
COMMENT ON TABLE vehicle_calibration_sheet IS '车辆标定总表';
CREATE INDEX IF NOT EXISTS vehicle_calibration_sheet_plate_no ON vehicle_calibration_sheet(plate_no);

-- 车辆称重记录表
CREATE TABLE IF NOT EXISTS vehicle_weight_record (
    id                   SERIAL PRIMARY KEY,
    plate_no             VARCHAR(20)  NOT NULL,
    vehicle_id           INTEGER     NOT NULL,
    axle_count           SMALLINT    NOT NULL DEFAULT 3,
    sensor_weights        JSONB       NOT NULL DEFAULT '{}',
    total_weight         NUMERIC(10,2) NOT NULL,
    tare_weight          NUMERIC(10,2) NOT NULL,
    load_weight          NUMERIC(10,2) NOT NULL,
    overload             BOOLEAN     NOT NULL DEFAULT false,
    overload_amount      NUMERIC(10,2) NOT NULL DEFAULT 0,
    recorded_at          TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    remark               TEXT
);
COMMENT ON TABLE vehicle_weight_record IS '车辆称重记录';
CREATE INDEX IF NOT EXISTS vehicle_weight_record_plate_no ON vehicle_weight_record(plate_no);
CREATE INDEX IF NOT EXISTS vehicle_weight_record_recorded_at ON vehicle_weight_record(recorded_at DESC);
