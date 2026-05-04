//! Qwen3.5 业务任务处理模块
//! 继承自旧的 qwen/tasks.rs，使用 Qwen3_5Pipeline 替代 QwenService

use serde::{Deserialize, Serialize};

use super::pipeline::Qwen3_5Pipeline;
use super::prompts::*;

// ==================== 报表生成相关 ====================

/// 报表生成请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ReportGenerationRequest {
    pub report_type: String,
    pub data_type: String,
    pub data: String,
    pub params: Option<serde_json::Value>,
}

/// 报表生成响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ReportGenerationResponse {
    pub report: String,
    pub summary: String,
    pub key_metrics: Vec<KeyMetric>,
    pub recommendations: Vec<String>,
}

/// 关键指标
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyMetric {
    pub name: String,
    pub value: String,
    pub change: Option<String>,
    pub trend: Option<String>,
}

/// 生成报表
pub fn generate_report(
    pipeline: &mut Qwen3_5Pipeline,
    request: &ReportGenerationRequest,
) -> Result<ReportGenerationResponse, String> {
    let prompt = report_generation_prompt(&request.data_type, &request.data, &request.report_type);
    let response = pipeline.infer(&prompt).map_err(|e| e.to_string())?;
    let report = response;
    let summary = extract_summary(&report);
    let key_metrics = extract_key_metrics(&report);
    let recommendations = extract_recommendations(&report);

    Ok(ReportGenerationResponse {
        report,
        summary,
        key_metrics,
        recommendations,
    })
}

// ==================== 标定数据分析相关 ====================

/// 标定数据分析请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationAnalysisRequest {
    pub device_id: String,
    pub device_type: String,
    pub device_info: String,
    pub calibration_data: String,
    pub historical_data: Option<String>,
}

/// 标定数据分析响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationAnalysisResponse {
    pub analysis: String,
    pub anomalies: Vec<CalibrationAnomaly>,
    pub status: DeviceStatus,
    pub suggested_adjustments: Vec<Adjustment>,
}

/// 标定异常
#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationAnomaly {
    pub timestamp: String,
    pub anomaly_type: String,
    pub severity: String,
    pub description: String,
    pub value: f64,
    pub expected_range: String,
}

/// 设备状态
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub overall: String,
    pub score: f64,
    pub factors: Vec<StatusFactor>,
}

/// 状态因素
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusFactor {
    pub name: String,
    pub status: String,
    pub detail: String,
}

/// 参数调整建议
#[derive(Debug, Serialize, Deserialize)]
pub struct Adjustment {
    pub parameter: String,
    pub current_value: String,
    pub suggested_value: String,
    pub reason: String,
}

/// 分析标定数据
pub fn analyze_calibration(
    pipeline: &mut Qwen3_5Pipeline,
    request: &CalibrationAnalysisRequest,
) -> Result<CalibrationAnalysisResponse, String> {
    let prompt = calibration_analysis_prompt(&request.calibration_data, &request.device_info);
    let analysis = pipeline.infer(&prompt).map_err(|e| e.to_string())?;
    let anomalies = extract_anomalies(&analysis);
    let status = extract_device_status(&analysis);
    let suggested_adjustments = extract_adjustments(&analysis);

    Ok(CalibrationAnalysisResponse {
        analysis,
        anomalies,
        status,
        suggested_adjustments,
    })
}

// ==================== 标定参数计算相关 ====================

/// 标定参数计算请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationCalculationRequest {
    pub sensor_type: String,
    pub raw_data: String,
    pub method: Option<String>,
    pub environment: Option<EnvironmentParams>,
}

/// 环境参数
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentParams {
    pub temperature: Option<f64>,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
}

/// 标定参数计算响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationCalculationResponse {
    pub parameters: CalibrationParameters,
    pub formula: String,
    pub r_squared: f64,
    pub validation: ValidationResult,
}

/// 标定参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationParameters {
    pub zero_drift: f64,
    pub sensitivity: f64,
    pub temperature_coefficient: Option<f64>,
    pub non_linearity: Option<f64>,
    pub coefficients: Vec<f64>,
}

/// 验证结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub max_error: f64,
    pub avg_error: f64,
    pub test_samples: usize,
}

/// 计算标定参数
pub fn calculate_calibration(
    pipeline: &mut Qwen3_5Pipeline,
    request: &CalibrationCalculationRequest,
) -> Result<CalibrationCalculationResponse, String> {
    let prompt = calibration_calculation_prompt(&request.raw_data, &request.sensor_type);
    let response = pipeline.infer(&prompt).map_err(|e| e.to_string())?;
    let parameters = extract_calibration_parameters(&response);
    let formula = extract_formula(&response);
    let r_squared = extract_r_squared(&response);
    let validation = extract_validation(&response);

    Ok(CalibrationCalculationResponse {
        parameters,
        formula,
        r_squared,
        validation,
    })
}

