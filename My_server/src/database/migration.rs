//! 数据库迁移管理模块
//!
//! 使用 `sqlx::migrate!()` 自动化迁移框架。
//! 迁移文件位于 `<项目根>/migrations/`，格式：`YYYYMMDDHHMMSS_description.sql`
//!
//! ## 存量迁移文件重命名
//!
//! 旧式 `01_description.sql` 需重命名为 sqlx 兼容格式：
//! ```powershell
//! .\scripts\rename_migrations.ps1
//! ```
//! 重命名后首次运行会自动创建 `_sqlx_migrations` 表来追踪已应用的迁移。

use log::{error, info, warn};
use sqlx::PgPool;

/// 执行数据库迁移
///
/// 通过 `sqlx::migrate!()` 嵌入并应用 `/migrations/` 中尚未执行的 SQL。
/// 幂等安全：已应用的迁移不会重复执行。
///
/// 存量迁移文件若未重命名（如 `01_create.sql`），`sqlx::migrate!` 会在编译时报错，
/// 此时请运行 `scripts/rename_migrations.ps1` 修正文件名。
///
/// 迁移失败不会阻塞服务启动，但会记录严重告警。
pub async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    info!("开始执行数据库迁移检查...");

    // 迁移文件路径相对于 Cargo.toml 所在目录（My_server/）
    match sqlx::migrate!().run(pool).await {
        Ok(_) => {
            info!("数据库迁移检查完成");
            Ok(())
        }
        Err(e) => {
            let msg: String = e.to_string();
            error!("数据库迁移失败: {}", msg);
            if msg.contains("invalid migration filename") || msg.contains("migration filename") {
                warn!(
                    "迁移文件名不符合 sqlx 格式。请运行: `cd My_server && .\\scripts\\rename_migrations.ps1`"
                );
            }
            warn!("服务将以当前数据库状态继续运行，可能有 Schema 不匹配风险");
            Err(Box::new(e))
        }
    }
}

/// 验证数据库 Schema 完整性
///
/// 检查关键表是否存在，返回缺失的表名称列表
pub async fn verify_schema_integrity(pool: &PgPool, required_tables: &[&str]) -> Vec<String> {
    let mut missing = Vec::new();
    for table in required_tables {
        let exists: Result<bool, sqlx::Error> = sqlx::query_scalar(
            r#"SELECT EXISTS (
                SELECT FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = $1
            )"#,
        )
        .bind(table)
        .fetch_one(pool)
        .await;

        match exists {
            Ok(true) => {}
            _ => missing.push(table.to_string()),
        }
    }
    missing
}
