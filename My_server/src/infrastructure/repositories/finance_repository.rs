//! 财务仓库实现

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::entities::finance::{
    FinanceCost, FinanceCostCreate, FinanceCostQuery, FinanceCostUpdate, FinanceInvoice,
    FinanceInvoiceCreate, FinanceInvoiceQuery, FinanceInvoiceUpdate, FinanceStatistics,
    FinanceStatisticsQuery,
};
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

    /// 从行解析费用记录
    fn parse_cost_row(row: sqlx::postgres::PgRow) -> Result<FinanceCost, anyhow::Error> {
        let cost_id: i32 = row.get("cost_id");
        let cost_type: String = row.get("cost_type");
        let amount: f64 = row.get("amount");
        let description: Option<String> = row.get("description");
        let cost_date: chrono::NaiveDateTime = row.get("cost_date");
        let create_time: chrono::NaiveDateTime = row.get("create_time");
        let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");

        Ok(FinanceCost {
            cost_id,
            cost_type,
            amount,
            description: description.unwrap_or_default(),
            cost_date,
            create_time,
            update_time,
        })
    }

    /// 从行解析发票记录
    fn parse_invoice_row(row: sqlx::postgres::PgRow) -> Result<FinanceInvoice, anyhow::Error> {
        let invoice_id: i32 = row.get("invoice_id");
        let invoice_number: String = row.get("invoice_number");
        let amount: f64 = row.get("amount");
        let invoice_date: chrono::NaiveDateTime = row.get("invoice_date");
        let description: Option<String> = row.get("description");
        let create_time: chrono::NaiveDateTime = row.get("create_time");
        let update_time: Option<chrono::NaiveDateTime> = row.get("update_time");

        Ok(FinanceInvoice {
            invoice_id,
            invoice_number,
            amount,
            invoice_date,
            description: description.unwrap_or_default(),
            create_time,
            update_time,
        })
    }
}

#[async_trait]
impl FinanceRepository for FinanceRepositoryImpl {
    // 费用管理
    async fn get_finance_costs(
        &self,
        query: FinanceCostQuery,
    ) -> Result<(Vec<FinanceCost>, i64), anyhow::Error> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        let mut conditions: Vec<String> = Vec::new();
        let mut string_params: Vec<String> = Vec::new();

        if let Some(ref cost_type) = query.cost_type {
            if !cost_type.is_empty() {
                let idx = string_params.len() + 1;
                conditions.push(format!("cost_type = ${}", idx));
                string_params.push(cost_type.clone());
            }
        }

        if let Some(ref start_date) = query.start_date {
            if !start_date.is_empty() {
                let idx = string_params.len() + 1;
                conditions.push(format!("cost_date >= ${}", idx));
                string_params.push(start_date.clone());
            }
        }

