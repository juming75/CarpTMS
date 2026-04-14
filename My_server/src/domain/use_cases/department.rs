//! 部门管理领域用例
//! 
//! 包含部门管理的核心业务逻辑，包括部门的增删改查和层级管理

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::domain::entities::department::{Department, DepartmentCreate, DepartmentUpdate, DepartmentQuery};
use crate::domain::ddd::ApplicationService;

// 部门仓库接口
#[async_trait::async_trait]
pub trait DepartmentRepository: Send + Sync {
    async fn get_departments(&self, query: DepartmentQuery) -> Result<(Vec<Department>, i64), anyhow::Error>;
    async fn get_department(&self, department_id: i32) -> Result<Option<Department>, anyhow::Error>;
    async fn create_department(&self, department: DepartmentCreate) -> Result<Department, anyhow::Error>;
    async fn update_department(&self, department_id: i32, department: DepartmentUpdate) -> Result<Option<Department>, anyhow::Error>;
    async fn delete_department(&self, department_id: i32) -> Result<bool, anyhow::Error>;
    async fn has_sub_departments(&self, department_id: i32) -> Result<bool, anyhow::Error>;
}

// 部门用例
pub struct DepartmentUseCases {
    repository: Arc<dyn DepartmentRepository>,
}

impl DepartmentUseCases {
    pub fn new(repository: Arc<dyn DepartmentRepository>) -> Self {
        Self {
            repository,
        }
    }
    
    pub async fn get_departments(&self, query: DepartmentQuery) -> Result<(Vec<Department>, i64)> {
        self.repository.get_departments(query).await
    }
    
    pub async fn get_department(&self, department_id: i32) -> Result<Option<Department>> {
        self.repository.get_department(department_id).await
    }
    
    pub async fn create_department(&self, department: DepartmentCreate) -> Result<Department> {
        // 业务逻辑：验证部门数据
        if department.department_name.is_empty() {
            return Err(anyhow::anyhow!("Department name is required"));
        }
        
        // 验证父部门是否存在
        if let Some(parent_id) = department.parent_department_id {
            if let Ok(None) = self.repository.get_department(parent_id).await {
                return Err(anyhow::anyhow!("Parent department not found"));
            }
        }
        
        self.repository.create_department(department).await
    }
    
    pub async fn update_department(&self, department_id: i32, department: DepartmentUpdate) -> Result<Option<Department>> {
        // 业务逻辑：验证部门数据
        if let Some(department_name) = &department.department_name {
            if department_name.is_empty() {
                return Err(anyhow::anyhow!("Department name is required"));
            }
        }
        
        // 验证父部门是否存在
        if let Some(parent_id) = department.parent_department_id {
            // 防止循环引用
            if parent_id == department_id {
                return Err(anyhow::anyhow!("Department cannot be its own parent"));
            }
            
            if let Ok(None) = self.repository.get_department(parent_id).await {
                return Err(anyhow::anyhow!("Parent department not found"));
            }
        }
        
        self.repository.update_department(department_id, department).await
    }
    
    pub async fn delete_department(&self, department_id: i32) -> Result<bool> {
        // 业务逻辑：检查是否有子部门
        if let Ok(true) = self.repository.has_sub_departments(department_id).await {
            return Err(anyhow::anyhow!("Cannot delete department with sub-departments"));
        }
        
        self.repository.delete_department(department_id).await
    }
}

// 实现 ApplicationService trait
#[async_trait]
impl ApplicationService for DepartmentUseCases {
    fn name(&self) -> &str {
        "department_service"
    }

    async fn initialize(&self) -> crate::errors::AppResult<()> {
        // 初始化逻辑，如果需要的话
        Ok(())
    }

