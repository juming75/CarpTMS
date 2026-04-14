//! / 车辆仓库PostgreSQL实现

use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};
use crate::domain::use_cases::vehicle::VehicleRepository;

/// 车辆仓库PostgreSQL实现
pub struct PgVehicleRepository {
    pool: Arc<PgPool>,
}

impl PgVehicleRepository {
    /// 创建车辆仓库实例
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl VehicleRepository for PgVehicleRepository {
    /// 获取车辆列表
    async fn get_vehicles(
        &self,
        query: VehicleQuery,
    ) -> Result<(Vec<Vehicle>, i64), anyhow::Error> {
        // 处理分页参数
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        log::info!("Getting vehicles with page={}, page_size={}, offset={}", page, page_size, offset);

        // 直接执行简单查询，不使用动态条件
        let count_query = "SELECT COUNT(*) FROM vehicles";
        let data_query = "SELECT * FROM vehicles ORDER BY vehicle_id DESC LIMIT $1 OFFSET $2";

        // 执行计数查询
        log::info!("Executing count query: {}", count_query);
        let total_count = sqlx::query_scalar::<_, i64>(count_query)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| {
                log::error!("Failed to execute count query: {:?}", e);
                anyhow::anyhow!("Failed to execute count query: {:?}", e)
            })?;
        log::info!("Total vehicles count: {}", total_count);

        // 执行数据查询
        log::info!("Executing data query: {}", data_query);
        let vehicles = sqlx::query_as::<_, Vehicle>(data_query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| {
                log::error!("Failed to execute data query: {:?}", e);
                anyhow::anyhow!("Failed to execute data query: {:?}", e)
            })?;
        log::info!("Fetched {} vehicles", vehicles.len());

        Ok((vehicles, total_count))
    }

