//! / 车辆领域用例

use std::sync::Arc;

use crate::domain::entities::vehicle::{Vehicle, VehicleCreate, VehicleQuery, VehicleUpdate};
use crate::redis::{del_cache_pattern, get_cache, set_cache};

/// 车辆仓库接口
#[async_trait::async_trait]
pub trait VehicleRepository: Send + Sync {
    /// 获取车辆列表
    async fn get_vehicles(&self, query: VehicleQuery)
        -> Result<(Vec<Vehicle>, i64), anyhow::Error>;

    /// 获取单个车辆
    async fn get_vehicle(&self, vehicle_id: i32) -> Result<Option<Vehicle>, anyhow::Error>;

    /// 批量获取车辆信息 (数据库批量查询优化)
    async fn get_vehicles_batch(&self, vehicle_ids: &[i32]) -> Result<Vec<Vehicle>, anyhow::Error>;

    /// 创建车辆
    async fn create_vehicle(&self, vehicle: VehicleCreate) -> Result<Vehicle, anyhow::Error>;

    /// 更新车辆
    async fn update_vehicle(
        &self,
        vehicle_id: i32,
        vehicle: VehicleUpdate,
    ) -> Result<Option<Vehicle>, anyhow::Error>;

    /// 删除车辆
    async fn delete_vehicle(&self, vehicle_id: i32) -> Result<bool, anyhow::Error>;
    
