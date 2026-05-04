//! 对讲机设备管理模块
//!
//! 支持数字对讲机协议：DMR、PDT、TETRA等

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use tracing::info;

/// 对讲机厂商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RadioVendor {
    Motorola, // 摩托罗拉 - DMR
    Hytera,   // 海能达 - DMR/PDT
    Sepura,   // 塞普拉 - TETRA
    Tait,     // 泰特 - DMR
    Icom,     // 爱康 - 模拟/数字
    Kenwood,  // 建伍 - 模拟/数字
    Other(String),
}

/// 对讲机工作模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RadioMode {
    Analog,  // 模拟模式
    Digital, // 数字模式
    Mixed,   // 混合模式
}

/// 对讲机状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RadioStatus {
    Idle,         // 空闲
    Transmitting, // 发射中
    Receiving,    // 接收中
    Scanning,     // 扫描中
    Emergency,    // 紧急呼叫
    Offline,      // 离线
}

/// 对讲机遥测数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioTelemetry {
    pub battery_percent: f64,
    pub battery_voltage: f64,
    pub signal_strength: f64, // 信号强度 (dBm)
    pub channel: String,      // 当前频道
    pub frequency: f64,       // 当前频率 (MHz)
    pub squelch_level: i32,   // 静噪等级
    pub volume_level: i32,    // 音量等级
    pub radio_status: RadioStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 对讲机信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioInfo {
    pub id: i64,
    pub name: String,
    pub serial_number: String,
    pub vendor: RadioVendor,
    pub model: String,
    pub firmware_version: String,
    pub radio_id: String, // 对讲机ID/呼号
    pub mode: RadioMode,
    pub status: String, // online/offline/maintenance
    pub telemetry: Option<RadioTelemetry>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_telemetry: Option<chrono::DateTime<chrono::Utc>>,
}

/// 对讲机指令
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadioCommand {
    SetChannel { channel: String },
    SetFrequency { frequency: f64 },
    SetVolume { level: i32 },
    SetSquelch { level: i32 },
    Transmit { message: Option<String> }, // 发射语音或消息
    Receive,                              // 接收模式
    EmergencyCall,                        // 紧急呼叫
    GroupCall { group_id: String },       // 组呼
    PrivateCall { target_id: String },    // 个呼
    ScanChannels,                         // 频道扫描
    StopScan,                             // 停止扫描
    Mute,                                 // 静音
    Unmute,                               // 取消静音
}

/// 对讲机服务
pub struct RadioService {
    pool: PgPool,
}

