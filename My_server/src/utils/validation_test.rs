//! 验证工具单元测试

#[cfg(test)]
mod tests {
    // 通用验证测试
    mod common_validation_tests {
        #[test]
        fn test_email_validation_valid() {
            let valid_emails = vec![
                "test@example.com",
                "user.name@domain.org",
                "admin@sub.domain.com",
            ];
            for email in valid_emails {
                assert!(is_valid_email(email), "Email {} should be valid", email);
            }
        }

        #[test]
        fn test_email_validation_invalid() {
            let invalid_emails = vec![
                "invalid",
                "@nodomain.com",
                "no@",
                "spaces in@email.com",
                "",
            ];
            for email in invalid_emails {
                assert!(!is_valid_email(email), "Email {} should be invalid", email);
            }
        }

        #[test]
        fn test_phone_validation_valid() {
            let valid_phones = vec![
                "13812345678",
                "+8613912345678",
                "010-12345678",
            ];
            for phone in valid_phones {
                assert!(is_valid_phone(phone), "Phone {} should be valid", phone);
            }
        }

        #[test]
        fn test_phone_validation_invalid() {
            let invalid_phones = vec![
                "123",
                "abc",
                "",
                "12-3456-7890",
            ];
            for phone in invalid_phones {
                assert!(!is_valid_phone(phone), "Phone {} should be invalid", phone);
            }
        }

        #[test]
        fn test_url_validation_valid() {
            let valid_urls = vec![
                "http://example.com",
                "https://domain.org/path",
                "https://sub.domain.com:8080/api",
            ];
            for url in valid_urls {
                assert!(is_valid_url(url), "URL {} should be valid", url);
            }
        }

        #[test]
        fn test_url_validation_invalid() {
            let invalid_urls = vec![
                "not-a-url",
                "ftp://",
                "",
            ];
            for url in invalid_urls {
                assert!(!is_valid_url(url), "URL {} should be invalid", url);
            }
        }

        #[test]
        fn test_ip_address_validation_valid() {
            let valid_ips = vec![
                "127.0.0.1",
                "192.168.1.1",
                "0.0.0.0",
            ];
            for ip in valid_ips {
                assert!(is_valid_ip(ip), "IP {} should be valid", ip);
            }
        }

        #[test]
        fn test_ip_address_validation_invalid() {
            let invalid_ips = vec![
                "256.0.0.1",
                "192.168.1",
                "not-an-ip",
                "",
            ];
            for ip in invalid_ips {
                assert!(!is_valid_ip(ip), "IP {} should be invalid", ip);
            }
        }

        // 辅助函数
        fn is_valid_email(email: &str) -> bool {
            if email.is_empty() {
                return false;
            }
            email.contains('@') && email.contains('.') && !email.contains(' ')
        }

        fn is_valid_phone(phone: &str) -> bool {
            if phone.is_empty() {
                return false;
            }
            // 简单验证：只包含数字、可选+号、可选-
            phone.chars().all(|c| c.is_ascii_digit() || c == '+' || c == '-')
        }

        fn is_valid_url(url: &str) -> bool {
            if url.is_empty() {
                return false;
            }
            url.starts_with("http://") || url.starts_with("https://")
        }

        fn is_valid_ip(ip: &str) -> bool {
            if ip.is_empty() {
                return false;
            }
            let parts: Vec<&str> = ip.split('.').collect();
            if parts.len() != 4 {
                return false;
            }
            parts.iter().all(|part| {
                part.parse::<u8>().is_ok()
            })
        }
    }

    // 字符串验证测试
    mod string_validation_tests {
        #[test]
        fn test_length_validation() {
            let min = 3;
            let max = 10;
            assert!(is_length_valid("abc", min, max));
            assert!(is_length_valid("abcdefghij", min, max));
            assert!(!is_length_valid("ab", min, max));
            assert!(!is_length_valid("abcdefghijk", min, max));
        }

        #[test]
        fn test_alphanumeric_validation() {
            assert!(is_alphanumeric("abc123"));
            assert!(!is_alphanumeric("abc-123"));
            assert!(!is_alphanumeric(""));
        }

        #[test]
        fn test_numeric_validation() {
            assert!(is_numeric("123456"));
            assert!(!is_numeric("123.45"));
            assert!(!is_numeric("abc"));
            assert!(!is_numeric(""));
        }

        #[test]
        fn test_alpha_validation() {
            assert!(is_alpha("abc"));
            assert!(!is_alpha("abc123"));
            assert!(!is_alpha(""));
        }

        fn is_length_valid(s: &str, min: usize, max: usize) -> bool {
            let len = s.len();
            len >= min && len <= max
        }

