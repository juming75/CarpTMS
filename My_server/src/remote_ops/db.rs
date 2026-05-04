//! 远程运维数据库操作

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::remote_ops::models::*;

fn err(msg: String) -> AppError {
    AppError::internal_error(&msg, None)
}

// ═══════════ 服务器 CRUD ═══════════

pub async fn create_server(
    pool: &PgPool,
    req: CreateServerRequest,
    user_id: Uuid,
    org_id: Uuid,
) -> Result<Server, AppError> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let port = req.port.unwrap_or(22);

    let server = sqlx::query_as::<_, Server>(
        r#"INSERT INTO remote_ops_servers (id, name, description, host, port, username, password, private_key, private_key_passphrase, group_id, tags, os_type, status, created_by, organization_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'offline', $13, $14, $15, $15)
        RETURNING id, name, description, host, port, username, password, private_key, private_key_passphrase,
                  status::TEXT as "status!: _", group_id, tags, os_type, last_connected_at, created_at, updated_at, created_by, organization_id"#
    )
    .bind(id).bind(&req.name).bind(&req.description).bind(&req.host).bind(port)
    .bind(&req.username).bind(&req.password).bind(&req.private_key).bind(&req.private_key_passphrase)
    .bind(&req.group_id).bind(&req.tags).bind(&req.os_type)
    .bind(user_id).bind(org_id).bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| err(format!("创建服务器失败: {}", e)))?;

    Ok(server)
}

pub async fn list_servers(
    pool: &PgPool,
    query: ServerListQuery,
    org_id: Uuid,
) -> Result<PaginatedResponse<ServerResponse>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).max(1).min(100);
    let offset = (page - 1) * page_size;

    let servers = sqlx::query_as::<_, Server>(
        r#"SELECT id, name, description, host, port, username, password, private_key, private_key_passphrase,
                  status::TEXT as "status!: _", group_id, tags, os_type, last_connected_at, created_at, updated_at, created_by, organization_id
         FROM remote_ops_servers
         WHERE organization_id = $1
           AND ($2::TEXT IS NULL OR name ILIKE '%' || $2 || '%' OR host ILIKE '%' || $2 || '%')
           AND ($3::TEXT IS NULL OR status::TEXT = $3)
           AND ($4::UUID IS NULL OR group_id = $4)
         ORDER BY created_at DESC
         LIMIT $5 OFFSET $6"#
    )
    .bind(org_id)
    .bind(&query.keyword)
    .bind(&query.status)
    .bind(query.group_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| err(format!("查询服务器列表失败: {}", e)))?;

    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM remote_ops_servers
         WHERE organization_id = $1
           AND ($2::TEXT IS NULL OR name ILIKE '%' || $2 || '%' OR host ILIKE '%' || $2 || '%')
           AND ($3::TEXT IS NULL OR status::TEXT = $3)
           AND ($4::UUID IS NULL OR group_id = $4)"#
    )
    .bind(org_id)
    .bind(&query.keyword)
    .bind(&query.status)
    .bind(query.group_id)
    .fetch_one(pool)
    .await
    .map_err(|e| err(format!("查询服务器总数失败: {}", e)))?;

    let items: Vec<ServerResponse> = servers.into_iter().map(ServerResponse::from).collect();

    Ok(PaginatedResponse {
        total: total.0,
        page,
        page_size,
        total_pages: (total.0 as f64 / page_size as f64).ceil() as i64,
        items,
    })
}

pub async fn get_server(pool: &PgPool, id: Uuid) -> Result<Option<Server>, AppError> {
    let result = sqlx::query_as::<_, Server>(
        r#"SELECT id, name, description, host, port, username, password, private_key, private_key_passphrase,
                  status::TEXT as "status!: _", group_id, tags, os_type, last_connected_at, created_at, updated_at, created_by, organization_id
         FROM remote_ops_servers WHERE id = $1"#
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| err(format!("查询服务器失败: {}", e)))?;
    Ok(result)
}

