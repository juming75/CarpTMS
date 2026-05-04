use ring::{
    aead::{self, Aad, LessSafeKey, Nonce, UnboundKey},
    hkdf::{self, Salt},
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use tracing::{error, info};

/// 密钥条目类型别名
type KeyEntry = (Vec<u8>, KeyMetadata);
/// 活跃密钥映射类型别名
type ActiveKeysMap = HashMap<String, KeyEntry>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub algorithm: String,
    pub usage_count: u64,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct KeyRotationConfig {
    pub rotation_interval: Duration,
    pub key_lifetime: Duration,
    pub grace_period: Duration,
    pub key_length: usize,
    pub algorithm: &'static str,
    pub max_usage_count: u64,
    pub enable_audit_log: bool,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            rotation_interval: Duration::from_secs(24 * 60 * 60), // 24 hours
            key_lifetime: Duration::from_secs(7 * 24 * 60 * 60),  // 7 days
            grace_period: Duration::from_secs(2 * 60 * 60),       // 2 hours
            key_length: 32,
            algorithm: "AES_256_GCM",
            max_usage_count: 10000,
            enable_audit_log: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyRotationManager {
    config: KeyRotationConfig,
    active_keys: Arc<RwLock<ActiveKeysMap>>,
    current_key_id: Arc<RwLock<String>>,
    audit_log: Arc<RwLock<Vec<AuditEvent>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub key_id: String,
    pub details: String,
}

impl KeyRotationManager {
    pub fn new(config: KeyRotationConfig) -> Self {
        let current_key_id = Arc::new(RwLock::new(String::new()));
        let active_keys = Arc::new(RwLock::new(HashMap::new()));
        let audit_log = Arc::new(RwLock::new(Vec::new()));

        Self {
            config,
            active_keys,
            current_key_id,
            audit_log,
        }
    }

    pub async fn initialize(&self) -> Result<String, KeyRotationError> {
        let key_id = self.generate_and_store_key().await?;
        *self.current_key_id.write().await = key_id.clone();

        self.log_audit_event("key_generation", &key_id, "Initial key generated")
            .await;
        info!("Key rotation manager initialized with key: {}", key_id);

        Ok(key_id)
    }

    pub async fn rotate_keys(&self) -> Result<String, KeyRotationError> {
        let new_key_id = self.generate_and_store_key().await?;
        let old_key_id = {
            let mut current = self.current_key_id.write().await;
            let old = current.clone();
            *current = new_key_id.clone();
            old
        };

        // Mark old key as inactive after grace period
        let grace_period = self.config.grace_period;
        let active_keys = self.active_keys.clone();
        let old_key_id_clone = old_key_id.clone();

        tokio::spawn(async move {
            tokio::time::sleep(grace_period).await;
            if let Some((_, ref mut metadata)) =
                active_keys.write().await.get_mut(&old_key_id_clone)
            {
                metadata.active = false;
            }
        });

        // Clean up expired keys
        self.cleanup_expired_keys().await?;

        self.log_audit_event(
            "key_rotation",
            &new_key_id,
            &format!("Rotated from key: {}", old_key_id),
        )
        .await;
        info!("Key rotated from {} to {}", old_key_id, new_key_id);

        Ok(new_key_id)
    }

    pub async fn get_current_key(&self) -> Result<(Vec<u8>, KeyMetadata), KeyRotationError> {
        let key_id = self.current_key_id.read().await.clone();

        if key_id.is_empty() {
            return Err(KeyRotationError::NoActiveKey);
        }

        let mut active_keys = self.active_keys.write().await;

        if let Some((key_data, ref mut metadata)) = active_keys.get_mut(&key_id) {
            // Check if key is expired
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is before UNIX epoch")
                .as_secs();

            if metadata.expires_at <= now {
                return Err(KeyRotationError::KeyExpired(key_id));
            }

            // Check usage count
            if metadata.usage_count >= self.config.max_usage_count {
                return Err(KeyRotationError::KeyUsageExceeded(key_id));
            }

            metadata.usage_count += 1;
            Ok((key_data.clone(), metadata.clone()))
        } else {
            Err(KeyRotationError::KeyNotFound(key_id))
        }
    }

    pub async fn get_key_by_id(
        &self,
        key_id: &str,
    ) -> Result<(Vec<u8>, KeyMetadata), KeyRotationError> {
        let mut active_keys = self.active_keys.write().await;

        if let Some((key_data, ref mut metadata)) = active_keys.get_mut(key_id) {
            metadata.usage_count += 1;
            Ok((key_data.clone(), metadata.clone()))
        } else {
            Err(KeyRotationError::KeyNotFound(key_id.to_string()))
        }
    }

    pub async fn encrypt_data(&self, data: &[u8]) -> Result<EncryptedData, KeyRotationError> {
        let (key_data, metadata) = self.get_current_key().await?;

        let encrypted = self.encrypt_with_key(&key_data, data)?;

        Ok(EncryptedData {
            ciphertext: encrypted,
            key_id: metadata.id.clone(),
            algorithm: metadata.algorithm.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is before UNIX epoch")
                .as_secs(),
        })
    }

    pub async fn decrypt_data(
        &self,
        encrypted_data: &EncryptedData,
    ) -> Result<Vec<u8>, KeyRotationError> {
        let (key_data, _) = self.get_key_by_id(&encrypted_data.key_id).await?;

        self.decrypt_with_key(&key_data, &encrypted_data.ciphertext)
    }

    pub async fn schedule_key_rotation(self: Arc<Self>) {
        let rotation_interval = self.config.rotation_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(rotation_interval);

            loop {
                interval.tick().await;

                if let Err(e) = self.rotate_keys().await {
                    error!("Failed to rotate keys: {}", e);
                }
            }
        });
    }

    pub async fn get_audit_log(&self) -> Vec<AuditEvent> {
        self.audit_log.read().await.clone()
    }

    pub async fn get_key_statistics(&self) -> HashMap<String, KeyMetadata> {
        let active_keys = self.active_keys.read().await;
        active_keys
            .iter()
            .map(|(id, (_, metadata))| (id.clone(), metadata.clone()))
            .collect()
    }

    async fn generate_and_store_key(&self) -> Result<String, KeyRotationError> {
        let random = SystemRandom::new();
        let mut key_bytes = vec![0u8; self.config.key_length];

        random
            .fill(&mut key_bytes)
            .map_err(|_| KeyRotationError::KeyGenerationFailed)?;

        let key_id = self.generate_key_id();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before UNIX epoch")
            .as_secs();

        let metadata = KeyMetadata {
            id: key_id.clone(),
            created_at: now,
            expires_at: now + self.config.key_lifetime.as_secs(),
            algorithm: self.config.algorithm.to_string(),
            usage_count: 0,
            active: true,
        };

        let mut active_keys = self.active_keys.write().await;
        active_keys.insert(key_id.clone(), (key_bytes, metadata));

        Ok(key_id)
    }

    async fn cleanup_expired_keys(&self) -> Result<(), KeyRotationError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before UNIX epoch")
            .as_secs();

        let mut active_keys = self.active_keys.write().await;
        let mut keys_to_remove = Vec::new();

        for (key_id, (_, metadata)) in active_keys.iter() {
            if metadata.expires_at <= now && !metadata.active {
                keys_to_remove.push(key_id.clone());
            }
        }

        for key_id in keys_to_remove {
            active_keys.remove(&key_id);
            self.log_audit_event("key_cleanup", &key_id, "Expired key removed")
                .await;
        }

        Ok(())
    }

    fn generate_key_id(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before UNIX epoch")
            .as_secs();

        format!("key_{}", timestamp)
    }

    async fn log_audit_event(&self, event_type: &str, key_id: &str, details: &str) {
        if !self.config.enable_audit_log {
            return;
        }

        let event = AuditEvent {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is before UNIX epoch")
                .as_secs(),
            event_type: event_type.to_string(),
            key_id: key_id.to_string(),
            details: details.to_string(),
        };

        let mut audit_log = self.audit_log.write().await;
        audit_log.push(event);

        // Keep only last 1000 events
        if audit_log.len() > 1000 {
            let new_len = audit_log.len() - 1000;
            audit_log.drain(0..new_len);
        }
    }

    fn encrypt_with_key(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>, KeyRotationError> {
        // Use HKDF to derive encryption key from master key
        let salt = Salt::new(hkdf::HKDF_SHA256, b"encryption_salt");
        let prk = salt.extract(key);

        let mut encryption_key = vec![0u8; 32];
        prk.expand(&[], &aead::AES_256_GCM)
            .map_err(|_| KeyRotationError::EncryptionFailed)?
            .fill(&mut encryption_key)
            .map_err(|_| KeyRotationError::EncryptionFailed)?;

        // Use AES-GCM for encryption
        let unbound_key = UnboundKey::new(&aead::AES_256_GCM, &encryption_key)
            .map_err(|_| KeyRotationError::EncryptionFailed)?;

        let key = LessSafeKey::new(unbound_key);

        // Generate random nonce
        let random = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        random
            .fill(&mut nonce_bytes)
            .map_err(|_| KeyRotationError::EncryptionFailed)?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = data.to_vec();
        let tag = key
            .seal_in_place_separate_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| KeyRotationError::EncryptionFailed)?;

        // Prepend nonce to ciphertext for storage
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&in_out);
        result.extend_from_slice(tag.as_ref());

        Ok(result)
    }

    fn decrypt_with_key(
        &self,
        key: &[u8],
        encrypted_data: &[u8],
    ) -> Result<Vec<u8>, KeyRotationError> {
        if encrypted_data.len() < 12 {
            return Err(KeyRotationError::DecryptionFailed);
        }

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext_and_tag) = encrypted_data.split_at(12);
        let nonce =
            Nonce::assume_unique_for_key(nonce_bytes.try_into().expect("nonce must be 12 bytes"));

        // Use HKDF to derive encryption key from master key
        let salt = Salt::new(hkdf::HKDF_SHA256, b"encryption_salt");
        let prk = salt.extract(key);

        let mut encryption_key = vec![0u8; 32];
        prk.expand(&[], &aead::AES_256_GCM)
            .map_err(|_| KeyRotationError::DecryptionFailed)?
            .fill(&mut encryption_key)
            .map_err(|_| KeyRotationError::DecryptionFailed)?;

        // Use AES-GCM for decryption
        let unbound_key = UnboundKey::new(&aead::AES_256_GCM, &encryption_key)
            .map_err(|_| KeyRotationError::DecryptionFailed)?;

        let key = LessSafeKey::new(unbound_key);

        let mut in_out = ciphertext_and_tag.to_vec();
        let plaintext = key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| KeyRotationError::DecryptionFailed)?;

        Ok(plaintext.to_vec())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub key_id: String,
    pub algorithm: String,
    pub timestamp: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum KeyRotationError {
    #[error("No active key available")]
    NoActiveKey,

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Key expired: {0}")]
    KeyExpired(String),

    #[error("Key usage count exceeded: {0}")]
    KeyUsageExceeded(String),

    #[error("Key generation failed")]
    KeyGenerationFailed,

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Database error: {0}")]
    DatabaseError(String),
}

