//! 数据同步验证模块
//!
//! 提供数据不同步问题的处理方案：
//! - GPS模块状态检查
//! - 坐标转换算法验证（GCJ-02/WGS-84）
//! - 时间同步问题排查
//! - 消息序列号连续性检查

use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// GPS位置数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpsPosition {
    /// 经度（WGS-84）
    pub longitude: f64,
    /// 纬度（WGS-84）
    pub latitude: f64,
    /// 海拔（米）
    pub altitude: Option<f64>,
    /// 速度（km/h）
    pub speed: Option<f64>,
    /// 方向（度）
    pub heading: Option<f64>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 坐标系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinateSystem {
    /// WGS-84（GPS原始坐标）
    Wgs84,
    /// GCJ-02（国测局坐标）
    Gcj02,
    /// BD-09（百度坐标）
    Bd09,
}

/// 同步状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    /// 同步正常
    InSync,
    /// 轻微偏差
    MinorDeviation,
    /// 严重不同步
    MajorDesync,
    /// 数据异常
    DataAnomaly,
}

/// 同步验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncVerificationResult {
    /// 设备ID
    pub device_id: String,
    /// 同步状态
    pub sync_status: SyncStatus,
    /// 位置偏差（米）
    pub position_deviation_meters: Option<f64>,
    /// 时间偏差（毫秒）
    pub time_deviation_ms: Option<u64>,
    /// 序列号是否连续
    pub sequence_continuous: bool,
    /// 缺失的序列号
    pub missing_sequences: Vec<u32>,
    /// 验证时间
    pub verification_time: DateTime<Utc>,
    /// 详细描述
    pub details: String,
}

/// 数据同步管理器
pub struct DataSyncManager {
    /// GPS位置缓存
    position_cache: Arc<RwLock<HashMap<String, Vec<GpsPosition>>>>,
    /// 序列号跟踪
    sequence_tracker: Arc<RwLock<HashMap<String, Vec<u32>>>>,
    /// 坐标转换配置
    coordinate_config: Arc<RwLock<CoordinateConfig>>,
    /// 验证历史
    verification_history: Arc<RwLock<HashMap<String, Vec<SyncVerificationResult>>>>,
}

/// 坐标转换配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateConfig {
    /// 目标坐标系
    pub target_system: CoordinateSystem,
    /// 是否自动纠正偏差
    pub auto_correct: bool,
    /// 允许的最大位置偏差（米）
    pub max_position_deviation_meters: f64,
    /// 允许的最大时间偏差（毫秒）
    pub max_time_deviation_ms: u64,
}

impl Default for CoordinateConfig {
    fn default() -> Self {
        Self {
            target_system: CoordinateSystem::Gcj02,
            auto_correct: true,
            max_position_deviation_meters: 50.0,
            max_time_deviation_ms: 1000,
        }
    }
}

