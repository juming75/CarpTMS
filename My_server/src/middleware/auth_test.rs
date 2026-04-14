//! / 认证中间件测试

use actix_web::{http::header::AUTHORIZATION, test, App, HttpRequest, HttpResponse, Responder};
use actix_web::middleware::Logger;
use actix_web::web::get;

use crate::middleware::auth::AuthMiddleware;

// 测试路由处理函数
async fn test_route(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Authenticated")
}

// 测试用例:认证中间件 - 无效token
#[actix_web::test]
async fn test_auth_middleware_invalid_token() {
    // 创建测试应用
    let app = test::init_service(
        App::new()
            .wrap(Logger::default())
            .wrap(AuthMiddleware)
            .route("/test", get().to(test_route))
    ).await;
    
    // 测试请求:无效token
    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header((AUTHORIZATION, "Bearer invalid_token"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 验证结果
    assert!(resp.status().is_client_error());
}

// 测试用例:认证中间件 - 无token
#[actix_web::test]
async fn test_auth_middleware_no_token() {
    // 创建测试应用
    let app = test::init_service(
        App::new()
            .wrap(Logger::default())
            .wrap(AuthMiddleware)
            .route("/test", get().to(test_route))
    ).await;
    
    // 测试请求:无token
    let req = test::TestRequest::get()
        .uri("/test")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // 验证结果
    assert!(resp.status().is_client_error());
}

// 测试用例:角色转换函数
#[test]
fn test_role_from_str() {
    use crate::middleware::auth::role_from_str;
    
    // 测试有效角色
    assert_eq!(role_from_str("admin"), "admin");
    assert_eq!(role_from_str("user"), "user");
    assert_eq!(role_from_str("driver"), "driver");
    
    // 测试无效角色(应该返回默认角色)
    assert_eq!(role_from_str("invalid_role"), "user");
}

// 测试用例:权限检查函数
#[test]
fn test_has_permission() {
    use crate::middleware::auth::has_permission;
    
    // 测试管理员权限
    assert!(has_permission("admin", vec!["admin", "user"].as_slice()));
    assert!(has_permission("admin", vec!["admin"].as_slice()));
    assert!(has_permission("admin", vec!["user"].as_slice()));
    
    // 测试普通用户权限
    assert!(has_permission("user", vec!["user"].as_slice()));
    assert!(!has_permission("user", vec!["admin"].as_slice()));
    
    // 测试司机权限
    assert!(has_permission("driver", vec!["driver"].as_slice()));
    assert!(!has_permission("driver", vec!["admin"].as_slice()));
    assert!(!has_permission("driver", vec!["user"].as_slice()));
}






