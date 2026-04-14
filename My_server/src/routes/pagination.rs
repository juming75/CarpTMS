//! / 分页工具模块
// 提供通用的分页参数处理和计算功能

/// 分页参数
#[derive(Debug, Clone, Copy)]
pub struct PaginationParams {
    /// 当前页码（从1开始）
    pub page: i64,
    /// 每页数量
    pub page_size: i64,
    /// 偏移量（数据库查询用）
    pub offset: i64,
}

impl PaginationParams {
    /// 从可选的分页参数创建 PaginationParams
    /// 
    /// # 参数
    /// - `page`: 可选的页码，默认为 1
    /// - `page_size`: 可选的每页数量，默认为 20，范围限制在 1-100
    /// 
    /// # 示例
    /// ```
    /// use crate::routes::pagination::PaginationParams;
    /// 
    /// let params = PaginationParams::new(Some(2), Some(50));
    /// assert_eq!(params.page, 2);
    /// assert_eq!(params.page_size, 50);
    /// assert_eq!(params.offset, 50);
    /// ```
    pub fn new(page: Option<i64>, page_size: Option<i64>) -> Self {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;
        
        Self {
            page,
            page_size,
            offset,
        }
    }

    /// 计算总页数
    /// 
    /// # 参数
    /// - `total`: 总记录数
    /// 
    /// # 返回
    /// 总页数（向上取整）
    pub fn total_pages(&self, total: i64) -> i64 {
        if total <= 0 {
            return 0;
        }
        (total + self.page_size - 1) / self.page_size
    }

    /// 判断是否有下一页
    pub fn has_next_page(&self, total: i64) -> bool {
        self.page < self.total_pages(total)
    }

    /// 判断是否有上一页
    pub fn has_prev_page(&self) -> bool {
        self.page > 1
    }

    /// 获取下一页页码（如果有）
    pub fn next_page(&self, total: i64) -> Option<i64> {
        if self.has_next_page(total) {
            Some(self.page + 1)
        } else {
            None
        }
    }

    /// 获取上一页页码（如果有）
    pub fn prev_page(&self) -> Option<i64> {
        if self.has_prev_page() {
            Some(self.page - 1)
        } else {
            None
        }
    }
}

/// 分页响应元数据
#[derive(Debug, Clone, serde::Serialize)]
pub struct PaginationMeta {
    /// 当前页码
    pub page: i64,
    /// 每页数量
    pub page_size: i64,
    /// 总记录数
    pub total: i64,
    /// 总页数
    pub total_pages: i64,
    /// 是否有下一页
    pub has_next: bool,
    /// 是否有上一页
    pub has_prev: bool,
}

impl PaginationMeta {
    /// 从分页参数和总数创建元数据
    pub fn from_params(params: &PaginationParams, total: i64) -> Self {
        Self {
            page: params.page,
            page_size: params.page_size,
            total,
            total_pages: params.total_pages(total),
            has_next: params.has_next_page(total),
            has_prev: params.has_prev_page(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_default() {
        let params = PaginationParams::new(None, None);
        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 20);
        assert_eq!(params.offset, 0);
    }

    #[test]
    fn test_pagination_params_custom() {
        let params = PaginationParams::new(Some(3), Some(50));
        assert_eq!(params.page, 3);
        assert_eq!(params.page_size, 50);
        assert_eq!(params.offset, 100);
    }

    #[test]
    fn test_pagination_params_bounds() {
        // page 最小为 1
        let params = PaginationParams::new(Some(0), None);
        assert_eq!(params.page, 1);
        
        // page_size 范围 1-100
        let params = PaginationParams::new(None, Some(0));
        assert_eq!(params.page_size, 1);
        
        let params = PaginationParams::new(None, Some(200));
        assert_eq!(params.page_size, 100);
    }

    #[test]
    fn test_total_pages() {
        let params = PaginationParams::new(Some(1), Some(10));
        
        assert_eq!(params.total_pages(0), 0);
        assert_eq!(params.total_pages(1), 1);
        assert_eq!(params.total_pages(10), 1);
        assert_eq!(params.total_pages(11), 2);
        assert_eq!(params.total_pages(100), 10);
    }

    #[test]
    fn test_has_next_prev_page() {
        let params = PaginationParams::new(Some(2), Some(10));
        
        assert!(params.has_prev_page());
        assert!(params.has_next_page(30));
        assert!(!params.has_next_page(20));
    }
}
