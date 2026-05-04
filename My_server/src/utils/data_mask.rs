//! 数据脱敏工具模块
//! 用于在日志、API响应、审计记录中保护敏感个人信息

use regex::Regex;
use serde::{Deserialize, Serialize};

/// 脱敏类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaskType {
    /// 手机号：138****5678
    Phone,
    /// 身份证：110101**********1234
    IdCard,
    /// 银行卡：622202**********0123
    BankCard,
    /// 邮箱：z****n@example.com
    Email,
    /// 姓名：张* / 欧*锋
    Name,
    /// 地址：隐藏具体门牌号
    Address,
    /// 车牌：京*2345
    LicensePlate,
    /// 密码：完全掩码
    Password,
    /// 自定义：只显示首尾字符
    Custom,
}

/// 手机号脱敏
/// 保留前3位和后4位
pub fn mask_phone(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 11 {
        return phone.to_string();
    }
    format!("{}****{}", &digits[..3], &digits[digits.len() - 4..])
}

/// 身份证号脱敏
/// 保留前6位和后4位
pub fn mask_id_card(id_card: &str) -> String {
    let cleaned: String = id_card.chars().filter(|c| c.is_ascii_digit() || c.is_ascii_alphabetic()).collect();
    if cleaned.len() != 18 {
        return id_card.to_string();
    }
    format!("{}**********{}", &cleaned[..6], &cleaned[cleaned.len() - 4..])
}

/// 银行卡号脱敏
/// 保留前6位和后4位
pub fn mask_bank_card(bank_card: &str) -> String {
    let digits: String = bank_card.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 12 {
        return bank_card.to_string();
    }
    let masked_len = digits.len() - 10;
    let masked = "*".repeat(masked_len);
    format!("{}{}{}", &digits[..6], masked, &digits[digits.len() - 4..])
}

/// 邮箱脱敏
/// 保留第1位、@、域名
pub fn mask_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return email.to_string();
    }

    let local = parts[0];
    let domain = parts[1];

    if local.len() <= 2 {
        return format!("{}@{}", "*".repeat(local.len()), domain);
    }

    format!(
        "{}{}{}@{}",
        local.chars().next().unwrap(),
        "*".repeat(local.len() - 2),
        local.chars().last().unwrap(),
        domain
    )
}

/// 姓名脱敏
/// 单字：张*；双字：张*；复姓：欧*锋
pub fn mask_name(name: &str) -> String {
    let compound_surnames = [
        "欧阳", "司马", "上官", "诸葛", "慕容", "令狐", "公孙", "西门",
        "南宫", "东方", "夏侯", "皇甫", "尉迟", "呼延", "赫连", "澹台",
        "长孙", "宇文", "司徒", "司空",
    ];

    // 检查复姓
    for surname in &compound_surnames {
        if name.starts_with(surname) {
            let remaining = &name[surname.len()..];
            if remaining.is_empty() {
                return format!("{}*", &surname[..1]);
            }
            if remaining.len() == 1 {
                return format!("{}{}*", &surname[..1], remaining);
            }
            return format!(
                "{}{}{}{}",
                &surname[..1],
                "*".repeat(surname.len() - 1),
                "*".repeat(remaining.len() - 1),
                remaining.chars().last().unwrap()
            );
        }
    }

    // 普通姓名
    if name.len() == 1 {
        return "*".to_string();
    }
    if name.len() == 2 {
        return format!("{}*", &name[..1]);
    }

    format!(
        "{}{}{}",
        name.chars().next().unwrap(),
        "*".repeat(name.len() - 2),
        name.chars().last().unwrap()
    )
}

/// 地址脱敏
/// 隐藏具体门牌号
pub fn mask_address(address: &str) -> String {
    let re = Regex::new(r"\d+(号|栋|号楼|单元|室|弄|巷|条|楼|#|-)").unwrap();
    re.replace_all(address, "*").to_string()
}

/// 车牌号脱敏
/// 保留第1位和最后一位
pub fn mask_license_plate(plate: &str) -> String {
    if plate.len() <= 2 {
        return "*".repeat(plate.len());
    }
    format!(
        "{}{}{}",
        plate.chars().next().unwrap(),
        "*".repeat(plate.len() - 2),
        plate.chars().last().unwrap()
    )
}

/// 密码脱敏
/// 完全掩码
pub fn mask_password(_password: &str) -> String {
    "********".to_string()
}

