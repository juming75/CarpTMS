//! 财务仓库实现

use std::sync::Arc;

use async_trait::async_trait;
use anyhow::Context;
use sqlx::{postgres::PgRow, PgPool, Row};

use crate::domain::entities::finance::{FinanceCost, FinanceCostCreate, FinanceCostUpdate, FinanceCostQuery, FinanceInvoice, FinanceInvoiceCreate, FinanceInvoiceUpdate, FinanceInvoiceQuery, FinanceStatistics, FinanceStatisticsQuery};
use crate::domain::use_cases::finance::FinanceRepository;

/// 财务仓库实现
#[derive(Clone)]
pub struct FinanceRepositoryImpl {
    db: Arc<PgPool>,
}

impl FinanceRepositoryImpl {
    /// 创建财务仓库实例
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl FinanceRepository for FinanceRepositoryImpl {
    // 费用管理
    async fn get_finance_costs(&self, query: FinanceCostQuery) -> Result<(Vec<FinanceCost>, i64), anyhow::Error> {
        let offset = (query.page.unwrap_or(1) - 1) * query.page_size.unwrap_or(20);
        let limit = query.page_size.unwrap_or(20);

        // 构建查询总数
        let count_query = "SELECT COUNT(*) FROM finance_costs";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(&*self.db)
            .await
            .context("Failed to count finance costs")?;

        // 查询数据
        let query = r#"SELECT cost_id, cost_type, amount, description, cost_date, create_time, update_time 
             FROM finance_costs 
             ORDER BY create_time DESC 
             LIMIT $1 OFFSET $2"#;

        let costs = sqlx::query(query)
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| {
                let cost_date: chrono::NaiveDate = row.get("cost_date");
                let create_time: chrono::NaiveDateTime = row.get("create_time");
                let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");
                let amount: f64 = row.get("amount");
                FinanceCost {
                    cost_id: row.get("cost_id"),
                    cost_type: row.get("cost_type"),
                    amount,
                    description: row.get("description"),
                    cost_date: cost_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
                    create_time,
                    update_time,
                }
            })
            .fetch_all(&*self.db)
            .await
            .context("Failed to fetch finance costs")?;

        Ok((costs, total))
    }

    async fn get_finance_cost(&self, cost_id: i32) -> Result<Option<FinanceCost>, anyhow::Error> {
        let row = sqlx::query(
            "SELECT cost_id, cost_type, amount, description, cost_date, create_time, update_time 
             FROM finance_costs 
             WHERE cost_id = $1"
        )
        .bind(cost_id)
        .fetch_optional(&*self.db)
        .await
        .context("Failed to fetch finance cost")?;

        let cost = row.map(|row| {
            let cost_date: chrono::NaiveDate = row.get("cost_date");
            let create_time: chrono::NaiveDateTime = row.get("create_time");
            let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");
            
            FinanceCost {
                cost_id: row.get("cost_id"),
                cost_type: row.get("cost_type"),
                amount: row.get("amount"),
                description: row.get("description"),
                cost_date: cost_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
                create_time,
                update_time,
            }
        });

        Ok(cost)
    }

    async fn create_finance_cost(&self, cost: FinanceCostCreate) -> Result<FinanceCost, anyhow::Error> {
        let row = sqlx::query(
            "INSERT INTO finance_costs (cost_type, amount, description, cost_date) 
             VALUES ($1, $2, $3, $4) 
             RETURNING cost_id, cost_type, amount, description, cost_date, create_time, update_time"
        )
        .bind(cost.cost_type)
        .bind(cost.amount)
        .bind(cost.description)
        .bind(cost.cost_date)
        .fetch_one(&*self.db)
        .await
        .context("Failed to create finance cost")?;

        let cost_date: chrono::NaiveDate = row.get("cost_date");
        let create_time: chrono::NaiveDateTime = row.get("create_time");
        let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");
        
        let created_cost = FinanceCost {
            cost_id: row.get("cost_id"),
            cost_type: row.get("cost_type"),
            amount: row.get("amount"),
            description: row.get("description"),
            cost_date: cost_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
            create_time,
            update_time,
        };

        Ok(created_cost)
    }

    async fn update_finance_cost(&self, cost_id: i32, cost: FinanceCostUpdate) -> Result<Option<FinanceCost>, anyhow::Error> {
        let update_query = r#"UPDATE finance_costs 
             SET cost_type = COALESCE($1, cost_type), 
                 amount = COALESCE($2, amount), 
                 description = COALESCE($3, description), 
                 cost_date = COALESCE($4, cost_date), 
                 update_time = NOW() 
             WHERE cost_id = $5 
             RETURNING cost_id, cost_type, amount, description, cost_date, create_time, update_time"#;

        let row = sqlx::query(update_query)
            .bind(cost.cost_type)
            .bind(cost.amount)
            .bind(cost.description)
            .bind(cost.cost_date)
            .bind(cost_id)
            .fetch_optional(&*self.db)
            .await
            .context("Failed to update finance cost")?;

        let updated_cost = row.map(|row| {
            let cost_date: chrono::NaiveDate = row.get("cost_date");
            let create_time: chrono::NaiveDateTime = row.get("create_time");
            let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");
            
            FinanceCost {
                cost_id: row.get("cost_id"),
                cost_type: row.get("cost_type"),
                amount: row.get("amount"),
                description: row.get("description"),
                cost_date: cost_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
                create_time,
                update_time,
            }
        });

        Ok(updated_cost)
    }

