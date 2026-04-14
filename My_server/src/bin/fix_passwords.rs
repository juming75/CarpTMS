//! / 密码修复工具
// 用于执行数据库密码更新和验证

use sqlx::{postgres::PgPool, Row};
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    println!("==============================================");
    println!("   CarpTMS 密码修复工具");
    println!("==============================================\n");

    // 获取数据库连接字符串
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:123@localhost:5432/carptms_db".to_string());

    println!("连接数据库: {}", database_url);
    println!();

    // 连接数据库
    let pool = match PgPool::connect(&database_url).await {
        Ok(pool) => {
            println!("✓ 数据库连接成功\n");
            pool
        }
        Err(e) => {
            println!("✗ 数据库连接失败: {}", e);
            println!("请确保PostgreSQL数据库服务已启动,并且连接配置正确");
            println!("默认连接配置: postgresql://postgres:123@localhost:5432/carptms_db");
            println!("您可以通过设置DATABASE_URL环境变量来修改连接配置");
            return Err(e);
        }
    };

    // 第一步:检查当前密码状态
    println!("=== 第一步:检查当前密码状态 ===");
    let row = sqlx::query(
        r#"
        SELECT
            COUNT(*) as total_users,
            SUM(CASE WHEN password LIKE '$argon2id$%' THEN 1 ELSE 0 END) as encrypted_users,
            SUM(CASE WHEN password NOT LIKE '$argon2id$%' THEN 1 ELSE 0 END) as plaintext_users
        FROM users
        "#,
    )
    .fetch_one(&pool)
    .await?;

    let total_users: i64 = row.get("total_users");
    let encrypted_users: i64 = row.get::<Option<i64>, _>("encrypted_users").unwrap_or(0);
    let plaintext_users: i64 = row.get::<Option<i64>, _>("plaintext_users").unwrap_or(0);

    println!("总用户数: {}", total_users);
    println!("已加密用户: {}", encrypted_users);
    println!("明文密码用户: {}", plaintext_users);
    println!();

    if plaintext_users == 0 {
        println!("✓ 所有密码已经是Argon2加密格式,无需修复\n");
        println!("==============================================");
        println!("修复完成！");
        println!("==============================================");
        return Ok(());
    }

    // 第二步:创建备份
    println!("=== 第二步:创建备份 ===");
    sqlx::query("DROP TABLE IF EXISTS users_backup_20260317")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE TABLE users_backup_20260317 AS SELECT * FROM users")
        .execute(&pool)
        .await?;
    println!("✓ 已创建备份表 users_backup_20260317\n");

    // 第三步:更新明文密码
    println!("=== 第三步:更新明文密码为Argon2哈希 ===\n");

    // 定义密码映射
    let password_mappings = vec![
        ("admin", "$argon2id$v=19$m=19456,t=2,p=1$Ne5P0SURb5RPBbGtEc6opw$fVIF2d6MrHLXn4t71m3KitDin3/8YTJ7ZZsYvEEXw7w"),
        ("manager", "$argon2id$v=19$m=19456,t=2,p=1$ZQkx7iF1Mj76x0ybhCoOBQ$X51qF28Nx+V+BIGE0YyIeX7MtHi/SCIFs8meEtidkYI"),
        ("user", "$argon2id$v=19$m=19456,t=2,p=1$Akn4MJE5HDESPtRBAW4Mwg$9WaxjlOxWFpsUpeSFu88zc2Blrye8oNh9IZ/AXveIpY"),
    ];

    // 更新特定用户
    for (username, password_hash) in password_mappings {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET password = $1, update_time = CURRENT_TIMESTAMP
            WHERE user_name = $2 AND password NOT LIKE '$argon2id$%'
            "#,
        )
        .bind(password_hash)
        .bind(username)
        .execute(&pool)
        .await?;

        if result.rows_affected() > 0 {
            println!("✓ 已更新 {} 密码(默认密码: {})", username, username);
        }
    }

    // 更新所有其他明文密码用户
    let temp_password = "$argon2id$v=19$m=19456,t=2,p=1$qMX22dfzWiTBmJrESrfE2w$Oc3yU3E8Nfn6sTOltygEhB1NJAKSuUjlq0STKWzhPmY";
    let result = sqlx::query(
        r#"
        UPDATE users
        SET password = $1, update_time = CURRENT_TIMESTAMP
        WHERE password NOT LIKE '$argon2id$%'
        "#,
    )
    .bind(temp_password)
    .execute(&pool)
    .await?;

    if result.rows_affected() > 0 {
        println!("✓ 已批量更新其他所有明文密码用户(临时密码: 123456)");
    }

    println!();

    // 第四步:验证修复结果
    println!("=== 第四步:验证修复结果 ===");
    let users = sqlx::query(
        r#"
        SELECT user_name, password, LENGTH(password) as hash_length
        FROM users
        ORDER BY user_name
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let mut all_fixed = true;
    for user in &users {
        let user_name: String = user.get("user_name");
        let password: String = user.get("password");
        let hash_length: i32 = user.get("hash_length");

        let status = if password.starts_with("$argon2id$") {
            "✓ 已修复"
        } else {
            all_fixed = false;
            "✗ 仍然需要修复"
        };
        println!("  {}: {} (哈希长度: {})", user_name, status, hash_length);
    }

    println!();

    // 第五步:显示默认密码对照表
    println!("=== 第五步:默认密码对照表 ===");
    println!(
        "  {:<15} | {:<20} | {:<20}",
        "用户名", "默认密码", "注意事项"
    );
    println!("  {}|{}|{}", "-".repeat(15), "-".repeat(20), "-".repeat(20));

    for user in &users {
        let user_name: String = user.get("user_name");
        let default_password = match user_name.as_str() {
            "admin" => "admin",
            "manager" => "manager",
            "user" => "user",
            _ => "123456(临时密码)",
        };
        let note = match user_name.as_str() {
            "admin" | "manager" | "user" => "⚠️ 建议修改",
            _ => "⚠️ 首次登录后请立即修改",
        };
        println!(
            "  {:<15} | {:<20} | {:<20}",
            user_name, default_password, note
        );
    }

    println!();
    println!("==============================================");
    if all_fixed {
        println!("✓ 所有密码已成功修复！");
    } else {
        println!("⚠️ 部分密码未修复,请手动检查");
    }
    println!("==============================================");
    println!();
    println!("下一步:");
    println!("  1. 启动后端服务: cd My_server && cargo run");
    println!("  2. 启动前端服务: cd My_client && npm run dev");
    println!("  3. 使用上述密码登录系统");
    println!("  4. 登录后立即修改密码");

    Ok(())
}
