//! / 车辆仓库 SQLx PostgreSQL 实现

use sqlx::Row;

use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};
use crate::domain::repositories::vehicle_repository::VehicleRepository;
use crate::errors::{AppError, AppResult};

/// SQLx PostgreSQL 车辆仓库实现
pub struct SqlxVehicleRepository;

impl SqlxVehicleRepository {
    pub fn new() -> Self {
        Self
    }

    /// 从数据库行构建 Vehicle 实体
    fn row_to_vehicle(row: &sqlx::postgres::PgRow) -> Vehicle {
        Vehicle {
            vehicle_id: row.try_get("vehicle_id").unwrap_or(0),
            vehicle_name: row.try_get("vehicle_name").unwrap_or_default(),
            license_plate: row.try_get("license_plate").unwrap_or_default(),
            vehicle_type: row.try_get("vehicle_type").unwrap_or_default(),
            vehicle_color: row.try_get("vehicle_color").unwrap_or_default(),
            vehicle_brand: row.try_get("vehicle_brand").unwrap_or_default(),
            vehicle_model: row.try_get("vehicle_model").unwrap_or_default(),
            engine_no: row.try_get("engine_no").unwrap_or_default(),
            frame_no: row.try_get("frame_no").unwrap_or_default(),
            register_date: row.try_get("register_date").unwrap_or_else(|_| {
                chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                    .expect("hardcoded date always valid")
                    .and_hms_opt(0, 0, 0)
                    .expect("midnight time always valid")
            }),
            inspection_date: row.try_get("inspection_date").unwrap_or_else(|_| {
                chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                    .expect("hardcoded date always valid")
                    .and_hms_opt(0, 0, 0)
                    .expect("midnight time always valid")
            }),
            insurance_date: row.try_get("insurance_date").unwrap_or_else(|_| {
                chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                    .expect("hardcoded date always valid")
                    .and_hms_opt(0, 0, 0)
                    .expect("midnight time always valid")
            }),
            seating_capacity: row.try_get("seating_capacity").unwrap_or(0),
            load_capacity: row.try_get("load_capacity").unwrap_or(0.0),
            vehicle_length: row.try_get("vehicle_length").unwrap_or(0.0),
            vehicle_width: row.try_get("vehicle_width").unwrap_or(0.0),
            vehicle_height: row.try_get("vehicle_height").unwrap_or(0.0),
            device_id: row.try_get("device_id").ok(),
            terminal_type: row.try_get("terminal_type").ok(),
            communication_type: row.try_get("communication_type").ok(),
            sim_card_no: row.try_get("sim_card_no").ok(),
            install_date: row.try_get("install_date").ok(),
            install_address: row.try_get("install_address").ok(),
            install_technician: row.try_get("install_technician").ok(),
            own_no: row.try_get("own_no").ok(),
            own_name: row.try_get("own_name").ok(),
            own_phone: row.try_get("own_phone").ok(),
            own_id_card: row.try_get("own_id_card").ok(),
            own_address: row.try_get("own_address").ok(),
            own_email: row.try_get("own_email").ok(),
            group_id: row.try_get("group_id").unwrap_or(1),
            operation_status: row.try_get("operation_status").unwrap_or(1),
            operation_route: row.try_get("operation_route").ok(),
            operation_area: row.try_get("operation_area").ok(),
            operation_company: row.try_get("operation_company").ok(),
            driver_name: row.try_get("driver_name").ok(),
            driver_phone: row.try_get("driver_phone").ok(),
            driver_license_no: row.try_get("driver_license_no").ok(),
            purchase_price: row.try_get("purchase_price").ok(),
            annual_fee: row.try_get("annual_fee").ok(),
            insurance_fee: row.try_get("insurance_fee").ok(),
            remark: row.try_get("remark").ok(),
            status: row.try_get("status").unwrap_or(1),
            create_time: row
                .try_get("create_time")
                .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            update_time: row.try_get("update_time").ok(),
            create_user_id: row.try_get("create_user_id").unwrap_or(1),
            update_user_id: row.try_get("update_user_id").ok(),
        }
    }
}

