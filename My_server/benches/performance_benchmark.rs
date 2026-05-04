//! / CarpTMS 性能压测脚本
//! 用于评估系统在高负载下的性能表现

use carptms::bff::export::{ExportFormat, ReportExportService};
use carptms::bff::models::*;
use chrono::{DateTime, TimeZone, Utc};
use rand::Rng;
use std::time::{Duration, Instant};

/// 生成随机 GPS 轨迹点
fn generate_random_gps_track(point_count: usize) -> Vec<GpsTrackPoint> {
    let mut rng = rand::thread_rng();
    let base_lat = 39.9042;
    let base_lon = 116.4074;

    (0..point_count)
        .map(|i| GpsTrackPoint {
            latitude: base_lat + rng.gen::<f64>() * 0.1,
            longitude: base_lon + rng.gen::<f64>() * 0.1,
            altitude: Some(rng.gen::<f64>() * 100.0),
            speed: rng.gen::<f64>() * 120.0,
            direction: rng.gen::<u16>() % 360,
            gps_time: Utc::now() - Duration::from_secs((point_count - i) as u64 * 60),
        })
        .collect()
}

/// 生成随机车辆运营报表
fn generate_random_vehicle_report(vehicle_count: usize) -> VehicleOperationReport {
    let mut rng = rand::thread_rng();

    let vehicles: Vec<VehicleData> = (0..vehicle_count)
        .map(|i| VehicleData {
            vehicle_id: i as i32,
            license_plate: format!("京{}12345", char::from(b'A' + (i % 26) as u8)),
            driver_name: Some(format!("司机{}", i)),
            total_mileage: rng.gen::<f64>() * 10000.0,
            average_speed: rng.gen::<f64>() * 100.0,
            max_speed: rng.gen::<f64>() * 150.0,
            online_duration: rng.gen::<f64>() * 720.0,
            track_point_count: rng.gen::<usize>() % 10000,
            fuel_consumption: Some(rng.gen::<f64>() * 500.0),
        })
        .collect();

    let total_mileage: f64 = vehicles.iter().map(|v| v.total_mileage).sum();
    let avg_speed: f64 =
        vehicles.iter().map(|v| v.average_speed).sum::<f64>() / vehicles.len() as f64;

    VehicleOperationReport {
        start_time: Utc::now() - Duration::from_secs(30 * 24 * 3600),
        end_time: Utc::now(),
        summary: VehicleSummary {
            total_vehicles: vehicle_count as i32,
            total_mileage,
            average_speed: avg_speed,
            max_speed: vehicles.iter().map(|v| v.max_speed).fold(0.0, f64::max),
            total_track_points: vehicles.iter().map(|v| v.track_point_count).sum(),
            online_vehicles: vehicle_count as i32,
        },
        vehicles,
    }
}

/// 生成随机称重报表
fn generate_random_weighing_report(record_count: usize) -> WeighingStatisticsReport {
    let mut rng = rand::thread_rng();

    let weighings: Vec<WeighingData> = (0..record_count)
        .map(|i| WeighingData {
            vehicle_id: i as i32,
            license_plate: format!("京{}12345", char::from(b'A' + (i % 26) as u8)),
            gross_weight: rng.gen::<f64>() * 100.0 + 20.0,
            tare_weight: rng.gen::<f64>() * 20.0 + 10.0,
            net_weight: rng.gen::<f64>() * 80.0 + 10.0,
            weighing_time: Utc::now() - Duration::from_secs(i as u64 * 3600),
            location: Some(format!("{}号地磅", i % 5 + 1)),
            operator: Some(format!("操作员{}", i % 10)),
            notes: None,
        })
        .collect();

    let total_net_weight: f64 = weighings.iter().map(|w| w.net_weight).sum();

    WeighingStatisticsReport {
        start_time: Utc::now() - Duration::from_secs(30 * 24 * 3600),
        end_time: Utc::now(),
        summary: WeighingSummary {
            total_weighings: record_count as i32,
            total_net_weight,
            average_net_weight: total_net_weight / record_count as f64,
        },
        weighings,
    }
}

/// 性能测试：CSV 导出
fn benchmark_csv_export(vehicle_count: usize) -> Duration {
    let report = generate_random_vehicle_report(vehicle_count);

    let start = Instant::now();
    let _ = ReportExportService::export_vehicle_operation_report(&report, ExportFormat::Csv);
    start.elapsed()
}

