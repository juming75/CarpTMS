//! / Truck Scale 模块测试

mod vehicle_handler_test;
mod user_handler_test;
mod service_integration_test;
mod db_test;

pub fn setup_test_db() -> anyhow::Result<sqlx::PgPool> {
    // 这里可以设置测试数据库连接
    // 暂时返回一个默认的连接池
    let db_url = std::env::var("DATABASE_URL").unwrap_or("postgres://postgres:postgres@localhost:5432/tms_test".to_string());
    let pool = sqlx::PgPool::connect(&db_url).await?;
    Ok(pool)
}






