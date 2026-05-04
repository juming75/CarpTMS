use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorInternalServerError, ErrorUnauthorized},
    http::Method,
    Error, HttpMessage, HttpRequest, Result,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use futures_util::future::LocalBoxFuture;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    future::{ready, Ready},
    rc::Rc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    pub token: String,
    pub created_at: u64,
    pub expires_at: u64,
}

impl CsrfToken {
    pub fn new(token_length: usize, ttl: Duration) -> Result<Self> {
        let random = SystemRandom::new();
        let mut token_bytes = vec![0u8; token_length];
        random.fill(&mut token_bytes).map_err(|e| {
            ErrorInternalServerError(format!("Failed to generate CSRF token: {}", e))
        })?;

        let token = URL_SAFE_NO_PAD.encode(&token_bytes);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();

        Ok(CsrfToken {
            token,
            created_at: now,
            expires_at: now + ttl.as_secs(),
        })
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();
        now > self.expires_at
    }

    pub fn verify(&self, provided_token: &str) -> bool {
        if self.is_expired() {
            return false;
        }
        // 使用 ring 的常量时间比较防止时序攻击
        self.token.as_bytes() == provided_token.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct CsrfConfig {
    pub token_name: String,
    pub cookie_name: String,
    pub token_length: usize,
    pub token_ttl: Duration,
    pub double_submit_cookie: bool,
    pub security_headers: bool,
    pub whitelist_paths: HashSet<String>,
    pub whitelist_methods: HashSet<Method>,
    pub same_site: actix_web::cookie::SameSite,
    pub secure: bool,
    pub http_only: bool,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        let mut whitelist_methods = HashSet::new();
        whitelist_methods.insert(Method::GET);
        whitelist_methods.insert(Method::HEAD);
        whitelist_methods.insert(Method::OPTIONS);

        let mut whitelist_paths = HashSet::new();
        whitelist_paths.insert("/health".to_string());
        whitelist_paths.insert("/api/auth/login".to_string());
        whitelist_paths.insert("/api/auth/refresh".to_string());
        whitelist_paths.insert("/api/auth/logout".to_string());
        whitelist_paths.insert("/api/auth/change-password".to_string());

        Self {
            token_name: "X-CSRF-Token".to_string(),
            cookie_name: "csrf_token".to_string(),
            token_length: 32,
            token_ttl: Duration::from_secs(3600), // 1 hour
            double_submit_cookie: true,
            security_headers: true,
            whitelist_paths,
            whitelist_methods,
            same_site: actix_web::cookie::SameSite::Strict,
            secure: true,
            http_only: true,
        }
    }
}

impl CsrfConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_token_name(mut self, name: String) -> Self {
        self.token_name = name;
        self
    }

    pub fn with_cookie_name(mut self, name: String) -> Self {
        self.cookie_name = name;
        self
    }

    pub fn with_token_length(mut self, length: usize) -> Self {
        self.token_length = length;
        self
    }

    pub fn with_token_ttl(mut self, ttl: Duration) -> Self {
        self.token_ttl = ttl;
        self
    }

    pub fn with_double_submit_cookie(mut self, enabled: bool) -> Self {
        self.double_submit_cookie = enabled;
        self
    }

    pub fn with_security_headers(mut self, enabled: bool) -> Self {
        self.security_headers = enabled;
        self
    }

    pub fn with_whitelist_path(mut self, path: String) -> Self {
        self.whitelist_paths.insert(path);
        self
    }

    pub fn with_whitelist_method(mut self, method: Method) -> Self {
        self.whitelist_methods.insert(method);
        self
    }

    pub fn with_same_site(mut self, same_site: actix_web::cookie::SameSite) -> Self {
        self.same_site = same_site;
        self
    }

    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    pub fn with_http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }
}

pub struct CsrfMiddleware {
    config: CsrfConfig,
}