/// 性能测试：JSON 导出
fn benchmark_json_export(vehicle_count: usize) -> Duration {
    let report = generate_random_vehicle_report(vehicle_count);

    let start = Instant::now();
    let _ = ReportExportService::export_vehicle_operation_report(&report, ExportFormat::Json);
    start.elapsed()
}

/// 性能测试：Excel 导出
fn benchmark_excel_export(vehicle_count: usize) -> Duration {
    let report = generate_random_vehicle_report(vehicle_count);

    let start = Instant::now();
    let _ = ReportExportService::export_vehicle_operation_report(&report, ExportFormat::Excel);
    start.elapsed()
}

/// 性能测试：GPS 轨迹导出
fn benchmark_gps_export(point_count: usize) -> Duration {
    let track_points = generate_random_gps_track(point_count);

    let start = Instant::now();
    let _ = ReportExportService::export_gps_track_report(
        &track_points,
        1,
        "京A12345",
        Utc::now() - Duration::from_secs(24 * 3600),
        Utc::now(),
        ExportFormat::Excel,
    );
    start.elapsed()
}

/// 性能测试：称重报表导出
fn benchmark_weighing_export(record_count: usize) -> Duration {
    let report = generate_random_weighing_report(record_count);

    let start = Instant::now();
    let _ = ReportExportService::export_weighing_statistics_report(&report, ExportFormat::Excel);
    start.elapsed()
}

fn main() {
    println!("===========================================");
    println!("CarpTMS 性能压测报告");
    println!("===========================================\n");

    // 1. 车辆运营报表导出测试
    println!("【1. 车辆运营报表导出测试】");
    println!("-------------------------------------------");

    for count in [100, 500, 1000] {
        let csv_time = benchmark_csv_export(count);
        let json_time = benchmark_json_export(count);
        let excel_time = benchmark_excel_export(count);

        println!("车辆数量: {}", count);
        println!(
            "  CSV 导出:  {:?} ({:.2} ms/记录)",
            csv_time,
            csv_time.as_secs_f64() * 1000.0 / count as f64
        );
        println!(
            "  JSON 导出: {:?} ({:.2} ms/记录)",
            json_time,
            json_time.as_secs_f64() * 1000.0 / count as f64
        );
        println!(
            "  Excel 导出: {:?} ({:.2} ms/记录)",
            excel_time,
            excel_time.as_secs_f64() * 1000.0 / count as f64
        );
        println!();
    }

    // 2. GPS 轨迹导出测试
    println!("【2. GPS 轨迹导出测试】");
    println!("-------------------------------------------");

    for count in [100, 1000, 10000] {
        let time = benchmark_gps_export(count);
        println!("轨迹点数: {}", count);
        println!(
            "  Excel 导出: {:?} ({:.2} ms/点)",
            time,
            time.as_secs_f64() * 1000.0 / count as f64
        );
        println!();
    }

    // 3. 称重报表导出测试
    println!("【3. 称重报表导出测试】");
    println!("-------------------------------------------");

    for count in [100, 1000, 10000] {
        let time = benchmark_weighing_export(count);
        println!("称重记录: {}", count);
        println!(
            "  Excel 导出: {:?} ({:.2} ms/记录)",
            time,
            time.as_secs_f64() * 1000.0 / count as f64
        );
        println!();
    }

    // 4. 大规模测试
    println!("【4. 大规模数据测试】");
    println!("-------------------------------------------");

    let large_vehicle_count = 5000;
    let large_gps_count = 50000;

    let start = Instant::now();
    let report = generate_random_vehicle_report(large_vehicle_count);
    let export_time = benchmark_excel_export(large_vehicle_count);
    println!("大规模车辆报表 ({} 辆):", large_vehicle_count);
    println!(
        "  生成时间: {:?}, 导出时间: {:?}",
        start.elapsed(),
        export_time
    );

    let start = Instant::now();
    let track_points = generate_random_gps_track(large_gps_count);
    let export_time = benchmark_gps_export(large_gps_count);
    println!("大规模 GPS 轨迹 ({} 点):", large_gps_count);
    println!(
        "  生成时间: {:?}, 导出时间: {:?}",
        start.elapsed(),
        export_time
    );

    println!();
    println!("===========================================");
    println!("压测完成");
    println!("===========================================");
}
