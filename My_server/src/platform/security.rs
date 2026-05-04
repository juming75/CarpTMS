//! Security Management Module
//!
//! Provides unified security services including:
//! - Authentication and authorization
//! - Key management and rotation
//! - Encryption and decryption
//! - CSRF protection
//! - Rate limiting

use crate::utils::key_rotation::KeyRotationManager;
use crate::middleware::csrf_protection::CsrfProtection;
use async_trait::async_trait;
use ring::{aead, rand};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;

/// Security error types
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Authorization failed: {0}")]
    Authorization(String),
    
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    #[error("Key management error: {0}")]
    KeyManagement(String),
    
    #[error("CSRF validation failed: {0}")]
    CsrfValidation(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: Duration,
    pub encryption_key: Option<String>,
    pub csrf_enabled: bool,
    pub rate_limit_enabled: bool,
    pub rate_limit_requests: u32,
    pub rate_limit_window: Duration,
    pub key_rotation_interval: Duration,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "default-secret-change-in-production".to_string(),
            jwt_expiration: Duration::from_secs(3600), // 1 hour
            encryption_key: None,
            csrf_enabled: true,
            rate_limit_enabled: true,
            rate_limit_requests: 100,
            rate_limit_window: Duration::from_secs(60), // 1 minute
            key_rotation_interval: Duration::from_secs(86400), // 24 hours
        }
    }
}

/// Authentication trait
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate a user
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<AuthUser, SecurityError>;
    
    /// Validate a token
    async fn validate_token(&self, token: &str) -> Result<AuthUser, SecurityError>;
    
    /// Generate a token for a user
    async fn generate_token(&self, user: &AuthUser) -> Result<String, SecurityError>;
}

/// Authorization trait
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Check if user has permission
    async fn has_permission(&self, user: &AuthUser, permission: &str) -> Result<bool, SecurityError>;
    
    /// Check if user has role
    async fn has_role(&self, user: &AuthUser, role: &str) -> Result<bool, SecurityError>;
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
    pub device_id: Option<String>,
}

/// Authenticated user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub device_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Encryption provider trait
#[async_trait]
pub trait EncryptionProvider: Send + Sync {
    /// Encrypt data
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;
    
    /// Decrypt data
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError>;
}

/// Ring-based encryption provider
pub struct RingEncryptionProvider {
    key: aead::LessSafeKey,
}

impl RingEncryptionProvider {
    pub fn new(key: &[u8]) -> Result<Self, SecurityError> {
        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|e| SecurityError::KeyManagement(e.to_string()))?;
        
        let key = aead::LessSafeKey::new(unbound_key);
        Ok(Self { key })
    }
}

#[async_trait]
impl EncryptionProvider for RingEncryptionProvider {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let nonce_bytes = rand::generate(&rand::SystemRandom::new())
            .map_err(|e| SecurityError::Encryption(e.to_string()))?;
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes.expose());
        
        let mut ciphertext = data.to_vec();
        self.key
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut ciphertext)
            .map_err(|e| SecurityError::Encryption(e.to_string()))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.expose().to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if data.len() < 12 {
            return Err(SecurityError::Decryption("Invalid ciphertext length".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|e| SecurityError::Decryption(e.to_string()))?;
        
        let mut plaintext = ciphertext.to_vec();
        self.key
            .open_in_place(nonce, aead::Aad::empty(), &mut plaintext)
            .map_err(|e| SecurityError::Decryption(e.to_string()))?;
        
        // Remove authentication tag
        let tag_len = self.key.algorithm().tag_len();
        plaintext.truncate(plaintext.len() - tag_len);
        
        Ok(plaintext)
    }
}

/// JWT-based authenticator
pub struct JwtAuthenticator {
    secret: String,
    expiration: Duration,
}

impl JwtAuthenticator {
    pub fn new(secret: String, expiration: Duration) -> Self {
        Self { secret, expiration }
    }
}

#[async_trait]
impl Authenticator for JwtAuthenticator {
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<AuthUser, SecurityError> {
        // This is a simplified implementation - in production, validate against database
        if credentials.username == "admin" && credentials.password == "admin" {
            Ok(AuthUser {
                id: "1".to_string(),
                username: credentials.username.clone(),
                roles: vec!["admin".to_string()],
                permissions: vec!["*".to_string()],
                device_id: credentials.device_id.clone(),
                created_at: chrono::Utc::now(),
            })
        } else {
            Err(SecurityError::Authentication("Invalid credentials".to_string()))
        }
    }
    
    async fn validate_token(&self, token: &str) -> Result<AuthUser, SecurityError> {
        jsonwebtoken::decode::<AuthUser>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| SecurityError::Authentication(e.to_string()))
    }
    
    async fn generate_token(&self, user: &AuthUser) -> Result<String, SecurityError> {
        let expiration = chrono::Utc::now() + chrono::Duration::from_std(self.expiration)
            .map_err(|e| SecurityError::Authentication(e.to_string()))?;
        
        let claims = jsonwebtoken::claims::Claims {
            sub: Some(user.id.clone()),
            exp: Some(expiration.timestamp() as u64),
            ..Default::default()
        };
        
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| SecurityError::Authentication(e.to_string()))
    }
}

