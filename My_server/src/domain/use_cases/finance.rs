//! 财务管理领域用例
//! 
//! 包含财务管理的核心业务逻辑，包括费用管理、发票管理和财务统计

use std::sync::Arc;

use anyhow::Result;

use crate::domain::entities::finance::{FinanceCost, FinanceCostCreate, FinanceCostUpdate, FinanceCostQuery, FinanceInvoice, FinanceInvoiceCreate, FinanceInvoiceUpdate, FinanceInvoiceQuery, FinanceStatistics, FinanceStatisticsQuery};

// 财务仓库接口
#[async_trait::async_trait]
pub trait FinanceRepository: Send + Sync {
    // 费用管理
    async fn get_finance_costs(&self, query: FinanceCostQuery) -> Result<(Vec<FinanceCost>, i64), anyhow::Error>;
    async fn get_finance_cost(&self, cost_id: i32) -> Result<Option<FinanceCost>, anyhow::Error>;
    async fn create_finance_cost(&self, cost: FinanceCostCreate) -> Result<FinanceCost, anyhow::Error>;
    async fn update_finance_cost(&self, cost_id: i32, cost: FinanceCostUpdate) -> Result<Option<FinanceCost>, anyhow::Error>;
    async fn delete_finance_cost(&self, cost_id: i32) -> Result<bool, anyhow::Error>;
    
    // 发票管理
    async fn get_finance_invoices(&self, query: FinanceInvoiceQuery) -> Result<(Vec<FinanceInvoice>, i64), anyhow::Error>;
    async fn get_finance_invoice(&self, invoice_id: i32) -> Result<Option<FinanceInvoice>, anyhow::Error>;
    async fn create_finance_invoice(&self, invoice: FinanceInvoiceCreate) -> Result<FinanceInvoice, anyhow::Error>;
    async fn update_finance_invoice(&self, invoice_id: i32, invoice: FinanceInvoiceUpdate) -> Result<Option<FinanceInvoice>, anyhow::Error>;
    async fn delete_finance_invoice(&self, invoice_id: i32) -> Result<bool, anyhow::Error>;
    
    // 财务统计
    async fn get_finance_statistics(&self, query: FinanceStatisticsQuery) -> Result<FinanceStatistics, anyhow::Error>;
}

// 财务用例
pub struct FinanceUseCases {
    repository: Arc<dyn FinanceRepository>,
}

impl FinanceUseCases {
    pub fn new(repository: Arc<dyn FinanceRepository>) -> Self {
        Self {
            repository,
        }
    }
    
    // 费用管理
    pub async fn get_finance_costs(&self, query: FinanceCostQuery) -> Result<(Vec<FinanceCost>, i64)> {
        // 可以在这里添加业务逻辑，如权限检查、数据过滤等
        self.repository.get_finance_costs(query).await
    }
    
    pub async fn get_finance_cost(&self, cost_id: i32) -> Result<Option<FinanceCost>> {
        self.repository.get_finance_cost(cost_id).await
    }
    
    pub async fn create_finance_cost(&self, cost: FinanceCostCreate) -> Result<FinanceCost> {
        // 业务逻辑：验证费用数据
        if cost.amount <= 0.0 {
            return Err(anyhow::anyhow!("Amount must be positive"));
        }
        
        self.repository.create_finance_cost(cost).await
    }
    
    pub async fn update_finance_cost(&self, cost_id: i32, cost: FinanceCostUpdate) -> Result<Option<FinanceCost>> {
        // 业务逻辑：验证费用数据
        if let Some(amount) = cost.amount {
            if amount <= 0.0 {
                return Err(anyhow::anyhow!("Amount must be positive"));
            }
        }
        
        self.repository.update_finance_cost(cost_id, cost).await
    }
    
    pub async fn delete_finance_cost(&self, cost_id: i32) -> Result<bool> {
        self.repository.delete_finance_cost(cost_id).await
    }
    
    // 发票管理
    pub async fn get_finance_invoices(&self, query: FinanceInvoiceQuery) -> Result<(Vec<FinanceInvoice>, i64)> {
        self.repository.get_finance_invoices(query).await
    }
    
    pub async fn get_finance_invoice(&self, invoice_id: i32) -> Result<Option<FinanceInvoice>> {
        self.repository.get_finance_invoice(invoice_id).await
    }
    
    pub async fn create_finance_invoice(&self, invoice: FinanceInvoiceCreate) -> Result<FinanceInvoice> {
        // 业务逻辑：验证发票数据
        if invoice.amount <= 0.0 {
            return Err(anyhow::anyhow!("Amount must be positive"));
        }
        
        if invoice.invoice_number.is_empty() {
            return Err(anyhow::anyhow!("Invoice number is required"));
        }
        
        self.repository.create_finance_invoice(invoice).await
    }
    