    async fn execute(&self, command: crate::domain::ddd::Command) -> crate::errors::AppResult<crate::domain::ddd::CommandResult> {
        use crate::domain::ddd::{CommandResult, DomainEvent};
        use crate::errors::AppError;
        
        match command.command_type.as_str() {
            "get_departments" => {
                let query = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse command data: {}", e), None))?;
                let (departments, total) = self.get_departments(query).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to get departments: {}", e), None))?;
                let data = serde_json::json!({ "departments": departments, "total": total });
                Ok(CommandResult::success_with_data(vec![], data))
            }
            "get_department" => {
                let department_id = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse department_id: {}", e), None))?;
                let department = self.get_department(department_id).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to get department: {}", e), None))?;
                let data = serde_json::json!({ "department": department });
                Ok(CommandResult::success_with_data(vec![], data))
            }
            "create_department" => {
                let department = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse department data: {}", e), None))?;
                let created_department = self.create_department(department).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to create department: {}", e), None))?;
                let data = serde_json::json!({ "department": created_department });
                let event = DomainEvent::new(
                    "Department",
                    &created_department.department_id.to_string(),
                    "DepartmentCreated",
                    serde_json::to_value(created_department)
                        .map_err(|e| AppError::internal_error(&format!("Failed to serialize department: {}", e), None))?,
                    1
                );
                Ok(CommandResult::success_with_data(vec![event], data))
            }
            "update_department" => {
                let (department_id, department) = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse update data: {}", e), None))?;
                let updated_department = self.update_department(department_id, department).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to update department: {}", e), None))?;
                let data = serde_json::json!({ "department": updated_department });
                if let Some(department) = updated_department {
                    let event = DomainEvent::new(
                        "Department",
                        &department.department_id.to_string(),
                        "DepartmentUpdated",
                        serde_json::to_value(department)
                            .map_err(|e| AppError::internal_error(&format!("Failed to serialize department: {}", e), None))?,
                        1
                    );
                    Ok(CommandResult::success_with_data(vec![event], data))
                } else {
                    Ok(CommandResult::success_with_data(vec![], data))
                }
            }
            "delete_department" => {
                let department_id = serde_json::from_value(command.data)
                    .map_err(|e| AppError::internal_error(&format!("Failed to parse department_id: {}", e), None))?;
                let deleted = self.delete_department(department_id).await
                    .map_err(|e| AppError::internal_error(&format!("Failed to delete department: {}", e), None))?;
                let data = serde_json::json!({ "deleted": deleted });
                if deleted {
                    let event = DomainEvent::new(
                        "Department",
                        &department_id.to_string(),
                        "DepartmentDeleted",
                        serde_json::json!({ "department_id": department_id }),
                        1
                    );
                    Ok(CommandResult::success_with_data(vec![event], data))
                } else {
                    Ok(CommandResult::success_with_data(vec![], data))
                }
            }
            _ => {
                Ok(CommandResult::error(format!("Unknown command: {}", command.command_type)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::department::*;
    
    // 模拟部门仓库
    struct MockDepartmentRepository;
    
    impl DepartmentRepository for MockDepartmentRepository {
        async fn get_departments(&self, _query: DepartmentQuery) -> Result<(Vec<Department>, i64)> {
            Ok((vec![], 0))
        }
        
        async fn get_department(&self, _department_id: i32) -> Result<Option<Department>> {
            Ok(None)
        }
        
        async fn create_department(&self, department: DepartmentCreate) -> Result<Department> {
            Ok(Department {
                department_id: 1,
                department_name: department.department_name,
                parent_department_id: department.parent_department_id,
                parent_department_name: None,
                manager_id: department.manager_id,
                manager_name: None,
                phone: department.phone,
                description: department.description,
                create_time: chrono::Utc::now().naive_utc(),
                update_time: None,
            })
        }
        
        async fn update_department(&self, _department_id: i32, _department: DepartmentUpdate) -> Result<Option<Department>> {
            Ok(None)
        }
        
        async fn delete_department(&self, _department_id: i32) -> Result<bool> {
            Ok(true)
        }
        
        async fn has_sub_departments(&self, _department_id: i32) -> Result<bool> {
            Ok(false)
        }
    }
    
    #[tokio::test]
    async fn test_create_department() {
        let repository = Arc::new(MockDepartmentRepository);
        let use_cases = DepartmentUseCases::new(repository);
        
        // 测试创建部门
        let department = DepartmentCreate {
            department_name: "测试部门".to_string(),
            parent_department_id: None,
            manager_id: None,
            phone: None,
            description: None,
        };
        
        let result = use_cases.create_department(department).await;
        assert!(result.is_ok());
        
        // 测试创建部门失败（部门名为空）
        let department = DepartmentCreate {
            department_name: "".to_string(),
            parent_department_id: None,
            manager_id: None,
            phone: None,
            description: None,
        };
        
        let result = use_cases.create_department(department).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_delete_department() {
        let repository = Arc::new(MockDepartmentRepository);
        let use_cases = DepartmentUseCases::new(repository);
        
        // 测试删除部门
        let result = use_cases.delete_department(1).await;
        assert!(result.is_ok());
    }
}
