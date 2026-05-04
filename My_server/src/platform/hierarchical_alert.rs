//! 分级报警体系模块
//!
//! 实现多级别的车辆报警系统
//! 包括：超速报警、疲劳驾驶、偏离路线、紧急制动等
//! 分级报警体系的设计与实现

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, debug, warn, error};
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

/// 报警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertLevel {
    /// 信息级别（仅记录）
    Info,
    /// 警告级别（提示注意）
    Warning,
    /// 严重级别（需要处理）
    Critical,
    /// 紧急级别（立即处理）
    Emergency,
}

impl AlertLevel {
    /// 获取报警级别名称
    pub fn name(&self) -> &str {
        match self {
            AlertLevel::Info => "信息",
            AlertLevel::Warning => "警告",
            AlertLevel::Critical => "严重",
            AlertLevel::Emergency => "紧急",
        }
    }

    /// 获取报警级别数值
    pub fn level_value(&self) -> u8 {
        match self {
            AlertLevel::Info => 1,
            AlertLevel::Warning => 2,
            AlertLevel::Critical => 3,
            AlertLevel::Emergency => 4,
        }
    }

    /// 获取推送方式
    pub fn push_methods(&self) -> Vec<PushMethod> {
        match self {
            AlertLevel::Info => vec![PushMethod::SystemLog],
            AlertLevel::Warning => vec![PushMethod::SystemLog, PushMethod::InApp],
            AlertLevel::Critical => vec![PushMethod::SystemLog, PushMethod::InApp, PushMethod::Sms],
            AlertLevel::Emergency => vec![PushMethod::SystemLog, PushMethod::InApp, PushMethod::Sms, PushMethod::Phone],
        }
    }
}

/// 推送方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PushMethod {
    /// 系统日志
    SystemLog,
    /// 应用内通知
    InApp,
    /// 短信通知
    Sms,
    /// 电话通知
    Phone,
    /// 邮件通知
    Email,
    /// 企业微信通知
    WeCom,
}

/// 报警类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertType {
    /// 超速报警
    Overspeed,
    /// 疲劳驾驶
    FatigueDriving,
    /// 偏离路线
    RouteDeviation,
    /// 紧急制动
    EmergencyBrake,
    /// 急加速
    RapidAcceleration,
    /// 急转弯
    SharpTurn,
    /// 车辆故障
    VehicleFault,
    /// 电子围栏报警
    Geofence,
    /// 停车超时
    ParkingTimeout,
    /// 温度异常（冷链）
    TemperatureAbnormal,
    /// 油量异常
    FuelAbnormal,
    /// 设备离线
    DeviceOffline,
}

impl AlertType {
    /// 获取报警类型名称
    pub fn name(&self) -> &str {
        match self {
            AlertType::Overspeed => "超速报警",
            AlertType::FatigueDriving => "疲劳驾驶",
            AlertType::RouteDeviation => "偏离路线",
            AlertType::EmergencyBrake => "紧急制动",
            AlertType::RapidAcceleration => "急加速",
            AlertType::SharpTurn => "急转弯",
            AlertType::VehicleFault => "车辆故障",
            AlertType::Geofence => "电子围栏报警",
            AlertType::ParkingTimeout => "停车超时",
            AlertType::TemperatureAbnormal => "温度异常",
            AlertType::FuelAbnormal => "油量异常",
            AlertType::DeviceOffline => "设备离线",
        }
    }
}

/// 报警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// 规则ID
    pub rule_id: String,
    /// 规则名称
    pub rule_name: String,
    /// 报警类型
    pub alert_type: AlertType,
    /// 报警级别
    pub alert_level: AlertLevel,
    /// 触发阈值
    pub threshold: f64,
    /// 持续时间（秒）- 超过此时间才触发
    pub duration_seconds: u64,
    /// 冷却时间（秒）- 触发后多长时间不再重复触发
    pub cooldown_seconds: u64,
    /// 是否启用
    pub enabled: bool,
    /// 适用企业类型（None表示所有企业）
    pub applicable_enterprise_types: Option<Vec<String>>,
}

/// 报警记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRecord {
    /// 报警ID
    pub alert_id: String,
    /// 规则ID
    pub rule_id: String,
    /// 车辆ID
    pub vehicle_id: String,
    /// 司机ID
    pub driver_id: Option<String>,
    /// 企业ID
    pub enterprise_id: String,
    /// 报警类型
    pub alert_type: AlertType,
    /// 报警级别
    pub alert_level: AlertLevel,
    /// 报警值
    pub alert_value: f64,
    /// 阈值
    pub threshold: f64,
    /// 报警时间
    pub alert_time: DateTime<Utc>,
    /// 位置
    pub location: Option<LocationInfo>,
    /// 是否已处理
    pub is_handled: bool,
    /// 处理人
    pub handler_id: Option<String>,
    /// 处理时间
    pub handled_time: Option<DateTime<Utc>>,
    /// 处理备注
    pub handled_note: Option<String>,
}