/// Role-based authorizer
pub struct RoleBasedAuthorizer {
    role_permissions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl RoleBasedAuthorizer {
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();
        
        // Default role permissions
        role_permissions.insert("admin".to_string(), vec!["*".to_string()]);
        role_permissions.insert("user".to_string(), vec!["read".to_string(), "write".to_string()]);
        role_permissions.insert("viewer".to_string(), vec!["read".to_string()]);
        
        Self {
            role_permissions: Arc::new(RwLock::new(role_permissions)),
        }
    }
}

#[async_trait]
impl Authorizer for RoleBasedAuthorizer {
    async fn has_permission(&self, user: &AuthUser, permission: &str) -> Result<bool, SecurityError> {
        let role_permissions = self.role_permissions.read().await;
        
        for role in &user.roles {
            if let Some(permissions) = role_permissions.get(role) {
                if permissions.contains(&"*".to_string()) || permissions.contains(&permission.to_string()) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    async fn has_role(&self, user: &AuthUser, role: &str) -> Result<bool, SecurityError> {
        Ok(user.roles.contains(&role.to_string()))
    }
}

/// Security manager that coordinates all security services
pub struct SecurityManager {
    config: SecurityConfig,
    authenticator: Arc<dyn Authenticator>,
    authorizer: Arc<dyn Authorizer>,
    encryption_provider: Option<Arc<dyn EncryptionProvider>>,
    key_rotation_manager: Arc<KeyRotationManager>,
    csrf_protection: Option<Arc<CsrfProtection>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub async fn new(config: Arc<crate::platform::config::ConfigManager>) -> Result<Self, SecurityError> {
        let security_config = config
            .get::<SecurityConfig>("security")
            .await
            .unwrap_or_default();
        
        let authenticator = Arc::new(JwtAuthenticator::new(
            security_config.jwt_secret.clone(),
            security_config.jwt_expiration,
        ));
        
        let authorizer = Arc::new(RoleBasedAuthorizer::new());
        
        let encryption_provider = if let Some(key) = &security_config.encryption_key {
            Some(Arc::new(RingEncryptionProvider::new(key.as_bytes())?) as Arc<dyn EncryptionProvider>)
        } else {
            None
        };
        
        let key_rotation_manager = Arc::new(KeyRotationManager::new(
            security_config.key_rotation_interval,
        ));
        
        let csrf_protection = if security_config.csrf_enabled {
            Some(Arc::new(CsrfProtection::new()))
        } else {
            None
        };
        
        Ok(Self {
            config: security_config,
            authenticator,
            authorizer,
            encryption_provider,
            key_rotation_manager,
            csrf_protection,
        })
    }
    
    /// Authenticate user
    pub async fn authenticate(&self, credentials: &AuthCredentials) -> Result<AuthUser, SecurityError> {
        self.authenticator.authenticate(credentials).await
    }
    
    /// Validate token
    pub async fn validate_token(&self, token: &str) -> Result<AuthUser, SecurityError> {
        self.authenticator.validate_token(token).await
    }
    
    /// Generate token
    pub async fn generate_token(&self, user: &AuthUser) -> Result<String, SecurityError> {
        self.authenticator.generate_token(user).await
    }
    
    /// Check permission
    pub async fn has_permission(&self, user: &AuthUser, permission: &str) -> Result<bool, SecurityError> {
        self.authorizer.has_permission(user, permission).await
    }
    
    /// Check role
    pub async fn has_role(&self, user: &AuthUser, role: &str) -> Result<bool, SecurityError> {
        self.authorizer.has_role(user, role).await
    }
    
    /// Encrypt data
    pub async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if let Some(provider) = &self.encryption_provider {
            provider.encrypt(data).await
        } else {
            Err(SecurityError::Encryption("Encryption not configured".to_string()))
        }
    }
    
    /// Decrypt data
    pub async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if let Some(provider) = &self.encryption_provider {
            provider.decrypt(data).await
        } else {
            Err(SecurityError::Decryption("Encryption not configured".to_string()))
        }
    }
    
    /// Validate CSRF token
    pub async fn validate_csrf(&self, token: &str, session_id: &str) -> Result<bool, SecurityError> {
        if let Some(csrf) = &self.csrf_protection {
            csrf.validate_token(token, session_id)
                .map_err(|e| SecurityError::CsrfValidation(e.to_string()))
        } else {
            Ok(true) // CSRF protection disabled
        }
    }
    
    /// Generate CSRF token
    pub async fn generate_csrf(&self, session_id: &str) -> Result<String, SecurityError> {
        if let Some(csrf) = &self.csrf_protection {
            csrf.generate_token(session_id)
                .map_err(|e| SecurityError::CsrfValidation(e.to_string()))
        } else {
            Err(SecurityError::CsrfValidation("CSRF protection disabled".to_string()))
        }
    }
    
    /// Get key rotation manager
    pub fn key_rotation_manager(&self) -> Arc<KeyRotationManager> {
        self.key_rotation_manager.clone()
    }
}