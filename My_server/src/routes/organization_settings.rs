//! Organization settings routes - delegates to OrganizationSettingsApplicationService

use actix_web::{web, HttpResponse};
use log::info;

use crate::application::services::organization_settings_service::OrganizationSettingsApplicationService;
use crate::errors::{success_response_with_message, AppResult};

pub use crate::application::services::organization_settings_service::{
    OrganizationBrandSettings, OrganizationThemeSettings,
};

#[utoipa::path(
    path = "/api/organizations/{organization_id}/settings/{setting_key}",
    get,
    responses(
        (status = 200, description = "Organization settings fetched successfully"),
        (status = 404, description = "Organization not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_organization_settings(
    service: web::Data<OrganizationSettingsApplicationService>,
    path: web::Path<(String, String)>,
) -> AppResult<HttpResponse> {
    let (organization_id, setting_key) = path.into_inner();
    info!("Fetching organization settings for {}: {}", organization_id, setting_key);

    let value = service.get_organization_settings(&organization_id, &setting_key).await?;
    Ok(success_response_with_message(
        "Organization settings fetched successfully",
        Some(value),
    ))
}

#[utoipa::path(
    path = "/api/organizations/{organization_id}/settings/{setting_key}",
    put,
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Organization settings updated successfully"),
        (status = 404, description = "Organization not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn update_organization_settings(
    service: web::Data<OrganizationSettingsApplicationService>,
    path: web::Path<(String, String)>,
    settings: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let (organization_id, setting_key) = path.into_inner();
    info!("Updating organization settings for {}: {}", organization_id, setting_key);

    let result = service.update_organization_settings(&organization_id, &setting_key, settings.into_inner()).await?;
    Ok(success_response_with_message("Organization settings updated successfully", Some(result)))
}

pub fn configure_organization_settings_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/organizations/{organization_id}/settings/{setting_key}",
        web::get().to(get_organization_settings),
    )
    .route(
        "/organizations/{organization_id}/settings/{setting_key}",
        web::put().to(update_organization_settings),
    );
}
