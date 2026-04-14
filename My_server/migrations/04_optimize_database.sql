-- Database optimization script
-- Fixes redundant columns, optimizes data types, and adds missing indexes

-- 1. Fix the users table
-- Remove redundant group_id column
ALTER TABLE IF EXISTS users DROP COLUMN IF EXISTS group_id;

-- Add missing status column
ALTER TABLE IF EXISTS users ADD COLUMN IF NOT EXISTS status SMALLINT NOT NULL DEFAULT 1;

-- Optimize email column to use VARCHAR instead of TEXT
ALTER TABLE IF EXISTS users ALTER COLUMN email TYPE VARCHAR(100);

-- 2. Optimize status columns in vehicles table
ALTER TABLE IF EXISTS vehicles ALTER COLUMN status TYPE SMALLINT;
ALTER TABLE IF EXISTS vehicles ALTER COLUMN operation_status TYPE SMALLINT;

-- 3. Fix foreign key references in report_templates table
-- Change reference from users(id) to users(user_id)
DO $$
BEGIN
    -- Check if the constraint exists
    IF EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'report_templates_create_user_id_fkey'
    ) THEN
        -- Drop the existing constraint
        ALTER TABLE report_templates DROP CONSTRAINT report_templates_create_user_id_fkey;
        -- Add the corrected constraint
        ALTER TABLE report_templates 
        ADD CONSTRAINT report_templates_create_user_id_fkey 
        FOREIGN KEY (create_user_id) REFERENCES users(user_id);
    END IF;
END $$;

-- 4. Fix foreign key references in audit_logs table
-- Change reference from users(id) to users(user_id)
DO $$
BEGIN
    -- Check if the constraint exists
    IF EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'audit_logs_user_id_fkey'
    ) THEN
        -- Drop the existing constraint
        ALTER TABLE audit_logs DROP CONSTRAINT audit_logs_user_id_fkey;
        -- Add the corrected constraint
        ALTER TABLE audit_logs 
        ADD CONSTRAINT audit_logs_user_id_fkey 
        FOREIGN KEY (user_id) REFERENCES users(user_id);
    END IF;
END $$;

-- 5. Add composite indexes for common queries
-- Composite index on weighing_data for vehicle_id and weighing_time
CREATE INDEX IF NOT EXISTS idx_weighing_data_vehicle_time 
ON weighing_data(vehicle_id, weighing_time DESC);

-- Composite index on vehicles for group_id and status
CREATE INDEX IF NOT EXISTS idx_vehicles_group_status 
ON vehicles(group_id, status);

-- Composite index on devices for device_type and status
CREATE INDEX IF NOT EXISTS idx_devices_type_status 
ON devices(device_type, status);

-- 6. Optimize device_data table (add indexes)
CREATE INDEX IF NOT EXISTS idx_device_data_device_id 
ON device_data(device_id);

CREATE INDEX IF NOT EXISTS idx_device_data_timestamp 
ON device_data("timestamp" DESC);

CREATE INDEX IF NOT EXISTS idx_device_data_command 
ON device_data(command);

-- 7. Add NOT NULL constraints for required fields
-- In user_groups table, group_name should be NOT NULL (already defined)

-- In vehicle_groups table, group_name should be NOT NULL (already defined)

-- In devices table, ensure required fields are NOT NULL
ALTER TABLE IF EXISTS devices ALTER COLUMN device_name SET NOT NULL;
ALTER TABLE IF EXISTS devices ALTER COLUMN device_type SET NOT NULL;
ALTER TABLE IF EXISTS devices ALTER COLUMN device_model SET NOT NULL;
ALTER TABLE IF EXISTS devices ALTER COLUMN manufacturer SET NOT NULL;
ALTER TABLE IF EXISTS devices ALTER COLUMN serial_number SET NOT NULL;
ALTER TABLE IF EXISTS devices ALTER COLUMN communication_type SET NOT NULL;

-- 8. Optimize weighing_data table for better performance
-- Add a composite index for frequent queries by device_id and time range
CREATE INDEX IF NOT EXISTS idx_weighing_data_device_time 
ON weighing_data(device_id, weighing_time DESC);

-- 9. Optimize report_data table with a composite index
CREATE INDEX IF NOT EXISTS idx_report_data_period 
ON report_data(period_type, period_value, report_time DESC);

-- 10. Add a unique constraint to prevent duplicate user names
ALTER TABLE IF EXISTS users ADD CONSTRAINT users_user_name_unique UNIQUE (user_name) DEFERRABLE INITIALLY DEFERRED;

-- 11. Add a unique constraint to prevent duplicate license plates
ALTER TABLE IF EXISTS vehicles ADD CONSTRAINT vehicles_license_plate_unique UNIQUE (license_plate) DEFERRABLE INITIALLY DEFERRED;