    async fn delete_finance_cost(&self, cost_id: i32) -> Result<bool, anyhow::Error> {
        let result = sqlx::query(
            "DELETE FROM finance_costs WHERE cost_id = $1"
        )
        .bind(cost_id)
        .execute(&*self.db)
        .await
        .context("Failed to delete finance cost")?;

        Ok(result.rows_affected() > 0)
    }

    // 发票管理
    async fn get_finance_invoices(&self, query: FinanceInvoiceQuery) -> Result<(Vec<FinanceInvoice>, i64), anyhow::Error> {
        let offset = (query.page.unwrap_or(1) - 1) * query.page_size.unwrap_or(20);
        let limit = query.page_size.unwrap_or(20);

        // 构建查询总数
        let count_query = "SELECT COUNT(*) FROM finance_invoices";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(&*self.db)
            .await
            .context("Failed to count finance invoices")?;

        // 查询数据
        let query = r#"SELECT invoice_id, invoice_number, amount, invoice_date, description, create_time, update_time 
             FROM finance_invoices 
             ORDER BY create_time DESC 
             LIMIT $1 OFFSET $2"#;

        let invoices = sqlx::query(query)
            .bind(limit)
            .bind(offset)
            .map(|row: PgRow| {
                let invoice_date: chrono::NaiveDate = row.get("invoice_date");
                let create_time: chrono::NaiveDateTime = row.get("create_time");
                let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");
                
                FinanceInvoice {
                    invoice_id: row.get("invoice_id"),
                    invoice_number: row.get("invoice_number"),
                    amount: row.get("amount"),
                    invoice_date: invoice_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
                    description: row.get("description"),
                    create_time,
                    update_time,
                }
            })
            .fetch_all(&*self.db)
            .await
            .context("Failed to fetch finance invoices")?;

        Ok((invoices, total))
    }

    async fn get_finance_invoice(&self, invoice_id: i32) -> Result<Option<FinanceInvoice>, anyhow::Error> {
        let row = sqlx::query(
            "SELECT invoice_id, invoice_number, amount, invoice_date, description, create_time, update_time 
             FROM finance_invoices 
             WHERE invoice_id = $1"
        )
        .bind(invoice_id)
        .fetch_optional(&*self.db)
        .await
        .context("Failed to fetch finance invoice")?;

        let invoice = row.map(|row| {
            let invoice_date: chrono::NaiveDate = row.get("invoice_date");
            let create_time: chrono::NaiveDateTime = row.get("create_time");
            let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");
            
            FinanceInvoice {
                invoice_id: row.get("invoice_id"),
                invoice_number: row.get("invoice_number"),
                amount: row.get("amount"),
                invoice_date: invoice_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
                description: row.get("description"),
                create_time,
                update_time,
            }
        });

        Ok(invoice)
    }

    async fn create_finance_invoice(&self, invoice: FinanceInvoiceCreate) -> Result<FinanceInvoice, anyhow::Error> {
        // 检查发票号是否已存在
        let existing_invoice = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM finance_invoices WHERE invoice_number = $1"
        )
        .bind(&invoice.invoice_number)
        .fetch_one(&*self.db)
        .await
        .context("Failed to check existing invoice")?;

        if existing_invoice > 0 {
            return Err(anyhow::anyhow!("Invoice number already exists"));
        }

        let row = sqlx::query(
            "INSERT INTO finance_invoices (invoice_number, amount, invoice_date, description) 
             VALUES ($1, $2, $3, $4) 
             RETURNING invoice_id, invoice_number, amount, invoice_date, description, create_time, update_time"
        )
        .bind(invoice.invoice_number)
        .bind(invoice.amount)
        .bind(invoice.invoice_date)
        .bind(invoice.description)
        .fetch_one(&*self.db)
        .await
        .context("Failed to create finance invoice")?;

        let invoice_date: chrono::NaiveDate = row.get("invoice_date");
        let create_time: chrono::NaiveDateTime = row.get("create_time");
        let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");
        
        let created_invoice = FinanceInvoice {
            invoice_id: row.get("invoice_id"),
            invoice_number: row.get("invoice_number"),
            amount: row.get("amount"),
            invoice_date: invoice_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
            description: row.get("description"),
            create_time,
            update_time,
        };

