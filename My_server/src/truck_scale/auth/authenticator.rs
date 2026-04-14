//! / 认证器
use anyhow::Result;
use argon2::Argon2;

/// 认证器
pub struct Authenticator;

impl Authenticator {
    /// 创建新的认证器
    pub fn new() -> Self {
        Self
    }

    /// 哈希密码
    pub fn hash_password(&self, password: &str) -> Result<String> {
        // 使用argon2::password_hash::PasswordHasher接口
        use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Password hashing error: {}", e))?;
        Ok(hashed.to_string())
    }

    /// 验证密码
    pub fn verify_password(&self, password: &str, hashed: &str) -> Result<bool> {
        // 使用argon2::password_hash::PasswordVerifier接口
        use argon2::password_hash::{PasswordHash, PasswordVerifier};

        let parsed_hash = PasswordHash::new(hashed)
            .map_err(|e| anyhow::anyhow!("Password hash parsing error: {}", e))?;
        let argon2 = Argon2::default();
        let is_valid = argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();
        Ok(is_valid)
    }

    /// 认证用户
    pub fn authenticate(&self, _username: &str, password: &str, stored_hash: &str) -> Result<bool> {
        let is_valid = self.verify_password(password, stored_hash)?;
        Ok(is_valid)
    }
}

impl Default for Authenticator {
    fn default() -> Self {
        Self::new()
    }
}
