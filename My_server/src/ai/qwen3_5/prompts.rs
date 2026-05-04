//! Qwen3.5 业务场景提示词模板
//! 继承自旧的 qwen/prompts.rs，现已迁移到 qwen3_5 模块

/// 系统角色定义
pub const SYSTEM_ROLE: &str = r#"你是一个专业的车联网运输管理系统(TMS)AI助手，名为CarpTMS助手。
你的职责包括：
1. 生成专业的报表和分析报告
2. 分析标定数据并提供参数优化建议
3. 监测BD/GPS/视频数据并识别异常
4. 检查数据库与前后端字段一致性
5. 分析系统代码质量问题

请用中文回答，保持专业、简洁、实用的风格。
"#;

/// 报表生成提示词模板
pub fn report_generation_prompt(data_type: &str, data: &str, report_type: &str) -> String {
    format!(
        r#"请根据以下{}-{}数据生成一份专业的分析报告。

数据内容：
{}

报告要求：
1. 结构清晰，包含摘要、正文、结论
2. 使用表格展示关键数据
3. 提供数据趋势分析
4. 给出改进建议和优化方案

请生成报告："#,
        data_type, report_type, data
    )
}

/// 标定数据分析提示词模板
pub fn calibration_analysis_prompt(calibration_data: &str, device_info: &str) -> String {
    format!(
        r#"请分析以下标定数据，识别潜在问题和异常。

设备信息：
{}

标定数据：
{}

请分析：
1. 数据是否在正常范围内
2. 是否存在异常值或漂移
3. 设备状态评估
4. 建议的标定参数调整"#,
        device_info, calibration_data
    )
}

/// 标定参数计算提示词模板
pub fn calibration_calculation_prompt(raw_data: &str, sensor_type: &str) -> String {
    format!(
        r#"根据以下原始数据，计算最优的{}标定参数。

原始数据：
{}

请计算：
1. 零漂补偿值
2. 灵敏度系数
3. 温度补偿系数（如适用）
4. 线性修正参数
5. 输出标准的标定公式和参数表"#,
        sensor_type, raw_data
    )
}

/// BD/GPS数据监测提示词模板
pub fn bd_gps_monitoring_prompt(location_data: &str, time_range: &str, vehicle_id: &str) -> String {
    format!(
        r#"分析以下车辆的BD/GPS轨迹数据，识别异常行为。

车辆ID：{}
时间范围：{}

轨迹数据：
{}

请检测：
1. 轨迹是否正常（起点、终点、路线）
2. 是否存在漂移、跳跃、静止异常
3. 速度异常（超速、急加速/急刹车）
4. 定位丢失或信号干扰
5. 给出异常事件的详细信息和建议"#,
        vehicle_id, time_range, location_data
    )
}

/// 视频数据异常检测提示词模板
pub fn video_anomaly_prompt(video_metadata: &str, analysis_type: &str) -> String {
    format!(
        r#"分析以下视频监控数据，识别{}。

元数据：
{}

请检测：
1. 画面质量异常（遮挡、模糊、过曝）
2. 异常事件（物体遗留、人员聚集）
3. 设备状态异常（掉线、存储异常）
4. 与其他系统（GPS、BD）的时序一致性
5. 生成异常报告和处理建议"#,
        analysis_type, video_metadata
    )
}

/// 数据库字段一致性检查提示词模板
pub fn field_consistency_check_prompt(
    db_schema: &str,
    backend_schema: &str,
    frontend_schema: &str,
) -> String {
    format!(
        r#"请检查以下数据库、后端、前端三端字段定义的一致性。

【数据库表结构】
{}

【后端API定义】
{}

【前端模型定义】
{}

请检查：
1. 字段名称是否一致（大小写、下划线）
2. 数据类型是否匹配
3. 字段长度/精度是否一致
4. 是否存在字段缺失或多余
5. 枚举值是否统一
6. 给出详细的差异报告和修复建议"#,
        db_schema, backend_schema, frontend_schema
    )
}

/// 系统代码质量分析提示词模板
pub fn code_quality_analysis_prompt(
    code_snippet: &str,
    language: &str,
    analysis_scope: &str,
) -> String {
    format!(
        r#"请分析以下{}代码的质量问题，重点关注：{}。

代码片段：
```
{}
```

请进行以下分析：
1. 代码结构和设计模式评估
2. 潜在bug和安全漏洞识别
3. 性能问题（如循环、内存泄漏）
4. 代码风格和可维护性问题
5. 错误处理是否完善
6. 给出具体的改进建议和优化后的代码示例"#,
        language, analysis_scope, code_snippet
    )
}
