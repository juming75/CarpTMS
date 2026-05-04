//! 无人机设备管理模块
//!
//! 支持大疆DJI Cloud API、PX4/MAVLink、道通Autel协议

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use tracing::info;

/// 无人机厂商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DroneVendor {
    Dji,   // 大疆 - GB28181/DJI Cloud API
    Autel, // 道通 - GB28181/Autel Cloud
    Px4,   // PX4飞控 - MAVLink
    Other(String),
}

/// 无人机飞行状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FlightStatus {
    Ground,       // 地面
    Takeoff,      // 起飞中
    Landing,      // 降落中
    Hover,        // 悬停
    Cruising,     // 巡航
    Rth,          // 返航中
    Emergency,    // 紧急状态
    Disconnected, // 失联
}

/// 无人机遥测数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneTelemetry {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,     // 相对高度 (m)
    pub altitude_agl: f64, // 离地高度 (m)
    pub speed: f64,        // 速度 (m/s)
    pub heading: f64,      // 航向 (度)
    pub battery_percent: f64,
    pub battery_voltage: f64,
    pub battery_current: f64,
    pub satellite_count: i32,
    pub signal_strength: f64,
    pub flight_status: FlightStatus,
    pub flight_mode: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 无人机信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneInfo {
    pub id: i64,
    pub name: String,
    pub serial_number: String,
    pub vendor: DroneVendor,
    pub model: String,
    pub firmware_version: String,
    pub registration_code: String, // 民航局注册号
    pub status: String,            // online/offline/maintenance
    pub telemetry: Option<DroneTelemetry>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_telemetry: Option<chrono::DateTime<chrono::Utc>>,
}

/// 无人机飞行指令
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DroneCommand {
    Takeoff {
        altitude: f64,
    },
    Land,
    Rth, // Return to Home
    Hover,
    SetSpeed {
        speed: f64,
    },
    SetAltitude {
        altitude: f64,
    },
    GotoWaypoint {
        latitude: f64,
        longitude: f64,
        altitude: f64,
    },
    StartMission {
        mission_id: i64,
    },
    PauseMission,
    ResumeMission,
    GimbalControl {
        pitch: f64,
        yaw: f64,
        roll: f64,
    },
    CameraCapture,
    CameraStartRecord,
    CameraStopRecord,
}

/// 无人机服务
pub struct DroneService {
    pool: PgPool,
}

impl DroneService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有在线无人机
    pub async fn get_online_drones(
        &self,
    ) -> Result<Vec<DroneInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT d.id, d.name, d.serial_number, d.vendor, d.model, 
                    d.firmware_version, d.registration_code, d.status,
                    d.created_at, t.timestamp as last_telemetry
             FROM drones d
             LEFT JOIN drone_telemetry t ON d.id = t.drone_id
             WHERE d.status = 'online'
             ORDER BY d.name",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut drones = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let name: String = row.get("name");
            let serial_number: String = row.get("serial_number");
            let vendor_str: String = row.get("vendor");
            let model: String = row.get("model");
            let firmware_version: String = row.get("firmware_version");
            let registration_code: String = row.get("registration_code");
            let status: String = row.get("status");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let last_telemetry: Option<chrono::DateTime<chrono::Utc>> = row.get("last_telemetry");

            // 获取最新遥测数据
            let telemetry = self.get_latest_telemetry(id).await.ok();

            drones.push(DroneInfo {
                id,
                name,
                serial_number,
                vendor: parse_vendor(&vendor_str),
                model,
                firmware_version,
                registration_code,
                status,
                telemetry,
                created_at,
                last_telemetry,
            });
        }

        Ok(drones)
    }

    /// 获取最新遥测数据
    pub async fn get_latest_telemetry(
        &self,
        drone_id: i64,
    ) -> Result<DroneTelemetry, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "SELECT latitude, longitude, altitude, altitude_agl, speed, heading,
                    battery_percent, battery_voltage, battery_current,
                    satellite_count, signal_strength, flight_status, flight_mode, timestamp
             FROM drone_telemetry
             WHERE drone_id = $1
             ORDER BY timestamp DESC
             LIMIT 1",
        )
        .bind(drone_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DroneTelemetry {
            latitude: row.get("latitude"),
            longitude: row.get("longitude"),
            altitude: row.get("altitude"),
            altitude_agl: row.get("altitude_agl"),
            speed: row.get("speed"),
            heading: row.get("heading"),
            battery_percent: row.get("battery_percent"),
            battery_voltage: row.get("battery_voltage"),
            battery_current: row.get("battery_current"),
            satellite_count: row.get("satellite_count"),
            signal_strength: row.get("signal_strength"),
            flight_status: parse_flight_status(row.get::<String, _>("flight_status").as_str()),
            flight_mode: row.get("flight_mode"),
            timestamp: row.get("timestamp"),
        })
    }

    /// 发送无人机指令
    pub async fn send_command(
        &self,
        drone_id: i64,
        command: DroneCommand,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let cmd_str = serde_json::to_string(&command)?;
        let command_id = format!("cmd-{}", uuid::Uuid::new_v4());

        // 存储指令到数据库
        sqlx::query(
            "INSERT INTO drone_commands (id, drone_id, command, status, created_at)
             VALUES ($1, $2, $3, 'pending', NOW())",
        )
        .bind(&command_id)
        .bind(drone_id)
        .bind(&cmd_str)
        .execute(&self.pool)
        .await?;

        info!(
            "Drone command {} sent to drone {}: {:?}",
            command_id, drone_id, command
        );
        Ok(command_id)
    }

    /// 更新遥测数据（由设备接入层调用）
    pub async fn update_telemetry(
        &self,
        drone_id: i64,
        telemetry: DroneTelemetry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query(
            "INSERT INTO drone_telemetry 
             (drone_id, latitude, longitude, altitude, altitude_agl, speed, heading,
              battery_percent, battery_voltage, battery_current,
              satellite_count, signal_strength, flight_status, flight_mode, timestamp)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
        )
        .bind(drone_id)
        .bind(telemetry.latitude)
        .bind(telemetry.longitude)
        .bind(telemetry.altitude)
        .bind(telemetry.altitude_agl)
        .bind(telemetry.speed)
        .bind(telemetry.heading)
        .bind(telemetry.battery_percent)
        .bind(telemetry.battery_voltage)
        .bind(telemetry.battery_current)
        .bind(telemetry.satellite_count)
        .bind(telemetry.signal_strength)
        .bind(format!("{:?}", telemetry.flight_status))
        .bind(&telemetry.flight_mode)
        .bind(telemetry.timestamp)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

pub fn parse_vendor(s: &str) -> DroneVendor {
    match s.to_lowercase().as_str() {
        "dji" => DroneVendor::Dji,
        "autel" => DroneVendor::Autel,
        "px4" => DroneVendor::Px4,
        _ => DroneVendor::Other(s.to_string()),
    }
}

pub fn parse_flight_status(s: &str) -> FlightStatus {
    match s.to_lowercase().as_str() {
        "ground" => FlightStatus::Ground,
        "takeoff" => FlightStatus::Takeoff,
        "landing" => FlightStatus::Landing,
        "hover" => FlightStatus::Hover,
        "cruising" => FlightStatus::Cruising,
        "rth" | "return_to_home" => FlightStatus::Rth,
        "emergency" => FlightStatus::Emergency,
        "disconnected" => FlightStatus::Disconnected,
        _ => FlightStatus::Ground,
    }
}