    pub async fn update_finance_invoice(&self, invoice_id: i32, invoice: FinanceInvoiceUpdate) -> Result<Option<FinanceInvoice>> {
        // 业务逻辑：验证发票数据
        if let Some(amount) = invoice.amount {
            if amount <= 0.0 {
                return Err(anyhow::anyhow!("Amount must be positive"));
            }
        }
        
        if let Some(invoice_number) = &invoice.invoice_number {
            if invoice_number.is_empty() {
                return Err(anyhow::anyhow!("Invoice number is required"));
            }
        }
        
        self.repository.update_finance_invoice(invoice_id, invoice).await
    }
    
    pub async fn delete_finance_invoice(&self, invoice_id: i32) -> Result<bool> {
        self.repository.delete_finance_invoice(invoice_id).await
    }
    
    // 财务统计
    pub async fn get_finance_statistics(&self, query: FinanceStatisticsQuery) -> Result<FinanceStatistics> {
        self.repository.get_finance_statistics(query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::finance::*;
    use chrono::NaiveDate;
    
    // 模拟财务仓库
    struct MockFinanceRepository;
    
    impl FinanceRepository for MockFinanceRepository {
        async fn get_finance_costs(&self, _query: FinanceCostQuery) -> Result<(Vec<FinanceCost>, i64)> {
            Ok((vec![], 0))
        }
        
        async fn get_finance_cost(&self, _cost_id: i32) -> Result<Option<FinanceCost>> {
            Ok(None)
        }
        
        async fn create_finance_cost(&self, cost: FinanceCostCreate) -> Result<FinanceCost> {
            Ok(FinanceCost {
                cost_id: 1,
                cost_type: cost.cost_type,
                amount: cost.amount,
                description: cost.description,
                cost_date: cost.cost_date,
                create_time: chrono::Utc::now().naive_utc(),
                update_time: None,
            })
        }
        
        async fn update_finance_cost(&self, _cost_id: i32, _cost: FinanceCostUpdate) -> Result<Option<FinanceCost>> {
            Ok(None)
        }
        
        async fn delete_finance_cost(&self, _cost_id: i32) -> Result<bool> {
            Ok(true)
        }
        
        async fn get_finance_invoices(&self, _query: FinanceInvoiceQuery) -> Result<(Vec<FinanceInvoice>, i64)> {
            Ok((vec![], 0))
        }
        
        async fn get_finance_invoice(&self, _invoice_id: i32) -> Result<Option<FinanceInvoice>> {
            Ok(None)
        }
        
        async fn create_finance_invoice(&self, invoice: FinanceInvoiceCreate) -> Result<FinanceInvoice> {
            Ok(FinanceInvoice {
                invoice_id: 1,
                invoice_number: invoice.invoice_number,
                amount: invoice.amount,
                invoice_date: invoice.invoice_date,
                description: invoice.description,
                create_time: chrono::Utc::now().naive_utc(),
                update_time: None,
            })
        }
        
        async fn update_finance_invoice(&self, _invoice_id: i32, _invoice: FinanceInvoiceUpdate) -> Result<Option<FinanceInvoice>> {
            Ok(None)
        }
        
        async fn delete_finance_invoice(&self, _invoice_id: i32) -> Result<bool> {
            Ok(true)
        }
        
        async fn get_finance_statistics(&self, _query: FinanceStatisticsQuery) -> Result<FinanceStatistics> {
            Ok(FinanceStatistics {
                total_cost: 1000.0,
                total_invoice: 1500.0,
                balance: 500.0,
            })
        }
    }
    
    #[tokio::test]
    async fn test_create_finance_cost() {
        let repository = Arc::new(MockFinanceRepository);
        let use_cases = FinanceUseCases::new(repository);
        
        // 测试创建费用
        let cost = FinanceCostCreate {
            cost_type: "测试费用".to_string(),
            amount: 100.0,
            description: "测试描述".to_string(),
            cost_date: NaiveDate::from_ymd(2024, 1, 1),
        };
        
        let result = use_cases.create_finance_cost(cost).await;
        assert!(result.is_ok());
        
        // 测试创建费用失败（金额为负）
        let cost = FinanceCostCreate {
            cost_type: "测试费用".to_string(),
            amount: -100.0,
            description: "测试描述".to_string(),
            cost_date: NaiveDate::from_ymd(2024, 1, 1),
        };
        
        let result = use_cases.create_finance_cost(cost).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_create_finance_invoice() {
        let repository = Arc::new(MockFinanceRepository);
        let use_cases = FinanceUseCases::new(repository);
        
        // 测试创建发票
        let invoice = FinanceInvoiceCreate {
            invoice_number: "INV-001".to_string(),
            amount: 1000.0,
            invoice_date: NaiveDate::from_ymd(2024, 1, 1),
            description: "测试发票".to_string(),
        };
        
        let result = use_cases.create_finance_invoice(invoice).await;
        assert!(result.is_ok());
        
        // 测试创建发票失败（发票号为空）
        let invoice = FinanceInvoiceCreate {
            invoice_number: "".to_string(),
            amount: 1000.0,
            invoice_date: NaiveDate::from_ymd(2024, 1, 1),
            description: "测试发票".to_string(),
        };
        
        let result = use_cases.create_finance_invoice(invoice).await;
        assert!(result.is_err());
    }
}
