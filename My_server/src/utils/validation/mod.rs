//! / 增强的输入验证模块
// 提供全面的输入验证功能,包括特殊字符检查和注入攻击防护

use regex::Regex;

use crate::errors::{AppError, AppResult};

/// 验证工具结构体
pub struct ValidationUtils;

impl ValidationUtils {
    /// 验证字符串非空
    pub fn validate_non_empty(value: &str, field_name: &str) -> AppResult<()> {
        if value.trim().is_empty() {
            let error_msg = format!("{}不能为空", field_name);
            return Err(AppError::validation(&error_msg));
        }
        Ok(())
    }

    /// 验证字符串长度
    pub fn validate_length(value: &str, min: usize, max: usize, field_name: &str) -> AppResult<()> {
        let len = value.len();
        if len < min {
            let error_msg = format!("{}长度不能少于{}个字符", field_name, min);
            return Err(AppError::validation(&error_msg));
        }
        if len > max {
            let error_msg = format!("{}长度不能超过{}个字符", field_name, max);
            return Err(AppError::validation(&error_msg));
        }
        Ok(())
    }

    /// 验证数字范围
    pub fn validate_range<T>(value: T, min: T, max: T, field_name: &str) -> AppResult<()>
    where
        T: PartialOrd + std::fmt::Display,
    {
        if value < min {
            let error_msg = format!("{}不能小于{}", field_name, min);
            return Err(AppError::validation(&error_msg));
        }
        if value > max {
            let error_msg = format!("{}不能大于{}", field_name, max);
            return Err(AppError::validation(&error_msg));
        }
        Ok(())
    }