    /// 检查车辆是否有关联数据
    async fn has_related_data(&self, vehicle_id: i32) -> Result<bool, anyhow::Error>;
}

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
            query.status.map(|s| s.to_string()).unwrap_or("".to_string()),
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
        let updated_vehicle = self.vehicle_repository
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use std::sync::Arc;

    struct MockVehicleRepo {
        vehicles: Vec<Vehicle>,
        has_related_data: bool,
    }

    #[async_trait::async_trait]
    impl VehicleRepository for MockVehicleRepo {
        async fn get_vehicles(
            &self,
            _query: VehicleQuery,
        ) -> Result<(Vec<Vehicle>, i64), anyhow::Error> {
            Ok((self.vehicles.clone(), self.vehicles.len() as i64))
        }

        async fn get_vehicle(&self, vehicle_id: i32) -> Result<Option<Vehicle>, anyhow::Error> {
            Ok(self
                .vehicles
                .iter()
                .find(|v| v.vehicle_id == vehicle_id)
                .cloned())
        }

        async fn get_vehicles_batch(
            &self,
            vehicle_ids: &[i32],
        ) -> Result<Vec<Vehicle>, anyhow::Error> {
            Ok(self
                .vehicles
                .iter()
                .filter(|v| vehicle_ids.contains(&v.vehicle_id) && v.status == 1)
                .cloned()
                .collect())
        }

        async fn create_vehicle(&self, vehicle: VehicleCreate) -> Result<Vehicle, anyhow::Error> {
            let now = NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
            let new_vehicle = Vehicle {
                vehicle_id: self.vehicles.len() as i32 + 1,
                vehicle_name: vehicle.vehicle_name,
                license_plate: vehicle.license_plate,
                vehicle_type: vehicle.vehicle_type,
                vehicle_color: vehicle.vehicle_color,
                vehicle_brand: vehicle.vehicle_brand,
                vehicle_model: vehicle.vehicle_model,
                engine_no: vehicle.engine_no,
                frame_no: vehicle.frame_no,
                register_date: vehicle.register_date,
                inspection_date: vehicle.inspection_date,
                insurance_date: vehicle.insurance_date,
                seating_capacity: vehicle.seating_capacity,
                load_capacity: vehicle.load_capacity,
                vehicle_length: vehicle.vehicle_length,
                vehicle_width: vehicle.vehicle_width,
                vehicle_height: vehicle.vehicle_height,
                device_id: vehicle.device_id,
                terminal_type: vehicle.terminal_type,
                communication_type: vehicle.communication_type,
                sim_card_no: vehicle.sim_card_no,
                install_date: vehicle.install_date,
                install_address: vehicle.install_address,
                install_technician: vehicle.install_technician,
                own_no: vehicle.own_no,
                own_name: vehicle.own_name,
                own_phone: vehicle.own_phone,
                own_id_card: vehicle.own_id_card,
                own_address: vehicle.own_address,
                own_email: vehicle.own_email,
                group_id: vehicle.group_id,
                operation_status: vehicle.operation_status,
                operation_route: vehicle.operation_route,
                operation_area: vehicle.operation_area,
                operation_company: vehicle.operation_company,
                driver_name: vehicle.driver_name,
                driver_phone: vehicle.driver_phone,
                driver_license_no: vehicle.driver_license_no,
                purchase_price: vehicle.purchase_price,
                annual_fee: vehicle.annual_fee,
                insurance_fee: vehicle.insurance_fee,

                remark: vehicle.remark,
                status: vehicle.status,
                create_time: now,
                update_time: None,
                create_user_id: vehicle.create_user_id,
                update_user_id: None,
            };
            Ok(new_vehicle)
        }

        async fn update_vehicle(
            &self,
            vehicle_id: i32,
            vehicle: VehicleUpdate,
        ) -> Result<Option<Vehicle>, anyhow::Error> {
            if let Some(mut existing_vehicle) = self.get_vehicle(vehicle_id).await? {
                if let Some(vehicle_name) = vehicle.vehicle_name {
                    existing_vehicle.vehicle_name = vehicle_name;
                }
                if let Some(license_plate) = vehicle.license_plate {
                    existing_vehicle.license_plate = license_plate;
                }
                if let Some(register_date) = vehicle.register_date {
                    existing_vehicle.register_date = register_date;
                }
                if let Some(inspection_date) = vehicle.inspection_date {
                    existing_vehicle.inspection_date = inspection_date;
                }
                if let Some(insurance_date) = vehicle.insurance_date {
                    existing_vehicle.insurance_date = insurance_date;
                }
                if let Some(status) = vehicle.status {
                    existing_vehicle.status = status;
                }
                // 更新时间
                let now = NaiveDateTime::parse_from_str("2026-01-13 11:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
                existing_vehicle.update_time = Some(now);

                Ok(Some(existing_vehicle))
            } else {
                Ok(None)
            }
        }

        async fn delete_vehicle(&self, vehicle_id: i32) -> Result<bool, anyhow::Error> {
            Ok(self.vehicles.iter().any(|v| v.vehicle_id == vehicle_id))
        }
        
        async fn has_related_data(&self, _vehicle_id: i32) -> Result<bool, anyhow::Error> {
            Ok(self.has_related_data)
        }
    }

    // 测试用例:获取车辆列表
    #[tokio::test]
    async fn test_get_vehicles() {
        // 准备测试数据
        let now = NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let register_date = NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let inspection_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let insurance_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let vehicles = vec![
            Vehicle {
                vehicle_id: 1,
                vehicle_name: "测试车辆1".to_string(),
                license_plate: "京A12345".to_string(),
                vehicle_type: "货车".to_string(),
                vehicle_color: "红色".to_string(),
                vehicle_brand: "福田".to_string(),
                vehicle_model: "欧曼".to_string(),
                engine_no: "1234567890".to_string(),
                frame_no: "ABC1234567890".to_string(),
                register_date,
                inspection_date,
                insurance_date,
                seating_capacity: 2,
                load_capacity: 10.0,
                vehicle_length: 6.0,
                vehicle_width: 2.5,
                vehicle_height: 3.0,
                device_id: None,
                terminal_type: None,
                communication_type: None,
                sim_card_no: None,
                install_date: None,
                install_address: None,
                install_technician: None,
                own_no: None,
                own_name: None,
                own_phone: None,
                own_id_card: None,
                own_address: None,
                own_email: None,
                group_id: 1,
                operation_status: 1,
                operation_route: None,
                operation_area: None,
                operation_company: None,
                driver_name: None,
                driver_phone: None,
                driver_license_no: None,
                purchase_price: None,
                annual_fee: None,
                insurance_fee: None,

                remark: None,
                status: 1,
                create_time: now,
                update_time: None,
                create_user_id: 1,
                update_user_id: None,
            },
            Vehicle {
                vehicle_id: 2,
                vehicle_name: "测试车辆2".to_string(),
                license_plate: "京B67890".to_string(),
                vehicle_type: "客车".to_string(),
                vehicle_color: "蓝色".to_string(),
                vehicle_brand: "宇通".to_string(),
                vehicle_model: "ZK6120".to_string(),
                engine_no: "0987654321".to_string(),
                frame_no: "XYZ0987654321".to_string(),
                register_date,
                inspection_date,
                insurance_date,
                seating_capacity: 45,
                load_capacity: 5.0,
                vehicle_length: 12.0,
                vehicle_width: 2.5,
                vehicle_height: 3.8,
                device_id: None,
                terminal_type: None,
                communication_type: None,
                sim_card_no: None,
                install_date: None,
                install_address: None,
                install_technician: None,
                own_no: None,
                own_name: None,
                own_phone: None,
                own_id_card: None,
                own_address: None,
                own_email: None,
                group_id: 2,
                operation_status: 1,
                operation_route: None,
                operation_area: None,
                operation_company: None,
                driver_name: None,
                driver_phone: None,
                driver_license_no: None,
                purchase_price: None,
                annual_fee: None,
                insurance_fee: None,

                remark: None,
                status: 1,
                create_time: now,
                update_time: None,
                create_user_id: 1,
                update_user_id: None,
            },
        ];

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleRepo {
            vehicles: vehicles.clone(),
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleUseCases::new(mock_repo);

        // 执行测试
        let query = VehicleQuery { page: None, page_size: None, vehicle_name: None, license_plate: None, vehicle_type: None, status: None };
        let result = use_cases.get_vehicles(query).await;

        // 验证结果
        assert!(result.is_ok());
        let (result_vehicles, result_total) = result.ok_or_else(|| AppError::resource_not_found("Failed to get vehicles"))?;
        assert_eq!(result_vehicles, vehicles);
        assert_eq!(result_total, 2);
    }

    // 测试用例:获取单个车辆
    #[tokio::test]
    async fn test_get_vehicle() {
        // 准备测试数据
        let now = NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let register_date = NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let inspection_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let insurance_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let vehicle = Vehicle {
            vehicle_id: 1,
            vehicle_name: "测试车辆".to_string(),
            license_plate: "京A12345".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "红色".to_string(),
            vehicle_brand: "福田".to_string(),
            vehicle_model: "欧曼".to_string(),
            engine_no: "1234567890".to_string(),
            frame_no: "ABC1234567890".to_string(),
            register_date,
            inspection_date,
            insurance_date,
            seating_capacity: 2,
            load_capacity: 10.0,
            vehicle_length: 6.0,
            vehicle_width: 2.5,
            vehicle_height: 3.0,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: 1,
            operation_status: 1,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,

            remark: None,
            status: 1,
            create_time: now,
            update_time: None,
            create_user_id: 1,
            update_user_id: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleRepo {
            vehicles: vec![vehicle.clone()],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.get_vehicle(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert_eq!(result.ok_or_else(|| AppError::resource_not_found("Vehicle not found"))?, Some(vehicle));
    }

    // 测试用例:创建车辆成功
    #[tokio::test]
    async fn test_create_vehicle_success() {
        // 准备测试数据
        let register_date = NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let inspection_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let insurance_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let vehicle_create = VehicleCreate {
            vehicle_name: "新测试车辆".to_string(),
            license_plate: "京C11111".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "红色".to_string(),
            vehicle_brand: "福田".to_string(),
            vehicle_model: "欧曼".to_string(),
            engine_no: "1234567890".to_string(),
            frame_no: "ABC1234567890".to_string(),
            register_date,
            inspection_date,
            insurance_date,
            seating_capacity: 2,
            load_capacity: 10.0,
            vehicle_length: 6.0,
            vehicle_width: 2.5,
            vehicle_height: 3.0,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: 1,
            operation_status: 1,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,

            remark: None,
            status: 1,
            create_user_id: 1,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleRepo {
            vehicles: Vec::new(),
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_vehicle(vehicle_create).await;

        // 验证结果
        assert!(result.is_ok());
        let vehicle = result.ok_or_else(|| AppError::resource_not_found("Failed to create vehicle"))?;
        assert_eq!(vehicle.vehicle_name, "新测试车辆");
        assert_eq!(vehicle.license_plate, "京C11111");
    }

    // 测试用例:创建车辆失败 - 年检日期早于注册日期
    #[tokio::test]
    async fn test_create_vehicle_invalid_inspection_date() {
        // 准备测试数据
        let register_date = NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let inspection_date = NaiveDateTime::parse_from_str("2025-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let insurance_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let vehicle_create = VehicleCreate {
            vehicle_name: "测试车辆".to_string(),
            license_plate: "京D22222".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "红色".to_string(),
            vehicle_brand: "福田".to_string(),
            vehicle_model: "欧曼".to_string(),
            engine_no: "1234567890".to_string(),
            frame_no: "ABC1234567890".to_string(),
            register_date,
            inspection_date,
            insurance_date,
            seating_capacity: 2,
            load_capacity: 10.0,
            vehicle_length: 6.0,
            vehicle_width: 2.5,
            vehicle_height: 3.0,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: 1,
            operation_status: 1,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,

            remark: None,
            status: 1,
            create_user_id: 1,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleRepo {
            vehicles: Vec::new(),
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_vehicle(vehicle_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "年检日期不能早于注册日期");
    }

    // 测试用例:创建车辆失败 - 保险日期早于注册日期
    #[tokio::test]
    async fn test_create_vehicle_invalid_insurance_date() {
        // 准备测试数据
        let register_date = NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let inspection_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let insurance_date = NaiveDateTime::parse_from_str("2025-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let vehicle_create = VehicleCreate {
            vehicle_name: "测试车辆".to_string(),
            license_plate: "京E33333".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "红色".to_string(),
            vehicle_brand: "福田".to_string(),
            vehicle_model: "欧曼".to_string(),
            engine_no: "1234567890".to_string(),
            frame_no: "ABC1234567890".to_string(),
            register_date,
            inspection_date,
            insurance_date,
            seating_capacity: 2,
            load_capacity: 10.0,
            vehicle_length: 6.0,
            vehicle_width: 2.5,
            vehicle_height: 3.0,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: 1,
            operation_status: 1,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,

            remark: None,
            status: 1,
            create_user_id: 1,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleRepo {
            vehicles: Vec::new(),
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.create_vehicle(vehicle_create).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "保险日期不能早于注册日期");
    }

    // 测试用例:更新车辆
    #[tokio::test]
    async fn test_update_vehicle() {
        // 准备测试数据
        let now = NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let register_date = NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let inspection_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let insurance_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let vehicle = Vehicle {
            vehicle_id: 1,
            vehicle_name: "测试车辆".to_string(),
            license_plate: "京A12345".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "红色".to_string(),
            vehicle_brand: "福田".to_string(),
            vehicle_model: "欧曼".to_string(),
            engine_no: "1234567890".to_string(),
            frame_no: "ABC1234567890".to_string(),
            register_date,
            inspection_date,
            insurance_date,
            seating_capacity: 2,
            load_capacity: 10.0,
            vehicle_length: 6.0,
            vehicle_width: 2.5,
            vehicle_height: 3.0,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: 1,
            operation_status: 1,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,

            remark: None,
            status: 1,
            create_time: now,
            update_time: None,
            create_user_id: 1,
            update_user_id: None,
        };

        let vehicle_update = VehicleUpdate {
            vehicle_name: Some("更新测试车辆".to_string()),
            license_plate: Some("京A67890".to_string()),
            status: Some(2),
            vehicle_type: None,
            vehicle_color: None,
            vehicle_brand: None,
            vehicle_model: None,
            engine_no: None,
            frame_no: None,
            register_date: None,
            inspection_date: None,
            insurance_date: None,
            seating_capacity: None,
            load_capacity: None,
            vehicle_length: None,
            vehicle_width: None,
            vehicle_height: None,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: None,
            operation_status: None,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,
            remark: None,
            update_user_id: None,
        };

        // 创建模拟仓库
        let mock_repo = Arc::new(MockVehicleRepo {
            vehicles: vec![vehicle.clone()],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.update_vehicle(1, vehicle_update).await;

        // 验证结果
        assert!(result.is_ok());
        let updated_vehicle = result.ok_or_else(|| AppError::resource_not_found("Failed to update vehicle"))?.ok_or_else(|| AppError::resource_not_found("Vehicle not found"))?;
        assert_eq!(updated_vehicle.vehicle_name, "更新测试车辆");
        assert_eq!(updated_vehicle.license_plate, "京A67890");
        assert_eq!(updated_vehicle.status, 2);
    }

    // 测试用例:删除车辆成功
    #[tokio::test]
    async fn test_delete_vehicle_success() {
        // 准备测试数据
        let now = NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let register_date = NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let inspection_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let insurance_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let vehicle = Vehicle {
            vehicle_id: 1,
            vehicle_name: "测试车辆".to_string(),
            license_plate: "京A12345".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "红色".to_string(),
            vehicle_brand: "福田".to_string(),
            vehicle_model: "欧曼".to_string(),
            engine_no: "1234567890".to_string(),
            frame_no: "ABC1234567890".to_string(),
            register_date,
            inspection_date,
            insurance_date,
            seating_capacity: 2,
            load_capacity: 10.0,
            vehicle_length: 6.0,
            vehicle_width: 2.5,
            vehicle_height: 3.0,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: 1,
            operation_status: 1,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,

            remark: None,
            status: 1,
            create_time: now,
            update_time: None,
            create_user_id: 1,
            update_user_id: None,
        };

        // 创建模拟仓库 (无关联数据)
        let mock_repo = Arc::new(MockVehicleRepo {
            vehicles: vec![vehicle.clone()],
            has_related_data: false,
        });

        // 创建用例实例
        let use_cases = VehicleUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.delete_vehicle(1).await;

        // 验证结果
        assert!(result.is_ok());
        assert!(result.ok_or_else(|| AppError::resource_not_found("Failed to delete vehicle"))?);
    }

    // 测试用例:删除车辆失败 - 有关联数据
    #[tokio::test]
    async fn test_delete_vehicle_with_related_data() {
        // 准备测试数据
        let now = NaiveDateTime::parse_from_str("2026-01-13 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let register_date = NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let inspection_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let insurance_date = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let vehicle = Vehicle {
            vehicle_id: 1,
            vehicle_name: "测试车辆".to_string(),
            license_plate: "京A12345".to_string(),
            vehicle_type: "货车".to_string(),
            vehicle_color: "红色".to_string(),
            vehicle_brand: "福田".to_string(),
            vehicle_model: "欧曼".to_string(),
            engine_no: "1234567890".to_string(),
            frame_no: "ABC1234567890".to_string(),
            register_date,
            inspection_date,
            insurance_date,
            seating_capacity: 2,
            load_capacity: 10.0,
            vehicle_length: 6.0,
            vehicle_width: 2.5,
            vehicle_height: 3.0,
            device_id: None,
            terminal_type: None,
            communication_type: None,
            sim_card_no: None,
            install_date: None,
            install_address: None,
            install_technician: None,
            own_no: None,
            own_name: None,
            own_phone: None,
            own_id_card: None,
            own_address: None,
            own_email: None,
            group_id: 1,
            operation_status: 1,
            operation_route: None,
            operation_area: None,
            operation_company: None,
            driver_name: None,
            driver_phone: None,
            driver_license_no: None,
            purchase_price: None,
            annual_fee: None,
            insurance_fee: None,

            remark: None,
            status: 1,
            create_time: now,
            update_time: None,
            create_user_id: 1,
            update_user_id: None,
        };

        // 创建模拟仓库 (有关联数据)
        let mock_repo = Arc::new(MockVehicleRepo {
            vehicles: vec![vehicle.clone()],
            has_related_data: true,
        });

        // 创建用例实例
        let use_cases = VehicleUseCases::new(mock_repo);

        // 执行测试
        let result = use_cases.delete_vehicle(1).await;

        // 验证结果
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "车辆有关联数据，无法删除");
    }
}
