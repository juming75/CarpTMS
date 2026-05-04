//! 多因素认证 (MFA) 模块
//! 支持手机验证码、邮箱验证码、TOTP 令牌

use base32::Alphabet;
use chrono::{Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// MFA 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MfaType {
    /// 手机验证码
    Sms,
    /// 邮箱验证码
    Email,
    /// TOTP 令牌
    Totp,
}

/// MFA 验证状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MfaStatus {
    /// 待验证
    Pending,
    /// 已验证
    Verified,
    /// 已过期
    Expired,
    /// 已失败
    Failed,
}

/// MFA 验证码记录
#[derive(Debug, Clone)]
pub struct MfaChallenge {
    pub challenge_id: String,
    pub mfa_type: MfaType,
    pub user_id: i32,
    pub code: String,
    pub secret: Option<String>,
    pub status: MfaStatus,
    pub created_at: chrono::DateTime<Utc>,
    pub expires_at: chrono::DateTime<Utc>,
    pub verified_at: Option<chrono::DateTime<Utc>>,
    pub attempt_count: u32,
}

/// MFA 验证请求
#[derive(Debug, Clone, Deserialize)]
pub struct MfaVerifyRequest {
    pub challenge_id: String,
    pub code: String,
}

/// MFA 注册请求
#[derive(Debug, Clone, Deserialize)]
pub struct MfaSetupRequest {
    pub mfa_type: MfaType,
    pub method: String, // phone or email
}

/// MFA 令牌信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaTokenInfo {
    pub enabled: bool,
    pub mfa_type: Option<MfaType>,
    pub method: Option<String>,
    pub verified: bool,
}

/// TOTP 生成器
pub struct TotpGenerator {
    secret_length: usize,
    time_step: u64,
    digits: u32,
}

impl TotpGenerator {
    pub fn new() -> Self {
        Self {
            secret_length: 32,
            time_step: 30,
            digits: 6,
        }
    }

    /// 生成随机密钥
    pub fn generate_secret(&self) -> String {
        let secret: Vec<u8> = (0..self.secret_length)
            .map(|_| rand::thread_rng().gen::<u8>())
            .collect();
        base32::encode(Alphabet::Rfc4648 { padding: false }, &secret)
    }

    /// 生成 TOTP 验证码
    pub fn generate_code(&self, secret: &str) -> Result<String, MfaError> {
        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
            .ok_or(MfaError::InvalidSecret)?;

        let time_counter = (Utc::now().timestamp() as u64) / self.time_step;

        let code = self.hotp(&secret_bytes, time_counter);
        Ok(format!("{:0>width$}", code, width = self.digits as usize))
    }

    /// 验证 TOTP 验证码
    pub fn verify_code(&self, secret: &str, code: &str, window: u64) -> bool {
        if let Ok(secret_bytes) = base32::decode(Alphabet::Rfc4648 { padding: false }, secret) {
            let time_counter = (Utc::now().timestamp() as u64) / self.time_step;

            // 检查当前时间窗口和前后 window 个窗口
            for i in (time_counter.saturating_sub(window))..=time_counter + window {
                let expected = self.hotp(&secret_bytes, i);
                let expected_str = format!("{:0>width$}", expected, width = self.digits as usize);
                if expected_str == code {
                    return true;
                }
            }
        }
        false
    }

    /// HOTP 算法
    fn hotp(&self, secret: &[u8], counter: u64) -> u32 {
        let mut counter_bytes = [0u8; 8];
        for (i, byte) in counter.to_be_bytes().iter().enumerate() {
            counter_bytes[7 - i] = *byte;
        }

        let hmac = hmac_sha1::hmac_sha1(secret, &counter_bytes);

        let offset = (hmac[hmac.len() - 1] & 0x0f) as usize;
        let truncated = u32::from_be_bytes([
            hmac[offset] & 0x7f,
            hmac[offset + 1],
            hmac[offset + 2],
            hmac[offset + 3],
        ]) % 10u32.pow(self.digits);

        truncated
    }