    /// 验证邮箱格式
    pub fn validate_email(email: &str) -> AppResult<()> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("valid email regex always compiles");
        if !email_regex.is_match(email) {
            return Err(AppError::validation("邮箱格式不正确"));
        }
        Ok(())
    }

    /// 验证电话号码格式
    pub fn validate_phone(phone: &str) -> AppResult<()> {
        let phone_regex = Regex::new(r"^1[3-9]\d{9}$").expect("valid phone regex always compiles");
        if !phone_regex.is_match(phone) {
            return Err(AppError::validation("电话号码格式不正确"));
        }
        Ok(())
    }

    /// 验证身份证号格式
    pub fn validate_id_card(id_card: &str) -> AppResult<()> {
        let id_card_regex = Regex::new(
            r"^[1-9]\d{5}(18|19|20)\d{2}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])\d{3}[\dXx]$",
        )
        .expect("valid id_card regex always compiles");
        if !id_card_regex.is_match(id_card) {
            return Err(AppError::validation("身份证号格式不正确"));
        }
        Ok(())
    }

    /// 验证车牌号格式
    pub fn validate_license_plate(plate: &str) -> AppResult<()> {
        // 简单的车牌号验证
        if plate.len() < 7 || plate.len() > 10 {
            return Err(AppError::validation("车牌号格式不正确"));
        }
        Ok(())
    }

    /// 检查并清理SQL注入风险
    pub fn sanitize_sql_input(input: &str) -> String {
        // 移除常见的SQL注入关键字
        let mut sanitized = input.to_string();
        let sql_keywords = [
            "DROP", "DELETE", "UPDATE", "INSERT", "SELECT", "FROM", "WHERE", "AND", "OR", "NOT",
            "JOIN", "UNION", "GROUP", "ORDER", "EXEC", "EXECUTE", "CALL", "DECLARE", "TRUNCATE",
            "ALTER", "CREATE",
        ];

        for keyword in &sql_keywords {
            let pattern = format!("(?i){}", keyword);
            let regex = Regex::new(&pattern).expect("keyword pattern always compiles");
            sanitized = regex.replace_all(&sanitized, "").to_string();
        }

        // 移除危险字符
        sanitized = sanitized
            .replace(';', "")
            .replace("--", "")
            .replace(
                [
                    '\'', '\"', '\\', '|', '&', '<', '>', '^', '%', '$', '+', '=',
                ],
                "",
            )
            .trim()
            .to_string();

        sanitized
    }

    /// 检查并清理XSS风险
    pub fn sanitize_xss_input(input: &str) -> String {
        let mut sanitized = input.to_string();

        // 移除HTML标签
        let html_regex = Regex::new(r"<[^>]*>").expect("html tag regex always compiles");
        sanitized = html_regex.replace_all(&sanitized, "").to_string();

        // 转义特殊字符
        sanitized = sanitized
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\"', "&quot;")
            .replace('\'', "&#39;")
            .trim()
            .to_string();

        sanitized
    }

    /// 验证输入是否包含危险字符
    pub fn validate_no_dangerous_chars(value: &str, field_name: &str) -> AppResult<()> {
        let dangerous_chars = [
            ";", "--", "'", "\"", "\\", "|", "&", "<", ">", "^", "%", "$", "+", "=",
        ];

        for &char_seq in &dangerous_chars {
            if value.contains(char_seq) {
                let error_msg = format!("{}不能包含特殊字符", field_name);
                return Err(AppError::validation(&error_msg));
            }
        }

        Ok(())
    }

    /// 验证输入是否包含SQL注入关键字
    pub fn validate_no_sql_injection(value: &str, field_name: &str) -> AppResult<()> {
        let sql_keywords = [
            "DROP", "DELETE", "UPDATE", "INSERT", "SELECT", "FROM", "WHERE", "AND", "OR", "NOT",
            "JOIN", "UNION", "GROUP", "ORDER", "EXEC", "EXECUTE", "CALL", "DECLARE", "TRUNCATE",
            "ALTER", "CREATE",
        ];

        let value_lower = value.to_lowercase();
        for keyword in &sql_keywords {
            if value_lower.contains(&keyword.to_lowercase()) {
                let error_msg = format!("{}不能包含SQL关键字", field_name);
                return Err(AppError::validation(&error_msg));
            }
        }

        Ok(())
    }

    /// 验证输入是否包含XSS攻击代码
    pub fn validate_no_xss(value: &str, field_name: &str) -> AppResult<()> {
        let xss_patterns = [
            "<script",
            "</script>",
            "javascript:",
            "onerror=",
            "onload=",
            "onclick=",
            "onmouseover=",
            "onkeydown=",
            "onkeyup=",
            "onfocus=",
        ];

        let value_lower = value.to_lowercase();
        for pattern in &xss_patterns {
            if value_lower.contains(pattern) {
                let error_msg = format!("{}不能包含脚本代码", field_name);
                return Err(AppError::validation(&error_msg));
            }
        }

        Ok(())
    }

    /// 验证密码强度
    pub fn validate_password_strength(password: &str) -> AppResult<()> {
        if password.len() < 8 {
            return Err(AppError::validation("密码长度不能少于8个字符"));
        }

        // 检查是否包含至少一个数字
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        if !has_digit {
            return Err(AppError::validation("密码必须包含至少一个数字"));
        }

        // 检查是否包含至少一个字母
        let has_letter = password.chars().any(|c| c.is_alphabetic());
        if !has_letter {
            return Err(AppError::validation("密码必须包含至少一个字母"));
        }

        // 检查是否包含至少一个特殊字符
        let has_special = password
            .chars()
            .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        if !has_special {
            return Err(AppError::validation("密码必须包含至少一个特殊字符"));
        }

        Ok(())
    }

    /// 批量验证
    pub fn validate_all(validations: Vec<AppResult<()>>) -> AppResult<()> {
        for result in validations {
            result?;
        }
        Ok(())
    }
}

/// 验证扩展特性
pub trait ValidationExt {
    /// 验证并清理输入
    fn validate_and_sanitize(&self) -> AppResult<String>;
}

impl ValidationExt for String {
    /// 验证并清理字符串输入
    fn validate_and_sanitize(&self) -> AppResult<String> {
        // 首先验证非空
        ValidationUtils::validate_non_empty(self, "输入")?;

        // 清理XSS风险
        let sanitized = ValidationUtils::sanitize_xss_input(self);

        Ok(sanitized)
    }
}

/// 常用验证规则
pub mod rules {
    use super::*;