pub async fn update_server(
    pool: &PgPool,
    id: Uuid,
    req: UpdateServerRequest,
) -> Result<Option<Server>, AppError> {
    let existing = get_server(pool, id).await?;
    let s = match existing {
        Some(s) => s,
        None => return Ok(None),
    };

    let name = req.name.unwrap_or(s.name);
    let host = req.host.unwrap_or(s.host);
    let port = req.port.unwrap_or(s.port);
    let username = req.username.unwrap_or(s.username);

    let result = sqlx::query_as::<_, Server>(
        r#"UPDATE remote_ops_servers SET
            name = $2, description = $3, host = $4, port = $5, username = $6,
            password = $7, private_key = $8, private_key_passphrase = $9,
            group_id = $10, tags = $11, os_type = $12, updated_at = NOW()
         WHERE id = $1
         RETURNING id, name, description, host, port, username, password, private_key, private_key_passphrase,
                   status::TEXT as "status!: _", group_id, tags, os_type, last_connected_at, created_at, updated_at, created_by, organization_id"#
    )
    .bind(id).bind(&name).bind(&req.description).bind(&host).bind(port)
    .bind(&username).bind(&req.password).bind(&req.private_key).bind(&req.private_key_passphrase)
    .bind(&req.group_id).bind(&req.tags).bind(&req.os_type)
    .fetch_optional(pool)
    .await
    .map_err(|e| err(format!("更新服务器失败: {}", e)))?;

    Ok(result)
}

pub async fn delete_server(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM remote_ops_servers WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| err(format!("删除服务器失败: {}", e)))?;
    Ok(())
}

// ═══════════ 服务器组 CRUD ═══════════

pub async fn list_groups(pool: &PgPool, org_id: Uuid) -> Result<Vec<ServerGroup>, AppError> {
    let groups = sqlx::query_as::<_, ServerGroup>(
        r#"SELECT g.id, g.name, g.description, g.parent_id,
                  (SELECT COUNT(*) FROM remote_ops_servers s WHERE s.group_id = g.id) as server_count,
                  g.created_at, g.updated_at
         FROM remote_ops_server_groups g
         WHERE g.organization_id = $1 ORDER BY g.name"#
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
    .map_err(|e| err(format!("查询服务器组失败: {}", e)))?;
    Ok(groups)
}

pub async fn create_group(
    pool: &PgPool,
    req: CreateServerGroupRequest,
    org_id: Uuid,
) -> Result<ServerGroup, AppError> {
    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let group = sqlx::query_as::<_, ServerGroup>(
        r#"INSERT INTO remote_ops_server_groups (id, name, description, parent_id, organization_id, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $6)
         RETURNING id, name, description, parent_id, 0 as server_count, created_at, updated_at"#
    )
    .bind(id).bind(&req.name).bind(&req.description).bind(&req.parent_id).bind(org_id).bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| err(format!("创建服务器组失败: {}", e)))?;

    Ok(group)
}

pub async fn update_group(
    pool: &PgPool,
    id: Uuid,
    req: UpdateServerGroupRequest,
) -> Result<Option<ServerGroup>, AppError> {
    let result = sqlx::query_as::<_, ServerGroup>(
        r#"UPDATE remote_ops_server_groups SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            parent_id = COALESCE($4, parent_id),
            updated_at = NOW()
         WHERE id = $1
         RETURNING id, name, description, parent_id,
                   (SELECT COUNT(*) FROM remote_ops_servers s WHERE s.group_id = $1) as server_count,
                   created_at, updated_at"#
    )
    .bind(id).bind(&req.name).bind(&req.description).bind(&req.parent_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| err(format!("更新服务器组失败: {}", e)))?;
    Ok(result)
}

pub async fn delete_group(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE remote_ops_servers SET group_id = NULL WHERE group_id = $1")
        .bind(id).execute(pool).await.ok();
    sqlx::query("DELETE FROM remote_ops_server_groups WHERE id = $1")
        .bind(id).execute(pool).await
        .map_err(|e| err(format!("删除服务器组失败: {}", e)))?;
    Ok(())
}

pub async fn get_command_history(pool: &PgPool, _server_id: Uuid) -> Result<Vec<CommandResult>, AppError> {
    Ok(Vec::new())
}
