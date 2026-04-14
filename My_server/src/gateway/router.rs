//! 架构模式路由器
//!
//! 根据当前配置的架构模式（MonolithDDD、MicroCRUD、MicroDDD）路由请求到相应的处理器。
//! 支持：
//! - 单体模式：直接调用 application 层
//! - 微服务模式：通过 gRPC 调用远程服务

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::config::{ArchitectureMode, ArchitectureConfig};
use crate::errors::AppResult;

/// 路由目标
#[derive(Debug, Clone)]
pub enum RouteTarget {
    /// 本地处理（单体模式）
    Local,
    /// 远程服务（微服务模式）
    Remote {
        service_name: String,
        endpoint: String,
    },
}

/// 路由规则
#[derive(Debug, Clone)]
pub struct RouteRule {
    /// 路由模式（如 "/api/vehicles/*"）
    pub pattern: String,
    /// 目标服务
    pub target: RouteTarget,
    /// 是否需要认证
    pub require_auth: bool,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
}

/// 架构模式路由器
pub struct ArchitectureRouter {
    /// 当前架构模式
    mode: ArchitectureMode,
    /// 路由规则表
    rules: Arc<RwLock<HashMap<String, RouteRule>>>,
    /// 架构配置
    config: ArchitectureConfig,
}

impl ArchitectureRouter {
    /// 创建新的路由器
    pub fn new(config: ArchitectureConfig) -> Self {
        let mode = config.mode;
        let rules = Arc::new(RwLock::new(HashMap::new()));
        
        Self { mode, rules, config }
    }

    /// 获取当前架构模式
    pub fn mode(&self) -> ArchitectureMode {
        self.mode
    }

    /// 注册路由规则
    pub async fn register_rule(&self, rule: RouteRule) {
        let mut rules = self.rules.write().await;
        rules.insert(rule.pattern.clone(), rule);
    }

    /// 批量注册路由规则
    pub async fn register_rules(&self, new_rules: Vec<RouteRule>) {
        let mut rules = self.rules.write().await;
        for rule in new_rules {
            rules.insert(rule.pattern.clone(), rule);
        }
    }

    /// 查找路由规则
    pub async fn find_rule(&self, path: &str) -> Option<RouteRule> {
        let rules = self.rules.read().await;
        
        // 精确匹配
        if let Some(rule) = rules.get(path) {
            return Some(rule.clone());
        }
        
        // 前缀匹配
        for (pattern, rule) in rules.iter() {
            if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len() - 1];
                if path.starts_with(prefix) {
                    return Some(rule.clone());
                }
            }
        }
        
        None
    }

    /// 路由请求
    pub async fn route(&self, path: &str) -> RouteTarget {
        match self.mode {
            ArchitectureMode::MonolithDDD => RouteTarget::Local,
            ArchitectureMode::MicroDDD => {
                // 查找路由规则
                if let Some(rule) = self.find_rule(path).await {
                    return rule.target;
                }
                
                // 默认根据路径推断服务
                self.infer_service_from_path(path)
            }
        }
    }

    /// 从路径推断服务
    fn infer_service_from_path(&self, path: &str) -> RouteTarget {
        // 路径到服务的映射
        let service_map = [
            ("/api/vehicles", "vehicle-service"),
            ("/api/drivers", "trip-service"),
            ("/api/trips", "trip-service"),
            ("/api/orders", "cargo-service"),
            ("/api/cargo", "cargo-service"),
            ("/api/billing", "billing-service"),
            ("/api/invoices", "billing-service"),
        ];
        
        for (prefix, service) in service_map {
            if path.starts_with(prefix) {
                return RouteTarget::Remote {
                    service_name: service.to_string(),
                    endpoint: path.to_string(),
                };
            }
        }
        
        // 默认路由到主服务
        RouteTarget::Local
    }

    /// 切换架构模式
    pub async fn switch_mode(&mut self, new_mode: ArchitectureMode) -> AppResult<()> {
        let old_mode = self.mode;
        self.mode = new_mode;
        self.config.mode = new_mode;
        
        log::info!(
            "Architecture mode switched from {} to {}",
            old_mode,
            new_mode
        );
        
        Ok(())
    }

    /// 获取配置
    pub fn config(&self) -> &ArchitectureConfig {
        &self.config
    }

    /// 初始化默认路由规则
    pub async fn initialize_default_rules(&self) {
        let rules = match self.mode {
            ArchitectureMode::MonolithDDD => {
                // 单体模式：所有请求本地处理
                vec![]
            }
            ArchitectureMode::MicroDDD => {
                // 微服务模式：根据服务拆分路由
                vec![
                    RouteRule {
                        pattern: "/api/vehicles/*".to_string(),
                        target: RouteTarget::Remote {
                            service_name: "vehicle-service".to_string(),
                            endpoint: "".to_string(),
                        },
                        require_auth: true,
                        timeout_ms: 5000,
                    },
                    RouteRule {
                        pattern: "/api/drivers/*".to_string(),
                        target: RouteTarget::Remote {
                            service_name: "trip-service".to_string(),
                            endpoint: "".to_string(),
                        },
                        require_auth: true,
                        timeout_ms: 5000,
                    },
                    RouteRule {
                        pattern: "/api/trips/*".to_string(),
                        target: RouteTarget::Remote {
                            service_name: "trip-service".to_string(),
                            endpoint: "".to_string(),
                        },
                        require_auth: true,
                        timeout_ms: 5000,
                    },
                    RouteRule {
                        pattern: "/api/orders/*".to_string(),
                        target: RouteTarget::Remote {
                            service_name: "cargo-service".to_string(),
                            endpoint: "".to_string(),
                        },
                        require_auth: true,
                        timeout_ms: 5000,
                    },
                    RouteRule {
                        pattern: "/api/cargo/*".to_string(),
                        target: RouteTarget::Remote {
                            service_name: "cargo-service".to_string(),
                            endpoint: "".to_string(),
                        },
                        require_auth: true,
                        timeout_ms: 5000,
                    },
                    RouteRule {
                        pattern: "/api/billing/*".to_string(),
                        target: RouteTarget::Remote {
                            service_name: "billing-service".to_string(),
                            endpoint: "".to_string(),
                        },
                        require_auth: true,
                        timeout_ms: 5000,
                    },
                    RouteRule {
                        pattern: "/api/invoices/*".to_string(),
                        target: RouteTarget::Remote {
                            service_name: "billing-service".to_string(),
                            endpoint: "".to_string(),
                        },
                        require_auth: true,
                        timeout_ms: 5000,
                    },
                ]
            }
        };
        
        self.register_rules(rules).await;
    }

    /// 验证路由配置
    pub async fn validate(&self) -> AppResult<()> {
        let rules = self.rules.read().await;
        
        for (pattern, rule) in rules.iter() {
            // 验证模式格式
            if pattern.is_empty() {
                return Err(crate::errors::AppError::validation_error(
                    "Route pattern cannot be empty",
                    None,
                ));
            }
            
            // 验证超时设置
            if rule.timeout_ms == 0 {
                return Err(crate::errors::AppError::validation_error(
                    &format!("Timeout cannot be zero for route: {}", pattern),
                    None,
                ));
            }
            
            // 微服务模式下验证远程服务配置
            if let RouteTarget::Remote { service_name, .. } = &rule.target {
                if service_name.is_empty() {
                    return Err(crate::errors::AppError::validation_error(
                        &format!("Service name cannot be empty for route: {}", pattern),
                        None,
                    ));
                }
            }
        }
        
        Ok(())
    }
}

