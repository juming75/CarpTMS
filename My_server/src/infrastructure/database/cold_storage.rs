//! 冷热数据分层存储模块
//! 实现历史数据归档策略，将旧数据迁移到冷存储

use log::{info, warn, error};
use sqlx::{PgPool, postgres::PgRow};
use chrono::{DateTime, Utc, Duration};
use anyhow::{Result, anyhow};

/// 历史数据归档配置
#[derive(Debug, Clone)]
pub struct ColdStorageConfig {
    /// 数据归档阈值（天数）
    pub archive_threshold_days: i64,
    /// 冷存储表前缀
    pub cold_table_prefix: String,
    /// 归档批处理大小
    pub batch_size: usize,
    /// 归档执行间隔（小时）
    pub archive_interval_hours: u64,
}

impl Default for ColdStorageConfig {
    fn default() -> Self {
        Self {
            archive_threshold_days: 30, // 默认30天
            cold_table_prefix: "cold_".to_string(),
            batch_size: 1000,
            archive_interval_hours: 24,
        }
    }
}

/// 历史数据归档服务
pub struct ColdStorageService {
    pool: PgPool,
    config: ColdStorageConfig,
}

impl ColdStorageService {
    /// 创建新的归档服务
    pub fn new(pool: PgPool, config: ColdStorageConfig) -> Self {
        Self {
            pool,
            config,
        }
    }

    /// 执行所有表的归档
    pub async fn archive_all_tables(&self) -> Result<()> {
        info!("开始执行历史数据归档");
        
        // 归档车辆实时位置数据
        if let Err(e) = self.archive_vehicle_locations().await {
            warn!("车辆位置数据归档失败: {}", e);
        }

        // 归档审计日志
        if let Err(e) = self.archive_audit_logs().await {
            warn!("审计日志归档失败: {}", e);
        }

        // 归档告警数据
        if let Err(e) = self.archive_alarms().await {
            warn!("告警数据归档失败: {}", e);
        }

        // 归档物流轨迹数据
        if let Err(e) = self.archive_logistics_tracks().await {
            warn!("物流轨迹数据归档失败: {}", e);
        }

        info!("历史数据归档完成");
        Ok(())
    }

    /// 归档车辆实时位置数据
    pub async fn archive_vehicle_locations(&self) -> Result<()> {
        let cutoff_date = Utc::now() - Duration::days(self.config.archive_threshold_days);
        info!("开始归档车辆位置数据，截止日期: {:?}", cutoff_date);

        // 确保冷存储表存在
        self.ensure_cold_table("vehicle_realtime_locations").await?;

        // 迁移数据
        let migrated = self.migrate_data(
            "vehicle_realtime_locations",
            "timestamp",
            cutoff_date
        ).await?;

        info!("成功归档 {} 条车辆位置数据", migrated);
        Ok(())
    }

    /// 归档审计日志
    pub async fn archive_audit_logs(&self) -> Result<()> {
        let cutoff_date = Utc::now() - Duration::days(self.config.archive_threshold_days);
        info!("开始归档审计日志，截止日期: {:?}", cutoff_date);

        // 确保冷存储表存在
        self.ensure_cold_table("audit_logs").await?;

        // 迁移数据
        let migrated = self.migrate_data(
            "audit_logs",
            "created_at",
            cutoff_date
        ).await?;

        info!("成功归档 {} 条审计日志", migrated);
        Ok(())
    }

    /// 归档告警数据
    pub async fn archive_alarms(&self) -> Result<()> {
        let cutoff_date = Utc::now() - Duration::days(self.config.archive_threshold_days);
        info!("开始归档告警数据，截止日期: {:?}", cutoff_date);

        // 确保冷存储表存在
        self.ensure_cold_table("alarms").await?;

        // 迁移数据
        let migrated = self.migrate_data(
            "alarms",
            "created_at",
            cutoff_date
        ).await?;

        info!("成功归档 {} 条告警数据", migrated);
        Ok(())
    }

    /// 归档物流轨迹数据
    pub async fn archive_logistics_tracks(&self) -> Result<()> {
        let cutoff_date = Utc::now() - Duration::days(self.config.archive_threshold_days);
        info!("开始归档物流轨迹数据，截止日期: {:?}", cutoff_date);

        // 确保冷存储表存在
        self.ensure_cold_table("logistics_tracks").await?;

        // 迁移数据
        let migrated = self.migrate_data(
            "logistics_tracks",
            "created_at",
            cutoff_date
        ).await?;

        info!("成功归档 {} 条物流轨迹数据", migrated);
        Ok(())
    }

