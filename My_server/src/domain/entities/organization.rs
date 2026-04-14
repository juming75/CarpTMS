//! 组织单位领域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// 组织单位实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organization {
    pub unit_id: String,
    pub name: String,
    #[sqlx(rename = "type")]
    pub r#type: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub status: String,
    pub create_time: DateTime<Utc>,
    pub update_time: Option<DateTime<Utc>>,
}

/// 组织单位创建请求
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct OrganizationCreateRequest {
    #[validate(length(min = 1, max = 50))]
    pub unit_id: String,
    #[validate(length(min = 2, max = 50))]
    pub name: String,
    pub r#type: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub status: Option<String>,
}

/// 组织单位更新请求
#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationUpdateRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
}

/// 组织单位查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub name: Option<String>,
    pub r#type: Option<String>,
}

// 类型别名，兼容旧代码
pub type OrganizationCreate = OrganizationCreateRequest;
pub type OrganizationUpdate = OrganizationUpdateRequest;

impl Organization {
    /// 创建新组织单位
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        unit_id: String,
        name: String,
        r#type: String,
        parent_id: Option<i32>,
        description: Option<String>,
        contact_person: Option<String>,
        contact_phone: Option<String>,
        status: String,
    ) -> Self {
        Self {
            unit_id,
            name,
            r#type,
            parent_id,
            description,
            contact_person,
            contact_phone,
            status,
            create_time: Utc::now(),
            update_time: None,
        }
    }

    /// 更新组织单位信息
    pub fn update(
        &mut self,
        name: Option<String>,
        r#type: Option<String>,
        parent_id: Option<i32>,
        description: Option<String>,
        contact_person: Option<String>,
        contact_phone: Option<String>,
    ) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(r#type) = r#type {
            self.r#type = r#type;
        }
        if parent_id.is_some() {
            self.parent_id = parent_id;
        }
        if description.is_some() {
            self.description = description;
        }
        if contact_person.is_some() {
            self.contact_person = contact_person;
        }
        if contact_phone.is_some() {
            self.contact_phone = contact_phone;
        }
        self.update_time = Some(Utc::now());
    }

    /// 更新状态
    pub fn update_status(&mut self, status: String) {
        self.status = status;
        self.update_time = Some(Utc::now());
    }

    /// 检查是否活跃
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}