    /// 生成 TOTP URI (用于二维码)
    pub fn generate_uri(&self, secret: &str, issuer: &str, account: &str) -> String {
        let issuer_encoded = urlencoding::encode(issuer);
        let account_encoded = urlencoding::encode(account);
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits={}&period={}",
            issuer_encoded, account_encoded, secret, issuer_encoded, self.digits, self.time_step
        )
    }
}

impl Default for TotpGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// MFA 管理器
pub struct MfaManager {
    challenges: RwLock<HashMap<String, MfaChallenge>>,
    totp_generator: TotpGenerator,
    code_length: usize,
    code_expiry_seconds: u64,
    max_attempts: u32,
}

impl MfaManager {
    pub fn new() -> Self {
        Self {
            challenges: RwLock::new(HashMap::new()),
            totp_generator: TotpGenerator::new(),
            code_length: 6,
            code_expiry_seconds: 300, // 5分钟
            max_attempts: 3,
        }
    }

    /// 生成验证码
    pub async fn generate_code(&self, user_id: i32, mfa_type: MfaType, method: &str) -> Result<MfaChallenge, MfaError> {
        let challenge_id = format!("mfa_{}_{}_{}", user_id, mfa_type.to_string().to_lowercase(), Utc::now().timestamp());
        let code = self.generate_random_code();

        let challenge = MfaChallenge {
            challenge_id: challenge_id.clone(),
            mfa_type,
            user_id,
            code: code.clone(),
            secret: None,
            status: MfaStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(self.code_expiry_seconds as i64),
            verified_at: None,
            attempt_count: 0,
        };

        // 存储挑战
        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge_id.clone(), challenge.clone());

        // 根据类型处理
        match mfa_type {
            MfaType::Sms => {
                // 实际发送短信 (集成短信网关)
                log::info!("MFA SMS code for user {}: {}", user_id, code);
            }
            MfaType::Email => {
                // 实际发送邮件 (集成邮件服务)
                log::info!("MFA Email code for user {}: {}", user_id, code);
            }
            MfaType::Totp => {
                // TOTP 实时生成，不需要发送
            }
        }

        Ok(challenge)
    }

    /// 生成随机验证码
    fn generate_random_code(&self) -> String {
        let range = 10u32.pow(self.code_length as u32);
        let code = rand::thread_rng().gen_range(0..range);
        format!("{:0>width$}", code, width = self.code_length)
    }

    /// 验证验证码
    pub async fn verify_code(&self, user_id: i32, challenge_id: &str, code: &str) -> Result<MfaChallenge, MfaError> {
        let mut challenges = self.challenges.write().await;

        let challenge = challenges
            .get_mut(challenge_id)
            .ok_or(MfaError::ChallengeNotFound)?;

        // 检查用户匹配
        if challenge.user_id != user_id {
            return Err(MfaError::ChallengeNotFound);
        }

        // 检查状态
        if challenge.status != MfaStatus::Pending {
            return Err(MfaError::ChallengeExpired);
        }

        // 检查过期
        if Utc::now() > challenge.expires_at {
            challenge.status = MfaStatus::Expired;
            return Err(MfaError::CodeExpired);
        }

        // 检查尝试次数
        if challenge.attempt_count >= self.max_attempts {
            challenge.status = MfaStatus::Failed;
            return Err(MfaError::MaxAttemptsExceeded);
        }

        // 增加尝试次数
        challenge.attempt_count += 1;

        // 验证 TOTP
        if challenge.mfa_type == MfaType::Totp {
            if let Some(secret) = &challenge.secret {
                if self.totp_generator.verify_code(secret, code, 1) {
                    challenge.status = MfaStatus::Verified;
                    challenge.verified_at = Some(Utc::now());
                    return Ok(challenge.clone());
                }
            }
            return Err(MfaError::InvalidCode);
        }

        // 验证普通验证码
        if challenge.code == code {
            challenge.status = MfaStatus::Verified;
            challenge.verified_at = Some(Utc::now());
            Ok(challenge.clone())
        } else {
            Err(MfaError::InvalidCode)
        }
    }

    /// 生成 TOTP 密钥并返回设置信息
    pub async fn setup_totp(&self, user_id: i32) -> Result<(String, String), MfaError> {
        let secret = self.totp_generator.generate_secret();
        let issuer = "CarpTMS";
        let account = format!("user_{}", user_id);
        let uri = self.totp_generator.generate_uri(&secret, issuer, &account);

        // 创建挑战记录
        let challenge_id = format!("totp_setup_{}_{}", user_id, Utc::now().timestamp());
        let challenge = MfaChallenge {
            challenge_id: challenge_id.clone(),
            mfa_type: MfaType::Totp,
            user_id,
            code: String::new(),
            secret: Some(secret.clone()),
            status: MfaStatus::Pending,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(10),
            verified_at: None,
            attempt_count: 0,
        };

        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge_id, challenge);

        Ok((secret, uri))
    }

    /// 验证 TOTP 设置
    pub async fn verify_totp_setup(&self, user_id: i32, challenge_id: &str, code: &str) -> Result<bool, MfaError> {
        self.verify_code(user_id, challenge_id, code).await?;
        Ok(true)
    }

    /// 清理过期挑战
    pub async fn cleanup_expired(&self) {
        let mut challenges = self.challenges.write().await;
        let now = Utc::now();

        challenges.retain(|_, challenge| {
            if challenge.status == MfaStatus::Verified {
                false
            } else {
                now < challenge.expires_at
            }
        });
    }
}

