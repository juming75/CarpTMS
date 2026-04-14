//! Event Sourced Repositories for Order/Device/User
use crate::domain::ddd::EventSourcedAggregate;
use crate::domain::device_aggregate::{DeviceAggregate, DeviceId};
use crate::domain::order_aggregate::{OrderAggregate, OrderId};
use crate::domain::user_aggregate::{UserAggregate, UserId};
use crate::errors::{AppError, AppResult};
use crate::events::event_store::{
    get_events_by_aggregate_id, load_snapshot, save_events, save_snapshot,
};
use log::{debug, info};
use sqlx::PgPool;

const SNAPSHOT_THRESHOLD: u64 = 100;

// ============== Order Event Sourced Repository ==============
pub struct EventSourcedOrderRepository {
    #[allow(dead_code)]
    db: PgPool,
}

impl EventSourcedOrderRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    async fn rebuild(&self, id: &OrderId) -> AppResult<Option<OrderAggregate>> {
        let aid = id.to_string();
        if let Some(snap) = load_snapshot(&aid).await? {
            let mut agg = serde_json::from_value::<OrderAggregate>(snap.data).map_err(|e| {
                AppError::internal_error(&format!("Snapshot deserialization failed: {}", e), None)
            })?;
            let later: Vec<_> = get_events_by_aggregate_id(&aid)
                .await?
                .into_iter()
                .filter(|e| e.version as u64 > agg.version)
                .collect();
            for e in &later {
                agg.apply_event_in_place(e);
            }
            debug!(
                "Rebuilt order {} from snapshot(v{}) + {} events",
                id,
                agg.version,
                later.len()
            );
            return Ok(Some(agg));
        }
        let events = get_events_by_aggregate_id(&aid).await?;
        if events.is_empty() {
            return Ok(None);
        }
        let mut agg = OrderAggregate::new(crate::domain::order_aggregate::OrderCreateParams {
            order_no: String::new(),
            vehicle_id: 0,
            customer_name: String::new(),
            customer_phone: String::new(),
            origin: String::new(),
            destination: String::new(),
            cargo_type: String::new(),
            cargo_weight: 0.0,
        });
        agg.rebuild_from_events(&events)?;
        debug!("Rebuilt order {} from {} events", id, events.len());
        Ok(Some(agg))
    }

    pub async fn find_by_id(&self, id: &OrderId) -> AppResult<Option<OrderAggregate>> {
        self.rebuild(id).await
    }

    pub async fn save(&self, agg: &mut OrderAggregate) -> AppResult<()> {
        let events = agg.get_uncommitted_events();
        if !events.is_empty() {
            save_events(events).await?;
            if agg.version > 0 && agg.version.is_multiple_of(SNAPSHOT_THRESHOLD) {
                let data = serde_json::to_value(&agg).expect("domain aggregate should serialize");
                save_snapshot(&agg.id.to_string(), agg.version as i64, data).await?;
                info!("Snapshot created for order {} at v{}", agg.id, agg.version);
            }
        }
        agg.mark_events_committed();
        Ok(())
    }
}

// ============== Device Event Sourced Repository ==============
pub struct EventSourcedDeviceRepository {
    #[allow(dead_code)]
    db: PgPool,
}

impl EventSourcedDeviceRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: &DeviceId) -> AppResult<Option<DeviceAggregate>> {
        let events = get_events_by_aggregate_id(&id.to_string()).await?;
        if events.is_empty() {
            return Ok(None);
        }
        let mut agg = DeviceAggregate::new(String::new(), String::new(), String::new());
        agg.rebuild_from_events(&events)?;
        Ok(Some(agg))
    }

    pub async fn save(&self, agg: &mut DeviceAggregate) -> AppResult<()> {
        let events = agg.get_uncommitted_events();
        if !events.is_empty() {
            save_events(events).await?;
            if agg.version > 0 && agg.version.is_multiple_of(SNAPSHOT_THRESHOLD) {
                let data = serde_json::to_value(&agg).expect("device aggregate should serialize");
                save_snapshot(&agg.id.to_string(), agg.version as i64, data).await?;
            }
        }
        agg.mark_events_committed();
        Ok(())
    }
}

// ============== User Event Sourced Repository ==============
pub struct EventSourcedUserRepository {
    #[allow(dead_code)]
    db: PgPool,
}

impl EventSourcedUserRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: &UserId) -> AppResult<Option<UserAggregate>> {
        let events = get_events_by_aggregate_id(&id.to_string()).await?;
        if events.is_empty() {
            return Ok(None);
        }
        let mut agg = UserAggregate::new(String::new(), String::new(), 0);
        agg.rebuild_from_events(&events)?;
        Ok(Some(agg))
    }

    pub async fn save(&self, agg: &mut UserAggregate) -> AppResult<()> {
        let events = agg.get_uncommitted_events();
        if !events.is_empty() {
            save_events(events).await?;
            if agg.version > 0 && agg.version.is_multiple_of(SNAPSHOT_THRESHOLD) {
                let data = serde_json::to_value(&agg).expect("user aggregate should serialize");
                save_snapshot(&agg.id.to_string(), agg.version as i64, data).await?;
            }
        }
        agg.mark_events_committed();
        Ok(())
    }
}
