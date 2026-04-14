-- 标定历史记录表
-- 用于记录校准操作的完整历史，支持审计和回滚

CREATE TABLE IF NOT EXISTS "calibration_history" (
    "id" SERIAL PRIMARY KEY,
    "sensor_no" INTEGER NOT NULL,
    "vehicle_id" INTEGER NOT NULL,
    "plate_no" VARCHAR(20),
    "polynomial_json" TEXT NOT NULL,
    "polynomial_order" INTEGER NOT NULL DEFAULT 2,
    "r2_score" NUMERIC(10,4) NOT NULL,
    "rmse" NUMERIC(10,2) NOT NULL,
    "max_error" NUMERIC(10,2) NOT NULL,
    "point_count" INTEGER NOT NULL,
    "operation_type" VARCHAR(20) NOT NULL DEFAULT 'auto',
    "operation_type_name" VARCHAR(50),
    "operator" VARCHAR(50),
    "remark" TEXT,
    "is_valid" BOOLEAN NOT NULL DEFAULT TRUE,
    "create_time" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "update_time" TIMESTAMP,
    CONSTRAINT "calibration_history_operation_type_check" 
        CHECK ("operation_type" IN ('auto', 'manual', 'import', 'adjust'))
);

COMMENT ON TABLE "calibration_history" IS '标定历史记录表';
COMMENT ON COLUMN "calibration_history"."id" IS '记录ID';
COMMENT ON COLUMN "calibration_history"."sensor_no" IS '传感器编号';
COMMENT ON COLUMN "calibration_history"."vehicle_id" IS '车辆ID';
COMMENT ON COLUMN "calibration_history"."plate_no" IS '车牌号';
COMMENT ON COLUMN "calibration_history"."polynomial_json" IS '多项式系数JSON';
COMMENT ON COLUMN "calibration_history"."polynomial_order" IS '多项式阶数';
COMMENT ON COLUMN "calibration_history"."r2_score" IS 'R²决定系数';
COMMENT ON COLUMN "calibration_history"."rmse" IS '均方根误差';
COMMENT ON COLUMN "calibration_history"."max_error" IS '最大误差';
COMMENT ON COLUMN "calibration_history"."point_count" IS '标定点数';
COMMENT ON COLUMN "calibration_history"."operation_type" IS '操作类型: auto自动/manual手动/import导入/adjust调整';
COMMENT ON COLUMN "calibration_history"."operation_type_name" IS '操作类型名称';
COMMENT ON COLUMN "calibration_history"."operator" IS '操作人';
COMMENT ON COLUMN "calibration_history"."remark" IS '备注';
COMMENT ON COLUMN "calibration_history"."is_valid" IS '是否有效';
COMMENT ON COLUMN "calibration_history"."create_time" IS '创建时间';
COMMENT ON COLUMN "calibration_history"."update_time" IS '更新时间';

-- 创建索引
CREATE INDEX IF NOT EXISTS "calibration_history_sensor_no_index" 
    ON "calibration_history"("sensor_no");
CREATE INDEX IF NOT EXISTS "calibration_history_vehicle_id_index" 
    ON "calibration_history"("vehicle_id");
CREATE INDEX IF NOT EXISTS "calibration_history_plate_no_index" 
    ON "calibration_history"("plate_no");
CREATE INDEX IF NOT EXISTS "calibration_history_create_time_index" 
    ON "calibration_history"("create_time" DESC);

-- 创建触发器自动 更新时间
CREATE OR REPLACE FUNCTION update_calibration_history_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW."update_time" = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER "calibration_history_update_time_trigger"
    BEFORE UPDATE ON "calibration_history"
    FOR EACH ROW
    EXECUTE FUNCTION update_calibration_history_timestamp();

-- 批量标定导入临时表
CREATE TABLE IF NOT EXISTS "calibration_import_temp" (
    "id" SERIAL PRIMARY KEY,
    "sensor_no" INTEGER NOT NULL,
    "pa_value" NUMERIC(10,2) NOT NULL,
    "actual_weight" NUMERIC(10,2) NOT NULL,
    "temperature" NUMERIC(10,2),
    "plate_no" VARCHAR(20),
    "imported_by" VARCHAR(50),
    "import_time" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "status" VARCHAR(20) NOT NULL DEFAULT 'pending',
    "error_message" TEXT
);

COMMENT ON TABLE "calibration_import_temp" IS '批量标定导入临时表';

-- 传感器标定表
CREATE TABLE IF NOT EXISTS "sensor_calibration" (
    "id" SERIAL PRIMARY KEY,
    "sensor_no" INTEGER NOT NULL,
    "vehicle_id" INTEGER NOT NULL,
    "plate_no" VARCHAR(20),
    "sensor_side" VARCHAR(10),
    "sensor_group" INTEGER,
    "self_weight" INTEGER,
    "polynomial_json" TEXT,
    "linear_segments_json" TEXT,
    "is_calibrated" BOOLEAN NOT NULL DEFAULT FALSE,
    "create_time" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "update_time" TIMESTAMP
);

COMMENT ON TABLE "sensor_calibration" IS '传感器标定表';
COMMENT ON COLUMN "sensor_calibration"."id" IS '记录ID';
COMMENT ON COLUMN "sensor_calibration"."sensor_no" IS '传感器编号';
COMMENT ON COLUMN "sensor_calibration"."vehicle_id" IS '车辆ID';
COMMENT ON COLUMN "sensor_calibration"."plate_no" IS '车牌号';
COMMENT ON COLUMN "sensor_calibration"."sensor_side" IS '传感器位置';
COMMENT ON COLUMN "sensor_calibration"."sensor_group" IS '传感器组';
COMMENT ON COLUMN "sensor_calibration"."self_weight" IS '自重';
COMMENT ON COLUMN "sensor_calibration"."polynomial_json" IS '多项式系数JSON';
COMMENT ON COLUMN "sensor_calibration"."linear_segments_json" IS '线性分段JSON';
COMMENT ON COLUMN "sensor_calibration"."is_calibrated" IS '是否已标定';
COMMENT ON COLUMN "sensor_calibration"."create_time" IS '创建时间';
COMMENT ON COLUMN "sensor_calibration"."update_time" IS '更新时间';

-- 创建索引
CREATE INDEX IF NOT EXISTS "sensor_calibration_sensor_no_index" 
    ON "sensor_calibration"("sensor_no");
CREATE INDEX IF NOT EXISTS "sensor_calibration_vehicle_id_index" 
    ON "sensor_calibration"("vehicle_id");
CREATE INDEX IF NOT EXISTS "sensor_calibration_plate_no_index" 
    ON "sensor_calibration"("plate_no");

-- 创建触发器自动 更新时间
CREATE OR REPLACE FUNCTION update_sensor_calibration_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW."update_time" = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER "sensor_calibration_update_time_trigger"
    BEFORE UPDATE ON "sensor_calibration"
    FOR EACH ROW
    EXECUTE FUNCTION update_sensor_calibration_timestamp();
