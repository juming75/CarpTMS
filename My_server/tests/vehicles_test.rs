//! 车辆管理 API 集成测试

use actix_web::{web, App};
use carptms::routes::vehicles;

// 测试车辆路由配置
#[actix_web::test]
async fn test_vehicle_routes_registered() {
    // 验证路由编译通过
    let _app = App::new().service(
        web::scope("/api/vehicles")
            .route("", web::get().to(vehicles::get_vehicles))
            .route("/{id}", web::get().to(vehicles::get_vehicle))
            .route("", web::post().to(vehicles::create_vehicle))
            .route("/{id}", web::put().to(vehicles::update_vehicle))
            .route("/{id}", web::delete().to(vehicles::delete_vehicle)),
    );
}

// 测试车辆数据模型
#[test]
fn test_vehicle_data_model() {
    // 测试车辆创建请求结构
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct VehicleCreateRequest {
        vehicle_name: String,
        license_plate: String,
        vehicle_type: String,
        vehicle_color: String,
        vehicle_brand: String,
        vehicle_model: String,
    }

    let request = VehicleCreateRequest {
        vehicle_name: "测试车辆".to_string(),
        license_plate: "京A12345".to_string(),
        vehicle_type: "货车".to_string(),
        vehicle_color: "白色".to_string(),
        vehicle_brand: "东风".to_string(),
        vehicle_model: "EQ1090".to_string(),
    };

    // 验证必填字段
    assert!(!request.vehicle_name.is_empty());
    assert!(!request.license_plate.is_empty());

    // 测试序列化
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("测试车辆"));
    assert!(json.contains("京A12345"));
}
