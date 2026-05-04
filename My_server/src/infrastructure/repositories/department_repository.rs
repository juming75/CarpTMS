//! 部门仓库实现

use std::sync::Arc;

use anyhow::Context;
use sqlx::{postgres::PgRow, PgPool, Row};

use crate::domain::entities::department::{
    Department, DepartmentCreate, DepartmentQuery, DepartmentUpdate,
};
use crate::domain::use_cases::department::DepartmentRepository;

/// 部门仓库实现
#[derive(Clone)]
pub struct DepartmentRepositoryImpl {
    db: Arc<PgPool>,
}

impl DepartmentRepositoryImpl {
    /// 创建部门仓库实例
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl DepartmentRepository for DepartmentRepositoryImpl {
    async fn get_departments(
        &self,
        query: DepartmentQuery,
    ) -> Result<(Vec<Department>, i64), anyhow::Error> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 查询总数
        let count_query = "SELECT COUNT(*) FROM departments";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(&*self.db)
            .await
            .context("Failed to count departments")?;

        // 查询数据
        let list_query = r#"SELECT d.*, 
                   COALESCE(p.department_name, '') as parent_department_name, 
                   COALESCE(u.real_name, '') as manager_name
        FROM departments d 
        LEFT JOIN departments p ON d.parent_department_id = p.department_id 
        LEFT JOIN users u ON d.manager_id = u.user_id 
        ORDER BY d.department_id 
        LIMIT $1 OFFSET $2"#;

        let departments = sqlx::query(list_query)
            .bind(page_size)
            .bind(offset)
            .map(|row: PgRow| Department {
                department_id: row.get("department_id"),
                department_name: row.get("department_name"),
                parent_department_id: row.get("parent_department_id"),
                parent_department_name: row.get("parent_department_name"),
                manager_id: row.get("manager_id"),
                manager_name: row.get("manager_name"),
                phone: row.get("phone"),
                description: row.get("description"),
                create_time: row.get("create_time"),
                update_time: row.get("update_time"),
            })
            .fetch_all(&*self.db)
            .await
            .context("Failed to fetch departments")?;

        Ok((departments, total))
    }

    async fn get_department(
        &self,
        department_id: i32,
    ) -> Result<Option<Department>, anyhow::Error> {
        let query = r#"SELECT d.*, 
                   COALESCE(p.department_name, '') as parent_department_name, 
                   COALESCE(u.real_name, '') as manager_name
        FROM departments d 
        LEFT JOIN departments p ON d.parent_department_id = p.department_id 
        LEFT JOIN users u ON d.manager_id = u.user_id 
        WHERE d.department_id = $1"#;

        let department = sqlx::query(query)
            .bind(department_id)
            .map(|row: PgRow| Department {
                department_id: row.get("department_id"),
                department_name: row.get("department_name"),
                parent_department_id: row.get("parent_department_id"),
                parent_department_name: row.get("parent_department_name"),
                manager_id: row.get("manager_id"),
                manager_name: row.get("manager_name"),
                phone: row.get("phone"),
                description: row.get("description"),
                create_time: row.get("create_time"),
                update_time: row.get("update_time"),
            })
            .fetch_optional(&*self.db)
            .await
            .context("Failed to fetch department")?;

        Ok(department)
    }

