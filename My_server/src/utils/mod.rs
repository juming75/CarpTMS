pub mod audit;
pub mod backup;
pub mod encryption;
pub mod jwt;
pub mod key_rotation;
pub mod log;
pub mod password;
pub mod permissions;
pub mod secret_manager;
pub mod serialization;
pub mod validation;

pub use backup::{BackupConfig, BackupError, BackupInfo, BackupManager, BackupStatus};

pub use key_rotation::{
    create_key_rotation_manager, EncryptedData, KeyMetadata, KeyRotationConfig, KeyRotationError,
    KeyRotationManager,
};

pub use encryption::{decrypt_string, encrypt_string, get_encryption_key, EncryptionError};

pub use password::{hash_password, verify_password};

pub use jwt::{
    generate_access_token, generate_refresh_token, generate_token, get_jwt_algorithm,
    verify_refresh_token, verify_token, Claims, RefreshClaims,
};

pub use permissions::{action_from_str, has_permission, resource_from_str, Action, Resource, Role};

pub use log::{init_logging, load_log_config_from_env, LogConfig, LogContextManager};

pub use secret_manager::{SecretError, SecretManager};

pub use audit::{log_audit_event, AuditLogRecord, AuditLogSearchParams, PaginationParams};

pub use serialization::{
    binary_deserialize, binary_serialize, json_deserialize, json_serialize, SerializationFormat,
    Serializer,
};

pub use validation::{rules, ValidationExt, ValidationUtils};

/// 缓存操作错误日志辅助函数
/// 缓存操作是 best-effort，失败时记录 warning 但不中断请求
#[allow(dead_code)]
pub fn log_cache_error<T, E: std::fmt::Display>(result: Result<T, E>, op: &str) {
    if let Err(e) = result {
        tracing::warn!(operation = %op, error = %e, "缓存操作失败");
    }
}
