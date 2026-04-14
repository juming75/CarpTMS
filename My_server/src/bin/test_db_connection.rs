use sqlx::{PgPool, Row};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用正确的数据库连接字符串
    let database_url = "postgresql://postgres:123@localhost:5432/carptms_db?client_encoding=UTF8";

    println!("连接数据库: {}", database_url);

    // 创建连接池
    let pool = PgPool::connect(database_url).await?;

    println!("数据库连接成功！");
    println!("连接池大小: {}", pool.size());
    println!("空闲连接数: {}", pool.num_idle());

    // 测试查询用户表
    println!("\n测试查询用户表...");
    let users = sqlx::query("SELECT user_id, user_name, user_group_id FROM users")
        .fetch_all(&pool)
        .await?;

    println!("数据库中的用户：");
    println!("用户ID | 用户名 | 用户组ID");
    println!("--------|--------|----------");
    
    for user in users {
        println!("{} | {} | {}", user.get::<i32, &str>("user_id"), user.get::<String, &str>("user_name"), user.get::<i32, &str>("user_group_id"));
    }

    // 测试查询用户组表
    println!("\n测试查询用户组表...");
    let user_groups = sqlx::query("SELECT group_id, group_name FROM user_groups")
        .fetch_all(&pool)
        .await?;

    println!("数据库中的用户组：");
    println!("用户组ID | 用户组名称");
    println!("----------|------------");
    
    for group in user_groups {
        println!("{} | {}", group.get::<i32, &str>("group_id"), group.get::<String, &str>("group_name"));
    }

    // 测试查询车辆表
    println!("\n测试查询车辆表...");
    let vehicles = sqlx::query("SELECT vehicle_id, vehicle_name, license_plate FROM vehicles LIMIT 5")
        .fetch_all(&pool)
        .await?;

    println!("数据库中的车辆（前5条）：");
    println!("车辆ID | 车辆名称 | 车牌号");
    println!("--------|----------|--------");
    
    for vehicle in vehicles {
        println!("{} | {} | {}", vehicle.get::<i32, &str>("vehicle_id"), vehicle.get::<String, &str>("vehicle_name"), vehicle.get::<String, &str>("license_plate"));
    }

    // 测试查询用户表结构
    println!("\n测试查询用户表结构...");
    let user_columns = sqlx::query("SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'users'")
        .fetch_all(&pool)
        .await?;

    println!("用户表字段：");
    println!("字段名 | 数据类型");
    println!("--------|----------");
    
    for column in user_columns {
        println!("{} | {}", column.get::<String, &str>("column_name"), column.get::<String, &str>("data_type"));
    }

    // 测试查询车辆表结构
    println!("\n测试查询车辆表结构...");
    let vehicle_columns = sqlx::query("SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'vehicles'")
        .fetch_all(&pool)
        .await?;

    println!("车辆表字段：");
    println!("字段名 | 数据类型");
    println!("--------|----------");
    
    for column in vehicle_columns {
        println!("{} | {}", column.get::<String, &str>("column_name"), column.get::<String, &str>("data_type"));
    }

    Ok(())
}