/// 根据脱敏类型自动选择脱敏方式
pub fn mask(value: &str, mask_type: &MaskType) -> String {
    match mask_type {
        MaskType::Phone => mask_phone(value),
        MaskType::IdCard => mask_id_card(value),
        MaskType::BankCard => mask_bank_card(value),
        MaskType::Email => mask_email(value),
        MaskType::Name => mask_name(value),
        MaskType::Address => mask_address(value),
        MaskType::LicensePlate => mask_license_plate(value),
        MaskType::Password => mask_password(value),
        MaskType::Custom => {
            if value.len() <= 4 {
                "*".repeat(value.len())
            } else {
                format!("{}{}{}", value.chars().next().unwrap(), "*".repeat(value.len() - 2), value.chars().last().unwrap())
            }
        }
    }
}

/// 批量脱敏结构体中的多个字段
#[macro_export]
macro_rules! mask_struct_fields {
    ($struct:expr, $($field:ident => $type:expr),*) => {{
        let mut result = $struct.clone();
        $(
            if let Some(ref val) = result.$field {
                result.$field = Some($type(val));
            }
        )*
        result
    }};
}

/// 用户信息脱敏
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedUser {
    pub id: Option<i32>,
    pub username: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub real_name: Option<String>,
    pub id_card: Option<String>,
}

impl From<&crate::domain::entities::user::User> for MaskedUser {
    fn from(user: &crate::domain::entities::user::User) -> Self {
        Self {
            id: Some(user.user_id),
            username: user.username.as_ref().map(|s| {
                if s.len() > 2 {
                    format!("{}***{}", &s[..1], &s[s.len() - 1..])
                } else {
                    "***".to_string()
                }
            }),
            phone: user.phone.as_ref().map(|s| mask_phone(s)),
            email: user.email.as_ref().map(|s| mask_email(s)),
            real_name: user.real_name.as_ref().map(|s| mask_name(s)),
            id_card: user.id_card.as_ref().map(|s| mask_id_card(s)),
        }
    }
}

/// 车辆信息脱敏
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedVehicle {
    pub vehicle_id: Option<i32>,
    pub license_plate: Option<String>,
    pub owner_name: Option<String>,
    pub owner_phone: Option<String>,
    pub owner_id_card: Option<String>,
}

impl From<&crate::domain::entities::vehicle::Vehicle> for MaskedVehicle {
    fn from(vehicle: &crate::domain::entities::vehicle::Vehicle) -> Self {
        Self {
            vehicle_id: Some(vehicle.vehicle_id),
            license_plate: vehicle.license_plate.as_ref().map(|s| mask_license_plate(s)),
            owner_name: vehicle.owner_name.as_ref().map(|s| mask_name(s)),
            owner_phone: vehicle.owner_phone.as_ref().map(|s| mask_phone(s)),
            owner_id_card: vehicle.owner_id_card.as_ref().map(|s| mask_id_card(s)),
        }
    }
}

/// 司机信息脱敏
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedDriver {
    pub driver_id: Option<i32>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub id_card: Option<String>,
}

impl From<&crate::domain::entities::driver::Driver> for MaskedDriver {
    fn from(driver: &crate::domain::entities::driver::Driver) -> Self {
        Self {
            driver_id: Some(driver.driver_id),
            name: driver.name.as_ref().map(|s| mask_name(s)),
            phone: driver.phone.as_ref().map(|s| mask_phone(s)),
            id_card: driver.id_card.as_ref().map(|s| mask_id_card(s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_phone() {
        assert_eq!(mask_phone("13812345678"), "138****5678");
        assert_eq!(mask_phone("1381234567"), "1381234567"); // 不足11位不脱敏
    }

    #[test]
    fn test_mask_id_card() {
        assert_eq!(mask_id_card("110101199001011234"), "110101**********1234");
    }

    #[test]
    fn test_mask_bank_card() {
        assert_eq!(mask_bank_card("6222021234567890123"), "622202**********0123");
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("zhangsan@example.com"), "z****n@example.com");
        assert_eq!(mask_email("ab@example.com"), "**@example.com");
    }

    #[test]
    fn test_mask_name() {
        assert_eq!(mask_name("张三"), "张*");
        assert_eq!(mask_name("欧阳锋"), "欧**锋");
        assert_eq!(mask_name("张"), "*");
    }

    #[test]
    fn test_mask_address() {
        assert_eq!(mask_address("北京市朝阳区建国路88号1号楼201室"), "北京市朝阳区建国路*号**室");
    }

    #[test]
    fn test_mask_license_plate() {
        assert_eq!(mask_license_plate("京A12345"), "京*2345");
    }
}