        fn is_alphanumeric(s: &str) -> bool {
            !s.is_empty() && s.chars().all(|c| c.is_alphanumeric())
        }

        fn is_numeric(s: &str) -> bool {
            !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
        }

        fn is_alpha(s: &str) -> bool {
            !s.is_empty() && s.chars().all(|c| c.is_alphabetic())
        }
    }

    // 密码验证测试
    mod password_validation_tests {
        #[test]
        fn test_password_strength() {
            // 弱密码
            assert_eq!(password_strength("123"), PasswordStrength::Weak);
            assert_eq!(password_strength("abc"), PasswordStrength::Weak);

            // 中等密码
            assert_eq!(password_strength("abc123"), PasswordStrength::Medium);

            // 强密码
            assert_eq!(password_strength("Abc123!@#"), PasswordStrength::Strong);
            assert_eq!(password_strength("VeryLongPassword123!"), PasswordStrength::Strong);
        }

        #[test]
        fn test_password_min_length() {
            assert!(!password_meets_min_length("Ab1!", 8));
            assert!(password_meets_min_length("Ab1!1234", 8));
        }

        #[test]
        fn test_password_has_uppercase() {
            assert!(password_has_uppercase("Abc"));
            assert!(!password_has_uppercase("abc"));
        }

        #[test]
        fn test_password_has_lowercase() {
            assert!(password_has_lowercase("Abc"));
            assert!(!password_has_lowercase("ABC"));
        }

        #[test]
        fn test_password_has_digit() {
            assert!(password_has_digit("Abc1"));
            assert!(!password_has_digit("Abc"));
        }

        #[test]
        fn test_password_has_special_char() {
            assert!(password_has_special_char("Abc!@#"));
            assert!(!password_has_special_char("Abc123"));
        }

        #[derive(Debug, PartialEq)]
        enum PasswordStrength {
            Weak,
            Medium,
            Strong,
        }

        fn password_strength(password: &str) -> PasswordStrength {
            let len = password.len();
            let has_upper = password_has_uppercase(password);
            let has_lower = password_has_lowercase(password);
            let has_digit = password_has_digit(password);
            let has_special = password_has_special_char(password);

            let score = [has_upper, has_lower, has_digit, has_special]
                .iter()
                .filter(|&&x| x)
                .count();

            if len >= 8 && score >= 3 {
                PasswordStrength::Strong
            } else if len >= 4 && score >= 2 {
                PasswordStrength::Medium
            } else {
                PasswordStrength::Weak
            }
        }

        fn password_meets_min_length(password: &str, min: usize) -> bool {
            password.len() >= min
        }

        fn password_has_uppercase(password: &str) -> bool {
            password.chars().any(|c| c.is_uppercase())
        }

        fn password_has_lowercase(password: &str) -> bool {
            password.chars().any(|c| c.is_lowercase())
        }

        fn password_has_digit(password: &str) -> bool {
            password.chars().any(|c| c.is_ascii_digit())
        }

        fn password_has_special_char(password: &str) -> bool {
            password.chars().any(|c| !c.is_alphanumeric())
        }
    }

    // JSON 验证测试
    mod json_validation_tests {
        #[test]
        fn test_valid_json() {
            let valid_jsons = vec![
                r#"{"key": "value"}"#,
                r#"[1, 2, 3]"#,
                r#"{"name": "test", "age": 25}"#,
            ];
            for json in valid_jsons {
                assert!(is_valid_json(json), "JSON {} should be valid", json);
            }
        }

        #[test]
        fn test_invalid_json() {
            let invalid_jsons = vec![
                "{key: value}",
                "{'key': 'value'}",
                "not json at all",
                "",
            ];
            for json in invalid_jsons {
                assert!(!is_valid_json(json), "JSON {} should be invalid", json);
            }
        }

        fn is_valid_json(s: &str) -> bool {
            if s.is_empty() {
                return false;
            }
            serde_json::from_str::<serde_json::Value>(s).is_ok()
        }
    }

    // UUID 验证测试
    mod uuid_validation_tests {
        use uuid::Uuid;

        #[test]
        fn test_valid_uuid_generation() {
            let uuid = Uuid::new_v4();
            assert_eq!(uuid.to_string().len(), 36);
        }

        #[test]
        fn test_valid_uuid_parsing() {
            let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
            let result = Uuid::parse_str(uuid_str);
            assert!(result.is_ok());
        }

        #[test]
        fn test_invalid_uuid_parsing() {
            let invalid_uuids = vec![
                "not-a-uuid",
                "550e8400-e29b-41d4-a716",
                "",
            ];
            for uuid_str in invalid_uuids {
                assert!(Uuid::parse_str(uuid_str).is_err());
            }
        }
    }
}