-- 12. Add a unique constraint to prevent duplicate device ids
ALTER TABLE IF EXISTS devices ADD CONSTRAINT devices_device_id_unique UNIQUE (device_id) DEFERRABLE INITIALLY DEFERRED;

-- 13. Optimize varchar lengths for better storage
-- Ensure consistent varchar lengths across tables
ALTER TABLE IF EXISTS users ALTER COLUMN user_name TYPE VARCHAR(50);
ALTER TABLE IF EXISTS users ALTER COLUMN password TYPE VARCHAR(100);
ALTER TABLE IF EXISTS users ALTER COLUMN phone TYPE VARCHAR(20);

ALTER TABLE IF EXISTS vehicles ALTER COLUMN vehicle_name TYPE VARCHAR(100);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN license_plate TYPE VARCHAR(20);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN vehicle_type TYPE VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN vehicle_color TYPE VARCHAR(20);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN vehicle_brand TYPE VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN vehicle_model TYPE VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN engine_no TYPE VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN frame_no TYPE VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN device_id TYPE VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN terminal_type TYPE VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN communication_type TYPE VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN sim_card_no TYPE VARCHAR(20);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN own_no TYPE VARCHAR(50);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN own_name TYPE VARCHAR(100);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN own_phone TYPE VARCHAR(20);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN own_id_card TYPE VARCHAR(20);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN own_email TYPE VARCHAR(100);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN driver_name TYPE VARCHAR(100);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN driver_phone TYPE VARCHAR(20);
ALTER TABLE IF EXISTS vehicles ALTER COLUMN driver_license_no TYPE VARCHAR(20);

ALTER TABLE IF EXISTS devices ALTER COLUMN device_id TYPE VARCHAR(50);
ALTER TABLE IF EXISTS devices ALTER COLUMN device_name TYPE VARCHAR(100);
ALTER TABLE IF EXISTS devices ALTER COLUMN device_type TYPE VARCHAR(50);
ALTER TABLE IF EXISTS devices ALTER COLUMN device_model TYPE VARCHAR(50);
ALTER TABLE IF EXISTS devices ALTER COLUMN manufacturer TYPE VARCHAR(100);
ALTER TABLE IF EXISTS devices ALTER COLUMN serial_number TYPE VARCHAR(100);
ALTER TABLE IF EXISTS devices ALTER COLUMN communication_type TYPE VARCHAR(50);
ALTER TABLE IF EXISTS devices ALTER COLUMN sim_card_no TYPE VARCHAR(20);
ALTER TABLE IF EXISTS devices ALTER COLUMN ip_address TYPE VARCHAR(50);
ALTER TABLE IF EXISTS devices ALTER COLUMN mac_address TYPE VARCHAR(50);

-- 14. Add a comment to important tables for documentation
COMMENT ON TABLE users IS '系统用户表';
COMMENT ON TABLE user_groups IS '用户组表';
COMMENT ON TABLE vehicles IS '车辆信息表';
COMMENT ON TABLE vehicle_groups IS '车辆分组表';
COMMENT ON TABLE weighing_data IS '称重数据表';
COMMENT ON TABLE devices IS '设备信息表';
COMMENT ON TABLE device_data IS '设备原始数据表';
COMMENT ON TABLE report_templates IS '报表模板表';
COMMENT ON TABLE report_data IS '报表数据表';
COMMENT ON TABLE sync_logs IS '同步日志表';
COMMENT ON TABLE audit_logs IS '审计日志表';

-- 15. Add a trigger to automatically update the update_time column
-- Create a function to update update_time
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.update_time = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add triggers to all tables with update_time columns
DO $$
BEGIN
    -- Users table
    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger 
        WHERE tgname = 'update_users_modtime' AND tgrelid = 'users'::regclass
    ) THEN
        CREATE TRIGGER update_users_modtime 
        BEFORE UPDATE ON users 
        FOR EACH ROW 
        EXECUTE FUNCTION update_modified_column();
    END IF;
    
    -- User groups table
    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger 
        WHERE tgname = 'update_user_groups_modtime' AND tgrelid = 'user_groups'::regclass
    ) THEN
        CREATE TRIGGER update_user_groups_modtime 
        BEFORE UPDATE ON user_groups 
        FOR EACH ROW 
        EXECUTE FUNCTION update_modified_column();
    END IF;
    
    -- Vehicles table
    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger 
        WHERE tgname = 'update_vehicles_modtime' AND tgrelid = 'vehicles'::regclass
    ) THEN
        CREATE TRIGGER update_vehicles_modtime 
        BEFORE UPDATE ON vehicles 
        FOR EACH ROW 
        EXECUTE FUNCTION update_modified_column();
    END IF;
    
    -- Vehicle groups table
    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger 
        WHERE tgname = 'update_vehicle_groups_modtime' AND tgrelid = 'vehicle_groups'::regclass
    ) THEN
        CREATE TRIGGER update_vehicle_groups_modtime 
        BEFORE UPDATE ON vehicle_groups 
        FOR EACH ROW 
        EXECUTE FUNCTION update_modified_column();
    END IF;
    
    -- Devices table
    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger 
        WHERE tgname = 'update_devices_modtime' AND tgrelid = 'devices'::regclass
    ) THEN
        CREATE TRIGGER update_devices_modtime 
        BEFORE UPDATE ON devices 
        FOR EACH ROW 
        EXECUTE FUNCTION update_modified_column();
    END IF;
    
    -- Report templates table
    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger 
        WHERE tgname = 'update_report_templates_modtime' AND tgrelid = 'report_templates'::regclass
    ) THEN
        CREATE TRIGGER update_report_templates_modtime 
        BEFORE UPDATE ON report_templates 
        FOR EACH ROW 
        EXECUTE FUNCTION update_modified_column();
    END IF;
    
    -- Report data table
    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger 
        WHERE tgname = 'update_report_data_modtime' AND tgrelid = 'report_data'::regclass
    ) THEN
        CREATE TRIGGER update_report_data_modtime 
        BEFORE UPDATE ON report_data 
        FOR EACH ROW 
        EXECUTE FUNCTION update_modified_column();
    END IF;
