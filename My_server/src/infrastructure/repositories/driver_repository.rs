//! 司机仓库实现

use std::sync::Arc;

use crate::domain::entities::driver::{Driver, DriverCreateRequest, DriverUpdateRequest, DriverQuery};
use crate::domain::use_cases::driver::DriverRepository;
use sqlx::{PgPool, Row};

/// 司机仓库实现
#[derive(Clone)]
pub struct PgDriverRepository {
    pool: Arc<PgPool>,
}

impl PgDriverRepository {
    /// 创建司机仓库实例
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DriverRepository for PgDriverRepository {
    async fn find_all(&self, page: i32, page_size: i32, query: DriverQuery) -> Result<(Vec<Driver>, i64), anyhow::Error> {
        let mut sql = String::from("SELECT * FROM drivers WHERE 1=1");
        let mut count_sql = String::from("SELECT COUNT(*) FROM drivers WHERE 1=1");
        let mut params = Vec::new();

        // 构建查询条件
        if let Some(driver_name) = &query.driver_name {
            if !driver_name.is_empty() {
                sql.push_str(" AND driver_name ILIKE $" );
                sql.push_str(&((params.len() + 1).to_string()));
                sql.push('\'');
                count_sql.push_str(" AND driver_name ILIKE $" );
                count_sql.push_str(&((params.len() + 1).to_string()));
                count_sql.push('\'');
                params.push(format!("%{}%", driver_name));
            }
        }

        if let Some(license_number) = &query.license_number {
            if !license_number.is_empty() {
                sql.push_str(" AND license_number ILIKE $" );
                sql.push_str(&((params.len() + 1).to_string()));
                sql.push('\'');
                count_sql.push_str(" AND license_number ILIKE $" );
                count_sql.push_str(&((params.len() + 1).to_string()));
                count_sql.push('\'');
                params.push(format!("%{}%", license_number));
            }
        }

        if let Some(status) = query.status {
            if status >= 0 {
                sql.push_str(" AND status = $" );
                sql.push_str(&((params.len() + 1).to_string()));
                count_sql.push_str(" AND status = $" );
                count_sql.push_str(&((params.len() + 1).to_string()));
                params.push(status.to_string());
            }
        }

        // 排序和分页
        sql.push_str(" ORDER BY create_time DESC");
        sql.push_str(" LIMIT $" );
        sql.push_str(&((params.len() + 1).to_string()));
        sql.push_str(" OFFSET $" );
        sql.push_str(&((params.len() + 2).to_string()));
        params.push(page_size.to_string());
        params.push(((page - 1) * page_size).to_string());

        // 执行查询
        let mut query = sqlx::query(&sql);
        
        // 绑定参数
        for param in &params[..params.len()-2] {
            query = query.bind(param);
        }
        
        let rows = query
            .bind(params[params.len()-2].parse::<i32>()?)
            .bind(params[params.len()-1].parse::<i32>()?)
            .fetch_all(&*self.pool)
            .await?;

        let drivers = rows
            .into_iter()
            .map(|row| -> Result<Driver, anyhow::Error> {
                Ok(Driver {
                    driver_id: row.try_get("driver_id")?,
                    driver_name: row.try_get("driver_name")?,
                    license_number: row.try_get("license_number")?,
                    phone_number: row.try_get("phone_number")?,
                    email: row.try_get("email")?,
                    status: row.try_get("status")?,
                    create_time: row.try_get("create_time")?,
                    update_time: row.try_get("update_time")?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // 执行计数查询
        let mut count_query = sqlx::query_scalar(&count_sql);
        
        // 绑定参数
        for param in &params[..params.len()-2] {
            count_query = count_query.bind(param);
        }
        
        let count = count_query
            .fetch_one(&*self.pool)
            .await?;

        Ok((drivers, count))
    }

    async fn find_by_id(&self, driver_id: i32) -> Result<Option<Driver>, anyhow::Error> {
        let driver = sqlx::query_as::<_, Driver>(
            "SELECT * FROM drivers WHERE driver_id = $1"
        )
        .bind(driver_id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(driver)
    }

    async fn create(&self, driver: DriverCreateRequest) -> Result<Driver, anyhow::Error> {
        let new_driver = sqlx::query_as::<_, Driver>(
            "INSERT INTO drivers (driver_name, license_number, phone_number, email, status, create_time)
             VALUES ($1, $2, $3, $4, $5, NOW())
             RETURNING *"
        )
        .bind(driver.driver_name)
        .bind(driver.license_number)
        .bind(driver.phone_number)
        .bind(driver.email)
        .bind(driver.status)
        .fetch_one(&*self.pool)
        .await?;

        Ok(new_driver)
    }

    async fn update(&self, driver_id: i32, driver: DriverUpdateRequest) -> Result<Driver, anyhow::Error> {
        let updated_driver = sqlx::query_as::<_, Driver>(
            "UPDATE drivers
             SET driver_name = COALESCE($1, driver_name),
                 license_number = COALESCE($2, license_number),
                 phone_number = COALESCE($3, phone_number),
                 email = COALESCE($4, email),
                 status = COALESCE($5, status),
                 update_time = NOW()
             WHERE driver_id = $6
             RETURNING *"
        )
        .bind(driver.driver_name)
        .bind(driver.license_number)
        .bind(driver.phone_number)
        .bind(driver.email)
        .bind(driver.status)
        .bind(driver_id)
        .fetch_one(&*self.pool)
        .await?;

        Ok(updated_driver)
    }

    async fn delete(&self, driver_id: i32) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM drivers WHERE driver_id = $1")
            .bind(driver_id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    async fn has_related_data(&self, driver_id: i32) -> Result<bool, anyhow::Error> {
        // 检查是否有关联的车辆
        let vehicle_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vehicles WHERE driver_id = $1"
        )
        .bind(driver_id)
        .fetch_one(&*self.pool)
        .await?;

        if vehicle_count > 0 {
            return Ok(true);
        }

        // 检查是否有关联的订单
        let order_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM orders WHERE driver_id = $1"
        )
        .bind(driver_id)
        .fetch_one(&*self.pool)
        .await?;

        Ok(order_count > 0)
    }

    async fn exists(&self, driver_id: i32) -> Result<bool, anyhow::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM drivers WHERE driver_id = $1"
        )
        .bind(driver_id)
        .fetch_one(&*self.pool)
        .await?;

        Ok(count > 0)
    }

    async fn count_by_name(&self, name: &str, exclude_id: Option<i32>) -> Result<i64, anyhow::Error> {
        let count: i64 = if let Some(id) = exclude_id {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM drivers WHERE driver_name = $1 AND driver_id != $2"
            )
            .bind(name)
            .bind(id)
            .fetch_one(&*self.pool)
            .await?
        } else {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM drivers WHERE driver_name = $1"
            )
            .bind(name)
            .fetch_one(&*self.pool)
            .await?
        };

        Ok(count)
    }

    async fn count_vehicles(&self, driver_id: i32) -> Result<i64, anyhow::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vehicles WHERE driver_id = $1"
        )
        .bind(driver_id)
        .fetch_one(&*self.pool)
        .await?;

        Ok(count)
    }

    async fn count_orders(&self, driver_id: i32) -> Result<i64, anyhow::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM orders WHERE driver_id = $1"
        )
        .bind(driver_id)
        .fetch_one(&*self.pool)
        .await?;

        Ok(count)
    }
}
