//! / CarpTMS 核心服务集成层
// 提供与 CarpTMS 核心服务(车辆管理、用户管理、数据上报等)的集成接口
use crate::truck_scale::db::TruckScaleDb;
use crate::truck_scale::handlers::{UserHandler, VehicleHandler};
use crate::truck_scale::protocol::message_protocol::*;
use crate::truck_scale::transformers::MessageTransformer;
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;

/// 服务集成器
pub struct ServiceIntegration {
    /// 数据库连接池
    _db: TruckScaleDb, // 使用下划线前缀表示字段暂时未使用
    /// WebSocket 发送器(可选)
    websocket_sender: Option<Arc<dyn WebSocketSender + Send + Sync>>,
}

impl ServiceIntegration {
    /// 创建新的服务集成器
    pub fn new(pool: PgPool) -> Self {
        Self {
            _db: TruckScaleDb::new(pool.into()),
            websocket_sender: None,
        }
    }

    /// 设置 WebSocket 发送器
    pub fn set_websocket_sender(&mut self, sender: Arc<dyn WebSocketSender + Send + Sync>) {
        self.websocket_sender = Some(sender);
    }

    /// 处理统一消息
    pub async fn handle_message(&self, message: UnifiedMessage) -> Result<UnifiedMessage> {
        match &message.body {
            MessageBody::Login(req) => self.handle_login(req, &message.header).await,
            MessageBody::QueryVehicle(req) => self.handle_query_vehicle(req, &message.header).await,
            MessageBody::QueryUser(req) => self.handle_query_user(req, &message.header).await,
            MessageBody::QueryVehicleGroup(req) => {
                self.handle_query_vehicle_group(req, &message.header).await
            }
            MessageBody::QueryUserGroup(req) => {
                self.handle_query_user_group(req, &message.header).await
            }
            MessageBody::DataReport(req) => self.handle_data_report(req, &message.header).await,
            MessageBody::Heartbeat(req) => self.handle_heartbeat(req, &message.header).await,
            _ => Err(anyhow::anyhow!("Unsupported message type")),
        }
    }

    /// 处理登录请求
    async fn handle_login(
        &self,
        req: &LoginRequest,
        _header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        let handler = UserHandler::new(self._db.pool().clone());
        let user = handler.query_user_by_name(&req.username).await?;

        if let Some(user) = user {
            if user.status == 1 {
                return Ok(MessageTransformer::create_login_response(false, None, None)
                    .with_error(4001, "用户已被禁用".to_string(), None));
            }

            Ok(MessageTransformer::create_login_response(
                true,
                Some(user.user_id.clone()),
                Some(user.real_name.clone()),
            ))
        } else {
            Ok(
                MessageTransformer::create_login_response(false, None, None).with_error(
                    4000,
                    "用户名或密码错误".to_string(),
                    None,
                ),
            )
        }
    }

    /// 处理车辆数据查询
    async fn handle_query_vehicle(
        &self,
        req: &QueryVehicleRequest,
        header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        let handler = VehicleHandler::new(self._db.pool().clone());
        let session_id = header.session_id.clone();

        if let Some(vehicle_id) = &req.vehicle_id {
            let vehicle = handler.query_vehicle(vehicle_id).await?;
            let vehicles = vehicle.map(|v| vec![v]).unwrap_or_default();
            let vehicles_len = vehicles.len() as i32;
            Ok(MessageTransformer::create_vehicle_data_message(
                vehicles,
                Some(vehicles_len),
                None,
                None,
                session_id,
            ))
        } else {
            let page = req.page.unwrap_or(1);
            let page_size = req.page_size.unwrap_or(20);
            let vehicles = handler.query_vehicle_list(page, page_size).await?;
            let vehicles_len = vehicles.len() as i32;
            Ok(MessageTransformer::create_vehicle_data_message(
                vehicles,
                Some(vehicles_len),
                Some(page),
                Some(page_size),
                session_id,
            ))
        }
    }

