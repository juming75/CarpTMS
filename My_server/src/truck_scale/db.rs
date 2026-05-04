//! 数据库操作模块
use anyhow::Result;
use sqlx::{PgPool, Row};
use std::sync::Arc;

/// 地磅系统数据库操作器
///
/// 提供地磅系统所有数据库操作的统一接口,包括:
/// - 车辆管理
/// - 车组管理
/// - 用户管理
/// - 用户组管理
/// - 地磅数据管理
///
/// # 设计原则
/// - 所有操作返回`Result<T>`类型,支持错误传播
/// - 使用参数化查询,防止SQL注入
/// - 支持事务操作
///
/// # 示例
/// ```ignore
/// let db = TruckScaleDb::new(pool);
/// let vehicle = db.query_vehicle("V001").await?;
/// ```
pub struct TruckScaleDb {
    pool: Arc<PgPool>,
}

impl TruckScaleDb {
    /// 创建新的数据库操作器
    ///
    /// # 参数
    /// - `pool`: PostgreSQL连接池
    ///
    /// # 返回
    /// 返回初始化好的`TruckScaleDb`实例
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// 获取数据库连接池
    ///
    /// # 返回
    /// 返回内部PostgreSQL连接池的引用
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// ==================== 车辆管理 ====================

impl TruckScaleDb {
    /// 查询车辆信息
    ///
    /// 根据车辆ID查询车辆的完整信息,包括:
    /// - 基础信息(车牌号、终端号等)
    /// - 车辆参数(载重、尺寸等)
    /// - 车主信息
    /// - 司机信息
    /// - 保险和年检信息
    ///
    /// # 参数
    /// - `vehicle_id`: 车辆ID
    ///
    /// # 返回
    /// 成功返回`Some(serde_json::Value)`包含车辆完整信息,
    /// 如果车辆不存在返回`None`
    ///
    /// # 错误
    /// - 数据库连接失败
    /// - SQL查询错误
    ///
    /// # 示例
    /// ```ignore
    /// if let Some(vehicle) = db.query_vehicle("V001").await? {
    ///     println!("Vehicle plate: {}", vehicle["plate_no"]);
    /// }
    /// ```
    pub async fn query_vehicle(&self, vehicle_id: &str) -> Result<Option<serde_json::Value>> {
        let vehicle = sqlx::query(
            "SELECT id, vehicle_id, plate_no, terminal_no, sim_no, engine_no, frame_no,
                    owner_name, owner_tel, owner_address, vehicle_type, vehicle_color,
                    vehicle_brand, vehicle_model, group_id, driver_name, driver_tel,
                    driver_license, max_weight, tare_weight, rated_weight, length,
                    width, height, fuel_type, manufacturer, manufacture_date, registration_date,
                    insurance_expire_date, annual_inspection_date, remark, status,
                    create_time, update_time, create_by, update_by
             FROM truck_scale_vehicles
             WHERE vehicle_id = $1 AND status = 0",
        )
        .bind(vehicle_id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(vehicle.map(|v| {
            serde_json::json!({
                "vehicle_id": v.get::<String, _>("vehicle_id"),
                "plate_no": v.get::<String, _>("plate_no"),
                "terminal_no": v.get::<String, _>("terminal_no"),
                "sim_no": v.get::<String, _>("sim_no"),
                "engine_no": v.get::<String, _>("engine_no"),
                "frame_no": v.get::<String, _>("frame_no"),
                "owner_name": v.get::<String, _>("owner_name"),
                "owner_tel": v.get::<String, _>("owner_tel"),
                "owner_address": v.get::<String, _>("owner_address"),
                "vehicle_type": v.get::<String, _>("vehicle_type"),
                "vehicle_color": v.get::<String, _>("vehicle_color"),
                "vehicle_brand": v.get::<String, _>("vehicle_brand"),
                "vehicle_model": v.get::<String, _>("vehicle_model"),
                "group_id": v.get::<String, _>("group_id"),
                "driver_name": v.get::<String, _>("driver_name"),
                "driver_tel": v.get::<String, _>("driver_tel"),
                "driver_license": v.get::<String, _>("driver_license"),
                "max_weight": v.get::<f64, _>("max_weight"),
                "tare_weight": v.get::<f64, _>("tare_weight"),
                "rated_weight": v.get::<f64, _>("rated_weight"),
                "length": v.get::<f64, _>("length"),
                "width": v.get::<f64, _>("width"),
                "height": v.get::<f64, _>("height"),
                "fuel_type": v.get::<String, _>("fuel_type"),
                "manufacturer": v.get::<String, _>("manufacturer"),
                "manufacture_date": v.get::<String, _>("manufacture_date"),
                "registration_date": v.get::<String, _>("registration_date"),
                "insurance_expire_date": v.get::<String, _>("insurance_expire_date"),
                "annual_inspection_date": v.get::<String, _>("annual_inspection_date"),
                "remark": v.get::<String, _>("remark"),
                "status": v.get::<i32, _>("status"),
                "create_time": v.get::<String, _>("create_time"),
                "update_time": v.get::<String, _>("update_time"),
                "create_by": v.get::<String, _>("create_by"),
                "update_by": v.get::<String, _>("update_by"),
            })
        }))
    }

    /// 添加车辆
    pub async fn add_vehicle(&self, vehicle_data: serde_json::Value) -> Result<String> {
        let vehicle_id = vehicle_data["vehicle_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing vehicle_id"))?;
        let plate_no = vehicle_data["plate_no"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing plate_no"))?;

        sqlx::query(
            "INSERT INTO truck_scale_vehicles (
                vehicle_id, plate_no, terminal_no, sim_no, engine_no, frame_no,
                owner_name, owner_tel, owner_address, vehicle_type, vehicle_color,
                vehicle_brand, vehicle_model, group_id, driver_name, driver_tel,
                driver_license, max_weight, tare_weight, rated_weight, length,
                width, height, fuel_type, manufacturer, manufacture_date,
                registration_date, insurance_expire_date, annual_inspection_date,
                remark, create_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 
                    $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, 
                    $25, $26, $27, $28, $29, $30, $31)",
        )
        .bind(vehicle_id)
        .bind(plate_no)
        .bind(vehicle_data["terminal_no"].as_str())
        .bind(vehicle_data["sim_no"].as_str())
        .bind(vehicle_data["engine_no"].as_str())
        .bind(vehicle_data["frame_no"].as_str())
        .bind(vehicle_data["owner_name"].as_str())
        .bind(vehicle_data["owner_tel"].as_str())
        .bind(vehicle_data["owner_address"].as_str())
        .bind(vehicle_data["vehicle_type"].as_str())
        .bind(vehicle_data["vehicle_color"].as_str())
        .bind(vehicle_data["vehicle_brand"].as_str())
        .bind(vehicle_data["vehicle_model"].as_str())
        .bind(vehicle_data["group_id"].as_str())
        .bind(vehicle_data["driver_name"].as_str())
        .bind(vehicle_data["driver_tel"].as_str())
        .bind(vehicle_data["driver_license"].as_str())
        .bind(vehicle_data["max_weight"].as_f64())
        .bind(vehicle_data["tare_weight"].as_f64())
        .bind(vehicle_data["rated_weight"].as_f64())
        .bind(vehicle_data["length"].as_f64())
        .bind(vehicle_data["width"].as_f64())
        .bind(vehicle_data["height"].as_f64())
        .bind(vehicle_data["fuel_type"].as_str())
        .bind(vehicle_data["manufacturer"].as_str())
        .bind(vehicle_data["manufacture_date"].as_str())
        .bind(vehicle_data["registration_date"].as_str())
        .bind(vehicle_data["insurance_expire_date"].as_str())
        .bind(vehicle_data["annual_inspection_date"].as_str())
        .bind(vehicle_data["remark"].as_str())
        .bind(vehicle_data["create_by"].as_str())
        .execute(&*self.pool)
        .await?;

        Ok(vehicle_id.to_string())
    }

