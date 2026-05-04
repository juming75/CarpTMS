//! 司机仓库实现

use std::sync::Arc;

use crate::domain::entities::driver::{
    Driver, DriverCreateRequest, DriverQuery, DriverUpdateRequest,
};
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
    async fn find_all(
        &self,
        page: i32,
        page_size: i32,
        query: DriverQuery,
    ) -> Result<(Vec<Driver>, i64), anyhow::Error> {
        let mut conditions: Vec<String> = Vec::new();
        let mut name_param: Option<String> = None;
        let mut license_param: Option<String> = None;
        let mut status_param: Option<i32> = None;
        let mut param_count = 0;

        if let Some(ref driver_name) = query.driver_name {
            if !driver_name.is_empty() {
                param_count += 1;
                conditions.push(format!("driver_name ILIKE ${}", param_count));
                name_param = Some(format!("%{}%", driver_name));
            }
        }

        if let Some(ref license_number) = query.license_number {
            if !license_number.is_empty() {
                param_count += 1;
                conditions.push(format!("license_number ILIKE ${}", param_count));
                license_param = Some(format!("%{}%", license_number));
            }
        }

        if let Some(status) = query.status {
            if status >= 0 {
                param_count += 1;
                conditions.push(format!("status = ${}", param_count));
                status_param = Some(status);
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 计数查询
        let count_sql = format!("SELECT COUNT(*) FROM drivers {}", where_clause);
        let total: i64 = {
            let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
            if let Some(ref p) = name_param {
                count_q = count_q.bind(p);
            }
            if let Some(ref p) = license_param {
                count_q = count_q.bind(p);
            }
            if let Some(s) = status_param {
                count_q = count_q.bind(s);
            }
            count_q
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to count drivers: {}", e))?
        };

        // 数据查询
        let param_count_for_limit = param_count;
        let data_sql = format!(
            "SELECT driver_id, driver_name, license_number, phone_number, email, status, create_time, update_time, 
                    license_no, license_type, license_expiry, id_card, address, emergency_contact, emergency_phone, hire_date
             FROM drivers {} ORDER BY create_time DESC LIMIT ${} OFFSET ${}",
            where_clause,
            param_count_for_limit + 1,
            param_count_for_limit + 2,
        );
        let drivers: Vec<Driver> = {
            let mut data_q = sqlx::query(&data_sql);
            if let Some(ref p) = name_param {
                data_q = data_q.bind(p);
            }
            if let Some(ref p) = license_param {
                data_q = data_q.bind(p);
            }
            if let Some(s) = status_param {
                data_q = data_q.bind(s);
            }
            data_q = data_q.bind(page_size).bind((page - 1) * page_size);
            let rows = data_q.fetch_all(&*self.pool).await.map_err(|e| {
                anyhow::anyhow!("Failed to fetch drivers: {} | SQL: {}", e, data_sql)
            })?;
            rows.into_iter()
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
                        license_no: row.try_get("license_no")?,
                        license_type: row.try_get("license_type")?,
                        license_expiry: row.try_get("license_expiry")?,
                        id_card: row.try_get("id_card")?,
                        address: row.try_get("address")?,
                        emergency_contact: row.try_get("emergency_contact")?,
                        emergency_phone: row.try_get("emergency_phone")?,
                        hire_date: row.try_get("hire_date")?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok((drivers, total))
    }

    async fn find_by_id(&self, driver_id: i32) -> Result<Option<Driver>, anyhow::Error> {
        let driver = sqlx::query_as::<_, Driver>("SELECT * FROM drivers WHERE driver_id = $1")
            .bind(driver_id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(driver)
    }

    async fn create(&self, driver: DriverCreateRequest) -> Result<Driver, anyhow::Error> {
        let new_driver = sqlx::query_as::<_, Driver>(
            "INSERT INTO drivers (driver_name, license_number, phone_number, email, status, 
                                license_no, license_type, license_expiry, id_card, address, 
                                emergency_contact, emergency_phone, hire_date, create_time)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW())
             RETURNING *",
        )
        .bind(driver.driver_name)
        .bind(driver.license_number)
        .bind(driver.phone_number)
        .bind(driver.email)
        .bind(driver.status)
        .bind(driver.license_no)
        .bind(driver.license_type)
        .bind(driver.license_expiry)
        .bind(driver.id_card)
        .bind(driver.address)
        .bind(driver.emergency_contact)
        .bind(driver.emergency_phone)
        .bind(driver.hire_date)
        .fetch_one(&*self.pool)
        .await?;

        Ok(new_driver)
    }

    async fn update(
        &self,
        driver_id: i32,
        driver: DriverUpdateRequest,
    ) -> Result<Driver, anyhow::Error> {
        let updated_driver = sqlx::query_as::<_, Driver>(
            "UPDATE drivers
             SET driver_name = COALESCE($1, driver_name),
                 license_number = COALESCE($2, license_number),
                 phone_number = COALESCE($3, phone_number),
                 email = COALESCE($4, email),
                 status = COALESCE($5, status),
                 license_no = COALESCE($6, license_no),
                 license_type = COALESCE($7, license_type),
                 license_expiry = COALESCE($8, license_expiry),
                 id_card = COALESCE($9, id_card),
                 address = COALESCE($10, address),
                 emergency_contact = COALESCE($11, emergency_contact),
                 emergency_phone = COALESCE($12, emergency_phone),
                 hire_date = COALESCE($13, hire_date),
                 update_time = NOW()
             WHERE driver_id = $14
             RETURNING *",
        )
        .bind(driver.driver_name)
        .bind(driver.license_number)
        .bind(driver.phone_number)
        .bind(driver.email)
        .bind(driver.status)
        .bind(driver.license_no)
        .bind(driver.license_type)
        .bind(driver.license_expiry)
        .bind(driver.id_card)
        .bind(driver.address)
        .bind(driver.emergency_contact)
        .bind(driver.emergency_phone)
        .bind(driver.hire_date)
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
        let vehicle_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE driver_id = $1")
                .bind(driver_id)
                .fetch_one(&*self.pool)
                .await?;

        if vehicle_count > 0 {
            return Ok(true);
        }

        // 检查是否有关联的订单
        let order_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE driver_id = $1")
                .bind(driver_id)
                .fetch_one(&*self.pool)
                .await?;

        Ok(order_count > 0)
    }

    async fn exists(&self, driver_id: i32) -> Result<bool, anyhow::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM drivers WHERE driver_id = $1")
            .bind(driver_id)
            .fetch_one(&*self.pool)
            .await?;

        Ok(count > 0)
    }

    async fn count_by_name(
        &self,
        name: &str,
        exclude_id: Option<i32>,
    ) -> Result<i64, anyhow::Error> {
        let count: i64 = if let Some(id) = exclude_id {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM drivers WHERE driver_name = $1 AND driver_id != $2",
            )
            .bind(name)
            .bind(id)
            .fetch_one(&*self.pool)
            .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM drivers WHERE driver_name = $1")
                .bind(name)
                .fetch_one(&*self.pool)
                .await?
        };

        Ok(count)
    }

    async fn count_vehicles(&self, driver_id: i32) -> Result<i64, anyhow::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE driver_id = $1")
            .bind(driver_id)
            .fetch_one(&*self.pool)
            .await?;

        Ok(count)
    }

    async fn count_orders(&self, driver_id: i32) -> Result<i64, anyhow::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE driver_id = $1")
            .bind(driver_id)
            .fetch_one(&*self.pool)
            .await?;

        Ok(count)
    }
}
