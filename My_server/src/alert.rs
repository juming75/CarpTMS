//! / 告警服务模块
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};

use crate::central::config::{AlertThresholds, MonitorConfig};
use crate::metrics::{
    API_REQUESTS_TOTAL, API_REQUEST_DURATION, CACHE_HIT_RATE, CPU_USAGE, DB_CONNECTIONS_IN_USE,
    DB_CONNECTIONS_TOTAL, DISK_USAGE, JWT_TOKENS_VALIDATED, MEMORY_USAGE,
    SERVICE_INSTANCES_HEALTHY, SERVICE_INSTANCES_UNHEALTHY,
};

// 告警级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
}

// 告警消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMessage {
    pub level: AlertLevel,
    pub message: String,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub timestamp: u64,
}

// 告警服务
pub struct AlertService {
    pool: Arc<PgPool>,
    monitor_config: MonitorConfig,
    alert_thresholds: AlertThresholds,
}

impl AlertService {
    pub fn new(pool: Arc<PgPool>, monitor_config: MonitorConfig) -> Self {
        Self {
            pool,
            alert_thresholds: monitor_config.alert_thresholds.clone(),
            monitor_config,
        }
    }

    // 启动告警服务
    pub async fn start(&self) {
        if !self.monitor_config.enable_monitor {
            info!("Alert service disabled");
            return;
        }

        info!(
            "Starting alert service with interval: {} seconds",
            self.monitor_config.monitor_interval
        );

        // 每5秒检查一次告警
        let mut interval = interval(Duration::from_secs(self.monitor_config.monitor_interval));

        loop {
            interval.tick().await;

            // 检查各种指标
            self.check_api_metrics().await;
            self.check_db_metrics().await;
            self.check_jwt_metrics().await;
            self.check_business_metrics().await;
            self.check_system_resources().await;
            self.check_service_registry().await;
            self.check_cache_metrics().await;
        }
    }

    // 检查API指标
    async fn check_api_metrics(&self) {
        // 计算API错误率
        let all_requests = API_REQUESTS_TOTAL.with_label_values(&["", "", ""]).get();
        let error_requests = API_REQUESTS_TOTAL.with_label_values(&["", "", "500"]).get()
            + API_REQUESTS_TOTAL.with_label_values(&["", "", "400"]).get()
            + API_REQUESTS_TOTAL.with_label_values(&["", "", "401"]).get()
            + API_REQUESTS_TOTAL.with_label_values(&["", "", "403"]).get()
            + API_REQUESTS_TOTAL.with_label_values(&["", "", "404"]).get();

        let error_rate = if all_requests > 0 {
            (error_requests as f64 / all_requests as f64) * 100.0
        } else {
            0.0
        };

        // 检查API错误率
        if error_rate > self.alert_thresholds.api_error_rate_threshold {
            self.send_alert(
                AlertLevel::Warning,
                "API错误率过高",
                "api_error_rate",
                error_rate,
                self.alert_thresholds.api_error_rate_threshold,
            );
        }

        // 检查API请求延迟
        let api_duration = API_REQUEST_DURATION
            .with_label_values(&["", "", ""])
            .get_sample_sum()
            / API_REQUEST_DURATION
                .with_label_values(&["", "", ""])
                .get_sample_count() as f64;
        if api_duration > self.alert_thresholds.api_request_duration_threshold {
            self.send_alert(
                AlertLevel::Warning,
                "API请求延迟过高",
                "api_request_duration",
                api_duration,
                self.alert_thresholds.api_request_duration_threshold,
            );
        }
    }

    // 检查数据库指标
    async fn check_db_metrics(&self) {
        // 检查数据库连接使用率
        let total_connections = DB_CONNECTIONS_TOTAL.get();
        let in_use_connections = DB_CONNECTIONS_IN_USE.get();

        let usage_rate = if total_connections > 0.0 {
            (in_use_connections / total_connections) * 100.0
        } else {
            0.0
        };

        if usage_rate > self.alert_thresholds.db_connections_usage_threshold {
            self.send_alert(
                AlertLevel::Warning,
                "数据库连接使用率过高",
                "db_connections_usage",
                usage_rate,
                self.alert_thresholds.db_connections_usage_threshold,
            );
        }
    }

    // 检查JWT指标
    async fn check_jwt_metrics(&self) {
        // 计算JWT验证失败率
        let total_validations = JWT_TOKENS_VALIDATED.with_label_values(&["success"]).get()
            + JWT_TOKENS_VALIDATED.with_label_values(&["failure"]).get();
        let failure_count = JWT_TOKENS_VALIDATED.with_label_values(&["failure"]).get();

        let failure_rate = if total_validations > 0 {
            (failure_count as f64 / total_validations as f64) * 100.0
        } else {
            0.0
        };

        if failure_rate > self.alert_thresholds.jwt_validation_failure_rate {
            self.send_alert(
                AlertLevel::Warning,
                "JWT验证失败率过高",
                "jwt_validation_failure_rate",
                failure_rate,
                self.alert_thresholds.jwt_validation_failure_rate,
            );
        }
    }

