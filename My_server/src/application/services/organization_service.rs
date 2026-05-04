use chrono::NaiveDateTime;
use sqlx::Row;

use crate::domain::entities::organization::{
    Organization, OrganizationCreate, OrganizationQuery, OrganizationUpdate,
};
use crate::errors::{AppError, AppResult};

pub struct OrganizationService {
    pool: sqlx::PgPool,
}

impl OrganizationService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_organizations(
        &self,
        query: OrganizationQuery,
    ) -> AppResult<(Vec<Organization>, i64)> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        let mut conditions = vec![];
        let mut params: Vec<String> = Vec::new();

        if let Some(name) = &query.name {
            if !name.is_empty() {
                let safe_name = name.chars().take(100).collect::<String>();
                conditions.push(format!("name LIKE ${}", params.len() + 1));
                params.push(format!("%{}%", safe_name));
            }
        }

        if let Some(org_type) = &query.r#type {
            if !org_type.is_empty() {
                let safe_type = org_type.chars().take(50).collect::<String>();
                conditions.push(format!("type = ${}", params.len() + 1));
                params.push(safe_type);
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM organizations {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for param in &params {
            count_query = count_query.bind(param);
        }
        let total = count_query.fetch_one(&self.pool).await.map_err(|e| {
            AppError::db_error(&format!("Failed to count organizations: {}", e), None)
        })?;

        let data_sql = format!(
            "SELECT * FROM organizations {} ORDER BY create_time DESC LIMIT {} OFFSET {}",
            where_clause, page_size, offset
        );
        let mut data_query = sqlx::query(&data_sql);
        for param in &params {
            data_query = data_query.bind(param);
        }

        let rows = data_query.fetch_all(&self.pool).await.map_err(|e| {
            AppError::db_error(&format!("Failed to list organizations: {}", e), None)
        })?;

        let items: Vec<Organization> = rows
            .iter()
            .map(|r| {
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
                    create_time,
                    update_time,
                }
            })
            .collect();

        Ok((items, total))
    }

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
                create_time,
                update_time,
            }
        }))
    }

    pub async fn create_organization(
        &self,
        request: OrganizationCreate,
    ) -> AppResult<Organization> {
        let existing: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM organizations WHERE unit_id = $1")
                .bind(&request.unit_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::db_error(&format!("Failed to check unit_id: {}", e), None)
                })?;

        if existing > 0 {
            return Err(AppError::business_error(
                "Organization unit ID already exists",
                None,
            ));
        }

        let now = chrono::Local::now().naive_utc();
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
            create_time,
            update_time,
        })
    }

    pub async fn update_organization(
        &self,
        unit_id: &str,
        request: OrganizationUpdate,
    ) -> AppResult<Organization> {
        let existing: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM organizations WHERE unit_id = $1")
                .bind(unit_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::db_error(&format!("Failed to check organization: {}", e), None)
                })?;

        if existing == 0 {
            return Err(AppError::not_found_error(
                "Organization not found".to_string(),
            ));
        }

        let now = chrono::Local::now().naive_utc();
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
               RETURNING *"#,
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
            create_time,
            update_time,
        })
    }

    pub async fn delete_organization(&self, unit_id: &str) -> AppResult<()> {
        let existing: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM organizations WHERE unit_id = $1")
                .bind(unit_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::db_error(&format!("Failed to check organization: {}", e), None)
                })?;

        if existing == 0 {
            return Err(AppError::not_found_error(
                "Organization not found".to_string(),
            ));
        }

        sqlx::query("DELETE FROM organizations WHERE unit_id = $1")
            .bind(unit_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::db_error(&format!("Failed to delete organization: {}", e), None)
            })?;

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
