// 监控和告警配置
// 使用Prometheus进行指标采集和Grafana进行可视化

use prometheus::{Counter, Histogram, Gauge, Registry, IntGauge};
use prometheus::Encoder;
use std::sync::Arc;
use actix_web::web;
use lazy_static::lazy_static;

/// 监控指标
pub struct Metrics {
    /// HTTP请求计数
    pub http_requests_total: Counter,
    /// HTTP请求延迟
    pub http_request_duration: Histogram,
    /// API响应时间
    pub api_response_time: Histogram,
    /// 数据库查询时间
    pub db_query_duration: Histogram,
    /// 缓存命中率
    pub cache_hit_rate: Gauge,
    /// 当前活跃连接数
    pub active_connections: IntGauge,
    /// Redis操作计数
    pub redis_operations_total: Counter,
    /// 报表生成时间
    pub report_generation_duration: Histogram,
    /// WebSocket连接数
    pub websocket_connections: IntGauge,
    /// 数据库连接池大小
    pub db_pool_size: IntGauge,
    pub db_pool_idle: IntGauge,
    pub db_pool_active: IntGauge,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            http_requests_total: Counter::new(
                "http_requests_total",
                "Total number of HTTP requests"
            ).unwrap(),

            http_request_duration: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "http_request_duration_seconds",
                    "HTTP request latency in seconds"
                ).buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
            ).unwrap(),

            api_response_time: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "api_response_time_seconds",
                    "API response time in seconds"
                ).buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0])
            ).unwrap(),

            db_query_duration: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "db_query_duration_seconds",
                    "Database query duration in seconds"
                ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25])
            ).unwrap(),

            cache_hit_rate: Gauge::new(
                "cache_hit_rate",
                "Cache hit rate percentage"
            ).unwrap(),

            active_connections: IntGauge::new(
                "active_connections",
                "Number of active connections"
            ).unwrap(),

            redis_operations_total: Counter::new(
                "redis_operations_total",
                "Total number of Redis operations"
            ).unwrap(),

            report_generation_duration: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "report_generation_duration_seconds",
                    "Report generation duration in seconds"
                ).buckets(vec![1.0, 2.5, 5.0, 10.0, 20.0, 30.0])
            ).unwrap(),

            websocket_connections: IntGauge::new(
                "websocket_connections",
                "Number of WebSocket connections"
            ).unwrap(),

            db_pool_size: IntGauge::new(
                "db_pool_size",
                "Database pool size"
            ).unwrap(),

            db_pool_idle: IntGauge::new(
                "db_pool_idle",
                "Database pool idle connections"
            ).unwrap(),

            db_pool_active: IntGauge::new(
                "db_pool_active",
                "Database pool active connections"
            ).unwrap(),
        }
    }

    /// 注册所有指标到Registry
    pub fn register(&self, registry: &Registry) {
        registry.register(Box::new(self.http_requests_total.clone())).unwrap();
        registry.register(Box::new(self.http_request_duration.clone())).unwrap();
        registry.register(Box::new(self.api_response_time.clone())).unwrap();
        registry.register(Box::new(self.db_query_duration.clone())).unwrap();
        registry.register(Box::new(self.cache_hit_rate.clone())).unwrap();
        registry.register(Box::new(self.active_connections.clone())).unwrap();
        registry.register(Box::new(self.redis_operations_total.clone())).unwrap();
        registry.register(Box::new(self.report_generation_duration.clone())).unwrap();
        registry.register(Box::new(self.websocket_connections.clone())).unwrap();
        registry.register(Box::new(self.db_pool_size.clone())).unwrap();
        registry.register(Box::new(self.db_pool_idle.clone())).unwrap();
        registry.register(Box::new(self.db_pool_active.clone())).unwrap();
    }
}

lazy_static! {
    /// 全局监控指标
    pub static ref METRICS: Arc<Metrics> = Arc::new(Metrics::new());
}

/// 告警规则
pub struct AlertRules;

impl AlertRules {
    /// API响应时间告警
    pub const API_RESPONSE_TIME_WARNING: f64 = 0.5;   // 500ms
    pub const API_RESPONSE_TIME_CRITICAL: f64 = 1.0;  // 1000ms

    /// 数据库查询时间告警
    pub const DB_QUERY_TIME_WARNING: f64 = 0.05;      // 50ms
    pub const DB_QUERY_TIME_CRITICAL: f64 = 0.1;       // 100ms

