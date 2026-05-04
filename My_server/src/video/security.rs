//! 数据安全工具模块
//!
//! 提供敏感数据保护功能：
//! - 数据脱敏
//! - 日志脱敏
//! - SQL 注入防护
//! - 审计日志
//!
//! # 使用示例
//! ```ignore
//! use crate::video::security::{SensitiveData, mask_phone, mask_plate};
//!
//! let phone = "13812345678";
//! let masked = mask_phone(phone); // "138****5678"
//!
//! let plate = "京A12345";
//! let masked = mask_plate(plate); // "京A***45"
//! ```

use serde::{Deserialize, Serialize};

/// 敏感数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensitiveDataType {
    /// 手机号
    PhoneNumber,
    /// 车牌号
    LicensePlate,
    /// 身份证号
    IdCard,
    /// 银行卡号
    BankCard,
    /// 密码
    Password,
    /// 邮箱
    Email,
    /// 地址
    Address,
    /// 自定义
    Custom(String),
}

/// 敏感数据包装器
#[derive(Debug, Clone)]
pub struct SensitiveData<T> {
    /// 原始数据
    value: T,
    /// 数据类型
    data_type: SensitiveDataType,
}

impl<T> SensitiveData<T> {
    /// 创建敏感数据
    pub fn new(value: T, data_type: SensitiveDataType) -> Self {
        Self { value, data_type }
    }

    /// 获取脱敏后的值
    pub fn masked(&self) -> String
    where
        T: AsRef<str>,
    {
        mask_value(self.value.as_ref(), &self.data_type)
    }

    /// 获取原始值引用
    pub fn inner(&self) -> &T {
        &self.value
    }

    /// 获取原始值（克隆）
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: AsRef<str>> AsRef<str> for SensitiveData<T> {
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

/// 脱敏手机号
///
/// # 示例
/// - `13812345678` -> `138****5678`
/// - `8613812345678` -> `8613****5678`
pub fn mask_phone(phone: &str) -> String {
    if phone.len() < 7 {
        return "*".repeat(phone.len());
    }

    let start = phone.len() - 8;
    let end = phone.len() - 4;

    let prefix = &phone[..start];
    let suffix = &phone[end..];

    format!("{}****{}", prefix, suffix)
}

/// 脱敏车牌号
///
/// # 示例
/// - `京A12345` -> `京A***45`
/// - `沪ABC123` -> `沪A***123`
pub fn mask_plate(plate: &str) -> String {
    if plate.len() < 4 {
        return "*".repeat(plate.len());
    }

    // 保留省份简称和最后3位
    if plate.len() <= 7 {
        let end_len = 3.min(plate.len());
        let prefix_len = plate.len() - end_len;
        let prefix = &plate[..prefix_len];
        let suffix = &plate[plate.len() - end_len..];
        let middle = "*".repeat(3.min(end_len));
        format!("{}{}{}", prefix, middle, suffix)
    } else {
        // 新能源车牌等长车牌
        let prefix = &plate[..2];
        let suffix = &plate[plate.len() - 3..];
        format!("{}*****{}", prefix, suffix)
    }
}

/// 脱敏身份证号
///
/// # 示例
/// - `110101199001011234` -> `110101**********1234`
pub fn mask_id_card(id_card: &str) -> String {
    if id_card.len() < 10 {
        return "*".repeat(id_card.len());
    }

    let prefix = &id_card[..6];
    let suffix = &id_card[id_card.len() - 4..];

    format!("{}**********{}", prefix, suffix)
}

/// 脱敏银行卡号
///
/// # 示例
/// - `6222021234567890123` -> `622202**********0123`
pub fn mask_bank_card(card: &str) -> String {
    if card.len() < 8 {
        return "*".repeat(card.len());
    }

    let prefix = &card[..6];
    let suffix = &card[card.len() - 4..];

    format!("{}**********{}", prefix, suffix)
}

/// 脱敏邮箱
///
/// # 示例
/// - `user@example.com` -> `u***@example.com`
/// - `test@company.com.cn` -> `t***@company.com.cn`
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let local = &email[..at_pos];
        let domain = &email[at_pos..];

        if local.is_empty() {
            return format!("_***@{}", domain.trim_start_matches('@'));
        }

        let masked_local = if local.len() <= 2 {
            "*".repeat(local.len())
        } else {
            format!("{}***", &local[..1])
        };

        format!("{}{}", masked_local, domain)
    } else {
        mask_value(email, &SensitiveDataType::Custom("unknown".to_string()))
    }
}

