use sqlx::{PgPool, Row};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用正确的数据库连接字符串
    let database_url = "postgresql://postgres:123@localhost:5432/carptms_db?client_encoding=UTF8";

    println!("连接数据库: {}", database_url);

    // 创建连接池
    let pool = PgPool::connect(database_url).await?;

    // 查询所有用户
    let users = sqlx::query("SELECT user_id, user_name, user_group_id FROM users LIMIT 10")
        .fetch_all(&pool)
        .await?;

    println!("数据库中的用户:");
    println!("用户ID | 用户名 | 用户组ID");
    println!("--------|--------|----------");

    for user in users {
        println!(
            "{} | {} | {}",
            user.get::<i32, &str>("user_id"),
            user.get::<String, &str>("user_name"),
            user.get::<i32, &str>("user_group_id")
        );
    }

    Ok(())
}
