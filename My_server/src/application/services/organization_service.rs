//! 组织单位应用服务

use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::Row;

use crate::domain::entities::organization::{Organization, OrganizationCreate, OrganizationUpdate, OrganizationQuery};
use crate::errors::{AppError, AppResult};

/// 组织单位应用服务
pub struct OrganizationService {
    pool: sqlx::PgPool,
}

/// 将 NaiveDateTime 转换为 DateTime<Utc>
fn to_utc(dt: NaiveDateTime) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(dt, Utc)
}

impl OrganizationService {
    /// 创建新组织单位服务
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取组织单位列表
    pub async fn get_organizations(&self, query: OrganizationQuery) -> AppResult<(Vec<Organization>, i64)> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 构建查询条件
        let mut conditions = vec![];
        let _param_index = 1;

        if let Some(name) = &query.name {
            if !name.is_empty() {
                conditions.push(format!("name LIKE '%{}%'", name.replace("'", "''")));
            }
        }

        if let Some(r#type) = &query.r#type {
            if !r#type.is_empty() {
                conditions.push(format!("type = '{}'", r#type.replace("'", "''")));
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) FROM organizations {}", where_clause);
        let total: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to count organizations: {}", e), None))?;

        // 查询列表
        let data_sql = format!(
            "SELECT * FROM organizations {} ORDER BY create_time DESC LIMIT {} OFFSET {}",
            where_clause, page_size, offset
        );

        let rows = sqlx::query(&data_sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to list organizations: {}", e), None))?;

        let items: Vec<Organization> = rows.iter().map(|r| {
            let create_time: NaiveDateTime = r.get("create_time");
            let update_time: Option<NaiveDateTime> = r.try_get("update_time").unwrap_or(None);
            Organization {
                unit_id: r.get("unit_id"),
                name: r.get("name"),
                r#type: r.get("type"),
                parent_id: r.try_get("parent_id").unwrap_or(None),
                description: r.try_get("description").unwrap_or(None),
                contact_person: r.try_get("contact_person").unwrap_or(None),
                contact_phone: r.try_get("contact_phone").unwrap_or(None),
                status: r.get("status"),
                create_time: to_utc(create_time),
                update_time: update_time.map(to_utc),
            }
        }).collect();

        Ok((items, total))
    }

    /// 获取组织单位详情
    pub async fn get_organization(&self, unit_id: &str) -> AppResult<Option<Organization>> {
        let row = sqlx::query("SELECT * FROM organizations WHERE unit_id = $1")
            .bind(unit_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to get organization: {}", e), None))?;

        Ok(row.map(|r| {
            let create_time: NaiveDateTime = r.get("create_time");
            let update_time: Option<NaiveDateTime> = r.try_get("update_time").unwrap_or(None);
            Organization {
                unit_id: r.get("unit_id"),
                name: r.get("name"),
                r#type: r.get("type"),
                parent_id: r.try_get("parent_id").unwrap_or(None),
                description: r.try_get("description").unwrap_or(None),
                contact_person: r.try_get("contact_person").unwrap_or(None),
                contact_phone: r.try_get("contact_phone").unwrap_or(None),
                status: r.get("status"),
                create_time: to_utc(create_time),
                update_time: update_time.map(to_utc),
            }
        }))
    }

    /// 创建组织单位
    pub async fn create_organization(&self, request: OrganizationCreate) -> AppResult<Organization> {
        // 检查 unit_id 是否已存在
        let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organizations WHERE unit_id = $1")
            .bind(&request.unit_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to check unit_id: {}", e), None))?;

        if existing > 0 {
            return Err(AppError::business_error("Organization unit ID already exists", None));
        }

        let now = Utc::now().naive_utc();
        let row = sqlx::query(
            r#"INSERT INTO organizations (unit_id, name, type, parent_id, description, contact_person, contact_phone, status, create_time, update_time) 
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9) 
               RETURNING *"#
        )
        .bind(&request.unit_id)
        .bind(&request.name)
        .bind(&request.r#type)
        .bind(request.parent_id)
        .bind(&request.description)
        .bind(&request.contact_person)
        .bind(&request.contact_phone)
        .bind(request.status.as_deref().unwrap_or("active"))
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to create organization: {}", e), None))?;

        let create_time: NaiveDateTime = row.get("create_time");
        let update_time: Option<NaiveDateTime> = row.try_get("update_time").unwrap_or(None);
        Ok(Organization {
            unit_id: row.get("unit_id"),
            name: row.get("name"),
            r#type: row.get("type"),
            parent_id: row.try_get("parent_id").unwrap_or(None),
            description: row.try_get("description").unwrap_or(None),
            contact_person: row.try_get("contact_person").unwrap_or(None),
            contact_phone: row.try_get("contact_phone").unwrap_or(None),
            status: row.get("status"),
            create_time: to_utc(create_time),
            update_time: update_time.map(to_utc),
        })
    }

    /// 更新组织单位
    pub async fn update_organization(&self, unit_id: &str, request: OrganizationUpdate) -> AppResult<Organization> {
        // 检查是否存在
        let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organizations WHERE unit_id = $1")
            .bind(unit_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to check organization: {}", e), None))?;

        if existing == 0 {
            return Err(AppError::not_found_error("Organization not found".to_string()));
        }

        let now = Utc::now().naive_utc();
        let row = sqlx::query(
            r#"UPDATE organizations 
               SET name = COALESCE($1, name), 
                   type = COALESCE($2, type), 
                   parent_id = $3, 
                   description = COALESCE($4, description), 
                   contact_person = COALESCE($5, contact_person), 
                   contact_phone = COALESCE($6, contact_phone), 
                   update_time = $7 
               WHERE unit_id = $8 
               RETURNING *"#
        )
        .bind(&request.name)
        .bind(&request.r#type)
        .bind(request.parent_id)
        .bind(&request.description)
        .bind(&request.contact_person)
        .bind(&request.contact_phone)
        .bind(now)
        .bind(unit_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to update organization: {}", e), None))?;

        let create_time: NaiveDateTime = row.get("create_time");
        let update_time: Option<NaiveDateTime> = row.try_get("update_time").unwrap_or(None);
        Ok(Organization {
            unit_id: row.get("unit_id"),
            name: row.get("name"),
            r#type: row.get("type"),
            parent_id: row.try_get("parent_id").unwrap_or(None),
            description: row.try_get("description").unwrap_or(None),
            contact_person: row.try_get("contact_person").unwrap_or(None),
            contact_phone: row.try_get("contact_phone").unwrap_or(None),
            status: row.get("status"),
            create_time: to_utc(create_time),
            update_time: update_time.map(to_utc),
        })
    }

    /// 删除组织单位
    pub async fn delete_organization(&self, unit_id: &str) -> AppResult<()> {
        // 检查是否存在
        let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organizations WHERE unit_id = $1")
            .bind(unit_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to check organization: {}", e), None))?;

        if existing == 0 {
            return Err(AppError::not_found_error("Organization not found".to_string()));
        }

        sqlx::query("DELETE FROM organizations WHERE unit_id = $1")
            .bind(unit_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to delete organization: {}", e), None))?;

        Ok(())
    }
}

use crate::application::ApplicationService;

#[async_trait::async_trait]
impl ApplicationService for OrganizationService {
    fn name(&self) -> &str {
        "OrganizationService"
    }

    async fn initialize(&self) -> AppResult<()> {
        Ok(())
    }
}
