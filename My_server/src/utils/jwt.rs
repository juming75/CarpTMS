use actix_web::{dev::Payload, FromRequest, HttpRequest};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

use crate::metrics::{JWT_TOKENS_GENERATED, JWT_TOKENS_VALIDATED};

// JWT声明结构体(访问令牌)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // 主题(用户ID)
    pub sub: String,
    // 过期时间
    pub exp: usize,
    // 颁发时间
    pub iat: usize,
    // 颁发者
    pub iss: String,
    // 用户角色
    pub role: String,
    // 用户组ID
    pub group_id: i32,
    // 令牌类型
    pub token_type: String,
}

// 刷新令牌声明结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    // 主题(用户ID)
    pub sub: String,
    // 过期时间
    pub exp: usize,
    // 颁发时间
    pub iat: usize,
    // 颁发者
    pub iss: String,
    // 令牌类型
    pub token_type: String,
}

// 从环境变量获取JWT算法类型
pub fn get_jwt_algorithm() -> Algorithm {
    match env::var("JWT_ALGORITHM").as_deref() {
        Ok("RS256") => Algorithm::RS256,
        _ => Algorithm::HS256,
    }
}

// 获取HS256密钥(使用 SecretManager)
pub async fn get_hs256_secret() -> Result<String, String> {
    // 从环境变量读取密钥
    std::env::var("JWT_SECRET").map_err(|_| {
        "JWT_SECRET environment variable is not set. Please set it to a secure secret.".to_string()
    })
}

// 检查JWT密钥是否为默认值
pub fn check_jwt_secret() -> Result<(), String> {
    let secret = std::env::var("JWT_SECRET").map_err(|_| {
        "JWT_SECRET environment variable is not set. Please set it to a secure secret.".to_string()
    })?;

    // 检查是否为默认值
    let default_secrets = [
        "dev_jwt_secret_change_in_production_123456789012345678901234567890",
        "your-secret-key",
        "secret",
        "jwt-secret",
    ];

    if default_secrets.contains(&secret.as_str()) {
        return Err("JWT_SECRET is set to a default value. Please set it to a secure, unique secret in production.".to_string());
    }

    // 检查密钥长度
    if secret.len() < 32 {
        return Err("JWT_SECRET must be at least 32 characters long for security.".to_string());
    }

    Ok(())
}

// 获取HS256密钥(阻塞版本)
pub fn get_hs256_secret_blocking() -> Result<String, String> {
    // 从环境变量读取密钥
    std::env::var("JWT_SECRET").map_err(|_| {
        "JWT_SECRET environment variable is not set. Please set it to a secure secret.".to_string()
    })
}

// 获取编码密钥(确保格式正确)
fn get_encoding_key() -> Result<EncodingKey, jsonwebtoken::errors::Error> {
    let secret = get_hs256_secret_blocking().map_err(|_| {
        jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat)
    })?;
    // HS256 需要至少32字节的密钥
    if secret.len() < 32 {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
        ));
    }
    Ok(EncodingKey::from_secret(secret.as_bytes()))
}

// 获取解码密钥
fn get_decoding_key() -> Result<DecodingKey, jsonwebtoken::errors::Error> {
    let secret = get_hs256_secret_blocking().map_err(|_| {
        jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat)
    })?;
    Ok(DecodingKey::from_secret(secret.as_bytes()))
}

// 获取RS256私钥
pub fn get_rs256_private_key() -> Result<Vec<u8>, jsonwebtoken::errors::Error> {
    let private_key_path = 
        env::var("JWT_PRIVATE_KEY_PATH").map_err(|_| {
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat)
        })?;
    if Path::new(&private_key_path).exists() {
        fs::read(private_key_path).map_err(|_| {
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat)
        })
    } else {
        Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat))
    }
}

// 获取RS256公钥
pub fn get_rs256_public_key() -> Result<Vec<u8>, jsonwebtoken::errors::Error> {
    let public_key_path = 
        env::var("JWT_PUBLIC_KEY_PATH").map_err(|_| {
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat)
        })?;
    if Path::new(&public_key_path).exists() {
        fs::read(public_key_path).map_err(|_| {
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat)
        })
    } else {
        Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat))
    }
}