#[async_trait::async_trait]
impl VehicleRepository for SqlxVehicleRepository {
    async fn find_all(
        &self,
        pool: &sqlx::PgPool,
        query: VehicleQuery,
    ) -> AppResult<(Vec<Vehicle>, i64)> {
        // 处理分页参数
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 构建动态查询条件
        let mut where_clauses = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        if let Some(vehicle_name) = &query.vehicle_name {
            if !vehicle_name.is_empty() {
                where_clauses.push(format!("vehicle_name LIKE ${}", param_index));
                params.push(format!("%{}", vehicle_name));
                param_index += 1;
            }
        }

        if let Some(license_plate) = &query.license_plate {
            if !license_plate.is_empty() {
                where_clauses.push(format!("license_plate LIKE ${}", param_index));
                params.push(format!("%{}", license_plate));
                param_index += 1;
            }
        }

        if let Some(vehicle_type) = &query.vehicle_type {
            if !vehicle_type.is_empty() {
                where_clauses.push(format!("vehicle_type = ${}", param_index));
                params.push(vehicle_type.clone());
                param_index += 1;
            }
        }

        if let Some(status) = query.status {
            where_clauses.push(format!("status = ${}", param_index));
            params.push(status.to_string());
            param_index += 1;
        }

        // 构建完整查询
        let where_sql = if where_clauses.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // 查询总记录数
        let count_query = format!("SELECT COUNT(*) FROM vehicles {}", where_sql);
        let mut count_sqlx_query = sqlx::query_scalar::<_, i64>(&count_query);

        // 绑定参数
        for param in &params {
            count_sqlx_query = count_sqlx_query.bind(param);
        }

        let total_count = count_sqlx_query
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to count vehicles: {}", e), None))?;

        // 查询分页数据
        let data_query = format!(
            "SELECT * FROM vehicles {}
             ORDER BY vehicle_id DESC
             LIMIT ${} OFFSET ${}",
            where_sql,
            param_index,
            param_index + 1
        );

        let mut sqlx_query = sqlx::query(&data_query);

        // 绑定参数
        for param in params {
            sqlx_query = sqlx_query.bind(param);
        }
        sqlx_query = sqlx_query.bind(page_size).bind(offset);

        let rows = sqlx_query
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to fetch vehicles: {}", e), None))?;

        let vehicles: Vec<Vehicle> = rows.iter().map(Self::row_to_vehicle).collect();

        Ok((vehicles, total_count))
    }

    async fn find_by_id(&self, pool: &sqlx::PgPool, vehicle_id: i32) -> AppResult<Option<Vehicle>> {
        let row = sqlx::query("SELECT * FROM vehicles WHERE vehicle_id = $1")
            .bind(vehicle_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to fetch vehicle: {}", e), None))?;

        Ok(row.as_ref().map(Self::row_to_vehicle))
    }

    async fn find_by_ids(
        &self,
        pool: &sqlx::PgPool,
        vehicle_ids: &[i32],
    ) -> AppResult<Vec<Vehicle>> {
        if vehicle_ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query("SELECT * FROM vehicles WHERE vehicle_id = ANY($1) AND status = 1")
            .bind(vehicle_ids)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::db_error(&format!("Failed to fetch vehicles batch: {}", e), None)
            })?;

