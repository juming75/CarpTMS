//! 驾驶行为评分模块
//!
//! 实现基于多维度数据的驾驶行为评分系统
//! 评分维度：速度控制、急加速/急减速、急转弯、疲劳驾驶、违规次数等
//! 驾驶行为评分的设计与实现

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, debug};
use chrono::{DateTime, Utc};

/// 评分维度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringDimension {
    /// 维度名称
    pub name: String,
    /// 维度权重（百分比）
    pub weight: f64,
    /// 当前得分（0-100）
    pub score: f64,
    /// 维度描述
    pub description: String,
}

/// 驾驶行为事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrivingEvent {
    /// 事件ID
    pub event_id: String,
    /// 车辆ID
    pub vehicle_id: String,
    /// 司机ID
    pub driver_id: String,
    /// 事件类型
    pub event_type: DrivingEventType,
    /// 事件严重程度（0-100，越高越严重）
    pub severity: u8,
    /// 事件时间
    pub event_time: DateTime<Utc>,
    /// 位置
    pub location: Option<String>,
    /// 速度（km/h）
    pub speed: Option<f64>,
    /// 备注
    pub note: Option<String>,
}

/// 驾驶行为事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrivingEventType {
    /// 正常行驶
    NormalDriving,
    /// 超速
    Overspeed,
    /// 急加速
    RapidAcceleration,
    /// 急减速
    RapidDeceleration,
    /// 急转弯
    SharpTurn,
    /// 疲劳驾驶
    FatigueDriving,
    /// 违规变道
    IllegalLaneChange,
    /// 未系安全带
    SeatbeltNotFastened,
    /// 打电话
    PhoneCall,
    /// 抽烟
    Smoking,
    /// 分心驾驶
    DistractedDriving,
}

impl DrivingEventType {
    /// 获取事件类型名称
    pub fn name(&self) -> &str {
        match self {
            DrivingEventType::NormalDriving => "正常行驶",
            DrivingEventType::Overspeed => "超速",
            DrivingEventType::RapidAcceleration => "急加速",
            DrivingEventType::RapidDeceleration => "急减速",
            DrivingEventType::SharpTurn => "急转弯",
            DrivingEventType::FatigueDriving => "疲劳驾驶",
            DrivingEventType::IllegalLaneChange => "违规变道",
            DrivingEventType::SeatbeltNotFastened => "未系安全带",
            DrivingEventType::PhoneCall => "打电话",
            DrivingEventType::Smoking => "抽烟",
            DrivingEventType::DistractedDriving => "分心驾驶",
        }
    }

    /// 获取默认扣分
    pub fn default_penalty(&self) -> f64 {
        match self {
            DrivingEventType::NormalDriving => 0.0,
            DrivingEventType::Overspeed => 10.0,
            DrivingEventType::RapidAcceleration => 5.0,
            DrivingEventType::RapidDeceleration => 5.0,
            DrivingEventType::SharpTurn => 3.0,
            DrivingEventType::FatigueDriving => 8.0,
            DrivingEventType::IllegalLaneChange => 6.0,
            DrivingEventType::SeatbeltNotFastened => 4.0,
            DrivingEventType::PhoneCall => 7.0,
            DrivingEventType::Smoking => 5.0,
            DrivingEventType::DistractedDriving => 6.0,
        }
    }
}

/// 驾驶评分记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrivingScoreRecord {
    /// 记录ID
    pub record_id: String,
    /// 司机ID
    pub driver_id: String,
    /// 车辆ID
    pub vehicle_id: String,
    /// 总评分（0-100）
    pub total_score: f64,
    /// 评分维度
    pub dimensions: Vec<ScoringDimension>,
    /// 事件数量
    pub event_count: u64,
    /// 严重事件数量
    pub critical_event_count: u64,
    /// 评分时间范围（开始）
    pub score_period_start: DateTime<Utc>,
    /// 评分时间范围（结束）
    pub score_period_end: DateTime<Utc>,
    /// 评分等级
    pub score_grade: ScoreGrade,
    /// 计算时间
    pub calculated_at: DateTime<Utc>,
}

