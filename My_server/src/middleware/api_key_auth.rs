use actix_web::{dev::ServiceRequest, error, web, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn api_key_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let api_key = credentials.token();

    if api_key.is_empty() {
        return Err((error::ErrorUnauthorized("API Key is required"), req));
    }

    let pool_option = req.app_data::<web::Data<sqlx::PgPool>>().cloned();

    let pool = match pool_option {
        Some(p) => p,
        None => {
            return Err((
                error::ErrorInternalServerError("Database pool not available"),
                req,
            ))
        }
    };

    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM openapi_platforms WHERE api_key = $1 AND status = 'active'",
    )
    .bind(api_key)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(count) if count > 0 => Ok(req),
        Ok(_) => Err((error::ErrorUnauthorized("Invalid or inactive API Key"), req)),
        Err(e) => {
            log::error!("API Key validation database error: {}", e);
            Err((error::ErrorInternalServerError("Authentication error"), req))
        }
    }
}