    async fn create_department(
        &self,
        department: DepartmentCreate,
    ) -> Result<Department, anyhow::Error> {
        let now = chrono::Utc::now().naive_utc();

        let created_department = sqlx::query(
            r#"INSERT INTO departments (department_name, parent_department_id, manager_id, phone, description, create_time, update_time) 
            VALUES ($1, $2, $3, $4, $5, $6, $6) 
            RETURNING department_id, department_name, parent_department_id, manager_id, phone, description, create_time, update_time"#
        )
        .bind(&department.department_name)
        .bind(department.parent_department_id)
        .bind(department.manager_id)
        .bind(&department.phone)
        .bind(&department.description)
        .bind(now)
        .fetch_one(&*self.db)
        .await
        .context("Failed to create department")?;

        // 获取父部门名称和经理名称
        let parent_name_query =
            r#"SELECT department_name FROM departments WHERE department_id = $1"#;
        let parent_department_name: Option<String> = if let Some(parent_id) =
            created_department.get::<Option<i32>, _>("parent_department_id")
        {
            sqlx::query_scalar(parent_name_query)
                .bind(parent_id)
                .fetch_optional(&*self.db)
                .await?
        } else {
            None
        };

        let manager_name_query = r#"SELECT real_name FROM users WHERE user_id = $1"#;
        let manager_name: Option<String> =
            if let Some(manager_id) = created_department.get::<Option<i32>, _>("manager_id") {
                sqlx::query_scalar(manager_name_query)
                    .bind(manager_id)
                    .fetch_optional(&*self.db)
                    .await?
            } else {
                None
            };

        Ok(Department {
            department_id: created_department.get("department_id"),
            department_name: created_department.get("department_name"),
            parent_department_id: created_department.get("parent_department_id"),
            parent_department_name,
            manager_id: created_department.get("manager_id"),
            manager_name,
            phone: created_department.get("phone"),
            description: created_department.get("description"),
            create_time: created_department.get("create_time"),
            update_time: created_department.get("update_time"),
        })
    }

    async fn update_department(
        &self,
        department_id: i32,
        department: DepartmentUpdate,
    ) -> Result<Option<Department>, anyhow::Error> {
        let now = chrono::Utc::now().naive_utc();

        let update_query = r#"UPDATE departments 
                          SET department_name = COALESCE($1, department_name), 
                              parent_department_id = COALESCE($2, parent_department_id), 
                              manager_id = COALESCE($3, manager_id), 
                              phone = COALESCE($4, phone), 
                              description = COALESCE($5, description), 
                              update_time = $6 
                          WHERE department_id = $7 
                          RETURNING department_id, department_name, parent_department_id, manager_id, phone, description, create_time, update_time"#;

        let updated_department = sqlx::query(update_query)
            .bind(&department.department_name)
            .bind(department.parent_department_id)
            .bind(department.manager_id)
            .bind(&department.phone)
            .bind(&department.description)
            .bind(now)
            .bind(department_id)
            .fetch_optional(&*self.db)
            .await
            .context("Failed to update department")?;

        match updated_department {
            Some(row) => {
                // 获取父部门名称和经理名称
                let parent_name_query =
                    r#"SELECT department_name FROM departments WHERE department_id = $1"#;
                let parent_department_name: Option<String> =
                    if let Some(parent_id) = row.get::<Option<i32>, _>("parent_department_id") {
                        sqlx::query_scalar(parent_name_query)
                            .bind(parent_id)
                            .fetch_optional(&*self.db)
                            .await?
                    } else {
                        None
                    };

                let manager_name_query = r#"SELECT real_name FROM users WHERE user_id = $1"#;
                let manager_name: Option<String> =
                    if let Some(manager_id) = row.get::<Option<i32>, _>("manager_id") {
                        sqlx::query_scalar(manager_name_query)
                            .bind(manager_id)
                            .fetch_optional(&*self.db)
                            .await?
                    } else {
                        None
                    };

                Ok(Some(Department {
                    department_id: row.get("department_id"),
                    department_name: row.get("department_name"),
                    parent_department_id: row.get("parent_department_id"),
                    parent_department_name,
                    manager_id: row.get("manager_id"),
                    manager_name,
                    phone: row.get("phone"),
                    description: row.get("description"),
                    create_time: row.get("create_time"),
                    update_time: row.get("update_time"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn delete_department(&self, department_id: i32) -> Result<bool, anyhow::Error> {
        let result = sqlx::query("DELETE FROM departments WHERE department_id = $1")
            .bind(department_id)
            .execute(&*self.db)
            .await
            .context("Failed to delete department")?;

        Ok(result.rows_affected() > 0)
    }

    async fn has_sub_departments(&self, department_id: i32) -> Result<bool, anyhow::Error> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM departments WHERE parent_department_id = $1")
                .bind(department_id)
                .fetch_one(&*self.db)
                .await
                .context("Failed to check sub-departments")?;

        Ok(count > 0)
    }
}
