use aes_gcm::{
    aead::{Aead, Payload},
    Aes256Gcm, KeyInit, Nonce,
};
use rand::RngCore;
use std::error::Error;
use std::fmt;

// 添加base64依赖
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine;

// 错误类型定义
#[derive(Debug)]
pub struct EncryptionError {
    message: String,
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EncryptionError {}

// 从环境变量获取加密密钥(使用 SecretManager)
pub async fn get_encryption_key() -> Result<[u8; 32], EncryptionError> {
    super::secret_manager::SecretManager::get_encryption_key()
        .await
        .map_err(|e| EncryptionError {
            message: format!("Failed to get encryption key: {}", e),
        })
}

// 从环境变量获取加密密钥(阻塞版本)
pub fn get_encryption_key_blocking() -> Result<[u8; 32], EncryptionError> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(get_encryption_key())
}

// 生成随机nonce
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut nonce);
    nonce
}

// 加密数据
pub async fn encrypt_data(data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    let key = get_encryption_key().await?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| EncryptionError {
        message: format!("Failed to create encryption cipher: {:?}", e),
    })?;

    let nonce = generate_nonce();
    let nonce = Nonce::from_slice(&nonce);

    let ciphertext = cipher
        .encrypt(nonce, Payload::from(data))
        .map_err(|e| EncryptionError {
            message: format!("Failed to encrypt data: {:?}", e),
        })?;

    // 结合nonce和密文,nonce需要用于解密
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend(nonce.as_slice());
    result.extend(ciphertext);

    Ok(result)
}

// 解密数据
pub async fn decrypt_data(encrypted_data: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    const NONCE_LENGTH: usize = 12;

    if encrypted_data.len() < NONCE_LENGTH {
        return Err(EncryptionError {
            message: "Encrypted data is too short".to_string(),
        });
    }

    let key = get_encryption_key().await?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| EncryptionError {
        message: format!("Failed to create decryption cipher: {:?}", e),
    })?;

    // 提取nonce和密文
    let nonce = Nonce::from_slice(&encrypted_data[..NONCE_LENGTH]);
    let ciphertext = &encrypted_data[NONCE_LENGTH..];

    let plaintext = cipher
        .decrypt(nonce, Payload::from(ciphertext))
        .map_err(|e| EncryptionError {
            message: format!("Failed to decrypt data: {:?}", e),
        })?;

    Ok(plaintext)
}

// 加密字符串
pub async fn encrypt_string(s: &str) -> Result<String, EncryptionError> {
    let encrypted = encrypt_data(s.as_bytes()).await?;
    Ok(BASE64_ENGINE.encode(encrypted))
}

// 加密字符串(阻塞版本)
pub fn encrypt_string_blocking(s: &str) -> Result<String, EncryptionError> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(encrypt_string(s))
}

// 解密字符串
pub async fn decrypt_string(s: &str) -> Result<String, EncryptionError> {
    let decoded = BASE64_ENGINE.decode(s).map_err(|e| EncryptionError {
        message: format!("Failed to decode base64 string: {:?}", e),
    })?;
    let decrypted = decrypt_data(&decoded).await?;
    String::from_utf8(decrypted).map_err(|e| EncryptionError {
        message: format!("Failed to convert decrypted data to string: {:?}", e),
    })
}

// 解密字符串(阻塞版本)
pub fn decrypt_string_blocking(s: &str) -> Result<String, EncryptionError> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(decrypt_string(s))
}

// 重新导出base64函数以简化使用
pub fn base64_encode(data: &[u8]) -> String {
    BASE64_ENGINE.encode(data)
}