    /// 更新车辆
    pub async fn update_vehicle(&self, vehicle_data: serde_json::Value) -> Result<()> {
        let vehicle_id = vehicle_data["vehicle_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing vehicle_id"))?;

        sqlx::query(
            "UPDATE truck_scale_vehicles SET
                plate_no = $2, terminal_no = $3, sim_no = $4, engine_no = $5,
                frame_no = $6, owner_name = $7, owner_tel = $8, owner_address = $9,
                vehicle_type = $10, vehicle_color = $11, vehicle_brand = $12,
                vehicle_model = $13, group_id = $14, driver_name = $15, driver_tel = $16,
                driver_license = $17, max_weight = $18, tare_weight = $19,
                rated_weight = $20, length = $21, width = $22, height = $23,
                fuel_type = $24, manufacturer = $25, manufacture_date = $26,
                registration_date = $27, insurance_expire_date = $28,
                annual_inspection_date = $29, remark = $30, update_by = $31
            WHERE vehicle_id = $1 AND status = 0",
        )
        .bind(vehicle_id)
        .bind(vehicle_data["plate_no"].as_str())
        .bind(vehicle_data["terminal_no"].as_str())
        .bind(vehicle_data["sim_no"].as_str())
        .bind(vehicle_data["engine_no"].as_str())
        .bind(vehicle_data["frame_no"].as_str())
        .bind(vehicle_data["owner_name"].as_str())
        .bind(vehicle_data["owner_tel"].as_str())
        .bind(vehicle_data["owner_address"].as_str())
        .bind(vehicle_data["vehicle_type"].as_str())
        .bind(vehicle_data["vehicle_color"].as_str())
        .bind(vehicle_data["vehicle_brand"].as_str())
        .bind(vehicle_data["vehicle_model"].as_str())
        .bind(vehicle_data["group_id"].as_str())
        .bind(vehicle_data["driver_name"].as_str())
        .bind(vehicle_data["driver_tel"].as_str())
        .bind(vehicle_data["driver_license"].as_str())
        .bind(vehicle_data["max_weight"].as_f64())
        .bind(vehicle_data["tare_weight"].as_f64())
        .bind(vehicle_data["rated_weight"].as_f64())
        .bind(vehicle_data["length"].as_f64())
        .bind(vehicle_data["width"].as_f64())
        .bind(vehicle_data["height"].as_f64())
        .bind(vehicle_data["fuel_type"].as_str())
        .bind(vehicle_data["manufacturer"].as_str())
        .bind(vehicle_data["manufacture_date"].as_str())
        .bind(vehicle_data["registration_date"].as_str())
        .bind(vehicle_data["insurance_expire_date"].as_str())
        .bind(vehicle_data["annual_inspection_date"].as_str())
        .bind(vehicle_data["remark"].as_str())
        .bind(vehicle_data["update_by"].as_str())
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// 删除车辆(软删除)
    pub async fn delete_vehicle(&self, vehicle_id: &str, delete_by: &str) -> Result<()> {
        sqlx::query(
            "UPDATE truck_scale_vehicles 
             SET status = 1, update_by = $2, update_time = CURRENT_TIMESTAMP
             WHERE vehicle_id = $1 AND status = 0",
        )
        .bind(vehicle_id)
        .bind(delete_by)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// 查询车辆列表(分页)
    pub async fn query_vehicle_list(
        &self,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<serde_json::Value>> {
        let vehicles = sqlx::query(
            "SELECT id, vehicle_id, plate_no, terminal_no, sim_no, engine_no, frame_no,
                    owner_name, owner_tel, owner_address, vehicle_type, vehicle_color,
                    vehicle_brand, vehicle_model, group_id, driver_name, driver_tel,
                    driver_license, max_weight, tare_weight, rated_weight, length,
                    width, height, fuel_type, manufacturer, manufacture_date, registration_date,
                    insurance_expire_date, annual_inspection_date, remark, status,
                    create_time, update_time, create_by, update_by
             FROM truck_scale_vehicles
             WHERE status = 0
             ORDER BY create_time DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?;

        Ok(vehicles
            .into_iter()
            .map(|v| {
                serde_json::json!({
                    "vehicle_id": v.get::<String, _>("vehicle_id"),
                    "plate_no": v.get::<String, _>("plate_no"),
                    "terminal_no": v.get::<String, _>("terminal_no"),
                    "sim_no": v.get::<String, _>("sim_no"),
                    "engine_no": v.get::<String, _>("engine_no"),
                    "frame_no": v.get::<String, _>("frame_no"),
                    "owner_name": v.get::<String, _>("owner_name"),
                    "owner_tel": v.get::<String, _>("owner_tel"),
                    "owner_address": v.get::<String, _>("owner_address"),
                    "vehicle_type": v.get::<String, _>("vehicle_type"),
                    "vehicle_color": v.get::<String, _>("vehicle_color"),
                    "vehicle_brand": v.get::<String, _>("vehicle_brand"),
                    "vehicle_model": v.get::<String, _>("vehicle_model"),
                    "group_id": v.get::<String, _>("group_id"),
                    "driver_name": v.get::<String, _>("driver_name"),
                    "driver_tel": v.get::<String, _>("driver_tel"),
                    "driver_license": v.get::<String, _>("driver_license"),
                    "max_weight": v.get::<f64, _>("max_weight"),
                    "tare_weight": v.get::<f64, _>("tare_weight"),
                    "rated_weight": v.get::<f64, _>("rated_weight"),
                    "length": v.get::<f64, _>("length"),
                    "width": v.get::<f64, _>("width"),
                    "height": v.get::<f64, _>("height"),
                    "fuel_type": v.get::<String, _>("fuel_type"),
                    "manufacturer": v.get::<String, _>("manufacturer"),
                    "manufacture_date": v.get::<String, _>("manufacture_date"),
                    "registration_date": v.get::<String, _>("registration_date"),
                    "insurance_expire_date": v.get::<String, _>("insurance_expire_date"),
                    "annual_inspection_date": v.get::<String, _>("annual_inspection_date"),
                    "remark": v.get::<String, _>("remark"),
                    "status": v.get::<i32, _>("status"),
                    "create_time": v.get::<String, _>("create_time"),
                    "update_time": v.get::<String, _>("update_time"),
                    "create_by": v.get::<String, _>("create_by"),
                    "update_by": v.get::<String, _>("update_by"),
                })
            })
            .collect())
    }
}

// ==================== 用户管理 ====================

impl TruckScaleDb {
    /// 查询用户信息
    pub async fn query_user(&self, user_id: &str) -> Result<Option<serde_json::Value>> {
        let user = sqlx::query(
            "SELECT user_id, user_name, password_hash, real_name, user_type, group_id,
                    company, department, tel, mobile, email, address, permission,
                    veh_group_list, status, expiration_time
             FROM truck_scale_users
             WHERE user_id = $1 AND status = 0",
        )
        .bind(user_id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(user.map(|u| {
            serde_json::json!({
                "user_id": u.get::<String, _>("user_id"),
                "user_name": u.get::<String, _>("user_name"),
                "password": u.get::<String, _>("password_hash"),
                "real_name": u.get::<String, _>("real_name"),
                "user_type": u.get::<i32, _>("user_type"),
                "group_id": u.get::<String, _>("group_id"),
                "company": u.get::<String, _>("company"),
                "department": u.get::<String, _>("department"),
                "tel": u.get::<String, _>("tel"),
                "mobile": u.get::<String, _>("mobile"),
                "email": u.get::<String, _>("email"),
                "address": u.get::<String, _>("address"),
                "permission": u.get::<String, _>("permission"),
                "veh_group_list": u.get::<String, _>("veh_group_list"),
                "status": u.get::<i32, _>("status"),
                "expiration_time": u.get::<String, _>("expiration_time"),
            })
        }))
    }

    /// 根据用户名查询用户
    pub async fn query_user_by_name(&self, user_name: &str) -> Result<Option<serde_json::Value>> {
        let user = sqlx::query(
            "SELECT user_id, user_name, password_hash, real_name, user_type, group_id,
                    company, department, tel, mobile, email, address, permission,
                    veh_group_list, status, expiration_time
             FROM truck_scale_users
             WHERE user_name = $1 AND status = 0",
        )
        .bind(user_name)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(user.map(|u| {
            serde_json::json!({
                "user_id": u.get::<String, _>("user_id"),
                "user_name": u.get::<String, _>("user_name"),
                "password": u.get::<String, _>("password_hash"),
                "real_name": u.get::<String, _>("real_name"),
                "user_type": u.get::<i32, _>("user_type"),
                "group_id": u.get::<String, _>("group_id"),
                "company": u.get::<String, _>("company"),
                "department": u.get::<String, _>("department"),
                "tel": u.get::<String, _>("tel"),
                "mobile": u.get::<String, _>("mobile"),
                "email": u.get::<String, _>("email"),
                "address": u.get::<String, _>("address"),
                "permission": u.get::<String, _>("permission"),
                "veh_group_list": u.get::<String, _>("veh_group_list"),
                "status": u.get::<i32, _>("status"),
                "expiration_time": u.get::<String, _>("expiration_time"),
            })
        }))
    }

    /// 添加用户
    pub async fn add_user(&self, user_data: serde_json::Value) -> Result<String> {
        let user_id = user_data["user_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?;
        let user_name = user_data["user_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user_name"))?;
        let password = user_data["password"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing password"))?;

        // 哈希密码
        // 使用argon2::password_hash::PasswordHasher接口
        use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
        use argon2::Argon2;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Password hashing error: {}", e))?;
        let hashed_password = hashed.to_string();

        sqlx::query(
            "INSERT INTO truck_scale_users (
                user_id, user_name, password_hash, real_name, user_type, group_id,
                company, department, tel, mobile, email, address, permission,
                veh_group_list, expiration_time, create_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 
                    $14, $15, $16)",
        )
        .bind(user_id)
        .bind(user_name)
        .bind(hashed_password)
        .bind(user_data["real_name"].as_str())
        .bind(user_data["user_type"].as_i64().unwrap_or(3) as i32)
        .bind(user_data["group_id"].as_str())
        .bind(user_data["company"].as_str())
        .bind(user_data["department"].as_str())
        .bind(user_data["tel"].as_str())
        .bind(user_data["mobile"].as_str())
        .bind(user_data["email"].as_str())
        .bind(user_data["address"].as_str())
        .bind(user_data["permission"].as_str())
        .bind(user_data["veh_group_list"].as_str())
        .bind(user_data["expiration_time"].as_str())
        .bind(user_data["create_by"].as_str())
        .execute(&*self.pool)
        .await?;

        Ok(user_id.to_string())
    }

    /// 更新用户
    pub async fn update_user(&self, user_data: serde_json::Value) -> Result<()> {
        let user_id = user_data["user_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?;

        // 如果提供了新密码,需要哈希
        let password_hash = if let Some(password) = user_data["password"].as_str() {
            Some({
                // 使用argon2::password_hash::PasswordHasher接口
                use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
                use argon2::Argon2;

                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let hashed = argon2
                    .hash_password(password.as_bytes(), &salt)
                    .map_err(|e| anyhow::anyhow!("Password hashing error: {}", e))?;
                hashed.to_string()
            })
        } else {
            None
        };

        let mut query = sqlx::query(
            "UPDATE truck_scale_users SET
                real_name = $2, user_type = $3, group_id = $4, company = $5,
                department = $6, tel = $7, mobile = $8, email = $9, address = $10,
                permission = $11, veh_group_list = $12, expiration_time = $13,
                update_by = $14",
        )
        .bind(user_id)
        .bind(user_data["real_name"].as_str())
        .bind(user_data["user_type"].as_i64().unwrap_or(3) as i32)
        .bind(user_data["group_id"].as_str())
        .bind(user_data["company"].as_str())
        .bind(user_data["department"].as_str())
        .bind(user_data["tel"].as_str())
        .bind(user_data["mobile"].as_str())
        .bind(user_data["email"].as_str())
        .bind(user_data["address"].as_str())
        .bind(user_data["permission"].as_str())
        .bind(user_data["veh_group_list"].as_str())
        .bind(user_data["expiration_time"].as_str())
        .bind(user_data["update_by"].as_str());

        // 如果提供了新密码,更新密码
        if let Some(hashed) = password_hash {
            query = sqlx::query(
                "UPDATE truck_scale_users SET
                    password_hash = $2, real_name = $3, user_type = $4, group_id = $5,
                    company = $6, department = $7, tel = $8, mobile = $9, email = $10, address = $11,
                    permission = $12, veh_group_list = $13, expiration_time = $14,
                    update_by = $15
                WHERE user_id = $1 AND status = 0"
            )
            .bind(user_id)
            .bind(hashed)
            .bind(user_data["real_name"].as_str())
            .bind(user_data["user_type"].as_i64().unwrap_or(3) as i32)
            .bind(user_data["group_id"].as_str())
            .bind(user_data["company"].as_str())
            .bind(user_data["department"].as_str())
            .bind(user_data["tel"].as_str())
            .bind(user_data["mobile"].as_str())
            .bind(user_data["email"].as_str())
            .bind(user_data["address"].as_str())
            .bind(user_data["permission"].as_str())
            .bind(user_data["veh_group_list"].as_str())
            .bind(user_data["expiration_time"].as_str())
            .bind(user_data["update_by"].as_str());
        }

        query.execute(&*self.pool).await?;
        Ok(())
    }

    /// 删除用户(软删除)
    pub async fn delete_user(&self, user_id: &str, delete_by: &str) -> Result<()> {
        sqlx::query(
            "UPDATE truck_scale_users 
             SET status = 1, update_by = $2, update_time = CURRENT_TIMESTAMP
             WHERE user_id = $1 AND status = 0",
        )
        .bind(user_id)
        .bind(delete_by)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// 查询用户列表(分页)
    pub async fn query_user_list(&self, offset: i32, limit: i32) -> Result<Vec<serde_json::Value>> {
        let users = sqlx::query(
            "SELECT user_id, user_name, password_hash, real_name, user_type, group_id,
                    company, department, tel, mobile, email, address, permission,
                    veh_group_list, status, expiration_time
             FROM truck_scale_users
             WHERE status = 0
             ORDER BY create_time DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?;

        Ok(users
            .into_iter()
            .map(|u| {
                serde_json::json!({
                    "user_id": u.get::<String, _>("user_id"),
                    "user_name": u.get::<String, _>("user_name"),
                    "password": u.get::<String, _>("password_hash"),
                    "real_name": u.get::<String, _>("real_name"),
                    "user_type": u.get::<i32, _>("user_type"),
                    "group_id": u.get::<String, _>("group_id"),
                    "company": u.get::<String, _>("company"),
                    "department": u.get::<String, _>("department"),
                    "tel": u.get::<String, _>("tel"),
                    "mobile": u.get::<String, _>("mobile"),
                    "email": u.get::<String, _>("email"),
                    "address": u.get::<String, _>("address"),
                    "permission": u.get::<String, _>("permission"),
                    "veh_group_list": u.get::<String, _>("veh_group_list"),
                    "status": u.get::<i32, _>("status"),
                    "expiration_time": u.get::<String, _>("expiration_time"),
                })
            })
            .collect())
    }
}

// ==================== 车组管理 ====================

impl TruckScaleDb {
    /// 查询车组信息
    pub async fn query_vehicle_group(&self, group_id: &str) -> Result<Option<serde_json::Value>> {
        let group = sqlx::query(
            "SELECT group_id, parent_id, group_name, contact_people, contact_tel
             FROM truck_scale_vehicle_groups
             WHERE group_id = $1 AND status = 0",
        )
        .bind(group_id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(group.map(|g| {
            serde_json::json!({
                "group_id": g.get::<String, _>("group_id"),
                "parent_id": g.get::<Option<String>, _>("parent_id"),
                "group_name": g.get::<String, _>("group_name"),
                "contact_people": g.get::<Option<String>, _>("contact_people"),
                "contact_tel": g.get::<Option<String>, _>("contact_tel"),
            })
        }))
    }

    /// 查询所有车组
    pub async fn query_all_vehicle_groups(&self) -> Result<Vec<serde_json::Value>> {
        let groups = sqlx::query(
            "SELECT group_id, parent_id, group_name, contact_people, contact_tel
             FROM truck_scale_vehicle_groups
             WHERE status = 0
             ORDER BY group_id",
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(groups
            .into_iter()
            .map(|g| {
                serde_json::json!({
                    "group_id": g.get::<String, _>("group_id"),
                    "parent_id": g.get::<Option<String>, _>("parent_id"),
                    "group_name": g.get::<String, _>("group_name"),
                    "contact_people": g.get::<Option<String>, _>("contact_people"),
                    "contact_tel": g.get::<Option<String>, _>("contact_tel"),
                })
            })
            .collect())
    }

    /// 添加车组
    pub async fn add_vehicle_group(&self, group_data: serde_json::Value) -> Result<String> {
        let group_id = group_data["group_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing group_id"))?;
        let group_name = group_data["group_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing group_name"))?;

        sqlx::query(
            "INSERT INTO truck_scale_vehicle_groups (
                group_id, parent_id, group_name, contact_people, contact_tel, create_by
            ) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(group_id)
        .bind(group_data["parent_id"].as_str())
        .bind(group_name)
        .bind(group_data["contact_people"].as_str())
        .bind(group_data["contact_tel"].as_str())
        .bind(group_data["create_by"].as_str())
        .execute(&*self.pool)
        .await?;

        Ok(group_id.to_string())
    }

    /// 更新车组
    pub async fn update_vehicle_group(&self, group_data: serde_json::Value) -> Result<()> {
        let group_id = group_data["group_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing group_id"))?;

        sqlx::query(
            "UPDATE truck_scale_vehicle_groups SET
                parent_id = $2, group_name = $3, contact_people = $4,
                contact_tel = $5, update_by = $6
            WHERE group_id = $1 AND status = 0",
        )
        .bind(group_id)
        .bind(group_data["parent_id"].as_str())
        .bind(group_data["group_name"].as_str())
        .bind(group_data["contact_people"].as_str())
        .bind(group_data["contact_tel"].as_str())
        .bind(group_data["update_by"].as_str())
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// 删除车组(软删除)
    pub async fn delete_vehicle_group(&self, group_id: &str, delete_by: &str) -> Result<()> {
        sqlx::query(
            "UPDATE truck_scale_vehicle_groups 
             SET status = 1, update_by = $2, update_time = CURRENT_TIMESTAMP
             WHERE group_id = $1 AND status = 0",
        )
        .bind(group_id)
        .bind(delete_by)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
}

// ==================== 用户组管理 ====================

impl TruckScaleDb {
    /// 查询用户组信息
    pub async fn query_user_group(&self, group_id: &str) -> Result<Option<serde_json::Value>> {
        let group = sqlx::query(
            "SELECT group_id, group_name, user_type, permission
             FROM truck_scale_user_groups
             WHERE group_id = $1 AND status = 0",
        )
        .bind(group_id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(group.map(|g| {
            serde_json::json!({
                "group_id": g.get::<String, _>("group_id"),
                "group_name": g.get::<String, _>("group_name"),
                "user_type": g.get::<i32, _>("user_type"),
                "permission": g.get::<Option<String>, _>("permission"),
            })
        }))
    }

    /// 查询所有用户组
    pub async fn query_all_user_groups(&self) -> Result<Vec<serde_json::Value>> {
        let groups = sqlx::query(
            "SELECT group_id, group_name, user_type, permission
             FROM truck_scale_user_groups
             WHERE status = 0
             ORDER BY group_id",
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(groups
            .into_iter()
            .map(|g| {
                serde_json::json!({
                    "group_id": g.get::<String, _>("group_id"),
                    "group_name": g.get::<String, _>("group_name"),
                    "user_type": g.get::<i32, _>("user_type"),
                    "permission": g.get::<Option<String>, _>("permission"),
                })
            })
            .collect())
    }

    /// 添加用户组
    pub async fn add_user_group(&self, group_data: serde_json::Value) -> Result<String> {
        let group_id = group_data["group_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing group_id"))?;
        let group_name = group_data["group_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing group_name"))?;

        sqlx::query(
            "INSERT INTO truck_scale_user_groups (
                group_id, group_name, user_type, permission, create_by
            ) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(group_id)
        .bind(group_name)
        .bind(group_data["user_type"].as_i64().unwrap_or(3) as i32)
        .bind(group_data["permission"].as_str())
        .bind(group_data["create_by"].as_str())
        .execute(&*self.pool)
        .await?;

        Ok(group_id.to_string())
    }

    /// 更新用户组
    pub async fn update_user_group(&self, group_data: serde_json::Value) -> Result<()> {
        let group_id = group_data["group_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing group_id"))?;

        sqlx::query(
            "UPDATE truck_scale_user_groups SET
                group_name = $2, user_type = $3, permission = $4, update_by = $5
            WHERE group_id = $1 AND status = 0",
        )
        .bind(group_id)
        .bind(group_data["group_name"].as_str())
        .bind(group_data["user_type"].as_i64().unwrap_or(3) as i32)
        .bind(group_data["permission"].as_str())
        .bind(group_data["update_by"].as_str())
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// 删除用户组(软删除)
    pub async fn delete_user_group(&self, group_id: &str, delete_by: &str) -> Result<()> {
        sqlx::query(
            "UPDATE truck_scale_user_groups 
             SET status = 1, update_by = $2, update_time = CURRENT_TIMESTAMP
             WHERE group_id = $1 AND status = 0",
        )
        .bind(group_id)
        .bind(delete_by)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
}
