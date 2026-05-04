//! 统一调度服务
//!
//! 提供车载终端、无人机、对讲机的统一调度和管理功能

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tracing::{info, warn};

pub mod routes;

/// 设备类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Vehicle,
    Drone,
    Radio,
}

/// 设备状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Online,
    Offline,
    Busy,
    Idle,
    Charging,
    Fault,
}

/// 位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub speed: Option<f64>,
    pub heading: Option<f64>,
}

/// 统一设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDevice {
    pub id: i64,
    pub device_type: DeviceType,
    pub name: String,
    pub status: DeviceStatus,
    pub location: Option<LocationInfo>,
    pub battery: Option<f64>,
    pub signal: Option<f64>,
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
    pub attributes: serde_json::Value,
}

/// 指令类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandType {
    Track,
    VideoStream,
    VoiceCall,
    GroupCall,
    Message,
    ReturnHome,
    EmergencyStop,
    PositionQuery,
}

/// 指令状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Pending,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

/// 调度指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchCommand {
    pub id: String,
    pub command_type: CommandType,
    pub target_devices: Vec<i64>,
    pub target_type: DeviceType,
    pub parameters: serde_json::Value,
    pub status: CommandStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 调度组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchGroup {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub devices: Vec<UnifiedDevice>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 统一调度服务
pub struct UnifiedDispatchService {
    pool: PgPool,
    drone_service: Arc<crate::devices::drones::DroneService>,
    radio_service: Arc<crate::devices::radios::RadioService>,
}

impl UnifiedDispatchService {
    pub fn new(pool: PgPool) -> Self {
        let drone_service = Arc::new(crate::devices::drones::DroneService::new(pool.clone()));
        let radio_service = Arc::new(crate::devices::radios::RadioService::new(pool.clone()));
        Self {
            pool,
            drone_service,
            radio_service,
        }
    }

    /// 获取所有设备
    pub async fn get_online_devices(
        &self,
    ) -> Result<Vec<UnifiedDevice>, Box<dyn std::error::Error + Send + Sync>> {
        let mut devices = Vec::new();

        // 查询真实车辆数据
        let vehicles_result = sqlx::query(
            "SELECT id, license_plate, status, lat, lon, speed, heading, battery_level, signal_strength, last_update_time 
             FROM vehicles WHERE status = 'online' LIMIT 100"
        )
        .fetch_all(&self.pool)
        .await;

        if let Ok(rows) = vehicles_result {
            for row in rows {
                let id: i64 = row.get("id");
                let name: Option<String> = row.get("license_plate");
                let status: Option<String> = row.get("status");
                let lat: Option<f64> = row.get("lat");
                let lon: Option<f64> = row.get("lon");
                let speed: Option<f32> = row.get("speed");
                let heading: Option<f32> = row.get("heading");
                let battery: Option<f32> = row.get("battery_level");
                let signal: Option<f32> = row.get("signal_strength");
                let last_update: Option<chrono::DateTime<chrono::Utc>> =
                    row.get("last_update_time");

                devices.push(UnifiedDevice {
                    id,
                    device_type: DeviceType::Vehicle,
                    name: name.unwrap_or_else(|| format!("Vehicle-{}", id)),
                    status: match status.as_deref().unwrap_or("offline") {
                        "online" => DeviceStatus::Online,
                        "busy" => DeviceStatus::Busy,
                        _ => DeviceStatus::Idle,
                    },
                    location: if let (Some(lat), Some(lon)) = (lat, lon) {
                        Some(LocationInfo {
                            latitude: lat,
                            longitude: lon,
                            altitude: None,
                            speed: speed.map(|s| s as f64),
                            heading: heading.map(|h| h as f64),
                        })
                    } else {
                        None
                    },
                    battery: battery.map(|b| b as f64),
                    signal: signal.map(|s| s as f64),
                    last_update,
                    attributes: serde_json::json!({}),
                });
            }
        } else {
            warn!("Failed to query vehicles from database");
        }

        // 查询真实无人机数据
        match self.drone_service.get_online_drones().await {
            Ok(drones) => {
                for drone in drones {
                    let telemetry = drone.telemetry.as_ref();
                    devices.push(UnifiedDevice {
                        id: drone.id,
                        device_type: DeviceType::Drone,
                        name: drone.name,
                        status: match drone.status.as_str() {
                            "online" => DeviceStatus::Online,
                            _ => DeviceStatus::Offline,
                        },
                        location: telemetry.map(|t| LocationInfo {
                            latitude: t.latitude,
                            longitude: t.longitude,
                            altitude: Some(t.altitude),
                            speed: Some(t.speed),
                            heading: Some(t.heading),
                        }),
                        battery: telemetry.map(|t| t.battery_percent),
                        signal: telemetry.map(|t| t.signal_strength),
                        last_update: telemetry.map(|t| t.timestamp),
                        attributes: serde_json::json!({
                            "vendor": format!("{:?}", drone.vendor),
                            "model": drone.model,
                            "serial_number": drone.serial_number,
                            "firmware_version": drone.firmware_version,
                            "registration_code": drone.registration_code,
                        }),
                    });
                }
            }
            Err(e) => {
                warn!("Failed to query drones: {}", e);
            }
        }

        // 查询真实对讲机数据
        match self.radio_service.get_online_radios().await {
            Ok(radios) => {
                for radio in radios {
                    let telemetry = radio.telemetry.as_ref();
                    devices.push(UnifiedDevice {
                        id: radio.id,
                        device_type: DeviceType::Radio,
                        name: radio.name,
                        status: match radio.status.as_str() {
                            "online" => DeviceStatus::Online,
                            _ => DeviceStatus::Offline,
                        },
                        location: None, // 对讲机通常没有GPS定位
                        battery: telemetry.map(|t| t.battery_percent),
                        signal: telemetry.map(|t| t.signal_strength),
                        last_update: telemetry.map(|t| t.timestamp),
                        attributes: serde_json::json!({
                            "vendor": format!("{:?}", radio.vendor),
                            "model": radio.model,
                            "radio_id": radio.radio_id,
                            "mode": format!("{:?}", radio.mode),
                            "channel": telemetry.map(|t| t.channel.clone()).unwrap_or_default(),
                            "frequency": telemetry.map(|t| t.frequency).unwrap_or(0.0),
                        }),
                    });
                }
            }
            Err(e) => {
                warn!("Failed to query radios: {}", e);
            }
        }

        Ok(devices)
    }

    pub async fn get_dispatch_groups(
        &self,
    ) -> Result<Vec<DispatchGroup>, Box<dyn std::error::Error + Send + Sync>> {
        // 从数据库查询调度组
        let groups_result = sqlx::query(
            "SELECT id, name, description, created_at FROM dispatch_groups ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await;

        if let Ok(rows) = groups_result {
            let mut groups = Vec::new();
            for row in rows {
                let id: i64 = row.get("id");
                let name: String = row.get("name");
                let description: Option<String> = row.try_get("description").ok();
                let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

                groups.push(DispatchGroup {
                    id,
                    name,
                    description,
                    devices: vec![],
                    created_at,
                });
            }
            return Ok(groups);
        }

        Ok(vec![])
    }

    pub async fn send_command(
        &self,
        command_type: CommandType,
        target_devices: Vec<i64>,
        target_type: DeviceType,
        parameters: serde_json::Value,
    ) -> Result<DispatchCommand, Box<dyn std::error::Error + Send + Sync>> {
        // 对无人机使用专用命令接口
        if target_type == DeviceType::Drone {
            for drone_id in &target_devices {
                let drone_command = match command_type {
                    CommandType::ReturnHome => crate::devices::drones::DroneCommand::Rth,
                    CommandType::EmergencyStop => crate::devices::drones::DroneCommand::Hover,
                    CommandType::Track => crate::devices::drones::DroneCommand::ResumeMission,
                    CommandType::VideoStream => {
                        crate::devices::drones::DroneCommand::CameraStartRecord
                    }
                    _ => crate::devices::drones::DroneCommand::Hover,
                };

                if let Err(e) = self
                    .drone_service
                    .send_command(*drone_id, drone_command)
                    .await
                {
                    warn!("Failed to send command to drone {}: {}", drone_id, e);
                }
            }
        }

        // 对对讲机使用专用命令接口
        if target_type == DeviceType::Radio {
            for radio_id in &target_devices {
                let radio_command = match command_type {
                    CommandType::VoiceCall => {
                        crate::devices::radios::RadioCommand::Transmit { message: None }
                    }
                    CommandType::GroupCall => {
                        if let Some(group_id) = parameters.get("group_id").and_then(|v| v.as_str())
                        {
                            crate::devices::radios::RadioCommand::GroupCall {
                                group_id: group_id.to_string(),
                            }
                        } else {
                            crate::devices::radios::RadioCommand::Transmit { message: None }
                        }
                    }
                    CommandType::Message => {
                        if let Some(msg) = parameters.get("message").and_then(|v| v.as_str()) {
                            crate::devices::radios::RadioCommand::Transmit {
                                message: Some(msg.to_string()),
                            }
                        } else {
                            crate::devices::radios::RadioCommand::Transmit { message: None }
                        }
                    }
                    _ => crate::devices::radios::RadioCommand::Receive,
                };

                if let Err(e) = self
                    .radio_service
                    .send_command(*radio_id, radio_command)
                    .await
                {
                    warn!("Failed to send command to radio {}: {}", radio_id, e);
                }
            }
        }

        // 存储调度指令到数据库
        let command_id = format!("cmd-{}", uuid::Uuid::new_v4());
        let cmd_str = serde_json::to_string(&command_type)?;

        sqlx::query(
            "INSERT INTO dispatch_commands (id, command_type, target_devices, target_type, parameters, status, created_at)
             VALUES ($1, $2, $3, $4, $5, 'pending', NOW())"
        )
        .bind(&command_id)
        .bind(&cmd_str)
        .bind(&serde_json::to_string(&target_devices)?)
        .bind(format!("{:?}", target_type))
        .bind(&serde_json::to_string(&parameters)?)
        .execute(&self.pool)
        .await?;

        let command = DispatchCommand {
            id: command_id,
            command_type,
            target_devices,
            target_type,
            parameters,
            status: CommandStatus::Pending,
            created_at: chrono::Utc::now(),
            executed_at: None,
        };

        info!("Dispatch command sent: {:?}", command);
        Ok(command)
    }

    pub async fn get_command_status(
        &self,
        command_id: &str,
    ) -> Result<Option<DispatchCommand>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query(
            "SELECT id, command_type, target_devices, target_type, parameters, status, created_at, executed_at
             FROM dispatch_commands WHERE id = $1"
        )
        .bind(command_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = result {
            let target_devices: Vec<i64> =
                serde_json::from_str(row.get::<String, _>("target_devices").as_str())?;
            let target_type: DeviceType =
                serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("target_type")))?;
            let parameters: serde_json::Value =
                serde_json::from_str(row.get::<String, _>("parameters").as_str())?;
            let command_type: CommandType =
                serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("command_type")))?;
            let status: CommandStatus =
                serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("status")))?;
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let executed_at: Option<chrono::DateTime<chrono::Utc>> =
                row.try_get("executed_at").ok();

            return Ok(Some(DispatchCommand {
                id: command_id.to_string(),
                command_type,
                target_devices,
                target_type,
                parameters,
                status,
                created_at,
                executed_at,
            }));
        }

        Ok(None)
    }
}