pub fn base64_decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    BASE64_ENGINE.decode(s)
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    // Set a fixed encryption key for testing (exactly 32 bytes)
    const TEST_ENCRYPTION_KEY: &str = "01234567890123456789012345678901";

    // Helper function to set the test encryption key using std::env::set_var
    fn set_test_encryption_key() {
        // In test environment, it's safe to use set_var
        unsafe {
            std::env::set_var("ENCRYPTION_KEY", TEST_ENCRYPTION_KEY);
        }
    }

    #[tokio::test]
    #[ignore] // 需要设置ENCRYPTION_KEY环境变量
    async fn test_get_encryption_key() {
        // Set test key first
        set_test_encryption_key();

        // Test that get_encryption_key returns a valid 32-byte key
        let key = get_encryption_key()
            .await
            .expect("Failed to get encryption key");
        assert_eq!(key.len(), 32, "Encryption key should be 32 bytes long");

        // Test that the key matches what we set
        assert_eq!(
            key,
            TEST_ENCRYPTION_KEY.as_bytes(),
            "Encryption key should match the one set in environment"
        );
    }

    #[test]
    fn test_generate_nonce() {
        // Test that generate_nonce returns a valid 12-byte nonce
        let nonce = generate_nonce();
        assert_eq!(nonce.len(), 12, "Nonce should be 12 bytes long");

        // Test that two consecutive nonces are different
        let nonce2 = generate_nonce();
        assert_ne!(nonce, nonce2, "Consecutive nonces should be different");
    }

    #[tokio::test]
    #[ignore] // 需要设置ENCRYPTION_KEY环境变量
    async fn test_encrypt_decrypt_data() {
        // Set test key first
        set_test_encryption_key();

        // Test that data can be encrypted and decrypted correctly
        let original_data = b"test data 12345";

        let encrypted = encrypt_data(original_data)
            .await
            .expect("Failed to encrypt data");
        let decrypted = decrypt_data(&encrypted)
            .await
            .expect("Failed to decrypt data");

        assert_eq!(
            original_data.to_vec(),
            decrypted,
            "Decrypted data should match original"
        );
    }

    #[tokio::test]
    #[ignore] // 需要设置ENCRYPTION_KEY环境变量
    async fn test_encrypt_decrypt_string() {
        // Set test key first
        set_test_encryption_key();

        // Test that strings can be encrypted and decrypted correctly
        let original_string = "This is a test string for encryption!";

        let encrypted = encrypt_string(original_string)
            .await
            .expect("Failed to encrypt string");
        let decrypted = decrypt_string(&encrypted)
            .await
            .expect("Failed to decrypt string");

        assert_eq!(
            original_string, decrypted,
            "Decrypted string should match original"
        );
    }

    #[tokio::test]
    #[ignore] // 需要设置ENCRYPTION_KEY环境变量
    async fn test_encrypt_decrypt_empty_string() {
        // Set test key first
        set_test_encryption_key();

        // Test that empty strings can be encrypted and decrypted correctly
        let original_string = "";

        let encrypted = encrypt_string(original_string)
            .await
            .expect("Failed to encrypt empty string");
        let decrypted = decrypt_string(&encrypted)
            .await
            .expect("Failed to decrypt empty string");

        assert_eq!(
            original_string, decrypted,
            "Decrypted empty string should match original"
        );
    }

    #[test]
    fn test_base64_encode_decode() {
        // Test base64 encoding and decoding
        let original_data = b"test base64 data";

        let encoded = base64_encode(original_data);
        let decoded = base64_decode(&encoded).expect("Failed to decode base64");

        assert_eq!(
            original_data.to_vec(),
            decoded,
            "Decoded base64 should match original"
        );
    }

    #[tokio::test]
    async fn test_decrypt_invalid_base64() {
        // Set test key first
        set_test_encryption_key();

        // Test that decrypting invalid base64 returns an error
        let result = decrypt_string("invalid base64 string").await;
        assert!(
            result.is_err(),
            "Decrypting invalid base64 should return an error"
        );

        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("Failed to decode base64"),
            "Error should be about base64 decoding"
        );
    }

    #[tokio::test]
    async fn test_decrypt_short_data() {
        // Set test key first
        set_test_encryption_key();

        // Test that decrypting data that's too short returns an error
        let short_data = base64_encode(b"short");
        let result = decrypt_string(&short_data).await;
        assert!(
            result.is_err(),
            "Decrypting short data should return an error"
        );

        let error_message = result.unwrap_err().to_string();
        assert!(
            error_message.contains("too short"),
            "Error should be about data being too short"
        );
    }
}
