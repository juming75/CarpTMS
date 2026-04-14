use actix_web::{web, HttpResponse, Responder};
use chrono::{TimeZone, Utc};
use log::{error, info, warn};
use std::sync::Arc;

use crate::application::services::finance_service::FinanceService;
use crate::domain::entities::finance::{FinanceCostQuery, FinanceInvoiceQuery, FinanceStatisticsQuery};
use crate::schemas::{
    ApiResponse, FinanceCostCreate, FinanceCostResponse, FinanceCostUpdate, FinanceInvoiceCreate,
    FinanceInvoiceResponse, FinanceInvoiceUpdate, FinanceStatisticsResponse, PagedResponse,
};



// 获取费用列表(支持分页和筛选)
#[utoipa::path(
    path = "/api/finance/costs",
    get,
    responses(
        (status = 200, description = "Finance costs fetched successfully", body = ApiResponse<PagedResponse<FinanceCostResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_finance_costs(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    query: web::Query<FinanceCostQuery>,
) -> impl Responder {
    let query = query.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    match finance_service.get_finance_costs(query).await {
        Ok((costs, total)) => {
            // 转换为响应格式
            let cost_responses: Vec<FinanceCostResponse> = costs
                .into_iter()
                .map(|cost| FinanceCostResponse {
                    cost_id: cost.cost_id,
                    cost_type: cost.cost_type,
                    amount: cost.amount,
                    description: cost.description,
                    cost_date: cost.cost_date,
                    create_time: Utc.from_utc_datetime(&cost.create_time),
                    update_time: cost.update_time.map(|t| Utc.from_utc_datetime(&t)),
                })
                .collect::<Vec<FinanceCostResponse>>();

            // 计算总页数
            let pages = if total % page_size as i64 == 0 {
                total / page_size as i64
            } else {
                total / page_size as i64 + 1
            };

            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Finance costs fetched successfully".to_string(),
                data: PagedResponse {
                    list: cost_responses,
                    total,
                    page,
                    page_size,
                    pages: pages as i32,
                },
            })
        }
        Err(e) => {
            error!("Failed to fetch finance costs: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to fetch finance costs".to_string(),
                data: (),
            })
        }
    }
}

// 创建费用
#[utoipa::path(
    path = "/api/finance/costs",
    post,
    request_body = FinanceCostCreate,
    responses(
        (status = 201, description = "Finance cost created successfully", body = ApiResponse<FinanceCostResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_finance_cost(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    cost: web::Json<FinanceCostCreate>,
) -> impl Responder {
    let cost_data = cost.into_inner();
    let description = cost_data.description.clone();
    info!("Creating finance cost: {}", description);

    match finance_service.create_finance_cost(crate::domain::entities::finance::FinanceCostCreate {
        cost_type: cost_data.cost_type,
        amount: cost_data.amount,
        description: cost_data.description,
        cost_date: cost_data.cost_date,
    }).await {
        Ok(cost) => {
            // 转换为响应格式
            let response = FinanceCostResponse {
                cost_id: cost.cost_id,
                cost_type: cost.cost_type,
                amount: cost.amount,
                description: cost.description,
                cost_date: cost.cost_date,
                create_time: Utc.from_utc_datetime(&cost.create_time),
                update_time: cost.update_time.map(|t| Utc.from_utc_datetime(&t)),
            };

            info!("Finance cost created successfully: {}", response.description);
            HttpResponse::Created().json(ApiResponse {
                code: 201,
                message: "Finance cost created successfully".to_string(),
                data: response,
            })
        }
        Err(e) => {
            error!("Failed to create finance cost: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to create finance cost".to_string(),
                data: (),
            })
        }
    }
}

// 获取费用详情
#[utoipa::path(
    path = "/api/finance/costs/{cost_id}",
    get,
    responses(
        (status = 200, description = "Finance cost fetched successfully", body = ApiResponse<FinanceCostResponse>),
        (status = 404, description = "Finance cost not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_finance_cost(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    cost_id: web::Path<i32>,
) -> impl Responder {
    let cost_id = cost_id.into_inner();

    match finance_service.get_finance_cost(cost_id).await {
        Ok(Some(cost)) => {
            let response = FinanceCostResponse {
                cost_id: cost.cost_id,
                cost_type: cost.cost_type,
                amount: cost.amount,
                description: cost.description,
                cost_date: cost.cost_date,
                create_time: Utc.from_utc_datetime(&cost.create_time),
                update_time: cost.update_time.map(|t| Utc.from_utc_datetime(&t)),
            };

            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Finance cost fetched successfully".to_string(),
                data: response,
            })
        }
        Ok(None) => {
            warn!("Finance cost not found: {}", cost_id);
            HttpResponse::NotFound().json(ApiResponse {
                code: 404,
                message: "Finance cost not found".to_string(),
                data: (),
            })
        }
        Err(e) => {
            error!("Failed to fetch finance cost: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to fetch finance cost".to_string(),
                data: (),
            })
        }
    }
}

