//! / 消息转换器
// 实现 JT808 协议数据与统一消息格式的双向转换

use log::debug;
use std::collections::HashMap;

use super::types::UnifiedMessage;
use crate::protocols::jt808::models::{AlarmType, LocationReport, SensorData};

/// JT808 到统一消息转换器
pub struct JT808ToUnifiedConverter;

impl JT808ToUnifiedConverter {
    /// 转换位置汇报消息 (0x0200)
    pub fn convert_location_report(
        phone: String,
        location: &LocationReport,
    ) -> Result<UnifiedMessage, String> {
        debug!(
            "Converting location report for device: {}, lat={}, lng={}",
            phone, location.latitude, location.longitude
        );

        // 构建传感器数据 payload
        let sensor_payload = Self::build_sensor_data_payload(&location.sensor_data);

        let _payload = serde_json::json!({
            "phone": phone,
            "latitude": location.latitude,
            "longitude": location.longitude,
            "altitude": location.altitude,
            "speed": location.speed,
            "direction": location.direction,
            "timestamp": location.timestamp.to_rfc3339(),
            "alarm_flag": location.alarm_flag,
            "status": location.status,
            "sensor_data": sensor_payload,
        });

        Ok(UnifiedMessage::location_data(
            phone,
            location.latitude,
            location.longitude,
            location.speed,
            location.direction,
            Some(sensor_payload),
        ))
    }

    /// 构建传感器数据 payload
    fn build_sensor_data_payload(sensor: &SensorData) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        map.insert(
            "device_id".to_string(),
            serde_json::Value::String(sensor.device_id.clone()),
        );

        if let Some(fuel) = sensor.fuel {
            map.insert("fuel".to_string(), serde_json::json!(fuel));
        }

        if let Some(water_temp) = sensor.water_temp {
            map.insert("water_temp".to_string(), serde_json::json!(water_temp));
        }

        if let Some(oil_temp) = sensor.oil_temp {
            map.insert("oil_temp".to_string(), serde_json::json!(oil_temp));
        }

        if let Some(engine_rpm) = sensor.engine_rpm {
            map.insert("engine_rpm".to_string(), serde_json::json!(engine_rpm));
        }

        if let Some(load_weight) = sensor.load_weight {
            map.insert("load_weight".to_string(), serde_json::json!(load_weight));
        }

        if let Some(mileage) = sensor.mileage {
            map.insert("mileage".to_string(), serde_json::json!(mileage));
        }

        if let Some(io_status) = sensor.io_status {
            map.insert(
                "io_status".to_string(),
                serde_json::Value::Number(io_status.into()),
            );
        }

        if !sensor.analog_inputs.is_empty() {
            let analog_map: serde_json::Value = sensor
                .analog_inputs
                .iter()
                .map(|(id, value)| (format!("analog_{}", id), serde_json::json!(value)))
                .collect();
            map.insert("analog_inputs".to_string(), analog_map);
        }

