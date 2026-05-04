use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载 .env 文件
    dotenv().ok();

    // 强制使用正确的数据库连接
    let database_url = "postgres://postgres:123@localhost:5432/CarpTMS_db".to_string();

    tracing::info!(database = %database_url, "数据库连接");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&database_url)
        .await?;

    tracing::info!("数据库连接成功");

    // 执行数据库修复脚本
    let drivers_sql = include_str!("../../migrations/20260101000034_fix_drivers_table.sql");
    let comprehensive_sql =
        include_str!("../../migrations/20260101000035_comprehensive_db_fix.sql");
    let group_templates_sql =
        include_str!("../../migrations/20260101000036_create_group_templates.sql");

    tracing::info!("开始执行数据库修复脚本...");
    tracing::info!("=== 修复 drivers 表 ===");
    execute_sql_batch(drivers_sql, &pool).await;

    tracing::info!("=== 修复综合数据库表 ===");
    execute_sql_batch(comprehensive_sql, &pool).await;

    tracing::info!("=== 创建组织模板表 ===");
    execute_sql_batch(group_templates_sql, &pool).await;

    tracing::info!("数据库修复脚本执行完成");

    pool.close().await;
    Ok(())
}

async fn execute_sql_batch(sql: &str, pool: &sqlx::PgPool) {
    // 分割SQL语句并逐条执行
    let statements: Vec<&str> = sql.split(';').collect();
    let mut executed = 0;
    let mut errors = 0;

    for statement in statements {
        let stmt = statement.trim();
        if stmt.is_empty() || stmt.starts_with("--") || stmt.starts_with("SELECT") {
            continue;
        }

        match sqlx::query(stmt).execute(pool).await {
            Ok(_) => {
                executed += 1;
            }
            Err(e) => {
                // 忽略"已存在"的错误
                if e.to_string().contains("already exists")
                    || e.to_string().contains("duplicate key")
                    || (e.to_string().contains("column") && e.to_string().contains("exists"))
                {
                    tracing::warn!(
                        stmt = %&stmt[..30.min(stmt.len())],
                        "已跳过（已存在）"
                    );
                } else {
                    tracing::error!(error = %e, "SQL执行错误");
                    errors += 1;
                }
            }
        }
    }

    tracing::info!(executed = executed, errors = errors, "SQL批处理完成");
}
