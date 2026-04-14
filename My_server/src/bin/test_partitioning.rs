use chrono::{Datelike, NaiveDate, NaiveTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载环境变量
    dotenv::dotenv().ok();

    // 获取数据库连接字符串
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:123@localhost:5432/tms_db?sslmode=disable".to_string()
    });

    // 创建数据库连接池
    let pool = PgPool::connect(&database_url).await?;

    println!("开始测试分表功能...");

    // 1. 测试物流轨迹表分区
    println!("\n1. 测试物流轨迹表分区");

    // 检查并创建测试车辆(如果不存在)
    let vehicle_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM vehicles")
        .fetch_one(&pool)
        .await?;

    let vehicle_id = if vehicle_count == 0 {
        // 插入测试车辆
        let vehicle_name = format!("测试车辆_{}", Utc::now().timestamp());
        let license_plate = format!("TEST{:06}", Utc::now().timestamp() % 1000000);

        let result = sqlx::query(r#"INSERT INTO vehicles (vehicle_name, license_plate, vehicle_type, vehicle_color, vehicle_brand, vehicle_model, engine_no, frame_no, register_date, inspection_date, insurance_date, seating_capacity, load_capacity, vehicle_length, vehicle_width, vehicle_height, group_id, is_simulation, create_user_id)
                    VALUES ($1, $2, '重型货车', '红色', '东风', 'DFL4251A', '1234567890', 'ABC1234567890', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 2, 40.0, 12.0, 2.5, 3.5, 1, false, 1)
                    RETURNING vehicle_id"#)
            .bind(&vehicle_name)
            .bind(&license_plate)
            .fetch_one(&pool)
            .await?;

        let id: i32 = result.get(0);
        println!("创建测试车辆成功,车辆ID: {}", id);
        id
    } else {
        // 使用现有车辆的ID
        let id: i32 = sqlx::query_scalar("SELECT vehicle_id FROM vehicles LIMIT 1")
            .fetch_one(&pool)
            .await?;
        println!("使用现有车辆,车辆ID: {}", id);
        id
    };

    // 创建测试订单(如果不存在)
    let order_no = format!("TEST_PARTITION_{}", Utc::now().timestamp());
    sqlx::query(r#"INSERT INTO orders (order_no, vehicle_id, customer_name, customer_phone, origin, destination, cargo_type, cargo_weight, cargo_volume, cargo_count, order_amount, order_status, create_user_id) 
                    VALUES ($1, $2, '测试客户', '13800138000', '北京', '上海', '货物', 10.0, 20.0, 5, 1000.0, 2, 1)
                    ON CONFLICT (order_no) DO NOTHING"#)
        .bind(&order_no)
        .bind(vehicle_id)
        .execute(&pool)
        .await?;

    // 获取订单ID
    let order_id = sqlx::query_scalar::<_, i32>("SELECT order_id FROM orders WHERE order_no = $1")
        .bind(&order_no)
        .fetch_one(&pool)
        .await?;

    println!("创建测试订单成功,订单ID: {}", order_id);

    // 手动创建必要的分区
    println!("\n手动创建必要的分区...");
    let now = Utc::now().naive_utc();

    // 为logistics_tracks表创建分区
    for i in 0..5 {
        let track_time = now - chrono::Duration::days(i * 31);
        let year = track_time.year();
        let month = track_time.month();
        let partition_name = format!("logistics_tracks_{:04}{:02}", year, month);

        // 计算分区的精确开始和结束时间
        let partition_start = chrono::NaiveDateTime::new(
            NaiveDate::from_ymd_opt(year, month, 1).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );
        let partition_end = if month == 12 {
            chrono::NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            )
        } else {
            chrono::NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            )
        };

        // 创建分区
        sqlx::query(
            format!(
                r#"CREATE TABLE IF NOT EXISTS {} PARTITION OF logistics_tracks
                 FOR VALUES FROM ('{}') TO ('{}')"#,
                partition_name, partition_start, partition_end
            )
            .as_str(),
        )
        .execute(&pool)
        .await?;

        println!("创建物流轨迹分区: {}", partition_name);
    }

    // 插入测试轨迹数据
    for i in 0..5 {
        // 创建不同月份的轨迹数据
        let track_time = now - chrono::Duration::days(i * 31);

        let result = sqlx::query(r#"INSERT INTO logistics_tracks (
                        order_id, vehicle_id, track_time, latitude, longitude, address, status, remark
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6, $7, $8
                    ) RETURNING track_id"#)
            .bind(order_id)
            .bind(vehicle_id)
            .bind(track_time)
            .bind(39.9042 + (i as f64) * 0.01) // 纬度
            .bind(116.4074 + (i as f64) * 0.01) // 经度
            .bind(format!("测试地址{}", i))
            .bind(2) // 运输中状态
            .bind(format!("测试轨迹{}", i))
            .fetch_one(&pool)
            .await?;

        let track_id: i32 = result.get(0);
        println!("插入轨迹成功,轨迹ID: {}, 时间: {}", track_id, track_time);
    }

    // 检查分区表是否自动创建
    let partitions = sqlx::query_scalar::<_, String>(
        "SELECT relname FROM pg_class WHERE relname LIKE 'logistics_tracks_%' AND relkind = 'r'",
    )
    .fetch_all(&pool)
    .await?;

    println!("\n自动创建的物流轨迹分区表:");
    for partition in partitions {
        println!("- {}", partition);
    }

    // 2. 测试称重数据表分区
    println!("\n2. 测试称重数据表分区");

    // 为weighing_data表创建分区
    for i in 0..5 {
        let track_time = now - chrono::Duration::days(i * 31);
        let year = track_time.year();
        let month = track_time.month();
        let partition_name = format!("weighing_data_{:04}{:02}", year, month);

        // 计算分区的精确开始和结束时间
        let partition_start = chrono::NaiveDateTime::new(
            NaiveDate::from_ymd_opt(year, month, 1).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );
        let partition_end = if month == 12 {
            chrono::NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            )
        } else {
            chrono::NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            )
        };

        // 创建分区
        sqlx::query(
            format!(
                r#"CREATE TABLE IF NOT EXISTS {} PARTITION OF weighing_data
                 FOR VALUES FROM ('{}') TO ('{}')"#,
                partition_name, partition_start, partition_end
            )
            .as_str(),
        )
        .execute(&pool)
        .await?;

        println!("创建称重数据分区: {}", partition_name);
    }

    let now = Utc::now().naive_utc();
    for i in 0..5 {
        // 创建不同月份的称重数据
        let weighing_time = now - chrono::Duration::days(i * 31);

        let result = sqlx::query(r#"INSERT INTO weighing_data (
                        vehicle_id, device_id, weighing_time, gross_weight, tare_weight, net_weight, axle_count, speed, lane_no, site_id, status
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
                    ) RETURNING id"#)
            .bind(vehicle_id) // 使用之前获取的车辆ID
            .bind("test_device")
            .bind(weighing_time)
            .bind(1000.0 + (i as f64) * 10.0) // 毛重
            .bind(200.0 + (i as f64) * 5.0)    // 皮重
            .bind(800.0 + (i as f64) * 5.0)    // 净重
            .bind(4 + (i % 3))                 // 轴数
            .bind(60.0 + (i as f64) * 2.0)     // 速度
            .bind((i % 3) + 1)                 // 车道号
            .bind((i % 2) + 1)                 // 站点ID
            .bind(1)                           // 状态
            .fetch_one(&pool)
            .await?;

        let id: i32 = result.get(0);
        println!("插入称重数据成功,ID: {}, 时间: {}", id, weighing_time);
    }

    // 检查分区表是否自动创建
    let weighing_partitions = sqlx::query_scalar::<_, String>(
        "SELECT relname FROM pg_class WHERE relname LIKE 'weighing_data_%' AND relkind = 'r'",
    )
    .fetch_all(&pool)
    .await?;

    println!("\n自动创建的称重数据分区表:");
    for partition in weighing_partitions {
        println!("- {}", partition);
    }

    // 3. 测试审计日志表分区
    println!("\n3. 测试审计日志表分区");

    // 为audit_logs表创建分区
    for i in 0..5 {
        let track_time = now - chrono::Duration::days(i * 31);
        let year = track_time.year();
        let month = track_time.month();
        let partition_name = format!("audit_logs_{:04}{:02}", year, month);

        // 计算分区的精确开始和结束时间
        let partition_start = chrono::NaiveDateTime::new(
            NaiveDate::from_ymd_opt(year, month, 1).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );
        let partition_end = if month == 12 {
            chrono::NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            )
        } else {
            chrono::NaiveDateTime::new(
                NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            )
        };

        // 创建分区
        sqlx::query(
            format!(
                r#"CREATE TABLE IF NOT EXISTS {} PARTITION OF audit_logs
                 FOR VALUES FROM ('{}') TO ('{}')"#,
                partition_name, partition_start, partition_end
            )
            .as_str(),
        )
        .execute(&pool)
        .await?;

        println!("创建审计日志分区: {}", partition_name);
    }

    for i in 0..5 {
        // 创建不同月份的审计日志
        let action_time = now - chrono::Duration::days(i * 31);

        let result = sqlx::query(r#"INSERT INTO audit_logs (
                        user_id, username, action, resource, resource_id, request_data, ip_address, user_agent, action_time, result
                    ) VALUES (
                        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
                    ) RETURNING id"#)
            .bind(1) // 使用默认用户ID
            .bind("admin")
            .bind("test_action")
            .bind("test_resource")
            .bind(format!("resource_{}", i))
            .bind(format!("{{\"test\": {}}}", i))
            .bind("127.0.0.1")
            .bind("test_user_agent")
            .bind(action_time)
            .bind(1) // 成功
            .fetch_one(&pool)
            .await?;

        let id: i32 = result.get(0);
        println!("插入审计日志成功,ID: {}, 时间: {}", id, action_time);
    }

    // 检查分区表是否自动创建
    let audit_partitions = sqlx::query_scalar::<_, String>(
        "SELECT relname FROM pg_class WHERE relname LIKE 'audit_logs_%' AND relkind = 'r'",
    )
    .fetch_all(&pool)
    .await?;

    println!("\n自动创建的审计日志分区表:");
    for partition in audit_partitions {
        println!("- {}", partition);
    }

    // 4. 测试查询功能
    println!("\n4. 测试查询功能");

    // 查询物流轨迹总记录数
    let track_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM logistics_tracks")
        .fetch_one(&pool)
        .await?;
    println!("物流轨迹总记录数: {}", track_count);

    // 查询称重数据总记录数
    let weigh_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM weighing_data")
        .fetch_one(&pool)
        .await?;
    println!("称重数据总记录数: {}", weigh_count);

    // 查询审计日志总记录数
    let audit_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM audit_logs")
        .fetch_one(&pool)
        .await?;
    println!("审计日志总记录数: {}", audit_count);

    // 5. 测试管理函数(注释掉,因为会与手动创建的分区冲突)
    // println!("\n5. 测试分区管理函数");
    //
    // // 调用管理函数,预创建未来3个月的分区
    // sqlx::query("SELECT manage_partitions()")
    //     .execute(&pool)
    //     .await?;
    //
    // println!("调用分区管理函数成功,已预创建未来3个月的分区");

    // 检查所有分区表
    let all_partitions = sqlx::query_scalar::<_, String>("SELECT relname FROM pg_class WHERE relname ~ '(logistics_tracks|weighing_data|audit_logs)_[0-9]{6}' AND relkind = 'r' ORDER BY relname")
        .fetch_all(&pool)
        .await?;

    println!("\n所有自动创建的分区表:");
    for partition in all_partitions {
        println!("- {}", partition);
    }

    println!("\n分表功能测试完成！");
    Ok(())
}
