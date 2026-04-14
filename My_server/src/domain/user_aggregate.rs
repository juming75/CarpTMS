//! User Aggregate Root
use crate::domain::ddd::{AggregateRoot, DomainEvent, Entity, EntityId, EventSourcedAggregate};
use crate::errors::AppResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i32);
impl EntityId for UserId {
    fn type_name(&self) -> &'static str {
        "UserId"
    }
}
impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Disabled,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAggregate {
    pub id: UserId,
    pub user_name: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub user_group_id: i32,
    pub status: UserStatus,
    pub version: u64,
    events: Vec<DomainEvent>,
}

impl UserAggregate {
    pub fn new(user_name: String, full_name: String, user_group_id: i32) -> Self {
        let mut u = Self {
            id: UserId(0),
            user_name,
            full_name,
            phone: None,
            email: None,
            user_group_id,
            status: UserStatus::Active,
            version: 0,
            events: Vec::new(),
        };
        u.raise_event("UserCreated", serde_json::json!({"user_name": u.user_name}));
        u
    }
    pub fn disable(&mut self) -> AppResult<()> {
        if self.status == UserStatus::Disabled {
            return Err(crate::errors::AppError::validation("Already disabled"));
        }
        self.status = UserStatus::Disabled;
        self.raise_event("UserDisabled", serde_json::json!({"user_id": self.id.0}));
        Ok(())
    }
    pub fn activate(&mut self) -> AppResult<()> {
        self.status = UserStatus::Active;
        self.raise_event("UserActivated", serde_json::json!({"user_id": self.id.0}));
        Ok(())
    }
    pub fn lock(&mut self) -> AppResult<()> {
        self.status = UserStatus::Locked;
        self.raise_event("UserLocked", serde_json::json!({"user_id": self.id.0}));
        Ok(())
    }
    pub fn update_profile(
        &mut self,
        full_name: Option<String>,
        phone: Option<String>,
        email: Option<String>,
    ) -> AppResult<()> {
        if let Some(n) = full_name {
            self.full_name = n;
        }
        if let Some(p) = phone {
            self.phone = Some(p);
        }
        if let Some(e) = email {
            self.email = Some(e);
        }
        self.raise_event(
            "UserProfileUpdated",
            serde_json::json!({"user_id": self.id.0}),
        );
        Ok(())
    }
    fn raise_event(&mut self, et: &str, data: serde_json::Value) {
        self.events.push(DomainEvent::new(
            "User",
            &self.id.to_string(),
            et,
            data,
            self.version as i32 + 1,
        ));
        self.version += 1;
    }
    fn apply_event(&mut self, e: &DomainEvent) {
        match e.event_type.as_str() {
            "UserCreated" | "UserActivated" => self.status = UserStatus::Active,
            "UserDisabled" => self.status = UserStatus::Disabled,
            "UserLocked" => self.status = UserStatus::Locked,
            _ => {}
        }
        self.version = e.version as u64;
    }
}
impl Entity for UserAggregate {
    fn id(&self) -> &impl EntityId {
        &self.id
    }
}
impl AggregateRoot for UserAggregate {
    fn version(&self) -> u64 {
        self.version
    }
    fn events(&self) -> &[DomainEvent] {
        &self.events
    }
    fn clear_events(&mut self) {
        self.events.clear();
    }
}
impl EventSourcedAggregate for UserAggregate {
    fn rebuild_from_events(&mut self, events: &[DomainEvent]) -> AppResult<()> {
        for e in events {
            self.apply_event(e);
        }
        Ok(())
    }
    fn get_uncommitted_events(&self) -> &[DomainEvent] {
        &self.events
    }
    fn mark_events_committed(&mut self) {
        self.events.clear();
    }
}
