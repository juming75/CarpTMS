use std::sync::Arc;

use crate::domain::entities::finance::{
    FinanceCost, FinanceCostCreate, FinanceCostQuery, FinanceCostUpdate, FinanceInvoice,
    FinanceInvoiceCreate, FinanceInvoiceQuery, FinanceInvoiceUpdate, FinanceStatistics,
    FinanceStatisticsQuery,
};
use crate::domain::use_cases::finance::FinanceRepository;
use crate::errors::AppResult;

#[async_trait::async_trait]
pub trait FinanceService: Send + Sync {
    // ==================== 费用相关 ====================
    async fn get_finance_costs(
        &self,
        query: FinanceCostQuery,
    ) -> AppResult<(Vec<FinanceCost>, i64)>;
    async fn get_finance_cost(&self, cost_id: i32) -> AppResult<Option<FinanceCost>>;
    async fn create_finance_cost(&self, cost: FinanceCostCreate) -> AppResult<FinanceCost>;
    async fn update_finance_cost(
        &self,
        cost_id: i32,
        cost: FinanceCostUpdate,
    ) -> AppResult<Option<FinanceCost>>;
    async fn delete_finance_cost(&self, cost_id: i32) -> AppResult<bool>;

    // ==================== 发票相关 ====================
    async fn get_finance_invoices(
        &self,
        query: FinanceInvoiceQuery,
    ) -> AppResult<(Vec<FinanceInvoice>, i64)>;
    async fn get_finance_invoice(&self, invoice_id: i32) -> AppResult<Option<FinanceInvoice>>;
    async fn create_finance_invoice(
        &self,
        invoice: FinanceInvoiceCreate,
    ) -> AppResult<FinanceInvoice>;
    async fn update_finance_invoice(
        &self,
        invoice_id: i32,
        invoice: FinanceInvoiceUpdate,
    ) -> AppResult<Option<FinanceInvoice>>;
    async fn delete_finance_invoice(&self, invoice_id: i32) -> AppResult<bool>;

    // ==================== 统计相关 ====================
    async fn get_finance_statistics(
        &self,
        query: FinanceStatisticsQuery,
    ) -> AppResult<FinanceStatistics>;
}

pub struct FinanceServiceImpl {
    finance_repository: Arc<dyn FinanceRepository>,
}

impl FinanceServiceImpl {
    pub fn new(finance_repository: Arc<dyn FinanceRepository>) -> Self {
        Self { finance_repository }
    }
}

#[async_trait::async_trait]
impl FinanceService for FinanceServiceImpl {
    // ==================== 费用相关 ====================
    async fn get_finance_costs(
        &self,
        query: FinanceCostQuery,
    ) -> AppResult<(Vec<FinanceCost>, i64)> {
        self.finance_repository
            .get_finance_costs(query)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to get finance costs", None)
            })
    }

    async fn get_finance_cost(&self, cost_id: i32) -> AppResult<Option<FinanceCost>> {
        self.finance_repository
            .get_finance_cost(cost_id)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to get finance cost", None)
            })
    }

    async fn create_finance_cost(&self, cost: FinanceCostCreate) -> AppResult<FinanceCost> {
        self.finance_repository
            .create_finance_cost(cost)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to create finance cost", None)
            })
    }

    async fn update_finance_cost(
        &self,
        cost_id: i32,
        cost: FinanceCostUpdate,
    ) -> AppResult<Option<FinanceCost>> {
        self.finance_repository
            .update_finance_cost(cost_id, cost)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to update finance cost", None)
            })
    }

    async fn delete_finance_cost(&self, cost_id: i32) -> AppResult<bool> {
        self.finance_repository
            .delete_finance_cost(cost_id)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to delete finance cost", None)
            })
    }

    // ==================== 发票相关 ====================
    async fn get_finance_invoices(
        &self,
        query: FinanceInvoiceQuery,
    ) -> AppResult<(Vec<FinanceInvoice>, i64)> {
        self.finance_repository
            .get_finance_invoices(query)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to get finance invoices", None)
            })
    }

    async fn get_finance_invoice(&self, invoice_id: i32) -> AppResult<Option<FinanceInvoice>> {
        self.finance_repository
            .get_finance_invoice(invoice_id)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to get finance invoice", None)
            })
    }

    async fn create_finance_invoice(
        &self,
        invoice: FinanceInvoiceCreate,
    ) -> AppResult<FinanceInvoice> {
        // 直接调用仓库方法，由仓库处理发票号检查
        self.finance_repository
            .create_finance_invoice(invoice)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to create finance invoice", None)
            })
    }

    async fn update_finance_invoice(
        &self,
        invoice_id: i32,
        invoice: FinanceInvoiceUpdate,
    ) -> AppResult<Option<FinanceInvoice>> {
        // 直接调用仓库方法，由仓库处理发票号检查
        self.finance_repository
            .update_finance_invoice(invoice_id, invoice)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to update finance invoice", None)
            })
    }

    async fn delete_finance_invoice(&self, invoice_id: i32) -> AppResult<bool> {
        self.finance_repository
            .delete_finance_invoice(invoice_id)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to delete finance invoice", None)
            })
    }

    // ==================== 统计相关 ====================
    async fn get_finance_statistics(
        &self,
        query: FinanceStatisticsQuery,
    ) -> AppResult<FinanceStatistics> {
        self.finance_repository
            .get_finance_statistics(query)
            .await
            .map_err(|_| {
                crate::errors::AppError::internal_error("Failed to get finance statistics", None)
            })
    }
}