impl Clone for ArchitectureRouter {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode,
            rules: self.rules.clone(),
            config: self.config.clone(),
        }
    }
}

/// 路由统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStats {
    /// 当前架构模式
    pub mode: String,
    /// 路由规则数量
    pub rule_count: usize,
    /// 本地路由数量
    pub local_routes: usize,
    /// 远程路由数量
    pub remote_routes: usize,
}

impl ArchitectureRouter {
    /// 获取路由统计信息
    pub async fn stats(&self) -> RouterStats {
        let rules = self.rules.read().await;
        
        let local_routes = rules.values()
            .filter(|r| matches!(r.target, RouteTarget::Local))
            .count();
        
        let remote_routes = rules.values()
            .filter(|r| matches!(r.target, RouteTarget::Remote { .. }))
            .count();
        
        RouterStats {
            mode: self.mode.to_string(),
            rule_count: rules.len(),
            local_routes,
            remote_routes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_creation() {
        let config = ArchitectureConfig::monolith_ddd();
        let router = ArchitectureRouter::new(config);
        
        assert_eq!(router.mode(), ArchitectureMode::MonolithDDD);
    }

    #[tokio::test]
    async fn test_route_inference() {
        let config = ArchitectureConfig::micro_ddd();
        let router = ArchitectureRouter::new(config);
        
        let target = router.route("/api/vehicles/123").await;
        assert!(matches!(target, RouteTarget::Remote { .. }));
        
        let config = ArchitectureConfig::monolith_ddd();
        let router = ArchitectureRouter::new(config);
        
        let target = router.route("/api/vehicles/123").await;
        assert!(matches!(target, RouteTarget::Local));
    }

    #[tokio::test]
    async fn test_register_rule() {
        let config = ArchitectureConfig::monolith_ddd();
        let router = ArchitectureRouter::new(config);
        
        let rule = RouteRule {
            pattern: "/api/test/*".to_string(),
            target: RouteTarget::Local,
            require_auth: true,
            timeout_ms: 5000,
        };
        
        router.register_rule(rule).await;
        
        let found = router.find_rule("/api/test/123").await;
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_switch_mode() {
        let config = ArchitectureConfig::monolith_ddd();
        let mut router = ArchitectureRouter::new(config);
        
        assert_eq!(router.mode(), ArchitectureMode::MonolithDDD);
        
        router.switch_mode(ArchitectureMode::MicroDDD).await.unwrap();
        assert_eq!(router.mode(), ArchitectureMode::MicroDDD);
    }

    #[tokio::test]
    async fn test_router_stats() {
        let config = ArchitectureConfig::micro_ddd();
        let router = ArchitectureRouter::new(config);
        router.initialize_default_rules().await;
        
        let stats = router.stats().await;
        assert!(stats.rule_count > 0);
        assert!(stats.remote_routes > 0);
    }
}