    /// 验证用户名
    pub fn validate_username(username: &str) -> AppResult<()> {
        ValidationUtils::validate_non_empty(username, "用户名")?;
        ValidationUtils::validate_length(username, 3, 20, "用户名")?;
        ValidationUtils::validate_no_dangerous_chars(username, "用户名")?;
        Ok(())
    }

    /// 验证密码
    pub fn validate_password(password: &str) -> AppResult<()> {
        ValidationUtils::validate_non_empty(password, "密码")?;
        ValidationUtils::validate_password_strength(password)?;
        Ok(())
    }

    /// 验证邮箱
    pub fn validate_email(email: &str) -> AppResult<()> {
        ValidationUtils::validate_non_empty(email, "邮箱")?;
        ValidationUtils::validate_email(email)?;
        Ok(())
    }

    /// 验证电话号码
    pub fn validate_phone(phone: &str) -> AppResult<()> {
        ValidationUtils::validate_non_empty(phone, "电话号码")?;
        ValidationUtils::validate_phone(phone)?;
        Ok(())
    }

    /// 验证身份证号
    pub fn validate_id_card(id_card: &str) -> AppResult<()> {
        ValidationUtils::validate_non_empty(id_card, "身份证号")?;
        ValidationUtils::validate_id_card(id_card)?;
        Ok(())
    }

    /// 验证车牌号
    pub fn validate_license_plate(plate: &str) -> AppResult<()> {
        ValidationUtils::validate_non_empty(plate, "车牌号")?;
        ValidationUtils::validate_license_plate(plate)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty() {
        assert!(ValidationUtils::validate_non_empty("test", "测试").is_ok());
        assert!(ValidationUtils::validate_non_empty("", "测试").is_err());
        assert!(ValidationUtils::validate_non_empty("   ", "测试").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(ValidationUtils::validate_length("test", 3, 10, "测试").is_ok());
        assert!(ValidationUtils::validate_length("te", 3, 10, "测试").is_err());
        assert!(ValidationUtils::validate_length("testtesttest", 3, 10, "测试").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(ValidationUtils::validate_email("test@example.com").is_ok());
        assert!(ValidationUtils::validate_email("test@").is_err());
        assert!(ValidationUtils::validate_email("test").is_err());
    }

    #[test]
    fn test_validate_phone() {
        assert!(ValidationUtils::validate_phone("13800138000").is_ok());
        assert!(ValidationUtils::validate_phone("1234567890").is_err());
        assert!(ValidationUtils::validate_phone("138001380000").is_err());
    }

    #[test]
    fn test_sanitize_sql_input() {
        let input = "SELECT * FROM users WHERE id = 1; -- drop table users";
        let sanitized = ValidationUtils::sanitize_sql_input(input);
        assert!(!sanitized.contains("SELECT"));
        assert!(!sanitized.contains("FROM"));
        assert!(!sanitized.contains("WHERE"));
        assert!(!sanitized.contains(";"));
        assert!(!sanitized.contains("--"));
    }

    #[test]
    fn test_sanitize_xss_input() {
        let input = "<script>alert('XSS')</script>";
        let sanitized = ValidationUtils::sanitize_xss_input(input);
        assert!(!sanitized.contains("<script>"));
        assert!(!sanitized.contains("</script>"));
    }

    #[test]
    fn test_validate_no_dangerous_chars() {
        assert!(ValidationUtils::validate_no_dangerous_chars("test", "测试").is_ok());
        assert!(ValidationUtils::validate_no_dangerous_chars("test; drop", "测试").is_err());
        assert!(ValidationUtils::validate_no_dangerous_chars("test--", "测试").is_err());
    }

    #[test]
    fn test_validate_password_strength() {
        assert!(ValidationUtils::validate_password_strength("Test123!").is_ok());
        assert!(ValidationUtils::validate_password_strength("test").is_err());
        assert!(ValidationUtils::validate_password_strength("12345678").is_err());
        assert!(ValidationUtils::validate_password_strength("Testtest").is_err());
        assert!(ValidationUtils::validate_password_strength("Test123").is_err());
    }
}