// 更新费用
#[utoipa::path(
    path = "/api/finance/costs/{cost_id}",
    put,
    request_body = FinanceCostUpdate,
    responses(
        (status = 200, description = "Finance cost updated successfully", body = ApiResponse<FinanceCostResponse>),
        (status = 404, description = "Finance cost not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_finance_cost(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    cost_id: web::Path<i32>,
    cost: web::Json<FinanceCostUpdate>,
) -> impl Responder {
    let cost_id = cost_id.into_inner();
    let cost_data = cost.into_inner();

    match finance_service.update_finance_cost(cost_id, crate::domain::entities::finance::FinanceCostUpdate {
        cost_type: cost_data.cost_type,
        amount: cost_data.amount,
        description: cost_data.description,
        cost_date: cost_data.cost_date,
    }).await {
        Ok(Some(cost)) => {
            // 转换为响应格式
            let response = FinanceCostResponse {
                cost_id: cost.cost_id,
                cost_type: cost.cost_type,
                amount: cost.amount,
                description: cost.description,
                cost_date: cost.cost_date,
                create_time: Utc.from_utc_datetime(&cost.create_time),
                update_time: cost.update_time.map(|t| Utc.from_utc_datetime(&t)),
            };

            info!(
                "Finance cost updated successfully: {}",
                response.description
            );
            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Finance cost updated successfully".to_string(),
                data: response,
            })
        }
        Ok(None) => {
            warn!("Finance cost not found for update: {}", cost_id);
            HttpResponse::NotFound().json(ApiResponse {
                code: 404,
                message: "Finance cost not found".to_string(),
                data: (),
            })
        }
        Err(e) => {
            error!("Failed to update finance cost: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to update finance cost".to_string(),
                data: (),
            })
        }
    }
}

// 删除费用
#[utoipa::path(
    path = "/api/finance/costs/{cost_id}",
    delete,
    responses(
        (status = 200, description = "Finance cost deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Finance cost not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_finance_cost(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    cost_id: web::Path<i32>,
) -> impl Responder {
    let cost_id = cost_id.into_inner();

    match finance_service.delete_finance_cost(cost_id).await {
        Ok(true) => {
            info!("Finance cost deleted successfully: {}", cost_id);
            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Finance cost deleted successfully".to_string(),
                data: (),
            })
        }
        Ok(false) => {
            warn!("Finance cost not found for deletion: {}", cost_id);
            HttpResponse::NotFound().json(ApiResponse {
                code: 404,
                message: "Finance cost not found".to_string(),
                data: (),
            })
        }
        Err(e) => {
            error!("Failed to delete finance cost: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to delete finance cost".to_string(),
                data: (),
            })
        }
    }
}

// 获取发票列表(支持分页和筛选)
#[utoipa::path(
    path = "/api/finance/invoices",
    get,
    responses(
        (status = 200, description = "Finance invoices fetched successfully", body = ApiResponse<PagedResponse<FinanceInvoiceResponse>>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_finance_invoices(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    query: web::Query<FinanceInvoiceQuery>,
) -> impl Responder {
    let query = query.into_inner();
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    match finance_service.get_finance_invoices(query).await {
        Ok((invoices, total)) => {
            // 转换为响应格式
            let invoice_responses: Vec<FinanceInvoiceResponse> = invoices
                .into_iter()
                .map(|invoice| FinanceInvoiceResponse {
                    invoice_id: invoice.invoice_id,
                    invoice_number: invoice.invoice_number,
                    amount: invoice.amount,
                    invoice_date: invoice.invoice_date,
                    description: invoice.description,
                    create_time: Utc.from_utc_datetime(&invoice.create_time),
                    update_time: invoice.update_time.map(|t| Utc.from_utc_datetime(&t)),
                })
                .collect::<Vec<FinanceInvoiceResponse>>();

            // 计算总页数
            let pages = if total % page_size as i64 == 0 {
                total / page_size as i64
            } else {
                total / page_size as i64 + 1
            };

            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Finance invoices fetched successfully".to_string(),
                data: PagedResponse {
                    list: invoice_responses,
                    total,
                    page,
                    page_size,
                    pages: pages as i32,
                },
            })
        }
        Err(e) => {
            error!("Failed to fetch finance invoices: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to fetch finance invoices".to_string(),
                data: (),
            })
        }
    }
}

