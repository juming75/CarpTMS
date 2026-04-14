//! / BFF报表导出服务 - 简化版本
// 修复编译错误

use crate::bff::models::*;
use anyhow::Result;
use chrono::{DateTime, Utc};

/// 报表导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Excel,
    Csv,
    Json,
    Html,
}

impl ExportFormat {
    pub fn parse_format(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "excel" | "xlsx" => Some(ExportFormat::Excel),
            "csv" => Some(ExportFormat::Csv),
            "json" => Some(ExportFormat::Json),
            "html" => Some(ExportFormat::Html),
            _ => None,
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            ExportFormat::Excel => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            ExportFormat::Csv => "text/csv; charset=utf-8",
            ExportFormat::Json => "application/json",
            ExportFormat::Html => "text/html; charset=utf-8",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Excel => ".xlsx",
            ExportFormat::Csv => ".csv",
            ExportFormat::Json => ".json",
            ExportFormat::Html => ".html",
        }
    }
}

/// 报表导出服务
pub struct ReportExportService;

impl ReportExportService {
    /// 导出车辆运营报表
    pub fn export_vehicle_operation_report(
        report: &VehicleOperationReport,
        format: ExportFormat,
    ) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Excel => Self::export_vehicle_operation_excel(report),
            ExportFormat::Csv => Self::export_vehicle_operation_csv(report),
            ExportFormat::Json => Self::export_vehicle_operation_json(report),
            ExportFormat::Html => Err(anyhow::anyhow!(
                "HTML export should use template engine, not export service"
            )),
        }
    }

    /// 导出称重统计报表
    pub fn export_weighing_statistics_report(
        report: &WeighingStatisticsReport,
        format: ExportFormat,
    ) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Excel => Self::export_weighing_statistics_excel(report),
            ExportFormat::Csv => Self::export_weighing_statistics_csv(report),
            ExportFormat::Json => Self::export_weighing_statistics_json(report),
            ExportFormat::Html => Err(anyhow::anyhow!(
                "HTML export should use template engine, not export service"
            )),
        }
    }

    /// 导出报警分析报表
    pub fn export_alarm_analysis_report(
        report: &AlarmAnalysisReport,
        format: ExportFormat,
    ) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Excel => Self::export_alarm_analysis_excel(report),
            ExportFormat::Csv => Self::export_alarm_analysis_csv(report),
            ExportFormat::Json => Self::export_alarm_analysis_json(report),
            ExportFormat::Html => Err(anyhow::anyhow!(
                "HTML export should use template engine, not export service"
            )),
        }
    }

    /// 导出GPS轨迹报表
    pub fn export_gps_track_report(
        track_points: &[GpsTrackPoint],
        _vehicle_id: i32,
        _license_plate: &str,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
        format: ExportFormat,
    ) -> Result<Vec<u8>> {
        match format {
            ExportFormat::Excel => Self::export_gps_track_excel(
                track_points,
                _vehicle_id,
                _license_plate,
                _start_time,
                _end_time,
            ),
            ExportFormat::Csv => Self::export_gps_track_csv(
                track_points,
                _vehicle_id,
                _license_plate,
                _start_time,
                _end_time,
            ),
            ExportFormat::Json => Self::export_gps_track_json(
                track_points,
                _vehicle_id,
                _license_plate,
                _start_time,
                _end_time,
            ),
            ExportFormat::Html => Err(anyhow::anyhow!(
                "HTML export should use template engine, not export service"
            )),
        }
    }

    // ========== CSV导出方法 (简化) ==========

    fn export_vehicle_operation_csv(report: &VehicleOperationReport) -> Result<Vec<u8>> {
        let mut csv = String::new();

        // 标题
        csv.push_str("车辆运营报表\n");
        csv.push_str(&format!(
            "报表时间: {} - {}\n\n",
            report.start_time.format("%Y-%m-%d %H:%M:%S"),
            report.end_time.format("%Y-%m-%d %H:%M:%S")
        ));

        // 汇总
        csv.push_str("汇总统计\n");
        csv.push_str(&format!("车辆总数,{}\n", report.summary.total_vehicles));
        csv.push_str(&format!("总里程,{:.2} km\n", report.summary.total_mileage));
        csv.push_str(&format!(
            "平均速度,{:.2} km/h\n\n",
            report.summary.average_speed
        ));

        // 表头
        csv.push_str("车辆ID,车牌号,司机姓名,总里程,平均速度,最高速度,在线时长(小时),轨迹点数\n");

        // 数据
        for vehicle in &report.vehicles {
            csv.push_str(&format!(
                "{},{},{},{:.2},{:.2},{:.2},{},{}\n",
                vehicle.vehicle_id,
                vehicle.license_plate,
                vehicle.driver_name.as_deref().unwrap_or("-"),
                vehicle.total_mileage,
                vehicle.average_speed,
                vehicle.max_speed,
                vehicle.online_duration,
                vehicle.track_point_count
            ));
        }

        Ok(csv.into_bytes())
    }

    fn export_vehicle_operation_json(report: &VehicleOperationReport) -> Result<Vec<u8>> {
        let json = serde_json::to_string_pretty(report)?;
        Ok(json.into_bytes())
    }

    fn export_weighing_statistics_csv(report: &WeighingStatisticsReport) -> Result<Vec<u8>> {
        let mut csv = String::new();

        csv.push_str("称重统计报表\n");
        csv.push_str(&format!(
            "报表时间: {} - {}\n\n",
            report.start_time.format("%Y-%m-%d %H:%M:%S"),
            report.end_time.format("%Y-%m-%d %H:%M:%S")
        ));

        csv.push_str("车辆ID,车牌号,毛重,皮重,净重,称重时间\n");

        for weighing in &report.weighings {
            csv.push_str(&format!(
                "{},{},{:.2},{:.2},{:.2},{}\n",
                weighing.vehicle_id,
                weighing.license_plate,
                weighing.gross_weight,
                weighing.tare_weight,
                weighing.net_weight,
                weighing.weighing_time.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        Ok(csv.into_bytes())
    }

    fn export_weighing_statistics_json(report: &WeighingStatisticsReport) -> Result<Vec<u8>> {
        let json = serde_json::to_string_pretty(report)?;
        Ok(json.into_bytes())
    }

    fn export_alarm_analysis_csv(report: &AlarmAnalysisReport) -> Result<Vec<u8>> {
        let mut csv = String::new();

        csv.push_str("报警分析报表\n");
        csv.push_str(&format!(
            "报表时间: {} - {}\n\n",
            report.start_time.format("%Y-%m-%d %H:%M:%S"),
            report.end_time.format("%Y-%m-%d %H:%M:%S")
        ));

        csv.push_str("汇总统计\n");
        csv.push_str(&format!("总报警次数,{}\n", report.summary.total_alarms));
        csv.push_str(&format!("已处理,{}\n", report.summary.handled_alarms));
        csv.push_str(&format!("未处理,{}\n\n", report.summary.unhandled_alarms));

        csv.push_str("报警ID,车辆ID,车牌号,报警类型,报警时间,是否已处理\n");

        for alarm in &report.alarms {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                alarm.alarm_id,
                alarm.vehicle_id,
                alarm.license_plate,
                alarm.alarm_type,
                alarm.alarm_time.format("%Y-%m-%d %H:%M:%S"),
                if alarm.is_handled { "是" } else { "否" }
            ));
        }

        Ok(csv.into_bytes())
    }

    fn export_alarm_analysis_json(report: &AlarmAnalysisReport) -> Result<Vec<u8>> {
        let json = serde_json::to_string_pretty(report)?;
        Ok(json.into_bytes())
    }

    fn export_alarm_analysis_excel(_report: &AlarmAnalysisReport) -> Result<Vec<u8>> {
        Err(anyhow::anyhow!(
            "Excel export not yet implemented, please use CSV or JSON format"
        ))
    }

    fn export_gps_track_csv(
        track_points: &[GpsTrackPoint],
        _vehicle_id: i32,
        _license_plate: &str,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<u8>> {
        let mut csv = String::new();

        csv.push_str("GPS轨迹报表\n");
        csv.push_str("时间,经度,纬度,速度,方向,高度\n");

        for point in track_points {
            csv.push_str(&format!(
                "{},{:.7},{:.7},{:.2},{},{:.2}\n",
                point.gps_time.format("%Y-%m-%d %H:%M:%S"),
                point.longitude,
                point.latitude,
                point.speed,
                point.direction as i32,
                point.altitude.unwrap_or(0.0)
            ));
        }

        Ok(csv.into_bytes())
    }

    fn export_gps_track_json(
        track_points: &[GpsTrackPoint],
        _vehicle_id: i32,
        _license_plate: &str,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<u8>> {
        let json = serde_json::to_string_pretty(track_points)?;
        Ok(json.into_bytes())
    }

    // ========== Excel导出方法 (占位) ==========

    fn export_vehicle_operation_excel(_report: &VehicleOperationReport) -> Result<Vec<u8>> {
        // TODO: 实现 Excel 导出
        // 目前返回 CSV 格式作为临时解决方案
        Err(anyhow::anyhow!(
            "Excel export not yet implemented, please use CSV or JSON format"
        ))
    }

    fn export_weighing_statistics_excel(_report: &WeighingStatisticsReport) -> Result<Vec<u8>> {
        // TODO: 实现 Excel 导出
        Err(anyhow::anyhow!(
            "Excel export not yet implemented, please use CSV or JSON format"
        ))
    }

    fn export_gps_track_excel(
        _track_points: &[GpsTrackPoint],
        _vehicle_id: i32,
        _license_plate: &str,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<u8>> {
        // TODO: 实现 Excel 导出
        Err(anyhow::anyhow!(
            "Excel export not yet implemented, please use CSV or JSON format"
        ))
    }
}
