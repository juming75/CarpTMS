//! Application Layer - 应用层
//!
//! 应用层是 DDD 架构中的重要组成部分，负责协调领域对象和基础设施服务。
//! 该层包含：
//! - Commands: 命令处理（写操作）
//! - Queries: 查询处理（读操作）
//! - Services: 应用服务
//! - DTOs: 数据传输对象

pub mod commands;
pub mod dto;
pub mod queries;
pub mod services;

#[cfg(test)]
mod application_tests;

pub use commands::*;
pub use dto::*;
pub use queries::*;
pub use services::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::AppResult;

/// 应用服务 trait - 定义应用层服务的通用接口
#[async_trait]
pub trait ApplicationService: Send + Sync {
    /// 服务名称
    fn name(&self) -> &str;

    /// 初始化服务
    async fn initialize(&self) -> AppResult<()>;
}

/// 命令处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult<T> {
    /// 是否成功
    pub success: bool,
    /// 结果数据
    pub data: Option<T>,
    /// 错误信息
    pub error: Option<String>,
}

impl<T> CommandResult<T> {
    /// 创建成功结果
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// 创建成功结果（无数据）
    pub fn success_empty() -> Self
    where
        T: Default,
    {
        Self {
            success: true,
            data: Some(T::default()),
            error: None,
        }
    }

    /// 创建错误结果
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
        }
    }
}

/// 查询处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    /// 是否成功
    pub success: bool,
    /// 结果数据
    pub data: Option<T>,
    /// 总数（用于分页）
    pub total: Option<i64>,
    /// 错误信息
    pub error: Option<String>,
}

impl<T> QueryResult<T> {
    /// 创建成功结果
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            total: None,
            error: None,
        }
    }

    /// 创建分页结果
    pub fn success_with_total(data: T, total: i64) -> Self {
        Self {
            success: true,
            data: Some(data),
            total: Some(total),
            error: None,
        }
    }

    /// 创建错误结果
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            total: None,
            error: Some(error.into()),
        }
    }
}

/// 分页参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// 当前页码
    pub page: i32,
    /// 每页大小
    pub page_size: i32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

impl Pagination {
    /// 创建分页参数
    pub fn new(page: i32, page_size: i32) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.clamp(1, 100),
        }
    }

    /// 计算偏移量
    pub fn offset(&self) -> i32 {
        (self.page - 1) * self.page_size
    }
}

/// 排序参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sorting {
    /// 排序字段
    pub field: String,
    /// 是否升序
    pub ascending: bool,
}

impl Default for Sorting {
    fn default() -> Self {
        Self {
            field: "id".to_string(),
            ascending: false,
        }
    }
}
