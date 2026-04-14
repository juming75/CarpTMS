//! / 称重数据领域实体

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// 称重数据实体
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WeighingData {
    /// 主键ID
    #[validate(range(min = 1))]
    pub id: i32,
    /// 车辆ID
    #[validate(range(min = 1))]
    pub vehicle_id: i32,
    /// 设备ID
    #[validate(length(min = 1, max = 50))]
    pub device_id: String,
    /// 车辆名称（通过 JOIN vehicles 表获取，非持久化字段）
    #[serde(skip_deserializing)]
    pub vehicle_name: String,
    /// 称重时间
    pub weighing_time: NaiveDateTime,
    /// 毛重
    #[validate(range(min = 0.0))]
    pub gross_weight: f64,
    /// 皮重
    #[validate(range(min = 0.0))]
    pub tare_weight: Option<f64>,
    /// 净重
    #[validate(range(min = 0.0))]
    pub net_weight: f64,
    /// 轴数
    #[validate(range(min = 1, max = 10))]
    pub axle_count: Option<i32>,
    /// 速度
    #[validate(range(min = 0.0, max = 200.0))]
    pub speed: Option<f64>,
    /// 车道号
    #[validate(range(min = 1, max = 10))]
    pub lane_no: Option<i32>,
    /// 站点ID
    #[validate(range(min = 1))]
    pub site_id: Option<i32>,
    /// 状态
    #[validate(range(min = 0, max = 5))]
    pub status: i32,
    /// 创建时间
    pub create_time: NaiveDateTime,
    /// 更新时间
    pub update_time: Option<NaiveDateTime>,
}

/// 称重数据创建实体
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WeighingDataCreate {
    /// 车辆ID
    #[validate(range(min = 1))]
    pub vehicle_id: i32,
    /// 设备ID
    #[validate(length(min = 1, max = 50))]
    pub device_id: String,
    /// 称重时间
    pub weighing_time: NaiveDateTime,
    /// 毛重
    #[validate(range(min = 0.0))]
    pub gross_weight: f64,
    /// 皮重
    #[validate(range(min = 0.0))]
    pub tare_weight: Option<f64>,
    /// 净重
    #[validate(range(min = 0.0))]
    pub net_weight: f64,
    /// 轴数
    #[validate(range(min = 1, max = 10))]
    pub axle_count: Option<i32>,
    /// 速度
    #[validate(range(min = 0.0, max = 200.0))]
    pub speed: Option<f64>,
    /// 车道号
    #[validate(range(min = 1, max = 10))]
    pub lane_no: Option<i32>,
    /// 站点ID
    #[validate(range(min = 1))]
    pub site_id: Option<i32>,
    /// 状态
    #[validate(range(min = 0, max = 5))]
    pub status: i32,
}

/// 称重数据更新实体
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WeighingDataUpdate {
    /// 车辆ID
    #[validate(range(min = 1))]
    pub vehicle_id: Option<i32>,
    /// 设备ID
    #[validate(length(min = 1, max = 50))]
    pub device_id: Option<String>,
    /// 称重时间
    pub weighing_time: Option<NaiveDateTime>,
    /// 毛重
    #[validate(range(min = 0.0))]
    pub gross_weight: Option<f64>,
    /// 皮重
    #[validate(range(min = 0.0))]
    pub tare_weight: Option<f64>,
    /// 净重
    #[validate(range(min = 0.0))]
    pub net_weight: Option<f64>,
    /// 轴数
    #[validate(range(min = 1, max = 10))]
    pub axle_count: Option<i32>,
    /// 速度
    #[validate(range(min = 0.0, max = 200.0))]
    pub speed: Option<f64>,
    /// 车道号
    #[validate(range(min = 1, max = 10))]
    pub lane_no: Option<i32>,
    /// 站点ID
    #[validate(range(min = 1))]
    pub site_id: Option<i32>,
    /// 状态
    #[validate(range(min = 0, max = 5))]
    pub status: Option<i32>,
}

/// 称重数据查询条件实体
#[derive(Debug, Clone, Serialize, Deserialize, Default, Validate)]
pub struct WeighingDataQuery {
    /// 页码
    #[validate(range(min = 1))]
    pub page: Option<i32>,
    /// 每页大小
    #[validate(range(min = 1, max = 100))]
    pub page_size: Option<i32>,
    /// 车辆ID
    #[validate(range(min = 1))]
    pub vehicle_id: Option<i32>,
    /// 设备ID
    #[validate(length(min = 1, max = 50))]
    pub device_id: Option<String>,
    /// 开始时间
    pub start_time: Option<NaiveDateTime>,
    /// 结束时间
    pub end_time: Option<NaiveDateTime>,
    /// 状态
    #[validate(range(min = 0, max = 5))]
    pub status: Option<i32>,
    /// 最小净重
    #[validate(range(min = 0.0))]
    pub min_net_weight: Option<f64>,
    /// 最大净重
    #[validate(range(min = 0.0))]
    pub max_net_weight: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_weighing_data_create_to_weighing_data() {
        // 创建WeighingDataCreate实例
        let weighing_create = WeighingDataCreate {
            vehicle_id: 1,
            device_id: "DEV001".to_string(),
            weighing_time: NaiveDate::from_ymd_opt(2023, 1, 1)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
            gross_weight: 10.5,
            tare_weight: Some(3.5),
            net_weight: 7.0,
            axle_count: Some(3),
            speed: Some(40.5),
            lane_no: Some(1),
            site_id: Some(1),
            status: 1,
        };

        // 手动转换为WeighingData实例
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let weighing_data = WeighingData {
            id: 1,
            vehicle_id: weighing_create.vehicle_id,
            device_id: weighing_create.device_id,
            weighing_time: weighing_create.weighing_time,
            gross_weight: weighing_create.gross_weight,
            tare_weight: weighing_create.tare_weight,
            net_weight: weighing_create.net_weight,
            axle_count: weighing_create.axle_count,
            speed: weighing_create.speed,
            lane_no: weighing_create.lane_no,
            site_id: weighing_create.site_id,
            status: weighing_create.status,
            create_time: now,
            update_time: None,
        };

        // 验证转换结果
        assert_eq!(weighing_data.vehicle_id, 1);
        assert_eq!(weighing_data.device_id, "DEV001");
        assert_eq!(weighing_data.gross_weight, 10.5);
        assert_eq!(weighing_data.tare_weight, Some(3.5));
        assert_eq!(weighing_data.net_weight, 7.0);
        assert_eq!(weighing_data.status, 1);
    }