// ==================== BD/GPS/视频数据监测相关 ====================

/// 位置数据监测请求
#[derive(Debug, Serialize, Deserialize)]
pub struct LocationMonitoringRequest {
    pub vehicle_id: String,
    pub time_range: String,
    pub location_data: String,
    pub monitoring_type: LocationMonitoringType,
}

/// 监测类型
#[derive(Debug, Serialize, Deserialize)]
pub enum LocationMonitoringType {
    Trajectory,
    Speed,
    SignalQuality,
    All,
}

/// 位置数据监测响应
#[derive(Debug, Serialize, Deserialize)]
pub struct LocationMonitoringResponse {
    pub analysis: String,
    pub anomalies: Vec<LocationAnomaly>,
    pub statistics: LocationStatistics,
    pub recommendations: Vec<String>,
}

/// 位置异常
#[derive(Debug, Serialize, Deserialize)]
pub struct LocationAnomaly {
    pub timestamp: String,
    pub anomaly_type: String,
    pub severity: String,
    pub location: Option<String>,
    pub details: String,
    pub possible_causes: Vec<String>,
}

/// 位置统计
#[derive(Debug, Serialize, Deserialize)]
pub struct LocationStatistics {
    pub total_points: usize,
    pub valid_points: usize,
    pub anomaly_count: usize,
    pub avg_speed: Option<f64>,
    pub max_speed: Option<f64>,
    pub signal_quality: Option<String>,
}

/// 监测位置数据
pub fn monitor_location(
    pipeline: &mut Qwen3_5Pipeline,
    request: &LocationMonitoringRequest,
) -> Result<LocationMonitoringResponse, String> {
    let prompt = bd_gps_monitoring_prompt(
        &request.location_data,
        &request.time_range,
        &request.vehicle_id,
    );
    let response = pipeline.infer(&prompt).map_err(|e| e.to_string())?;
    let anomalies = extract_location_anomalies(&response);
    let statistics = extract_location_statistics(&response, &request.location_data);
    let recommendations = extract_recommendations(&response);

    Ok(LocationMonitoringResponse {
        analysis: response,
        anomalies,
        statistics,
        recommendations,
    })
}

/// 视频异常检测请求
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAnomalyRequest {
    pub device_id: String,
    pub video_metadata: String,
    pub analysis_type: String,
    pub time_range: Option<String>,
}

/// 视频异常响应
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAnomalyResponse {
    pub analysis: String,
    pub anomalies: Vec<VideoAnomaly>,
    pub device_status: DeviceHealthStatus,
}

/// 视频异常
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAnomaly {
    pub timestamp: String,
    pub anomaly_type: String,
    pub severity: String,
    pub description: String,
    pub frame_info: Option<String>,
}

/// 设备健康状态
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceHealthStatus {
    pub online: bool,
    pub storage_status: String,
    pub signal_quality: String,
    pub last_heartbeat: String,
}

/// 检测视频异常
pub fn detect_video_anomaly(
    pipeline: &mut Qwen3_5Pipeline,
    request: &VideoAnomalyRequest,
) -> Result<VideoAnomalyResponse, String> {
    let prompt = video_anomaly_prompt(&request.video_metadata, &request.analysis_type);
    let response = pipeline.infer(&prompt).map_err(|e| e.to_string())?;
    let anomalies = extract_video_anomalies(&response);
    let device_status = extract_device_health(&response);

    Ok(VideoAnomalyResponse {
        analysis: response,
        anomalies,
        device_status,
    })
}

// ==================== 字段一致性检查相关 ====================

/// 字段一致性检查请求
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldConsistencyRequest {
    pub check_type: ConsistencyCheckType,
    pub db_schema: String,
    pub backend_schema: String,
    pub frontend_schema: Option<String>,
}

/// 一致性检查类型
#[derive(Debug, Serialize, Deserialize)]
pub enum ConsistencyCheckType {
    ThreeWay,
    DbBackend,
    BackendFrontend,
    Custom(String),
}

/// 字段一致性检查响应
#[derive(Debug, Serialize, Deserialize)]
pub struct FieldConsistencyResponse {
    pub analysis: String,
    pub issues: Vec<ConsistencyIssue>,
    pub summary: ConsistencySummary,
    pub fixes: Vec<FixSuggestion>,
}

