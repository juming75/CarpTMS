//! / 密码加密工具模块

use anyhow::{anyhow, Result};
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::{self, Argon2, PasswordHash, PasswordVerifier};

/// 密码强度验证结果
#[derive(Debug, Clone, Default)]
pub struct PasswordStrength {
    /// 是否有效
    pub is_valid: bool,
    /// 强度等级: 0=无效, 1=弱, 2=中等, 3=强
    pub level: u8,
    /// 验证错误信息
    pub errors: Vec<String>,
}

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
    let parsed_hash =
        PasswordHash::new(hashed_password).map_err(|e| anyhow!("Invalid password hash: {}", e))?;
    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map(|_| true)
        .map_err(|e| anyhow!("Password verification failed: {}", e))
}

/// 验证密码强度
///
/// 要求:
/// - 长度至少8位
/// - 包含大写字母
/// - 包含小写字母
/// - 包含数字
/// - 包含特殊字符 (!@#$%^&*()_+-=[]{}|;:,.<>?)
///
/// 返回 PasswordStrength 结构，包含是否有效、强度等级和错误信息
pub fn validate_password_strength(password: &str) -> PasswordStrength {
    let mut errors = Vec::new();
    let mut score = 0u8;

    // 检查长度
    if password.len() < 8 {
        errors.push("密码长度至少需要8位".to_string());
    } else {
        score += 1;
    }

    // 检查大写字母
    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("密码必须包含至少一个大写字母".to_string());
    } else {
        score += 1;
    }

    // 检查小写字母
    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("密码必须包含至少一个小写字母".to_string());
    } else {
        score += 1;
    }

    // 检查数字
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("密码必须包含至少一个数字".to_string());
    } else {
        score += 1;
    }

    // 检查特殊字符
    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    if !password.chars().any(|c| special_chars.contains(c)) {
        errors.push("密码必须包含至少一个特殊字符 (!@#$%^&*等)".to_string());
    } else {
        score += 1;
    }

    // 额外检查：长度超过12位增加强度
    if password.len() >= 12 {
        score += 1;
    }

    // 计算强度等级
    let level = match score {
        0..=2 => 1, // 弱
        3..=4 => 2, // 中等
        _ => 3,     // 强
    };

    PasswordStrength {
        is_valid: errors.is_empty(),
        level,
        errors,
    }
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
        tracing::debug!(hashed = %hashed, "密码哈希完成");

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

        tracing::debug!(hashed1 = %hashed1, hashed2 = %hashed2, "不同密码哈希结果");

        assert_ne!(hashed1, hashed2, "不同密码应该生成不同哈希");
    }

    #[test]
    fn test_hash_same_password_multiple_times() {
        // 测试相同密码多次哈希生成不同结果(因为Argon2包含随机盐)
        let password = "same_password";

        let hashed1 = hash_password(password).unwrap();
        let hashed2 = hash_password(password).unwrap();

        tracing::debug!(hashed1 = %hashed1, hashed2 = %hashed2, "相同密码多次哈希结果");

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

        tracing::debug!(length = hashed.len(), "哈希长度");
        assert!(hashed.len() > 100, "哈希长度应该足够长");
        assert!(hashed.starts_with("$argon2id$"), "哈希应该以$argon2id$开头");
    }
}
