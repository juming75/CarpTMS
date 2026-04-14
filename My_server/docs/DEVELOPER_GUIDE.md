# CarpTMS Developer Guide

## Overview

CarpTMS (My Transportation Management System) is a comprehensive platform for managing transportation and logistics operations. This guide provides developers with the necessary information to understand, contribute to, and extend the system.

## Technology Stack

### Backend
- **Language**: Rust (Edition 2021)
- **Web Framework**: Actix-web 4.x
- **Database**: PostgreSQL with SQLx
- **Cache**: Redis with connection pooling
- **Async Runtime**: Tokio
- **Serialization**: Serde, JSON, Bincode
- **Authentication**: JWT with Ring cryptography
- **Configuration**: Environment variables, JSON, YAML, TOML

### Frontend
- **Framework**: Vue.js 3.x
- **Build Tool**: Vite
- **UI Library**: Element Plus
- **State Management**: Pinia
- **Charts**: ECharts
- **Maps**: OpenLayers

### Infrastructure
- **Containerization**: Docker & Docker Compose
- **Orchestration**: Kubernetes (optional)
- **Monitoring**: Prometheus, Grafana
- **Logging**: Structured logging with Tracing
- **CI/CD**: GitHub Actions

## Architecture

### Clean Architecture

The system follows Clean Architecture principles with clear separation of concerns:

```
src/
├── platform/          # Platform layer - core services
├── shared/           # Shared kernel - DDD concepts
├── domain/           # Domain layer - business logic
├── infrastructure/   # Infrastructure layer - external services
├── application/      # Application layer - use cases
├── presentation/     # Presentation layer - API endpoints
└── main.rs          # Application entry point
```

### Domain-Driven Design (DDD)

The system implements DDD patterns:

- **Entities**: Objects with unique identity
- **Value Objects**: Immutable objects without identity
- **Aggregate Roots**: Consistency boundaries
- **Domain Events**: Events that represent business occurrences
- **Repositories**: Data access abstractions
- **Domain Services**: Business logic that doesn't belong to entities

### Platform Layer

The platform layer provides unified services:

- **Cache**: Multi-provider caching (memory, Redis, distributed)
- **Configuration**: Multi-source configuration management
- **Security**: Authentication, authorization, encryption
- **Protocols**: Protocol parsing and management
- **Events**: Event handling and dispatching

## Development Setup

### Prerequisites

1. **Rust Development Environment**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **PostgreSQL Database**
   ```bash
   # Install PostgreSQL (Ubuntu/Debian)
   sudo apt-get install postgresql postgresql-contrib
   
   # Start PostgreSQL service
   sudo systemctl start postgresql
   ```

3. **Redis Cache**
   ```bash
   # Install Redis (Ubuntu/Debian)
   sudo apt-get install redis-server
   
   # Start Redis service
   sudo systemctl start redis-server
   ```

### Project Setup

1. **Clone the Repository**
   ```bash
   git clone https://github.com/your-org/CarpTMS.git
   cd CarpTMS
   ```

2. **Database Setup**
   ```bash
   # Create database
   sudo -u postgres createdb CarpTMS
   
   # Run migrations
   cargo sqlx migrate run
   ```

3. **Environment Configuration**
   ```bash
   # Copy example configuration
   cp .env.example .env
   
   # Edit configuration
   nano .env
   ```

4. **Build and Run**
   ```bash
   # Build the project
   cargo build --release
   
   # Run the server
   cargo run --release
   ```

### Development Tools

1. **Code Quality Tools**
   ```bash
   # Install Rust tools
   cargo install cargo-watch cargo-audit cargo-outdated
   
   # Run code quality checks
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo audit
   ```

2. **Testing**
   ```bash
   # Run all tests
   cargo test
   
   # Run tests with coverage
   cargo tarpaulin --out Html
   
   # Run integration tests
   cargo test --test '*'
   ```

3. **Documentation**
   ```bash
   # Generate documentation
   cargo doc --open
   
   # Generate API documentation
   cargo run --bin generate_api_docs
   ```

## Coding Standards

### Rust Code Style