impl DataSyncManager {
    /// 创建新的数据同步管理器
    pub fn new() -> Self {
        Self {
            position_cache: Arc::new(RwLock::new(HashMap::new())),
            sequence_tracker: Arc::new(RwLock::new(HashMap::new())),
            coordinate_config: Arc::new(RwLock::new(CoordinateConfig::default())),
            verification_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 验证设备数据同步状态
    pub async fn verify_sync_status(
        &self,
        device_id: &str,
        current_position: &GpsPosition,
        current_sequence: u32,
    ) -> SyncVerificationResult {
        debug!("Verifying sync status for device {}", device_id);

        // 1. 检查GPS模块状态
        let gps_status = self.check_gps_status(device_id, current_position).await;

        // 2. 验证坐标转换
        let coordinate_result = self.verify_coordinate_conversion(device_id, current_position).await;

        // 3. 检查时间同步
        let time_result = self.check_time_sync(device_id, current_position).await;

        // 4. 检查序列号连续性
        let sequence_result = self.check_sequence_continuity(device_id, current_sequence).await;

        // 综合评估同步状态
        let sync_status = self.evaluate_sync_status(
            &gps_status,
            &coordinate_result,
            &time_result,
            &sequence_result,
        );

        // 生成详细描述
        let details = self.generate_sync_details(
            &gps_status,
            &coordinate_result,
            &time_result,
            &sequence_result,
        );

        let result = SyncVerificationResult {
            device_id: device_id.to_string(),
            sync_status,
            position_deviation_meters: coordinate_result.deviation,
            time_deviation_ms: time_result.deviation_ms,
            sequence_continuous: sequence_result.is_continuous,
            missing_sequences: sequence_result.missing_sequences.clone(),
            verification_time: Utc::now(),
            details,
        };

        // 保存验证历史
        self.save_verification_result(device_id, &result).await;

        result
    }

    /// 检查GPS模块状态
    async fn check_gps_status(&self, device_id: &str, position: &GpsPosition) -> GpsStatusCheck {
        debug!("Checking GPS status for device {}", device_id);

        // 检查坐标是否在合理范围内
        let is_valid_longitude = (-180.0..=180.0).contains(&position.longitude);
        let is_valid_latitude = (-90.0..=90.0).contains(&position.latitude);

        let is_valid = is_valid_longitude && is_valid_latitude;

        GpsStatusCheck {
            is_valid,
            details: if is_valid {
                "GPS坐标在有效范围内".to_string()
            } else {
                format!(
                    "GPS坐标超出范围: lon={}, lat={}",
                    position.longitude, position.latitude
                )
            },
        }
    }

    /// 验证坐标转换
    async fn verify_coordinate_conversion(
        &self,
        device_id: &str,
        position: &GpsPosition,
    ) -> CoordinateVerificationResult {
        debug!("Verifying coordinate conversion for device {}", device_id);

        let config = self.coordinate_config.read().await;

        // WGS-84 转 GCJ-02
        let (gcj_lon, gcj_lat) = wgs84_to_gcj02(position.longitude, position.latitude);

        // 计算偏差（如果之前有缓存的位置）
        let deviation = self.calculate_position_deviation(device_id, gcj_lon, gcj_lat).await;

        CoordinateVerificationResult {
            original_system: CoordinateSystem::Wgs84,
            converted_system: config.target_system.clone(),
            converted_longitude: gcj_lon,
            converted_latitude: gcj_lat,
            deviation: Some(deviation),
            is_within_tolerance: deviation < config.max_position_deviation_meters,
        }
    }

    /// 检查时间同步
    async fn check_time_sync(
        &self,
        device_id: &str,
        position: &GpsPosition,
    ) -> TimeSyncCheckResult {
        debug!("Checking time sync for device {}", device_id);

        let now = Utc::now();
        let device_time = position.timestamp;
        let time_diff = (now - device_time).num_milliseconds().abs() as u64;

        let config = self.coordinate_config.read().await;

        TimeSyncCheckResult {
            deviation_ms: Some(time_diff),
            is_synchronized: time_diff < config.max_time_deviation_ms,
            server_time: now,
            device_time,
        }
    }

    /// 检查序列号连续性
    async fn check_sequence_continuity(
        &self,
        device_id: &str,
        current_sequence: u32,
    ) -> SequenceCheckResult {
        debug!("Checking sequence continuity for device {}", device_id);

        let mut tracker = self.sequence_tracker.write().await;
        let sequences = tracker
            .entry(device_id.to_string())
            .or_insert_with(Vec::new);

        let missing_sequences = if sequences.is_empty() {
            Vec::new()
        } else {
            let last_sequence = *sequences.last().unwrap();
            let mut missing = Vec::new();

            // 检查是否有序列号缺失
            if current_sequence > last_sequence + 1 {
                for seq in (last_sequence + 1)..current_sequence {
                    missing.push(seq);
                }
            }
            missing
        };

        // 添加当前序列号
        sequences.push(current_sequence);

        // 只保留最近1000个序列号
        if sequences.len() > 1000 {
            sequences.drain(..sequences.len() - 1000);
        }

        SequenceCheckResult {
            is_continuous: missing_sequences.is_empty(),
            last_sequence: current_sequence,
            missing_sequences,
        }
    }

    /// 计算位置偏差
    async fn calculate_position_deviation(
        &self,
        device_id: &str,
        new_longitude: f64,
        new_latitude: f64,
    ) -> f64 {
        let position_cache = self.position_cache.read().await;

        if let Some(positions) = position_cache.get(device_id) {
            if let Some(last_position) = positions.last() {
                // 使用Haversine公式计算两点间距离
                return haversine_distance(
                    last_position.latitude,
                    last_position.longitude,
                    new_latitude,
                    new_longitude,
                );
            }
        }

        0.0 // 没有历史数据，偏差为0
    }

    /// 评估同步状态
    fn evaluate_sync_status(
        &self,
        gps_status: &GpsStatusCheck,
        coordinate_result: &CoordinateVerificationResult,
        time_result: &TimeSyncCheckResult,
        sequence_result: &SequenceCheckResult,
    ) -> SyncStatus {
        if !gps_status.is_valid {
            return SyncStatus::DataAnomaly;
        }

        if !sequence_result.is_continuous && sequence_result.missing_sequences.len() > 10 {
            return SyncStatus::MajorDesync;
        }

        if let Some(deviation) = coordinate_result.deviation {
            if deviation > 100.0 {
                return SyncStatus::MajorDesync;
            } else if deviation > 50.0 {
                return SyncStatus::MinorDeviation;
            }
        }

        if let Some(time_dev) = time_result.deviation_ms {
            if time_dev > 5000 {
                return SyncStatus::MajorDesync;
            } else if time_dev > 2000 {
                return SyncStatus::MinorDeviation;
            }
        }

        SyncStatus::InSync
    }

    /// 生成同步详情
    fn generate_sync_details(
        &self,
        gps_status: &GpsStatusCheck,
        coordinate_result: &CoordinateVerificationResult,
        time_result: &TimeSyncCheckResult,
        sequence_result: &SequenceCheckResult,
    ) -> String {
        let mut details = Vec::new();

        details.push(format!("GPS状态: {}", gps_status.details));

        if let Some(deviation) = coordinate_result.deviation {
            details.push(format!("位置偏差: {:.2}米", deviation));
        }

        if let Some(time_dev) = time_result.deviation_ms {
            details.push(format!("时间偏差: {}毫秒", time_dev));
        }

        if !sequence_result.is_continuous {
            details.push(format!(
                "缺失序列号: {}个",
                sequence_result.missing_sequences.len()
            ));
        }

        details.join("; ")
    }

    /// 保存验证结果
    async fn save_verification_result(
        &self,
        device_id: &str,
        result: &SyncVerificationResult,
    ) {
        let mut history = self.verification_history.write().await;
        let results = history
            .entry(device_id.to_string())
            .or_insert_with(Vec::new);
        results.push(result.clone());

        // 只保留最近100条记录
        if results.len() > 100 {
            results.drain(..results.len() - 100);
        }

        info!("Saved sync verification result for device {}", device_id);
    }

    /// 获取验证历史
    pub async fn get_verification_history(
        &self,
        device_id: &str,
    ) -> Vec<SyncVerificationResult> {
        let history = self.verification_history.read().await;
        history.get(device_id).cloned().unwrap_or_default()
    }

    /// 转换坐标系
    pub async fn convert_coordinates(
        &self,
        longitude: f64,
        latitude: f64,
        from: CoordinateSystem,
        to: CoordinateSystem,
    ) -> (f64, f64) {
        match (from, to) {
            (CoordinateSystem::Wgs84, CoordinateSystem::Gcj02) => {
                wgs84_to_gcj02(longitude, latitude)
            }
            (CoordinateSystem::Gcj02, CoordinateSystem::Wgs84) => {
                gcj02_to_wgs84(longitude, latitude)
            }
            _ => (longitude, latitude), // 其他转换暂不支持
        }
    }
}

/// GPS状态检查
#[derive(Debug, Clone)]
struct GpsStatusCheck {
    is_valid: bool,
    details: String,
}

/// 坐标验证结果
#[derive(Debug, Clone)]
struct CoordinateVerificationResult {
    original_system: CoordinateSystem,
    converted_system: CoordinateSystem,
    converted_longitude: f64,
    converted_latitude: f64,
    deviation: Option<f64>,
    is_within_tolerance: bool,
}

/// 时间同步检查
#[derive(Debug, Clone)]
struct TimeSyncCheckResult {
    deviation_ms: Option<u64>,
    is_synchronized: bool,
    server_time: DateTime<Utc>,
    device_time: DateTime<Utc>,
}

/// 序列号检查
#[derive(Debug, Clone)]
struct SequenceCheckResult {
    is_continuous: bool,
    last_sequence: u32,
    missing_sequences: Vec<u32>,
}

// 坐标转换算法实现
const PI: f64 = 3.1415926535897932384626;
const A: f64 = 6378245.0; // 长半轴
const EE: f64 = 0.00669342162296594323; // 偏心率平方

/// WGS-84 转 GCJ-02
pub fn wgs84_to_gcj02(lng: f64, lat: f64) -> (f64, f64) {
    if is_out_of_china(lng, lat) {
        return (lng, lat);
    }

    let (mut dlat, mut dlng) = transform_lat(lng - 105.0, lat - 35.0);
    dlat *= 2.0;
    dlng *= 2.0;

    let radlat = lat / 180.0 * PI;
    let mut magic = f64::sin(radlat);
    magic = 1.0 - EE * magic * magic;
    let sqrtmagic = f64::sqrt(magic);

    dlat = (dlat * 180.0) / ((A * (1.0 - EE)) / (magic * sqrtmagic) * PI);
    dlng = (dlng * 180.0) / (A / sqrtmagic * f64::cos(radlat) * PI);

    let mglat = lat + dlat;
    let mglng = lng + dlng;

    (mglng, mglat)
}

/// GCJ-02 转 WGS-84
pub fn gcj02_to_wgs84(lng: f64, lat: f64) -> (f64, f64) {
    if is_out_of_china(lng, lat) {
        return (lng, lat);
    }

    let (dlat, dlng) = transform_lat(lng - 105.0, lat - 35.0);
    let radlat = lat / 180.0 * PI;
    let mut magic = f64::sin(radlat);
    magic = 1.0 - EE * magic * magic;
    let sqrtmagic = f64::sqrt(magic);

    let dlat_calc =
        (dlat * 2.0 * 180.0) / ((A * (1.0 - EE)) / (magic * sqrtmagic) * PI);
    let dlng_calc =
        (dlng * 2.0 * 180.0) / (A / sqrtmagic * f64::cos(radlat) * PI);

    let mglat = lat + dlat_calc;
    let mglng = lng + dlng_calc;

    (lng * 2.0 - mglng, lat * 2.0 - mglat)
}

fn transform_lat(x: f64, y: f64) -> (f64, f64) {
    let mut ret =
        -100.0 + 2.0 * x + 3.0 * y + 0.2 * y * y + 0.1 * x * y + 0.2 * f64::sqrt(f64::abs(x));
    ret += (20.0 * f64::sin(6.0 * x * PI) + 20.0 * f64::sin(2.0 * x * PI)) * 2.0 / 3.0;
    ret += (20.0 * f64::sin(y * PI) + 40.0 * f64::sin(y / 3.0 * PI)) * 2.0 / 3.0;
    ret += (160.0 * f64::sin(y / 12.0 * PI) + 320.0 * f64::sin(y * PI / 30.0)) * 2.0 / 3.0;

    let mut ret2 =
        300.0 + x + 2.0 * y + 0.1 * x * x + 0.1 * x * y + 0.1 * f64::sqrt(f64::abs(x));
    ret2 += (20.0 * f64::sin(6.0 * x * PI) + 20.0 * f64::sin(2.0 * x * PI)) * 2.0 / 3.0;
    ret2 += (20.0 * f64::sin(x * PI) + 40.0 * f64::sin(x / 3.0 * PI)) * 2.0 / 3.0;
    ret2 +=
        (150.0 * f64::sin(x / 12.0 * PI) + 300.0 * f64::sin(x / 30.0 * PI)) * 2.0 / 3.0;

    (ret, ret2)
}

fn is_out_of_china(lng: f64, lat: f64) -> bool {
    lng < 72.004 || lng > 137.8347 || lat < 0.8293 || lat > 55.8271
}

/// Haversine公式计算两点间距离（米）
pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let lon1_rad = lon1.to_radians();
    let lon2_rad = lon2.to_radians();

    let dlat = lat2_rad - lat1_rad;
    let dlon = lon2_rad - lon1_rad;

    let a = f64::sin(dlat / 2.0).powi(2)
        + f64::cos(lat1_rad) * f64::cos(lat2_rad) * f64::sin(dlon / 2.0).powi(2);
    let c = 2.0 * f64::atan2(f64::sqrt(a), f64::sqrt(1.0 - a));

    6371000.0 * c // 地球平均半径（米）
}

/// 创建数据同步管理器实例
pub fn create_data_sync_manager() -> DataSyncManager {
    DataSyncManager::new()
}
