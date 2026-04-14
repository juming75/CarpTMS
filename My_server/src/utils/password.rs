//! / 密码加密工具模块

use anyhow::{anyhow, Result};
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::{self, Argon2, PasswordHash, PasswordVerifier};

/// 密码哈希函数
/// 使用Argon2id算法,提供高安全性
pub fn hash_password(password: &str) -> Result<String> {
    // 使用默认配置,提供安全的参数
    let argon2 = Argon2::default();

    // 生成随机盐
    let salt = SaltString::generate(&mut rand::thread_rng());

    // 哈希密码
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("Password hashing failed: {}", e))?;
    Ok(password_hash.to_string())
}

/// 密码验证函数
/// 验证密码与哈希值是否匹配
#[allow(dead_code)]
pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool> {
    // 只支持Argon2哈希验证
    let parsed_hash = PasswordHash::new(hashed_password)
        .map_err(|e| anyhow!("Invalid password hash: {}", e))?;
    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map(|_| true)
        .map_err(|e| anyhow!("Password verification failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        // 测试密码哈希和验证
        let password = "test_password123";

        // 哈希密码
        let hashed = hash_password(password).unwrap();
        println!("Hashed password: {}", hashed);

        // 验证正确密码
        let result = verify_password(password, &hashed).unwrap();
        assert!(result, "密码验证应该成功");

        // 验证错误密码
        let wrong_password = "wrong_password";
        let result = verify_password(wrong_password, &hashed).unwrap();
        assert!(!result, "错误密码验证应该失败");
    }

    #[test]
    fn test_hash_password_with_different_cost() {
        // 测试不同密码生成不同哈希
        let password1 = "password1";
        let password2 = "password2";

        let hashed1 = hash_password(password1).unwrap();
        let hashed2 = hash_password(password2).unwrap();

        println!("Hashed password 1: {}", hashed1);
        println!("Hashed password 2: {}", hashed2);

        assert_ne!(hashed1, hashed2, "不同密码应该生成不同哈希");
    }

    #[test]
    fn test_hash_same_password_multiple_times() {
        // 测试相同密码多次哈希生成不同结果(因为Argon2包含随机盐)
        let password = "same_password";

        let hashed1 = hash_password(password).unwrap();
        let hashed2 = hash_password(password).unwrap();

        println!("Hashed password 1: {}", hashed1);
        println!("Hashed password 2: {}", hashed2);

        assert_ne!(hashed1, hashed2, "相同密码多次哈希应该生成不同结果");

        // 但两者都应该能够验证通过
        assert!(verify_password(password, &hashed1).unwrap());
        assert!(verify_password(password, &hashed2).unwrap());
    }

    #[test]
    fn test_verify_with_invalid_hash() {
        // 测试使用无效哈希值进行验证
        let password = "test_password";
        let invalid_hash = "invalid_argon2_hash";

        let result = verify_password(password, invalid_hash);
        assert!(result.is_err(), "无效哈希验证应该失败");
    }

    #[test]
    fn test_hash_complexity() {
        // 测试哈希长度和格式
        let password = "test_password";
        let hashed = hash_password(password).unwrap();

        println!("Hashed password length: {}", hashed.len());
        assert!(hashed.len() > 100, "哈希长度应该足够长");
        assert!(hashed.starts_with("$argon2id$"), "哈希应该以$argon2id$开头");
    }
}
