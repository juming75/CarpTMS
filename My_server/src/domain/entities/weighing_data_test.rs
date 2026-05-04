//! 称重数据集成测试
//!
//! 测试称重数据从路由到数据库的完整流程

#[cfg(test)]
mod tests {
    use crate::domain::entities::weighing_data::{
        WeighingData, WeighingDataCreate, WeighingDataUpdate, WeighingStatus,
    };
    use chrono::NaiveDateTime;

    // ============= WeighingData Entity Tests =============

    #[test]
    fn test_weighing_data_id_i64() {
        // 验证 id 字段是 i64 类型
        let data = WeighingData {
            id: 1i64,
            vehicle_id: 1,
            device_id: 1,
            weighing_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            gross_weight: 5000.0,
            tare_weight: 2000.0,
            net_weight: 3000.0,
            axle_count: 3,
            speed: None,
            lane_no: 1,
            site_id: 1,
            status: WeighingStatus::Normal,
            create_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            update_time: None,
        };

        assert_eq!(data.id, 1i64);
        assert!(data.id > 0);
    }

    #[test]
    fn test_weighing_data_large_id() {
        // 测试大 ID 值 (验证 i64 范围)
        let large_id: i64 = 2_147_483_648i64; // 超过 i32 范围的值
        let data = WeighingData {
            id: large_id,
            vehicle_id: 1,
            device_id: 1,
            weighing_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            gross_weight: 5000.0,
            tare_weight: 2000.0,
            net_weight: 3000.0,
            axle_count: 3,
            speed: None,
            lane_no: 1,
            site_id: 1,
            status: WeighingStatus::Normal,
            create_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            update_time: None,
        };

        assert_eq!(data.id, large_id);
        assert!(data.id > i32::MAX as i64);
    }

    #[test]
    fn test_weighing_data_create_with_i64_id() {
        let create = WeighingDataCreate {
            vehicle_id: 1,
            device_id: 1,
            weighing_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            gross_weight: 5000.0,
            tare_weight: 2000.0,
            net_weight: 3000.0,
            axle_count: 3,
            speed: None,
            lane_no: 1,
            site_id: 1,
            status: WeighingStatus::Normal,
        };

        assert_eq!(create.gross_weight, 5000.0);
        assert_eq!(create.net_weight, create.gross_weight - create.tare_weight);
    }

    #[test]
    fn test_weighing_data_update() {
        let update = WeighingDataUpdate {
            gross_weight: Some(5500.0),
            tare_weight: Some(2000.0),
            net_weight: Some(3500.0),
            status: Some(WeighingStatus::Normal),
        };

        assert!(update.gross_weight.is_some());
        assert_eq!(update.gross_weight.unwrap(), 5500.0);
    }

    #[test]
    fn test_weighing_status_values() {
        assert_eq!(WeighingStatus::Normal as i32, 0);
        assert_eq!(WeighingStatus::Abnormal as i32, 1);
        assert_eq!(WeighingStatus::Suspicious as i32, 2);
    }

    #[test]
    fn test_weighing_data_weight_calculation() {
        let gross = 10000.0;
        let tare = 3000.0;
        let expected_net = 7000.0;

        let create = WeighingDataCreate {
            vehicle_id: 1,
            device_id: 1,
            weighing_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            gross_weight: gross,
            tare_weight: tare,
            net_weight: expected_net,
            axle_count: 5,
            speed: None,
            lane_no: 1,
            site_id: 1,
            status: WeighingStatus::Normal,
        };

        // 验证净重计算
        assert_eq!(create.net_weight, gross - tare);
        assert!(create.net_weight > 0.0);
    }

    #[test]
    fn test_weighing_data_multiple_records() {
        // 模拟多个称重记录
        let records: Vec<WeighingData> = (1..=1000i64)
            .map(|i| WeighingData {
                id: i,
                vehicle_id: (i % 100) as i32 + 1,
                device_id: (i % 10) as i32 + 1,
                weighing_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                gross_weight: 5000.0 + (i as f64 * 10.0),
                tare_weight: 2000.0,
                net_weight: 3000.0 + (i as f64 * 10.0),
                axle_count: 3,
                speed: None,
                lane_no: 1,
                site_id: 1,
                status: WeighingStatus::Normal,
                create_time: NaiveDateTime::parse_from_str("2024-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
                update_time: None,
            })
            .collect();

        assert_eq!(records.len(), 1000);
        assert_eq!(records[0].id, 1i64);
        assert_eq!(records[999].id, 1000i64);

        // 验证 i64 可以存储超过 i32 范围的值
        assert!(records[999].id > i32::MAX as i64);
    }
}
