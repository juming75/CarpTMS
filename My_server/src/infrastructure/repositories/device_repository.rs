//! / 设备仓库PostgreSQL实现

use anyhow::Result;
use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::domain::entities::device::{Device, DeviceCreate, DeviceQuery, DeviceUpdate};
use crate::domain::use_cases::device::DeviceRepository;

/// 设备仓库PostgreSQL实现
pub struct PgDeviceRepository {
    pool: Arc<PgPool>,
}

impl PgDeviceRepository {
    /// 创建设备仓库实例
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DeviceRepository for PgDeviceRepository {
    /// 获取设备列表
    async fn get_devices(&self, query: DeviceQuery) -> Result<(Vec<Device>, i64), anyhow::Error> {
        // 处理分页参数
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        // 构建动态查询条件
        let mut where_clauses = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        if let Some(device_id) = &query.device_id {
            if !device_id.is_empty() {
                where_clauses.push(format!("device_id LIKE ${}", param_index));
                params.push(format!("%{}%", device_id));
                param_index += 1;
            }
        }

        if let Some(device_name) = &query.device_name {
            if !device_name.is_empty() {
                where_clauses.push(format!("device_name LIKE ${}", param_index));
                params.push(format!("%{}%", device_name));
                param_index += 1;
            }
        }

        if let Some(device_type) = &query.device_type {
            if !device_type.is_empty() {
                where_clauses.push(format!("device_type = ${}", param_index));
                params.push(device_type.clone());
                param_index += 1;
            }
        }

        if let Some(manufacturer) = &query.manufacturer {
            if !manufacturer.is_empty() {
                where_clauses.push(format!("manufacturer LIKE ${}", param_index));
                params.push(format!("%{}%", manufacturer));
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
        let count_query = format!("SELECT COUNT(*) FROM devices {}", where_sql);
        let mut count_sqlx_query = sqlx::query_scalar::<_, i64>(&count_query);

        // 绑定参数
        for param in &params {
            count_sqlx_query = count_sqlx_query.bind(param);
        }

        let total_count = count_sqlx_query.fetch_one(&*self.pool).await?;

        // 查询分页数据 - 优化：使用明确的列名
        let data_query = format!(
            r#"
            SELECT 
                device_id, device_name, device_type, device_model, manufacturer,
                serial_number, communication_type, sim_card_no, ip_address,
                port, mac_address, install_date, install_address,
                install_technician, status, remark, create_user_id, update_user_id,
                create_time, update_time
            FROM devices {} 
             ORDER BY device_id DESC 
             LIMIT ${} OFFSET ${}"#,
            where_sql,
            param_index,
            param_index + 1
        );

        // 使用query方法执行动态查询
        let mut sqlx_query = sqlx::query(&data_query);

        // 绑定参数
        for param in params {
            sqlx_query = sqlx_query.bind(param);
        }
        sqlx_query = sqlx_query.bind(page_size).bind(offset);

        let devices = sqlx_query
            .fetch_all(&*self.pool)
            .await?
            .into_iter()
            .map(|row| Device {
                device_id: row.try_get("device_id").unwrap_or_default(),
                device_name: row.try_get("device_name").unwrap_or_default(),
                device_type: row.try_get("device_type").unwrap_or_default(),
                device_model: row.try_get("device_model").unwrap_or_default(),
                manufacturer: row.try_get("manufacturer").unwrap_or_default(),
                serial_number: row.try_get("serial_number").unwrap_or_default(),
                communication_type: row.try_get("communication_type").unwrap_or_default(),
                sim_card_no: row.try_get("sim_card_no").ok(),
                ip_address: row.try_get("ip_address").ok(),
                port: row.try_get("port").ok(),
                mac_address: row.try_get("mac_address").ok(),
                install_date: row.try_get("install_date").ok(),
                install_address: row.try_get("install_address").ok(),
                install_technician: row.try_get("install_technician").ok(),
                status: row.try_get("status").unwrap_or(1),
                remark: row.try_get("remark").ok(),
                create_time: row
                    .try_get("create_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: row.try_get("update_time").ok(),
                create_user_id: row.try_get("create_user_id").unwrap_or(1),
                update_user_id: row.try_get("update_user_id").ok(),
            })
            .collect::<Vec<Device>>();

        Ok((devices, total_count))
    }

    /// 获取单个设备 - 优化版：使用明确的列选择
    async fn get_device(&self, device_id: &str) -> Result<Option<Device>, anyhow::Error> {
        let device = sqlx::query(
            r#"
            SELECT 
                device_id, device_name, device_type, device_model, manufacturer,
                serial_number, communication_type, sim_card_no, ip_address,
                port, mac_address, install_date, install_address,
                install_technician, status, remark, create_user_id, update_user_id,
                create_time, update_time
            FROM devices WHERE device_id = $1"#,
        )
        .bind(device_id)
        .fetch_optional(&*self.pool)
        .await?
        .map(|row| Device {
            device_id: row.try_get("device_id").unwrap_or_default(),
            device_name: row.try_get("device_name").unwrap_or_default(),
            device_type: row.try_get("device_type").unwrap_or_default(),
            device_model: row.try_get("device_model").unwrap_or_default(),
            manufacturer: row.try_get("manufacturer").unwrap_or_default(),
            serial_number: row.try_get("serial_number").unwrap_or_default(),
            communication_type: row.try_get("communication_type").unwrap_or_default(),
            sim_card_no: row.try_get("sim_card_no").ok(),
            ip_address: row.try_get("ip_address").ok(),
            port: row.try_get("port").ok(),
            mac_address: row.try_get("mac_address").ok(),
            install_date: row.try_get("install_date").ok(),
            install_address: row.try_get("install_address").ok(),
            install_technician: row.try_get("install_technician").ok(),
            status: row.try_get("status").unwrap_or(1),
            remark: row.try_get("remark").ok(),
            create_time: row
                .try_get("create_time")
                .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            update_time: row.try_get("update_time").ok(),
            create_user_id: row.try_get("create_user_id").unwrap_or(1),
            update_user_id: row.try_get("update_user_id").ok(),
        });

        Ok(device)
    }

    /// 创建设备
    async fn create_device(&self, device: DeviceCreate) -> Result<Device, anyhow::Error> {
        let result = sqlx::query(
            r#"INSERT INTO devices (
                device_id, device_name, device_type, device_model, manufacturer,
                serial_number, communication_type, sim_card_no, ip_address,
                port, mac_address, install_date, install_address,
                install_technician, status, remark, create_user_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
            ) RETURNING *"#,
        )
        .bind(&device.device_id)
        .bind(&device.device_name)
        .bind(&device.device_type)
        .bind(&device.device_model)
        .bind(&device.manufacturer)
        .bind(&device.serial_number)
        .bind(&device.communication_type)
        .bind(&device.sim_card_no)
        .bind(&device.ip_address)
        .bind(device.port)
        .bind(&device.mac_address)
        .bind(device.install_date)
        .bind(&device.install_address)
        .bind(&device.install_technician)
        .bind(device.status)
        .bind(&device.remark)
        .bind(device.create_user_id)
        .fetch_one(&*self.pool)
        .await;

        let d = match result {
            Ok(row) => Device {
                device_id: row.try_get("device_id").unwrap_or_default(),
                device_name: row.try_get("device_name").unwrap_or_default(),
                device_type: row.try_get("device_type").unwrap_or_default(),
                device_model: row.try_get("device_model").unwrap_or_default(),
                manufacturer: row.try_get("manufacturer").unwrap_or_default(),
                serial_number: row.try_get("serial_number").unwrap_or_default(),
                communication_type: row.try_get("communication_type").unwrap_or_default(),
                sim_card_no: row.try_get("sim_card_no").ok(),
                ip_address: row.try_get("ip_address").ok(),
                port: row.try_get("port").ok(),
                mac_address: row.try_get("mac_address").ok(),
                install_date: row.try_get("install_date").ok(),
                install_address: row.try_get("install_address").ok(),
                install_technician: row.try_get("install_technician").ok(),
                status: row.try_get("status").unwrap_or(1),
                remark: row.try_get("remark").ok(),
                create_time: row
                    .try_get("create_time")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                update_time: row.try_get("update_time").ok(),
                create_user_id: row.try_get("create_user_id").unwrap_or(1),
                update_user_id: row.try_get("update_user_id").ok(),
            },
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to create device: {:?}", e));
            }
        };

        Ok(d)
    }

    /// 更新设备
    async fn update_device(
        &self,
        device_id: &str,
        device: DeviceUpdate,
    ) -> Result<Option<Device>, anyhow::Error> {
        let result = sqlx::query(
            r#"UPDATE devices 
               SET 
                   device_name = COALESCE($1, device_name),
                   device_type = COALESCE($2, device_type),
                   device_model = COALESCE($3, device_model),
                   manufacturer = COALESCE($4, manufacturer),
                   serial_number = COALESCE($5, serial_number),
                   communication_type = COALESCE($6, communication_type),
                   sim_card_no = COALESCE($7, sim_card_no),
                   ip_address = COALESCE($8, ip_address),
                   port = COALESCE($9, port),
                   mac_address = COALESCE($10, mac_address),
                   install_date = COALESCE($11, install_date),
                   install_address = COALESCE($12, install_address),
                   install_technician = COALESCE($13, install_technician),
                   status = COALESCE($14, status),
                   remark = COALESCE($15, remark),
                   update_user_id = COALESCE($16, update_user_id),
                   update_time = CURRENT_TIMESTAMP 
               WHERE device_id = $17 
               RETURNING *"#,
        )
        .bind(&device.device_name)
        .bind(&device.device_type)
        .bind(&device.device_model)
        .bind(&device.manufacturer)
        .bind(&device.serial_number)
        .bind(&device.communication_type)
        .bind(&device.sim_card_no)
        .bind(&device.ip_address)
        .bind(device.port)
        .bind(&device.mac_address)
        .bind(device.install_date)
        .bind(&device.install_address)
        .bind(&device.install_technician)
        .bind(device.status)
        .bind(&device.remark)
        .bind(device.update_user_id)
        .bind(device_id)
        .fetch_optional(&*self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let device = Device {
                    device_id: row.try_get("device_id").unwrap_or_default(),
                    device_name: row.try_get("device_name").unwrap_or_default(),
                    device_type: row.try_get("device_type").unwrap_or_default(),
                    device_model: row.try_get("device_model").unwrap_or_default(),
                    manufacturer: row.try_get("manufacturer").unwrap_or_default(),
                    serial_number: row.try_get("serial_number").unwrap_or_default(),
                    communication_type: row.try_get("communication_type").unwrap_or_default(),
                    sim_card_no: row.try_get("sim_card_no").ok(),
                    ip_address: row.try_get("ip_address").ok(),
                    port: row.try_get("port").ok(),
                    mac_address: row.try_get("mac_address").ok(),
                    install_date: row.try_get("install_date").ok(),
                    install_address: row.try_get("install_address").ok(),
                    install_technician: row.try_get("install_technician").ok(),
                    status: row.try_get("status").unwrap_or(1),
                    remark: row.try_get("remark").ok(),
                    create_time: row
                        .try_get("create_time")
                        .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                    update_time: row.try_get("update_time").ok(),
                    create_user_id: row.try_get("create_user_id").unwrap_or(1),
                    update_user_id: row.try_get("update_user_id").ok(),
                };

                Ok(Some(device))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Failed to update device: {:?}", e)),
        }
    }

    /// 删除设备
    async fn delete_device(&self, device_id: &str) -> Result<bool, anyhow::Error> {
        // 先删除相关的称重数据
        let _weighing_result = sqlx::query(r#"DELETE FROM weighing_data WHERE device_id = $1"#)
            .bind(device_id)
            .execute(&*self.pool)
            .await?;

        // 再删除设备
        let device_result = sqlx::query(r#"DELETE FROM devices WHERE device_id = $1"#)
            .bind(device_id)
            .execute(&*self.pool)
            .await?;

        Ok(device_result.rows_affected() > 0)
    }
}
