//! / JT808 数据持久化
// 将 JT808 设备数据存储到数据库

use chrono::{DateTime, Utc};
use log::{debug, info};
use sqlx::postgres::PgPool;

/// 位置数据元组类型 (用于批量插入)
pub type LocationTuple = (
    String,        // device_id
    String,        // phone
    DateTime<Utc>, // timestamp
    f64,           // lat
    f64,           // lng
    Option<f32>,   // altitude
    Option<f32>,   // speed
    Option<i32>,   // direction
    Option<i32>,   // status
    Option<i32>,   // alarm_flag
);

/// 位置数据参数结构体
#[derive(Debug, Clone)]
pub struct LocationData {
    pub device_id: String,
    pub phone: String,
    pub timestamp: DateTime<Utc>,
    pub lat: f64,
    pub lng: f64,
    pub altitude: Option<f32>,
    pub speed: Option<f32>,
    pub direction: Option<i32>,
    pub status: Option<i32>,
    pub alarm_flag: Option<i32>,
}

/// JT808 设备会话记录
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Jt808DeviceSessionRecord {
    pub id: i32,
    pub device_id: String,
    pub phone: String,
    pub auth_status: String,
    pub last_activity: DateTime<Utc>,
    pub heartbeat_time: Option<DateTime<Utc>>,
    pub flow_no: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// JT808 位置历史记录
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Jt808LocationRecord {
    pub id: i64,
    pub device_id: String,
    pub phone: String,
    pub timestamp: DateTime<Utc>,
    pub lat: f64,
    pub lng: f64,
    pub altitude: Option<f32>,
    pub speed: Option<f32>,
    pub direction: Option<i32>,
    pub status: Option<i32>,
    pub alarm_flag: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// JT808 报警历史记录
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Jt808AlarmRecord {
    pub id: i64,
    pub device_id: String,
    pub phone: String,
    pub alarm_type: String,
    pub alarm_level: i32,
    pub alarm_time: DateTime<Utc>,
    pub location: Option<serde_json::Value>,
    pub description: Option<String>,
    pub acknowledged: bool,
    pub created_at: DateTime<Utc>,
}

/// 报警数据参数结构体
#[derive(Debug, Clone)]
pub struct AlarmData {
    pub device_id: String,
    pub phone: String,
    pub alarm_type: String,
    pub alarm_level: i32,
    pub alarm_time: DateTime<Utc>,
    pub location: Option<serde_json::Value>,
    pub description: Option<String>,
}

/// JT808 指令日志记录
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Jt808CommandRecord {
    pub id: i64,
    pub device_id: String,
    pub command_id: i32,
    pub command_name: Option<String>,
    pub params: Option<serde_json::Value>,
    pub status: String,
    pub sent_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub response_data: Option<serde_json::Value>,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
}

/// JT808 数据存储库
pub struct Jt808Repository {
    pool: PgPool,
}

impl Jt808Repository {
    /// 创建新的存储库
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 保存设备会话
    pub async fn save_device_session(
        &self,
        device_id: &str,
        phone: &str,
        auth_status: &str,
        flow_no: u16,
    ) -> Result<i32, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query_as::<_, (i32,)>(
            r#"
            INSERT INTO jt808_device_sessions (device_id, phone, auth_status, flow_no, last_activity, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (device_id)
            DO UPDATE SET
                phone = EXCLUDED.phone,
                auth_status = EXCLUDED.auth_status,
                flow_no = EXCLUDED.flow_no,
                last_activity = EXCLUDED.last_activity,
                updated_at = $5
            RETURNING id
            "#
        )
        .bind(device_id)
        .bind(phone)
        .bind(auth_status)
        .bind(flow_no as i32)
        .bind(now)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    /// 更新设备会话活动时间
    pub async fn update_session_activity(
        &self,
        device_id: &str,
        flow_no: Option<u16>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        let mut query =
            sqlx::query("UPDATE jt808_device_sessions SET last_activity = $1, updated_at = $1");
        query = query.bind(now);

        if let Some(flow_no) = flow_no {
            query = sqlx::query(
                "UPDATE jt808_device_sessions SET last_activity = $1, flow_no = $2, updated_at = $1 WHERE device_id = $3"
            );
            query = query.bind(now).bind(flow_no as i32);
        }

        query.bind(device_id).execute(&self.pool).await?;

        Ok(())
    }

    /// 删除设备会话
    pub async fn delete_device_session(&self, device_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM jt808_device_sessions WHERE device_id = $1")
            .bind(device_id)
            .execute(&self.pool)
            .await?;

        info!("Device session deleted: {}", device_id);
        Ok(())
    }

    /// 保存位置数据
    pub async fn save_location(&self, data: &LocationData) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            INSERT INTO jt808_location_history
            (device_id, phone, timestamp, lat, lng, altitude, speed, direction, status, alarm_flag, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id
            "#
        )
        .bind(&data.device_id)
        .bind(&data.phone)
        .bind(data.timestamp)
        .bind(data.lat)
        .bind(data.lng)
        .bind(data.altitude)
        .bind(data.speed)
        .bind(data.direction)
        .bind(data.status)
        .bind(data.alarm_flag)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        debug!(
            "Location saved: {} at ({}, {})",
            data.device_id, data.lat, data.lng
        );
        Ok(result.0)
    }

    /// 批量保存位置数据
    pub async fn save_locations_batch(
        &self,
        locations: Vec<LocationTuple>,
    ) -> Result<u64, sqlx::Error> {
        if locations.is_empty() {
            return Ok(0);
        }

        let mut query = String::from(
            "INSERT INTO jt808_location_history \
             (device_id, phone, timestamp, lat, lng, altitude, speed, direction, status, alarm_flag, created_at) \
             VALUES ",
        );

        let mut values = Vec::new();
        for (
            i,
            (device_id, phone, timestamp, lat, lng, altitude, speed, direction, status, alarm_flag),
        ) in locations.iter().enumerate()
        {
            if i > 0 {
                query.push_str(", ");
            }
            let param_num = i * 10 + 1;
            query.push_str(&format!(
                "(${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, NOW())",
                param_num,
                param_num + 1,
                param_num + 2,
                param_num + 3,
                param_num + 4,
                param_num + 5,
                param_num + 6,
                param_num + 7,
                param_num + 8,
                param_num + 9
            ));

            values.push(device_id.clone());
            values.push(phone.clone());
            values.push(timestamp.to_rfc3339()); // DateTime<Utc> -> String
            values.push(lat.to_string()); // f64 -> String
            values.push(lng.to_string()); // f64 -> String
            values.push(
                altitude
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "NULL".to_string()),
            ); // Option<f32> -> String
            values.push(
                speed
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "NULL".to_string()),
            ); // Option<f32> -> String
            values.push(
                direction
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "NULL".to_string()),
            ); // Option<i32> -> String
            values.push(
                status
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "NULL".to_string()),
            ); // Option<i32> -> String
            values.push(
                alarm_flag
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "NULL".to_string()),
            ); // Option<i32> -> String
        }

        let mut q = sqlx::query(&query);
        for value in values {
            q = q.bind(value);
        }

        let result = q.execute(&self.pool).await?;
        info!("Batch inserted {} locations", result.rows_affected());
        Ok(result.rows_affected())
    }

    /// 保存报警数据
    pub async fn save_alarm(&self, data: &AlarmData) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            INSERT INTO jt808_alarm_history
            (device_id, phone, alarm_type, alarm_level, alarm_time, location, description, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#
        )
        .bind(&data.device_id)
        .bind(&data.phone)
        .bind(&data.alarm_type)
        .bind(data.alarm_level)
        .bind(data.alarm_time)
        .bind(&data.location)
        .bind(&data.description)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        info!("Alarm saved: {} - {}", data.device_id, data.alarm_type);
        Ok(result.0)
    }

    /// 保存指令日志
    pub async fn save_command_log(
        &self,
        device_id: &str,
        command_id: i32,
        command_name: Option<String>,
        params: Option<serde_json::Value>,
        status: &str,
        retry_count: i32,
    ) -> Result<i64, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query_as::<_, (i64,)>(
            r#"
            INSERT INTO jt808_command_log
            (device_id, command_id, command_name, params, status, sent_at, retry_count, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
        )
        .bind(device_id)
        .bind(command_id)
        .bind(command_name)
        .bind(params)
        .bind(status)
        .bind(now)
        .bind(retry_count)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        debug!("Command log saved: {} - 0x{:04X}", device_id, command_id);
        Ok(result.0)
    }

    /// 更新指令状态
    pub async fn update_command_status(
        &self,
        command_id: i64,
        status: &str,
        acknowledged_at: Option<DateTime<Utc>>,
        response_data: Option<serde_json::Value>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE jt808_command_log
            SET status = $1, acknowledged_at = $2, response_data = $3
            WHERE id = $4
            "#,
        )
        .bind(status)
        .bind(acknowledged_at)
        .bind(response_data)
        .bind(command_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 查询设备会话
    pub async fn query_device_session(
        &self,
        device_id: &str,
    ) -> Result<Option<Jt808DeviceSessionRecord>, sqlx::Error> {
        sqlx::query_as::<_, Jt808DeviceSessionRecord>(
            "SELECT * FROM jt808_device_sessions WHERE device_id = $1",
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await
    }

    /// 查询位置历史
    pub async fn query_location_history(
        &self,
        device_id: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: i64,
    ) -> Result<Vec<Jt808LocationRecord>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM jt808_location_history WHERE device_id = $1");
        let mut param_count = 1;

        if start_time.is_some() || end_time.is_some() {
            query.push_str(" AND ");
            let mut conditions = Vec::new();

            if let Some(_start) = start_time {
                conditions.push(format!("${} >= timestamp", param_count + 1));
                param_count += 1;
            }

            if let Some(_end) = end_time {
                conditions.push(format!("${} <= timestamp", param_count + 1));
                param_count += 1;
            }

            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(&format!(
            " ORDER BY timestamp DESC LIMIT ${}",
            param_count + 1
        ));

        let mut q = sqlx::query_as::<_, Jt808LocationRecord>(&query);
        q = q.bind(device_id);

        if let Some(start) = start_time {
            q = q.bind(start);
        }

        if let Some(end) = end_time {
            q = q.bind(end);
        }

        q = q.bind(limit);

        q.fetch_all(&self.pool).await
    }

    /// 查询报警历史
    pub async fn query_alarm_history(
        &self,
        device_id: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: i64,
    ) -> Result<Vec<Jt808AlarmRecord>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM jt808_alarm_history WHERE 1=1");
        let mut param_count = 0;

        if let Some(_id) = device_id {
            param_count += 1;
            query.push_str(&format!(" AND device_id = ${}", param_count));
        }

        if start_time.is_some() || end_time.is_some() {
            param_count += 1;
            if let Some(_start) = start_time {
                query.push_str(&format!(" AND alarm_time >= ${}", param_count));
            }

            param_count += 1;
            if let Some(_end) = end_time {
                query.push_str(&format!(" AND alarm_time <= ${}", param_count));
            }
        }

        query.push_str(&format!(
            " ORDER BY alarm_time DESC LIMIT ${}",
            param_count + 1
        ));

        let mut q = sqlx::query_as::<_, Jt808AlarmRecord>(&query);

        if let Some(id) = device_id {
            q = q.bind(id);
        }

        if let Some(start) = start_time {
            q = q.bind(start);
        }

        if let Some(end) = end_time {
            q = q.bind(end);
        }

        q = q.bind(limit);

        q.fetch_all(&self.pool).await
    }

    /// 初始化数据库表
    pub async fn init_tables(&self) -> Result<(), sqlx::Error> {
        info!("Initializing JT808 database tables...");

        // 设备会话表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jt808_device_sessions (
                id SERIAL PRIMARY KEY,
                device_id VARCHAR(20) NOT NULL UNIQUE,
                phone VARCHAR(20) NOT NULL,
                auth_status VARCHAR(20) NOT NULL DEFAULT 'unauthenticated',
                last_activity TIMESTAMP NOT NULL DEFAULT NOW(),
                heartbeat_time TIMESTAMP,
                flow_no INT NOT NULL DEFAULT 0,
                created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_jt808_sessions_device ON jt808_device_sessions(device_id);
            CREATE INDEX IF NOT EXISTS idx_jt808_sessions_activity ON jt808_device_sessions(last_activity);
            "#
        )
        .execute(&self.pool)
        .await?;

        // 位置历史表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jt808_location_history (
                id BIGSERIAL PRIMARY KEY,
                device_id VARCHAR(20) NOT NULL,
                phone VARCHAR(20) NOT NULL,
                timestamp TIMESTAMP NOT NULL,
                lat NUMERIC(10, 7) NOT NULL,
                lng NUMERIC(10, 7) NOT NULL,
                altitude NUMERIC(8, 2),
                speed NUMERIC(6, 2),
                direction INT,
                status INT,
                alarm_flag INT,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_jt808_location_device ON jt808_location_history(device_id);
            CREATE INDEX IF NOT EXISTS idx_jt808_location_time ON jt808_location_history(timestamp);
            CREATE INDEX IF NOT EXISTS idx_jt808_location_device_time ON jt808_location_history(device_id, timestamp);
            "#
        )
        .execute(&self.pool)
        .await?;

        // 报警历史表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jt808_alarm_history (
                id BIGSERIAL PRIMARY KEY,
                device_id VARCHAR(20) NOT NULL,
                phone VARCHAR(20) NOT NULL,
                alarm_type VARCHAR(50) NOT NULL,
                alarm_level INT NOT NULL,
                alarm_time TIMESTAMP NOT NULL,
                location JSONB,
                description TEXT,
                acknowledged BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_jt808_alarm_device ON jt808_alarm_history(device_id);
            CREATE INDEX IF NOT EXISTS idx_jt808_alarm_time ON jt808_alarm_history(alarm_time);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 指令日志表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jt808_command_log (
                id BIGSERIAL PRIMARY KEY,
                device_id VARCHAR(20) NOT NULL,
                command_id INT NOT NULL,
                command_name VARCHAR(100),
                params JSONB,
                status VARCHAR(20) NOT NULL DEFAULT 'pending',
                sent_at TIMESTAMP NOT NULL DEFAULT NOW(),
                acknowledged_at TIMESTAMP,
                response_data JSONB,
                retry_count INT DEFAULT 0,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_jt808_command_device ON jt808_command_log(device_id);
            CREATE INDEX IF NOT EXISTS idx_jt808_command_status ON jt808_command_log(status);
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("JT808 database tables initialized successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