/// 位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    /// 纬度
    pub latitude: f64,
    /// 经度
    pub longitude: f64,
    /// 地址
    pub address: Option<String>,
    /// 速度（km/h）
    pub speed: f64,
}

/// 报警统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatistics {
    /// 总报警数
    pub total_alerts: u64,
    /// 按级别统计
    pub alerts_by_level: HashMap<AlertLevel, u64>,
    /// 按类型统计
    pub alerts_by_type: HashMap<AlertType, u64>,
    /// 已处理报警数
    pub handled_alerts: u64,
    /// 未处理报警数
    pub unhandled_alerts: u64,
    /// 平均处理时间（秒）
    pub avg_handle_time_seconds: f64,
}

/// 报警跟踪器（用于跟踪持续报警状态）
struct AlertTracker {
    /// 开始时间
    start_time: Instant,
    /// 最后触发时间
    last_triggered: Instant,
    /// 触发次数
    trigger_count: u64,
}

/// 分级报警管理器
/// 管理报警规则、触发报警、处理报警
pub struct HierarchicalAlertManager {
    /// 报警规则
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    /// 报警记录
    records: Arc<RwLock<Vec<AlertRecord>>>,
    /// 报警跟踪器（用于检测持续报警）
    trackers: Arc<RwLock<HashMap<String, AlertTracker>>>,
    /// 统计信息
    stats: Arc<RwLock<AlertStatistics>>,
}