    /// 处理用户数据查询
    async fn handle_query_user(
        &self,
        req: &QueryUserRequest,
        header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        let handler = UserHandler::new(self._db.pool().clone());
        let session_id = header.session_id.clone();

        if let Some(user_id) = &req.user_id {
            let user = handler.query_user(user_id).await?;
            let users = user.map(|u| vec![u]).unwrap_or_default();
            let users_len = users.len() as i32;
            Ok(MessageTransformer::create_user_data_message(
                users,
                Some(users_len),
                None,
                None,
                session_id,
            ))
        } else {
            let page = req.page.unwrap_or(1);
            let page_size = req.page_size.unwrap_or(20);
            let users = handler.query_user_list(page, page_size).await?;
            let users_len = users.len() as i32;
            Ok(MessageTransformer::create_user_data_message(
                users,
                Some(users_len),
                Some(page),
                Some(page_size),
                session_id,
            ))
        }
    }

    /// 处理车组数据查询
    async fn handle_query_vehicle_group(
        &self,
        req: &QueryVehicleGroupRequest,
        _header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        use crate::truck_scale::handlers::vehicle_group_handler::VehicleGroupHandler;
        let handler = VehicleGroupHandler::new_with_pool(self._db.pool().clone());

        if let Some(group_id) = &req.group_id {
            let group = handler.query_group(group_id).await?;
            let groups = group.map(|g| vec![g]).unwrap_or_default();
            Ok(UnifiedMessage::new(
                MessageType::VehicleGroupData,
                MessageBody::VehicleGroupData(VehicleGroupData {
                    groups: groups
                        .iter()
                        .map(MessageTransformer::vehicle_group_info_from_handler)
                        .collect(),
                }),
            ))
        } else if let Some(parent_id) = &req.parent_id {
            let groups = handler.query_child_groups(parent_id).await?;
            Ok(UnifiedMessage::new(
                MessageType::VehicleGroupData,
                MessageBody::VehicleGroupData(VehicleGroupData {
                    groups: groups
                        .iter()
                        .map(MessageTransformer::vehicle_group_info_from_handler)
                        .collect(),
                }),
            ))
        } else {
            let groups = handler.query_all_groups().await?;
            Ok(UnifiedMessage::new(
                MessageType::VehicleGroupData,
                MessageBody::VehicleGroupData(VehicleGroupData {
                    groups: groups
                        .iter()
                        .map(MessageTransformer::vehicle_group_info_from_handler)
                        .collect(),
                }),
            ))
        }
    }

    /// 处理用户组数据查询
    async fn handle_query_user_group(
        &self,
        req: &QueryUserGroupRequest,
        _header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        use crate::truck_scale::handlers::user_group_handler::UserGroupHandler;
        let handler = UserGroupHandler::new_with_pool(self._db.pool().clone());

        if let Some(group_id) = &req.group_id {
            let group = handler.query_group(group_id).await?;
            let groups = group.map(|g| vec![g]).unwrap_or_default();
            Ok(UnifiedMessage::new(
                MessageType::UserGroupData,
                MessageBody::UserGroupData(UserGroupData {
                    groups: groups
                        .iter()
                        .map(MessageTransformer::user_group_info_from_handler)
                        .collect(),
                }),
            ))
        } else {
            let groups = handler.query_all_groups().await?;
            Ok(UnifiedMessage::new(
                MessageType::UserGroupData,
                MessageBody::UserGroupData(UserGroupData {
                    groups: groups
                        .iter()
                        .map(MessageTransformer::user_group_info_from_handler)
                        .collect(),
                }),
            ))
        }
    }

    /// 处理数据上报
    async fn handle_data_report(
        &self,
        req: &DataReport,
        header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        // 根据上报类型处理数据
        match req.report_type.as_str() {
            "weighing" => self.handle_weighing_report(req, header).await,
            "monitoring" => self.handle_monitoring_report(req, header).await,
            "alarm" => self.handle_alarm_report(req, header).await,
            _ => Ok(UnifiedMessage::new(
                MessageType::DataReportResponse,
                MessageBody::DataReportResponse(DataReportResponse {
                    success: false,
                    report_id: None,
                    error_code: Some(4002),
                    error_message: Some("Unsupported report type".to_string()),
                }),
            )),
        }
    }

