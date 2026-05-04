use std::sync::Arc;

use chrono::Utc;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::domain::entities::openapi_platform::OpenapiPlatform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushPayload {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResult {
    pub platform_id: i32,
    pub platform_name: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub endpoint: String,
    pub method: String,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResult {
    pub success: bool,
    pub status_code: Option<u16>,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct PlatformIntegrationService {
    pool: Arc<PgPool>,
    client: reqwest::Client,
}

impl PlatformIntegrationService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self { pool, client }
    }

    pub async fn get_active_platforms(&self) -> AppResult<Vec<OpenapiPlatform>> {
        let platforms = sqlx::query_as::<_, OpenapiPlatform>(
            "SELECT * FROM openapi_platforms WHERE status = 'active' ORDER BY id",
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch active platforms: {}", e);
            AppError::db_error("Failed to fetch active platforms", Some(&e.to_string()))
        })?;

        Ok(platforms)
    }

    pub async fn push_to_all_platforms(&self, payload: &PushPayload) -> Vec<PushResult> {
        let platforms = match self.get_active_platforms().await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to get active platforms: {}", e);
                return vec![];
            }
        };

        let mut results = Vec::new();
        for platform in platforms {
            let result = self.push_to_platform(&platform, payload).await;
            results.push(result);
        }

        results
    }

    pub async fn push_to_platform(
        &self,
        platform: &OpenapiPlatform,
        payload: &PushPayload,
    ) -> PushResult {
        info!(
            "Pushing {} to platform: {}",
            payload.event_type, platform.name
        );

        let url = format!("{}/api/webhook", platform.url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", platform.api_key))
            .header("Content-Type", "application/json")
            .header("X-CarpTMS-Event", &payload.event_type)
            .header("X-CarpTMS-Timestamp", payload.timestamp.to_rfc3339())
            .json(payload)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if resp.status().is_success() {
                    info!("Push to {} succeeded with status {}", platform.name, status);
                    PushResult {
                        platform_id: platform.id,
                        platform_name: platform.name.clone(),
                        success: true,
                        status_code: Some(status),
                        error: None,
                    }
                } else {
                    let body = resp.text().await.unwrap_or_default();
                    warn!(
                        "Push to {} failed with status {}: {}",
                        platform.name, status, body
                    );
                    PushResult {
                        platform_id: platform.id,
                        platform_name: platform.name.clone(),
                        success: false,
                        status_code: Some(status),
                        error: Some(format!(
                            "HTTP {}: {}",
                            status,
                            body.chars().take(200).collect::<String>()
                        )),
                    }
                }
            }
            Err(e) => {
                error!("Push to {} failed: {}", platform.name, e);
                PushResult {
                    platform_id: platform.id,
                    platform_name: platform.name.clone(),
                    success: false,
                    status_code: None,
                    error: Some(e.to_string()),
                }
            }
        }
    }

    pub async fn pull_from_endpoint(&self, request: &PullRequest) -> PullResult {
        info!("Pulling data from: {} {}", request.method, request.endpoint);

        let method = match request.method.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            _ => reqwest::Method::GET,
        };

        let mut req_builder = self.client.request(method, &request.endpoint);

        if let Some(headers) = &request.headers {
            for (key, value) in headers {
                req_builder = req_builder.header(key.as_str(), value.as_str());
            }
        }

        if let Some(body) = &request.body {
            req_builder = req_builder.json(body);
        }

        let response = req_builder.send().await;

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                let data = serde_json::from_str::<serde_json::Value>(&body).ok();

                if (200..300).contains(&status) {
                    info!("Pull succeeded with status {}", status);
                    PullResult {
                        success: true,
                        status_code: Some(status),
                        data,
                        error: None,
                    }
                } else {
                    warn!("Pull failed with status {}: {}", status, body);
                    PullResult {
                        success: false,
                        status_code: Some(status),
                        data,
                        error: Some(format!("HTTP {}", status)),
                    }
                }
            }
            Err(e) => {
                error!("Pull failed: {}", e);
                PullResult {
                    success: false,
                    status_code: None,
                    data: None,
                    error: Some(e.to_string()),
                }
            }
        }
    }

    pub async fn push_vehicle_location(
        &self,
        vehicle_id: i32,
        latitude: f64,
        longitude: f64,
        speed: Option<f64>,
        heading: Option<f64>,
    ) -> Vec<PushResult> {
        let payload = PushPayload {
            event_type: "vehicle_location".to_string(),
            data: serde_json::json!({
                "vehicle_id": vehicle_id,
                "latitude": latitude,
                "longitude": longitude,
                "speed": speed,
                "heading": heading,
            }),
            timestamp: Utc::now(),
            source: "carptms".to_string(),
        };

        self.push_to_all_platforms(&payload).await
    }

    pub async fn push_alarm(
        &self,
        alarm_type: &str,
        vehicle_id: i32,
        description: &str,
        severity: &str,
    ) -> Vec<PushResult> {
        let payload = PushPayload {
            event_type: "alarm".to_string(),
            data: serde_json::json!({
                "alarm_type": alarm_type,
                "vehicle_id": vehicle_id,
                "description": description,
                "severity": severity,
            }),
            timestamp: Utc::now(),
            source: "carptms".to_string(),
        };

        self.push_to_all_platforms(&payload).await
    }

    pub async fn push_weighing_data(
        &self,
        vehicle_id: i32,
        weight: f64,
        unit: &str,
    ) -> Vec<PushResult> {
        let payload = PushPayload {
            event_type: "weighing_data".to_string(),
            data: serde_json::json!({
                "vehicle_id": vehicle_id,
                "weight": weight,
                "unit": unit,
            }),
            timestamp: Utc::now(),
            source: "carptms".to_string(),
        };

        self.push_to_all_platforms(&payload).await
    }
}

use crate::errors::AppError;
use crate::errors::AppResult;
