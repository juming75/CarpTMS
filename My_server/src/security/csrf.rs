//! CSRF Protection Middleware
//! Provides Cross-Site Request Forgery protection

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName, HeaderValue},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use log::debug;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::RwLock;

const CSRF_TOKEN_HEADER: &str = "X-CSRF-Token";
const CSRF_ORIGIN_HEADER: &str = "Origin";
const CSRF_METHODS: [&str; 3] = ["POST", "PUT", "DELETE"];

/// CSRF Token Manager
pub struct CsrfTokenManager {
    tokens: Arc<RwLock<std::collections::HashSet<String>>>,
}

impl CsrfTokenManager {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }

    /// Generate a new CSRF token
    pub async fn generate_token(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH")
            .as_nanos();
        
        let random_part: u64 = rand::random();
        format!("{:x}-{:x}", timestamp, random_part)
    }

    /// Store a token
    pub async fn store_token(&self, token: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.to_string());
    }

    /// Validate a token
    pub async fn validate_token(&self, token: &str) -> bool {
        let tokens = self.tokens.read().await;
        tokens.contains(token)
    }

    /// Remove a used token
    pub async fn remove_token(&self, token: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.remove(token);
    }
}

impl Default for CsrfTokenManager {
    fn default() -> Self {
        Self::new()
    }
}

/// CSRF Protection Middleware Factory
pub struct CsrfProtection {
    token_manager: Arc<CsrfTokenManager>,
    trusted_origins: Vec<String>,
}

impl CsrfProtection {
    pub fn new() -> Self {
        Self {
            token_manager: Arc::new(CsrfTokenManager::new()),
            trusted_origins: vec![],
        }
    }

    pub fn with_trusted_origins(origins: Vec<String>) -> Self {
        Self {
            token_manager: Arc::new(CsrfTokenManager::new()),
            trusted_origins: origins,
        }
    }
}

impl Default for CsrfProtection {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CsrfMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CsrfMiddleware {
            service: Rc::new(service),
            token_manager: self.token_manager.clone(),
            trusted_origins: self.trusted_origins.clone(),
        })
    }
}

pub struct CsrfMiddleware<S> {
    service: Rc<S>,
    token_manager: Arc<CsrfTokenManager>,
    trusted_origins: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for CsrfMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let token_manager = self.token_manager.clone();
        let trusted_origins = self.trusted_origins.clone();

        Box::pin(async move {
            let method = req.method().as_str().to_uppercase();
            
            // Only check CSRF for state-changing methods
            if CSRF_METHODS.contains(&method.as_str()) {
                // Check for CSRF token header
                let csrf_token = req
                    .headers()
                    .get(CSRF_TOKEN_HEADER)
                    .and_then(|v| v.to_str().ok());

                // Check for Origin header
                let origin = req
                    .headers()
                    .get(CSRF_ORIGIN_HEADER)
                    .and_then(|v| v.to_str().ok());

                // If Origin is present, validate it
                if let Some(origin) = origin {
                    if !trusted_origins.is_empty() && !trusted_origins.contains(&origin.to_string()) {
                        debug!("CSRF check failed: untrusted origin {}", origin);
                        return Ok(req.into_response(
                            HttpResponse::Forbidden()
                                .content_type("application/json")
                                .json(serde_json::json!({"error": "CSRF validation failed: untrusted origin"}))
                        ).map_into_right_body());
                    }
                }

                // Validate CSRF token
                if let Some(token) = csrf_token {
                    if !token_manager.validate_token(token).await {
                        debug!("CSRF check failed: invalid token");
                        return Ok(req.into_response(
                            HttpResponse::Forbidden()
                                .content_type("application/json")
                                .json(serde_json::json!({"error": "CSRF validation failed: invalid token"}))
                        ).map_into_right_body());
                    }
                } else {
                    // No token provided - check if Origin is trusted
                    if origin.is_none() || trusted_origins.is_empty() {
                        debug!("CSRF check failed: missing token and no trusted origin");
                        return Ok(req.into_response(
                            HttpResponse::Forbidden()
                                .content_type("application/json")
                                .json(serde_json::json!({"error": "CSRF validation failed: missing token"}))
                        ).map_into_right_body());
                    }
                }
            }

            service.call(req).await.map(|res| res.map_into_left_body())
        })
    }
}

/// CSRF Protection Helper Functions
pub mod helpers {
    use super::*;

    /// Generate a CSRF token and store it
    pub async fn generate_and_store_csrf_token(manager: &CsrfTokenManager) -> String {
        let token = manager.generate_token().await;
        manager.store_token(&token).await;
        token
    }

    /// Extract CSRF token from request headers
    pub fn extract_csrf_token(req: &actix_web::HttpRequest) -> Option<String> {
        req.headers()
            .get(CSRF_TOKEN_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
}
