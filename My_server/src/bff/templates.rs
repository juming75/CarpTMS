//! / BFF报表模板引擎
// 使用Tera模板引擎支持自定义报表模板

use crate::bff::models::*;
use anyhow::Result;
use std::path::Path;
use tera::{Context, Tera};

/// 报表模板引擎
pub struct ReportTemplateEngine {
    tera: Tera,
}

impl ReportTemplateEngine {
    /// 创建新的报表模板引擎
    pub fn new(template_dir: &str) -> Result<Self> {
        let template_path = Path::new(template_dir);

        // 如果模板目录不存在,使用内置模板
        let tera = if template_path.exists() {
            Tera::new(&format!("{}/**/*.html", template_dir))?
        } else {
            // 使用内置模板(字符串形式)
            Self::load_builtin_templates()
        };

        Ok(Self { tera })
    }

    /// 加载内置报表模板
    fn load_builtin_templates() -> Tera {
        let mut tera = Tera::default();

        // 车辆运营报表HTML模板
        let vehicle_operation_template = r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>车辆运营报表</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; text-align: center; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
        th { background-color: #4CAF50; color: white; }
        tr:nth-child(even) { background-color: #f2f2f2; }
        .summary { background-color: #f8f9fa; padding: 15px; margin-bottom: 20px; border-radius: 5px; }
        .summary-item { display: inline-block; margin: 10px 20px; }
        .summary-label { font-weight: bold; color: #555; }
        .summary-value { color: #2196F3; font-size: 1.1em; }
    </style>
</head>
<body>
    <h1>车辆运营报表</h1>
    
    <div class="summary">
        <h2>汇总统计</h2>
        <div class="summary-item">
            <span class="summary-label">报表时间:</span>
            <span class="summary-value">{{ start_time }} - {{ end_time }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">车辆总数:</span>
            <span class="summary-value">{{ summary.total_vehicles }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">总里程(km):</span>
            <span class="summary-value">{{ summary.total_mileage | round(precision=2) }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">平均速度(km/h):</span>
            <span class="summary-value">{{ summary.average_speed | round(precision=2) }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">最高速度(km/h):</span>
            <span class="summary-value">{{ summary.max_speed | round(precision=2) }}</span>
        </div>
    </div>

    <table>
        <thead>
            <tr>
                <th>车辆ID</th>
                <th>车牌号</th>
                <th>司机姓名</th>
                <th>总里程(km)</th>
                <th>平均速度(km/h)</th>
                <th>最高速度(km/h)</th>
                <th>在线时长(小时)</th>
                <th>轨迹点数</th>
            </tr>
        </thead>
        <tbody>
        {% for vehicle in vehicles %}
            <tr>
                <td>{{ vehicle.vehicle_id }}</td>
                <td>{{ vehicle.license_plate }}</td>
                <td>{{ vehicle.driver_name | default(value="-") }}</td>
                <td>{{ vehicle.total_mileage | round(precision=2) }}</td>
                <td>{{ vehicle.average_speed | round(precision=2) }}</td>
                <td>{{ vehicle.max_speed | round(precision=2) }}</td>
                <td>{{ vehicle.online_duration | round(precision=2) }}</td>
                <td>{{ vehicle.track_point_count }}</td>
            </tr>
        {% endfor %}
        </tbody>
    </table>
</body>
</html>
"#;

        // 称重统计报表HTML模板
        let weighing_statistics_template = r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>称重统计报表</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; text-align: center; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
        th { background-color: #FF9800; color: white; }
        tr:nth-child(even) { background-color: #f2f2f2; }
        .summary { background-color: #fff3e0; padding: 15px; margin-bottom: 20px; border-radius: 5px; }
        .summary-item { display: inline-block; margin: 10px 20px; }
        .summary-label { font-weight: bold; color: #555; }
        .summary-value { color: #FF9800; font-size: 1.1em; }
    </style>
</head>
<body>
    <h1>称重统计报表</h1>
    
    <div class="summary">
        <h2>汇总统计</h2>
        <div class="summary-item">
            <span class="summary-label">报表时间:</span>
            <span class="summary-value">{{ start_time }} - {{ end_time }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">称重次数:</span>
            <span class="summary-value">{{ summary.total_weighings }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">总毛重(kg):</span>
            <span class="summary-value">{{ summary.total_gross_weight | round(precision=2) }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">总净重(kg):</span>
            <span class="summary-value">{{ summary.total_net_weight | round(precision=2) }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">平均净重(kg):</span>
            <span class="summary-value">{{ summary.average_net_weight | round(precision=2) }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">最大净重(kg):</span>
            <span class="summary-value">{{ summary.max_net_weight | round(precision=2) }}</span>
        </div>
    </div>

    <table>
        <thead>
            <tr>
                <th>称重ID</th>
                <th>车牌号</th>
                <th>毛重(kg)</th>
                <th>皮重(kg)</th>
                <th>净重(kg)</th>
                <th>物料类型</th>
                <th>称重地点</th>
                <th>称重时间</th>
            </tr>
        </thead>
        <tbody>
        {% for weighing in weighings %}
            <tr>
                <td>{{ weighing.weighing_id }}</td>
                <td>{{ weighing.license_plate }}</td>
                <td>{{ weighing.gross_weight | round(precision=2) }}</td>
                <td>{{ weighing.tare_weight | round(precision=2) }}</td>
                <td>{{ weighing.net_weight | round(precision=2) }}</td>
                <td>{{ weighing.material_type | default(value="-") }}</td>
                <td>{{ weighing.location | default(value="-") }}</td>
                <td>{{ weighing.weighing_time }}</td>
            </tr>
        {% endfor %}
        </tbody>
    </table>
</body>
</html>
"#;

        // 报警分析报表HTML模板
        let alarm_analysis_template = r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>报警分析报表</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; text-align: center; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
        th { background-color: #f44336; color: white; }
        tr:nth-child(even) { background-color: #f2f2f2; }
        .summary { background-color: #ffebee; padding: 15px; margin-bottom: 20px; border-radius: 5px; }
        .summary-item { display: inline-block; margin: 10px 20px; }
        .summary-label { font-weight: bold; color: #555; }
        .summary-value { color: #f44336; font-size: 1.1em; }
        .handled { color: green; font-weight: bold; }
        .unhandled { color: red; font-weight: bold; }
    </style>
</head>
<body>
    <h1>报警分析报表</h1>
    
    <div class="summary">
        <h2>汇总统计</h2>
        <div class="summary-item">
            <span class="summary-label">报表时间:</span>
            <span class="summary-value">{{ start_time }} - {{ end_time }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">总报警数:</span>
            <span class="summary-value">{{ summary.total_alarms }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">已处理:</span>
            <span class="summary-value">{{ summary.handled_alarms }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">未处理:</span>
            <span class="summary-value">{{ summary.unhandled_alarms }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">处理率:</span>
            <span class="summary-value">{{ summary.handling_rate | round(precision=2) }}%</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">紧急报警:</span>
            <span class="summary-value">{{ summary.urgent_alarms }}</span>
        </div>
        <div class="summary-item">
            <span class="summary-label">重要报警:</span>
            <span class="summary-value">{{ summary.high_alarms }}</span>
        </div>
    </div>

    <h2>报警明细</h2>
    <table>
        <thead>
            <tr>
                <th>报警ID</th>
                <th>车牌号</th>
                <th>报警类型</th>
                <th>报警等级</th>
                <th>报警信息</th>
                <th>报警时间</th>
                <th>位置</th>
                <th>状态</th>
                <th>处理人</th>
            </tr>
        </thead>
        <tbody>
        {% for alarm in alarms %}
            <tr>
                <td>{{ alarm.alarm_id }}</td>
                <td>{{ alarm.license_plate }}</td>
                <td>{{ alarm.alarm_type }}</td>
                <td>{{ alarm.alarm_level }}</td>
                <td>{{ alarm.alarm_message }}</td>
                <td>{{ alarm.alarm_time }}</td>
                <td>{{ alarm.location | default(value="-") }}</td>
                <td class="{% if alarm.is_handled %}handled{% else %}unhandled{% endif %}">
                    {% if alarm.is_handled %}已处理{% else %}未处理{% endif %}
                </td>
                <td>{{ alarm.handler | default(value="-") }}</td>
            </tr>
        {% endfor %}
        </tbody>
    </table>

    <h2>按类型统计</h2>
    <table>
        <thead>
            <tr>
                <th>报警类型</th>
                <th>数量</th>
                <th>占比(%)</th>
            </tr>
        </thead>
        <tbody>
        {% for stat in by_type %}
            <tr>
                <td>{{ stat.alarm_type }}</td>
                <td>{{ stat.count }}</td>
                <td>{{ stat.percentage | round(precision=2) }}</td>
            </tr>
        {% endfor %}
        </tbody>
    </table>

    <h2>按车辆统计</h2>
    <table>
        <thead>
            <tr>
                <th>车牌号</th>
                <th>报警总数</th>
                <th>紧急报警</th>
                <th>重要报警</th>
            </tr>
        </thead>
        <tbody>
        {% for stat in by_vehicle %}
            <tr>
                <td>{{ stat.license_plate }}</td>
                <td>{{ stat.alarm_count }}</td>
                <td>{{ stat.urgent_count }}</td>
                <td>{{ stat.high_count }}</td>
            </tr>
        {% endfor %}
        </tbody>
    </table>
</body>
</html>
"#;

        // 注册内置模板
        tera.add_raw_template("vehicle_operation.html", vehicle_operation_template)
            .expect("Failed to add vehicle operation template");
        tera.add_raw_template("weighing_statistics.html", weighing_statistics_template)
            .expect("Failed to add weighing statistics template");
        tera.add_raw_template("alarm_analysis.html", alarm_analysis_template)
            .expect("Failed to add alarm analysis template");

        tera
    }

    /// 渲染车辆运营报表为HTML
    pub fn render_vehicle_operation_html(&self, report: &VehicleOperationReport) -> Result<String> {
        let mut context = Context::new();
        context.insert("generated_at", &report.generated_at.to_rfc3339());
        context.insert(
            "start_time",
            &report.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        );
        context.insert(
            "end_time",
            &report.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        );
        context.insert("vehicles", &report.vehicles);
        context.insert("summary", &report.summary);

        let html = self.tera.render("vehicle_operation.html", &context)?;
        Ok(html)
    }

    /// 渲染称重统计报表为HTML
    pub fn render_weighing_statistics_html(
        &self,
        report: &WeighingStatisticsReport,
    ) -> Result<String> {
        let mut context = Context::new();
        context.insert("generated_at", &report.generated_at.to_rfc3339());
        context.insert(
            "start_time",
            &report.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        );
        context.insert(
            "end_time",
            &report.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        );
        context.insert("weighings", &report.weighings);
        context.insert("summary", &report.summary);

        let html = self.tera.render("weighing_statistics.html", &context)?;
        Ok(html)
    }

    /// 渲染报警分析报表为HTML
    pub fn render_alarm_analysis_html(&self, report: &AlarmAnalysisReport) -> Result<String> {
        let mut context = Context::new();
        context.insert("generated_at", &report.generated_at.to_rfc3339());
        context.insert(
            "start_time",
            &report.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        );
        context.insert(
            "end_time",
            &report.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        );
        context.insert("alarms", &report.alarms);
        context.insert("by_type", &report.by_type);
        context.insert("by_vehicle", &report.by_vehicle);
        context.insert("summary", &report.summary);

        let html = self.tera.render("alarm_analysis.html", &context)?;
        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_template_engine_creation() {
        let engine = ReportTemplateEngine::new("").expect("Failed to create template engine");
        assert!(engine.tera.get_template_names().count() > 0);
    }

    #[test]
    fn test_render_vehicle_operation_report() {
        let engine = ReportTemplateEngine::new("").expect("Failed to create template engine");

        let report = VehicleOperationReport {
            generated_at: Utc::now(),
            start_time: Utc::now() - chrono::Duration::days(7),
            end_time: Utc::now(),
            vehicles: vec![],
            summary: OperationSummary {
                total_vehicles: 0,
                total_mileage: 0.0,
                total_duration_hours: 0.0,
                total_fuel_consumption: None,
                average_speed: 0.0,
                max_speed: 0.0,
                total_online_hours: 0.0,
            },
        };

        let html = engine
            .render_vehicle_operation_html(&report)
            .expect("Failed to render report");
        assert!(html.contains("车辆运营报表"));
        assert!(html.contains("汇总统计"));
    }
}