// Convenience functions for common operations
pub async fn create_key_rotation_manager() -> Arc<KeyRotationManager> {
    let config = KeyRotationConfig::default();
    let manager = Arc::new(KeyRotationManager::new(config));

    match manager.initialize().await {
        Ok(key_id) => {
            info!("Key rotation manager initialized with key: {}", key_id);
            let mgr = manager.clone();
            tokio::spawn(async move {
                let _ = mgr.schedule_key_rotation().await;
            });
            manager
        }
        Err(e) => {
            error!("Failed to initialize key rotation manager: {}", e);
            panic!("Key rotation manager initialization failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_generation() {
        let config = KeyRotationConfig::default();
        let manager = KeyRotationManager::new(config);

        let key_id = manager.initialize().await.unwrap();
        assert!(!key_id.is_empty());

        let stats = manager.get_key_statistics().await;
        assert_eq!(stats.len(), 1);
        assert!(stats.contains_key(&key_id));
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let config = KeyRotationConfig {
            rotation_interval: Duration::from_secs(1), // Fast rotation for testing
            ..Default::default()
        };

        let manager = KeyRotationManager::new(config);
        let initial_key = manager.initialize().await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let new_key = manager.rotate_keys().await.unwrap();
        assert_ne!(initial_key, new_key);

        let current_key = manager.get_current_key().await.unwrap();
        assert_eq!(current_key.1.id, new_key);
    }

    #[tokio::test]
    async fn test_encryption_decryption() {
        let manager = Arc::new(KeyRotationManager::new(KeyRotationConfig::default()));
        manager.initialize().await.unwrap();

        let test_data = b"Hello, World! This is a test message.";

        let encrypted = manager.encrypt_data(test_data).await.unwrap();
        assert_ne!(encrypted.ciphertext, test_data.to_vec());

        let decrypted = manager.decrypt_data(&encrypted).await.unwrap();
        assert_eq!(decrypted, test_data);
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let manager = KeyRotationManager::new(KeyRotationConfig::default());
        manager.initialize().await.unwrap();

        let audit_log = manager.get_audit_log().await;
        assert!(!audit_log.is_empty());
        assert_eq!(audit_log[0].event_type, "key_generation");
    }
}
