//! Queries - 查询处理模块
//!
//! 查询代表对系统的读操作，遵循 CQRS 模式。
//! 每个查询都有对应的处理器负责执行。

pub mod get_order;
pub mod get_vehicle;
pub mod list_orders;
pub mod list_vehicles;
pub mod search_vehicles;

pub use get_order::*;
pub use get_vehicle::*;
pub use list_orders::*;
pub use list_vehicles::*;
pub use search_vehicles::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::AppResult;

/// 查询 trait - 所有查询必须实现此 trait
pub trait Query: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// 查询类型名称
    fn query_type() -> &'static str;
}

/// 查询处理器 trait
#[async_trait]
pub trait QueryHandler<Q: Query, R>: Send + Sync {
    /// 处理查询
    async fn handle(&self, query: Q) -> AppResult<R>;
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResult<T> {
    /// 数据列表
    pub items: Vec<T>,
    /// 总数
    pub total: i64,
    /// 当前页码
    pub page: i32,
    /// 每页大小
    pub page_size: i32,
    /// 总页数
    pub total_pages: i32,
}

impl<T> PagedResult<T> {
    /// 创建分页结果
    pub fn new(items: Vec<T>, total: i64, page: i32, page_size: i32) -> Self {
        let total_pages = if page_size > 0 {
            ((total as f64) / (page_size as f64)).ceil() as i32
        } else {
            0
        };

        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }

    /// 是否有下一页
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    /// 是否有上一页
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
}