    /// 处理称重数据上报
    async fn handle_weighing_report(
        &self,
        req: &DataReport,
        header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        // 将称重数据存入数据库
        // 这里可以调用相应的数据库操作
        let report_id = uuid::Uuid::new_v4().to_string();

        // 如果有 WebSocket 发送器,实时推送到前端
        if let Some(ws_sender) = &self.websocket_sender {
            let notification = NotificationMessage {
                notification_type: "weighing_report".to_string(),
                title: "新的称重数据".to_string(),
                content: format!("设备 {} 上报了称重数据", req.device_id),
                data: Some(req.data.clone()),
            };

            let message = UnifiedMessage::new(
                MessageType::Notification,
                MessageBody::Notification(notification),
            )
            .with_session_id(header.session_id.clone().unwrap_or_default());

            let _ = ws_sender.send_to_all(message).await;
        }

        Ok(UnifiedMessage::new(
            MessageType::DataReportResponse,
            MessageBody::DataReportResponse(DataReportResponse {
                success: true,
                report_id: Some(report_id),
                error_code: None,
                error_message: None,
            }),
        ))
    }

    /// 处理监控数据上报
    async fn handle_monitoring_report(
        &self,
        _req: &DataReport,
        _header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        let report_id = uuid::Uuid::new_v4().to_string();

        Ok(UnifiedMessage::new(
            MessageType::DataReportResponse,
            MessageBody::DataReportResponse(DataReportResponse {
                success: true,
                report_id: Some(report_id),
                error_code: None,
                error_message: None,
            }),
        ))
    }

    /// 处理报警数据上报
    async fn handle_alarm_report(
        &self,
        req: &DataReport,
        header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        let report_id = uuid::Uuid::new_v4().to_string();

        // 如果有 WebSocket 发送器,实时推送报警通知
        if let Some(ws_sender) = &self.websocket_sender {
            let notification = NotificationMessage {
                notification_type: "alarm".to_string(),
                title: "报警通知".to_string(),
                content: format!("设备 {} 发生报警", req.device_id),
                data: Some(req.data.clone()),
            };

            let message = UnifiedMessage::new(
                MessageType::Notification,
                MessageBody::Notification(notification),
            )
            .with_session_id(header.session_id.clone().unwrap_or_default());

            let _ = ws_sender.send_to_all(message).await;
        }

        Ok(UnifiedMessage::new(
            MessageType::DataReportResponse,
            MessageBody::DataReportResponse(DataReportResponse {
                success: true,
                report_id: Some(report_id),
                error_code: None,
                error_message: None,
            }),
        ))
    }

    /// 处理心跳
    async fn handle_heartbeat(
        &self,
        _req: &Heartbeat,
        _header: &MessageHeader,
    ) -> Result<UnifiedMessage> {
        Ok(UnifiedMessage::new(
            MessageType::Heartbeat,
            MessageBody::Heartbeat(Heartbeat {
                timestamp: Utc::now(),
                status: "ok".to_string(),
            }),
        ))
    }
}

impl UnifiedMessage {
    fn with_error(
        mut self,
        error_code: i32,
        error_message: String,
        _details: Option<String>,
    ) -> Self {
        if let MessageBody::LoginResponse(ref mut resp) = self.body {
            resp.error_code = Some(error_code);
            resp.error_message = Some(error_message);
        }
        self
    }
}

/// WebSocket 发送器 trait
#[async_trait::async_trait]
pub trait WebSocketSender {
    /// 发送消息到所有连接
    async fn send_to_all(&self, message: UnifiedMessage) -> Result<()>;

    /// 发送消息到指定会话
    async fn send_to_session(&self, session_id: &str, message: UnifiedMessage) -> Result<()>;
}

/// 默认的 WebSocket 发送器实现
pub struct DefaultWebSocketSender;

#[async_trait::async_trait]
impl WebSocketSender for DefaultWebSocketSender {
    async fn send_to_all(&self, _message: UnifiedMessage) -> Result<()> {
        // 默认实现:不做任何操作
        Ok(())
    }

    async fn send_to_session(&self, _session_id: &str, _message: UnifiedMessage) -> Result<()> {
        // 默认实现:不做任何操作
        Ok(())
    }
}
