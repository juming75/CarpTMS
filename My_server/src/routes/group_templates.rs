//! Group template configuration routes

use actix_web::{web, HttpResponse};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::errors::{success_response_with_message, AppResult};

/// Group template entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTemplate {
    pub template_id: i32,
    pub template_name: String,
    pub description: Option<String>,
    pub industry: String,
    pub config: Option<serde_json::Value>,
    pub create_time: String,
}

/// Create template request
#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub template_name: String,
    pub description: Option<String>,
    pub industry: String,
}

/// Update template request
#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub template_name: Option<String>,
    pub description: Option<String>,
    pub industry: Option<String>,
}

/// Template config request
#[derive(Debug, Deserialize)]
pub struct TemplateConfigRequest {
    pub template_id: i32,
    pub base_config: Option<serde_json::Value>,
    pub team_levels: Option<Vec<serde_json::Value>>,
    pub role_permissions: Option<Vec<serde_json::Value>>,
}

/// Apply template request
#[derive(Debug, Deserialize)]
pub struct ApplyTemplateRequest {
    pub organization_ids: Vec<i32>,
}

/// Get all group templates
pub async fn get_templates(pool: web::Data<PgPool>) -> AppResult<HttpResponse> {
    info!("Fetching group templates");

    let templates = sqlx::query(
        "SELECT template_id, template_name, description, industry, config::text as config, create_time \
         FROM group_templates ORDER BY create_time DESC"
    )
    .fetch_all(pool.get_ref())
    .await?;

    let template_list: Vec<GroupTemplate> = templates
        .into_iter()
        .map(|row| {
            let template_id: i32 = row.get("template_id");
            let template_name: String = row.get("template_name");
            let description: Option<String> = row.get("description");
            let industry: String = row.get("industry");
            let config: Option<String> = row.get("config");
            let create_time: chrono::NaiveDateTime = row.get("create_time");

            GroupTemplate {
                template_id,
                template_name,
                description,
                industry,
                config: config.and_then(|s| serde_json::from_str(&s).ok()),
                create_time: create_time.to_string(),
            }
        })
        .collect();

    Ok(success_response_with_message(
        "Templates fetched successfully",
        Some(serde_json::json!({
            "list": template_list,
            "total": template_list.len()
        })),
    ))
}

/// Get single group template
pub async fn get_template(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let template_id = path.into_inner();
    info!("Fetching group template: {}", template_id);

    let template = sqlx::query(
        "SELECT template_id, template_name, description, industry, config::text as config, create_time \
         FROM group_templates WHERE template_id = $1"
    )
    .bind(template_id)
    .fetch_optional(pool.get_ref())
    .await?;

    match template {
        Some(row) => {
            let template = GroupTemplate {
                template_id: row.get("template_id"),
                template_name: row.get("template_name"),
                description: row.get("description"),
                industry: row.get("industry"),
                config: row
                    .get::<Option<String>, _>("config")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                create_time: row
                    .get::<chrono::NaiveDateTime, _>("create_time")
                    .to_string(),
            };
            Ok(success_response_with_message(
                "Template fetched successfully",
                Some(template),
            ))
        }
        None => Ok(success_response_with_message(
            "Template not found",
            None::<serde_json::Value>,
        )),
    }
}

/// Create new group template
pub async fn create_template(
    pool: web::Data<PgPool>,
    body: web::Json<CreateTemplateRequest>,
) -> AppResult<HttpResponse> {
    info!("Creating new group template: {}", body.template_name);

    let result = sqlx::query(
        "INSERT INTO group_templates (template_name, description, industry) \
         VALUES ($1, $2, $3) \
         RETURNING template_id, template_name, description, industry, create_time",
    )
    .bind(&body.template_name)
    .bind(&body.description)
    .bind(&body.industry)
    .fetch_one(pool.get_ref())
    .await?;

    let template = GroupTemplate {
        template_id: result.get("template_id"),
        template_name: result.get("template_name"),
        description: result.get("description"),
        industry: result.get("industry"),
        config: None,
        create_time: result
            .get::<chrono::NaiveDateTime, _>("create_time")
            .to_string(),
    };

    Ok(success_response_with_message(
        "Template created successfully",
        Some(template),
    ))
}

