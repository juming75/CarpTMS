//! 部门领域实体

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 部门实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Department {
    pub department_id: i32,
    pub department_name: String,
    pub parent_department_id: Option<i32>,
    pub parent_department_name: Option<String>,
    pub manager_id: Option<i32>,
    pub manager_name: Option<String>,
    pub phone: Option<String>,
    pub description: Option<String>,
    pub create_time: NaiveDateTime,
    pub update_time: Option<NaiveDateTime>,
}

/// 部门创建请求
#[derive(Debug, Clone, Deserialize)]
pub struct DepartmentCreateRequest {
    pub department_name: String,
    pub parent_department_id: Option<i32>,
    pub manager_id: Option<i32>,
    pub phone: Option<String>,
    pub description: Option<String>,
}

/// 部门更新请求
#[derive(Debug, Clone, Deserialize)]
pub struct DepartmentUpdateRequest {
    pub department_name: Option<String>,
    pub parent_department_id: Option<i32>,
    pub manager_id: Option<i32>,
    pub phone: Option<String>,
    pub description: Option<String>,
}

/// 部门查询参数
#[derive(Debug, Clone, Deserialize)]
pub struct DepartmentQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

// 类型别名，兼容旧代码
pub type DepartmentCreate = DepartmentCreateRequest;
pub type DepartmentUpdate = DepartmentUpdateRequest;

impl Department {
    /// 创建新部门
    pub fn new(
        department_id: i32,
        department_name: String,
        parent_department_id: Option<i32>,
        manager_id: Option<i32>,
        phone: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            department_id,
            department_name,
            parent_department_id,
            parent_department_name: None,
            manager_id,
            manager_name: None,
            phone,
            description,
            create_time: Utc::now().naive_utc(),
            update_time: None,
        }
    }

    /// 更新部门信息
    pub fn update(
        &mut self,
        name: Option<String>,
        parent_id: Option<i32>,
        manager_id: Option<i32>,
        phone: Option<String>,
        description: Option<String>,
    ) {
        if let Some(name) = name {
            self.department_name = name;
        }
        if parent_id.is_some() {
            self.parent_department_id = parent_id;
        }
        if manager_id.is_some() {
            self.manager_id = manager_id;
        }
        if phone.is_some() {
            self.phone = phone;
        }
        if description.is_some() {
            self.description = description;
        }
        self.update_time = Some(Utc::now().naive_utc());
    }

    /// 设置父部门名称
    pub fn set_parent_department_name(&mut self, name: Option<String>) {
        self.parent_department_name = name;
    }

    /// 设置经理名称
    pub fn set_manager_name(&mut self, name: Option<String>) {
        self.manager_name = name;
    }
}
