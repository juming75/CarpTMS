use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::FromRow;

// ==================== 费用实体 ====================
#[derive(Debug, Clone, FromRow)]
pub struct FinanceCost {
    pub cost_id: i32,
    pub cost_type: String,
    pub amount: f64,
    pub description: String,
    pub cost_date: NaiveDateTime,
    pub create_time: NaiveDateTime,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct FinanceCostCreate {
    pub cost_type: String,
    pub amount: f64,
    pub description: String,
    pub cost_date: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct FinanceCostUpdate {
    pub cost_type: Option<String>,
    pub amount: Option<f64>,
    pub description: Option<String>,
    pub cost_date: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinanceCostQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub cost_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// ==================== 发票实体 ====================
#[derive(Debug, Clone, FromRow)]
pub struct FinanceInvoice {
    pub invoice_id: i32,
    pub invoice_number: String,
    pub amount: f64,
    pub invoice_date: NaiveDateTime,
    pub description: String,
    pub create_time: NaiveDateTime,
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct FinanceInvoiceCreate {
    pub invoice_number: String,
    pub amount: f64,
    pub invoice_date: NaiveDateTime,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct FinanceInvoiceUpdate {
    pub invoice_number: Option<String>,
    pub amount: Option<f64>,
    pub invoice_date: Option<NaiveDateTime>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinanceInvoiceQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub invoice_number: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

// ==================== 统计实体 ====================
#[derive(Debug, Clone)]
pub struct FinanceStatistics {
    pub total_cost: f64,
    pub total_invoice: f64,
    pub balance: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinanceStatisticsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub cost_type: Option<String>,
}
