//! 仪表盘相关路由

use actix_web::{web, HttpResponse};

// 获取仪表盘统计数据
pub async fn get_dashboard_stats() -> HttpResponse {
    // 构建统计数据
    let stats = serde_json::json!({
        "todayOrders": 128,
        "onlineVehicles": 45,
        "todayIncome": 15680.50,
        "pendingOrders": 12
    });

    HttpResponse::Ok().json(stats)
}

// 获取仪表盘最新动态
pub async fn get_dashboard_dynamics() -> HttpResponse {
    // 构建最新动态数据
    let dynamics = serde_json::json!([
        {
            "id": "1",
            "title": "车辆123456 完成运输任务",
            "description": "车辆123456 成功完成了从北京到上海的运输任务",
            "time": "2026-04-07 08:30:00",
            "type": "success"
        },
        {
            "id": "2",
            "title": "车辆789012 发生告警",
            "description": "车辆789012 在高速公路上超速行驶",
            "time": "2026-04-07 07:15:00",
            "type": "warning"
        },
        {
            "id": "3",
            "title": "新订单创建",
            "description": "客户 ABC 公司创建了新的运输订单",
            "time": "2026-04-07 06:45:00",
            "type": "info"
        },
        {
            "id": "4",
            "title": "车辆345678 开始运输任务",
            "description": "车辆345678 开始执行从广州到深圳的运输任务",
            "time": "2026-04-07 05:30:00",
            "type": "success"
        },
        {
            "id": "5",
            "title": "系统维护",
            "description": "系统进行了例行维护，优化了性能",
            "time": "2026-04-07 04:00:00",
            "type": "info"
        }
    ]);

    HttpResponse::Ok().json(dynamics)
}

// 获取仪表盘待处理事项
pub async fn get_dashboard_tasks() -> HttpResponse {
    // 构建待处理事项数据
    let tasks = serde_json::json!([
        {
            "id": "1",
            "title": "处理车辆789012的超速告警",
            "priority": "high",
            "deadline": "2026-04-07 12:00:00",
            "status": "pending"
        },
        {
            "id": "2",
            "title": "审核新订单",
            "priority": "medium",
            "deadline": "2026-04-07 14:00:00",
            "status": "pending"
        },
        {
            "id": "3",
            "title": "更新车辆信息",
            "priority": "low",
            "deadline": "2026-04-08 10:00:00",
            "status": "pending"
        },
        {
            "id": "4",
            "title": "生成月度报表",
            "priority": "medium",
            "deadline": "2026-04-08 16:00:00",
            "status": "pending"
        },
        {
            "id": "5",
            "title": "检查设备状态",
            "priority": "low",
            "deadline": "2026-04-09 09:00:00",
            "status": "pending"
        }
    ]);

    HttpResponse::Ok().json(tasks)
}

// 配置仪表盘路由
pub fn configure_dashboard_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/dashboard/stats", web::get().to(get_dashboard_stats))
        .route("/dashboard/dynamics", web::get().to(get_dashboard_dynamics))
        .route("/dashboard/tasks", web::get().to(get_dashboard_tasks));
}