END $$;

-- 16. Create a function to clean up old audit logs
CREATE OR REPLACE FUNCTION cleanup_old_logs()
RETURNS VOID AS $$
BEGIN
    -- Delete audit logs older than 1 year
    DELETE FROM audit_logs WHERE action_time < CURRENT_TIMESTAMP - INTERVAL '1 year';
    
    -- Delete sync logs older than 6 months
    DELETE FROM sync_logs WHERE sync_time < CURRENT_TIMESTAMP - INTERVAL '6 months';
    
    -- Delete device_data older than 3 months (adjust based on storage requirements)
    DELETE FROM device_data WHERE "timestamp" < CURRENT_TIMESTAMP - INTERVAL '3 months';
END;
$$ language 'plpgsql';

-- 17. Create a monthly cleanup job (requires pg_cron extension)
-- Note: This requires the pg_cron extension to be installed
-- CREATE EXTENSION IF NOT EXISTS pg_cron;
-- SELECT cron.schedule('0 0 1 * *', 'SELECT cleanup_old_logs();');

-- 18. Vacuum analyze tables to update statistics
-- 注意：VACUUM不能在事务块中运行，所以这些命令需要手动执行
-- VACUUM ANALYZE users;
-- VACUUM ANALYZE user_groups;
-- VACUUM ANALYZE vehicles;
-- VACUUM ANALYZE vehicle_groups;
-- VACUUM ANALYZE weighing_data;
-- VACUUM ANALYZE devices;
-- VACUUM ANALYZE device_data;
-- VACUUM ANALYZE report_templates;
-- VACUUM ANALYZE report_data;
-- VACUUM ANALYZE sync_logs;
-- VACUUM ANALYZE audit_logs;

-- 19. Optimize the database configuration
-- These settings should be applied to postgresql.conf
-- shared_buffers = 25% of RAM
-- effective_cache_size = 50% of RAM
-- maintenance_work_mem = 10% of RAM (up to 1GB)
-- work_mem = 4MB (adjust based on concurrent users)
-- random_page_cost = 1.1 (for SSDs)
-- effective_io_concurrency = 200 (for SSDs)
-- max_worker_processes = number of CPU cores
-- max_parallel_workers_per_gather = number of CPU cores / 2
-- wal_buffers = 16MB
-- checkpoint_completion_target = 0.9
-- min_wal_size = 1GB
-- max_wal_size = 4GB

-- 20. Add a note about monitoring
-- It's recommended to set up regular monitoring of:
-- - Slow queries using pg_stat_statements
-- - Index usage using pg_stat_user_indexes
-- - Table bloat using pgstattuple extension
-- - Connection pool usage
-- - Disk space usage

-- 21. Add a note about backup strategy
-- Implement regular backups:
-- - Daily full backups
-- - Hourly incremental backups
-- - Point-in-time recovery setup
-- - Test backups regularly

-- 22. Add a note about scaling
-- For large datasets, consider:
-- - Partitioning the weighing_data table by time
-- - Partitioning log tables by time
-- - Using read replicas for reporting queries
-- - Sharding for very large datasets

-- 23. Add a note about security
-- Ensure:
-- - Strong passwords for all database users
-- - Least privilege principle for database roles
-- - Regular security audits
-- - Encryption at rest and in transit
-- - Regular updates to PostgreSQL

-- 24. Add a note about performance testing
-- Regularly test database performance with:
-- - pgbench for basic benchmarking
-- - Custom workload testing with realistic data
-- - Load testing with expected concurrent users
-- - Query plan analysis for slow queries


