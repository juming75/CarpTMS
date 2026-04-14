use sqlx::{PgPool, Row};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用正确的数据库连接字符串
    let database_url = "postgresql://postgres:123@localhost:5432/carptms_db?client_encoding=UTF8";

    println!("连接数据库: {}", database_url);

    // 创建连接池
    let pool = PgPool::connect(database_url).await?;

    // 查询用户组
    let user_groups = sqlx::query("SELECT group_id, group_name FROM user_groups")
        .fetch_all(&pool)
        .await?;

    println!("数据库中的用户组:");
    println!("用户组ID | 用户组名称");
    println!("----------|------------");

    for group in user_groups {
        println!(
            "{} | {}",
            group.get::<i32, &str>("group_id"),
            group.get::<String, &str>("group_name")
        );
    }

    Ok(())
}
