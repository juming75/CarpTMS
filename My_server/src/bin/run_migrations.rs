use sqlx::PgPool;

async fn run_migration(
    pool: &PgPool,
    name: &str,
    sql_commands: &[&str],
) -> Result<(), sqlx::Error> {
    println!("=== 执行迁移: {}", name);
    for (i, sql) in sql_commands.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }
    Ok(())
}

async fn check_table_exists(pool: &PgPool, table: &str) -> Result<bool, sqlx::Error> {
    let result: (bool,) = sqlx::query_as(
        "SELECT EXISTS(
            SELECT 1 FROM information_schema.tables 
            WHERE table_name = $1
        )",
    )
    .bind(table)
    .fetch_one(pool)
    .await?;
    Ok(result.0)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 开始执行数据库迁移 ===\n");

    let database_url = "postgres://postgres:123@localhost:5432/postgres?client_encoding=UTF8";

    println!("连接数据库: {}", database_url);
    let pool = PgPool::connect(database_url).await?;
    println!("✓ 数据库连接成功\n");

    // 1. 创建车辆表
    if !check_table_exists(&pool, "vehicles").await? {
        let vehicles_commands = [r#"CREATE TABLE IF NOT EXISTS vehicles (
                vehicle_id SERIAL PRIMARY KEY,
                plate_number VARCHAR(20) NOT NULL UNIQUE,
                vehicle_type VARCHAR(50),
                capacity NUMERIC(10, 2),
                status SMALLINT NOT NULL DEFAULT 1,
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#];
        run_migration(&pool, "创建车辆表", &vehicles_commands).await?;
    } else {
        println!("✓ 车辆表已存在");
    }

    // 2. 创建用户组表
    if !check_table_exists(&pool, "user_groups").await? {
        let user_groups_commands = [
            r#"CREATE TABLE IF NOT EXISTS user_groups (
                group_id SERIAL PRIMARY KEY,
                group_name VARCHAR(100) NOT NULL,
                description TEXT,
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#,
            r#"INSERT INTO user_groups (group_name, description) 
               VALUES 
                   ('管理员', '系统管理员角色'),
                   ('普通用户', '普通用户角色'),
                   ('经理', '部门经理角色')
               ON CONFLICT DO NOTHING"#,
        ];
        run_migration(&pool, "创建用户组表", &user_groups_commands).await?;
    } else {
        println!("✓ 用户组表已存在");
    }

    // 3. 创建用户表
    if !check_table_exists(&pool, "users").await? {
        let users_commands = [
            r#"CREATE TABLE IF NOT EXISTS users (
                user_id SERIAL PRIMARY KEY,
                user_name VARCHAR(50) NOT NULL UNIQUE,
                password VARCHAR(100) NOT NULL,
                real_name VARCHAR(100),
                email TEXT,
                phone VARCHAR(20),
                user_group_id INTEGER NOT NULL REFERENCES user_groups(group_id),
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#,
            r#"INSERT INTO users (user_name, password, real_name, user_group_id)
               VALUES
                   ('admin', 'admin123', '管理员', 1)
               ON CONFLICT DO NOTHING"#,
        ];
        run_migration(&pool, "创建用户表", &users_commands).await?;
    } else {
        println!("✓ 用户表已存在");
    }

    // 4. 创建车组表
    if !check_table_exists(&pool, "vehicle_groups").await? {
        let vehicle_groups_commands = [
            r#"CREATE TABLE IF NOT EXISTS vehicle_groups (
                group_id SERIAL PRIMARY KEY,
                group_name VARCHAR(100) NOT NULL,
                parent_id INTEGER REFERENCES vehicle_groups(group_id),
                description TEXT,
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#,
            r#"INSERT INTO vehicle_groups (group_name, description) 
               VALUES 
                   ('默认车队', '系统默认车队'),
                   ('第一车队', '第一运输车队'),
                   ('第二车队', '第二运输车队')
               ON CONFLICT DO NOTHING"#,
        ];
        run_migration(&pool, "创建车组表", &vehicle_groups_commands).await?;
    } else {
        println!("✓ 车组表已存在");
    }

    // 5. 创建部门表
    if !check_table_exists(&pool, "departments").await? {
        let departments_commands = [
            r#"CREATE TABLE IF NOT EXISTS departments (
                department_id SERIAL PRIMARY KEY,
                department_name VARCHAR(100) NOT NULL,
                parent_department_id INTEGER REFERENCES departments(department_id),
                manager_id INTEGER REFERENCES users(user_id),
                phone VARCHAR(20),
                description TEXT,
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#,
            r#"INSERT INTO departments (department_name, description) 
               VALUES 
                   ('总公司', '总公司'),
                   ('技术部', '技术部门'),
                   ('市场部', '市场部门'),
                   ('财务部', '财务部门')
               ON CONFLICT DO NOTHING"#,
        ];
        run_migration(&pool, "创建部门表", &departments_commands).await?;
    } else {
        println!("✓ 部门表已存在");
    }

    // 6. 创建订单表
    if !check_table_exists(&pool, "orders").await? {
        let orders_commands = [
            r#"CREATE TABLE IF NOT EXISTS orders (
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
                update_time TIMESTAMP
            )"#,
            r#"CREATE TABLE IF NOT EXISTS order_items (
                item_id SERIAL PRIMARY KEY,
                order_id INTEGER NOT NULL,
                item_name VARCHAR(100) NOT NULL,
                item_description TEXT,
                quantity INTEGER NOT NULL,
                unit_price NUMERIC(10, 2) NOT NULL,
                total_price NUMERIC(10, 2) NOT NULL,
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP,
                FOREIGN KEY (order_id) REFERENCES orders(order_id) ON DELETE CASCADE
            )"#,
        ];
        run_migration(&pool, "创建订单表", &orders_commands).await?;
    } else {
        println!("✓ 订单表已存在");
    }

    // 7. 创建司机表
    if !check_table_exists(&pool, "drivers").await? {
        let drivers_commands = [r#"CREATE TABLE IF NOT EXISTS drivers (
                driver_id SERIAL PRIMARY KEY,
                driver_name VARCHAR(100) NOT NULL,
                license_number VARCHAR(50) NOT NULL UNIQUE,
                phone_number VARCHAR(20),
                email VARCHAR(100),
                status SMALLINT NOT NULL DEFAULT 1,
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#];
        run_migration(&pool, "创建司机表", &drivers_commands).await?;
    } else {
        println!("✓ 司机表已存在");
    }

    // 8. 创建财务表
    if !check_table_exists(&pool, "finance_costs").await? {
        let finance_commands = [
            r#"CREATE TABLE IF NOT EXISTS finance_costs (
                cost_id SERIAL PRIMARY KEY,
                cost_type VARCHAR(50) NOT NULL,
                amount NUMERIC(12, 2) NOT NULL,
                description TEXT,
                cost_date TIMESTAMP NOT NULL,
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#,
            r#"CREATE TABLE IF NOT EXISTS finance_invoices (
                invoice_id SERIAL PRIMARY KEY,
                invoice_number VARCHAR(50) NOT NULL UNIQUE,
                amount NUMERIC(12, 2) NOT NULL,
                invoice_date TIMESTAMP NOT NULL,
                description TEXT,
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#,
        ];
        run_migration(&pool, "创建财务表", &finance_commands).await?;
    } else {
        println!("✓ 财务表已存在");
    }

    // 9. 创建组织单位表
    if !check_table_exists(&pool, "organizations").await? {
        let organizations_commands = [
            r#"CREATE TABLE IF NOT EXISTS organizations (
                unit_id VARCHAR(50) PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                type VARCHAR(50) NOT NULL,
                parent_id INTEGER,
                description TEXT,
                contact_person VARCHAR(100),
                contact_phone VARCHAR(20),
                status VARCHAR(20) NOT NULL DEFAULT 'active',
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#,
            r#"CREATE INDEX IF NOT EXISTS idx_organizations_name ON organizations(name)"#,
            r#"CREATE INDEX IF NOT EXISTS idx_organizations_type ON organizations(type)"#,
            r#"CREATE INDEX IF NOT EXISTS idx_organizations_status ON organizations(status)"#,
            r#"INSERT INTO organizations (unit_id, name, type, parent_id, description, contact_person, contact_phone, status) 
               VALUES 
                   ('ORG001', '总公司', 'enterprise', NULL, '公司总部', '张总', '13800138000', 'active'),
                   ('ORG002', '技术部', 'enterprise', NULL, '技术部门', '李经理', '13900139000', 'active'),
                   ('ORG003', '市场部', 'enterprise', NULL, '市场部门', '王经理', '13700137000', 'active'),
                   ('ORG004', '财务部', 'enterprise', NULL, '财务部门', '赵经理', '13600136000', 'active')
               ON CONFLICT DO NOTHING"#,
        ];
        run_migration(&pool, "创建组织单位表", &organizations_commands).await?;
    } else {
        println!("✓ 组织单位表已存在");
    }

    // 9. 创建其他必要的表
    if !check_table_exists(&pool, "devices").await? {
        let devices_commands = [r#"CREATE TABLE IF NOT EXISTS devices (
                device_id VARCHAR(50) PRIMARY KEY,
                device_name VARCHAR(100) NOT NULL,
                device_type VARCHAR(50) NOT NULL,
                status SMALLINT NOT NULL DEFAULT 1,
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                update_time TIMESTAMP
            )"#];
        run_migration(&pool, "创建设备表", &devices_commands).await?;
    } else {
        println!("✓ 设备表已存在");
    }

    // 10. 创建位置相关表
    if !check_table_exists(&pool, "location_fences").await? {
        let location_fences_commands = [r#"CREATE TABLE IF NOT EXISTS location_fences (
                fence_id SERIAL PRIMARY KEY,
                fence_name VARCHAR(100) NOT NULL,
                fence_type VARCHAR(50) NOT NULL,
                center_latitude DOUBLE PRECISION,
                center_longitude DOUBLE PRECISION,
                radius DOUBLE PRECISION,
                polygon_points JSONB,
                rectangle_bounds JSONB,
                status VARCHAR(20) NOT NULL DEFAULT 'active',
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP
            )"#];
        run_migration(&pool, "创建电子围栏表", &location_fences_commands).await?;
    } else {
        println!("✓ 电子围栏表已存在");
    }

    if !check_table_exists(&pool, "location_positions").await? {
        let location_positions_commands = [r#"CREATE TABLE IF NOT EXISTS location_positions (
                position_id SERIAL PRIMARY KEY,
                place_name VARCHAR(100) NOT NULL,
                latitude DOUBLE PRECISION NOT NULL,
                longitude DOUBLE PRECISION NOT NULL,
                address TEXT,
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP
            )"#];
        run_migration(&pool, "创建位置表", &location_positions_commands).await?;
    } else {
        println!("✓ 位置表已存在");
    }

    if !check_table_exists(&pool, "location_places").await? {
        let location_places_commands = [r#"CREATE TABLE IF NOT EXISTS location_places (
                place_id SERIAL PRIMARY KEY,
                place_name VARCHAR(100) NOT NULL,
                address TEXT NOT NULL,
                contact_person VARCHAR(100),
                contact_phone VARCHAR(20),
                contact_email VARCHAR(100),
                latitude DOUBLE PRECISION,
                longitude DOUBLE PRECISION,
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP
            )"#];
        run_migration(&pool, "创建地点表", &location_places_commands).await?;
    } else {
        println!("✓ 地点表已存在");
    }

    if !check_table_exists(&pool, "location_routes").await? {
        let location_routes_commands = [r#"CREATE TABLE IF NOT EXISTS location_routes (
                route_id SERIAL PRIMARY KEY,
                route_name VARCHAR(100) NOT NULL,
                start_point VARCHAR(100) NOT NULL,
                start_latitude DOUBLE PRECISION,
                start_longitude DOUBLE PRECISION,
                end_point VARCHAR(100) NOT NULL,
                end_latitude DOUBLE PRECISION,
                end_longitude DOUBLE PRECISION,
                waypoints JSONB,
                distance DOUBLE PRECISION,
                estimated_duration INTEGER,
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP
            )"#];
        run_migration(&pool, "创建路线表", &location_routes_commands).await?;
    } else {
        println!("✓ 路线表已存在");
    }

    // 11. 创建载重标定相关表
    if !check_table_exists(&pool, "calibration_history").await? {
        let calibration_commands = [
            r#"-- 标定历史记录表
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
    EXECUTE FUNCTION update_sensor_calibration_timestamp();"#
        ];
        run_migration(&pool, "创建载重标定相关表", &calibration_commands).await?;
    } else {
        println!("✓ 载重标定相关表已存在");
    }

    println!("\n=== 数据库迁移完成！ ===");
    Ok(())
}
