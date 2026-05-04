//! 熔断器中间件
//!
//! 使用 `errors/circuit_breaker.rs` 的统一实现作为后端，
//! 提供 Actix-web 集成层。

use actix_web::{web, Error, HttpRequest};
use std::sync::Arc;

/// 熔断器中间件
///
/// 快速检查请求是否被熔断器拦截。
/// 需要先通过 `web::Data::<Arc<crate::errors::CircuitBreaker>>` 注入实例。
pub async fn circuit_breaker_middleware(
    _req: HttpRequest,
    circuit_breaker: web::Data<Arc<crate::errors::CircuitBreaker>>,
) -> Result<(), Error> {
    if !circuit_breaker.is_allowed() {
        return Err(actix_web::error::ErrorServiceUnavailable("服务暂时不可用"));
    }
    Ok(())
}