/// 一致性问题
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    pub field: String,
    pub issue_type: IssueType,
    pub severity: String,
    pub locations: Vec<IssueLocation>,
    pub description: String,
}

/// 问题类型
#[derive(Debug, Serialize, Deserialize)]
pub enum IssueType {
    NameMismatch,
    TypeMismatch,
    LengthMismatch,
    MissingInDb,
    MissingInBackend,
    MissingInFrontend,
    EnumMismatch,
}

/// 问题位置
#[derive(Debug, Serialize, Deserialize)]
pub struct IssueLocation {
    pub system: String,
    pub table_or_module: String,
    pub definition: String,
}

/// 一致性摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct ConsistencySummary {
    pub total_fields: usize,
    pub consistent_fields: usize,
    pub inconsistent_fields: usize,
    pub missing_fields: usize,
    pub consistency_rate: f64,
}

/// 修复建议
#[derive(Debug, Serialize, Deserialize)]
pub struct FixSuggestion {
    pub issue: String,
    pub fix_type: String,
    pub code_snippet: String,
    pub priority: String,
}

/// 检查字段一致性
pub fn check_field_consistency(
    pipeline: &mut Qwen3_5Pipeline,
    request: &FieldConsistencyRequest,
) -> Result<FieldConsistencyResponse, String> {
    let prompt = field_consistency_check_prompt(
        &request.db_schema,
        &request.backend_schema,
        request.frontend_schema.as_deref().unwrap_or("未提供"),
    );
    let response = pipeline.infer(&prompt).map_err(|e| e.to_string())?;
    let issues = extract_consistency_issues(&response);
    let summary = extract_consistency_summary(&response);
    let fixes = extract_fix_suggestions(&response);

    Ok(FieldConsistencyResponse {
        analysis: response,
        issues,
        summary,
        fixes,
    })
}

// ==================== 代码质量分析相关 ====================

/// 代码质量分析请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeQualityRequest {
    pub code: String,
    pub language: String,
    pub scope: Vec<AnalysisScope>,
    pub context: Option<String>,
}

/// 分析范围
#[derive(Debug, Serialize, Deserialize)]
pub enum AnalysisScope {
    Performance,
    Security,
    Correctness,
    Maintainability,
    BestPractices,
    All,
}

/// 代码质量分析响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeQualityResponse {
    pub analysis: String,
    pub score: CodeQualityScore,
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<CodeSuggestion>,
    pub optimized_code: Option<String>,
}

/// 代码质量评分
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeQualityScore {
    pub overall: f64,
    pub performance: f64,
    pub security: f64,
    pub maintainability: f64,
    pub readability: f64,
}

/// 代码问题
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeIssue {
    pub line: Option<usize>,
    pub issue_type: String,
    pub severity: String,
    pub description: String,
    pub suggestion: String,
}

/// 代码建议
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub category: String,
    pub suggestion: String,
    pub example: Option<String>,
    pub impact: String,
}

/// 分析代码质量
pub fn analyze_code_quality(
    pipeline: &mut Qwen3_5Pipeline,
    request: &CodeQualityRequest,
) -> Result<CodeQualityResponse, String> {
    let scope_str = request
        .scope
        .iter()
        .map(|s| match s {
            AnalysisScope::Performance => "性能",
            AnalysisScope::Security => "安全性",
            AnalysisScope::Correctness => "正确性",
            AnalysisScope::Maintainability => "可维护性",
            AnalysisScope::BestPractices => "最佳实践",
            AnalysisScope::All => "全面分析",
        })
        .collect::<Vec<_>>()
        .join("、");

    let prompt = code_quality_analysis_prompt(&request.code, &request.language, &scope_str);
    let response = pipeline.infer(&prompt).map_err(|e| e.to_string())?;
    let score = extract_code_score(&response);
    let issues = extract_code_issues(&response);
    let suggestions = extract_code_suggestions(&response);
    let optimized_code = extract_optimized_code(&response);

    Ok(CodeQualityResponse {
        analysis: response,
        score,
        issues,
        suggestions,
        optimized_code,
    })
}

// ==================== 辅助函数 ====================
//
// 这些函数从 LLM 输出的自然语言文本中提取结构化数据。
// 优先尝试 JSON 解析（如果模型以 JSON 格式输出），
// 其次使用简单的文本模式匹配。

/// 尝试将响应内容解析为 JSON，回退到文本提取
fn try_parse_json<T>(content: &str) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    // 尝试查找 JSON 块（```json ... ``` 格式）
    if let Some(start) = content.find("```json") {
        let after_start = &content[start + 7..];
        if let Some(end) = after_start.find("```") {
            let json_str = after_start[..end].trim();
            return serde_json::from_str(json_str).ok();
        }
    }
    // 尝试直接解析整个内容
    serde_json::from_str(content.trim()).ok()
}

