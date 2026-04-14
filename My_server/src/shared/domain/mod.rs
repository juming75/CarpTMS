//! Shared domain module - Core domain models and abstractions
//!
//! This module contains the fundamental domain concepts that are shared
//! across multiple business modules in the system.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
};

/// Aggregate root trait - represents the root entity of an aggregate
#[async_trait]
pub trait AggregateRoot: Send + Sync + Debug {
    type Id: Send + Sync + Debug + Clone;
    
    fn id(&self) -> &Self::Id;
    fn version(&self) -> u64;
    fn domain_events(&self) -> &[Arc<dyn DomainEvent>];
    fn mark_events_as_committed(&mut self);
}

/// Entity trait - represents a domain entity
#[async_trait]
pub trait Entity: Send + Sync + Debug {
    type Id: Send + Sync + Debug + Clone;
    
    fn id(&self) -> &Self::Id;
    fn is_same(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

/// Value object trait - represents a value object without identity
pub trait ValueObject: Send + Sync + Debug + Clone + PartialEq {
    fn validate(&self) -> Result<(), DomainError>;
}

/// Domain event trait
#[async_trait]
pub trait DomainEvent: Send + Sync + Debug {
    fn event_type(&self) -> &str;
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn aggregate_id(&self) -> &str;
    fn payload(&self) -> serde_json::Value;
}

/// Repository trait - defines the interface for aggregate persistence
#[async_trait]
pub trait Repository<T: AggregateRoot>: Send + Sync {
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>, DomainError>;
    async fn save(&self, aggregate: &mut T) -> Result<(), DomainError>;
    async fn delete(&self, id: &T::Id) -> Result<(), DomainError>;
}

/// Specification pattern for complex business rules
#[async_trait]
pub trait Specification<T>: Send + Sync {
    async fn is_satisfied_by(&self, candidate: &T) -> bool;
    fn and<S: Specification<T>>(&self, other: &S) -> AndSpecification<T>;
    fn or<S: Specification<T>>(&self, other: &S) -> OrSpecification<T>;
    fn not(&self) -> NotSpecification<T>;
}

/// And specification - combines two specifications with AND logic
pub struct AndSpecification<T> {
    left: Arc<dyn Specification<T>>,
    right: Arc<dyn Specification<T>>,
}

impl<T> AndSpecification<T> {
    pub fn new(left: Arc<dyn Specification<T>>, right: Arc<dyn Specification<T>>) -> Self {
        Self { left, right }
    }
}

#[async_trait]
impl<T: Send + Sync> Specification<T> for AndSpecification<T> {
    async fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.left.is_satisfied_by(candidate).await && self.right.is_satisfied_by(candidate).await
    }

    fn and<S: Specification<T>>(&self, other: &S) -> AndSpecification<T> {
        AndSpecification::new(Arc::new(self.clone()), Arc::new(CompositeSpecification::new(other)))
    }

    fn or<S: Specification<T>>(&self, other: &S) -> OrSpecification<T> {
        OrSpecification::new(Arc::new(self.clone()), Arc::new(CompositeSpecification::new(other)))
    }

    fn not(&self) -> NotSpecification<T> {
        NotSpecification::new(Arc::new(self.clone()))
    }
}

impl<T> Clone for AndSpecification<T> {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

/// Or specification - combines two specifications with OR logic
pub struct OrSpecification<T> {
    left: Arc<dyn Specification<T>>,
    right: Arc<dyn Specification<T>>,
}

impl<T> OrSpecification<T> {
    pub fn new(left: Arc<dyn Specification<T>>, right: Arc<dyn Specification<T>>) -> Self {
        Self { left, right }
    }
}

#[async_trait]
impl<T: Send + Sync> Specification<T> for OrSpecification<T> {
    async fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.left.is_satisfied_by(candidate).await || self.right.is_satisfied_by(candidate).await
    }

    fn and<S: Specification<T>>(&self, other: &S) -> AndSpecification<T> {
        AndSpecification::new(Arc::new(self.clone()), Arc::new(CompositeSpecification::new(other)))
    }

    fn or<S: Specification<T>>(&self, other: &S) -> OrSpecification<T> {
        OrSpecification::new(Arc::new(self.clone()), Arc::new(CompositeSpecification::new(other)))
    }

    fn not(&self) -> NotSpecification<T> {
        NotSpecification::new(Arc::new(self.clone()))
    }
}

impl<T> Clone for OrSpecification<T> {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

/// Not specification - negates a specification
pub struct NotSpecification<T> {
    spec: Arc<dyn Specification<T>>,
}

impl<T> NotSpecification<T> {
    pub fn new(spec: Arc<dyn Specification<T>>) -> Self {
        Self { spec }
    }
}

#[async_trait]
impl<T: Send + Sync> Specification<T> for NotSpecification<T> {
    async fn is_satisfied_by(&self, candidate: &T) -> bool {
        !self.spec.is_satisfied_by(candidate).await
    }