/// Update group template
pub async fn update_template(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    body: web::Json<UpdateTemplateRequest>,
) -> AppResult<HttpResponse> {
    let template_id = path.into_inner();
    info!("Updating group template: {}", template_id);

    let result = sqlx::query(
        "UPDATE group_templates \
         SET template_name = COALESCE($1, template_name), \
             description = COALESCE($2, description), \
             industry = COALESCE($3, industry) \
         WHERE template_id = $4 \
         RETURNING template_id, template_name, description, industry, create_time",
    )
    .bind(&body.template_name)
    .bind(&body.description)
    .bind(&body.industry)
    .bind(template_id)
    .fetch_optional(pool.get_ref())
    .await?;

    match result {
        Some(row) => {
            let template = GroupTemplate {
                template_id: row.get("template_id"),
                template_name: row.get("template_name"),
                description: row.get("description"),
                industry: row.get("industry"),
                config: None,
                create_time: row
                    .get::<chrono::NaiveDateTime, _>("create_time")
                    .to_string(),
            };
            Ok(success_response_with_message(
                "Template updated successfully",
                Some(template),
            ))
        }
        None => Ok(success_response_with_message(
            "Template not found",
            None::<serde_json::Value>,
        )),
    }
}

/// Delete group template
pub async fn delete_template(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let template_id = path.into_inner();
    info!("Deleting group template: {}", template_id);

    let result = sqlx::query("DELETE FROM group_templates WHERE template_id = $1")
        .bind(template_id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() > 0 {
        Ok(success_response_with_message(
            "Template deleted successfully",
            None::<serde_json::Value>,
        ))
    } else {
        Ok(success_response_with_message(
            "Template not found",
            None::<serde_json::Value>,
        ))
    }
}

/// Update template configuration
pub async fn update_template_config(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    body: web::Json<TemplateConfigRequest>,
) -> AppResult<HttpResponse> {
    let template_id = path.into_inner();
    info!("Updating template config: {}", template_id);

    let config = serde_json::json!({
        "base_config": body.base_config,
        "team_levels": body.team_levels,
        "role_permissions": body.role_permissions
    });

    let result = sqlx::query(
        "UPDATE group_templates \
         SET config = $1::jsonb \
         WHERE template_id = $2 \
         RETURNING template_id, template_name, description, industry, create_time",
    )
    .bind(config.to_string())
    .bind(template_id)
    .fetch_optional(pool.get_ref())
    .await?;

    match result {
        Some(_) => Ok(success_response_with_message(
            "Template config updated successfully",
            None::<serde_json::Value>,
        )),
        None => Ok(success_response_with_message(
            "Template not found",
            None::<serde_json::Value>,
        )),
    }
}

/// Apply template to organizations
pub async fn apply_template_to_orgs(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    body: web::Json<ApplyTemplateRequest>,
) -> AppResult<HttpResponse> {
    let template_id = path.into_inner();
    info!(
        "Applying template {} to {} organizations",
        template_id,
        body.organization_ids.len()
    );

    let mut success_count = 0;
    for org_id in &body.organization_ids {
        let result = sqlx::query(
            "INSERT INTO organization_templates (organization_id, template_id) \
             VALUES ($1, $2) \
             ON CONFLICT (organization_id) DO UPDATE SET template_id = $2",
        )
        .bind(org_id)
        .bind(template_id)
        .execute(pool.get_ref())
        .await;

        if result.is_ok() {
            success_count += 1;
        }
    }

    Ok(success_response_with_message(
        &format!("Template applied to {} organizations", success_count),
        Some(serde_json::json!({
            "success_count": success_count,
            "total": body.organization_ids.len()
        })),
    ))
}

/// Configure group template routes
pub fn configure_group_template_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/group-templates", web::get().to(get_templates))
        .route("/group-templates", web::post().to(create_template))
        .route(
            "/group-templates/{template_id}",
            web::get().to(get_template),
        )
        .route(
            "/group-templates/{template_id}",
            web::put().to(update_template),
        )
        .route(
            "/group-templates/{template_id}",
            web::delete().to(delete_template),
        )
        .route(
            "/group-templates/{template_id}/config",
            web::put().to(update_template_config),
        )
        .route(
            "/group-templates/{template_id}/apply",
            web::post().to(apply_template_to_orgs),
        );
}