impl RadioService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有在线对讲机
    pub async fn get_online_radios(
        &self,
    ) -> Result<Vec<RadioInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = sqlx::query(
            "SELECT r.id, r.name, r.serial_number, r.vendor, r.model,
                    r.firmware_version, r.radio_id, r.mode, r.status,
                    r.created_at, t.timestamp as last_telemetry
             FROM radios r
             LEFT JOIN radio_telemetry t ON r.id = t.radio_id
             WHERE r.status = 'online'
             ORDER BY r.name",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut radios = Vec::new();
        for row in rows {
            let id: i64 = row.get("id");
            let name: String = row.get("name");
            let serial_number: String = row.get("serial_number");
            let vendor_str: String = row.get("vendor");
            let model: String = row.get("model");
            let firmware_version: String = row.get("firmware_version");
            let radio_id: String = row.get("radio_id");
            let mode_str: String = row.get("mode");
            let status: String = row.get("status");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let last_telemetry: Option<chrono::DateTime<chrono::Utc>> = row.get("last_telemetry");

            // 获取最新遥测数据
            let telemetry = self.get_latest_telemetry(id).await.ok();

            radios.push(RadioInfo {
                id,
                name,
                serial_number,
                vendor: parse_vendor(&vendor_str),
                model,
                firmware_version,
                radio_id,
                mode: parse_mode(&mode_str),
                status,
                telemetry,
                created_at,
                last_telemetry,
            });
        }

        Ok(radios)
    }

    /// 获取最新遥测数据
    pub async fn get_latest_telemetry(
        &self,
        radio_id: i64,
    ) -> Result<RadioTelemetry, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "SELECT battery_percent, battery_voltage, signal_strength,
                    channel, frequency, squelch_level, volume_level,
                    radio_status, timestamp
             FROM radio_telemetry
             WHERE radio_id = $1
             ORDER BY timestamp DESC
             LIMIT 1",
        )
        .bind(radio_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(RadioTelemetry {
            battery_percent: row.get("battery_percent"),
            battery_voltage: row.get("battery_voltage"),
            signal_strength: row.get("signal_strength"),
            channel: row.get("channel"),
            frequency: row.get("frequency"),
            squelch_level: row.get("squelch_level"),
            volume_level: row.get("volume_level"),
            radio_status: parse_radio_status(row.get::<String, _>("radio_status").as_str()),
            timestamp: row.get("timestamp"),
        })
    }

    /// 发送对讲机指令
    pub async fn send_command(
        &self,
        radio_id: i64,
        command: RadioCommand,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let cmd_str = serde_json::to_string(&command)?;
        let command_id = format!("radio-cmd-{}", uuid::Uuid::new_v4());

        // 存储指令到数据库
        sqlx::query(
            "INSERT INTO radio_commands (id, radio_id, command, status, created_at)
             VALUES ($1, $2, $3, 'pending', NOW())",
        )
        .bind(&command_id)
        .bind(radio_id)
        .bind(&cmd_str)
        .execute(&self.pool)
        .await?;

        info!(
            "Radio command {} sent to radio {}: {:?}",
            command_id, radio_id, command
        );
        Ok(command_id)
    }

    /// 获取对讲机详情
    pub async fn get_radio(
        &self,
        radio_id: i64,
    ) -> Result<Option<RadioInfo>, Box<dyn std::error::Error + Send + Sync>> {
        let row = sqlx::query(
            "SELECT id, name, serial_number, vendor, model,
                    firmware_version, radio_id, mode, status, created_at
             FROM radios WHERE id = $1",
        )
        .bind(radio_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: i64 = row.get("id");
            let name: String = row.get("name");
            let serial_number: String = row.get("serial_number");
            let vendor_str: String = row.get("vendor");
            let model: String = row.get("model");
            let firmware_version: String = row.get("firmware_version");
            let radio_id_str: String = row.get("radio_id");
            let mode_str: String = row.get("mode");
            let status: String = row.get("status");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

            let telemetry = self.get_latest_telemetry(id).await.ok();
            let last_telemetry = telemetry.as_ref().map(|t| t.timestamp);

            Ok(Some(RadioInfo {
                id,
                name,
                serial_number,
                vendor: parse_vendor(&vendor_str),
                model,
                firmware_version,
                radio_id: radio_id_str,
                mode: parse_mode(&mode_str),
                status,
                telemetry,
                created_at,
                last_telemetry,
            }))
        } else {
            Ok(None)
        }
    }

    /// 创建新对讲机
    pub async fn create_radio(
        &self,
        radio: &RadioInfo,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let vendor_str = serde_json::to_string(&radio.vendor)?;
        let mode_str = serde_json::to_string(&radio.mode)?;

        let row = sqlx::query(
            "INSERT INTO radios (name, serial_number, vendor, model, firmware_version,
                               radio_id, mode, status, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
             RETURNING id",
        )
        .bind(&radio.name)
        .bind(&radio.serial_number)
        .bind(&vendor_str)
        .bind(&radio.model)
        .bind(&radio.firmware_version)
        .bind(&radio.radio_id)
        .bind(&mode_str)
        .bind(&radio.status)
        .fetch_one(&self.pool)
        .await?;

        let id: i64 = row.get("id");
        info!("Created radio {}: {}", id, radio.name);
        Ok(id)
    }

    /// 更新对讲机状态
    pub async fn update_radio_status(
        &self,
        radio_id: i64,
        status: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query("UPDATE radios SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(radio_id)
            .execute(&self.pool)
            .await?;

        info!("Updated radio {} status to {}", radio_id, status);
        Ok(())
    }

    /// 更新对讲机遥测数据
    pub async fn update_telemetry(
        &self,
        radio_id: i64,
        telemetry: &RadioTelemetry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status_str = serde_json::to_string(&telemetry.radio_status)?;

        sqlx::query(
            "INSERT INTO radio_telemetry (radio_id, battery_percent, battery_voltage,
                                        signal_strength, channel, frequency,
                                        squelch_level, volume_level, radio_status, timestamp)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(radio_id)
        .bind(telemetry.battery_percent)
        .bind(telemetry.battery_voltage)
        .bind(telemetry.signal_strength)
        .bind(&telemetry.channel)
        .bind(telemetry.frequency)
        .bind(telemetry.squelch_level)
        .bind(telemetry.volume_level)
        .bind(&status_str)
        .bind(telemetry.timestamp)
        .execute(&self.pool)
        .await?;

        info!("Updated telemetry for radio {}", radio_id);
        Ok(())
    }
}

/// 解析厂商字符串
fn parse_vendor(vendor_str: &str) -> RadioVendor {
    match vendor_str.to_lowercase().as_str() {
        "motorola" => RadioVendor::Motorola,
        "hytera" => RadioVendor::Hytera,
        "sepura" => RadioVendor::Sepura,
        "tait" => RadioVendor::Tait,
        "icom" => RadioVendor::Icom,
        "kenwood" => RadioVendor::Kenwood,
        _ => RadioVendor::Other(vendor_str.to_string()),
    }
}

/// 解析工作模式字符串
fn parse_mode(mode_str: &str) -> RadioMode {
    match mode_str.to_lowercase().as_str() {
        "analog" => RadioMode::Analog,
        "digital" => RadioMode::Digital,
        "mixed" => RadioMode::Mixed,
        _ => RadioMode::Digital, // 默认数字模式
    }
}

/// 解析对讲机状态字符串 (pub for testing)
pub fn parse_radio_status(status_str: &str) -> RadioStatus {
    match status_str.to_lowercase().as_str() {
        "idle" => RadioStatus::Idle,
        "transmitting" => RadioStatus::Transmitting,
        "receiving" => RadioStatus::Receiving,
        "scanning" => RadioStatus::Scanning,
        "emergency" => RadioStatus::Emergency,
        "offline" => RadioStatus::Offline,
        _ => RadioStatus::Idle,
    }
}