/// 评分等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreGrade {
    /// 优秀（90-100）
    Excellent,
    /// 良好（80-89）
    Good,
    /// 一般（70-79）
    Fair,
    /// 较差（60-69）
    Poor,
    /// 极差（<60）
    VeryPoor,
}

impl ScoreGrade {
    /// 从分数获取等级
    pub fn from_score(score: f64) -> Self {
        if score >= 90.0 {
            ScoreGrade::Excellent
        } else if score >= 80.0 {
            ScoreGrade::Good
        } else if score >= 70.0 {
            ScoreGrade::Fair
        } else if score >= 60.0 {
            ScoreGrade::Poor
        } else {
            ScoreGrade::VeryPoor
        }
    }

    /// 获取等级名称
    pub fn name(&self) -> &str {
        match self {
            ScoreGrade::Excellent => "优秀",
            ScoreGrade::Good => "良好",
            ScoreGrade::Fair => "一般",
            ScoreGrade::Poor => "较差",
            ScoreGrade::VeryPoor => "极差",
        }
    }

    /// 获取等级颜色
    pub fn color(&self) -> &str {
        match self {
            ScoreGrade::Excellent => "#4CAF50",
            ScoreGrade::Good => "#8BC34A",
            ScoreGrade::Fair => "#FFC107",
            ScoreGrade::Poor => "#FF9800",
            ScoreGrade::VeryPoor => "#F44336",
        }
    }
}

/// 驾驶行为评分计算器
/// 计算和管理司机的驾驶行为评分
pub struct DrivingBehaviorScorer {
    /// 驾驶事件记录
    events: Arc<RwLock<Vec<DrivingEvent>>>,
    /// 评分记录
    score_records: Arc<RwLock<HashMap<String, DrivingScoreRecord>>>,
    /// 司机当前评分
    driver_scores: Arc<RwLock<HashMap<String, f64>>>,
    /// 评分配置
    scoring_config: Arc<RwLock<ScoringConfig>>,
}

/// 评分配置
#[derive(Debug, Clone)]
pub struct ScoringConfig {
    /// 评分周期（天）
    pub scoring_period_days: u64,
    /// 基础分
    pub base_score: f64,
    /// 最低分
    pub min_score: f64,
    /// 最高分
    pub max_score: f64,
    /// 评分维度权重
    pub dimension_weights: HashMap<String, f64>,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        let mut dimension_weights = HashMap::new();
        dimension_weights.insert("speed_control".to_string(), 0.25);
        dimension_weights.insert("acceleration_control".to_string(), 0.20);
        dimension_weights.insert("turning_control".to_string(), 0.15);
        dimension_weights.insert("fatigue_prevention".to_string(), 0.15);
        dimension_weights.insert("violation_record".to_string(), 0.25);

        Self {
            scoring_period_days: 30,
            base_score: 100.0,
            min_score: 0.0,
            max_score: 100.0,
            dimension_weights,
        }
    }
}