/// 脱敏密码
///
/// # 示例
/// - `mypassword123` -> `***********`
pub fn mask_password(password: &str) -> String {
    "*".repeat(password.len().min(20))
}

/// 通用脱敏函数
pub fn mask_value(value: &str, data_type: &SensitiveDataType) -> String {
    match data_type {
        SensitiveDataType::PhoneNumber => mask_phone(value),
        SensitiveDataType::LicensePlate => mask_plate(value),
        SensitiveDataType::IdCard => mask_id_card(value),
        SensitiveDataType::BankCard => mask_bank_card(value),
        SensitiveDataType::Password => mask_password(value),
        SensitiveDataType::Email => mask_email(value),
        SensitiveDataType::Address => mask_address(value),
        SensitiveDataType::Custom(_) => {
            // 自定义类型默认脱敏中间部分
            if value.len() <= 4 {
                "*".repeat(value.len())
            } else {
                let prefix_len = value.len() / 4;
                let suffix_len = value.len() / 4;
                format!(
                    "{}****{}",
                    &value[..prefix_len],
                    &value[value.len() - suffix_len..]
                )
            }
        }
    }
}

/// 脱敏地址
pub fn mask_address(address: &str) -> String {
    if address.len() <= 8 {
        return "*".repeat(address.len());
    }

    // 保留省市区信息，隐藏详细地址
    let chars: Vec<char> = address.chars().collect();
    if chars.len() <= 10 {
        format!("{}****", &address[..6])
    } else {
        format!("{}****{}", &address[..8], &address[address.len() - 4..])
    }
}

/// SQL 注入检测
///
/// # 参数
/// - `input`: 待检测的字符串
///
/// # 返回
/// - `true`: 存在注入风险
/// - `false`: 未检测到注入风险
pub fn detect_sql_injection(input: &str) -> bool {
    let dangerous_patterns = [
        // SQL 注释
        "--",
        "/*",
        "*/",
        // SQL 关键字
        "UNION",
        "SELECT",
        "INSERT",
        "UPDATE",
        "DELETE",
        "DROP",
        "CREATE",
        "ALTER",
        "EXEC",
        "EXECUTE",
        "SCRIPT",
        // 逻辑操作
        "OR 1",
        "OR TRUE",
        "AND 1",
        "AND TRUE",
        // 字符串闭合
        "' OR '",
        "' OR \"",
        "';--",
        "\";--",
        // 其他
        "INFORMATION_SCHEMA",
        "LOAD_FILE",
        "INTO OUTFILE",
        "INTO DUMPFILE",
    ];

    let lower_input = input.to_lowercase();

    for pattern in &dangerous_patterns {
        if lower_input.contains(&pattern.to_lowercase()) {
            return true;
        }
    }

    false
}

/// SQL 注入防护 - 转义特殊字符
pub fn escape_sql_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\x00', "\\0")
        .replace('\x1a', "\\Z")
}

/// 审计日志操作类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditAction {
    /// 登录
    Login,
    /// 登出
    Logout,
    /// 登录失败
    LoginFailed,
    /// 查看
    View,
    /// 创建
    Create,
    /// 更新
    Update,
    /// 删除
    Delete,
    /// 导出
    Export,
    /// 导入
    Import,
    /// 敏感操作
    SensitiveOp,
    /// 配置变更
    ConfigChange,
    /// 权限变更
    PermissionChange,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Login => write!(f, "LOGIN"),
            Self::Logout => write!(f, "LOGOUT"),
            Self::LoginFailed => write!(f, "LOGIN_FAILED"),
            Self::View => write!(f, "VIEW"),
            Self::Create => write!(f, "CREATE"),
            Self::Update => write!(f, "UPDATE"),
            Self::Delete => write!(f, "DELETE"),
            Self::Export => write!(f, "EXPORT"),
            Self::Import => write!(f, "IMPORT"),
            Self::SensitiveOp => write!(f, "SENSITIVE_OP"),
            Self::ConfigChange => write!(f, "CONFIG_CHANGE"),
            Self::PermissionChange => write!(f, "PERMISSION_CHANGE"),
        }
    }
}

