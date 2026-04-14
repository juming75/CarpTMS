use sqlx::PgPool;

async fn run_migration(
    pool: &PgPool,
    name: &str,
    sql_commands: &[&str],
) -> Result<(), sqlx::Error> {
    println!("=== 执行迁移: {}", name);
    for (i, sql) in sql_commands.iter().enumerate() {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => println!("✓ 命令 {} 执行成功", i + 1),
            Err(e) => println!("⚠ 命令 {} 执行失败: {}", i + 1, e),
        }
    }
    Ok(())
}

async fn check_table_exists(pool: &PgPool, table: &str) -> Result<bool, sqlx::Error> {
    let result: (bool,) = sqlx::query_as(
        "SELECT EXISTS(
            SELECT 1 FROM information_schema.tables 
            WHERE table_name = $1
        )",
    )
    .bind(table)
    .fetch_one(pool)
    .await?;
    Ok(result.0)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 开始执行数据库迁移 - 补充缺失表 ===\n");

    let database_url = "postgres://postgres:123@localhost:5432/carptms_db?client_encoding=UTF8";

    println!("连接数据库: {}", database_url);
    let pool = PgPool::connect(database_url).await?;
    println!("✓ 数据库连接成功\n");

    // 1. 创建位置围栏表
    if !check_table_exists(&pool, "location_fences").await? {
        let fences_commands = [r#"CREATE TABLE IF NOT EXISTS location_fences (
                fence_id SERIAL PRIMARY KEY,
                fence_name VARCHAR(100) NOT NULL,
                fence_type VARCHAR(50) NOT NULL,
                center_latitude NUMERIC(10, 6),
                center_longitude NUMERIC(10, 6),
                radius INTEGER,
                polygon_points JSONB,
                rectangle_bounds JSONB,
                status VARCHAR(20) NOT NULL DEFAULT 'active',
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP
            )"#];
        run_migration(&pool, "创建位置围栏表", &fences_commands).await?;
    } else {
        println!("✓ 位置围栏表已存在");
    }

    // 2. 创建位置路由表
    if !check_table_exists(&pool, "location_routes").await? {
        let routes_commands = [r#"CREATE TABLE IF NOT EXISTS location_routes (
                route_id SERIAL PRIMARY KEY,
                route_name VARCHAR(100) NOT NULL,
                start_point JSONB NOT NULL,
                end_point JSONB NOT NULL,
                waypoints JSONB,
                distance NUMERIC(10, 2),
                estimated_time INTEGER,
                status VARCHAR(20) NOT NULL DEFAULT 'active',
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP
            )"#];
        run_migration(&pool, "创建位置路由表", &routes_commands).await?;
    } else {
        println!("✓ 位置路由表已存在");
    }

    // 3. 创建位置信息表
    if !check_table_exists(&pool, "location_positions").await? {
        let positions_commands = [r#"CREATE TABLE IF NOT EXISTS location_positions (
                position_id SERIAL PRIMARY KEY,
                latitude NUMERIC(10, 6) NOT NULL,
                longitude NUMERIC(10, 6) NOT NULL,
                altitude NUMERIC(10, 2),
                speed NUMERIC(10, 2),
                direction INTEGER,
                timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                status VARCHAR(20) NOT NULL DEFAULT 'active',
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP
            )"#];
        run_migration(&pool, "创建位置信息表", &positions_commands).await?;
    } else {
        println!("✓ 位置信息表已存在");
    }

    // 4. 创建地点信息表
    if !check_table_exists(&pool, "location_places").await? {
        let places_commands = [r#"CREATE TABLE IF NOT EXISTS location_places (
                place_id SERIAL PRIMARY KEY,
                place_name VARCHAR(100) NOT NULL,
                latitude NUMERIC(10, 6) NOT NULL,
                longitude NUMERIC(10, 6) NOT NULL,
                address TEXT,
                place_type VARCHAR(50),
                status VARCHAR(20) NOT NULL DEFAULT 'active',
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP
            )"#];
        run_migration(&pool, "创建地点信息表", &places_commands).await?;
    } else {
        println!("✓ 地点信息表已存在");
    }

    // 5. 创建角色表
    if !check_table_exists(&pool, "roles").await? {
        let roles_commands = [
            r#"CREATE TABLE IF NOT EXISTS roles (
                role_id SERIAL PRIMARY KEY,
                role_name VARCHAR(100) NOT NULL UNIQUE,
                description TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP
            )"#,
            r#"INSERT INTO roles (role_name, description) 
               VALUES 
                   ('admin', '系统管理员'),
                   ('user', '普通用户'),
                   ('manager', '部门经理')
               ON CONFLICT DO NOTHING"#,
        ];
        run_migration(&pool, "创建角色表", &roles_commands).await?;
    } else {
        println!("✓ 角色表已存在");
    }

    // 6. 创建系统设置表
    if !check_table_exists(&pool, "system_settings").await? {
        let settings_commands = [
            r#"CREATE TABLE IF NOT EXISTS system_settings (
                setting_key VARCHAR(100) PRIMARY KEY,
                setting_value JSONB NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )"#,
            r#"INSERT INTO system_settings (setting_key, setting_value) 
               VALUES 
                   ('system_config', '{"server_url": "http://127.0.0.1:8081", "sync_interval": 5, "auto_sync": true}'),
                   ('communication_config', '{"server_ip": "127.0.0.1", "server_port": 8988, "heartbeat_interval": 30, "timeout": 10, "reconnect_count": 3, "protocol": "tcp", "compression": true, "encryption": true}')
               ON CONFLICT DO NOTHING"#,
        ];
        run_migration(&pool, "创建系统设置表", &settings_commands).await?;
    } else {
        println!("✓ 系统设置表已存在");
    }

    println!("\n=== 数据库迁移 - 补充缺失表完成！ ===");
    Ok(())
}