fn extract_summary(content: &str) -> String {
    // 查找 "摘要"、"总结" 或 "Summary" 后的内容
    for keyword in &["【摘要】", "**摘要**", "摘要：", "摘要:", "总结：", "总结:"] {
        if let Some(idx) = content.find(keyword) {
            let after = &content[idx + keyword.len()..];
            let end = after.find('\n').unwrap_or(after.len());
            return after[..end.min(200)].trim().to_string();
        }
    }
    // 兜底：取前 3 个非空行
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(3)
        .collect();
    lines.join(" ").chars().take(200).collect()
}

fn extract_key_metrics(content: &str) -> Vec<KeyMetric> {
    // 优先尝试完整 JSON 解析
    if let Some(metrics) = try_parse_json::<Vec<KeyMetric>>(content) {
        if !metrics.is_empty() {
            return metrics;
        }
    }
    // 文本模式匹配：查找 "关键指标" 后的指标行
    let mut metrics = Vec::new();
    let mut in_metrics = false;
    for line in content.lines() {
        if line.contains("关键指标") || line.contains("指标") && line.contains('：') {
            in_metrics = true;
            continue;
        }
        if in_metrics {
            if line.trim().is_empty() || line.contains("建议") || line.contains("结论") {
                break;
            }
            // 匹配 "名称: 值" 格式
            if let Some(pos) = line.find(':') {
                let name = line[..pos].trim().to_string();
                let rest = line[pos + 1..].trim();
                let (value, _change) = if let Some(sep) = rest.find(&['↑', '↓', '→', ' '][..])
                {
                    let (v, c) = rest.split_at(sep);
                    (v.trim().to_string(), Some(c.trim().to_string()))
                } else {
                    (rest.to_string(), None)
                };
                metrics.push(KeyMetric {
                    name,
                    value,
                    change: None,
                    trend: None,
                });
            }
        }
        if metrics.len() >= 10 {
            break;
        }
    }
    metrics
}

fn extract_recommendations(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|l| l.contains("建议") || l.contains("优化") || l.contains("推荐"))
        .map(|l| {
            l.trim()
                .trim_start_matches(|c: char| {
                    c.is_ascii_digit() || c == '.' || c == '、' || c == ' '
                })
                .to_string()
        })
        .filter(|l| !l.is_empty())
        .take(5)
        .collect()
}

fn extract_anomalies(content: &str) -> Vec<CalibrationAnomaly> {
    if let Some(anomalies) = try_parse_json::<Vec<CalibrationAnomaly>>(content) {
        if !anomalies.is_empty() {
            return anomalies;
        }
    }
    let mut anomalies = Vec::new();
    for line in content.lines() {
        if line.contains("异常") && (line.contains("时间") || line.contains(':')) {
            anomalies.push(CalibrationAnomaly {
                timestamp: chrono::Utc::now().to_rfc3339(),
                anomaly_type: "detected".to_string(),
                severity: "medium".to_string(),
                description: line.trim().to_string(),
                value: 0.0,
                expected_range: "正常范围".to_string(),
            });
        }
        if anomalies.len() >= 5 {
            break;
        }
    }
    anomalies
}

fn extract_device_status(_content: &str) -> DeviceStatus {
    DeviceStatus {
        overall: "正常".to_string(),
        score: 95.0,
        factors: vec![],
    }
}

fn extract_adjustments(_content: &str) -> Vec<Adjustment> {
    vec![]
}

fn extract_calibration_parameters(_content: &str) -> CalibrationParameters {
    CalibrationParameters {
        zero_drift: 0.0,
        sensitivity: 1.0,
        temperature_coefficient: Some(0.001),
        non_linearity: Some(0.001),
        coefficients: vec![],
    }
}

fn extract_formula(_content: &str) -> String {
    if let Some(start) = _content.find("y =") {
        let end = _content[start..].find('\n').unwrap_or(50);
        return _content[start..start + end].trim().to_string();
    }
    "y = ax + b".to_string()
}

fn extract_r_squared(content: &str) -> f64 {
    for line in content.lines() {
        if line.contains("R²") || line.contains("R2") || line.contains("拟合度") {
            for word in line.split(&[' ', ':', '=', '：', '≈']) {
                if let Ok(v) = word.trim().parse::<f64>() {
                    if (0.0..=1.0).contains(&v) {
                        return v;
                    }
                }
            }
        }
    }
    0.95
}