    /// 确保冷存储表存在
    async fn ensure_cold_table(&self, table_name: &str) -> Result<()> {
        let cold_table_name = format!("{}{}", self.config.cold_table_prefix, table_name);
        
        // 检查表是否存在
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables 
                WHERE table_name = $1
            )"
        )
        .bind(&cold_table_name)
        .fetch_one(&self.pool)
        .await?;

        if !exists {
            info!("创建冷存储表: {}", cold_table_name);
            
            // 根据表名创建相应的冷存储表
            match table_name {
                "vehicle_realtime_locations" => {
                    sqlx::query(r#"
                        CREATE TABLE IF NOT EXISTS cold_vehicle_realtime_locations (
                            id BIGINT PRIMARY KEY,
                            vehicle_id INTEGER NOT NULL,
                            latitude DOUBLE PRECISION NOT NULL,
                            longitude DOUBLE PRECISION NOT NULL,
                            speed DOUBLE PRECISION NOT NULL,
                            direction INTEGER NOT NULL,
                            altitude DOUBLE PRECISION NOT NULL,
                            accuracy DOUBLE PRECISION,
                            status INTEGER NOT NULL,
                            timestamp TIMESTAMP NOT NULL,
                            update_time TIMESTAMP NOT NULL,
                            created_at TIMESTAMP NOT NULL
                        )
                    "#)
                    .execute(&self.pool)
                    .await?;
                }
                "audit_logs" => {
                    sqlx::query(r#"
                        CREATE TABLE IF NOT EXISTS cold_audit_logs (
                            id BIGINT PRIMARY KEY,
                            user_id INTEGER NOT NULL,
                            action VARCHAR NOT NULL,
                            resource VARCHAR NOT NULL,
                            ip_address VARCHAR NOT NULL,
                            user_agent VARCHAR NOT NULL,
                            created_at TIMESTAMP NOT NULL
                        )
                    "#)
                    .execute(&self.pool)
                    .await?;
                }
                "alarms" => {
                    sqlx::query(r#"
                        CREATE TABLE IF NOT EXISTS cold_alarms (
                            id INTEGER PRIMARY KEY,
                            vehicle_id INTEGER NOT NULL,
                            alarm_type VARCHAR NOT NULL,
                            alarm_level VARCHAR NOT NULL,
                            alarm_content VARCHAR NOT NULL,
                            status VARCHAR NOT NULL,
                            timestamp TIMESTAMP NOT NULL,
                            created_at TIMESTAMP NOT NULL,
                            updated_at TIMESTAMP NOT NULL
                        )
                    "#)
                    .execute(&self.pool)
                    .await?;
                }
                "logistics_tracks" => {
                    sqlx::query(r#"
                        CREATE TABLE IF NOT EXISTS cold_logistics_tracks (
                            track_id INTEGER PRIMARY KEY,
                            order_id INTEGER NOT NULL,
                            vehicle_id INTEGER NOT NULL,
                            track_time TIMESTAMP,
                            latitude DOUBLE PRECISION NOT NULL,
                            longitude DOUBLE PRECISION NOT NULL,
                            address VARCHAR,
                            status INTEGER NOT NULL,
                            remark VARCHAR,
                            create_time TIMESTAMP NOT NULL,
                            created_at TIMESTAMP NOT NULL
                        )
                    "#)
                    .execute(&self.pool)
                    .await?;
                }
                _ => {
                    return Err(anyhow!("不支持的表名: {}", table_name));
                }
            }

            info!("冷存储表 {} 创建成功", cold_table_name);
        }

        Ok(())
    }

    /// 迁移数据到冷存储
    async fn migrate_data(&self, table_name: &str, date_column: &str, cutoff_date: DateTime<Utc>) -> Result<u64> {
        let cold_table_name = format!("{}{}", self.config.cold_table_prefix, table_name);
        let mut total_migrated = 0;

        loop {
            // 开启事务
            let mut tx = self.pool.begin().await?;

            // 选择要迁移的数据
            let query = format!(
                "SELECT * FROM {} WHERE {} < $1 ORDER BY {} LIMIT {}",
                table_name, date_column, date_column, self.config.batch_size
            );

            let rows = sqlx::query(&query)
                .bind(cutoff_date)
                .fetch_all(&mut tx)
                .await?;

            if rows.is_empty() {
                tx.commit().await?;
                break;
            }

            // 迁移数据到冷存储表
            let insert_query = match table_name {
                "vehicle_realtime_locations" => r#"
                    INSERT INTO cold_vehicle_realtime_locations 
                    (id, vehicle_id, latitude, longitude, speed, direction, altitude, accuracy, status, timestamp, update_time, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
                "audit_logs" => r#"
                    INSERT INTO cold_audit_logs 
                    (id, user_id, action, resource, ip_address, user_agent, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                "alarms" => r#"
                    INSERT INTO cold_alarms 
                    (id, vehicle_id, alarm_type, alarm_level, alarm_content, status, timestamp, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                "logistics_tracks" => r#"
                    INSERT INTO cold_logistics_tracks 
                    (track_id, order_id, vehicle_id, track_time, latitude, longitude, address, status, remark, create_time, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
                _ => {
                    tx.rollback().await?;
                    return Err(anyhow!("不支持的表名: {}", table_name));
                }
            };

            // 插入数据
            for row in &rows {
                match table_name {
                    "vehicle_realtime_locations" => {
                        sqlx::query(insert_query)
                            .bind(row.get::<i64, _>("id"))
                            .bind(row.get::<i32, _>("vehicle_id"))
                            .bind(row.get::<f64, _>("latitude"))
                            .bind(row.get::<f64, _>("longitude"))
                            .bind(row.get::<f64, _>("speed"))
                            .bind(row.get::<i32, _>("direction"))
                            .bind(row.get::<f64, _>("altitude"))
                            .bind(row.get::<Option<f64>, _>("accuracy"))
                            .bind(row.get::<i32, _>("status"))
                            .bind(row.get::<DateTime<Utc>, _>("timestamp"))
                            .bind(row.get::<DateTime<Utc>, _>("update_time"))
                            .bind(row.get::<DateTime<Utc>, _>("created_at"))
                            .execute(&mut tx)
                            .await?;
                    }
                    "audit_logs" => {
                        sqlx::query(insert_query)
                            .bind(row.get::<i64, _>("id"))
                            .bind(row.get::<i32, _>("user_id"))
                            .bind(row.get::<String, _>("action"))
                            .bind(row.get::<String, _>("resource"))
                            .bind(row.get::<String, _>("ip_address"))
                            .bind(row.get::<String, _>("user_agent"))
                            .bind(row.get::<DateTime<Utc>, _>("created_at"))
                            .execute(&mut tx)
                            .await?;
                    }
                    "alarms" => {
                        sqlx::query(insert_query)
                            .bind(row.get::<i32, _>("id"))
                            .bind(row.get::<i32, _>("vehicle_id"))
                            .bind(row.get::<String, _>("alarm_type"))
                            .bind(row.get::<String, _>("alarm_level"))
                            .bind(row.get::<String, _>("alarm_content"))
                            .bind(row.get::<String, _>("status"))
                            .bind(row.get::<DateTime<Utc>, _>("timestamp"))
                            .bind(row.get::<DateTime<Utc>, _>("created_at"))
                            .bind(row.get::<DateTime<Utc>, _>("updated_at"))
                            .execute(&mut tx)
                            .await?;
                    }
                    "logistics_tracks" => {
                        sqlx::query(insert_query)
                            .bind(row.get::<i32, _>("track_id"))
                            .bind(row.get::<i32, _>("order_id"))
                            .bind(row.get::<i32, _>("vehicle_id"))
                            .bind(row.get::<Option<DateTime<Utc>>, _>("track_time"))
                            .bind(row.get::<f64, _>("latitude"))
                            .bind(row.get::<f64, _>("longitude"))
                            .bind(row.get::<Option<String>, _>("address"))
                            .bind(row.get::<i32, _>("status"))
                            .bind(row.get::<Option<String>, _>("remark"))
                            .bind(row.get::<DateTime<Utc>, _>("create_time"))
                            .bind(row.get::<DateTime<Utc>, _>("created_at"))
                            .execute(&mut tx)
                            .await?;
                    }
                    _ => {}
                }
            }

            // 删除原表中的数据
            let delete_query = format!(
                "DELETE FROM {} WHERE id IN (SELECT id FROM {} WHERE {} < $1 ORDER BY {} LIMIT {})",
                table_name, table_name, date_column, date_column, self.config.batch_size
            );

            let result = sqlx::query(&delete_query)
                .bind(cutoff_date)
                .execute(&mut tx)
                .await?;

            tx.commit().await?;

            let migrated = result.rows_affected();
            total_migrated += migrated;

            if migrated < self.config.batch_size as u64 {
                break;
            }
        }

        Ok(total_migrated)
    }

    /// 从冷存储查询数据
    pub async fn query_cold_data(&self, table_name: &str, query: &str, params: &[&dyn sqlx::Encode<'_, sqlx::Postgres>]) -> Result<Vec<PgRow>> {
        let cold_table_name = format!("{}{}", self.config.cold_table_prefix, table_name);
        let query = query.replace(table_name, &cold_table_name);
        
        let rows = sqlx::query(&query)
            .bind_all(params)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }
}