// 创建发票
#[utoipa::path(
    path = "/api/finance/invoices",
    post,
    request_body = FinanceInvoiceCreate,
    responses(
        (status = 201, description = "Finance invoice created successfully", body = ApiResponse<FinanceInvoiceResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn create_finance_invoice(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    invoice: web::Json<FinanceInvoiceCreate>,
) -> impl Responder {
    let invoice_data = invoice.into_inner();
    let invoice_number = invoice_data.invoice_number.clone();
    info!("Creating finance invoice: {}", invoice_number);

    match finance_service.create_finance_invoice(crate::domain::entities::finance::FinanceInvoiceCreate {
        invoice_number: invoice_data.invoice_number,
        amount: invoice_data.amount,
        invoice_date: invoice_data.invoice_date,
        description: invoice_data.description,
    }).await {
        Ok(invoice) => {
            // 转换为响应格式
            let response = FinanceInvoiceResponse {
                invoice_id: invoice.invoice_id,
                invoice_number: invoice.invoice_number,
                amount: invoice.amount,
                invoice_date: invoice.invoice_date,
                description: invoice.description,
                create_time: Utc.from_utc_datetime(&invoice.create_time),
                update_time: invoice.update_time.map(|t| Utc.from_utc_datetime(&t)),
            };

            info!(
                "Finance invoice created successfully: {}",
                response.invoice_number
            );
            HttpResponse::Created().json(ApiResponse {
                code: 201,
                message: "Finance invoice created successfully".to_string(),
                data: response,
            })
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Invoice number already exists") {
                return HttpResponse::BadRequest().json(ApiResponse {
                    code: 400,
                    message: "Invoice number already exists".to_string(),
                    data: (),
                });
            }
            error!("Failed to create finance invoice: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to create finance invoice".to_string(),
                data: (),
            })
        }
    }
}

// 获取发票详情
#[utoipa::path(
    path = "/api/finance/invoices/{invoice_id}",
    get,
    responses(
        (status = 200, description = "Finance invoice fetched successfully", body = ApiResponse<FinanceInvoiceResponse>),
        (status = 404, description = "Finance invoice not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_finance_invoice(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    invoice_id: web::Path<i32>,
) -> impl Responder {
    let invoice_id = invoice_id.into_inner();

    match finance_service.get_finance_invoice(invoice_id).await {
        Ok(Some(invoice)) => {
            let response = FinanceInvoiceResponse {
                invoice_id: invoice.invoice_id,
                invoice_number: invoice.invoice_number,
                amount: invoice.amount,
                invoice_date: invoice.invoice_date,
                description: invoice.description,
                create_time: Utc.from_utc_datetime(&invoice.create_time),
                update_time: invoice.update_time.map(|t| Utc.from_utc_datetime(&t)),
            };

            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Finance invoice fetched successfully".to_string(),
                data: response,
            })
        }
        Ok(None) => {
            warn!("Finance invoice not found: {}", invoice_id);
            HttpResponse::NotFound().json(ApiResponse {
                code: 404,
                message: "Finance invoice not found".to_string(),
                data: (),
            })
        }
        Err(e) => {
            error!("Failed to fetch finance invoice: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to fetch finance invoice".to_string(),
                data: (),
            })
        }
    }
}

// 更新发票
#[utoipa::path(
    path = "/api/finance/invoices/{invoice_id}",
    put,
    request_body = FinanceInvoiceUpdate,
    responses(
        (status = 200, description = "Finance invoice updated successfully", body = ApiResponse<FinanceInvoiceResponse>),
        (status = 400, description = "Invalid request parameters", body = ApiResponse<()>),
        (status = 404, description = "Finance invoice not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn update_finance_invoice(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    invoice_id: web::Path<i32>,
    invoice: web::Json<FinanceInvoiceUpdate>,
) -> impl Responder {
    let invoice_id = invoice_id.into_inner();
    let invoice_data = invoice.into_inner();

    match finance_service.update_finance_invoice(invoice_id, crate::domain::entities::finance::FinanceInvoiceUpdate {
        invoice_number: invoice_data.invoice_number,
        amount: invoice_data.amount,
        invoice_date: invoice_data.invoice_date,
        description: invoice_data.description,
    }).await {
        Ok(Some(invoice)) => {
            // 转换为响应格式
            let response = FinanceInvoiceResponse {
                invoice_id: invoice.invoice_id,
                invoice_number: invoice.invoice_number,
                amount: invoice.amount,
                invoice_date: invoice.invoice_date,
                description: invoice.description,
                create_time: Utc.from_utc_datetime(&invoice.create_time),
                update_time: invoice.update_time.map(|t| Utc.from_utc_datetime(&t)),
            };

            info!(
                "Finance invoice updated successfully: {}",
                response.invoice_number
            );
            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Finance invoice updated successfully".to_string(),
                data: response,
            })
        }
        Ok(None) => {
            warn!("Finance invoice not found for update: {}", invoice_id);
            HttpResponse::NotFound().json(ApiResponse {
                code: 404,
                message: "Finance invoice not found".to_string(),
                data: (),
            })
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Invoice number already exists") {
                return HttpResponse::BadRequest().json(ApiResponse {
                    code: 400,
                    message: "Invoice number already exists".to_string(),
                    data: (),
                });
            }
            error!("Failed to update finance invoice: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to update finance invoice".to_string(),
                data: (),
            })
        }
    }
}