impl DrivingBehaviorScorer {
    /// 创建新的驾驶行为评分计算器
    pub fn new(config: Option<ScoringConfig>) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            score_records: Arc::new(RwLock::new(HashMap::new())),
            driver_scores: Arc::new(RwLock::new(HashMap::new())),
            scoring_config: Arc::new(RwLock::new(config.unwrap_or_default())),
        }
    }

    /// 记录驾驶事件
    pub async fn record_event(&self, event: DrivingEvent) {
        let mut events = self.events.write().await;
        events.push(event);

        // 限制事件数量，保留最近10000条
        if events.len() > 10000 {
            events.drain(0..events.len() - 10000);
        }

        debug!("Driving event recorded: {} - {:?}", event.driver_id, event.event_type);
    }

    /// 计算司机评分
    pub async fn calculate_driver_score(&self, driver_id: &str) -> DrivingScoreRecord {
        let events = self.events.read().await;
        let config = self.scoring_config.read().await;

        let now = Utc::now();
        let period_start = now - chrono::Duration::days(config.scoring_period_days as i64);

        // 筛选该司机在评分周期内的事件
        let driver_events: Vec<&DrivingEvent> = events.iter()
            .filter(|e| e.driver_id == driver_id && e.event_time >= period_start)
            .collect();

        let mut total_penalty = 0.0;
        let mut critical_count = 0u64;

        for event in &driver_events {
            let penalty = event.event_type.default_penalty() * (event.severity as f64 / 50.0);
            total_penalty += penalty;
            if event.severity >= 70 {
                critical_count += 1;
            }
        }

        // 计算总分
        let total_score = (config.base_score - total_penalty)
            .max(config.min_score)
            .min(config.max_score);

        // 计算各维度得分
        let dimensions = self.calculate_dimension_scores(&driver_events, &config);

        let score_grade = ScoreGrade::from_score(total_score);

        let record = DrivingScoreRecord {
            record_id: format!("SCORE_{}_{}", driver_id, now.timestamp_millis()),
            driver_id: driver_id.to_string(),
            vehicle_id: driver_events.first().map(|e| e.vehicle_id.clone()).unwrap_or_default(),
            total_score,
            dimensions,
            event_count: driver_events.len() as u64,
            critical_event_count: critical_count,
            score_period_start: period_start,
            score_period_end: now,
            score_grade,
            calculated_at: now,
        };

        // 保存评分记录
        let mut score_records = self.score_records.write().await;
        score_records.insert(driver_id.to_string(), record.clone());

        // 更新司机当前评分
        let mut driver_scores = self.driver_scores.write().await;
        driver_scores.insert(driver_id.to_string(), total_score);

        debug!(
            "Driver score calculated: {} = {:.1} ({:?})",
            driver_id, total_score, score_grade
        );

        record
    }

    /// 计算各维度得分
    fn calculate_dimension_scores(
        &self,
        events: &[&DrivingEvent],
        config: &ScoringConfig,
    ) -> Vec<ScoringDimension> {
        let mut dimensions = Vec::new();

        // 速度控制维度
        let speed_events: Vec<&&DrivingEvent> = events.iter()
            .filter(|e| matches!(e.event_type, DrivingEventType::Overspeed))
            .collect();
        let speed_score = (100.0 - speed_events.len() as f64 * 10.0).max(0.0);
        dimensions.push(ScoringDimension {
            name: "速度控制".to_string(),
            weight: config.dimension_weights.get("speed_control").copied().unwrap_or(0.25),
            score: speed_score,
            description: "评估司机的速度控制能力，包括超速等行为".to_string(),
        });

        // 加减速控制维度
        let accel_events: Vec<&&DrivingEvent> = events.iter()
            .filter(|e| matches!(e.event_type, DrivingEventType::RapidAcceleration | DrivingEventType::RapidDeceleration))
            .collect();
        let accel_score = (100.0 - accel_events.len() as f64 * 5.0).max(0.0);
        dimensions.push(ScoringDimension {
            name: "加减速控制".to_string(),
            weight: config.dimension_weights.get("acceleration_control").copied().unwrap_or(0.20),
            score: accel_score,
            description: "评估司机的加减速平稳性".to_string(),
        });

        // 转弯控制维度
        let turn_events: Vec<&&DrivingEvent> = events.iter()
            .filter(|e| matches!(e.event_type, DrivingEventType::SharpTurn))
            .collect();
        let turn_score = (100.0 - turn_events.len() as f64 * 8.0).max(0.0);
        dimensions.push(ScoringDimension {
            name: "转弯控制".to_string(),
            weight: config.dimension_weights.get("turning_control").copied().unwrap_or(0.15),
            score: turn_score,
            description: "评估司机的转弯操作规范性".to_string(),
        });

        // 疲劳预防维度
        let fatigue_events: Vec<&&DrivingEvent> = events.iter()
            .filter(|e| matches!(e.event_type, DrivingEventType::FatigueDriving))
            .collect();
        let fatigue_score = (100.0 - fatigue_events.len() as f64 * 12.0).max(0.0);
        dimensions.push(ScoringDimension {
            name: "疲劳预防".to_string(),
            weight: config.dimension_weights.get("fatigue_prevention").copied().unwrap_or(0.15),
            score: fatigue_score,
            description: "评估司机的疲劳驾驶情况".to_string(),
        });

        // 违规记录维度
        let violation_events: Vec<&&DrivingEvent> = events.iter()
            .filter(|e| matches!(e.event_type, 
                DrivingEventType::IllegalLaneChange | 
                DrivingEventType::SeatbeltNotFastened |
                DrivingEventType::PhoneCall |
                DrivingEventType::Smoking |
                DrivingEventType::DistractedDriving))
            .collect();
        let violation_score = (100.0 - violation_events.len() as f64 * 6.0).max(0.0);
        dimensions.push(ScoringDimension {
            name: "违规记录".to_string(),
            weight: config.dimension_weights.get("violation_record").copied().unwrap_or(0.25),
            score: violation_score,
            description: "评估司机的交通违规情况".to_string(),
        });

        dimensions
    }

    /// 获取司机当前评分
    pub async fn get_driver_score(&self, driver_id: &str) -> Option<f64> {
        let driver_scores = self.driver_scores.read().await;
        driver_scores.get(driver_id).copied()
    }

    /// 获取司机评分记录
    pub async fn get_driver_score_record(&self, driver_id: &str) -> Option<DrivingScoreRecord> {
        let score_records = self.score_records.read().await;
        score_records.get(driver_id).cloned()
    }

    /// 获取司机事件列表
    pub async fn get_driver_events(&self, driver_id: &str, limit: usize) -> Vec<DrivingEvent> {
        let events = self.events.read().await;
        events.iter()
            .filter(|e| e.driver_id == driver_id)
            .take(limit)
            .cloned()
            .collect()
    }

    /// 获取评分排名
    pub async fn get_score_ranking(&self) -> Vec<(String, f64)> {
        let driver_scores = self.driver_scores.read().await;
        let mut ranking: Vec<(String, f64)> = driver_scores.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        
        // 按分数降序排列
        ranking.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranking
    }

    /// 清理过期数据
    pub async fn cleanup_expired_data(&self) {
        let config = self.scoring_config.read().await;
        let cutoff = Utc::now() - chrono::Duration::days((config.scoring_period_days * 2) as i64);
        
        let mut events = self.events.write().await;
        let before_count = events.len();
        events.retain(|e| e.event_time >= cutoff);
        
        let removed = before_count - events.len();
        if removed > 0 {
            debug!("Cleaned up {} expired driving events", removed);
        }
    }
}

/// 创建驾驶行为评分计算器（便捷函数）
pub fn create_driving_behavior_scorer() -> Arc<DrivingBehaviorScorer> {
    Arc::new(DrivingBehaviorScorer::new(None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_grade_from_score() {
        assert_eq!(ScoreGrade::from_score(95.0), ScoreGrade::Excellent);
        assert_eq!(ScoreGrade::from_score(85.0), ScoreGrade::Good);
        assert_eq!(ScoreGrade::from_score(75.0), ScoreGrade::Fair);
        assert_eq!(ScoreGrade::from_score(65.0), ScoreGrade::Poor);
        assert_eq!(ScoreGrade::from_score(55.0), ScoreGrade::VeryPoor);
    }

    #[tokio::test]
    async fn test_scorer_creation() {
        let scorer = DrivingBehaviorScorer::new(None);
        assert_eq!(scorer.driver_scores.read().await.len(), 0);
    }
}
