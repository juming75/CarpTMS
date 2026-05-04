//! 区块链路由
//! 提供区块链服务的API接口

use actix_web::{web, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use crate::blockchain::{BlockchainService, TransactionType, BlockchainError};

/// 提交交易请求
#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitTransactionRequest {
    pub transaction_type: String,
    pub data: serde_json::Value,
    pub submitter: String,
}

/// 验证交易请求
#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyTransactionRequest {
    pub transaction_id: String,
}

/// 验证数据请求
#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyDataRequest {
    pub data: serde_json::Value,
    pub expected_hash: String,
}

/// 配置区块链路由
pub fn configure_blockchain_routes() -> Scope {
    web::scope("/api/blockchain")
        .route("/transactions", web::get().to(get_all_transactions))
        .route("/transactions", web::post().to(submit_transaction))
        .route("/transactions/{transaction_id}", web::get().to(get_transaction))
        .route("/verify/transaction", web::post().to(verify_transaction))
        .route("/verify/data", web::post().to(verify_data))
        .route("/connect", web::post().to(connect))
        .route("/disconnect", web::post().to(disconnect))
}

/// 获取所有交易
async fn get_all_transactions(
    blockchain_service: web::Data<BlockchainService>,
) -> HttpResponse {
    let transactions = blockchain_service.get_all_transactions().await;
    HttpResponse::Ok().json(transactions)
}

/// 提交交易
async fn submit_transaction(
    blockchain_service: web::Data<BlockchainService>,
    req: web::Json<SubmitTransactionRequest>,
) -> HttpResponse {
    let transaction_type = match req.transaction_type.as_str() {
        "VehicleData" => TransactionType::VehicleData,
        "OrderData" => TransactionType::OrderData,
        "AlarmData" => TransactionType::AlarmData,
        "UserOperation" => TransactionType::UserOperation,
        "SystemConfig" => TransactionType::SystemConfig,
        _ => return HttpResponse::BadRequest().json({"error": "Invalid transaction type"}),
    };
    
    match blockchain_service.submit_transaction(transaction_type, req.data.clone(), &req.submitter).await {
        Ok(transaction_id) => HttpResponse::Ok().json({"transaction_id": transaction_id, "message": "Transaction submitted successfully"}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 获取交易
async fn get_transaction(
    blockchain_service: web::Data<BlockchainService>,
    transaction_id: web::Path<String>,
) -> HttpResponse {
    match blockchain_service.get_transaction(&transaction_id).await {
        Ok(transaction) => HttpResponse::Ok().json(transaction),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 验证交易
async fn verify_transaction(
    blockchain_service: web::Data<BlockchainService>,
    req: web::Json<VerifyTransactionRequest>,
) -> HttpResponse {
    match blockchain_service.verify_transaction(&req.transaction_id).await {
        Ok(valid) => HttpResponse::Ok().json({"valid": valid, "message": "Transaction verified"}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 验证数据
async fn verify_data(
    blockchain_service: web::Data<BlockchainService>,
    req: web::Json<VerifyDataRequest>,
) -> HttpResponse {
    match blockchain_service.verify_data(&req.data, &req.expected_hash).await {
        Ok(valid) => HttpResponse::Ok().json({"valid": valid, "message": "Data verified"}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 连接到区块链网络
async fn connect(
    blockchain_service: web::Data<BlockchainService>,
) -> HttpResponse {
    match blockchain_service.connect().await {
        Ok(_) => HttpResponse::Ok().json({"message": "Connected to blockchain network"}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}

/// 断开与区块链网络的连接
async fn disconnect(
    blockchain_service: web::Data<BlockchainService>,
) -> HttpResponse {
    match blockchain_service.disconnect().await {
        Ok(_) => HttpResponse::Ok().json({"message": "Disconnected from blockchain network"}),
        Err(error) => HttpResponse::InternalServerError().json({"error": error.to_string()}),
    }
}