    #[test]
    fn test_weighing_data_update() {
        // 创建初始WeighingData实例
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let mut weighing_data = WeighingData {
            id: 1,
            vehicle_id: 1,
            device_id: "DEV001".to_string(),
            weighing_time: now,
            gross_weight: 10.5,
            tare_weight: Some(3.5),
            net_weight: 7.0,
            axle_count: Some(3),
            speed: Some(40.5),
            lane_no: Some(1),
            site_id: Some(1),
            status: 1,
            create_time: now,
            update_time: None,
        };

        // 创建WeighingDataUpdate实例
        let weighing_update = WeighingDataUpdate {
            vehicle_id: Some(2),
            device_id: Some("DEV002".to_string()),
            weighing_time: None,
            gross_weight: Some(15.5),
            tare_weight: Some(4.5),
            net_weight: Some(11.0),
            axle_count: Some(4),
            speed: Some(50.5),
            lane_no: Some(2),
            site_id: Some(2),
            status: Some(2),
        };

        // 手动应用更新
        if let Some(vehicle_id) = weighing_update.vehicle_id {
            weighing_data.vehicle_id = vehicle_id;
        }
        if let Some(device_id) = weighing_update.device_id {
            weighing_data.device_id = device_id;
        }
        if let Some(gross_weight) = weighing_update.gross_weight {
            weighing_data.gross_weight = gross_weight;
        }
        if let Some(tare_weight) = weighing_update.tare_weight {
            weighing_data.tare_weight = Some(tare_weight);
        }
        if let Some(net_weight) = weighing_update.net_weight {
            weighing_data.net_weight = net_weight;
        }
        if let Some(axle_count) = weighing_update.axle_count {
            weighing_data.axle_count = Some(axle_count);
        }
        if let Some(speed) = weighing_update.speed {
            weighing_data.speed = Some(speed);
        }
        if let Some(lane_no) = weighing_update.lane_no {
            weighing_data.lane_no = Some(lane_no);
        }
        if let Some(site_id) = weighing_update.site_id {
            weighing_data.site_id = Some(site_id);
        }
        if let Some(status) = weighing_update.status {
            weighing_data.status = status;
        }
        // 更新更新时间
        let update_time = NaiveDate::from_ymd_opt(2023, 1, 2)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        weighing_data.update_time = Some(update_time);

        // 验证更新结果
        assert_eq!(weighing_data.vehicle_id, 2);
        assert_eq!(weighing_data.device_id, "DEV002");
        assert_eq!(weighing_data.gross_weight, 15.5);
        assert_eq!(weighing_data.tare_weight, Some(4.5));
        assert_eq!(weighing_data.net_weight, 11.0);
        assert_eq!(weighing_data.axle_count, Some(4));
        assert_eq!(weighing_data.speed, Some(50.5));
        assert_eq!(weighing_data.lane_no, Some(2));
        assert_eq!(weighing_data.site_id, Some(2));
        assert_eq!(weighing_data.status, 2);
        assert!(weighing_data.update_time.is_some());
    }

    #[test]
    fn test_weighing_data_without_tare_weight() {
        // 创建WeighingDataCreate实例,不包含皮重
        let weighing_create = WeighingDataCreate {
            vehicle_id: 1,
            device_id: "DEV001".to_string(),
            weighing_time: NaiveDate::from_ymd_opt(2023, 1, 1)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
            gross_weight: 10.5,
            tare_weight: None,
            net_weight: 10.5, // 净重等于毛重
            axle_count: None,
            speed: None,
            lane_no: None,
            site_id: None,
            status: 1,
        };

        // 手动转换为WeighingData实例
        let now = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let weighing_data = WeighingData {
            id: 1,
            vehicle_id: weighing_create.vehicle_id,
            device_id: weighing_create.device_id,
            weighing_time: weighing_create.weighing_time,
            gross_weight: weighing_create.gross_weight,
            tare_weight: weighing_create.tare_weight,
            net_weight: weighing_create.net_weight,
            axle_count: weighing_create.axle_count,
            speed: weighing_create.speed,
            lane_no: weighing_create.lane_no,
            site_id: weighing_create.site_id,
            status: weighing_create.status,
            create_time: now,
            update_time: None,
        };

        // 验证转换结果
        assert_eq!(weighing_data.gross_weight, 10.5);
        assert_eq!(weighing_data.tare_weight, None);
        assert_eq!(weighing_data.net_weight, 10.5); // 净重等于毛重
        assert_eq!(weighing_data.axle_count, None);
    }
}