        Ok(rows.iter().map(Self::row_to_vehicle).collect())
    }

    async fn create(&self, pool: &sqlx::PgPool, vehicle: VehicleCreate) -> AppResult<Vehicle> {
        let row = sqlx::query(
            r#"INSERT INTO vehicles (
                vehicle_name, license_plate, vehicle_type, vehicle_color, 
                vehicle_brand, vehicle_model, engine_no, frame_no, 
                register_date, inspection_date, insurance_date, seating_capacity, 
                load_capacity, vehicle_length, vehicle_width, vehicle_height, 
                device_id, terminal_type, communication_type, sim_card_no, 
                install_date, install_address, install_technician, 
                own_no, own_name, own_phone, own_id_card, 
                own_address, own_email, 
                group_id, operation_status, operation_route, operation_area, 
                operation_company, driver_name, driver_phone, driver_license_no, 
                purchase_price, annual_fee, insurance_fee, 
                remark, create_user_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, 
                $17, $18, $19, $20, $21, $22, $23, 
                $24, $25, $26, $27, $28, $29, 
                $30, $31, $32, $33, $34, $35, $36, $37, 
                $38, $39, $40, 
                $41, $42
            ) RETURNING *"#,
        )
        .bind(&vehicle.vehicle_name)
        .bind(&vehicle.license_plate)
        .bind(&vehicle.vehicle_type)
        .bind(&vehicle.vehicle_color)
        .bind(&vehicle.vehicle_brand)
        .bind(&vehicle.vehicle_model)
        .bind(&vehicle.engine_no)
        .bind(&vehicle.frame_no)
        .bind(vehicle.register_date)
        .bind(vehicle.inspection_date)
        .bind(vehicle.insurance_date)
        .bind(vehicle.seating_capacity)
        .bind(vehicle.load_capacity)
        .bind(vehicle.vehicle_length)
        .bind(vehicle.vehicle_width)
        .bind(vehicle.vehicle_height)
        .bind(&vehicle.device_id)
        .bind(&vehicle.terminal_type)
        .bind(&vehicle.communication_type)
        .bind(&vehicle.sim_card_no)
        .bind(vehicle.install_date)
        .bind(&vehicle.install_address)
        .bind(&vehicle.install_technician)
        .bind(&vehicle.own_no)
        .bind(&vehicle.own_name)
        .bind(&vehicle.own_phone)
        .bind(&vehicle.own_id_card)
        .bind(&vehicle.own_address)
        .bind(&vehicle.own_email)
        .bind(vehicle.group_id)
        .bind(vehicle.operation_status)
        .bind(&vehicle.operation_route)
        .bind(&vehicle.operation_area)
        .bind(&vehicle.operation_company)
        .bind(&vehicle.driver_name)
        .bind(&vehicle.driver_phone)
        .bind(&vehicle.driver_license_no)
        .bind(vehicle.purchase_price)
        .bind(vehicle.annual_fee)
        .bind(vehicle.insurance_fee)
        .bind(&vehicle.remark)
        .bind(vehicle.create_user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to create vehicle: {}", e), None))?;

        Ok(Self::row_to_vehicle(&row))
    }

    async fn update(
        &self,
        pool: &sqlx::PgPool,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> AppResult<Option<Vehicle>> {
        let row = sqlx::query(
            r#"UPDATE vehicles 
               SET 
                   vehicle_name = COALESCE($1, vehicle_name),
                   license_plate = COALESCE($2, license_plate),
                   vehicle_type = COALESCE($3, vehicle_type),
                   vehicle_color = COALESCE($4, vehicle_color),
                   vehicle_brand = COALESCE($5, vehicle_brand),
                   vehicle_model = COALESCE($6, vehicle_model),
                   engine_no = COALESCE($7, engine_no),
                   frame_no = COALESCE($8, frame_no),
                   register_date = COALESCE($9, register_date),
                   inspection_date = COALESCE($10, inspection_date),
                   insurance_date = COALESCE($11, insurance_date),
                   seating_capacity = COALESCE($12, seating_capacity),
                   load_capacity = COALESCE($13, load_capacity),
                   vehicle_length = COALESCE($14, vehicle_length),
                   vehicle_width = COALESCE($15, vehicle_width),
                   vehicle_height = COALESCE($16, vehicle_height),
                   device_id = COALESCE($17, device_id),
                   terminal_type = COALESCE($18, terminal_type),
                   communication_type = COALESCE($19, communication_type),
                   sim_card_no = COALESCE($20, sim_card_no),
                   install_date = COALESCE($21, install_date),
                   install_address = COALESCE($22, install_address),
                   install_technician = COALESCE($23, install_technician),
                   own_no = COALESCE($24, own_no),
                   own_name = COALESCE($25, own_name),
                   own_phone = COALESCE($26, own_phone),
                   own_id_card = COALESCE($27, own_id_card),
                   own_address = COALESCE($28, own_address),
                   own_email = COALESCE($29, own_email),
                   group_id = COALESCE($30, group_id),
                   operation_status = COALESCE($31, operation_status),
                   operation_route = COALESCE($32, operation_route),
                   operation_area = COALESCE($33, operation_area),
                   operation_company = COALESCE($34, operation_company),
                   driver_name = COALESCE($35, driver_name),
                   driver_phone = COALESCE($36, driver_phone),
                   driver_license_no = COALESCE($37, driver_license_no),
                   purchase_price = COALESCE($38, purchase_price),
                   annual_fee = COALESCE($39, annual_fee),
                   insurance_fee = COALESCE($40, insurance_fee),
                   remark = COALESCE($41, remark),
                   status = COALESCE($42, status),
                   update_user_id = COALESCE($43, update_user_id),
                   update_time = CURRENT_TIMESTAMP 
               WHERE vehicle_id = $44 
               RETURNING *"#,
        )
        .bind(&vehicle.vehicle_name)
        .bind(&vehicle.license_plate)
        .bind(&vehicle.vehicle_type)
        .bind(&vehicle.vehicle_color)
        .bind(&vehicle.vehicle_brand)
        .bind(&vehicle.vehicle_model)
        .bind(&vehicle.engine_no)
        .bind(&vehicle.frame_no)
        .bind(vehicle.register_date)
        .bind(vehicle.inspection_date)
        .bind(vehicle.insurance_date)
        .bind(vehicle.seating_capacity)
        .bind(vehicle.load_capacity)
        .bind(vehicle.vehicle_length)
        .bind(vehicle.vehicle_width)
        .bind(vehicle.vehicle_height)
        .bind(&vehicle.device_id)
        .bind(&vehicle.terminal_type)
        .bind(&vehicle.communication_type)
        .bind(&vehicle.sim_card_no)
        .bind(vehicle.install_date)
        .bind(&vehicle.install_address)
        .bind(&vehicle.install_technician)
        .bind(&vehicle.own_no)
        .bind(&vehicle.own_name)
        .bind(&vehicle.own_phone)
        .bind(&vehicle.own_id_card)
        .bind(&vehicle.own_address)
        .bind(&vehicle.own_email)
        .bind(vehicle.group_id)
        .bind(vehicle.operation_status)
        .bind(&vehicle.operation_route)
        .bind(&vehicle.operation_area)
        .bind(&vehicle.operation_company)
        .bind(&vehicle.driver_name)
        .bind(&vehicle.driver_phone)
        .bind(&vehicle.driver_license_no)
        .bind(vehicle.purchase_price)
        .bind(vehicle.annual_fee)
        .bind(vehicle.insurance_fee)
        .bind(&vehicle.remark)
        .bind(vehicle.status)
        .bind(vehicle.update_user_id)
        .bind(vehicle_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::db_error(&format!("Failed to update vehicle: {}", e), None))?;

        Ok(row.as_ref().map(Self::row_to_vehicle))
    }

    async fn delete(&self, pool: &sqlx::PgPool, vehicle_id: i32) -> AppResult<bool> {
        // 先删除相关的称重数据
        sqlx::query("DELETE FROM weighing_data WHERE vehicle_id = $1")
            .bind(vehicle_id)
            .execute(pool)
            .await
            .map_err(|e| {
                AppError::db_error(&format!("Failed to delete weighing data: {}", e), None)
            })?;

        // 删除车辆
        let result = sqlx::query("DELETE FROM vehicles WHERE vehicle_id = $1")
            .bind(vehicle_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::db_error(&format!("Failed to delete vehicle: {}", e), None))?;

        Ok(result.rows_affected() > 0)
    }
}

impl Default for SqlxVehicleRepository {
    fn default() -> Self {
        Self::new()
    }
}
