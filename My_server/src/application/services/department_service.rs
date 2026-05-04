//! 部门应用服务

use chrono::{NaiveDateTime, Utc};
use sqlx::Row;

use crate::domain::entities::department::{
    Department, DepartmentCreateRequest, DepartmentUpdateRequest,
};
use crate::errors::{AppError, AppResult};

/// 部门应用服务
pub struct DepartmentService {
    pool: sqlx::PgPool,
}

impl DepartmentService {
    /// 创建新部门服务
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// 获取部门列表
    pub async fn get_departments(
        &self,
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<Department>, i64)> {
        let offset = (page - 1) * page_size;

        // 查询总数
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM departments")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::db_error(&format!("Failed to count departments: {}", e), None)
            })?;

        // 查询列表
        let rows = sqlx::query(
            r#"SELECT d.*, 
               COALESCE(p.department_name, '') as parent_department_name, 
               COALESCE(u.real_name, '') as manager_name
        FROM departments d 
        LEFT JOIN departments p ON d.parent_department_id = p.department_id 
        LEFT JOIN users u ON d.manager_id = u.user_id 
        ORDER BY d.department_id 
        LIMIT $1 OFFSET $2"#,
        )
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to list departments: {}", e), None))?;

        let departments: Vec<Department> = rows
            .iter()
            .map(|r| {
                let create_time: NaiveDateTime = r.get("create_time");
                let update_time: Option<NaiveDateTime> = r.try_get("update_time").unwrap_or(None);
                Department {
                    department_id: r.get("department_id"),
                    department_name: r.get("department_name"),
                    parent_department_id: r.try_get("parent_department_id").unwrap_or(None),
                    parent_department_name: r.try_get("parent_department_name").ok(),
                    manager_id: r.try_get("manager_id").unwrap_or(None),
                    manager_name: r.try_get("manager_name").ok(),
                    phone: r.try_get("phone").unwrap_or(None),
                    description: r.try_get("description").unwrap_or(None),
                    create_time,
                    update_time,
                }
            })
            .collect();

        Ok((departments, total))
    }

    /// 获取部门详情
    pub async fn get_department(&self, department_id: i32) -> AppResult<Option<Department>> {
        let row = sqlx::query(
            r#"SELECT d.*, 
               COALESCE(p.department_name, '') as parent_department_name, 
               COALESCE(u.real_name, '') as manager_name
        FROM departments d 
        LEFT JOIN departments p ON d.parent_department_id = p.department_id 
        LEFT JOIN users u ON d.manager_id = u.user_id 
        WHERE d.department_id = $1"#,
        )
        .bind(department_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to get department: {}", e), None))?;

        Ok(row.map(|r| {
            let create_time: NaiveDateTime = r.get("create_time");
            let update_time: Option<NaiveDateTime> = r.try_get("update_time").unwrap_or(None);
            Department {
                department_id: r.get("department_id"),
                department_name: r.get("department_name"),
                parent_department_id: r.try_get("parent_department_id").unwrap_or(None),
                parent_department_name: r.try_get("parent_department_name").ok(),
                manager_id: r.try_get("manager_id").unwrap_or(None),
                manager_name: r.try_get("manager_name").ok(),
                phone: r.try_get("phone").unwrap_or(None),
                description: r.try_get("description").unwrap_or(None),
                create_time,
                update_time,
            }
        }))
    }

    /// 创建部门
    pub async fn create_department(
        &self,
        request: DepartmentCreateRequest,
    ) -> AppResult<Department> {
        // 检查父部门是否存在
        if let Some(parent_id) = request.parent_department_id {
            let exists: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM departments WHERE department_id = $1")
                    .bind(parent_id)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| {
                        AppError::db_error(&format!("Failed to check parent: {}", e), None)
                    })?;

            if exists == 0 {
                return Err(AppError::business_error(
                    "Parent department not found",
                    None,
                ));
            }
        }

        let row = sqlx::query(
            r#"INSERT INTO departments (department_name, parent_department_id, manager_id, phone, description, create_time, update_time) 
               VALUES ($1, $2, $3, $4, $5, $6, $6) 
               RETURNING *"#
        )
        .bind(&request.department_name)
        .bind(request.parent_department_id)
        .bind(request.manager_id)
        .bind(&request.phone)
        .bind(&request.description)
        .bind(Utc::now().naive_utc())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to create department: {}", e), None))?;

        let create_time: NaiveDateTime = row.get("create_time");
        let update_time: Option<NaiveDateTime> = row.try_get("update_time").unwrap_or(None);
        Ok(Department {
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            parent_department_id: row.try_get("parent_department_id").unwrap_or(None),
            parent_department_name: None,
            manager_id: row.try_get("manager_id").unwrap_or(None),
            manager_name: None,
            phone: row.try_get("phone").unwrap_or(None),
            description: row.try_get("description").unwrap_or(None),
            create_time,
            update_time,
        })
    }

    /// 更新部门
    pub async fn update_department(
        &self,
        department_id: i32,
        request: DepartmentUpdateRequest,
    ) -> AppResult<Department> {
        // 检查部门是否存在
        let exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM departments WHERE department_id = $1")
                .bind(department_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::db_error(&format!("Failed to check department: {}", e), None)
                })?;

        if exists == 0 {
            return Err(AppError::not_found_error(
                "Department not found".to_string(),
            ));
        }

        // 检查父部门是否存在且不是自己
        if let Some(parent_id) = request.parent_department_id {
            if parent_id == department_id {
                return Err(AppError::business_error(
                    "Department cannot be its own parent",
                    None,
                ));
            }
            let parent_exists: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM departments WHERE department_id = $1")
                    .bind(parent_id)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| {
                        AppError::db_error(&format!("Failed to check parent: {}", e), None)
                    })?;
            if parent_exists == 0 {
                return Err(AppError::business_error(
                    "Parent department not found",
                    None,
                ));
            }
        }

        let row = sqlx::query(
            r#"UPDATE departments 
               SET department_name = COALESCE($1, department_name), 
                   parent_department_id = COALESCE($2, parent_department_id), 
                   manager_id = COALESCE($3, manager_id), 
                   phone = COALESCE($4, phone), 
                   description = COALESCE($5, description), 
                   update_time = $6 
               WHERE department_id = $7 
               RETURNING *"#,
        )
        .bind(&request.department_name)
        .bind(request.parent_department_id)
        .bind(request.manager_id)
        .bind(&request.phone)
        .bind(&request.description)
        .bind(Utc::now().naive_utc())
        .bind(department_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to update department: {}", e), None))?;

        let create_time: NaiveDateTime = row.get("create_time");
        let update_time: Option<NaiveDateTime> = row.try_get("update_time").unwrap_or(None);
        Ok(Department {
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            parent_department_id: row.try_get("parent_department_id").unwrap_or(None),
            parent_department_name: None,
            manager_id: row.try_get("manager_id").unwrap_or(None),
            manager_name: None,
            phone: row.try_get("phone").unwrap_or(None),
            description: row.try_get("description").unwrap_or(None),
            create_time,
            update_time,
        })
    }

    /// 删除部门
    pub async fn delete_department(&self, department_id: i32) -> AppResult<()> {
        // 检查部门是否存在
        let exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM departments WHERE department_id = $1")
                .bind(department_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::db_error(&format!("Failed to check department: {}", e), None)
                })?;

        if exists == 0 {
            return Err(AppError::not_found_error(
                "Department not found".to_string(),
            ));
        }

        // 检查是否有子部门
        let child_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM departments WHERE parent_department_id = $1")
                .bind(department_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    AppError::db_error(&format!("Failed to check children: {}", e), None)
                })?;

        if child_count > 0 {
            return Err(AppError::business_error(
                "Cannot delete department with sub-departments",
                None,
            ));
        }

        sqlx::query("DELETE FROM departments WHERE department_id = $1")
            .bind(department_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::db_error(&format!("Failed to delete department: {}", e), None)
            })?;

        Ok(())
    }
}

use crate::application::ApplicationService;

#[async_trait::async_trait]
impl ApplicationService for DepartmentService {
    fn name(&self) -> &str {
        "DepartmentService"
    }

    async fn initialize(&self) -> AppResult<()> {
        Ok(())
    }
}
