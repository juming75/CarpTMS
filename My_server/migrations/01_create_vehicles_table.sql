-- 先创建车组表，因为vehicles表需要引用它
CREATE TABLE IF NOT EXISTS vehicle_groups (
    group_id SERIAL PRIMARY KEY,
    group_name VARCHAR(100) NOT NULL,
    parent_id INTEGER REFERENCES vehicle_groups(group_id),
    description TEXT,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP
);

-- 创建车辆表
CREATE TABLE IF NOT EXISTS vehicles (
    -- 基本信息
    vehicle_id SERIAL PRIMARY KEY,
    vehicle_name VARCHAR(255) NOT NULL,
    license_plate VARCHAR(50) NOT NULL,
    vehicle_type VARCHAR(100) NOT NULL,
    vehicle_color VARCHAR(50) NOT NULL,
    vehicle_brand VARCHAR(100) NOT NULL,
    vehicle_model VARCHAR(100) NOT NULL,
    engine_no VARCHAR(100) NOT NULL,
    frame_no VARCHAR(100) NOT NULL,
    register_date TIMESTAMP NOT NULL,
    inspection_date TIMESTAMP NOT NULL,
    insurance_date TIMESTAMP NOT NULL,
    seating_capacity INTEGER NOT NULL,
    load_capacity NUMERIC(10, 2) NOT NULL,
    vehicle_length NUMERIC(10, 2) NOT NULL,
    vehicle_width NUMERIC(10, 2) NOT NULL,
    vehicle_height NUMERIC(10, 2) NOT NULL,
    
    -- 终端信息
    device_id VARCHAR(100),
    terminal_type VARCHAR(100),
    communication_type VARCHAR(100),
    sim_card_no VARCHAR(50),
    install_date TIMESTAMP,
    install_address TEXT,
    install_technician VARCHAR(100),
    
    -- 车主信息
    own_no VARCHAR(100),
    own_name VARCHAR(100),
    own_phone VARCHAR(20),
    own_id_card VARCHAR(20),
    own_address TEXT,
    own_email VARCHAR(100),
    
    -- 运营信息
    group_id INTEGER NOT NULL REFERENCES vehicle_groups(group_id),
    operation_status SMALLINT NOT NULL DEFAULT 1,
    operation_route TEXT,
    operation_area TEXT,
    operation_company VARCHAR(200),
    driver_name VARCHAR(100),
    driver_phone VARCHAR(20),
    driver_license_no VARCHAR(50),
    
    -- 财务信息
    purchase_price NUMERIC(12, 2),
    annual_fee NUMERIC(10, 2),
    insurance_fee NUMERIC(10, 2),
    
    -- 模拟数据信息
    is_simulation BOOLEAN NOT NULL DEFAULT FALSE,
    simulation_source VARCHAR(50),
    
    -- 其他信息
    remark TEXT,
    status SMALLINT NOT NULL DEFAULT 1,
    create_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time TIMESTAMP,
    create_user_id INTEGER NOT NULL,
    update_user_id INTEGER,
    
    -- 约束
    UNIQUE(license_plate),
    UNIQUE(engine_no),
    UNIQUE(frame_no)
);

-- 如果表已存在，添加缺少的列
-- 基本信息
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS license_plate VARCHAR(50) NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS vehicle_type VARCHAR(100) NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS vehicle_color VARCHAR(50) NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS vehicle_brand VARCHAR(100) NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS vehicle_model VARCHAR(100) NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS engine_no VARCHAR(100) NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS frame_no VARCHAR(100) NOT NULL DEFAULT '';
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS register_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS inspection_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS insurance_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS seating_capacity INTEGER NOT NULL DEFAULT 0;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS load_capacity NUMERIC(10, 2) NOT NULL DEFAULT 0;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS vehicle_length NUMERIC(10, 2) NOT NULL DEFAULT 0;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS vehicle_width NUMERIC(10, 2) NOT NULL DEFAULT 0;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS vehicle_height NUMERIC(10, 2) NOT NULL DEFAULT 0;

-- 终端信息
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS terminal_type VARCHAR(100);
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS communication_type VARCHAR(100);
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS sim_card_no VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS install_date TIMESTAMP;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS install_address TEXT;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS install_technician VARCHAR(100);

-- 车主信息
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS own_id_card VARCHAR(20);
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS own_address TEXT;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS own_email VARCHAR(100);

-- 运营信息
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS operation_status SMALLINT NOT NULL DEFAULT 1;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS operation_route TEXT;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS operation_area TEXT;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS operation_company VARCHAR(200);
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS driver_name VARCHAR(100);
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS driver_phone VARCHAR(20);
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS driver_license_no VARCHAR(50);

-- 财务信息
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS purchase_price NUMERIC(12, 2);
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS annual_fee NUMERIC(10, 2);
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS insurance_fee NUMERIC(10, 2);

-- 模拟数据信息
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS is_simulation BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS simulation_source VARCHAR(50);

-- 其他信息
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS remark TEXT;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS status SMALLINT NOT NULL DEFAULT 1;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS create_user_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE IF EXISTS vehicles ADD COLUMN IF NOT EXISTS update_user_id INTEGER;


