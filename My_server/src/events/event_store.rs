//! /! 事件存储服务
//!
//! 实现事件溯源(Event Sourcing)的核心功能,包括事件持久化和重放

use crate::domain::ddd::DomainEvent;
use crate::errors::{AppError, AppResult};
use chrono::DateTime;
use log::{debug, info};
use sqlx::PgPool;

/// 事件存储服务
pub struct EventStore {
    pub db: PgPool,
}

impl EventStore {
    /// 创建新的事件存储服务
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// 初始化事件存储表
    pub async fn initialize(&self) -> AppResult<()> {
        let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS domain_events (
            id VARCHAR(36) PRIMARY KEY,
            aggregate_type VARCHAR(255) NOT NULL,
            aggregate_id VARCHAR(36) NOT NULL,
            event_type VARCHAR(255) NOT NULL,
            event_data JSONB NOT NULL,
            occurred_at TIMESTAMP WITH TIME ZONE NOT NULL,
            version BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_domain_events_aggregate_id ON domain_events(aggregate_id);
        CREATE INDEX IF NOT EXISTS idx_domain_events_aggregate_type ON domain_events(aggregate_type);
        CREATE INDEX IF NOT EXISTS idx_domain_events_event_type ON domain_events(event_type);
        CREATE INDEX IF NOT EXISTS idx_domain_events_occurred_at ON domain_events(occurred_at);
        "#;

        sqlx::query(create_table_sql)
            .execute(&self.db)
            .await
            .map_err(|e| {
                AppError::db_error(
                    &format!("Failed to create domain_events table: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        info!("Event store initialized successfully");
        Ok(())
    }

    /// 保存单个领域事件
    pub async fn save_event(&self, event: &DomainEvent) -> AppResult<()> {
        let sql = r#"
        INSERT INTO domain_events (
            id, aggregate_type, aggregate_id, event_type, event_data, occurred_at, version
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7
        )
        "#;

        let result = sqlx::query(sql)
            .bind(&event.id)
            .bind(&event.aggregate_type)
            .bind(&event.aggregate_id)
            .bind(&event.event_type)
            .bind(&event.event_data)
            .bind(event.occurred_at)
            .bind(0) // 初始版本
            .execute(&self.db)
            .await
            .map_err(|e| {
                AppError::db_error(
                    &format!("Failed to save event: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        debug!(
            "Event saved successfully: {} ({} rows affected)",
            event.id,
            result.rows_affected()
        );
        Ok(())
    }

    /// 批量保存领域事件
    pub async fn save_events(&self, events: &[DomainEvent]) -> AppResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        // 直接使用数据库连接执行,暂时不使用事务
        let sql = r#"
        INSERT INTO domain_events (
            id, aggregate_type, aggregate_id, event_type, event_data, occurred_at, version
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        for event in events {
            sqlx::query::<sqlx::Postgres>(sql)
                .bind(&event.id)
                .bind(&event.aggregate_type)
                .bind(&event.aggregate_id)
                .bind(&event.event_type)
                .bind(&event.event_data)
                .bind(event.occurred_at)
                .bind(event.version)
                .execute(&self.db)
                .await
                .map_err(|e| {
                    AppError::db_error(
                        &format!("Failed to save event: {}", e),
                        Some(&e.to_string()),
                    )
                })?;
        }

        info!("Saved {} events successfully", events.len());
        Ok(())
    }

    /// 根据聚合ID获取所有事件
    pub async fn get_events_by_aggregate_id(
        &self,
        aggregate_id: &str,
    ) -> AppResult<Vec<DomainEvent>> {
        let sql = r#"
        SELECT id, aggregate_type, aggregate_id, event_type, event_data, occurred_at
        FROM domain_events
        WHERE aggregate_id = $1
        ORDER BY occurred_at ASC
        "#;

        let events = sqlx::query_as::<sqlx::Postgres, DomainEvent>(sql)
            .bind(aggregate_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| {
                AppError::db_error(
                    &format!("Failed to get events: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        debug!(
            "Retrieved {} events for aggregate: {}",
            events.len(),
            aggregate_id
        );
        Ok(events)
    }

    /// 根据聚合类型获取所有事件
    pub async fn get_events_by_aggregate_type(
        &self,
        aggregate_type: &str,
    ) -> AppResult<Vec<DomainEvent>> {
        let sql = r#"
        SELECT id, aggregate_type, aggregate_id, event_type, event_data, occurred_at
        FROM domain_events
        WHERE aggregate_type = $1
        ORDER BY occurred_at ASC
        "#;

        let events = sqlx::query_as::<sqlx::Postgres, DomainEvent>(sql)
            .bind(aggregate_type)
            .fetch_all(&self.db)
            .await
            .map_err(|e: sqlx::Error| {
                AppError::db_error(
                    &format!("Failed to get events: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        debug!(
            "Retrieved {} events for aggregate type: {}",
            events.len(),
            aggregate_type
        );
        Ok(events)
    }

    /// 根据事件类型获取所有事件
    pub async fn get_events_by_event_type(&self, event_type: &str) -> AppResult<Vec<DomainEvent>> {
        let sql = r#"
        SELECT id, aggregate_type, aggregate_id, event_type, event_data, occurred_at
        FROM domain_events
        WHERE event_type = $1
        ORDER BY occurred_at ASC
        "#;

        let events = sqlx::query_as::<sqlx::Postgres, DomainEvent>(sql)
            .bind(event_type)
            .fetch_all(&self.db)
            .await
            .map_err(|e: sqlx::Error| {
                AppError::db_error(
                    &format!("Failed to get events: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        debug!(
            "Retrieved {} events for event type: {}",
            events.len(),
            event_type
        );
        Ok(events)
    }

    /// 获取指定时间范围内的事件
    pub async fn get_events_by_time_range(
        &self,
        start_time: DateTime<chrono::Utc>,
        end_time: DateTime<chrono::Utc>,
    ) -> AppResult<Vec<DomainEvent>> {
        let sql = r#"
        SELECT id, aggregate_type, aggregate_id, event_type, event_data, occurred_at
        FROM domain_events
        WHERE occurred_at BETWEEN $1 AND $2
        ORDER BY occurred_at ASC
        "#;

        let events = sqlx::query_as::<sqlx::Postgres, DomainEvent>(sql)
            .bind(start_time)
            .bind(end_time)
            .fetch_all(&self.db)
            .await
            .map_err(|e: sqlx::Error| {
                AppError::db_error(
                    &format!("Failed to get events: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        debug!(
            "Retrieved {} events between {} and {}",
            events.len(),
            start_time,
            end_time
        );
        Ok(events)
    }

    /// 获取所有事件(分页)
    pub async fn get_all_events(
        &self,
        page: i32,
        page_size: i32,
    ) -> AppResult<(Vec<DomainEvent>, i64)> {
        let offset = (page - 1) * page_size;

        // 获取事件总数
        let count_sql = "SELECT COUNT(*) FROM domain_events";
        let total: i64 = sqlx::query_scalar(count_sql)
            .fetch_one(&self.db)
            .await
            .map_err(|e| {
                AppError::db_error(
                    &format!("Failed to count events: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        // 获取分页事件
        let events_sql = r#"
        SELECT id, aggregate_type, aggregate_id, event_type, event_data, occurred_at
        FROM domain_events
        ORDER BY occurred_at DESC
        LIMIT $1 OFFSET $2
        "#;

        let events = sqlx::query_as::<sqlx::Postgres, DomainEvent>(events_sql)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.db)
            .await
            .map_err(|e: sqlx::Error| {
                AppError::db_error(
                    &format!("Failed to get events: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        debug!(
            "Retrieved {} events (page {} of {})",
            events.len(),
            page,
            (total + page_size as i64 - 1) / page_size as i64
        );
        Ok((events, total))
    }

    /// 删除指定聚合的所有事件
    pub async fn delete_events_by_aggregate_id(&self, aggregate_id: &str) -> AppResult<()> {
        let sql = "DELETE FROM domain_events WHERE aggregate_id = $1";

        let result = sqlx::query(sql)
            .bind(aggregate_id)
            .execute(&self.db)
            .await
            .map_err(|e| {
                AppError::db_error(
                    &format!("Failed to delete events: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        info!(
            "Deleted {} events for aggregate: {}",
            result.rows_affected(),
            aggregate_id
        );
        Ok(())
    }

    /// 清理指定时间之前的事件
    pub async fn cleanup_events_before(&self, before: DateTime<chrono::Utc>) -> AppResult<()> {
        let sql = "DELETE FROM domain_events WHERE occurred_at < $1";

        let result = sqlx::query(sql)
            .bind(before)
            .execute(&self.db)
            .await
            .map_err(|e| {
                AppError::db_error(
                    &format!("Failed to cleanup events: {}", e),
                    Some(&e.to_string()),
                )
            })?;

        info!(
            "Cleaned up {} events before {}",
            result.rows_affected(),
            before
        );
        Ok(())
    }
}

/// 全局事件存储
static GLOBAL_EVENT_STORE: tokio::sync::OnceCell<EventStore> = tokio::sync::OnceCell::const_new();

/// 初始化全局事件存储
pub async fn init_global_event_store(db: PgPool) {
    let event_store = EventStore::new(db);
    event_store
        .initialize()
        .await
        .expect("Failed to initialize event store");

    GLOBAL_EVENT_STORE
        .get_or_init(|| async { event_store })
        .await;
}

/// 获取全局事件存储
pub fn global_event_store() -> &'static EventStore {
    GLOBAL_EVENT_STORE
        .get()
        .expect("Event store not initialized")
}

/// 便捷函数:保存事件
pub async fn save_event(event: &DomainEvent) -> AppResult<()> {
    global_event_store().save_event(event).await
}

/// 便捷函数:批量保存事件
pub async fn save_events(events: &[DomainEvent]) -> AppResult<()> {
    global_event_store().save_events(events).await
}

/// 便捷函数:获取聚合的事件
pub async fn get_events_by_aggregate_id(aggregate_id: &str) -> AppResult<Vec<DomainEvent>> {
    global_event_store()
        .get_events_by_aggregate_id(aggregate_id)
        .await
}

// ============== 快照支持 ==============

/// 快照数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    pub aggregate_id: String,
    pub version: i64,
    pub data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 全局事件存储
static SNAPSHOT_STORE: tokio::sync::OnceCell<SnapshotStore> = tokio::sync::OnceCell::const_new();

/// 快照存储(复用 event_store 的 db 连接)
pub struct SnapshotStore {
    db: PgPool,
}

impl SnapshotStore {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// 初始化快照表
    pub async fn initialize(&self) -> AppResult<()> {
        let sql = r#"
        CREATE TABLE IF NOT EXISTS event_snapshots (
            aggregate_id VARCHAR(255) PRIMARY KEY,
            version BIGINT NOT NULL,
            data JSONB NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        )
        "#;
        sqlx::query(sql).execute(&self.db).await.map_err(|e| {
            AppError::db_error("Failed to create snapshots table", Some(&e.to_string()))
        })?;
        Ok(())
    }

    /// 保存快照
    pub async fn save(
        &self,
        aggregate_id: &str,
        version: i64,
        data: serde_json::Value,
    ) -> AppResult<()> {
        let sql = r#"INSERT INTO event_snapshots (aggregate_id, version, data) VALUES ($1, $2, $3)
                      ON CONFLICT (aggregate_id) DO UPDATE SET version = $2, data = $3"#;
        sqlx::query(sql)
            .bind(aggregate_id)
            .bind(version)
            .bind(&data)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::db_error("Failed to save snapshot", Some(&e.to_string())))?;
        debug!(
            "Snapshot saved for aggregate {} at v{}",
            aggregate_id, version
        );
        Ok(())
    }

    /// 加载最新快照
    pub async fn load(&self, aggregate_id: &str) -> AppResult<Option<Snapshot>> {
        let sql = "SELECT aggregate_id, version, data, created_at FROM event_snapshots WHERE aggregate_id = $1";
        let result = sqlx::query_as::<
            sqlx::Postgres,
            (
                String,
                i64,
                serde_json::Value,
                chrono::DateTime<chrono::Utc>,
            ),
        >(sql)
        .bind(aggregate_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::db_error("Failed to load snapshot", Some(&e.to_string())))?;
        Ok(
            result.map(|(aggregate_id, version, data, created_at)| Snapshot {
                aggregate_id,
                version,
                data,
                created_at,
            }),
        )
    }
}

/// 初始化全局快照存储
pub async fn init_snapshot_store(db: PgPool) {
    let store = SnapshotStore::new(db.clone());
    store
        .initialize()
        .await
        .expect("Failed to initialize snapshot store");
    SNAPSHOT_STORE.get_or_init(|| async { store }).await;
}

/// 获取快照
pub async fn load_snapshot(aggregate_id: &str) -> AppResult<Option<Snapshot>> {
    SNAPSHOT_STORE
        .get()
        .expect("Snapshot store not initialized")
        .load(aggregate_id)
        .await
}

/// 保存快照
pub async fn save_snapshot(
    aggregate_id: &str,
    version: i64,
    data: serde_json::Value,
) -> AppResult<()> {
    SNAPSHOT_STORE
        .get()
        .expect("Snapshot store not initialized")
        .save(aggregate_id, version, data)
        .await
}

/// 批量保存事件(性能优化)
pub async fn save_events_batch(events: &[DomainEvent]) -> AppResult<()> {
    if events.is_empty() {
        return Ok(());
    }
    let mut tx = global_event_store()
        .db
        .begin()
        .await
        .map_err(|e| AppError::db_error("Failed to begin transaction", Some(&e.to_string())))?;
    let sql = r#"INSERT INTO domain_events (id, aggregate_type, aggregate_id, event_type, event_data, occurred_at, version)
                  VALUES ($1, $2, $3, $4, $5, $6, $7)"#;
    for event in events {
        sqlx::query::<sqlx::Postgres>(sql)
            .bind(&event.id)
            .bind(&event.aggregate_type)
            .bind(&event.aggregate_id)
            .bind(&event.event_type)
            .bind(&event.event_data)
            .bind(event.occurred_at)
            .bind(event.version)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::db_error("Failed to batch save event", Some(&e.to_string())))?;
    }
    tx.commit()
        .await
        .map_err(|e| AppError::db_error("Failed to commit batch", Some(&e.to_string())))?;
    info!("Batch saved {} events", events.len());
    Ok(())
}
