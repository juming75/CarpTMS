//! 抗抵赖机制模块
//! 支持操作数字签名、时间戳服务、证据固化

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use signature::{Signer, Verifier};
use std::collections::HashMap;

/// 签名类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureType {
    /// 数据签名
    DataSignature,
    /// 操作签名
    OperationSignature,
    /// 事务签名
    TransactionSignature,
    /// 文档签名
    DocumentSignature,
}

/// 签名状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureStatus {
    /// 待签名
    Pending,
    /// 已签名
    Signed,
    /// 已验证
    Verified,
    /// 已失效
    Invalid,
    /// 已过期
    Expired,
}

/// 签名算法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// SHA256 with RSA
    RsaSha256,
    /// SHA256 with ECDSA
    EcdsaSha256,
    /// SHA256 with HMAC
    HmacSha256,
}

/// 签名记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRecord {
    /// 签名ID
    pub signature_id: String,
    /// 签名类型
    pub signature_type: SignatureType,
    /// 签名算法
    pub algorithm: SignatureAlgorithm,
    /// 签名者ID
    pub signer_id: i32,
    /// 签名者名称
    pub signer_name: String,
    /// 签名者角色
    pub signer_role: String,
    /// 原始数据哈希
    pub data_hash: String,
    /// 签名值 (Base64)
    pub signature_value: String,
    /// 签名状态
    pub status: SignatureStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 验证时间
    pub verified_at: Option<DateTime<Utc>>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 关联的操作类型
    pub operation_type: Option<String>,
    /// 关联的资源ID
    pub resource_id: Option<String>,
    /// 关联的资源类型
    pub resource_type: Option<String>,
    /// IP地址
    pub ip_address: Option<String>,
    /// 用户代理
    pub user_agent: Option<String>,
    /// 附加元数据
    pub metadata: Option<serde_json::Value>,
}

/// 时间戳记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampRecord {
    /// 时间戳ID
    pub timestamp_id: String,
    /// 数据哈希
    pub data_hash: String,
    /// 时间戳值 (RFC 3161)
    pub timestamp_value: String,
    /// 签名算法
    pub algorithm: String,
    /// 颁发者
    pub issuer: String,
    /// 签名时间
    pub signed_at: DateTime<Utc>,
    /// 精度 (毫秒)
    pub precision: u32,
    /// 序列号
    pub serial_number: String,
}

/// 证据记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    /// 证据ID
    pub evidence_id: String,
    /// 证据类型
    pub evidence_type: String,
    /// 证据哈希
    pub evidence_hash: String,
    /// 证据内容摘要
    pub content_digest: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 创建者
    pub creator_id: i32,
    /// 创建者名称
    pub creator_name: String,
    /// 关联的签名记录
    pub signature_ids: Vec<String>,
    /// 关联的时间戳记录
    pub timestamp_ids: Vec<String>,
    /// 证据完整性状态
    pub integrity_verified: bool,
    /// 证据存储位置
    pub storage_location: Option<String>,
    /// 证据备份位置
    pub backup_locations: Vec<String>,
    /// 元数据
    pub metadata: Option<serde_json::Value>,
}

/// 证据固化请求
#[derive(Debug, Clone, Deserialize)]
pub struct EvidenceCreationRequest {
    pub evidence_type: String,
    pub content: String,
    pub operation_type: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub user_id: i32,
    pub user_name: String,
    pub user_role: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// 签名验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureVerificationResult {
    pub valid: bool,
    pub signature_id: String,
    pub signer_id: i32,
    pub signer_name: String,
    pub signed_at: DateTime<Utc>,
    pub verified_at: DateTime<Utc>,
    pub error_message: Option<String>,
}

