//! Service Mesh module
//! Provides advanced traffic management, service discovery, and load balancing

mod traffic_manager;
mod service_registry;
mod health_checker;
mod circuit_breaker;
mod routing;

pub use traffic_manager::TrafficManager;
pub use service_registry::ServiceRegistry;
pub use health_checker::HealthChecker;
pub use circuit_breaker::CircuitBreaker;
pub use routing::ServiceRouter;
