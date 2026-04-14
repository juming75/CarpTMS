-- 创建系统设置表
CREATE TABLE IF NOT EXISTS system_settings (
    id SERIAL PRIMARY KEY,
    setting_key VARCHAR(100) NOT NULL UNIQUE,
    setting_value JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 添加注释
COMMENT ON TABLE system_settings IS '系统配置设置表';
COMMENT ON COLUMN system_settings.setting_key IS '设置键名';
COMMENT ON COLUMN system_settings.setting_value IS '设置值（JSON 格式）';
COMMENT ON COLUMN system_settings.description IS '设置描述';

-- 创建索引
CREATE INDEX idx_system_settings_key ON system_settings(setting_key);

-- 插入默认的系统配置
INSERT INTO system_settings (setting_key, setting_value, description) 
VALUES 
    ('system_config', 
     '{"server_url": "http://127.0.0.1:8081", "sync_interval": 5, "auto_sync": true, "home_page_name": "车辆运营监控平台"}'::jsonb,
     '系统基本配置')
ON CONFLICT (setting_key) DO NOTHING;

-- 插入默认的通信配置
INSERT INTO system_settings (setting_key, setting_value, description) 
VALUES 
    ('communication_config', 
     '{"server_ip": "127.0.0.1", "server_port": 8988, "heartbeat_interval": 30, "timeout": 10, "reconnect_count": 3, "protocol": "tcp", "compression": true, "encryption": true}'::jsonb,
     '通信配置')
ON CONFLICT (setting_key) DO NOTHING;


