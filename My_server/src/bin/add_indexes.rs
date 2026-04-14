use sqlx::PgPool;

async fn add_indexes(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("=== 开始添加数据库索引 ===\n");

    // 1. 为vehicles表添加索引
    println!("1. 为vehicles表添加索引");
    let vehicle_indexes = [
        // 为plate_number添加唯一索引(已经在表定义中设置了UNIQUE)
        r#"CREATE INDEX IF NOT EXISTS idx_vehicles_status ON vehicles(status)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_vehicles_vehicle_type ON vehicles(vehicle_type)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_vehicles_create_time ON vehicles(create_time)"#,
    ];

    for (i, sql) in vehicle_indexes.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("  ✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("  ⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }

    // 2. 为orders表添加索引
    println!("\n2. 为orders表添加索引");
    let order_indexes = [
        // 为order_no添加唯一索引(已经在表定义中设置了UNIQUE)
        r#"CREATE INDEX IF NOT EXISTS idx_orders_vehicle_id ON orders(vehicle_id)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_orders_driver_id ON orders(driver_id)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_orders_order_status ON orders(order_status)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_orders_departure_time ON orders(departure_time)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_orders_arrival_time ON orders(arrival_time)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_orders_create_time ON orders(create_time)"#,
    ];

    for (i, sql) in order_indexes.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("  ✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("  ⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }

    // 3. 为users表添加索引
    println!("\n3. 为users表添加索引");
    let user_indexes = [
        // 为user_name添加唯一索引(已经在表定义中设置了UNIQUE)
        r#"CREATE INDEX IF NOT EXISTS idx_users_user_group_id ON users(user_group_id)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_users_phone ON users(phone)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_users_create_time ON users(create_time)"#,
    ];

    for (i, sql) in user_indexes.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("  ✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("  ⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }

    // 4. 为drivers表添加索引
    println!("\n4. 为drivers表添加索引");
    let driver_indexes = [
        // 为license_number添加唯一索引(已经在表定义中设置了UNIQUE)
        r#"CREATE INDEX IF NOT EXISTS idx_drivers_status ON drivers(status)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_drivers_phone_number ON drivers(phone_number)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_drivers_email ON drivers(email)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_drivers_create_time ON drivers(create_time)"#,
    ];

    for (i, sql) in driver_indexes.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("  ✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("  ⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }

    // 5. 为location_positions表添加索引
    println!("\n5. 为location_positions表添加索引");
    let location_indexes = [
        r#"CREATE INDEX IF NOT EXISTS idx_location_positions_latitude ON location_positions(latitude)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_location_positions_longitude ON location_positions(longitude)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_location_positions_created_at ON location_positions(created_at)"#,
    ];

    for (i, sql) in location_indexes.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("  ✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("  ⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }

    // 6. 为location_routes表添加索引
    println!("\n6. 为location_routes表添加索引");
    let route_indexes = [
        r#"CREATE INDEX IF NOT EXISTS idx_location_routes_start_latitude ON location_routes(start_latitude)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_location_routes_start_longitude ON location_routes(start_longitude)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_location_routes_end_latitude ON location_routes(end_latitude)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_location_routes_end_longitude ON location_routes(end_longitude)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_location_routes_created_at ON location_routes(created_at)"#,
    ];

    for (i, sql) in route_indexes.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("  ✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("  ⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }

    // 7. 为finance_costs表添加索引
    println!("\n7. 为finance_costs表添加索引");
    let finance_cost_indexes = [
        r#"CREATE INDEX IF NOT EXISTS idx_finance_costs_cost_type ON finance_costs(cost_type)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_finance_costs_cost_date ON finance_costs(cost_date)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_finance_costs_create_time ON finance_costs(create_time)"#,
    ];

    for (i, sql) in finance_cost_indexes.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("  ✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("  ⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }

    // 8. 为finance_invoices表添加索引
    println!("\n8. 为finance_invoices表添加索引");
    let finance_invoice_indexes = [
        // 为invoice_number添加唯一索引(已经在表定义中设置了UNIQUE)
        r#"CREATE INDEX IF NOT EXISTS idx_finance_invoices_invoice_date ON finance_invoices(invoice_date)"#,
        r#"CREATE INDEX IF NOT EXISTS idx_finance_invoices_create_time ON finance_invoices(create_time)"#,
    ];

    for (i, sql) in finance_invoice_indexes.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("  ✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("  ⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }

    println!("\n=== 数据库索引添加完成！ ===");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "postgres://postgres:123@localhost:5432/postgres?client_encoding=UTF8";

    println!("连接数据库: {}", database_url);
    let pool = PgPool::connect(database_url).await?;
    println!("✓ 数据库连接成功\n");

    add_indexes(&pool).await?;

    Ok(())
}
