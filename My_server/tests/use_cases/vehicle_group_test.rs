//! 车组用例集成测试
//!
//! 独立的集成测试，不依赖内嵌测试模块

use std::sync::Arc;
use carptms::domain::use_cases::vehicle_group::VehicleGroupUseCases;
use carptms::domain::use_cases::vehicle_group::repository::VehicleGroupRepository;
use carptms::domain::entities::vehicle_group::{
    VehicleGroup, VehicleGroupCreateRequest, VehicleGroupTreeNode, VehicleGroupUpdateRequest,
};
use chrono::Utc;

#[allow(dead_code)]
struct MockVehicleGroupRepo {
    groups: Vec<VehicleGroup>,
    has_related_data: bool,
}

#[async_trait::async_trait]
impl VehicleGroupRepository for MockVehicleGroupRepo {
    async fn find_all(
        &self,
        _page: i32,
        _page_size: i32,
    ) -> Result<(Vec<VehicleGroup>, i64), anyhow::Error> {
        Ok((self.groups.clone(), self.groups.len() as i64))
    }

    async fn find_by_id(&self, group_id: i32) -> Result<Option<VehicleGroup>, anyhow::Error> {
        Ok(self.groups.iter().find(|g| g.group_id == group_id).cloned())
    }

    async fn create(
        &self,
        group: VehicleGroupCreateRequest,
    ) -> Result<VehicleGroup, anyhow::Error> {
        let now = Utc::now();
        Ok(VehicleGroup {
            group_id: self.groups.len() as i32 + 1,
            group_name: group.group_name,
            parent_id: group.parent_id,
            parent_name: None,
            description: group.description,
            vehicle_count: 0,
            create_time: now,
            update_time: None,
        })
    }

    async fn update(
        &self,
        group_id: i32,
        group: VehicleGroupUpdateRequest,
    ) -> Result<VehicleGroup, anyhow::Error> {
        if let Some(mut existing_group) = self.find_by_id(group_id).await? {
            if let Some(group_name) = group.group_name {
                existing_group.group_name = group_name;
            }
            if let Some(description) = group.description {
                existing_group.description = Some(description);
            }
            existing_group.update_time = Some(Utc::now());
            Ok(existing_group)
        } else {
            Err(anyhow::anyhow!("车组不存在"))
        }
    }

    async fn delete(&self, group_id: i32) -> Result<(), anyhow::Error> {
        if self.groups.iter().any(|g| g.group_id == group_id) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("车组不存在"))
        }
    }

    async fn has_related_data(&self, _group_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.has_related_data)
    }

    async fn get_tree(&self) -> Result<Vec<VehicleGroupTreeNode>, anyhow::Error> {
        Ok(Vec::new())
    }

    async fn exists(&self, group_id: i32) -> Result<bool, anyhow::Error> {
        Ok(self.groups.iter().any(|g| g.group_id == group_id))
    }

    async fn count_by_name(
        &self,
        name: &str,
        exclude_id: Option<i32>,
    ) -> Result<i64, anyhow::Error> {
        Ok(self
            .groups
            .iter()
            .filter(|g| {
                g.group_name == name && exclude_id.map(|id| g.group_id != id).unwrap_or(true)
            })
            .count() as i64)
    }

    async fn count_vehicles(&self, _group_id: i32) -> Result<i64, anyhow::Error> {
        Ok(0)
    }

    async fn count_children(&self, _group_id: i32) -> Result<i64, anyhow::Error> {
        Ok(0)
    }
}

#[tokio::test]
async fn test_create_vehicle_group_invalid_name() {
    let group_create = VehicleGroupCreateRequest {
        group_name: "".to_string(),
        parent_id: None,
        description: Some("测试描述".to_string()),
    };

    let mock_repo = Arc::new(MockVehicleGroupRepo {
        groups: Vec::new(),
        has_related_data: false,
    });

    let use_cases = VehicleGroupUseCases::new(mock_repo);
    let result: Result<VehicleGroup, anyhow::Error> =
        use_cases.create_vehicle_group(group_create).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "车组名称不能为空");
}

#[tokio::test]
async fn test_delete_vehicle_group_with_related_data() {
    let now = Utc::now();
    let group = VehicleGroup {
        group_id: 1,
        group_name: "测试车组".to_string(),
        parent_id: None,
        parent_name: None,
        description: Some("测试描述".to_string()),
        vehicle_count: 5,
        create_time: now,
        update_time: None,
    };

    let mock_repo = Arc::new(MockVehicleGroupRepo {
        groups: vec![group],
        has_related_data: true,
    });

    let use_cases = VehicleGroupUseCases::new(mock_repo);
    let result: Result<bool, anyhow::Error> = use_cases.delete_vehicle_group(1).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "车组有关联数据，无法删除");
}
