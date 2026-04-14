//! / 权限数据转换器
use anyhow::Result;

/// 权限数据转换器
pub struct PermissionTransformer;

impl PermissionTransformer {
    /// 创建新的权限数据转换器
    pub fn new() -> Self {
        Self
    }

    /// 将权限字符串转换为权限列表
    pub fn parse_permissions(&self, permission_str: &str) -> Result<Vec<String>> {
        Ok(permission_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// 将权限列表转换为权限字符串
    pub fn format_permissions(&self, permissions: &[String]) -> String {
        permissions.join(",")
    }
}

impl Default for PermissionTransformer {
    fn default() -> Self {
        Self::new()
    }
}
