use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 加载环境变量
    dotenv::dotenv().ok();

    // 获取数据库URL
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // 创建数据库连接池
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("=== Checking users table structure ===");

    // 查询表结构
    type ColumnRow = (String, String, String, Option<String>);
    let rows = sqlx::query_as::<_, ColumnRow>(r#"
        SELECT column_name, data_type, is_nullable, column_default
        FROM information_schema.columns
        WHERE table_name = 'users'
        ORDER BY ordinal_position
    "#)
    .fetch_all(&pool)
    .await?;

    println!("Columns in users table:");
    for (column_name, data_type, is_nullable, column_default) in rows {
        println!("- {}: {} (Nullable: {}, Default: {:?})
",
                 column_name,
                 data_type,
                 is_nullable,
                 column_default);
    }

    // 检查user_groups表
    println!("=== Checking user_groups table structure ===");
    let group_rows = sqlx::query_as::<_, ColumnRow>(r#"
        SELECT column_name, data_type, is_nullable, column_default
        FROM information_schema.columns
        WHERE table_name = 'user_groups'
        ORDER BY ordinal_position
    "#)
    .fetch_all(&pool)
    .await?;
    
    println!("Columns in user_groups table:");
    for row in group_rows {
        println!("- {}: {} (Nullable: {}, Default: {:?})
", 
                 row.column_name, 
                 row.data_type, 
                 row.is_nullable, 
                 row.column_default);
    }
    
    Ok(())
}




