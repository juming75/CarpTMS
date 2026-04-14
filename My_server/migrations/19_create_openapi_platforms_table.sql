-- 创建 OpenAPI 平台表
CREATE TABLE IF NOT EXISTS openapi_platforms (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    url VARCHAR(255) NOT NULL,
    api_key VARCHAR(255) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_openapi_platforms_name ON openapi_platforms(name);
CREATE INDEX IF NOT EXISTS idx_openapi_platforms_status ON openapi_platforms(status);

-- 插入默认数据
INSERT INTO openapi_platforms (name, url, api_key, status) 
VALUES 
    ('Default Platform', 'https://api.example.com', 'default-api-key', 'active')
ON CONFLICT DO NOTHING;
