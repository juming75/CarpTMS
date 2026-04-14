-- 创建统计数据表
CREATE TABLE IF NOT EXISTS statistics (
    id SERIAL PRIMARY KEY,
    stat_type VARCHAR(50) NOT NULL,
    value NUMERIC(18, 6) NOT NULL,
    unit VARCHAR(20) NOT NULL,
    category VARCHAR(50),
    sub_category VARCHAR(50),
    period VARCHAR(20) NOT NULL, -- 日、周、月、年
    period_start TIMESTAMP NOT NULL,
    period_end TIMESTAMP NOT NULL,
    create_time TIMESTAMP NOT NULL DEFAULT NOW(),
    update_time TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_statistics_stat_type ON statistics(stat_type);
CREATE INDEX IF NOT EXISTS idx_statistics_period ON statistics(period);
CREATE INDEX IF NOT EXISTS idx_statistics_period_start ON statistics(period_start);
CREATE INDEX IF NOT EXISTS idx_statistics_period_end ON statistics(period_end);
CREATE INDEX IF NOT EXISTS idx_statistics_category ON statistics(category);
CREATE INDEX IF NOT EXISTS idx_statistics_sub_category ON statistics(sub_category);

-- 创建仪表盘任务表
CREATE TABLE IF NOT EXISTS dashboard_tasks (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    priority VARCHAR(20) NOT NULL, -- high, medium, low
    deadline TIMESTAMP NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, in_progress, completed
    description TEXT,
    create_time TIMESTAMP NOT NULL DEFAULT NOW(),
    update_time TIMESTAMP
);

-- 创建仪表盘动态表
CREATE TABLE IF NOT EXISTS dashboard_dynamics (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    time TIMESTAMP NOT NULL DEFAULT NOW(),
    type VARCHAR(20) NOT NULL, -- success, warning, info, error
    create_time TIMESTAMP NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_dashboard_tasks_status ON dashboard_tasks(status);
CREATE INDEX IF NOT EXISTS idx_dashboard_tasks_priority ON dashboard_tasks(priority);
CREATE INDEX IF NOT EXISTS idx_dashboard_tasks_deadline ON dashboard_tasks(deadline);
CREATE INDEX IF NOT EXISTS idx_dashboard_dynamics_type ON dashboard_dynamics(type);
CREATE INDEX IF NOT EXISTS idx_dashboard_dynamics_time ON dashboard_dynamics(time);

-- 插入一些默认数据
INSERT INTO dashboard_tasks (title, priority, deadline, status, description) VALUES
('处理车辆789012的超速告警', 'high', NOW() + INTERVAL '1 hour', 'pending', '车辆789012在高速公路上超速行驶'),
('审核新订单', 'medium', NOW() + INTERVAL '3 hours', 'pending', '客户ABC公司创建了新的运输订单'),
('更新车辆信息', 'low', NOW() + INTERVAL '1 day', 'pending', '更新车辆345678的保险信息'),
('生成月度报表', 'medium', NOW() + INTERVAL '1 day 6 hours', 'pending', '生成4月份的运输报表'),
('检查设备状态', 'low', NOW() + INTERVAL '2 days', 'pending', '检查所有车辆的GPS设备状态')
ON CONFLICT DO NOTHING;

INSERT INTO dashboard_dynamics (title, description, time, type) VALUES
('车辆123456 完成运输任务', '车辆123456成功完成了从北京到上海的运输任务', NOW() - INTERVAL '2 hours', 'success'),
('车辆789012 发生告警', '车辆789012在高速公路上超速行驶', NOW() - INTERVAL '3 hours', 'warning'),
('新订单创建', '客户ABC公司创建了新的运输订单', NOW() - INTERVAL '4 hours', 'info'),
('车辆345678 开始运输任务', '车辆345678开始执行从广州到深圳的运输任务', NOW() - INTERVAL '5 hours', 'success'),
('系统维护', '系统进行了例行维护，优化了性能', NOW() - INTERVAL '6 hours', 'info')
ON CONFLICT DO NOTHING;