// 删除发票
#[utoipa::path(
    path = "/api/finance/invoices/{invoice_id}",
    delete,
    responses(
        (status = 200, description = "Finance invoice deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Finance invoice not found", body = ApiResponse<()>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn delete_finance_invoice(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    invoice_id: web::Path<i32>,
) -> impl Responder {
    let invoice_id = invoice_id.into_inner();

    match finance_service.delete_finance_invoice(invoice_id).await {
        Ok(true) => {
            info!("Finance invoice deleted successfully: {}", invoice_id);
            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Finance invoice deleted successfully".to_string(),
                data: (),
            })
        }
        Ok(false) => {
            warn!("Finance invoice not found for deletion: {}", invoice_id);
            HttpResponse::NotFound().json(ApiResponse {
                code: 404,
                message: "Finance invoice not found".to_string(),
                data: (),
            })
        }
        Err(e) => {
            error!("Failed to delete finance invoice: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to delete finance invoice".to_string(),
                data: (),
            })
        }
    }
}

// 获取财务统计数据
#[utoipa::path(
    path = "/api/finance/statistics",
    get,
    responses(
        (status = 200, description = "Finance statistics fetched successfully", body = ApiResponse<FinanceStatisticsResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn get_finance_statistics(
    finance_service: web::Data<Arc<dyn FinanceService>>,
    query: web::Query<FinanceStatisticsQuery>,
) -> impl Responder {
    let query = query.into_inner();

    match finance_service.get_finance_statistics(query).await {
        Ok(statistics) => {
            // 构建响应
            let response = FinanceStatisticsResponse {
                total_cost: statistics.total_cost,
                total_invoice: statistics.total_invoice,
                balance: statistics.balance,
            };

            HttpResponse::Ok().json(ApiResponse {
                code: 200,
                message: "Finance statistics fetched successfully".to_string(),
                data: response,
            })
        }
        Err(e) => {
            error!("Failed to fetch finance statistics: {:?}", e);
            HttpResponse::InternalServerError().json(ApiResponse {
                code: 500,
                message: "Failed to fetch finance statistics".to_string(),
                data: (),
            })
        }
    }
}

// 导出财务数据
#[utoipa::path(
    path = "/api/finance/export",
    get,
    responses(
        (status = 200, description = "Finance data exported successfully", body = String),
        (status = 500, description = "Internal server error", body = ApiResponse<()>)
    )
)]
pub async fn export_finance_data(_finance_service: web::Data<dyn FinanceService>) -> impl Responder {
    // 这里实现导出逻辑,暂时返回一个示例响应
    HttpResponse::Ok().body("Finance data exported successfully")
}

// 配置财务路由
pub fn configure_finance_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // 费用管理路由
        .route("/finance/costs", web::get().to(get_finance_costs))
        .route("/finance/costs", web::post().to(create_finance_cost))
        .route("/finance/costs/{cost_id}", web::get().to(get_finance_cost))
        .route(
            "/finance/costs/{cost_id}",
            web::put().to(update_finance_cost),
        )
        .route(
            "/finance/costs/{cost_id}",
            web::delete().to(delete_finance_cost),
        )
        // 发票管理路由
        .route("/finance/invoices", web::get().to(get_finance_invoices))
        .route("/finance/invoices", web::post().to(create_finance_invoice))
        .route(
            "/finance/invoices/{invoice_id}",
            web::get().to(get_finance_invoice),
        )
        .route(
            "/finance/invoices/{invoice_id}",
            web::put().to(update_finance_invoice),
        )
        .route(
            "/finance/invoices/{invoice_id}",
            web::delete().to(delete_finance_invoice),
        )
        // 统计路由
        .route("/finance/statistics", web::get().to(get_finance_statistics))
        // 导出路由
        .route("/finance/export", web::get().to(export_finance_data));
}