        if !sensor.alarms.is_empty() {
            let alarm_list: Vec<String> = sensor
                .alarms
                .iter()
                .map(|alarm| match alarm {
                    AlarmType::Overspeed => "overspeed".to_string(),
                    AlarmType::FatigueDriving => "fatigue_driving".to_string(),
                    AlarmType::EmergencyBrake => "emergency_brake".to_string(),
                    AlarmType::FuelLeakage => "fuel_leakage".to_string(),
                    AlarmType::TemperatureHigh => "temperature_high".to_string(),
                    AlarmType::IOStateChanged => "io_state_changed".to_string(),
                    AlarmType::GpsLost => "gps_lost".to_string(),
                    AlarmType::Custom(s) => s.clone(),
                })
                .collect();
            map.insert(
                "alarms".to_string(),
                serde_json::Value::Array(
                    alarm_list
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }

        serde_json::Value::Object(map)
    }

    /// 转换报警消息 (0x1201)
    pub fn convert_alarm(
        phone: String,
        alarm_type: String,
        alarm_level: i32,
        description: String,
    ) -> Result<UnifiedMessage, String> {
        debug!(
            "Converting alarm for device: {}, type={}",
            phone, alarm_type
        );

        Ok(UnifiedMessage::alarm_data(
            phone,
            alarm_type,
            alarm_level,
            description,
        ))
    }

    /// 转换传感器数据消息
    pub fn convert_sensor_data(
        device_id: String,
        sensor_type: String,
        sensor_value: serde_json::Value,
    ) -> Result<UnifiedMessage, String> {
        debug!(
            "Converting sensor data for device: {}, type={}",
            device_id, sensor_type
        );

        Ok(UnifiedMessage::sensor_data(
            device_id,
            sensor_type,
            sensor_value,
        ))
    }

    /// 转换设备状态消息
    pub fn convert_device_status(
        device_id: String,
        status: String,
        online: bool,
    ) -> Result<UnifiedMessage, String> {
        debug!(
            "Converting device status: {}, status={}, online={}",
            device_id, status, online
        );

        Ok(UnifiedMessage::device_status(device_id, status, online))
    }

    /// 根据报警标志生成报警消息列表
    pub fn extract_alarms_from_flag(alarm_flag: u32) -> Vec<UnifiedMessage> {
        let mut messages = Vec::new();
        let device_id = "unknown".to_string(); // 需要从上下文获取

        // 报警标志位解析 (参考 JT808 标准定义)
        const ALARM_EMERGENCY: u32 = 1 << 0; // 紧急报警
        const ALARM_SPEEDING: u32 = 1 << 1; // 超速报警
        const ALARM_FATIGUE: u32 = 1 << 2; // 疲劳驾驶
        const ALARM_GPS_LOST: u32 = 1 << 4; // GPS 模块发生故障
        const ALARM_GPS_NO_SIGNAL: u32 = 1 << 5; // GPS 天线被剪断或未接
        const ALARM_POWER_OFF: u32 = 1 << 7; // 终端主电源欠压

        if alarm_flag & ALARM_EMERGENCY != 0 {
            messages.push(UnifiedMessage::alarm_data(
                device_id.clone(),
                "emergency".to_string(),
                3,
                "紧急报警".to_string(),
            ));
        }

        if alarm_flag & ALARM_SPEEDING != 0 {
            messages.push(UnifiedMessage::alarm_data(
                device_id.clone(),
                "overspeed".to_string(),
                2,
                "超速报警".to_string(),
            ));
        }

        if alarm_flag & ALARM_FATIGUE != 0 {
            messages.push(UnifiedMessage::alarm_data(
                device_id.clone(),
                "fatigue_driving".to_string(),
                2,
                "疲劳驾驶".to_string(),
            ));
        }

        if alarm_flag & ALARM_GPS_LOST != 0 || alarm_flag & ALARM_GPS_NO_SIGNAL != 0 {
            messages.push(UnifiedMessage::alarm_data(
                device_id.clone(),
                "gps_lost".to_string(),
                2,
                "GPS 信号丢失".to_string(),
            ));
        }

        if alarm_flag & ALARM_POWER_OFF != 0 {
            messages.push(UnifiedMessage::alarm_data(
                device_id.clone(),
                "power_low".to_string(),
                2,
                "主电源欠压".to_string(),
            ));
        }

        messages
    }
}

/// 统一消息到 JT808 转换器
pub struct UnifiedToJT808Converter;

impl UnifiedToJT808Converter {
    /// 转换为 JT808 通用应答 (0x8100)
    pub fn to_0x8100(msg_id: u16, flow_no: u16, result: u8) -> Result<Vec<u8>, String> {
        // JT808 0x8100 平台通用应答格式
        // [应答流水号(2)][应答ID(2)][结果(1)]
        let mut data = Vec::with_capacity(5);

        data.extend_from_slice(&flow_no.to_be_bytes());
        data.extend_from_slice(&msg_id.to_be_bytes());
        data.push(result);

        Ok(data)
    }

    /// 转换为 JT808 终端参数查询 (0x8101)
    pub fn to_0x8101(params: &HashMap<u32, serde_json::Value>) -> Result<Vec<u8>, String> {
        // JT808 0x8101 终端参数查询格式
        // [参数总数(1)][参数ID列表(4*N)]
        let mut data = Vec::new();

        data.push(params.len() as u8);

        for param_id in params.keys() {
            data.extend_from_slice(&param_id.to_be_bytes());
        }

        Ok(data)
    }

    /// 转换为 JT808 终端控制指令 (0x8300)
    pub fn to_0x8300(command_word: u8, _params: &serde_json::Value) -> Result<Vec<u8>, String> {
        // JT808 0x8300 终端控制指令格式
        // [命令字(1)][参数列表]
        let data = vec![command_word];

        // 根据命令字处理不同的参数
        match command_word {
            0x01 => {
                // 终端复位
                // 无额外参数
            }
            0x02 => {
                // 终端关机
                // 无额外参数
            }
            0x03 => {
                // 恢复出厂设置
                // 无额外参数
            }
            0x04 => {
                // 关闭终端网络连接
                // 无额外参数
            }
            0x05 => {
                // 关闭终端所有无线通信
                // 无额外参数
            }
            _ => {
                return Err(format!("Unknown command word: 0x{:02X}", command_word));
            }
        }

        Ok(data)
    }

    /// 转换为 JT808 摄像头立即拍照指令 (0x8801)
    pub fn to_0x8801(
        channel_id: u8,
        save_flag: u8,
        time_interval: u16,
        resolution: u8,
        quality: u8,
        lighting: u8,
        contrast: u8,
    ) -> Result<Vec<u8>, String> {
        // JT808 0x8801 摄像头立即拍照指令格式
        // [通道ID(1)][保存标志(1)][时间间隔(2)][图像/视频质量(1)][分辨率(1)][亮度(1)][对比度(1)]
        let mut data = Vec::with_capacity(7);

        data.push(channel_id);
        data.push(save_flag);
        data.extend_from_slice(&time_interval.to_be_bytes());
        data.push(quality);
        data.push(resolution);
        data.push(lighting);
        data.push(contrast);

        Ok(data)
    }

