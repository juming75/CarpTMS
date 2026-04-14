//! Organization Settings Application Service
//!
//! Encapsulates all SQL for organization settings management.

use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use utoipa::ToSchema;

use crate::errors::AppResult;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct OrganizationBrandSettings {
    pub company_name: Option<String>,
    pub subtitle: Option<String>,
    pub login_url: Option<String>,
    pub logo: Option<String>,
    pub favicon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct OrganizationThemeSettings {
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub font_family: Option<String>,
    pub layout: Option<String>,
}

pub struct OrganizationSettingsApplicationService {
    pool: PgPool,
}

impl OrganizationSettingsApplicationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn ensure_table(&self) {
        info!("开始检查并创建组织设置表");
        let queries = [
            "CREATE TABLE IF NOT EXISTS organization_settings (
                setting_id SERIAL PRIMARY KEY,
                organization_id VARCHAR(50) NOT NULL,
                setting_key VARCHAR(50) NOT NULL,
                setting_value JSONB NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )",
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_organization_setting_unique ON organization_settings(organization_id, setting_key)",
            "CREATE INDEX IF NOT EXISTS idx_organization_id ON organization_settings(organization_id)",
            "CREATE INDEX IF NOT EXISTS idx_setting_key ON organization_settings(setting_key)",
            r###"INSERT INTO organization_settings (organization_id, setting_key, setting_value)
            VALUES 
                ('root', 'brand', '{"company_name": "CarpTMS", "subtitle": "智慧运输管理系统", "login_url": "/login", "logo": "", "favicon": ""}'),
                ('root', 'theme', '{"primary_color": "#409eff", "secondary_color": "#67C23A", "font_family": "Arial, sans-serif", "layout": "default"}')
            ON CONFLICT (organization_id, setting_key) DO NOTHING"###,
        ];

        for sql in &queries {
            if let Err(e) = sqlx::query(sql).execute(&self.pool).await {
                info!("DDL 执行失败: {:?}", e);
            }
        }
    }

    pub async fn get_organization_settings(
        &self,
        organization_id: &str,
        setting_key: &str,
    ) -> AppResult<serde_json::Value> {
        self.ensure_table().await;

        let row = sqlx::query(
            "SELECT setting_value FROM organization_settings WHERE organization_id = $1 AND setting_key = $2 LIMIT 1",
        )
        .bind(organization_id)
        .bind(setting_key)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let value: serde_json::Value = row.get("setting_value");
                Ok(value)
            }
            None => {
                // Try parent org
                let parent_row = sqlx::query(
                    "SELECT os.setting_value 
                     FROM organizations o 
                     LEFT JOIN organization_settings os ON o.parent_id = os.organization_id 
                     WHERE o.unit_id = $1 AND os.setting_key = $2 LIMIT 1",
                )
                .bind(organization_id)
                .bind(setting_key)
                .fetch_optional(&self.pool)
                .await?;

                if let Some(row) = parent_row {
                    let value: serde_json::Value = row.get("setting_value");
                    return Ok(value);
                }

                // Try root defaults
                let default_row = sqlx::query(
                    "SELECT setting_value FROM organization_settings WHERE organization_id = 'root' AND setting_key = $1 LIMIT 1",
                )
                .bind(setting_key)
                .fetch_optional(&self.pool)
                .await?;

                if let Some(row) = default_row {
                    let value: serde_json::Value = row.get("setting_value");
                    Ok(value)
                } else {
                    Ok(serde_json::json!({}))
                }
            }
        }
    }

    pub async fn update_organization_settings(
        &self,
        organization_id: &str,
        setting_key: &str,
        settings: serde_json::Value,
    ) -> AppResult<serde_json::Value> {
        self.ensure_table().await;

        let _ = sqlx::query(
            "INSERT INTO organization_settings (organization_id, setting_key, setting_value, updated_at)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (organization_id, setting_key)
             DO UPDATE SET setting_value = $3, updated_at = NOW()",
        )
        .bind(organization_id)
        .bind(setting_key)
        .bind(settings)
        .execute(&self.pool)
        .await?;

        Ok(serde_json::json!({"status": "success"}))
    }
}