// 生成访问令牌
pub fn generate_access_token(
    user_id: i32,
    role: &str,
    group_id: i32,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiration = now + Duration::hours(1); // 缩短访问令牌有效期为1小时,提高安全性
    let algorithm = get_jwt_algorithm();

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: "tms_server".to_string(),
        role: role.to_string(),
        group_id,
        token_type: "access".to_string(),
    };

    let header = Header::new(algorithm);
    let encoding_key = match algorithm {
        Algorithm::RS256 => {
            let private_key = get_rs256_private_key()?;
            EncodingKey::from_rsa_pem(&private_key)?
        }
        _ => get_encoding_key()?,
    };

    let result = encode(&header, &claims, &encoding_key);

    // 增加令牌生成计数器
    if result.is_ok() {
        JWT_TOKENS_GENERATED.with_label_values(&["access"]).inc();
    }

    result
}

// 生成刷新令牌
pub fn generate_refresh_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiration = now + Duration::days(7); // 刷新令牌有效期7天
    let algorithm = get_jwt_algorithm();

    let claims = RefreshClaims {
        sub: user_id.to_string(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: "tms_server".to_string(),
        token_type: "refresh".to_string(),
    };

    let header = Header::new(algorithm);
    let encoding_key = match algorithm {
        Algorithm::RS256 => {
            let private_key = get_rs256_private_key()?;
            EncodingKey::from_rsa_pem(&private_key)?
        }
        _ => get_encoding_key()?,
    };

    let result = encode(&header, &claims, &encoding_key);

    // 增加令牌生成计数器
    if result.is_ok() {
        JWT_TOKENS_GENERATED.with_label_values(&["refresh"]).inc();
    }

    result
}

// 为了向后兼容,保留原函数名称
pub fn generate_token(
    user_id: i32,
    role: &str,
    group_id: i32,
) -> Result<String, jsonwebtoken::errors::Error> {
    generate_access_token(user_id, role, group_id)
}

// 验证访问令牌
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let algorithm = get_jwt_algorithm();
    let decoding_key = match algorithm {
        Algorithm::RS256 => {
            let public_key = get_rs256_public_key()?;
            DecodingKey::from_rsa_pem(&public_key)?
        }
        _ => get_decoding_key()?,
    };
    let mut validation = Validation::new(algorithm);
    validation.leeway = 30; // 30秒的容错时间

    let result = decode::<Claims>(token, &decoding_key, &validation);

    // 增加令牌验证计数器
    match &result {
        Ok(_) => JWT_TOKENS_VALIDATED.with_label_values(&["success"]).inc(),
        Err(_) => JWT_TOKENS_VALIDATED.with_label_values(&["failure"]).inc(),
    }

    result.map(|data| data.claims)
}

// 验证刷新令牌
pub fn verify_refresh_token(token: &str) -> Result<RefreshClaims, jsonwebtoken::errors::Error> {
    let algorithm = get_jwt_algorithm();
    let decoding_key = match algorithm {
        Algorithm::RS256 => {
            let public_key = get_rs256_public_key()?;
            DecodingKey::from_rsa_pem(&public_key)?
        }
        _ => get_decoding_key()?,
    };
    let mut validation = Validation::new(algorithm);
    validation.leeway = 30; // 30秒的容错时间

    let result = decode::<RefreshClaims>(token, &decoding_key, &validation);

    // 增加令牌验证计数器
    match &result {
        Ok(_) => JWT_TOKENS_VALIDATED.with_label_values(&["success"]).inc(),
        Err(_) => JWT_TOKENS_VALIDATED.with_label_values(&["failure"]).inc(),
    }

    result.map(|data| data.claims)
}

// 实现FromRequest trait,用于从请求中提取JWT声明
#[async_trait::async_trait]
impl FromRequest for Claims {
    type Error = actix_web::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let token = extract_token_from_header(req);
        let fut = async move {
            match token {
                Some(token) => match verify_token(&token) {
                    Ok(claims) => Ok(claims),
                    Err(_) => Err(actix_web::error::ErrorUnauthorized(
                        "Invalid or expired token",
                    )),
                },
                None => Err(actix_web::error::ErrorUnauthorized("Token not provided")),
            }
        };