1. **Naming Conventions**
   - Use `snake_case` for functions and variables
   - Use `CamelCase` for types and traits
   - Use `SCREAMING_SNAKE_CASE` for constants
   - Use `snake_case` with `_` prefix for private fields

2. **Error Handling**
   - Use `Result<T, E>` for fallible operations
   - Create custom error types with `thiserror`
   - Provide meaningful error messages
   - Use `?` operator for error propagation

3. **Async/Await**
   - Use `async` functions for I/O operations
   - Use `tokio::spawn` for concurrent tasks
   - Use `Arc<Mutex<T>>` for shared state
   - Avoid blocking operations in async code

### Example Code Structure

```rust
use thiserror::Error;
use async_trait::async_trait;

/// Custom error type
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
}

/// Service trait
#[async_trait]
pub trait UserService: Send + Sync {
    async fn create_user(&self, request: CreateUserRequest) -> Result<User, ServiceError>;
    async fn get_user(&self, id: &str) -> Result<Option<User>, ServiceError>;
}

/// Service implementation
pub struct UserServiceImpl {
    repository: Arc<dyn UserRepository>,
    validator: Arc<dyn UserValidator>,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn create_user(&self, request: CreateUserRequest) -> Result<User, ServiceError> {
        // Validate request
        self.validator.validate(&request)
            .map_err(|e| ServiceError::Validation(e.to_string()))?;
        
        // Create user
        let user = User::new(request);
        
        // Save to repository
        self.repository.save(&user).await
            .map_err(|e| ServiceError::Database(e.to_string()))?;
        
        Ok(user)
    }
    
    async fn get_user(&self, id: &str) -> Result<Option<User>, ServiceError> {
        self.repository.find_by_id(id).await
            .map_err(|e| ServiceError::Database(e.to_string()))
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_create_user_success() {
        // Arrange
        let mut mock_repo = MockUserRepository::new();
        let mut mock_validator = MockUserValidator::new();
        
        mock_validator.expect_validate()
            .returning(|_| Ok(()));
        
        mock_repo.expect_save()
            .returning(|_| Ok(()));
        
        let service = UserServiceImpl::new(
            Arc::new(mock_repo),
            Arc::new(mock_validator),
        );
        
        let request = CreateUserRequest {
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // Act
        let result = service.create_user(request).await;
        
        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "test_user");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_user_api_integration() {
    // Setup test database
    let test_db = setup_test_database().await;
    
    // Create test server
    let app = create_test_app(test_db).await;
    
    // Test API endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/users")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"username":"test","email":"test@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

### Performance Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_user_creation(c: &mut Criterion) {
    c.bench_function("create_user", |b| {
        let service = create_user_service();
        let request = create_test_request();
        
        b.iter(|| {
            black_box(service.create_user(request.clone()))
        });
    });
}

criterion_group!(benches, benchmark_user_creation);
criterion_main!(benches);
```

## API Documentation

### REST API Endpoints

The system provides RESTful API endpoints following OpenAPI 3.0 specification:

#### Authentication
- `POST /api/auth/login` - User login
- `POST /api/auth/logout` - User logout
- `POST /api/auth/refresh` - Refresh access token

#### Users
- `GET /api/users` - List users
- `POST /api/users` - Create user
- `GET /api/users/{id}` - Get user by ID
- `PUT /api/users/{id}` - Update user
- `DELETE /api/users/{id}` - Delete user

#### Vehicles
- `GET /api/vehicles` - List vehicles
- `POST /api/vehicles` - Create vehicle
- `GET /api/vehicles/{id}` - Get vehicle by ID
- `PUT /api/vehicles/{id}` - Update vehicle
- `DELETE /api/vehicles/{id}` - Delete vehicle

#### Real-time Tracking
- `GET /api/vehicles/{id}/track` - Get vehicle track
- `GET /api/vehicles/{id}/location` - Get current location
- `GET /api/vehicles/{id}/status` - Get vehicle status

### WebSocket API

