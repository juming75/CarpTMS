use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取数据库连接
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:123@localhost:5432/CarpTMS_db".to_string());

    tracing::info!(
        database = %database_url.split('@').next_back().unwrap_or(""),
        "连接数据库"
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("数据库连接成功");

    // 创建费用表
    let create_finance_costs = r#"
        CREATE TABLE IF NOT EXISTS finance_costs (
            cost_id SERIAL PRIMARY KEY,
            cost_type VARCHAR(100) NOT NULL,
            amount DECIMAL(15,2) NOT NULL,
            description TEXT,
            cost_date DATE NOT NULL,
            create_time TIMESTAMP NOT NULL DEFAULT NOW(),
            update_time TIMESTAMP
        )
    "#;

    tracing::info!("创建 finance_costs 表...");
    sqlx::query(create_finance_costs).execute(&pool).await?;
    tracing::info!("finance_costs 表创建成功");

    // 创建发票表
    let create_finance_invoices = r#"
        CREATE TABLE IF NOT EXISTS finance_invoices (
            invoice_id SERIAL PRIMARY KEY,
            invoice_number VARCHAR(100) NOT NULL UNIQUE,
            amount DECIMAL(15,2) NOT NULL,
            invoice_date DATE NOT NULL,
            description TEXT,
            create_time TIMESTAMP NOT NULL DEFAULT NOW(),
            update_time TIMESTAMP
        )
    "#;

    tracing::info!("创建 finance_invoices 表...");
    sqlx::query(create_finance_invoices).execute(&pool).await?;
    tracing::info!("finance_invoices 表创建成功");

    // 创建传感器标定表（完整DDD改造，24列）
    let create_sensor_calibration = r#"
        CREATE TABLE IF NOT EXISTS sensor_calibration (
            id SERIAL PRIMARY KEY,
            sensor_no INTEGER NOT NULL,
            vehicle_id INTEGER NOT NULL DEFAULT 0,
            plate_no VARCHAR(20) NOT NULL DEFAULT '',
            sensor_side VARCHAR(10) NOT NULL,
            sensor_group INTEGER,
            self_weight INTEGER,
            polynomial_json TEXT,
            linear_segments_json TEXT,
            is_calibrated BOOLEAN NOT NULL DEFAULT FALSE,
            create_time TIMESTAMP NOT NULL DEFAULT NOW(),
            update_time TIMESTAMP,
            calibration_points TEXT,
            pa_raw INTEGER,
            axle_number INTEGER,
            is_left_wheel BOOLEAN,
            turn_point INTEGER,
            polynomial_order INTEGER,
            r2_score NUMERIC(10,4),
            rmse NUMERIC(10,2),
            max_error NUMERIC(10,2),
            point_count INTEGER,
            rated_total_weight NUMERIC(10,2),
            tare_weight NUMERIC(10,2)
        )
    "#;

    tracing::info!("创建 sensor_calibration 表...");
    sqlx::query(create_sensor_calibration)
        .execute(&pool)
        .await?;
    tracing::info!("sensor_calibration 表创建成功");

    // 创建索引
    let create_indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_finance_costs_cost_type ON finance_costs(cost_type)",
        "CREATE INDEX IF NOT EXISTS idx_finance_costs_cost_date ON finance_costs(cost_date)",
        "CREATE INDEX IF NOT EXISTS idx_finance_invoices_invoice_number ON finance_invoices(invoice_number)",
        "CREATE INDEX IF NOT EXISTS idx_finance_invoices_invoice_date ON finance_invoices(invoice_date)",
        "CREATE INDEX IF NOT EXISTS idx_sensor_calibration_sensor_no ON sensor_calibration(sensor_no)",
        "CREATE INDEX IF NOT EXISTS idx_sensor_calibration_vehicle_id ON sensor_calibration(vehicle_id)",
        "CREATE INDEX IF NOT EXISTS idx_sensor_calibration_plate_no ON sensor_calibration(plate_no)",
        "CREATE INDEX IF NOT EXISTS idx_sensor_calibration_create_time ON sensor_calibration(create_time)",
    ];

    tracing::info!("创建索引...");
    for index_sql in create_indexes {
        sqlx::query(index_sql).execute(&pool).await?;
    }
    tracing::info!("索引创建成功");

    tracing::info!("所有数据库表创建完成!");

    // 验证表是否存在
    let tables = sqlx::query_scalar::<_, String>(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public' AND tablename IN ('finance_costs', 'finance_invoices', 'sensor_calibration') ORDER BY tablename"
    )
    .fetch_all(&pool)
    .await?;

    tracing::info!("已创建的表:");
    for table in &tables {
        tracing::info!(table = %table, "  表创建确认");
    }

    Ok(())
}
