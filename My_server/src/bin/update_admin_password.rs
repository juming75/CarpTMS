use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用正确的数据库连接字符串
    let database_url = "postgresql://postgres:123@localhost:5432/carptms_db?client_encoding=UTF8";

    println!("连接数据库: {}", database_url);

    // 创建连接池
    let pool = PgPool::connect(database_url).await?;

    // 更新管理员密码
    let result = sqlx::query(
        "UPDATE users 
         SET password = $1, update_time = CURRENT_TIMESTAMP 
         WHERE user_name = 'admin'"
    )
    .bind("$argon2id$v=19$m=19456,t=2,p=1$KzpdADbbqsztDWwTTfGQvw$bkDAPzLQYB3ni86UPTZwg2UQqXiYDwj3oDQGvAmCNAg")
    .execute(&pool)
    .await?;

    println!("✓ 管理员密码更新成功！");
    println!("  用户名: admin");
    println!("  密码: admin123");
    println!("  影响行数: {}", result.rows_affected());

    Ok(())
}