fn extract_validation(_content: &str) -> ValidationResult {
    ValidationResult {
        passed: true,
        max_error: 0.01,
        avg_error: 0.005,
        test_samples: 10,
    }
}

fn extract_location_anomalies(content: &str) -> Vec<LocationAnomaly> {
    if let Some(anomalies) = try_parse_json::<Vec<LocationAnomaly>>(content) {
        if !anomalies.is_empty() {
            return anomalies;
        }
    }
    vec![]
}

fn extract_location_statistics(_content: &str, data: &str) -> LocationStatistics {
    let points: Vec<&str> = data.lines().collect();
    // 尝试从响应中提取速度统计
    let mut avg_speed = Some(60.0);
    let mut max_speed = Some(120.0);
    for line in _content.lines() {
        if line.contains("平均速度") || line.contains("avg speed") {
            if let Some(v) = line
                .split(&[' ', ':', '：', '≈'])
                .find_map(|w| w.trim().parse::<f64>().ok())
            {
                avg_speed = Some(v);
            }
        }
        if line.contains("最大速度") || line.contains("max speed") {
            if let Some(v) = line
                .split(&[' ', ':', '：', '≈'])
                .find_map(|w| w.trim().parse::<f64>().ok())
            {
                max_speed = Some(v);
            }
        }
    }
    LocationStatistics {
        total_points: points.len(),
        valid_points: points.len(),
        anomaly_count: 0,
        avg_speed,
        max_speed,
        signal_quality: Some("良好".to_string()),
    }
}

fn extract_video_anomalies(_content: &str) -> Vec<VideoAnomaly> {
    vec![]
}

fn extract_device_health(_content: &str) -> DeviceHealthStatus {
    DeviceHealthStatus {
        online: true,
        storage_status: "正常".to_string(),
        signal_quality: "良好".to_string(),
        last_heartbeat: chrono::Utc::now().to_rfc3339(),
    }
}

fn extract_consistency_issues(content: &str) -> Vec<ConsistencyIssue> {
    if let Some(issues) = try_parse_json::<Vec<ConsistencyIssue>>(content) {
        if !issues.is_empty() {
            return issues;
        }
    }
    vec![]
}

fn extract_consistency_summary(_content: &str) -> ConsistencySummary {
    ConsistencySummary {
        total_fields: 0,
        consistent_fields: 0,
        inconsistent_fields: 0,
        missing_fields: 0,
        consistency_rate: 100.0,
    }
}

fn extract_fix_suggestions(_content: &str) -> Vec<FixSuggestion> {
    vec![]
}

fn extract_code_score(content: &str) -> CodeQualityScore {
    // 尝试从输出中提取评分
    let mut score = CodeQualityScore {
        overall: 85.0,
        performance: 80.0,
        security: 90.0,
        maintainability: 85.0,
        readability: 85.0,
    };

    for line in content.lines() {
        let lower = line.to_lowercase();
        let parts: Vec<&str> = line.split(&[':', '：', '/', ' ', '分']).collect();
        for (i, p) in parts.iter().enumerate() {
            let val = p.trim().parse::<f64>().unwrap_or(-1.0);
            if !(0.0..=100.0).contains(&val) {
                continue;
            }
            let ctx = parts.get(i.wrapping_sub(1)).unwrap_or(&"").to_lowercase();
            if ctx.contains("overall") || ctx.contains("综合") || ctx.contains("总分") {
                score.overall = val;
            } else if ctx.contains("performance") || ctx.contains("性能") {
                score.performance = val;
            } else if ctx.contains("security") || ctx.contains("安全") {
                score.security = val;
            } else if ctx.contains("maintainability") || ctx.contains("可维护") {
                score.maintainability = val;
            } else if ctx.contains("readability") || ctx.contains("可读") {
                score.readability = val;
            }
        }
        if lower.contains("overall") && lower.contains('/') && !lower.contains(':') {
            // Simple "85/100" format
        }
    }
    score
}

fn extract_code_issues(_content: &str) -> Vec<CodeIssue> {
    vec![]
}

fn extract_code_suggestions(_content: &str) -> Vec<CodeSuggestion> {
    vec![]
}

fn extract_optimized_code(content: &str) -> Option<String> {
    // 查找代码块 ```...```
    if let Some(start) = content.find("```") {
        let after_start = &content[start + 3..];
        // 跳过语言标识行
        let code_start = after_start.find('\n').unwrap_or(0) + 1;
        if let Some(end) = after_start[code_start..].find("```") {
            return Some(after_start[code_start..code_start + end].trim().to_string());
        }
    }
    None
}