/// 数据哈希工具
pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// 计算带盐的哈希
pub fn compute_salted_hash(data: &[u8], salt: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// 抗抵赖服务
pub struct NonRepudiationService {
    signature_records: HashMap<String, SignatureRecord>,
    timestamp_records: HashMap<String, TimestampRecord>,
    evidence_records: HashMap<String, EvidenceRecord>,
}

impl NonRepudiationService {
    pub fn new() -> Self {
        Self {
            signature_records: HashMap::new(),
            timestamp_records: HashMap::new(),
            evidence_records: HashMap::new(),
        }
    }

    /// 创建数据签名
    pub fn sign_data(
        &mut self,
        data: &[u8],
        signer_id: i32,
        signer_name: &str,
        signer_role: &str,
        algorithm: SignatureAlgorithm,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<SignatureRecord, NonRepudiationError> {
        let signature_id = format!("sig_{}_{}_{}", signer_id, algorithm.to_string().to_lowercase(), Utc::now().timestamp_millis());
        let data_hash = compute_hash(data);

        // 生成签名值（简化实现，实际应使用真正的签名算法）
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(signer_id.to_le_bytes());
        hasher.update(signer_name.as_bytes());
        hasher.update(Utc::now().timestamp().to_le_bytes());
        let signature_value = hex::encode(hasher.finalize());

        let record = SignatureRecord {
            signature_id: signature_id.clone(),
            signature_type: SignatureType::DataSignature,
            algorithm,
            signer_id,
            signer_name: signer_name.to_string(),
            signer_role: signer_role.to_string(),
            data_hash,
            signature_value,
            status: SignatureStatus::Signed,
            created_at: Utc::now(),
            verified_at: None,
            expires_at: None,
            operation_type: None,
            resource_id: None,
            resource_type: None,
            ip_address,
            user_agent,
            metadata: None,
        };

        self.signature_records.insert(signature_id.clone(), record.clone());

        Ok(record)
    }

    /// 创建操作签名
    pub fn sign_operation(
        &mut self,
        operation_data: &[u8],
        operation_type: &str,
        resource_id: &str,
        resource_type: &str,
        signer_id: i32,
        signer_name: &str,
        signer_role: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<SignatureRecord, NonRepudiationError> {
        let signature_id = format!("sig_op_{}_{}_{}", operation_type, resource_id, Utc::now().timestamp_millis());
        let data_hash = compute_hash(operation_data);

        // 生成签名值
        let mut hasher = Sha256::new();
        hasher.update(operation_data);
        hasher.update(operation_type.as_bytes());
        hasher.update(resource_id.as_bytes());
        hasher.update(signer_id.to_le_bytes());
        hasher.update(Utc::now().timestamp().to_le_bytes());
        let signature_value = hex::encode(hasher.finalize());

        let record = SignatureRecord {
            signature_id: signature_id.clone(),
            signature_type: SignatureType::OperationSignature,
            algorithm: SignatureAlgorithm::RsaSha256,
            signer_id,
            signer_name: signer_name.to_string(),
            signer_role: signer_role.to_string(),
            data_hash,
            signature_value,
            status: SignatureStatus::Signed,
            created_at: Utc::now(),
            verified_at: None,
            expires_at: None,
            operation_type: Some(operation_type.to_string()),
            resource_id: Some(resource_id.to_string()),
            resource_type: Some(resource_type.to_string()),
            ip_address,
            user_agent,
            metadata: None,
        };

        self.signature_records.insert(signature_id.clone(), record.clone());

        Ok(record)
    }

    /// 验证签名
    pub fn verify_signature(&mut self, signature_id: &str, original_data: &[u8]) -> Result<SignatureVerificationResult, NonRepudiationError> {
        let record = self.signature_records
            .get_mut(signature_id)
            .ok_or(NonRepudiationError::SignatureNotFound)?;

        // 重新计算数据哈希
        let computed_hash = compute_hash(original_data);

        // 验证哈希匹配
        if computed_hash != record.data_hash {
            return Ok(SignatureVerificationResult {
                valid: false,
                signature_id: signature_id.to_string(),
                signer_id: record.signer_id,
                signer_name: record.signer_name.clone(),
                signed_at: record.created_at,
                verified_at: Utc::now(),
                error_message: Some("数据哈希不匹配".to_string()),
            });
        }

        // 更新验证状态
        record.status = SignatureStatus::Verified;
        record.verified_at = Some(Utc::now());

        Ok(SignatureVerificationResult {
            valid: true,
            signature_id: signature_id.to_string(),
            signer_id: record.signer_id,
            signer_name: record.signer_name.clone(),
            signed_at: record.created_at,
            verified_at: Utc::now(),
            error_message: None,
        })
    }

    /// 创建时间戳
    pub fn create_timestamp(&mut self, data: &[u8]) -> Result<TimestampRecord, NonRepudiationError> {
        let timestamp_id = format!("ts_{}", Utc::now().timestamp_millis());
        let data_hash = compute_hash(data);

        // 生成时间戳值（简化实现，实际应使用 RFC 3161）
        let mut hasher = Sha256::new();
        hasher.update(&data_hash);
        hasher.update(Utc::now().timestamp().to_le_bytes());
        let timestamp_value = hex::encode(hasher.finalize());

        let record = TimestampRecord {
            timestamp_id: timestamp_id.clone(),
            data_hash,
            timestamp_value,
            algorithm: "SHA256".to_string(),
            issuer: "CarpTMS TSA".to_string(),
            signed_at: Utc::now(),
            precision: 1,
            serial_number: timestamp_id.clone(),
        };

        self.timestamp_records.insert(timestamp_id.clone(), record.clone());

        Ok(record)
    }

    /// 验证时间戳
    pub fn verify_timestamp(&self, timestamp_id: &str) -> Result<bool, NonRepudiationError> {
        let record = self.timestamp_records
            .get(timestamp_id)
            .ok_or(NonRepudiationError::TimestampNotFound)?;

        // 验证时间戳有效性（简化检查）
        if record.signed_at > Utc::now() {
            return Ok(false);
        }

        Ok(true)
    }

    /// 创建证据固化记录
    pub fn create_evidence(
        &mut self,
        request: EvidenceCreationRequest,
    ) -> Result<EvidenceRecord, NonRepudiationError> {
        let evidence_id = format!("ev_{}_{}", request.evidence_type, Utc::now().timestamp_millis());
        let content_digest = compute_hash(request.content.as_bytes());
        let evidence_hash = compute_salted_hash(content_digest.as_bytes(), evidence_id.as_bytes());

        let record = EvidenceRecord {
            evidence_id: evidence_id.clone(),
            evidence_type: request.evidence_type,
            evidence_hash,
            content_digest,
            created_at: Utc::now(),
            creator_id: request.user_id,
            creator_name: request.user_name,
            signature_ids: Vec::new(),
            timestamp_ids: Vec::new(),
            integrity_verified: true,
            storage_location: None,
            backup_locations: Vec::new(),
            metadata: None,
        };

        self.evidence_records.insert(evidence_id.clone(), record.clone());

        Ok(record)
    }

    /// 验证证据完整性
    pub fn verify_evidence(&self, evidence_id: &str, content: &[u8]) -> Result<bool, NonRepudiationError> {
        let record = self.evidence_records
            .get(evidence_id)
            .ok_or(NonRepudiationError::EvidenceNotFound)?;

        // 重新计算内容摘要
        let computed_digest = compute_hash(content);

        // 验证内容摘要匹配
        if computed_digest != record.content_digest {
            return Ok(false);
        }

        Ok(true)
    }

    /// 关联签名和时间戳到证据
    pub fn attach_to_evidence(
        &mut self,
        evidence_id: &str,
        signature_ids: Vec<String>,
        timestamp_ids: Vec<String>,
    ) -> Result<(), NonRepudiationError> {
        let record = self.evidence_records
            .get_mut(evidence_id)
            .ok_or(NonRepudiationError::EvidenceNotFound)?;

        record.signature_ids.extend(signature_ids);
        record.timestamp_ids.extend(timestamp_ids);

        Ok(())
    }

    /// 获取签名记录
    pub fn get_signature_record(&self, signature_id: &str) -> Option<&SignatureRecord> {
        self.signature_records.get(signature_id)
    }

    /// 获取时间戳记录
    pub fn get_timestamp_record(&self, timestamp_id: &str) -> Option<&TimestampRecord> {
        self.timestamp_records.get(timestamp_id)
    }

    /// 获取证据记录
    pub fn get_evidence_record(&self, evidence_id: &str) -> Option<&EvidenceRecord> {
        self.evidence_records.get(evidence_id)
    }

    /// 查询用户的签名记录
    pub fn get_user_signatures(&self, user_id: i32) -> Vec<&SignatureRecord> {
        self.signature_records
            .values()
            .filter(|r| r.signer_id == user_id)
            .collect()
    }

    /// 查询资源的签名记录
    pub fn get_resource_signatures(&self, resource_type: &str, resource_id: &str) -> Vec<&SignatureRecord> {
        self.signature_records
            .values()
            .filter(|r| {
                r.resource_type.as_deref() == Some(resource_type)
                    && r.resource_id.as_deref() == Some(resource_id)
            })
            .collect()
    }
}

impl Default for NonRepudiationService {
    fn default() -> Self {
        Self::new()
    }
}

/// 抗抵赖错误类型
#[derive(Debug)]
pub enum NonRepudiationError {
    SignatureNotFound,
    TimestampNotFound,
    EvidenceNotFound,
    SignatureInvalid,
    HashMismatch,
    StorageError,
}

impl std::fmt::Display for NonRepudiationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NonRepudiationError::SignatureNotFound => write!(f, "签名记录不存在"),
            NonRepudiationError::TimestampNotFound => write!(f, "时间戳记录不存在"),
            NonRepudiationError::EvidenceNotFound => write!(f, "证据记录不存在"),
            NonRepudiationError::SignatureInvalid => write!(f, "签名无效"),
            NonRepudiationError::HashMismatch => write!(f, "哈希值不匹配"),
            NonRepudiationError::StorageError => write!(f, "存储错误"),
        }
    }
}