        if let Some(ref end_date) = query.end_date {
            if !end_date.is_empty() {
                let idx = string_params.len() + 1;
                conditions.push(format!("cost_date <= ${}", idx));
                string_params.push(end_date.clone());
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM finance_costs {}", where_clause);
        let total: i64 = {
            let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
            for p in &string_params {
                cq = cq.bind(p);
            }
            cq.fetch_one(&*self.db).await?
        };

        let data_sql = format!(
            "SELECT cost_id, cost_type, amount, description, cost_date, create_time, update_time 
             FROM finance_costs {} ORDER BY create_time DESC LIMIT ${} OFFSET ${}",
            where_clause,
            string_params.len() + 1,
            string_params.len() + 2,
        );

        let costs = {
            let mut q = sqlx::query(&data_sql);
            for p in &string_params {
                q = q.bind(p);
            }
            q = q.bind(page_size).bind(offset);
            let rows = q.fetch_all(&*self.db).await?;
            rows.into_iter()
                .map(Self::parse_cost_row)
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok((costs, total))
    }

    async fn get_finance_cost(&self, cost_id: i32) -> Result<Option<FinanceCost>, anyhow::Error> {
        let row = sqlx::query(
            "SELECT cost_id, cost_type, amount, description, cost_date, create_time, update_time 
             FROM finance_costs WHERE cost_id = $1",
        )
        .bind(cost_id)
        .fetch_optional(&*self.db)
        .await?;

        let cost = row.map(Self::parse_cost_row).transpose()?;
        Ok(cost)
    }

    async fn create_finance_cost(
        &self,
        cost: FinanceCostCreate,
    ) -> Result<FinanceCost, anyhow::Error> {
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
        .await?;

        Self::parse_cost_row(row)
    }

    async fn update_finance_cost(
        &self,
        cost_id: i32,
        cost: FinanceCostUpdate,
    ) -> Result<Option<FinanceCost>, anyhow::Error> {
        let row = sqlx::query(
            r#"UPDATE finance_costs 
             SET cost_type = COALESCE($1, cost_type), 
                 amount = COALESCE($2, amount), 
                 description = COALESCE($3, description), 
                 cost_date = COALESCE($4, cost_date), 
                 update_time = NOW() 
             WHERE cost_id = $5 
             RETURNING cost_id, cost_type, amount, description, cost_date, create_time, update_time"#
        )
        .bind(cost.cost_type)
        .bind(cost.amount)
        .bind(cost.description)
        .bind(cost.cost_date)
        .bind(cost_id)
        .fetch_optional(&*self.db)
        .await?;

        let updated = row.map(Self::parse_cost_row).transpose()?;
        Ok(updated)
    }

    async fn delete_finance_cost(&self, cost_id: i32) -> Result<bool, anyhow::Error> {
        let result = sqlx::query("DELETE FROM finance_costs WHERE cost_id = $1")
            .bind(cost_id)
            .execute(&*self.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // 发票管理
    async fn get_finance_invoices(
        &self,
        query: FinanceInvoiceQuery,
    ) -> Result<(Vec<FinanceInvoice>, i64), anyhow::Error> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        let mut conditions: Vec<String> = Vec::new();
        let mut string_params: Vec<String> = Vec::new();

        if let Some(ref invoice_number) = query.invoice_number {
            if !invoice_number.is_empty() {
                let idx = string_params.len() + 1;
                conditions.push(format!("invoice_number ILIKE ${}", idx));
                string_params.push(format!("%{}%", invoice_number));
            }
        }

        if let Some(ref start_date) = query.start_date {
            if !start_date.is_empty() {
                let idx = string_params.len() + 1;
                conditions.push(format!("invoice_date >= ${}", idx));
                string_params.push(start_date.clone());
            }
        }

        if let Some(ref end_date) = query.end_date {
            if !end_date.is_empty() {
                let idx = string_params.len() + 1;
                conditions.push(format!("invoice_date <= ${}", idx));
                string_params.push(end_date.clone());
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM finance_invoices {}", where_clause);
        let total: i64 = {
            let mut cq = sqlx::query_scalar::<_, i64>(&count_sql);
            for p in &string_params {
                cq = cq.bind(p);
            }
            cq.fetch_one(&*self.db).await?
        };

        let data_sql = format!(
            "SELECT invoice_id, invoice_number, amount, invoice_date, description, create_time, update_time 
             FROM finance_invoices {} ORDER BY create_time DESC LIMIT ${} OFFSET ${}",
            where_clause,
            string_params.len() + 1,
            string_params.len() + 2,
        );

        let invoices = {
            let mut q = sqlx::query(&data_sql);
            for p in &string_params {
                q = q.bind(p);
            }
            q = q.bind(page_size).bind(offset);
            let rows = q.fetch_all(&*self.db).await?;
            rows.into_iter()
                .map(Self::parse_invoice_row)
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok((invoices, total))
    }

    async fn get_finance_invoice(
        &self,
        invoice_id: i32,
    ) -> Result<Option<FinanceInvoice>, anyhow::Error> {
        let row = sqlx::query(
            "SELECT invoice_id, invoice_number, amount, invoice_date, description, create_time, update_time 
             FROM finance_invoices WHERE invoice_id = $1"
        )
        .bind(invoice_id)
        .fetch_optional(&*self.db)
        .await?;

        let invoice = row.map(Self::parse_invoice_row).transpose()?;
        Ok(invoice)
    }

    async fn create_finance_invoice(
        &self,
        invoice: FinanceInvoiceCreate,
    ) -> Result<FinanceInvoice, anyhow::Error> {
        // 检查发票号是否已存在
        let existing_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM finance_invoices WHERE invoice_number = $1")
                .bind(&invoice.invoice_number)
                .fetch_one(&*self.db)
                .await?;

        if existing_count > 0 {
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
        .await?;

        Self::parse_invoice_row(row)
    }

    async fn update_finance_invoice(
        &self,
        invoice_id: i32,
        invoice: FinanceInvoiceUpdate,
    ) -> Result<Option<FinanceInvoice>, anyhow::Error> {
        // 检查发票号是否已被其他发票使用
        if let Some(ref invoice_number) = invoice.invoice_number {
            let existing_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM finance_invoices WHERE invoice_number = $1 AND invoice_id != $2"
            )
            .bind(invoice_number)
            .bind(invoice_id)
            .fetch_one(&*self.db)
            .await?;

            if existing_count > 0 {
                return Err(anyhow::anyhow!("Invoice number already exists"));
            }
        }

        let row = sqlx::query(
            r#"UPDATE finance_invoices 
             SET invoice_number = COALESCE($1, invoice_number), 
                 amount = COALESCE($2, amount), 
                 invoice_date = COALESCE($3, invoice_date), 
                 description = COALESCE($4, description), 
                 update_time = NOW() 
             WHERE invoice_id = $5 
             RETURNING invoice_id, invoice_number, amount, invoice_date, description, create_time, update_time"#
        )
        .bind(invoice.invoice_number)
        .bind(invoice.amount)
        .bind(invoice.invoice_date)
        .bind(invoice.description)
        .bind(invoice_id)
        .fetch_optional(&*self.db)
        .await?;

        let updated = row.map(Self::parse_invoice_row).transpose()?;
        Ok(updated)
    }

    async fn delete_finance_invoice(&self, invoice_id: i32) -> Result<bool, anyhow::Error> {
        let result = sqlx::query("DELETE FROM finance_invoices WHERE invoice_id = $1")
            .bind(invoice_id)
            .execute(&*self.db)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // 财务统计
    async fn get_finance_statistics(
        &self,
        query: FinanceStatisticsQuery,
    ) -> Result<FinanceStatistics, anyhow::Error> {
        let mut cost_conditions: Vec<String> = Vec::new();
        let mut cost_params: Vec<String> = Vec::new();

        if let Some(ref start_date) = query.start_date {
            if !start_date.is_empty() {
                let idx = cost_params.len() + 1;
                cost_conditions.push(format!("cost_date >= ${}", idx));
                cost_params.push(start_date.clone());
            }
        }
        if let Some(ref end_date) = query.end_date {
            if !end_date.is_empty() {
                let idx = cost_params.len() + 1;
                cost_conditions.push(format!("cost_date <= ${}", idx));
                cost_params.push(end_date.clone());
            }
        }
        if let Some(ref cost_type) = query.cost_type {
            if !cost_type.is_empty() {
                let idx = cost_params.len() + 1;
                cost_conditions.push(format!("cost_type = ${}", idx));
                cost_params.push(cost_type.clone());
            }
        }

        let cost_where = if cost_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", cost_conditions.join(" AND "))
        };

        let cost_sql = format!(
            "SELECT COALESCE(SUM(amount), 0) FROM finance_costs {}",
            cost_where
        );
        let mut cost_q = sqlx::query_scalar::<_, f64>(&cost_sql);
        for p in &cost_params {
            cost_q = cost_q.bind(p);
        }
        let total_cost: f64 = cost_q.fetch_one(&*self.db).await?;

        // 发票统计（不区分费用类型）
        let mut inv_conditions: Vec<String> = Vec::new();
        let mut inv_params: Vec<String> = Vec::new();

        if let Some(ref start_date) = query.start_date {
            if !start_date.is_empty() {
                let idx = inv_params.len() + 1;
                inv_conditions.push(format!("invoice_date >= ${}", idx));
                inv_params.push(start_date.clone());
            }
        }
        if let Some(ref end_date) = query.end_date {
            if !end_date.is_empty() {
                let idx = inv_params.len() + 1;
                inv_conditions.push(format!("invoice_date <= ${}", idx));
                inv_params.push(end_date.clone());
            }
        }

        let inv_where = if inv_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", inv_conditions.join(" AND "))
        };

        let invoice_sql = format!(
            "SELECT COALESCE(SUM(amount), 0) FROM finance_invoices {}",
            inv_where
        );
        let mut inv_q = sqlx::query_scalar::<_, f64>(&invoice_sql);
        for p in &inv_params {
            inv_q = inv_q.bind(p);
        }
        let total_invoice: f64 = inv_q.fetch_one(&*self.db).await?;

        let balance = total_invoice - total_cost;

        Ok(FinanceStatistics {
            total_cost,
            total_invoice,
            balance,
        })
    }
}
