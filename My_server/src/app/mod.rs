//! /! 应用配置模块
//!
//! 提供 CORS 配置辅助函数

use log::warn;

/// 构建 CORS 配置
pub fn build_cors(allowed_origins: &[String]) -> actix_cors::Cors {
    // 复制 allowed_origins 以便在闭包中使用
    let allowed_origins_owned: Vec<String> = allowed_origins.to_vec();
    
    actix_cors::Cors::default()
        .allowed_methods(["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers([
            "Authorization",
            "Content-Type",
            "Accept",
            "Origin",
            "X-Request-ID",
        ])
        .expose_headers(["Content-Length", "X-Requested-With", "X-Request-ID"])
        .max_age(3600)
        .supports_credentials()
        // 允许 WebSocket 连接
        .allowed_origin_fn(move |origin, _req| {
            // 允许所有 localhost Origins（开发环境）
            let origin_str = origin.as_bytes();
            if origin_str.starts_with(b"http://localhost") || 
               origin_str.starts_with(b"http://127.0.0.1") ||
               origin_str.starts_with(b"http://[::1]") {
                return true;
            }
            // 允许配置的 origins
            for allowed in &allowed_origins_owned {
                if origin_str == allowed.as_bytes() {
                    return true;
                }
            }
            warn!("CORS: Rejected origin: {:?}", origin);
            false
        })
}