Real-time communication using WebSocket:

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
    console.log('Connected to server');
    
    // Subscribe to vehicle updates
    ws.send(JSON.stringify({
        type: 'subscribe',
        vehicle_id: 'vehicle_123'
    }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};
```

## Deployment

### Docker Deployment

1. **Build Docker Image**
   ```bash
   docker build -t CarpTMS:latest .
   ```

2. **Run with Docker Compose**
   ```bash
   docker-compose up -d
   ```

### Kubernetes Deployment

1. **Create Namespace**
   ```bash
   kubectl create namespace CarpTMS
   ```

2. **Apply Configurations**
   ```bash
   kubectl apply -f k8s/configmap.yaml
   kubectl apply -f k8s/secret.yaml
   kubectl apply -f k8s/deployment.yaml
   kubectl apply -f k8s/service.yaml
   kubectl apply -f k8s/ingress.yaml
   ```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| DATABASE_URL | PostgreSQL connection string | - |
| REDIS_URL | Redis connection string | - |
| JWT_SECRET | JWT signing secret | - |
| LOG_LEVEL | Logging level | info |
| SERVER_PORT | HTTP server port | 8080 |
| RUST_LOG | Rust logging configuration | info |

## Monitoring and Observability

### Logging

The system uses structured logging with the Tracing crate:

```rust
use tracing::{info, warn, error, debug};

async fn process_request(&self, request: Request) -> Result<Response, Error> {
    let request_id = Uuid::new_v4();
    
    info!(
        request_id = %request_id,
        method = %request.method(),
        path = %request.path(),
        "Processing request"
    );
    
    match self.handle_request(request).await {
        Ok(response) => {
            info!(
                request_id = %request_id,
                status = %response.status(),
                "Request processed successfully"
            );
            Ok(response)
        }
        Err(error) => {
            error!(
                request_id = %request_id,
                error = %error,
                "Request processing failed"
            );
            Err(error)
        }
    }
}
```

### Metrics

Prometheus metrics are exposed at `/metrics` endpoint:

```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref REQUEST_COUNTER: Counter = Counter::new("requests_total", "Total requests")
        .expect("metric can be created");
    
    static ref REQUEST_DURATION: Histogram = Histogram::new("request_duration_seconds", "Request duration")
        .expect("metric can be created");
}

pub fn register_metrics(registry: &Registry) -> Result<(), PrometheusError> {
    registry.register(Box::new(REQUEST_COUNTER.clone()))?;
    registry.register(Box::new(REQUEST_DURATION.clone()))?;
    Ok(())
}
```

### Health Checks

Health check endpoints:

- `GET /health` - Basic health check
- `GET /health/ready` - Readiness probe
- `GET /health/live` - Liveness probe

### Distributed Tracing

OpenTelemetry integration for distributed tracing:

```rust
use opentelemetry::{global, KeyValue};
use opentelemetry_jaeger::JaegerTraceRuntime;