impl Default for MfaManager {
    fn default() -> Self {
        Self::new()
    }
}

/// MFA 错误类型
#[derive(Debug)]
pub enum MfaError {
    ChallengeNotFound,
    ChallengeExpired,
    CodeExpired,
    InvalidCode,
    InvalidSecret,
    MaxAttemptsExceeded,
    InvalidMethod,
    SendFailed,
}

impl std::fmt::Display for MfaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MfaError::ChallengeNotFound => write!(f, "验证挑战不存在"),
            MfaError::ChallengeExpired => write!(f, "验证挑战已过期"),
            MfaError::CodeExpired => write!(f, "验证码已过期"),
            MfaError::InvalidCode => write!(f, "验证码无效"),
            MfaError::InvalidSecret => write!(f, "密钥格式无效"),
            MfaError::MaxAttemptsExceeded => write!(f, "尝试次数超限"),
            MfaError::InvalidMethod => write!(f, "验证方式无效"),
            MfaError::SendFailed => write!(f, "发送验证码失败"),
        }
    }
}

impl std::error::Error for MfaError {}

/// MFA 验证中间件
pub async fn mfa_verification_required(
    user_id: i32,
    mfa_manager: &MfaManager,
) -> Result<bool, MfaError> {
    // 检查用户是否启用了 MFA
    // 这里应该查询数据库或缓存
    // 暂时返回 false，实际应该根据用户配置判断
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_generation() {
        let generator = TotpGenerator::new();
        let secret = generator.generate_secret();
        assert_eq!(secret.len(), 52); // 32 bytes -> 52 base32 chars (no padding)

        let code = generator.generate_code(&secret).unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_totp_verification() {
        let generator = TotpGenerator::new();
        let secret = generator.generate_secret();

        let code = generator.generate_code(&secret).unwrap();
        assert!(generator.verify_code(&secret, &code, 1));
    }

    #[tokio::test]
    async fn test_mfa_manager() {
        let manager = MfaManager::new();

        // 测试生成验证码
        let challenge = manager
            .generate_code(1, MfaType::Sms, "13812345678")
            .await
            .unwrap();

        assert_eq!(challenge.code.len(), 6);
        assert_eq!(challenge.mfa_type, MfaType::Sms);

        // 测试验证
        let result = manager.verify_code(1, &challenge.challenge_id, &challenge.code).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, MfaStatus::Verified);
    }
}
