//! / 报警解析器
// 解析 JT808 协议中的报警数据

use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 报警类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlarmType {
    /// 紧急报警
    Emergency,
    /// 超速报警
    Overspeed,
    /// 疲劳驾驶
    FatigueDriving,
    /// 危险预警
    DangerWarning,
    /// 非法移动
    IllegalMovement,
    /// 断电报警
    PowerCut,
    /// 盗警
    Theft,
    /// 超区域报警
    OutOfArea,
    /// 进区域报警
    InOfArea,
    /// 电子围栏报警
    Geofence,
    /// 未设防报警
    NotArmed,
    /// 设备异常
    DeviceAbnormal,
    /// 其他报警
    Other(String),
}

impl AlarmType {
    /// 从报警标志解析报警类型
    pub fn from_alarm_flag(flag: u32) -> Vec<AlarmType> {
        let mut alarms = Vec::new();

        // 根据 JT808 标准解析报警标志
        if flag & 0x01 != 0 {
            alarms.push(AlarmType::Emergency);
        }
        if flag & 0x02 != 0 {
            alarms.push(AlarmType::Overspeed);
        }
        if flag & 0x04 != 0 {
            alarms.push(AlarmType::FatigueDriving);
        }
        if flag & 0x08 != 0 {
            alarms.push(AlarmType::DangerWarning);
        }
        if flag & 0x10 != 0 {
            alarms.push(AlarmType::IllegalMovement);
        }
        if flag & 0x20 != 0 {
            alarms.push(AlarmType::PowerCut);
        }
        if flag & 0x40 != 0 {
            alarms.push(AlarmType::Theft);
        }

        alarms
    }

    /// 获取报警描述
    pub fn description(&self) -> String {
        match self {
            AlarmType::Emergency => "紧急报警".to_string(),
            AlarmType::Overspeed => "超速报警".to_string(),
            AlarmType::FatigueDriving => "疲劳驾驶".to_string(),
            AlarmType::DangerWarning => "危险预警".to_string(),
            AlarmType::IllegalMovement => "非法移动".to_string(),
            AlarmType::PowerCut => "断电报警".to_string(),
            AlarmType::Theft => "盗警".to_string(),
            AlarmType::OutOfArea => "超区域报警".to_string(),
            AlarmType::InOfArea => "进区域报警".to_string(),
            AlarmType::Geofence => "电子围栏报警".to_string(),
            AlarmType::NotArmed => "未设防报警".to_string(),
            AlarmType::DeviceAbnormal => "设备异常".to_string(),
            AlarmType::Other(desc) => format!("其他报警: {}", desc),
        }
    }

    /// 获取报警级别
    pub fn level(&self) -> super::AlarmLevel {
        match self {
            AlarmType::Emergency => super::AlarmLevel::Critical,
            AlarmType::Theft | AlarmType::PowerCut => super::AlarmLevel::High,
            AlarmType::Overspeed | AlarmType::FatigueDriving | AlarmType::DangerWarning => super::AlarmLevel::Warning,
            _ => super::AlarmLevel::Tip,
        }
    }
}

/// 报警解析器
pub struct AlarmParser {
    alarm_descriptions: HashMap<u32, String>,
}

impl AlarmParser {
    pub fn new() -> Self {
        info!("Creating alarm parser");

        let mut alarm_descriptions = HashMap::new();

        // JT808 报警类型映射
        alarm_descriptions.insert(0x01, "紧急报警:按键紧急触发".to_string());
        alarm_descriptions.insert(0x02, "超速报警:车辆超速行驶".to_string());
        alarm_descriptions.insert(0x04, "疲劳驾驶:司机疲劳".to_string());
        alarm_descriptions.insert(0x08, "危险预警:危险路段".to_string());
        alarm_descriptions.insert(0x10, "非法移动:未授权移动".to_string());
        alarm_descriptions.insert(0x20, "断电报警:车辆断电".to_string());
        alarm_descriptions.insert(0x40, "盗警:车辆被盗".to_string());

        Self { alarm_descriptions }
    }

    /// 解析报警消息
    pub fn parse(&self, alarm_flag: u32, alarm_id: Option<u32>) -> Vec<AlarmType> {
        AlarmType::from_alarm_flag(alarm_flag)
    }

    /// 获取报警描述
    pub fn get_description(&self, alarm_flag: u32) -> String {
        if let Some(desc) = self.alarm_descriptions.get(&alarm_flag) {
            desc.clone()
        } else {
            format!("未知报警: 0x{:08X}", alarm_flag)
        }
    }
}

impl Default for AlarmParser {
    fn default() -> Self {
        Self::new()
    }
}






