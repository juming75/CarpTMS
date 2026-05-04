use actix_web::{web, HttpResponse};
use chrono::{TimeZone, Utc};
use log::{info, warn};
use std::sync::Arc;
use validator::Validate;

use crate::application::services::user_service::{UserService, UserServiceImpl};
use crate::domain::entities::user::{
    UserCreate as DomainUserCreate, UserQuery as DomainUserQuery, UserUpdate as DomainUserUpdate,
};
use crate::errors::{
    created_response_with_message, success_response_with_message, AppError, AppResult,
};
use crate::schemas::{PagedResponse, UserCreate, UserQuery, UserResponse, UserUpdate};

// 获取用户列表(支持分页和筛选)
#[utoipa::path(
    path = "/api/users",
    get,
    responses(
        (status = 200, description = "Users fetched successfully", body = ApiResponse<PagedResponse<UserResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_users(
    user_service: web::Data<Arc<UserServiceImpl>>,
    query: web::Query<UserQuery>,
) -> AppResult<HttpResponse> {
    let page = query.0.page.unwrap_or(1);
    let page_size = query.0.page_size.unwrap_or(20);

    // 转换为领域查询对象
    let domain_query = DomainUserQuery {
        page: Some(page),
        page_size: Some(page_size),
        user_name: query.0.username.clone(),
        full_name: None,     // 暂时不支持按真实姓名筛选
        status: None,        // 暂时不支持按状态筛选
        user_group_id: None, // 暂时不支持按用户组筛选
    };

    // 调用服务获取用户列表
    let (users, total) = user_service.get_users(domain_query).await?;

    // 转换为响应格式
    let user_responses: Vec<UserResponse> = users
        .into_iter()
        .map(|user| {
            UserResponse {
                id: user.user_id,
                username: user.user_name,
                full_name: user.full_name,
                phone_number: user.phone_number,
                email: user.email,
                user_group_id: user.user_group_id,
                user_group_name: None,   // 暂时不获取用户组名称
                department_id: None,     // 数据库中没有department_id字段
                department_name: None,   // 数据库中没有department_id字段,所以也没有department_name
                organization_id: None,   // 暂时不获取组织ID
                organization_name: None, // 暂时不获取组织名称
                status: user.status,     // 转换为i16
                last_login_time: user.last_login_time.map(|t| Utc.from_utc_datetime(&t)),
                create_time: Utc.from_utc_datetime(&user.create_time),
                update_time: user.update_time.map(|t| Utc.from_utc_datetime(&t)),
            }
        })
        .collect();

    // 计算总页数
    let pages = if total % page_size as i64 == 0 {
        total / page_size as i64
    } else {
        total / page_size as i64 + 1
    };

    // 构建分页响应
    let paged_response = PagedResponse {
        list: user_responses,
        total,
        page,
        page_size,
        pages: pages as i32,
    };

    Ok(success_response_with_message(
        "Users fetched successfully",
        Some(paged_response),
    ))
}

// 创建用户
#[utoipa::path(
    path = "/api/users",
    post,
    request_body = UserCreate,
    responses(
        (status = 201, description = "User created successfully", body = ApiResponse<UserResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_user(
    user_service: web::Data<Arc<UserServiceImpl>>,
    user: web::Json<UserCreate>,
) -> AppResult<HttpResponse> {
    // 验证请求数据
    let user_data = &user.into_inner();
    if let Err(errors) = user_data.validate() {
        let validation_errors: Vec<String> = errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let msg = errors[0]
                    .message
                    .as_ref()
                    .map(|s| s.as_ref())
                    .unwrap_or("Unknown validation error");
                format!("{}: {}", field, msg)
            })
            .collect();

        warn!(
            "Validation failed for user creation: {:?}",
            validation_errors
        );
        return Err(AppError::validation_error(
            &validation_errors.join(", "),
            None,
        ));
    }

    info!("Creating user: {}", user_data.username);

    // 转换为领域创建对象
    let domain_user_create = DomainUserCreate {
        user_name: user_data.username.clone(),
        password: user_data.password.clone(),
        full_name: user_data.full_name.clone(),
        phone_number: user_data.phone_number.clone(),
        email: user_data.email.clone(),
        user_group_id: user_data.user_group_id,
        status: 1, // 默认状态为1（启用）
    };

    // 调用服务创建用户
    let created_user = user_service.create_user(domain_user_create).await?;

    // 转换为响应格式
    let response = UserResponse {
        id: created_user.user_id,
        username: created_user.user_name,
        full_name: created_user.full_name,
        phone_number: created_user.phone_number,
        email: created_user.email,
        user_group_id: created_user.user_group_id,
        user_group_name: None,              // 新创建的用户没有用户组名称
        department_id: None,                // 数据库中没有department_id字段
        department_name: None, // 数据库中没有department_id字段,所以也没有department_name
        organization_id: None, // 暂时不获取组织ID
        organization_name: None, // 新创建的用户没有组织名称
        status: created_user.status as i16, // 转换为i16
        last_login_time: created_user
            .last_login_time
            .map(|t| Utc.from_utc_datetime(&t)),
        create_time: Utc.from_utc_datetime(&created_user.create_time),
        update_time: created_user.update_time.map(|t| Utc.from_utc_datetime(&t)),
    };

    info!("User created successfully: {}", user_data.username);
    Ok(created_response_with_message(
        "User created successfully",
        Some(response),
    ))
}

// 获取用户详情
#[utoipa::path(
    path = "/api/users/{user_id}",
    get,
    responses(
        (status = 200, description = "User fetched successfully", body = ApiResponse<UserResponse>),
        (status = 404, description = "User not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_user(
    user_service: web::Data<Arc<UserServiceImpl>>,
    user_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let user_id = user_id.into_inner();

    // 调用服务获取用户
    let user = user_service.get_user(user_id).await?;

    match user {
        Some(user) => {
            let response = UserResponse {
                id: user.user_id,
                username: user.user_name,
                full_name: user.full_name,
                phone_number: user.phone_number,
                email: user.email,
                user_group_id: user.user_group_id,
                user_group_name: None,   // 暂时不获取用户组名称
                department_id: None,     // 数据库中没有department_id字段
                department_name: None,   // 数据库中没有department_id字段,所以也没有department_name
                organization_id: None,   // 暂时不获取组织ID
                organization_name: None, // 暂时不获取组织名称
                status: user.status,     // 转换为i16
                last_login_time: user.last_login_time.map(|t| Utc.from_utc_datetime(&t)),
                create_time: Utc.from_utc_datetime(&user.create_time),
                update_time: user.update_time.map(|t| Utc.from_utc_datetime(&t)),
            };

            Ok(success_response_with_message(
                "User fetched successfully",
                Some(response),
            ))
        }
        None => {
            warn!("User not found: {}", user_id);
            Err(AppError::not_found_error("User not found".to_string()))
        }
    }
}

// 更新用户
#[utoipa::path(
    path = "/api/users/{user_id}",
    put,
    request_body = UserUpdate,
    responses(
        (status = 200, description = "User updated successfully", body = ApiResponse<UserResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "User not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_user(
    user_service: web::Data<Arc<UserServiceImpl>>,
    user_id: web::Path<i32>,
    user: web::Json<UserUpdate>,
) -> AppResult<HttpResponse> {
    let user_id = user_id.into_inner();

    // 转换为领域更新对象
    let domain_user_update = DomainUserUpdate {
        password: user
            .password
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
        full_name: user.full_name.clone(),
        phone_number: user.phone_number.clone(),
        email: user.email.clone(),
        user_group_id: user.user_group_id,
        status: user.status,
        last_login_time: None,
    };

    // 调用服务更新用户
    let updated_user = user_service
        .update_user(user_id, domain_user_update)
        .await?;

    match updated_user {
        Some(user) => {
            // 转换为响应格式
            let response = UserResponse {
                id: user.user_id,
                username: user.user_name,
                full_name: user.full_name,
                phone_number: user.phone_number,
                email: user.email,
                user_group_id: user.user_group_id,
                user_group_name: None,   // 暂时不获取用户组名称
                department_id: None,     // 数据库中没有department_id字段
                department_name: None,   // 数据库中没有department_id字段,所以也没有department_name
                organization_id: None,   // 暂时不获取组织ID
                organization_name: None, // 暂时不获取组织名称
                status: user.status,     // 转换为i16
                last_login_time: user.last_login_time.map(|t| Utc.from_utc_datetime(&t)),
                create_time: Utc.from_utc_datetime(&user.create_time),
                update_time: user.update_time.map(|t| Utc.from_utc_datetime(&t)),
            };

            info!("User updated successfully: {}", response.username);
            Ok(success_response_with_message(
                "User updated successfully",
                Some(response),
            ))
        }
        None => {
            warn!("User not found for update: {}", user_id);
            Err(AppError::not_found_error("User not found".to_string()))
        }
    }
}

// 删除用户
#[utoipa::path(
    path = "/api/users/{user_id}",
    delete,
    responses(
        (status = 200, description = "User deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "User not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_user(
    user_service: web::Data<Arc<UserServiceImpl>>,
    user_id: web::Path<i32>,
) -> AppResult<HttpResponse> {
    let user_id = user_id.into_inner();

    // 调用服务删除用户
    let deleted = user_service.delete_user(user_id).await?;

    if deleted {
        info!("User deleted successfully: {}", user_id);
        Ok(success_response_with_message(
            "User deleted successfully",
            (),
        ))
    } else {
        warn!("User not found for deletion: {}", user_id);
        Err(AppError::not_found_error("User not found".to_string()))
    }
}

// 分配用户权限(示例方法,实际实现需要根据权限系统调整)
#[utoipa::path(
    path = "/api/users/{user_id}/permissions",
    post,
    responses(
        (status = 200, description = "Permissions assigned successfully", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn assign_user_permissions(
    _user_service: web::Data<Arc<UserServiceImpl>>,
    _user_id: web::Path<i32>,
    _permissions: web::Json<Vec<(String, String)>>,
) -> AppResult<HttpResponse> {
    // 这里实现用户权限分配逻辑
    // 注意:实际实现需要根据权限系统的设计来调整
    Ok(success_response_with_message(
        "Permissions assigned successfully",
        (),
    ))
}

// 配置用户路由
pub fn configure_users_routes(cfg: &mut web::ServiceConfig) {
    use crate::application::services::user_service::UserServiceImpl;

    cfg.route(
        "/users",
        web::get().to::<_, (web::Data<Arc<UserServiceImpl>>, web::Query<UserQuery>)>(get_users),
    )
    .route(
        "/users",
        web::post().to::<_, (web::Data<Arc<UserServiceImpl>>, web::Json<UserCreate>)>(create_user),
    )
    .route(
        "/users/{user_id}",
        web::get().to::<_, (web::Data<Arc<UserServiceImpl>>, web::Path<i32>)>(get_user),
    )
    .route(
        "/users/{user_id}",
        web::put().to::<_, (
            web::Data<Arc<UserServiceImpl>>,
            web::Path<i32>,
            web::Json<UserUpdate>,
        )>(update_user),
    )
    .route(
        "/users/{user_id}",
        web::delete().to::<_, (web::Data<Arc<UserServiceImpl>>, web::Path<i32>)>(delete_user),
    )
    .route(
        "/users/{user_id}/permissions",
        web::post().to::<_, (
            web::Data<Arc<UserServiceImpl>>,
            web::Path<i32>,
            web::Json<Vec<(String, String)>>,
        )>(assign_user_permissions),
    );
}
