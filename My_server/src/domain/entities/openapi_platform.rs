//! OpenAPI 平台领域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// OpenAPI 平台实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OpenapiPlatform {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// OpenAPI 平台创建请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct OpenapiPlatformCreateRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub url: String,
    #[validate(length(min = 1, max = 255))]
    pub api_key: String,
    pub status: Option<String>,
}

/// OpenAPI 平台更新请求
#[derive(Debug, Clone, Deserialize)]
pub struct OpenapiPlatformUpdateRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub api_key: Option<String>,
    pub status: Option<String>,
}

/// OpenAPI 平台查询参数
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct OpenapiPlatformQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub name: Option<String>,
    pub status: Option<String>,
}

impl OpenapiPlatform {
    /// 创建新 OpenAPI 平台
    pub fn new(name: String, url: String, api_key: String, status: String) -> Self {
        Self {
            id: 0,
            name,
            url,
            api_key,
            status,
            created_at: Utc::now(),
            updated_at: None,
        }
    }

    /// 更新 OpenAPI 平台信息
    pub fn update(
        &mut self,
        name: Option<String>,
        url: Option<String>,
        api_key: Option<String>,
        status: Option<String>,
    ) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(url) = url {
            self.url = url;
        }
        if let Some(api_key) = api_key {
            self.api_key = api_key;
        }
        if let Some(status) = status {
            self.status = status;
        }
        self.updated_at = Some(Utc::now());
    }

    /// 检查是否活跃
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}
