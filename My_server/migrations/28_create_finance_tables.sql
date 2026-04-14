-- 创建财务费用表
CREATE TABLE IF NOT EXISTS finance_costs (
    cost_id SERIAL PRIMARY KEY,
    cost_type VARCHAR(100) NOT NULL,
    amount DECIMAL(10, 2) NOT NULL,
    description TEXT,
    cost_date DATE NOT NULL,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

-- 创建财务发票表
CREATE TABLE IF NOT EXISTS finance_invoices (
    invoice_id SERIAL PRIMARY KEY,
    invoice_number VARCHAR(100) NOT NULL UNIQUE,
    amount DECIMAL(10, 2) NOT NULL,
    invoice_date DATE NOT NULL,
    description TEXT,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

-- 为财务表添加索引
CREATE INDEX IF NOT EXISTS idx_finance_costs_cost_date ON finance_costs(cost_date);
CREATE INDEX IF NOT EXISTS idx_finance_invoices_invoice_date ON finance_invoices(invoice_date);