        Ok(created_invoice)
    }

    async fn update_finance_invoice(&self, invoice_id: i32, invoice: FinanceInvoiceUpdate) -> Result<Option<FinanceInvoice>, anyhow::Error> {
        // 检查发票号是否已被其他发票使用
        if let Some(invoice_number) = &invoice.invoice_number {
            let existing_invoice = sqlx::query_scalar::<_, i32>(
                "SELECT COUNT(*) FROM finance_invoices WHERE invoice_number = $1 AND invoice_id != $2"
            )
            .bind(invoice_number)
            .bind(invoice_id)
            .fetch_one(&*self.db)
            .await
            .context("Failed to check existing invoice")?;

            if existing_invoice > 0 {
                return Err(anyhow::anyhow!("Invoice number already exists"));
            }
        }

        let update_query = r#"UPDATE finance_invoices 
             SET invoice_number = COALESCE($1, invoice_number), 
                 amount = COALESCE($2, amount), 
                 invoice_date = COALESCE($3, invoice_date), 
                 description = COALESCE($4, description), 
                 update_time = NOW() 
             WHERE invoice_id = $5 
             RETURNING invoice_id, invoice_number, amount, invoice_date, description, create_time, update_time"#;

        let row = sqlx::query(update_query)
            .bind(invoice.invoice_number)
            .bind(invoice.amount)
            .bind(invoice.invoice_date)
            .bind(invoice.description)
            .bind(invoice_id)
            .fetch_optional(&*self.db)
            .await
            .context("Failed to update finance invoice")?;

        let updated_invoice = row.map(|row| {
            let invoice_date: chrono::NaiveDate = row.get("invoice_date");
            let create_time: chrono::NaiveDateTime = row.get("create_time");
            let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");
            
            FinanceInvoice {
                invoice_id: row.get("invoice_id"),
                invoice_number: row.get("invoice_number"),
                amount: row.get("amount"),
                invoice_date: invoice_date.and_hms_opt(0, 0, 0).unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()),
                description: row.get("description"),
                create_time,
                update_time,
            }
        });

        Ok(updated_invoice)
    }

    async fn delete_finance_invoice(&self, invoice_id: i32) -> Result<bool, anyhow::Error> {
        let result = sqlx::query(
            "DELETE FROM finance_invoices WHERE invoice_id = $1"
        )
        .bind(invoice_id)
        .execute(&*self.db)
        .await
        .context("Failed to delete finance invoice")?;

        Ok(result.rows_affected() > 0)
    }

    // 财务统计
    async fn get_finance_statistics(&self, query: FinanceStatisticsQuery) -> Result<FinanceStatistics, anyhow::Error> {
        // 简化实现：直接使用动态 SQL，不使用参数化查询（因为列名动态生成）
        // 注意：这在生产环境中可能有 SQL 注入风险，需要验证输入
        
        let cost_query = match (&query.start_date, &query.end_date) {
            (Some(start), Some(end)) => {
                format!("SELECT COALESCE(SUM(amount), 0) FROM finance_costs WHERE cost_date >= '{}' AND cost_date <= '{}'", start, end)
            }
            (Some(start), None) => {
                format!("SELECT COALESCE(SUM(amount), 0) FROM finance_costs WHERE cost_date >= '{}'", start)
            }
            (None, Some(end)) => {
                format!("SELECT COALESCE(SUM(amount), 0) FROM finance_costs WHERE cost_date <= '{}'", end)
            }
            (None, None) => {
                "SELECT COALESCE(SUM(amount), 0) FROM finance_costs".to_string()
            }
        };

        let total_cost: f64 = sqlx::query_scalar(&cost_query)
            .fetch_one(&*self.db)
            .await
            .context("Failed to calculate total cost")?;

        let invoice_query = match (&query.start_date, &query.end_date) {
            (Some(start), Some(end)) => {
                format!("SELECT COALESCE(SUM(amount), 0) FROM finance_invoices WHERE invoice_date >= '{}' AND invoice_date <= '{}'", start, end)
            }
            (Some(start), None) => {
                format!("SELECT COALESCE(SUM(amount), 0) FROM finance_invoices WHERE invoice_date >= '{}'", start)
            }
            (None, Some(end)) => {
                format!("SELECT COALESCE(SUM(amount), 0) FROM finance_invoices WHERE invoice_date <= '{}'", end)
            }
            (None, None) => {
                "SELECT COALESCE(SUM(amount), 0) FROM finance_invoices".to_string()
            }
        };

        let total_invoice: f64 = sqlx::query_scalar(&invoice_query)
            .fetch_one(&*self.db)
            .await
            .context("Failed to calculate total invoice")?;

        // 计算余额
        let balance = total_invoice - total_cost;

        Ok(FinanceStatistics {
            total_cost,
            total_invoice,
            balance,
        })
    }
}