    fn and<S: Specification<T>>(&self, other: &S) -> AndSpecification<T> {
        AndSpecification::new(Arc::new(self.clone()), Arc::new(CompositeSpecification::new(other)))
    }

    fn or<S: Specification<T>>(&self, other: &S) -> OrSpecification<T> {
        OrSpecification::new(Arc::new(self.clone()), Arc::new(CompositeSpecification::new(other)))
    }

    fn not(&self) -> NotSpecification<T> {
        NotSpecification::new(Arc::new(self.clone()))
    }
}

impl<T> Clone for NotSpecification<T> {
    fn clone(&self) -> Self {
        Self {
            spec: self.spec.clone(),
        }
    }
}

/// Composite specification wrapper for trait objects
struct CompositeSpecification<T> {
    inner: Arc<dyn Fn(&T) -> bool + Send + Sync>,
}

impl<T> CompositeSpecification<T> {
    fn new<S: Specification<T>>(spec: &S) -> Self {
        let spec_ptr = Arc::new(spec);
        Self {
            inner: Arc::new(move |candidate: &T| {
                // This is a simplified implementation
                // In a real implementation, you'd need to handle the async nature
                false
            }),
        }
    }
}

/// Domain service trait - represents a domain service
#[async_trait]
pub trait DomainService: Send + Sync {
    async fn execute(&self) -> Result<(), DomainError>;
}

/// Business rule trait - represents a business rule
#[async_trait]
pub trait BusinessRule: Send + Sync {
    async fn is_satisfied(&self) -> Result<bool, DomainError>;
    fn error_message(&self) -> String;
}

/// Domain error - represents domain-level errors
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
    
    #[error("Invalid value object: {0}")]
    InvalidValueObject(String),
    
    #[error("Domain service error: {0}")]
    DomainServiceError(String),
    
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    #[error("Concurrency conflict: {0}")]
    ConcurrencyConflict(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Shared value objects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailAddress {
    pub value: String,
}

impl EmailAddress {
    pub fn new(value: String) -> Result<Self, DomainError> {
        // Simple email validation
        if value.contains('@') && value.contains('.') {
            Ok(Self { value })
        } else {
            Err(DomainError::ValidationError("Invalid email format".to_string()))
        }
    }
}

impl ValueObject for EmailAddress {
    fn validate(&self) -> Result<(), DomainError> {
        if self.value.contains('@') && self.value.contains('.') {
            Ok(())
        } else {
            Err(DomainError::ValidationError("Invalid email format".to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub country_code: String,
    pub number: String,
}

impl PhoneNumber {
    pub fn new(country_code: String, number: String) -> Result<Self, DomainError> {
        // Simple phone validation
        if number.len() >= 7 && number.len() <= 15 {
            Ok(Self { country_code, number })
        } else {
            Err(DomainError::ValidationError("Invalid phone number format".to_string()))
        }
    }
}

impl ValueObject for PhoneNumber {
    fn validate(&self) -> Result<(), DomainError> {
        if self.number.len() >= 7 && self.number.len() <= 15 {
            Ok(())
        } else {
            Err(DomainError::ValidationError("Invalid phone number format".to_string()))
        }
    }
}

/// Base aggregate root implementation
#[derive(Debug, Clone)]
pub struct BaseAggregateRoot {
    pub id: String,
    pub version: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub domain_events: Vec<Arc<dyn DomainEvent>>,
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

    pub fn add_domain_event(&mut self, event: Arc<dyn DomainEvent>) {
        self.domain_events.push(event);
    }

    pub fn mark_events_as_committed(&mut self) {
        self.domain_events.clear();
    }
}

#[async_trait]
impl AggregateRoot for BaseAggregateRoot {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn domain_events(&self) -> &[Arc<dyn DomainEvent>] {
        &self.domain_events
    }

    fn mark_events_as_committed(&mut self) {
        self.domain_events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_address_validation() {
        let valid_email = EmailAddress::new("test@example.com".to_string()).unwrap();
        assert_eq!(valid_email.value, "test@example.com");
        
        let invalid_email = EmailAddress::new("invalid-email".to_string());
        assert!(invalid_email.is_err());
    }

    #[tokio::test]
    async fn test_phone_number_validation() {
        let valid_phone = PhoneNumber::new("+1".to_string(), "1234567890".to_string()).unwrap();
        assert_eq!(valid_phone.country_code, "+1");
        assert_eq!(valid_phone.number, "1234567890");
        
        let invalid_phone = PhoneNumber::new("+1".to_string(), "123".to_string());
        assert!(invalid_phone.is_err());
    }

    #[tokio::test]
    async fn test_base_aggregate_root() {
        let mut aggregate = BaseAggregateRoot::new("test-123".to_string());
        assert_eq!(aggregate.id(), "test-123");
        assert_eq!(aggregate.version(), 0);
        assert!(aggregate.domain_events().is_empty());
    }
}