pub fn init_tracer() -> Result<impl trace::Tracer, TraceError> {
    opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("CarpTMS")
        .with_trace_config(
            trace::config()
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", "CarpTMS"),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
        )
        .install_batch(JaegerTraceRuntime::Tokio)
}
```

## Performance Optimization

### Database Optimization

1. **Indexing Strategy**
   ```sql
   -- Create indexes for common queries
   CREATE INDEX idx_vehicles_device_id ON vehicles(device_id);
   CREATE INDEX idx_tracks_vehicle_id_timestamp ON tracks(vehicle_id, timestamp DESC);
   CREATE INDEX idx_users_email ON users(email);
   ```

2. **Query Optimization**
   ```rust
   // Use prepared statements
   let query = sqlx::query!(
       "SELECT * FROM vehicles WHERE device_id = $1",
       device_id
   );
   
   // Use connection pooling
   let pool = PgPoolOptions::new()
       .max_connections(20)
       .connect(&database_url)
       .await?;
   ```

### Caching Strategy

1. **Multi-level Caching**
   ```rust
   // Memory cache for hot data
   let memory_cache = MemoryCache::new(1000);
   
   // Redis cache for distributed data
   let redis_cache = RedisCache::new(redis_url).await?;
   
   // Database cache for persistent data
   let db_cache = DatabaseCache::new(pool.clone());
   ```

2. **Cache Warming**
   ```rust
   async fn warm_cache(&self) -> Result<(), CacheError> {
       // Pre-load frequently accessed data
       let popular_vehicles = self.get_popular_vehicles().await?;
       
       for vehicle in popular_vehicles {
           self.cache.set(&format!("vehicle:{}", vehicle.id), &vehicle).await?;
       }
       
       Ok(())
   }
   ```

### Concurrency Optimization

1. **Async/Await Patterns**
   ```rust
   // Process multiple requests concurrently
   let futures = requests.into_iter()
       .map(|request| self.process_request(request))
       .collect::<Vec<_>>();
   
   let results = futures::future::join_all(futures).await;
   ```

2. **Parallel Processing**
   ```rust
   use rayon::prelude::*;
   
   // Process large datasets in parallel
   let results: Vec<_> = data
       .par_iter()
       .map(|item| expensive_computation(item))
       .collect();
   ```

## Security Best Practices

### Authentication & Authorization

1. **JWT Token Management**
   ```rust
   // Use secure JWT implementation
   let token = jsonwebtoken::encode(
       &Header::default(),
       &claims,
       &EncodingKey::from_secret(secret.as_bytes()),
   )?;
   
   // Validate tokens properly
   let claims = jsonwebtoken::decode::<Claims>(
       token,
       &DecodingKey::from_secret(secret.as_bytes()),
       &Validation::default(),
   )?;
   ```

2. **Role-Based Access Control**
   ```rust
   pub fn check_permission(&self, user: &User, permission: &str) -> bool {
       user.roles.iter()
           .flat_map(|role| role.permissions.iter())
           .any(|p| p == permission || p == "*")
   }
   ```

### Data Protection

1. **Encryption**
   ```rust
   use ring::aead::{self, AES_256_GCM};
   
   pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
       let unbound_key = aead::UnboundKey::new(&AES_256_GCM, key)?;
       let key = aead::LessSafeKey::new(unbound_key);
       
       let nonce = aead::Nonce::assume_unique_for_key([0; 12]);
       let mut ciphertext = data.to_vec();
       
       key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut ciphertext)?;
       
       Ok(ciphertext)
   }
   ```

2. **Input Validation**
   ```rust
   use validator::{Validate, ValidationError};
   
   #[derive(Debug, Validate)]
   pub struct CreateUserRequest {
       #[validate(length(min = 3, max = 20))]
       pub username: String,
       
       #[validate(email)]
       pub email: String,
       
       #[validate(length(min = 8))]
       pub password: String,
   }
   ```

## Troubleshooting

### Common Issues

1. **Database Connection Issues**
   - Check PostgreSQL is running
   - Verify connection string in configuration
   - Check database permissions

2. **Redis Connection Issues**
   - Ensure Redis server is running
   - Check Redis configuration
   - Verify network connectivity

3. **Compilation Errors**
   - Update Rust toolchain: `rustup update`
   - Clean build cache: `cargo clean`
   - Check dependency versions

### Debugging Techniques

1. **Enable Debug Logging**
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Use Debug Tools**
   ```bash
   # Memory profiling
   valgrind --tool=memcheck ./target/debug/CarpTMS
   
   # Performance profiling
   perf record -g ./target/release/CarpTMS
   perf report
   ```

3. **Database Query Debugging**
   ```rust
   // Enable SQL logging
   sqlx::query!("SELECT * FROM users WHERE id = $1", user_id)
       .fetch_optional(&pool)
       .await?;
   ```

## Contributing

### Development Workflow

1. **Fork the Repository**
2. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Follow coding standards
   - Write tests
   - Update documentation

4. **Run Tests**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Submit Pull Request**
   - Provide clear description
   - Reference related issues
   - Ensure CI passes

### Code Review Guidelines

- Review for correctness and performance
- Check for security vulnerabilities
- Verify test coverage
- Ensure documentation is updated

## Resources

### Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Actix-web Documentation](https://actix.rs/docs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Tokio Documentation](https://tokio.rs/docs/)

### Tools
- [Rust Playground](https://play.rust-lang.org/)
- [Rustlings](https://github.com/rust-lang/rustlings)
- [Clippy](https://github.com/rust-lang/rust-clippy)

### Community
- [Rust Discord](https://discord.gg/rust-lang)
- [Rust Users Forum](https://users.rust-lang.org/)
- [Rust Reddit](https://www.reddit.com/r/rust/)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

