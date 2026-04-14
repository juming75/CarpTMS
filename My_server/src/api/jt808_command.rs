//! / JT808 指令下发 API
// 提供REST API接口用于向JT808设备下发指令

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use log::{debug, error, info};
use actix::Addr;

use crate::protocols::jt808::command::{Jt808Command, SendCommand, Jt808CommandQueue};
use crate::protocols::jt808::models::Jt808CommandId;

/// JT808 指令请求体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jt808CommandRequest {
    /// 指令类型
    pub command_type: String,
    /// 指令参数
    pub params: serde_json::Value,
    /// 超时时间(秒)
    #[serde(default = "default_timeout")]
    pub timeout: u32,
    /// 重试次数
    #[serde(default = "default_retry")]
    pub retry: u32,
}

fn default_timeout() -> u32 { 30 }
fn default_retry() -> u32 { 3 }

/// 下发 JT808 指令
#[utoipa::path(
    post,
    path = "/api/v1/devices/{device_id}/jt808/command",
    tag = "JT808",
    request_body = Jt808CommandRequest,
    responses(
        (status = 200, description = "指令下发成功", body = CommandResponse),
        (status = 400, description = "参数错误"),
        (status = 404, description = "设备不存在"),
        (status = 500, description = "服务器错误")
    ),
    params(
        ("device_id" = String, Path, description = "设备ID")
    )
)]
pub async fn send_jt808_command(
    device_id: web::Path<String>,
    command: web::Json<Jt808CommandRequest>,
    command_queue: web::Data<Addr<Jt808CommandQueue>>,
) -> impl Responder {
    let device_id = device_id.into_inner();
    let command_req = command.into_inner();

    info!(
        "Sending JT808 command to device {}: type={}, params={:?}",
        device_id, command_req.command_type, command_req.params
    );

    // 根据指令类型构建 JT808 指令
    let jt808_command = match build_jt808_command(&command_req) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to build JT808 command: {}", e);
            return HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Invalid command: {}", e),
                "code": "INVALID_COMMAND"
            }));
        }
    };

    // 发送指令到队列
    match command_queue
        .send(SendCommand {
            device_id: device_id.clone(),
            command: jt808_command,
        })
        .await
    {
        Ok(Ok(())) => {
            info!("JT808 command sent successfully to device {}", device_id);
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Command sent successfully",
                "device_id": device_id,
                "command_type": command_req.command_type
            }))
        }
        Ok(Err(e)) => {
            error!("Failed to send JT808 command: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Command send failed: {:?}", e),
                "code": "SEND_FAILED"
            }))
        }
        Err(e) => {
            error!("Command queue error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Command queue error: {}", e),
                "code": "QUEUE_ERROR"
            }))
        }
    }
}

/// 查询设备在线状态
#[utoipa::path(
    get,
    path = "/api/v1/devices/{device_id}/jt808/status",
    tag = "JT808",
    responses(
        (status = 200, description = "设备状态", body = DeviceStatusResponse),
        (status = 404, description = "设备不存在"),
        (status = 500, description = "服务器错误")
    ),
    params(
        ("device_id" = String, Path, description = "设备ID")
    )
)]
pub async fn get_device_status(
    device_id: web::Path<String>,
    session_manager: web::Data<Addr<crate::protocols::jt808::session::Jt808SessionManager>>,
) -> impl Responder {
    let device_id = device_id.into_inner();

    // 查询会话状态
    let status = match session_manager
        .send(crate::protocols::jt808::session::QuerySession {
            device_id: device_id.clone(),
        })
        .await
    {
        Ok(Ok(Some(session))) => {
            serde_json::json!({
                "device_id": device_id,
                "online": true,
                "auth_status": session.auth_status,
                "last_activity": session.last_activity,
                "heartbeat_time": session.heartbeat_time,
                "flow_no": session.flow_no,
            })
        }
        Ok(Ok(None)) => {
            serde_json::json!({
                "device_id": device_id,
                "online": false,
                "auth_status": "unknown",
                "message": "Device not found"
            })
        }
        Ok(Err(e)) => {
            error!("Failed to query device status: {:?}", e);
            serde_json::json!({
                "device_id": device_id,
                "online": false,
                "error": format!("Query failed: {:?}", e)
            })
        }
        Err(e) => {
            error!("Session manager error: {}", e);
            serde_json::json!({
                "device_id": device_id,
                "online": false,
                "error": format!("Session manager error: {}", e)
            })
        }
    };

    HttpResponse::Ok().json(status)
}

/// 构建 JT808 指令
fn build_jt808_command(req: &Jt808CommandRequest) -> Result<Jt808Command, String> {
    let command_id = match req.command_type.as_str() {
        "terminal_terminal_query" => Jt808CommandId::TerminalTerminalQuery,
        "terminal_terminal_control" => Jt808CommandId::TerminalTerminalControl,
        "position_query" => Jt808CommandId::PositionQuery,
        "temporary_position_control" => Jt808CommandId::TemporaryPositionControl,
        "telephone_callback" => Jt808CommandId::TelephoneCallback,
        "text_message" => Jt808CommandId::TextMessage,
        "event_setting" => Jt808CommandId::EventSetting,
        "question_upload" => Jt808CommandId::QuestionUpload,
        "information_menu" => Jt808CommandId::InformationMenu,
        "vehicle_control" => Jt808CommandId::VehicleControl,
        "properties_setting" => Jt808CommandId::PropertiesSetting,
        "query_specific_properties" => Jt808CommandId::QuerySpecificProperties,
        "device_upgrade" => Jt808CommandId::DeviceUpgrade,
        "location_data_upload" => Jt808CommandId::LocationDataUpload,
        "data_transparently" => Jt808CommandId::DataTransParently,
        _ => return Err(format!("Unknown command type: {}", req.command_type)),
    };

    Ok(Jt808Command {
        command_id,
        params: req.params.clone(),
        timeout: req.timeout as i32,
        retry_count: req.retry as i32,
        created_at: chrono::Utc::now(),
    })
}

/// 响应类型
#[derive(Debug, Serialize, Deserialize)]
struct CommandResponse {
    pub success: bool,
    pub message: String,
    pub device_id: String,
    pub command_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceStatusResponse {
    pub device_id: String,
    pub online: bool,
    pub auth_status: String,
    pub last_activity: Option<String>,
    pub heartbeat_time: Option<String>,
}

/// 配置 JT808 指令 API 路由
pub fn configure_jt808_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(send_jt808_command)
        .service(get_device_status);
}