    /// 转换为 JT808 录像控制指令 (0x8802)
    pub fn to_0x8802(
        channel_id: u8,
        control_flag: u8,
        time_interval: u16,
        save_flag: u8,
    ) -> Result<Vec<u8>, String> {
        // JT808 0x8802 录像控制指令格式
        // [通道ID(1)][控制标志(1)][时间间隔(2)][保存标志(1)]
        let mut data = Vec::with_capacity(5);

        data.push(channel_id);
        data.push(control_flag);
        data.extend_from_slice(&time_interval.to_be_bytes());
        data.push(save_flag);

        Ok(data)
    }

    /// 解析 WebSocket 指令并转换为 JT808 协议帧
    pub fn parse_websocket_command(
        command: &str,
        params: &serde_json::Value,
    ) -> Result<(u16, Vec<u8>), String> {
        let (msg_id, data) = match command {
            "text_query" => {
                // 文本下发 (0x8300 command_word=0x01)
                if let Some(text) = params.get("text").and_then(|v| v.as_str()) {
                    let mut data = vec![0x01]; // 命令字
                    data.extend_from_slice(text.as_bytes());
                    (0x8300, data)
                } else {
                    return Err("Missing 'text' parameter for text_query".to_string());
                }
            }
            "take_photo" => {
                // 立即拍照 (0x8801)
                let channel_id = params
                    .get("channel_id")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u8;
                let save_flag = params
                    .get("save_flag")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;
                let time_interval = params
                    .get("time_interval")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u16;
                let quality = params.get("quality").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
                let resolution = params
                    .get("resolution")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u8;
                let lighting = params
                    .get("lighting")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(128) as u8;
                let contrast = params
                    .get("contrast")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(64) as u8;

                (
                    0x8801,
                    Self::to_0x8801(
                        channel_id,
                        save_flag,
                        time_interval,
                        resolution,
                        quality,
                        lighting,
                        contrast,
                    )?,
                )
            }
            "video_control" => {
                // 录像控制 (0x8802)
                let channel_id = params
                    .get("channel_id")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as u8;
                let control_flag = params
                    .get("control_flag")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;
                let time_interval = params
                    .get("time_interval")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u16;
                let save_flag = params
                    .get("save_flag")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8;

                (
                    0x8802,
                    Self::to_0x8802(channel_id, control_flag, time_interval, save_flag)?,
                )
            }
            "terminal_reset" => {
                // 终端复位 (0x8300 command_word=0x01)
                (0x8300, vec![0x01])
            }
            "terminal_shutdown" => {
                // 终端关机 (0x8300 command_word=0x02)
                (0x8300, vec![0x02])
            }
            "query_params" => {
                // 查询终端参数 (0x8101)
                let mut params_map = HashMap::new();
                if let Some(param_ids) = params.get("param_ids").and_then(|v| v.as_array()) {
                    for param_id in param_ids {
                        if let Some(id) = param_id.as_u64() {
                            params_map.insert(id as u32, serde_json::Value::Null);
                        }
                    }
                }
                (0x8101, Self::to_0x8101(&params_map)?)
            }
            _ => {
                return Err(format!("Unknown command: {}", command));
            }
        };

        Ok((msg_id, data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::message_router::{MessagePriority, MessageType};
    use chrono::Utc;

    #[test]
    fn test_convert_location_report() {
        let phone = "13800138000".to_string();
        let location = LocationReport {
            alarm_flag: 0,
            status: 0,
            latitude: 22.5431,
            longitude: 114.0579,
            altitude: 10.0,
            speed: 60.0,
            direction: 90.0,
            timestamp: Utc::now(),
            sensor_data: SensorData::new(),
        };

        let result = JT808ToUnifiedConverter::convert_location_report(phone.clone(), &location);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.msg_type, MessageType::Data);
        assert_eq!(msg.device_id, Some(phone.clone()));
        assert_eq!(msg.command, Some("location_report".to_string()));
    }

    #[test]
    fn test_convert_alarm() {
        let result = JT808ToUnifiedConverter::convert_alarm(
            "13800138000".to_string(),
            "overspeed".to_string(),
            2,
            "超速报警".to_string(),
        );
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert_eq!(msg.msg_type, MessageType::Notification);
        assert_eq!(msg.priority, MessagePriority::High);
    }

    #[test]
    fn test_parse_websocket_command() {
        let params = serde_json::json!({
            "channel_id": 1,
            "save_flag": 1,
            "time_interval": 5,
            "quality": 2,
            "resolution": 1,
            "lighting": 128,
            "contrast": 64,
        });

        let result = UnifiedToJT808Converter::parse_websocket_command("take_photo", &params);
        assert!(result.is_ok());

        let (msg_id, data) = result.unwrap();
        assert_eq!(msg_id, 0x8801);
        assert_eq!(data.len(), 7);
    }
}