    /// 缓存命中率告警
    pub const CACHE_HIT_RATE_WARNING: f64 = 60.0;      // 60%
    pub const CACHE_HIT_RATE_CRITICAL: f64 = 40.0;     // 40%

    /// 连接数告警
    pub const ACTIVE_CONNECTIONS_WARNING: u32 = 800;
    pub const ACTIVE_CONNECTIONS_CRITICAL: u32 = 950;

    /// 错误率告警
    pub const ERROR_RATE_WARNING: f64 = 0.01;         // 1%
    pub const ERROR_RATE_CRITICAL: f64 = 0.05;        // 5%
}

/// Prometheus指标端点
pub async fn metrics_endpoint() -> web::HttpResponse {
    let registry = prometheus::default_registry();
    let metric_families = registry.gather();
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    web::HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(String::from_utf8(buffer).unwrap())
}

/// Grafana仪表盘配置（JSON格式）
pub fn grafana_dashboard_config() -> serde_json::Value {
    serde_json::json!({
        "dashboard": {
            "title": "MyTMS BFF Dashboard",
            "tags": ["tms", "bff", "rust"],
            "timezone": "browser",
            "panels": [
                {
                    "title": "API响应时间",
                    "type": "graph",
                    "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0},
                    "targets": [{
                        "expr": "histogram_quantile(0.95, rate(api_response_time_seconds_bucket[5m]))",
                        "legendFormat": "P95"
                    }, {
                        "expr": "histogram_quantile(0.99, rate(api_response_time_seconds_bucket[5m]))",
                        "legendFormat": "P99"
                    }]
                },
                {
                    "title": "数据库查询时间",
                    "type": "graph",
                    "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0},
                    "targets": [{
                        "expr": "histogram_quantile(0.95, rate(db_query_duration_seconds_bucket[5m]))",
                        "legendFormat": "P95"
                    }]
                },
                {
                    "title": "缓存命中率",
                    "type": "graph",
                    "gridPos": {"h": 8, "w": 12, "x": 0, "y": 8},
                    "targets": [{
                        "expr": "cache_hit_rate",
                        "legendFormat": "Hit Rate"
                    }]
                },
                {
                    "title": "活跃连接数",
                    "type": "graph",
                    "gridPos": {"h": 8, "w": 12, "x": 12, "y": 8},
                    "targets": [{
                        "expr": "active_connections",
                        "legendFormat": "Connections"
                    }]
                },
                {
                    "title": "HTTP请求速率",
                    "type": "graph",
                    "gridPos": {"h": 8, "w": 12, "x": 0, "y": 16},
                    "targets": [{
                        "expr": "rate(http_requests_total[1m])",
                        "legendFormat": "Requests/sec"
                    }]
                },
                {
                    "title": "报表生成时间",
                    "type": "graph",
                    "gridPos": {"h": 8, "w": 12, "x": 12, "y": 16},
                    "targets": [{
                        "expr": "histogram_quantile(0.95, rate(report_generation_duration_seconds_bucket[5m]))",
                        "legendFormat": "P95"
                    }]
                },
                {
                    "title": "数据库连接池状态",
                    "type": "graph",
                    "gridPos": {"h": 8, "w": 12, "x": 0, "y": 24},
                    "targets": [{
                        "expr": "db_pool_size",
                        "legendFormat": "Pool Size"
                    }, {
                        "expr": "db_pool_idle",
                        "legendFormat": "Idle"
                    }, {
                        "expr": "db_pool_active",
                        "legendFormat": "Active"
                    }]
                },
                {
                    "title": "WebSocket连接数",
                    "type": "graph",
                    "gridPos": {"h": 8, "w": 12, "x": 12, "y": 24},
                    "targets": [{
                        "expr": "websocket_connections",
                        "legendFormat": "Connections"
                    }]
                }
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert!(metrics.http_requests_total.get() == 0.0);
    }

    #[test]
    fn test_grafana_config() {
        let config = grafana_dashboard_config();
        assert!(config["dashboard"]["title"] == "MyTMS BFF Dashboard");
    }

    #[test]
    fn test_alert_rules() {
        assert!(AlertRules::API_RESPONSE_TIME_WARNING == 0.5);
        assert!(AlertRules::CACHE_HIT_RATE_WARNING == 60.0);
    }
}
