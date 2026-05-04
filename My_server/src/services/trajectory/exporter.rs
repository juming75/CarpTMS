//! / 轨迹导出器
// 支持导出为 CSV、Excel、KML、JSON 等格式

use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::infrastructure::cache::TrajectoryPoint;

/// 导出格式
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    /// CSV 格式
    Csv,
    /// Excel 格式
    Excel,
    /// KML 格式(Google Earth)
    Kml,
    /// JSON 格式
    Json,
}

/// 轨迹导出器
pub struct TrajectoryExporter;

impl TrajectoryExporter {
    pub fn new() -> Self {
        info!("Creating trajectory exporter");
        Self
    }

    /// 导出为 CSV
    pub fn export_csv(&self, points: &[TrajectoryPoint]) -> Result<Vec<u8>, String> {
        debug!("Exporting {} points to CSV", points.len());

        let mut csv = String::new();
        csv.push_str("设备ID,纬度,经度,海拔(m),速度(km/h),方向,时间,地址,是否停车,停车时长(秒)\n");

        for point in points {
            csv.push_str(&format!(
                "{},{:.6},{:.6},{},{},{},{},{},{},{}\n",
                point.device_id,
                point.latitude,
                point.longitude,
                point.altitude.map_or("".to_string(), |a| a.to_string()),
                point.speed.map_or("".to_string(), |s| s.to_string()),
                point.direction.map_or("".to_string(), |d| d.to_string()),
                point.timestamp.format("%Y-%m-%d %H:%M:%S"),
                point.address.as_deref().unwrap_or(""),
                point.is_parking,
                point.parking_duration.map_or("".to_string(), |d| d.to_string())
            ));
        }

        Ok(csv.into_bytes())
    }

    /// 导出为 Excel(简化实现,返回 CSV 格式)
    pub fn export_excel(&self, points: &[TrajectoryPoint]) -> Result<Vec<u8>, String> {
        debug!("Exporting {} points to Excel", points.len());
        
        // 简化实现:返回 CSV 格式,实际应使用 xlsx 库
        self.export_csv(points)
    }

    /// 导出为 KML
    pub fn export_kml(&self, points: &[TrajectoryPoint]) -> Result<Vec<u8>, String> {
        debug!("Exporting {} points to KML", points.len());

        if points.is_empty() {
            return Ok(r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <name>轨迹</name>
  </Document>
</kml>"#.as_bytes().to_vec());
        }

        let device_id = &points[0].device_id;

        // 生成坐标字符串
        let coordinates: Vec<String> = points
            .iter()
            .map(|p| format!("{},{},{}", p.longitude, p.latitude, p.altitude.map_or(0.0, |a| a as f64)))
            .collect();

        let kml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <name>轨迹 - {}</name>
    <Placemark>
      <name>行驶轨迹</name>
      <LineString>
        <coordinates>
{}
        </coordinates>
      </LineString>
      <Style>
        <LineStyle>
          <color>ff0000ff</color>
          <width>3</width>
        </LineStyle>
      </Style>
    </Placemark>
  </Document>
</kml>"#, device_id, coordinates.join("\n"));

        Ok(kml.into_bytes())
    }

    /// 导出为 JSON
    pub fn export_json(&self, points: &[TrajectoryPoint]) -> Result<Vec<u8>, String> {
        debug!("Exporting {} points to JSON", points.len());

        let device_id = points.first().map(|p| &p.device_id).unwrap_or(&"unknown".to_string());

        let json = serde_json::json!({
            "device_id": device_id,
            "point_count": points.len(),
            "points": points
        });

        serde_json::to_vec_pretty(&json)
            .map_err(|e| format!("JSON serialization error: {}", e))
    }
}

impl Default for TrajectoryExporter {
    fn default() -> Self {
        Self::new()
    }
}