impl std::error::Error for NonRepudiationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let data = b"test data";
        let hash = compute_hash(data);
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_signature_creation() {
        let mut service = NonRepudiationService::new();
        let data = b"test operation data";

        let record = service.sign_data(
            data,
            1,
            "test_user",
            "admin",
            SignatureAlgorithm::RsaSha256,
            Some("127.0.0.1".to_string()),
            Some("Test Agent".to_string()),
        ).unwrap();

        assert_eq!(record.status, SignatureStatus::Signed);
        assert_eq!(record.signer_id, 1);
    }

    #[test]
    fn test_signature_verification() {
        let mut service = NonRepudiationService::new();
        let data = b"test operation data";

        let record = service.sign_data(
            data,
            1,
            "test_user",
            "admin",
            SignatureAlgorithm::RsaSha256,
            None,
            None,
        ).unwrap();

        let result = service.verify_signature(&record.signature_id, data).unwrap();
        assert!(result.valid);
    }

    #[test]
    fn test_timestamp_creation() {
        let mut service = NonRepudiationService::new();
        let data = b"test data";

        let record = service.create_timestamp(data).unwrap();
        assert_eq!(record.algorithm, "SHA256");
    }

    #[test]
    fn test_evidence_creation() {
        let mut service = NonRepudiationService::new();

        let request = EvidenceCreationRequest {
            evidence_type: "operation_log".to_string(),
            content: "User performed sensitive operation".to_string(),
            operation_type: Some("data_export".to_string()),
            resource_id: Some("123".to_string()),
            resource_type: Some("user".to_string()),
            user_id: 1,
            user_name: "admin".to_string(),
            user_role: "administrator".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
        };

        let evidence = service.create_evidence(request).unwrap();
        assert_eq!(evidence.evidence_type, "operation_log");
        assert!(evidence.integrity_verified);
    }
}
