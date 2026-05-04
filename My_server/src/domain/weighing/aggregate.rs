//! 称重数据聚合根
//!
//! 聚合根是领域驱动设计中的核心模式，确保业务规则的一致性

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::events::{AbnormalType, WeighingEvent};

/// 聚合错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum AggregateError {
    #[error("毛重必须大于0: {0}")]
    InvalidGrossWeight(f64),
    #[error("净重必须大于0: {0}")]
    InvalidNetWeight(f64),
    #[error("皮重不能大于毛重")]
    TareWeightExceedsGross,
    #[error("无效的车辆ID: {0}")]
    InvalidVehicleId(i32),
}

/// 称重聚合根
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeighingAggregate {
    pub id: Option<i64>,
    pub vehicle_id: i32,
    pub device_id: String,
    pub weighing_time: NaiveDateTime,
    pub gross_weight: f64,
    pub tare_weight: Option<f64>,
    pub net_weight: f64,
    pub axle_count: Option<i32>,
    pub speed: Option<f64>,
    pub lane_no: Option<i32>,
    pub site_id: Option<i32>,
    pub status: i32, // 0=待称重, 1=已称重, 2=已结算, 3=异常
    pub version: i64,
    #[serde(skip)]
    pending_events: Vec<WeighingEvent>,
}

#[allow(clippy::too_many_arguments)]
impl WeighingAggregate {
    /// 创建新的称重记录
    pub fn create(
        vehicle_id: i32,
        device_id: String,
        weighing_time: NaiveDateTime,
        gross_weight: f64,
        net_weight: f64,
        tare_weight: Option<f64>,
        axle_count: Option<i32>,
        speed: Option<f64>,
        lane_no: Option<i32>,
        site_id: Option<i32>,
    ) -> Result<Self, AggregateError> {
        // 业务规则校验
        Self::validate(vehicle_id, gross_weight, net_weight, tare_weight)?;

        let mut aggregate = Self {
            id: None,
            vehicle_id,
            device_id,
            weighing_time,
            gross_weight,
            tare_weight,
            net_weight,
            axle_count,
            speed,
            lane_no,
            site_id,
            status: 0,
            version: 1,
            pending_events: Vec::new(),
        };

        aggregate.add_event(WeighingEvent::Created {
            vehicle_id,
            device_id: aggregate.device_id.clone(),
            gross_weight,
            net_weight,
            weighing_time,
        });

        Ok(aggregate)
    }

    /// 从实体加载
    pub fn from_entity(entity: &super::super::entities::weighing_data::WeighingData) -> Self {
        Self {
            id: Some(entity.id),
            vehicle_id: entity.vehicle_id,
            device_id: entity.device_id.clone(),
            weighing_time: entity.weighing_time,
            gross_weight: entity.gross_weight,
            tare_weight: entity.tare_weight,
            net_weight: entity.net_weight,
            axle_count: entity.axle_count,
            speed: entity.speed,
            lane_no: entity.lane_no,
            site_id: entity.site_id,
            status: entity.status,
            version: 1,
            pending_events: Vec::new(),
        }
    }

    /// 业务规则校验
    fn validate(
        vehicle_id: i32,
        gross_weight: f64,
        net_weight: f64,
        tare_weight: Option<f64>,
    ) -> Result<(), AggregateError> {
        if vehicle_id <= 0 {
            return Err(AggregateError::InvalidVehicleId(vehicle_id));
        }
        if gross_weight <= 0.0 {
            return Err(AggregateError::InvalidGrossWeight(gross_weight));
        }
        if net_weight <= 0.0 {
            return Err(AggregateError::InvalidNetWeight(net_weight));
        }
        if let Some(tare) = tare_weight {
            if tare > gross_weight {
                return Err(AggregateError::TareWeightExceedsGross);
            }
        }
        Ok(())
    }

    /// 添加待发布事件
    fn add_event(&mut self, event: WeighingEvent) {
        self.pending_events.push(event);
    }

    /// 获取并清除待发布事件
    pub fn take_events(&mut self) -> Vec<WeighingEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// 状态变更
    pub fn change_status(&mut self, new_status: i32) {
        let old_status = self.status;
        self.status = new_status;
        if let Some(id) = self.id {
            self.add_event(WeighingEvent::StatusChanged {
                weighing_id: id,
                old_status,
                new_status,
            });
        }
    }

    /// 检测超载
    pub fn check_overload(&self, max_weight: f64) -> Option<WeighingEvent> {
        if self.gross_weight > max_weight {
            self.id.map(|id| WeighingEvent::OverloadWarning {
                weighing_id: id,
                vehicle_id: self.vehicle_id,
                gross_weight: self.gross_weight,
                max_weight,
            })
        } else {
            None
        }
    }

    /// 检测超速
    pub fn check_overspeed(&self, max_speed: f64) -> Option<WeighingEvent> {
        if let Some(speed) = self.speed {
            if speed > max_speed {
                return self.id.map(|id| WeighingEvent::AbnormalDetected {
                    weighing_id: id,
                    vehicle_id: self.vehicle_id,
                    abnormal_type: AbnormalType::Overspeed,
                    description: format!("速度{}超过限制{}", speed, max_speed),
                });
            }
        }
        None
    }
}
