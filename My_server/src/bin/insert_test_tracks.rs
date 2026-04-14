use chrono::{Duration, Utc};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 连接到数据库
    let pool = PgPool::connect("postgres://postgres:123@localhost:5432/tms_db").await?;

    // 检查vehicles表中是否有车辆数据
    let vehicle_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM vehicles")
        .fetch_one(&pool)
        .await?;

    println!("vehicles表中的车辆数: {}", vehicle_count);

    // 获取所有车辆ID
    let vehicles =
        sqlx::query_scalar::<_, i32>("SELECT vehicle_id FROM vehicles ORDER BY vehicle_id")
            .fetch_all(&pool)
            .await?;

    if vehicles.is_empty() {
        println!("没有找到车辆数据,请先添加车辆");
        return Ok(());
    }

    // 打印找到的车辆ID
    println!("找到的车辆ID: {:?}", vehicles);

    // 检查orders表中是否有订单数据
    let order_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders")
        .fetch_one(&pool)
        .await?;

    println!("orders表中的订单数: {}", order_count);

    // 为每辆车创建一个订单
    for vehicle_id in &vehicles {
        // 检查是否已经有该车辆的订单
        let existing_order =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM orders WHERE vehicle_id = $1")
                .bind(vehicle_id)
                .fetch_one(&pool)
                .await?;

        if existing_order == 0 {
            // 为该车辆创建订单
            println!("为车辆ID {} 创建订单...", vehicle_id);

            let order_no = format!("TEST{:06}", vehicle_id);
            sqlx::query(r#"INSERT INTO orders (order_no, vehicle_id, customer_name, customer_phone, origin, destination, cargo_type, cargo_weight, cargo_volume, cargo_count, order_amount, order_status, create_user_id) VALUES ($1, $2, '测试客户', '13800138000', '北京', '上海', '货物', 10.0, 5.0, 100, 10000.0, 1, 1)"#)
                .bind(order_no)
                .bind(vehicle_id)
                .execute(&pool)
                .await?;
        }
    }

    // 检查logistics_tracks表中是否有轨迹数据
    let track_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM logistics_tracks")
        .fetch_one(&pool)
        .await?;

    println!("logistics_tracks表中的轨迹数: {}", track_count);

    // 如果没有轨迹数据,添加测试数据
    if track_count == 0 {
        println!("正在添加测试轨迹数据...");

        // 获取当前时间
        let now = Utc::now().naive_utc();

        // 为每辆车生成100个轨迹点
        for vehicle_id in &vehicles {
            // 获取该车辆的订单ID
            let order_id = sqlx::query_scalar::<_, i32>(
                "SELECT order_id FROM orders WHERE vehicle_id = $1 LIMIT 1",
            )
            .bind(vehicle_id)
            .fetch_one(&pool)
            .await?;

            // 生成随机路线(北京到上海的大致路线)
            let start_lat = 39.9042;
            let start_lng = 116.4074;
            let end_lat = 31.2304;
            let end_lng = 121.4737;

            // 生成100个轨迹点
            for i in 0..100 {
                let progress = i as f64 / 99.0;

                // 计算当前位置(线性插值)
                let lat = start_lat
                    + (end_lat - start_lat) * progress
                    + (rand::random::<f64>() - 0.5) * 0.1;
                let lng = start_lng
                    + (end_lng - start_lng) * progress
                    + (rand::random::<f64>() - 0.5) * 0.1;

                // 计算时间(均匀分布在过去24小时内)
                let track_time = now - Duration::hours(24) + Duration::minutes(i as i64 * 14);

                // 插入轨迹点
                sqlx::query(r#"INSERT INTO logistics_tracks (order_id, vehicle_id, track_time, latitude, longitude, status) VALUES ($1, $2, $3, $4, $5, 1)"#)
                    .bind(order_id)
                    .bind(vehicle_id)
                    .bind(track_time)
                    .bind(lat)
                    .bind(lng)
                    .execute(&pool)
                    .await?;
            }
        }

        println!("成功添加测试轨迹数据");
    } else {
        println!("logistics_tracks表中已有轨迹数据,跳过添加");
    }

    println!("\n数据检查和初始化完成！");

    Ok(())
}