impl HierarchicalAlertManager {
    /// 创建新的分级报警管理器
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            records: Arc::new(RwLock::new(Vec::new())),
            trackers: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(AlertStatistics {
                total_alerts: 0,
                alerts_by_level: HashMap::new(),
                alerts_by_type: HashMap::new(),
                handled_alerts: 0,
                unhandled_alerts: 0,
                avg_handle_time_seconds: 0.0,
            })),
        }
    }

    /// 添加报警规则
    pub async fn add_rule(&self, rule: AlertRule) -> Result<(), String> {
        let mut rules = self.rules.write().await;
        if rules.contains_key(&rule.rule_id) {
            return Err(format!("Rule {} already exists", rule.rule_id));
        }
        rules.insert(rule.rule_id.clone(), rule);
        info!("Alert rule added: {} ({:?})", rule.rule_name, rule.alert_type);
        Ok(())
    }

    /// 检查报警规则
    /// 返回是否需要触发报警
    pub async fn check_alert(
        &self,
        vehicle_id: &str,
        enterprise_id: &str,
        alert_type: AlertType,
        current_value: f64,
    ) -> Option<AlertRecord> {
        let rules = self.rules.read().await;

        // 查找匹配的规则
        let matching_rule = rules.values()
            .find(|r| r.alert_type == alert_type && r.enabled)
            .cloned()?;

        let tracker_key = format!("{}:{}", vehicle_id, alert_type.name());
        let now = Instant::now();

        // 检查跟踪器
        let mut trackers = self.trackers.write().await;
        let tracker = trackers.entry(tracker_key.clone()).or_insert_with(|| AlertTracker {
            start_time: now,
            last_triggered: now,
            trigger_count: 0,
        });

        // 检查是否超过阈值
        if current_value <= matching_rule.threshold {
            return None;
        }

        // 检查是否超过持续时间要求
        let elapsed = now.duration_since(tracker.start_time).as_secs();
        if elapsed < matching_rule.duration_seconds {
            return None;
        }

        // 检查冷却时间
        let time_since_last = now.duration_since(tracker.last_triggered).as_secs();
        if time_since_last < matching_rule.cooldown_seconds {
            return None;
        }

        // 触发报警
        tracker.last_triggered = now;
        tracker.trigger_count += 1;

        let alert_id = format!("ALT_{}_{}", vehicle_id, chrono::Utc::now().timestamp_millis());
        let alert_record = AlertRecord {
            alert_id: alert_id.clone(),
            rule_id: matching_rule.rule_id.clone(),
            vehicle_id: vehicle_id.to_string(),
            driver_id: None, // 从车辆信息中获取
            enterprise_id: enterprise_id.to_string(),
            alert_type,
            alert_level: matching_rule.alert_level,
            alert_value: current_value,
            threshold: matching_rule.threshold,
            alert_time: Utc::now(),
            location: None,
            is_handled: false,
            handler_id: None,
            handled_time: None,
            handled_note: None,
        };

        // 保存报警记录
        let mut records = self.records.write().await;
        records.push(alert_record.clone());

        // 更新统计信息
        let mut stats = self.stats.write().await;
        stats.total_alerts += 1;
        stats.unhandled_alerts += 1;
        *stats.alerts_by_level.entry(matching_rule.alert_level).or_insert(0) += 1;
        *stats.alerts_by_type.entry(alert_type).or_insert(0) += 1;

        // 推送报警通知
        self.push_alert_notification(&alert_record).await;

        info!(
            "Alert triggered: {} - {:?} (value: {}, threshold: {})",
            alert_id, alert_type, current_value, matching_rule.threshold
        );

        Some(alert_record)
    }

    /// 处理报警
    pub async fn handle_alert(
        &self,
        alert_id: &str,
        handler_id: &str,
        note: &str,
    ) -> Result<(), String> {
        let mut records = self.records.write().await;
        if let Some(record) = records.iter_mut().find(|r| r.alert_id == alert_id) {
            record.is_handled = true;
            record.handler_id = Some(handler_id.to_string());
            record.handled_time = Some(Utc::now());
            record.handled_note = Some(note.to_string());

            // 更新统计信息
            let mut stats = self.stats.write().await;
            stats.handled_alerts += 1;
            stats.unhandled_alerts = stats.unhandled_alerts.saturating_sub(1);

            info!("Alert handled: {} by {}", alert_id, handler_id);
            Ok(())
        } else {
            Err(format!("Alert {} not found", alert_id))
        }
    }

    /// 推送报警通知
    async fn push_alert_notification(&self, alert: &AlertRecord) {
        let push_methods = alert.alert_level.push_methods();

        for method in push_methods {
            match method {
                PushMethod::SystemLog => {
                    debug!(
                        "[SystemLog] Alert {}: {:?} - {}",
                        alert.alert_id, alert.alert_type, alert.alert_type.name()
                    );
                }
                PushMethod::InApp => {
                    info!(
                        "[InApp] Pushing alert {} to user",
                        alert.alert_id
                    );
                }
                PushMethod::Sms => {
                    warn!(
                        "[SMS] Sending alert {} via SMS",
                        alert.alert_id
                    );
                }
                PushMethod::Phone => {
                    error!(
                        "[Phone] Calling emergency contact for alert {}",
                        alert.alert_id
                    );
                }
                _ => {}
            }
        }
    }

    /// 获取未处理报警列表
    pub async fn get_unhandled_alerts(&self, limit: usize) -> Vec<AlertRecord> {
        let records = self.records.read().await;
        records.iter()
            .filter(|r| !r.is_handled)
            .take(limit)
            .cloned()
            .collect()
    }

    /// 获取报警统计
    pub async fn get_statistics(&self) -> AlertStatistics {
        self.stats.read().await.clone()
    }

    /// 获取指定车辆的报警记录
    pub async fn get_vehicle_alerts(&self, vehicle_id: &str, limit: usize) -> Vec<AlertRecord> {
        let records = self.records.read().await;
        records.iter()
            .filter(|r| r.vehicle_id == vehicle_id)
            .take(limit)
            .cloned()
            .collect()
    }

    /// 清理过期报警记录（保留最近30天）
    pub async fn cleanup_expired_records(&self) {
        let cutoff = Utc::now() - chrono::Duration::days(30);
        let mut records = self.records.write().await;
        let before_count = records.len();

        records.retain(|r| r.alert_time > cutoff);

        let removed = before_count - records.len();
        if removed > 0 {
            info!("Cleaned up {} expired alert records", removed);
        }
    }
}

impl Default for HierarchicalAlertManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建分级报警管理器（便捷函数）
pub fn create_hierarchical_alert_manager() -> Arc<HierarchicalAlertManager> {
    Arc::new(HierarchicalAlertManager::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_rule_creation() {
        let manager = HierarchicalAlertManager::new();
        let rule = AlertRule {
            rule_id: "overspeed_rule".to_string(),
            rule_name: "超速报警规则".to_string(),
            alert_type: AlertType::Overspeed,
            alert_level: AlertLevel::Warning,
            threshold: 120.0,
            duration_seconds: 10,
            cooldown_seconds: 60,
            enabled: true,
            applicable_enterprise_types: None,
        };

        assert!(manager.add_rule(rule).await.is_ok());
    }
}
