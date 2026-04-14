//! Shared Kernel - Domain-Driven Design Core Concepts
//!
//! This module contains the shared domain concepts and abstractions that are
//! used across different bounded contexts in the system.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Base trait for entities that have a unique identity
pub trait Entity: Debug + Send + Sync {
    /// Get the unique identifier of this entity
    fn id(&self) -> &str;
    
    /// Check if this entity is the same as another entity
    fn same_identity_as(&self, other: &dyn Entity) -> bool {
        self.id() == other.id()
    }
}

/// Base trait for value objects that are immutable and have no identity
pub trait ValueObject: Debug + Clone + Send + Sync + PartialEq {
    /// Get the type name of this value object
    fn value_object_type(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Base trait for aggregate roots that are the consistency boundary
pub trait AggregateRoot: Entity {
    /// Get the version of this aggregate for optimistic concurrency control
    fn version(&self) -> u64;
    
    /// Get the domain events that have occurred on this aggregate
    fn domain_events(&self) -> &[Box<dyn DomainEvent>];
    
    /// Clear domain events after they've been processed
    fn clear_domain_events(&mut self);
}

/// Base trait for domain events
pub trait DomainEvent: Debug + Send + Sync {
    /// Get the name/type of this domain event
    fn event_type(&self) -> &'static str;
    
    /// Get when this event occurred
    fn occurred_on(&self) -> chrono::DateTime<chrono::Utc>;
    
    /// Get the aggregate ID that this event relates to
    fn aggregate_id(&self) -> &str;
    
    /// Convert this event to JSON for serialization
    fn to_json(&self) -> serde_json::Value;
}

/// Base trait for domain services that contain business logic
pub trait DomainService: Send + Sync {
    /// Get the name of this domain service
    fn service_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Base trait for repositories that provide access to aggregates
pub trait Repository<T: AggregateRoot>: Send + Sync {
    /// Find an aggregate by its ID
    async fn find_by_id(&self, id: &str) -> Result<Option<T>, RepositoryError>;
    
    /// Save an aggregate
    async fn save(&self, aggregate: &mut T) -> Result<(), RepositoryError>;
    
    /// Delete an aggregate
    async fn delete(&self, id: &str) -> Result<(), RepositoryError>;
}

/// Repository error types
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Aggregate not found: {0}")]
    NotFound(String),
    
    #[error("Concurrency conflict: {0}")]
    ConcurrencyConflict(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Base implementation for domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseDomainEvent {
    pub event_type: String,
    pub aggregate_id: String,
    pub occurred_on: chrono::DateTime<chrono::Utc>,
    pub version: u64,
    pub payload: serde_json::Value,
}

impl BaseDomainEvent {
    pub fn new(event_type: String, aggregate_id: String, version: u64, payload: serde_json::Value) -> Self {
        Self {
            event_type,
            aggregate_id,
            occurred_on: chrono::Utc::now(),
            version,
            payload,
        }
    }
}

impl DomainEvent for BaseDomainEvent {
    fn event_type(&self) -> &'static str {
        Box::leak(self.event_type.clone().into_boxed_str())
    }
    
    fn occurred_on(&self) -> chrono::DateTime<chrono::Utc> {
        self.occurred_on
    }
    
    fn aggregate_id(&self) -> &str {
        &self.aggregate_id
    }
    
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "event_type": self.event_type,
            "aggregate_id": self.aggregate_id,
            "occurred_on": self.occurred_on,
            "version": self.version,
            "payload": self.payload,
        })
    }
}

/// Base implementation for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl BaseEntity {
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }
}

impl Entity for BaseEntity {
    fn id(&self) -> &str {
        &self.id
    }
}

/// Base implementation for value objects
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseValueObject {
    pub value: serde_json::Value,
    pub value_type: String,
}

impl BaseValueObject {
    pub fn new<T: Serialize>(value: T, value_type: String) -> Self {
        Self {
            value: serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
            value_type,
        }
    }
}

impl ValueObject for BaseValueObject {}

/// Base implementation for aggregate roots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseAggregateRoot {
    pub id: String,
    pub version: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip)]
    pub domain_events: Vec<Box<dyn DomainEvent>>,
}

impl BaseAggregateRoot {
    pub fn new(id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            version: 0,
            created_at: now,
            updated_at: now,
            domain_events: Vec::new(),
        }
    }
    
    /// Add a domain event to this aggregate
    pub fn add_domain_event(&mut self, event: Box<dyn DomainEvent>) {
        self.domain_events.push(event);
        self.version += 1;
        self.updated_at = chrono::Utc::now();
    }
    
    /// Generate a new ID for this aggregate
    pub fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }
}

impl Entity for BaseAggregateRoot {
    fn id(&self) -> &str {
        &self.id
    }
}

impl AggregateRoot for BaseAggregateRoot {
    fn version(&self) -> u64 {
        self.version
    }
    
    fn domain_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.domain_events
    }
    
    fn clear_domain_events(&mut self) {
        self.domain_events.clear();
    }
}

/// Specification pattern for business rules
pub trait Specification<T>: Send + Sync {
    /// Check if the given object satisfies this specification
    fn is_satisfied_by(&self, candidate: &T) -> bool;
    
    /// Combine this specification with another using AND logic
    fn and<S: Specification<T>>(self, other: S) -> AndSpecification<T, Self, S>
    where
        Self: Sized,
    {
        AndSpecification::new(self, other)
    }
    
    /// Combine this specification with another using OR logic
    fn or<S: Specification<T>>(self, other: S) -> OrSpecification<T, Self, S>
    where
        Self: Sized,
    {
        OrSpecification::new(self, other)
    }
    
    /// Negate this specification
    fn not(self) -> NotSpecification<T, Self>
    where
        Self: Sized,
    {
        NotSpecification::new(self)
    }
}

/// AND specification that combines two specifications
pub struct AndSpecification<T, S1, S2> {
    spec1: S1,
    spec2: S2,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, S1: Specification<T>, S2: Specification<T>> AndSpecification<T, S1, S2> {
    pub fn new(spec1: S1, spec2: S2) -> Self {
        Self {
            spec1,
            spec2,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, S1: Specification<T>, S2: Specification<T>> Specification<T> for AndSpecification<T, S1, S2> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.spec1.is_satisfied_by(candidate) && self.spec2.is_satisfied_by(candidate)
    }
}

/// OR specification that combines two specifications
pub struct OrSpecification<T, S1, S2> {
    spec1: S1,
    spec2: S2,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, S1: Specification<T>, S2: Specification<T>> OrSpecification<T, S1, S2> {
    pub fn new(spec1: S1, spec2: S2) -> Self {
        Self {
            spec1,
            spec2,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, S1: Specification<T>, S2: Specification<T>> Specification<T> for OrSpecification<T, S1, S2> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.spec1.is_satisfied_by(candidate) || self.spec2.is_satisfied_by(candidate)
    }
}

/// NOT specification that negates a specification
pub struct NotSpecification<T, S> {
    spec: S,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, S: Specification<T>> NotSpecification<T, S> {
    pub fn new(spec: S) -> Self {
        Self {
            spec,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, S: Specification<T>> Specification<T> for NotSpecification<T, S> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        !self.spec.is_satisfied_by(candidate)
    }
}

/// Business rule validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }
    
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }
}

/// Trait for objects that can be validated against business rules
pub trait Validatable: Send + Sync {
    /// Validate this object against business rules
    fn validate(&self) -> ValidationResult;
}