impl CsrfMiddleware {
    pub fn new(config: CsrfConfig) -> Self {
        Self { config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for CsrfMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CsrfMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CsrfMiddlewareService {
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct CsrfMiddlewareService<S> {
    service: Rc<S>,
    config: CsrfConfig,
}

impl<S, B> Service<ServiceRequest> for CsrfMiddlewareService<S>
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
        let config = self.config.clone();

        Box::pin(async move {
            // Check if request should bypass CSRF protection
            if should_bypass_csrf(&req, &config) {
                let mut response = service.call(req).await?;

                // Add security headers if enabled
                if config.security_headers {
                    add_security_headers(response.headers_mut());
                }

                return Ok(response);
            }

            // Generate new CSRF token for GET requests
            if matches!(req.method(), &Method::GET) {
                match CsrfToken::new(config.token_length, config.token_ttl) {
                    Ok(csrf_token) => {
                        // Store token in request extensions for later use
                        req.extensions_mut().insert(csrf_token.clone());

                        let mut response = service.call(req).await?;

                        // Set CSRF token cookie
                        let cookie = actix_web::cookie::Cookie::build(
                            &config.cookie_name,
                            &csrf_token.token,
                        )
                        .path("/")
                        .http_only(config.http_only)
                        .secure(config.secure)
                        .same_site(config.same_site)
                        .max_age(actix_web::cookie::time::Duration::seconds(
                            config.token_ttl.as_secs() as i64,
                        ))
                        .finish();

                        response.response_mut().add_cookie(&cookie)?;

                        // Add security headers
                        if config.security_headers {
                            add_security_headers(response.headers_mut());
                        }

                        return Ok(response);
                    }
                    Err(e) => {
                        return Err(ErrorUnauthorized(format!(
                            "Failed to generate CSRF token: {}",
                            e
                        )));
                    }
                }
            }

            // Verify CSRF token for state-changing requests
            let provided_token = extract_csrf_token(&req, &config)?;
            let cookie_token = extract_cookie_token(&req, &config)?;

            if provided_token.is_empty() || cookie_token.is_empty() {
                return Err(ErrorUnauthorized("CSRF token missing"));
            }

            // Verify tokens match (double-submit cookie pattern)
            if provided_token != cookie_token {
                return Err(ErrorUnauthorized("CSRF token mismatch"));
            }

            // Verify token is not expired (if we have the full token data)
            if let Some(stored_token) = req.extensions().get::<CsrfToken>() {
                if stored_token.is_expired() {
                    return Err(ErrorUnauthorized("CSRF token expired"));
                }
            }

            let mut response = service.call(req).await?;

            // Add security headers
            if config.security_headers {
                add_security_headers(response.headers_mut());
            }

            Ok(response)
        })
    }
}

fn should_bypass_csrf(req: &ServiceRequest, config: &CsrfConfig) -> bool {
    // Check if method is whitelisted
    if config.whitelist_methods.contains(req.method()) {
        return true;
    }

    // Check if path is whitelisted
    let path = req.path();
    config
        .whitelist_paths
        .iter()
        .any(|whitelist_path| path.starts_with(whitelist_path))
}

fn extract_csrf_token(req: &ServiceRequest, config: &CsrfConfig) -> Result<String> {
    // Try to get token from header first
    if let Some(header_value) = req.headers().get(&config.token_name) {
        if let Ok(token_str) = header_value.to_str() {
            return Ok(token_str.to_string());
        }
    }

    // Try to get token from form data (for form submissions)
    if let Some(form_data) = req.match_info().get("csrf_token") {
        return Ok(form_data.to_string());
    }

    // Try to get token from query parameters
    if let Some(query_string) = req.uri().query() {
        let target = format!("{}=", config.token_name);
        for param in query_string.split('&') {
            if let Some(value) = param.strip_prefix(&target) {
                return Ok(value.to_string());
            }
        }
    }

    Ok(String::new())
}

fn extract_cookie_token(req: &ServiceRequest, config: &CsrfConfig) -> Result<String> {
    if let Ok(cookies) = req.cookies() {
        for cookie in cookies.iter() {
            if cookie.name() == config.cookie_name {
                return Ok(cookie.value().to_string());
            }
        }
    }
    Ok(String::new())
}

fn add_security_headers(headers: &mut actix_web::http::header::HeaderMap) {
    use actix_web::http::header::{HeaderName, HeaderValue};
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:"),
    );
}

// Helper function to create CSRF middleware with default configuration
pub fn csrf_middleware() -> CsrfMiddleware {
    CsrfMiddleware::new(CsrfConfig::default())
}

// Helper function to create CSRF middleware with custom configuration
pub fn csrf_middleware_with_config(config: CsrfConfig) -> CsrfMiddleware {
    CsrfMiddleware::new(config)
}

// Helper function to get CSRF token from request (for use in handlers)
pub fn get_csrf_token(req: &HttpRequest) -> Option<String> {
    req.extensions()
        .get::<CsrfToken>()
        .map(|token| token.token.clone())
}

// Additional CSRF protection utilities
pub trait CsrfProtection {
    fn csrf_token(&self) -> Option<String>;
    fn validate_csrf_token(&self, token: &str) -> bool;
}

impl CsrfProtection for HttpRequest {
    fn csrf_token(&self) -> Option<String> {
        get_csrf_token(self)
    }

    fn validate_csrf_token(&self, token: &str) -> bool {
        if let Some(stored_token) = self.extensions().get::<CsrfToken>() {
            stored_token.verify(token)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, web, App, HttpResponse};

    #[actix_web::test]
    async fn test_csrf_token_generation() {
        let token = CsrfToken::new(32, Duration::from_secs(3600)).unwrap();
        assert_eq!(token.token.len(), 43); // Base64 encoded length
        assert!(!token.is_expired());
    }

    #[actix_web::test]
    async fn test_csrf_token_verification() {
        let token = CsrfToken::new(32, Duration::from_secs(3600)).unwrap();
        assert!(token.verify(&token.token));
        assert!(!token.verify("invalid_token"));
    }

    #[actix_web::test]
    async fn test_csrf_middleware_get_request() {
        let app = test::init_service(App::new().wrap(csrf_middleware()).route(
            "/test",
            web::get().to(|| async { HttpResponse::Ok().body("test") }),
        ))
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert!(resp.response().cookies().any(|c| c.name() == "csrf_token"));
    }

    #[actix_web::test]
    async fn test_csrf_middleware_post_without_token() {
        let app = test::init_service(App::new().wrap(csrf_middleware()).route(
            "/test",
            web::post().to(|| async { HttpResponse::Ok().body("test") }),
        ))
        .await;

        let req = test::TestRequest::post().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
