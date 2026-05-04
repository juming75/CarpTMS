//! 车辆服务
//!
//! 实现车辆 CRUD 业务逻辑

use std::sync::Arc;

use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};
use crate::domain::use_cases::vehicle::repository::VehicleRepository;
use crate::redis::{del_cache_pattern, get_cache, set_cache};

/// 车辆用例结构
#[derive(Clone)]
pub struct VehicleUseCases {
    vehicle_repository: Arc<dyn VehicleRepository + Send + Sync>,
}

impl VehicleUseCases {
    /// 创建车辆用例实例
    pub fn new(vehicle_repository: Arc<dyn VehicleRepository>) -> Self {
        Self { vehicle_repository }
    }

    /// 获取车辆列表用例
    pub async fn get_vehicles(
        &self,
        query: VehicleQuery,
    ) -> Result<(Vec<Vehicle>, i64), anyhow::Error> {
        // 构建缓存键
        let cache_key = format!(
            "vehicles:list:name_{}:plate_{}:type_{}:status_{}:page_{}:size_{}",
            query.vehicle_name.as_deref().unwrap_or(""),
            query.license_plate.as_deref().unwrap_or(""),
            query.vehicle_type.as_deref().unwrap_or(""),
            query
                .status
                .map(|s| s.to_string())
                .unwrap_or("".to_string()),
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20)
        );

        // 尝试从缓存获取
        if let Ok(Some(cached)) = get_cache::<(Vec<Vehicle>, i64)>(&cache_key).await {
            return Ok(cached);
        }

        // 从数据库获取
        let result = self.vehicle_repository.get_vehicles(query).await?;

        // 缓存结果,过期时间30分钟
        let _ = set_cache(&cache_key, &result, 1800).await;

        Ok(result)
    }

    /// 获取单个车辆用例
    pub async fn get_vehicle(&self, vehicle_id: i32) -> Result<Option<Vehicle>, anyhow::Error> {
        // 构建缓存键
        let cache_key = format!("vehicle:{}:details", vehicle_id);

        // 尝试从缓存获取
        if let Ok(Some(cached)) = get_cache::<Option<Vehicle>>(&cache_key).await {
            return Ok(cached);
        }

        // 从数据库获取
        let result = self.vehicle_repository.get_vehicle(vehicle_id).await?;

        // 缓存结果,过期时间30分钟
        let _ = set_cache(&cache_key, &result, 1800).await;

        Ok(result)
    }

    /// 创建车辆用例
    pub async fn create_vehicle(&self, vehicle: VehicleCreate) -> Result<Vehicle, anyhow::Error> {
        // 业务逻辑:数据验证
        if vehicle.vehicle_name.is_empty() {
            return Err(anyhow::anyhow!("车辆名称不能为空"));
        }

        if vehicle.license_plate.is_empty() {
            return Err(anyhow::anyhow!("车牌号不能为空"));
        }

        if vehicle.inspection_date < vehicle.register_date {
            return Err(anyhow::anyhow!("年检日期不能早于注册日期"));
        }

        if vehicle.insurance_date < vehicle.register_date {
            return Err(anyhow::anyhow!("保险日期不能早于注册日期"));
        }

        // 调用仓库创建车辆
        let created_vehicle = self.vehicle_repository.create_vehicle(vehicle).await?;

        // 清理相关缓存
        let _ = del_cache_pattern("vehicles:list:*").await;
        let _ = del_cache_pattern("statistics:vehicles").await;

        Ok(created_vehicle)
    }

    /// 更新车辆用例
    pub async fn update_vehicle(
        &self,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> Result<Option<Vehicle>, anyhow::Error> {
        // 业务逻辑:数据验证
        if let Some(vehicle_name) = &vehicle.vehicle_name {
            if vehicle_name.is_empty() {
                return Err(anyhow::anyhow!("车辆名称不能为空"));
            }
        }

        if let Some(license_plate) = &vehicle.license_plate {
            if license_plate.is_empty() {
                return Err(anyhow::anyhow!("车牌号不能为空"));
            }
        }

        // 检查年检日期不能早于注册日期
        if let (Some(register_date), Some(inspection_date)) =
            (vehicle.register_date, vehicle.inspection_date)
        {
            if inspection_date < register_date {
                return Err(anyhow::anyhow!("年检日期不能早于注册日期"));
            }
        }

        // 检查保险日期不能早于注册日期
        if let (Some(register_date), Some(insurance_date)) =
            (vehicle.register_date, vehicle.insurance_date)
        {
            if insurance_date < register_date {
                return Err(anyhow::anyhow!("保险日期不能早于注册日期"));
            }
        }

        // 调用仓库更新车辆
        let updated_vehicle = self
            .vehicle_repository
            .update_vehicle(vehicle_id, vehicle)
            .await?;

        // 清理相关缓存
        if updated_vehicle.is_some() {
            let _ = del_cache_pattern(&format!("vehicle:{}:*", vehicle_id)).await;
            let _ = del_cache_pattern("vehicles:list:*").await;
            let _ = del_cache_pattern("statistics:vehicles").await;
        }

        Ok(updated_vehicle)
    }

    /// 删除车辆用例
    pub async fn delete_vehicle(&self, vehicle_id: i32) -> Result<bool, anyhow::Error> {
        // 业务逻辑:检查关联数据
        if let Ok(has_related) = self.vehicle_repository.has_related_data(vehicle_id).await {
            if has_related {
                return Err(anyhow::anyhow!("车辆有关联数据，无法删除"));
            }
        }

        // 调用仓库删除车辆
        let result = self.vehicle_repository.delete_vehicle(vehicle_id).await?;

        // 清理相关缓存
        if result {
            let _ = del_cache_pattern(&format!("vehicle:{}:*", vehicle_id)).await;
            let _ = del_cache_pattern("vehicles:list:*").await;
            let _ = del_cache_pattern("statistics:vehicles").await;
        }

        Ok(result)
    }

    /// 批量获取车辆
    pub async fn get_vehicles_batch(
        &self,
        vehicle_ids: &[i32],
    ) -> Result<Vec<Vehicle>, anyhow::Error> {
        self.vehicle_repository
            .get_vehicles_batch(vehicle_ids)
            .await
    }
}