        Box::pin(fut)
    }
}

// 从请求头中提取JWT令牌
fn extract_token_from_header(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer ").map(|s| s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    // 设置测试环境变量
    fn setup_test_env() {
        unsafe {
            std::env::set_var("JWT_ALGORITHM", "HS256");
            std::env::set_var("JWT_SECRET", "test_secret_key_123456789012345678901234");
        }
    }

    #[test]
    #[ignore] // 需要设置环境变量
    fn test_get_jwt_algorithm() {
        // 测试默认算法
        unsafe {
            std::env::remove_var("JWT_ALGORITHM");
        }
        assert_eq!(get_jwt_algorithm(), Algorithm::HS256);

        // 测试HS256算法
        unsafe {
            std::env::set_var("JWT_ALGORITHM", "HS256");
        }
        assert_eq!(get_jwt_algorithm(), Algorithm::HS256);

        // 测试RS256算法
        unsafe {
            std::env::set_var("JWT_ALGORITHM", "RS256");
        }
        assert_eq!(get_jwt_algorithm(), Algorithm::RS256);

        // 恢复默认设置,避免影响其他测试
        unsafe {
            std::env::set_var("JWT_ALGORITHM", "HS256");
        }
    }

    #[tokio::test]
    #[ignore] // 需要设置环境变量
    async fn test_get_hs256_secret() {
        // 测试默认密钥生成
        unsafe {
            std::env::remove_var("JWT_SECRET");
        }
        let secret = get_hs256_secret().await.unwrap();
        assert_eq!(secret.len(), 64);

        // 测试环境变量密钥
        unsafe {
            std::env::set_var("JWT_SECRET", "test_secret_key");
        }
        assert_eq!(get_hs256_secret().await.unwrap(), "test_secret_key");

        // 恢复测试密钥,避免影响其他测试
        unsafe {
            std::env::set_var("JWT_SECRET", "test_secret_key_123456789012345678901234");
        }
    }

    #[test]
    fn test_generate_and_verify_access_token() {
        setup_test_env();

        // 确保使用 HS256 算法
        assert_eq!(get_jwt_algorithm(), Algorithm::HS256);

        // 生成访问令牌
        let user_id = 1;
        let role = "admin";
        let group_id = 1;
        let token = generate_access_token(user_id, role, group_id).unwrap();

        // 验证访问令牌
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.role, role);
        assert_eq!(claims.group_id, group_id);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_generate_and_verify_refresh_token() {
        setup_test_env();

        // 确保使用 HS256 算法
        assert_eq!(get_jwt_algorithm(), Algorithm::HS256);

        // 生成刷新令牌
        let user_id = 1;
        let token = generate_refresh_token(user_id).unwrap();

        // 验证刷新令牌
        let claims = verify_refresh_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_generate_token_compatibility() {
        setup_test_env();

        // 确保使用 HS256 算法
        assert_eq!(get_jwt_algorithm(), Algorithm::HS256);

        // 测试向后兼容函数
        let user_id = 1;
        let role = "admin";
        let group_id = 1;
        let token = generate_token(user_id, role, group_id).unwrap();

        // 验证令牌
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.role, role);
        assert_eq!(claims.group_id, group_id);
    }

    #[test]
    fn test_verify_invalid_token() {
        setup_test_env();

        // 测试验证无效令牌
        let invalid_token = "invalid_jwt_token";
        let result = verify_token(invalid_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_from_header() {
        // 测试从请求头中提取有效令牌
        let token = "test_token123";
        let req = TestRequest::default()
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_http_request();

        let extracted_token = extract_token_from_header(&req);
        assert_eq!(extracted_token, Some(token.to_string()));

        // 测试没有Authorization头的情况
        let req = TestRequest::default().to_http_request();
        let extracted_token = extract_token_from_header(&req);
        assert_eq!(extracted_token, None);

        // 测试无效Authorization头的情况
        let req = TestRequest::default()
            .insert_header(("Authorization", "InvalidTokenFormat"))
            .to_http_request();
        let extracted_token = extract_token_from_header(&req);
        assert_eq!(extracted_token, None);
    }
}