/// 审计日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// 操作者 ID
    pub user_id: Option<i64>,
    /// 操作者名称
    pub username: Option<String>,
    /// 操作类型
    pub action: AuditAction,
    /// 操作目标类型
    pub resource_type: String,
    /// 操作目标 ID
    pub resource_id: Option<String>,
    /// 操作描述
    pub description: String,
    /// 操作参数（已脱敏）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<String>,
    /// 客户端 IP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    /// 用户代理
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// 操作结果
    pub success: bool,
    /// 错误信息（失败时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AuditLogEntry {
    /// 创建审计日志条目
    pub fn new(
        user_id: Option<i64>,
        username: Option<String>,
        action: AuditAction,
        resource_type: &str,
        resource_id: Option<String>,
        description: &str,
    ) -> Self {
        Self {
            user_id,
            username,
            action,
            resource_type: resource_type.to_string(),
            resource_id,
            description: description.to_string(),
            params: None,
            client_ip: None,
            user_agent: None,
            success: true,
            error_message: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 记录失败操作
    pub fn with_failure(mut self, error: &str) -> Self {
        self.success = false;
        self.error_message = Some(error.to_string());
        self
    }

    /// 添加客户端信息
    pub fn with_client_info(mut self, ip: Option<String>, user_agent: Option<String>) -> Self {
        self.client_ip = ip;
        self.user_agent = user_agent;
        self
    }

    /// 添加参数（会自动脱敏）
    pub fn with_params(mut self, params: serde_json::Value) -> Self {
        // 对敏感字段进行脱敏
        let sanitized = sanitize_params_for_audit(params);
        self.params = Some(sanitized.to_string());
        self
    }

    /// 记录审计日志
    pub fn log(&self) {
        let action_str = self.action.to_string();

        if self.success {
            tracing::info!(
                user_id = ?self.user_id,
                username = %self.username.as_ref().unwrap_or(&"anonymous".to_string()),
                action = %action_str,
                resource_type = %self.resource_type,
                resource_id = ?self.resource_id,
                client_ip = ?self.client_ip,
                "{}",
                self.description
            );
        } else {
            tracing::warn!(
                user_id = ?self.user_id,
                username = %self.username.as_ref().unwrap_or(&"anonymous".to_string()),
                action = %action_str,
                resource_type = %self.resource_type,
                resource_id = ?self.resource_id,
                error = %self.error_message.as_ref().unwrap_or(&"Unknown error".to_string()),
                client_ip = ?self.client_ip,
                "{}",
                self.description
            );
        }
    }
}

/// 审计日志参数脱敏
fn sanitize_params_for_audit(params: serde_json::Value) -> serde_json::Value {
    let sensitive_keys = [
        "password",
        "passwd",
        "pwd",
        "secret",
        "token",
        "api_key",
        "apikey",
        "auth",
        "credential",
        "credit_card",
        "card_number",
        "cvv",
        "ssn",
    ];

    match params {
        serde_json::Value::Object(map) => {
            let sanitized: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .map(|(key, value)| {
                    let lower_key = key.to_lowercase();
                    let is_sensitive = sensitive_keys.iter().any(|s| lower_key.contains(s));

                    if is_sensitive {
                        (key, serde_json::Value::String("[REDACTED]".to_string()))
                    } else {
                        (key, sanitize_params_for_audit(value))
                    }
                })
                .collect();
            serde_json::Value::Object(sanitized)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(sanitize_params_for_audit).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_phone() {
        assert_eq!(mask_phone("13812345678"), "138****5678");
        assert_eq!(mask_phone("1234567"), "*******");
    }

    #[test]
    fn test_mask_plate() {
        assert_eq!(mask_plate("京A12345"), "京A***45");
        assert_eq!(mask_plate("沪ABC123"), "沪A***123");
    }

    #[test]
    fn test_mask_id_card() {
        assert_eq!(mask_id_card("110101199001011234"), "110101**********1234");
    }

    #[test]
    fn test_detect_sql_injection() {
        assert!(detect_sql_injection("'; DROP TABLE users;--"));
        assert!(detect_sql_injection("1 OR 1=1"));
        assert!(detect_sql_injection("normal input"));
    }

    #[test]
    fn test_escape_sql_string() {
        assert_eq!(escape_sql_string("O'Reilly"), "O\\'Reilly");
        assert_eq!(escape_sql_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_audit_log_entry() {
        let entry = AuditLogEntry::new(
            Some(1),
            Some("admin".to_string()),
            AuditAction::Login,
            "session",
            None,
            "User logged in",
        );

        entry.log();
    }
}