    // 检查业务指标
    async fn check_business_metrics(&self) {
        // 检查设备离线率
        let total_devices: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM devices")
            .fetch_one(&*self.pool)
            .await
            .unwrap_or_default();

        if total_devices > 0 {
            let offline_devices: i32 =
                sqlx::query_scalar("SELECT COUNT(*) FROM devices WHERE status = 0")
                    .fetch_one(&*self.pool)
                    .await
                    .unwrap_or_default();

            let offline_rate = (offline_devices as f64 / total_devices as f64) * 100.0;

            if offline_rate > self.alert_thresholds.device_offline_rate_threshold {
                self.send_alert(
                    AlertLevel::Warning,
                    "设备离线率过高",
                    "device_offline_rate",
                    offline_rate,
                    self.alert_thresholds.device_offline_rate_threshold,
                );
            }
        }

        // 检查待处理订单数
        let pending_orders: i32 =
            sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE order_status = 1")
                .fetch_one(&*self.pool)
                .await
                .unwrap_or_default();

        if pending_orders as f64 > self.alert_thresholds.orders_pending_threshold {
            self.send_alert(
                AlertLevel::Info,
                "待处理订单数异常",
                "orders_pending",
                pending_orders as f64,
                self.alert_thresholds.orders_pending_threshold,
            );
        }
    }

    // 检查系统资源指标
    async fn check_system_resources(&self) {
        // CPU使用率告警
        let cpu_usage = CPU_USAGE.get();
        if cpu_usage > self.alert_thresholds.cpu_usage_threshold {
            self.send_alert(
                AlertLevel::Warning,
                "CPU使用率过高",
                "cpu_usage_percentage",
                cpu_usage,
                self.alert_thresholds.cpu_usage_threshold,
            );
        }

        // 内存使用率告警
        let memory_usage = MEMORY_USAGE.get();
        if memory_usage > self.alert_thresholds.memory_usage_threshold {
            self.send_alert(
                AlertLevel::Warning,
                "内存使用率过高",
                "memory_usage_percentage",
                memory_usage,
                self.alert_thresholds.memory_usage_threshold,
            );
        }

        // 磁盘使用率告警
        let disk_usage = DISK_USAGE.get();
        if disk_usage > self.alert_thresholds.disk_usage_threshold {
            self.send_alert(
                AlertLevel::Error,
                "磁盘使用率过高",
                "disk_usage_percentage",
                disk_usage,
                self.alert_thresholds.disk_usage_threshold,
            );
        }
    }

    // 检查服务注册中心指标
    async fn check_service_registry(&self) {
        // 服务实例健康状态告警
        let healthy_instances = SERVICE_INSTANCES_HEALTHY.get();
        let unhealthy_instances = SERVICE_INSTANCES_UNHEALTHY.get();

        if unhealthy_instances > self.alert_thresholds.service_instances_unhealthy_threshold {
            self.send_alert(
                AlertLevel::Warning,
                "不健康服务实例过多",
                "service_instances_unhealthy",
                unhealthy_instances,
                self.alert_thresholds.service_instances_unhealthy_threshold,
            );
        }

        // 如果没有健康实例,发送错误告警
        if healthy_instances == 0.0 && SERVICE_INSTANCES_HEALTHY.get() > 0.0 {
            self.send_alert(
                AlertLevel::Error,
                "所有服务实例均不健康",
                "service_instances_healthy",
                healthy_instances,
                0.0,
            );
        }
    }

    // 检查缓存指标
    async fn check_cache_metrics(&self) {
        // 缓存命中率告警
        let cache_hit_rate = CACHE_HIT_RATE.get();
        if cache_hit_rate < self.alert_thresholds.cache_hit_rate_threshold {
            self.send_alert(
                AlertLevel::Warning,
                "缓存命中率过低",
                "cache_hit_rate",
                cache_hit_rate,
                self.alert_thresholds.cache_hit_rate_threshold,
            );
        }
    }

    // 发送告警
    fn send_alert(
        &self,
        level: AlertLevel,
        message: &str,
        metric_name: &str,
        current_value: f64,
        threshold: f64,
    ) {
        // 记录告警日志
        let alert_msg = format!(
            "[ALERT] {}: {} - {}: {:.2} (阈值: {:.2})
",
            match level {
                AlertLevel::Info => "INFO",
                AlertLevel::Warning => "WARNING",
                AlertLevel::Error => "ERROR",
            },
            message,
            metric_name,
            current_value,
            threshold
        );

        match level {
            AlertLevel::Info => info!("{}", alert_msg),
            AlertLevel::Warning => warn!("{}", alert_msg),
            AlertLevel::Error => error!("{}", alert_msg),
        }

        // 这里可以添加其他告警方式,如邮件、短信、WebSocket等
        // 例如:self.send_email_alert(&alert_msg).await;
        // 例如:self.send_sms_alert(&alert_msg).await;
        // 例如:self.send_websocket_alert(&alert_msg).await;
    }
}
