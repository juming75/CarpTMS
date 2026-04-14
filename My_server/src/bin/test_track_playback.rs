use chrono::{NaiveDateTime, Utc};
use rand::Rng;
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 设置日志
    tracing_subscriber::fmt::init();

    // 获取数据库连接字符串
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:123@localhost:5432/tms_db?sslmode=disable".to_string()
    });

    info!("Connecting to database: {}", database_url);

    // 连接到数据库
    let pool = PgPool::connect(&database_url).await?;
    info!("Successfully connected to database");

    // 创建测试数据
    let vehicle_id = 1;
    let order_id = 1;

    // 生成测试轨迹点
    info!("Generating test track points...");
    generate_test_track_points(&pool, vehicle_id, order_id).await?;

    // 生成轨迹线
    info!("Generating track lines from track points...");
    generate_track_lines(&pool, vehicle_id, order_id).await?;

    // 测试轨迹查询性能
    info!("Testing track query performance...");
    test_track_query_performance(&pool, vehicle_id).await?;

    info!("Track playback performance test completed successfully!");
    Ok(())
}

// 生成测试轨迹点
async fn generate_test_track_points(
    pool: &PgPool,
    vehicle_id: i32,
    order_id: i32,
) -> Result<(), sqlx::Error> {
    // 删除旧的测试数据
    sqlx::query(r#"DELETE FROM logistics_tracks WHERE vehicle_id = $1 AND order_id = $2"#)
        .bind(vehicle_id)
        .bind(order_id)
        .execute(pool)
        .await?;

    // 生成1000个轨迹点,模拟2000秒(约33分钟)的行驶
    let mut rng = rand::thread_rng();
    let start_time = Utc::now().naive_utc() - chrono::Duration::seconds(2000);
    let mut latitude = 39.9042;
    let mut longitude = 116.4074;

    for i in 0..1000 {
        // 计算当前时间
        let track_time = start_time + chrono::Duration::seconds(i * 2);

        // 模拟车辆移动,每次移动0.0005度左右
        latitude += (rng.gen_range(-0.0008..0.0008)) * 0.5;
        longitude += (rng.gen_range(-0.0008..0.0008)) * 0.5;

        // 添加一些随机噪声
        latitude += rng.gen_range(-0.0001..0.0001);
        longitude += rng.gen_range(-0.0001..0.0001);

        // 插入轨迹点
        sqlx::query(
            r#"INSERT INTO logistics_tracks 
            (order_id, vehicle_id, track_time, latitude, longitude, address, status) 
            VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(order_id)
        .bind(vehicle_id)
        .bind(track_time)
        .bind(latitude)
        .bind(longitude)
        .bind(format!("Test address {}", i))
        .bind(1)
        .execute(pool)
        .await?;

        if i % 100 == 0 {
            info!("Generated {} track points", i);
        }
    }

    info!("Generated 1000 track points successfully");
    Ok(())
}

// 生成轨迹线
async fn generate_track_lines(
    pool: &PgPool,
    vehicle_id: i32,
    order_id: i32,
) -> Result<(), sqlx::Error> {
    // 删除旧的轨迹线数据
    sqlx::query(r#"DELETE FROM logistics_track_lines WHERE vehicle_id = $1 AND order_id = $2"#)
        .bind(vehicle_id)
        .bind(order_id)
        .execute(pool)
        .await?;

    // 使用PostGIS生成轨迹线
    let result = sqlx::query(r#" 
        INSERT INTO logistics_track_lines (vehicle_id, order_id, start_time, end_time, geom, point_count, distance, status) 
        SELECT 
            $1 as vehicle_id,
            $2 as order_id,
            MIN(track_time) as start_time,
            MAX(track_time) as end_time,
            ST_MakeLine(geom ORDER BY track_time) as geom,
            COUNT(*) as point_count,
            ST_Length(ST_MakeLine(geom ORDER BY track_time)::geography) as distance,
            1 as status
        FROM logistics_tracks 
        WHERE vehicle_id = $1 AND order_id = $2
    "#)
    .bind(vehicle_id)
    .bind(order_id)
    .execute(pool)
    .await?;

    info!("Generated {} track lines", result.rows_affected());
    Ok(())
}

// 测试轨迹查询性能
async fn test_track_query_performance(pool: &PgPool, vehicle_id: i32) -> Result<(), sqlx::Error> {
    use std::time::Instant;

    // 获取查询时间范围
    let row = sqlx::query(
        r#" 
        SELECT MIN(track_time) as start_time, MAX(track_time) as end_time 
        FROM logistics_tracks 
        WHERE vehicle_id = $1
    "#,
    )
    .bind(vehicle_id)
    .fetch_one(pool)
    .await?;

    let start_time: NaiveDateTime = row.try_get_unchecked(0)?;
    let end_time: NaiveDateTime = row.try_get_unchecked(1)?;

    // 测试1:查询所有轨迹点(传统方式)
    info!("Testing traditional track point query...");
    let start = Instant::now();
    let rows = sqlx::query(
        r#" 
        SELECT * FROM logistics_tracks 
        WHERE vehicle_id = $1 AND track_time BETWEEN $2 AND $3 
        ORDER BY track_time
    "#,
    )
    .bind(vehicle_id)
    .bind(start_time)
    .bind(end_time)
    .fetch_all(pool)
    .await?;
    let duration = start.elapsed();
    info!("Traditional query: {} rows in {:?}", rows.len(), duration);

    // 测试2:查询轨迹线(优化方式)
    info!("Testing optimized track line query...");
    let start = Instant::now();
    let line_rows = sqlx::query(
        r#" 
        SELECT * FROM logistics_track_lines 
        WHERE vehicle_id = $1
    "#,
    )
    .bind(vehicle_id)
    .fetch_all(pool)
    .await?;
    let duration = start.elapsed();
    info!(
        "Optimized query: {} rows in {:?}",
        line_rows.len(),
        duration
    );

    // 测试3:查询特定时间段的轨迹点
    info!("Testing specific time range track point query...");
    let mid_time = start_time + (end_time - start_time) / 2;
    let start = Instant::now();
    let range_rows = sqlx::query(
        r#" 
        SELECT * FROM logistics_tracks 
        WHERE vehicle_id = $1 AND track_time BETWEEN $2 AND $3 
        ORDER BY track_time
    "#,
    )
    .bind(vehicle_id)
    .bind(start_time)
    .bind(mid_time)
    .fetch_all(pool)
    .await?;
    let duration = start.elapsed();
    info!(
        "Specific range query: {} rows in {:?}",
        range_rows.len(),
        duration
    );

    Ok(())
}
