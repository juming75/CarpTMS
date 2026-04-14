use actix_cors::Cors;

// CORS中间件配置
pub fn cors_middleware() -> Cors {
    Cors::default()
        .allowed_origin("http://localhost:5173")
        .allowed_origin("http://127.0.0.1:5173")
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec!["Content-Type", "Authorization", "Accept"])
        .supports_credentials()
        .max_age(3600)
}
