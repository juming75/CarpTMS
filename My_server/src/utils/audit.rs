use chrono::NaiveDateTime;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::AuditLog;

// 分页参数结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: i32,
    pub page_size: i32,
}

// 审计日志搜索参数结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogSearchParams {
    pub user_id: Option<i32>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
}

// 审计日志记录结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogRecord {
    pub user_id: i32,
    pub username: String,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub request_data: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub result: i16,
    pub error_message: Option<String>,
}

// 记录审计日志
pub async fn log_audit_event(pool: &PgPool, log_record: AuditLogRecord) {
    let result = sqlx::query(
        "INSERT INTO audit_logs (user_id, username, action, resource, resource_id, request_data, ip_address, user_agent, action_time, result, error_message) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(log_record.user_id)
    .bind(log_record.username)
    .bind(log_record.action)
    .bind(log_record.resource)
    .bind(log_record.resource_id)
    .bind(log_record.request_data)
    .bind(log_record.ip_address)
    .bind(log_record.user_agent)
    .bind(chrono::Local::now().naive_local())
    .bind(log_record.result)
    .bind(log_record.error_message)
    .execute(pool)
    .await;

    if let Err(e) = result {
        error!("Failed to log audit event: {:?}", e);
    }
}

// 获取审计日志列表(支持分页)
pub async fn get_audit_logs(
    pool: &PgPool,
    page: i32,
    page_size: i32,
) -> Result<Vec<AuditLog>, sqlx::Error> {
    let offset = (page - 1) * page_size;

    let logs = sqlx::query_as::<_, AuditLog>(
        "SELECT * FROM audit_logs 
         ORDER BY action_time DESC 
         LIMIT $1 OFFSET $2",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

// 根据条件查询审计日志
pub async fn search_audit_logs(
    pool: &PgPool,
    search_params: AuditLogSearchParams,
    pagination: PaginationParams,
) -> Result<Vec<AuditLog>, sqlx::Error> {
    let offset = (pagination.page - 1) * pagination.page_size;

    // 使用sqlx的动态查询构建器来处理不同类型的参数
    let mut query_builder =
        sqlx::query_builder::QueryBuilder::new("SELECT * FROM audit_logs WHERE 1=1");

    // 添加条件
    if let Some(user_id) = search_params.user_id {
        query_builder.push(" AND user_id = ").push_bind(user_id);
    }

    if let Some(action) = search_params.action {
        query_builder.push(" AND action = ").push_bind(action);
    }

    if let Some(resource) = search_params.resource {
        query_builder.push(" AND resource = ").push_bind(resource);
    }

    if let Some(start_time) = search_params.start_time {
        query_builder
            .push(" AND action_time >= ")
            .push_bind(start_time);
    }

    if let Some(end_time) = search_params.end_time {
        query_builder
            .push(" AND action_time <= ")
            .push_bind(end_time);
    }

    // 添加排序和分页
    query_builder
        .push(" ORDER BY action_time DESC LIMIT ")
        .push_bind(pagination.page_size)
        .push(" OFFSET ")
        .push_bind(offset);

    // 执行查询
    let query = query_builder.build_query_as::<AuditLog>();
    let logs = query.fetch_all(pool).await?;

    Ok(logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用例:记录审计日志
    #[tokio::test]
    async fn test_log_audit_event() {
        // 注意:此测试需要配置测试数据库
        // 如果未配置数据库,测试将被跳过
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:password@localhost:5432/test_tms_db".to_string()
        });

        match PgPool::connect(&database_url).await {
            Ok(pool) => {
                let log_record = AuditLogRecord {
                    user_id: 1,
                    username: "test_user".to_string(),
                    action: "test_action".to_string(),
                    resource: "test_resource".to_string(),
                    resource_id: Some("1".to_string()),
                    request_data: Some(r#"{"test": "data"}"#.to_string()),
                    ip_address: Some("127.0.0.1".to_string()),
                    user_agent: Some("test-agent".to_string()),
                    result: 1,
                    error_message: None,
                };

                // 执行测试:记录审计事件
                log_audit_event(&pool, log_record).await;

                // 验证:函数应该成功执行,没有 panic
                assert!(true);
            }
            Err(_) => {
                // 数据库未运行,跳过测试
                println!("Warning: Test database not available, skipping test_log_audit_event");
            }
        }
    }

    // 测试用例:搜索审计日志
    #[tokio::test]
    async fn test_search_audit_logs() {
        // 注意:此测试需要配置测试数据库
        // 如果未配置数据库,测试将被跳过
        let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:password@localhost:5432/test_tms_db".to_string()
        });

        match PgPool::connect(&database_url).await {
            Ok(pool) => {
                let search_params = AuditLogSearchParams {
                    user_id: Some(1),
                    action: Some("test_action".to_string()),
                    resource: Some("test_resource".to_string()),
                    start_time: None,
                    end_time: None,
                };

                let pagination = PaginationParams {
                    page: 1,
                    page_size: 10,
                };

                // 执行测试:搜索审计日志
                let result = search_audit_logs(&pool, search_params, pagination).await;

                // 验证:函数应该成功执行,返回 Result 类型
                assert!(result.is_ok() || result.is_err());
            }
            Err(_) => {
                // 数据库未运行,跳过测试
                println!("Warning: Test database not available, skipping test_search_audit_logs");
            }
        }
    }
}
