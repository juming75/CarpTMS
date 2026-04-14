use sqlx::PgPool;
use std::env;

async fn check_column_exists(
    pool: &PgPool,
    table: &str,
    column: &str,
) -> Result<bool, sqlx::Error> {
    let result: (bool,) = sqlx::query_as(
        "SELECT EXISTS(
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = $1 AND column_name = $2
        )",
    )
    .bind(table)
    .bind(column)
    .fetch_one(pool)
    .await?;
    Ok(result.0)
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

async fn fix_sensor_data_status(pool: &PgPool) -> Result<(), sqlx::Error> {
    if check_column_exists(pool, "sensor_data", "status").await? {
        println!("✓ sensor_data.status 字段已存在");
        return Ok(());
    }

    sqlx::query("ALTER TABLE sensor_data ADD COLUMN status INTEGER DEFAULT 1")
        .execute(pool)
        .await?;

    println!("✓ 已添加 sensor_data.status 字段");
    Ok(())
}

async fn fix_sensor_data_aggregated(pool: &PgPool) -> Result<(), sqlx::Error> {
    if !check_table_exists(pool, "sensor_data_aggregated").await? {
        println!("✓ 创建 sensor_data_aggregated 表");
        sqlx::query(
            "CREATE TABLE sensor_data_aggregated (
                id SERIAL PRIMARY KEY,
                vehicle_id INTEGER NOT NULL REFERENCES vehicles(vehicle_id),
                sensor_type VARCHAR(50) NOT NULL,
                start_time TIMESTAMP NOT NULL,
                end_time TIMESTAMP NOT NULL,
                count INTEGER NOT NULL,
                min_value NUMERIC(20, 4),
                max_value NUMERIC(20, 4),
                avg_value NUMERIC(20, 4),
                sum_value NUMERIC(20, 4),
                unit VARCHAR(50),
                create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(pool)
        .await?;
        return Ok(());
    }

    if check_column_exists(pool, "sensor_data_aggregated", "updated_at").await? {
        println!("✓ sensor_data_aggregated.updated_at 字段已存在");
    } else {
        sqlx::query("ALTER TABLE sensor_data_aggregated ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP")
            .execute(pool)
            .await?;
        println!("✓ 已添加 sensor_data_aggregated.updated_at 字段");
    }

    Ok(())
}

async fn fix_vehicles_update_time(pool: &PgPool) -> Result<(), sqlx::Error> {
    if check_column_exists(pool, "vehicles", "updated_at").await? {
        println!("✓ 重命名 vehicles.updated_at 为 update_time");
        sqlx::query("ALTER TABLE vehicles RENAME COLUMN updated_at TO update_time")
            .execute(pool)
            .await?;
        return Ok(());
    }

    if check_column_exists(pool, "vehicles", "update_time").await? {
        println!("✓ vehicles.update_time 字段已存在");
    } else {
        sqlx::query(
            "ALTER TABLE vehicles ADD COLUMN update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
        )
        .execute(pool)
        .await?;
        println!("✓ 已添加 vehicles.update_time 字段");
    }

    Ok(())
}

async fn fix_users_user_name(pool: &PgPool) -> Result<(), sqlx::Error> {
    if check_column_exists(pool, "users", "username").await? {
        println!("✓ 重命名 users.username 为 user_name");
        sqlx::query("ALTER TABLE users RENAME COLUMN username TO user_name")
            .execute(pool)
            .await?;
        return Ok(());
    }

    if check_column_exists(pool, "users", "user_name").await? {
        println!("✓ users.user_name 字段已存在");
    } else {
        sqlx::query("ALTER TABLE users ADD COLUMN user_name VARCHAR(50)")
            .execute(pool)
            .await?;
        println!("✓ 已添加 users.user_name 字段");
    }

    Ok(())
}

async fn create_indexes(pool: &PgPool) -> Result<(), sqlx::Error> {
    let indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_sensor_data_vehicle_time ON sensor_data(vehicle_id, collect_time)",
        "CREATE INDEX IF NOT EXISTS idx_sensor_data_type_time ON sensor_data(sensor_type, collect_time)",
        "CREATE INDEX IF NOT EXISTS idx_sensor_agg_vehicle_type ON sensor_data_aggregated(vehicle_id, sensor_type)",
        "CREATE INDEX IF NOT EXISTS idx_sensor_agg_time ON sensor_data_aggregated(start_time, end_time)",
    ];

    for idx in indexes {
        sqlx::query(idx).execute(pool).await?;
    }

    println!("✓ 已创建所有索引");
    Ok(())
}

async fn verify_fixes(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("\n=== 验证修复结果 ===");

    let checks = vec![
        ("sensor_data.status", "sensor_data", "status"),
        (
            "sensor_data_aggregated 表",
            "sensor_data_aggregated",
            NULL_COL,
        ),
        (
            "sensor_data_aggregated.updated_at",
            "sensor_data_aggregated",
            "updated_at",
        ),
        ("vehicles.update_time", "vehicles", "update_time"),
        ("users.user_name", "users", "user_name"),
    ];

    for (name, table, column) in checks {
        let exists = if column == NULL_COL {
            check_table_exists(pool, table).await?
        } else {
            check_column_exists(pool, table, column).await?
        };

        let status = if exists { "✓" } else { "✗" };
        println!("{} {}", status, name);
    }

    Ok(())
}

const NULL_COL: &str = "__null__";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 开始修复数据库 Schema ===\n");

    // 获取数据库连接
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:123@localhost:5432/tms_db".to_string());

    println!("连接数据库: {}", database_url);
    let pool = PgPool::connect(&database_url).await?;
    println!("✓ 数据库连接成功\n");

    // 执行修复
    println!("=== 1. 修复 sensor_data 表 ===");
    fix_sensor_data_status(&pool).await?;

    println!("\n=== 2. 修复 sensor_data_aggregated 表 ===");
    fix_sensor_data_aggregated(&pool).await?;

    println!("\n=== 3. 修复 vehicles 表 ===");
    fix_vehicles_update_time(&pool).await?;

    println!("\n=== 4. 修复 users 表 ===");
    fix_users_user_name(&pool).await?;

    println!("\n=== 5. 创建索引 ===");
    create_indexes(&pool).await?;

    // 验证
    verify_fixes(&pool).await?;

    println!("\n=== 数据库修复完成！ ===");
    Ok(())
}