    /// 获取单个车辆
    async fn get_vehicle(&self, vehicle_id: i32) -> Result<Option<Vehicle>, anyhow::Error> {
        let vehicle =
            sqlx::query_as::<_, Vehicle>(r#"SELECT * FROM vehicles WHERE vehicle_id = $1"#)
                .bind(vehicle_id)
                .fetch_optional(&*self.pool)
                .await?;

        Ok(vehicle)
    }

    /// 批量获取车辆信息 (数据库批量查询优化)
    async fn get_vehicles_batch(&self, vehicle_ids: &[i32]) -> Result<Vec<Vehicle>, anyhow::Error> {
        if vehicle_ids.is_empty() {
            return Ok(Vec::new());
        }

        let vehicles = sqlx::query_as::<_, Vehicle>(
            "SELECT * FROM vehicles WHERE vehicle_id = ANY($1) AND status = 1",
        )
        .bind(vehicle_ids)
        .fetch_all(&*self.pool)
        .await?;

        Ok(vehicles)
    }

    /// 创建车辆
    async fn create_vehicle(&self, vehicle: VehicleCreate) -> Result<Vehicle, anyhow::Error> {
        let result = sqlx::query_as::<_, Vehicle>(
            r#"INSERT INTO vehicles (
                -- 基本信息
                vehicle_name, license_plate, vehicle_type, vehicle_color, 
                vehicle_brand, vehicle_model, engine_no, frame_no, 
                register_date, inspection_date, insurance_date, seating_capacity, 
                load_capacity, vehicle_length, vehicle_width, vehicle_height, 
                
                -- 终端信息
                device_id, terminal_type, communication_type, sim_card_no, 
                install_date, install_address, install_technician, 
                
                -- 车主信息
                own_no, own_name, own_phone, own_id_card, 
                own_address, own_email, 
                
                -- 运营信息
                group_id, operation_status, operation_route, operation_area, 
                operation_company, driver_name, driver_phone, driver_license_no, 
                
                -- 财务信息
                purchase_price, annual_fee, insurance_fee, 
                
                -- 其他信息
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
        // 基本信息
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
        // 终端信息
        .bind(&vehicle.device_id)
        .bind(&vehicle.terminal_type)
        .bind(&vehicle.communication_type)
        .bind(&vehicle.sim_card_no)
        .bind(vehicle.install_date)
        .bind(&vehicle.install_address)
        .bind(&vehicle.install_technician)
        // 车主信息
        .bind(&vehicle.own_no)
        .bind(&vehicle.own_name)
        .bind(&vehicle.own_phone)
        .bind(&vehicle.own_id_card)
        .bind(&vehicle.own_address)
        .bind(&vehicle.own_email)
        // 运营信息
        .bind(vehicle.group_id)
        .bind(vehicle.operation_status)
        .bind(&vehicle.operation_route)
        .bind(&vehicle.operation_area)
        .bind(&vehicle.operation_company)
        .bind(&vehicle.driver_name)
        .bind(&vehicle.driver_phone)
        .bind(&vehicle.driver_license_no)
        // 财务信息
        .bind(vehicle.purchase_price)
        .bind(vehicle.annual_fee)
        .bind(vehicle.insurance_fee)
        // 其他信息
        .bind(&vehicle.remark)
        .bind(vehicle.create_user_id)
        .fetch_one(&*self.pool)
        .await;

        let v = match result {
            Ok(vehicle) => vehicle,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to create vehicle: {:?}", e));
            }
        };

        Ok(v)
    }

    /// 更新车辆
    async fn update_vehicle(
        &self,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> Result<Option<Vehicle>, anyhow::Error> {
        let result = sqlx::query_as::<_, Vehicle>(
            r#"UPDATE vehicles 
               SET 
                   -- 基本信息
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
                   
                   -- 终端信息
                   device_id = COALESCE($17, device_id),
                   terminal_type = COALESCE($18, terminal_type),
                   communication_type = COALESCE($19, communication_type),
                   sim_card_no = COALESCE($20, sim_card_no),
                   install_date = COALESCE($21, install_date),
                   install_address = COALESCE($22, install_address),
                   install_technician = COALESCE($23, install_technician),
                   
                   -- 车主信息
                   own_no = COALESCE($24, own_no),
                   own_name = COALESCE($25, own_name),
                   own_phone = COALESCE($26, own_phone),
                   own_id_card = COALESCE($27, own_id_card),
                   own_address = COALESCE($28, own_address),
                   own_email = COALESCE($29, own_email),
                   
                   -- 运营信息
                   group_id = COALESCE($30, group_id),
                   operation_status = COALESCE($31, operation_status),
                   operation_route = COALESCE($32, operation_route),
                   operation_area = COALESCE($33, operation_area),
                   operation_company = COALESCE($34, operation_company),
                   driver_name = COALESCE($35, driver_name),
                   driver_phone = COALESCE($36, driver_phone),
                   driver_license_no = COALESCE($37, driver_license_no),
                   
                   -- 财务信息
                   purchase_price = COALESCE($38, purchase_price),
                   annual_fee = COALESCE($39, annual_fee),
                   insurance_fee = COALESCE($40, insurance_fee),
                   
                   -- 其他信息
                   remark = COALESCE($41, remark),
                   status = COALESCE($42, status),
                   update_user_id = COALESCE($43, update_user_id),
                   update_time = CURRENT_TIMESTAMP 
               WHERE vehicle_id = $44 
               RETURNING *"#,
        )
        // 基本信息
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
        // 终端信息
        .bind(&vehicle.device_id)
        .bind(&vehicle.terminal_type)
        .bind(&vehicle.communication_type)
        .bind(&vehicle.sim_card_no)
        .bind(vehicle.install_date)
        .bind(&vehicle.install_address)
        .bind(&vehicle.install_technician)
        // 车主信息
        .bind(&vehicle.own_no)
        .bind(&vehicle.own_name)
        .bind(&vehicle.own_phone)
        .bind(&vehicle.own_id_card)
        .bind(&vehicle.own_address)
        .bind(&vehicle.own_email)
        // 运营信息
        .bind(vehicle.group_id)
        .bind(vehicle.operation_status)
        .bind(&vehicle.operation_route)
        .bind(&vehicle.operation_area)
        .bind(&vehicle.operation_company)
        .bind(&vehicle.driver_name)
        .bind(&vehicle.driver_phone)
        .bind(&vehicle.driver_license_no)
        // 财务信息
        .bind(vehicle.purchase_price)
        .bind(vehicle.annual_fee)
        .bind(vehicle.insurance_fee)
        // 其他信息
        .bind(&vehicle.remark)
        .bind(vehicle.status)
        .bind(vehicle.update_user_id)
        .bind(vehicle_id)
        .fetch_optional(&*self.pool)
        .await;

        match result {
            Ok(Some(vehicle)) => Ok(Some(vehicle)),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to update vehicle: {:?}", e)),
        }
    }

    /// 删除车辆
    async fn delete_vehicle(&self, vehicle_id: i32) -> Result<bool, anyhow::Error> {
        // 先删除相关的称重数据
        let _weighing_result = sqlx::query(r#"DELETE FROM weighing_data WHERE vehicle_id = $1"#)
            .bind(vehicle_id)
            .execute(&*self.pool)
            .await?;

        // 再删除车辆
        let vehicle_result = sqlx::query(r#"DELETE FROM vehicles WHERE vehicle_id = $1"#)
            .bind(vehicle_id)
            .execute(&*self.pool)
            .await?;

        Ok(vehicle_result.rows_affected() > 0)
    }
    
    /// 检查车辆是否有关联数据
    async fn has_related_data(&self, vehicle_id: i32) -> Result<bool, anyhow::Error> {
        // 检查是否有关联的订单
        let has_orders = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM orders WHERE vehicle_id = $1)"#
        )
        .bind(vehicle_id)
        .fetch_one(&*self.pool)
        .await?;
        
        if has_orders {
            return Ok(true);
        }
        
        // 检查是否有关联的物流轨迹
        let has_logistics = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM logistics_tracks WHERE vehicle_id = $1)"#
        )
        .bind(vehicle_id)
        .fetch_one(&*self.pool)
        .await?;
        
        if has_logistics {
            return Ok(true);
        }
        
        // 检查是否有关联的称重数据
        let has_weighing = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(SELECT 1 FROM weighing_data WHERE vehicle_id = $1)"#
        )
        .bind(vehicle_id)
        .fetch_one(&*self.pool)
        .await?;
        
        Ok(has_weighing)
    }
}
