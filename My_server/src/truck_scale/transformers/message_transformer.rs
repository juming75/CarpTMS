//! / 消息转换器
// 将 Truck Scale 协议消息转换为统一消息格式
use crate::truck_scale::handlers::user_group_handler::UserGroupInfo as HandlerUserGroupInfo;
use crate::truck_scale::handlers::user_handler::UserInfo as HandlerUserInfo;
use crate::truck_scale::handlers::vehicle_group_handler::VehicleGroupInfo as HandlerVehicleGroupInfo;
use crate::truck_scale::handlers::vehicle_handler::VehicleInfo as HandlerVehicleInfo;
use crate::truck_scale::protocol::message_protocol::*;

/// 消息转换器
pub struct MessageTransformer;

impl MessageTransformer {
    /// 创建新的消息转换器
    pub fn new() -> Self {
        Self
    }

    /// 将处理器车辆信息转换为协议车辆信息
    pub fn vehicle_info_from_handler(handler_info: &HandlerVehicleInfo) -> VehicleInfo {
        VehicleInfo {
            vehicle_id: handler_info.vehicle_id.clone(),
            plate_no: handler_info.plate_no.clone(),
            terminal_no: if handler_info.terminal_no.is_empty() {
                None
            } else {
                Some(handler_info.terminal_no.clone())
            },
            sim_no: if handler_info.sim_no.is_empty() {
                None
            } else {
                Some(handler_info.sim_no.clone())
            },
            group_id: if handler_info.group_id.is_empty() {
                None
            } else {
                Some(handler_info.group_id.clone())
            },
            driver_name: if handler_info.driver_name.is_empty() {
                None
            } else {
                Some(handler_info.driver_name.clone())
            },
            driver_tel: if handler_info.driver_tel.is_empty() {
                None
            } else {
                Some(handler_info.driver_tel.clone())
            },
            vehicle_type: if handler_info.vehicle_type.is_empty() {
                None
            } else {
                Some(handler_info.vehicle_type.clone())
            },
            vehicle_brand: if handler_info.vehicle_brand.is_empty() {
                None
            } else {
                Some(handler_info.vehicle_brand.clone())
            },
            vehicle_model: if handler_info.vehicle_model.is_empty() {
                None
            } else {
                Some(handler_info.vehicle_model.clone())
            },
            max_weight: if handler_info.max_weight == 0.0 {
                None
            } else {
                Some(handler_info.max_weight)
            },
            tare_weight: if handler_info.tare_weight == 0.0 {
                None
            } else {
                Some(handler_info.tare_weight)
            },
            status: Some(handler_info.status),
        }
    }

    /// 将协议车辆信息转换为处理器车辆信息
    pub fn vehicle_info_to_handler(protocol_info: &VehicleInfo) -> HandlerVehicleInfo {
        HandlerVehicleInfo {
            vehicle_id: protocol_info.vehicle_id.clone(),
            plate_no: protocol_info.plate_no.clone(),
            terminal_no: protocol_info.terminal_no.clone().unwrap_or_default(),
            sim_no: protocol_info.sim_no.clone().unwrap_or_default(),
            engine_no: String::new(),
            frame_no: String::new(),
            owner_name: String::new(),
            owner_tel: String::new(),
            owner_address: String::new(),
            vehicle_type: protocol_info.vehicle_type.clone().unwrap_or_default(),
            vehicle_color: String::new(),
            vehicle_brand: protocol_info.vehicle_brand.clone().unwrap_or_default(),
            vehicle_model: protocol_info.vehicle_model.clone().unwrap_or_default(),
            group_id: protocol_info.group_id.clone().unwrap_or_default(),
            driver_name: protocol_info.driver_name.clone().unwrap_or_default(),
            driver_tel: protocol_info.driver_tel.clone().unwrap_or_default(),
            driver_license: String::new(),
            max_weight: protocol_info.max_weight.unwrap_or(0.0),
            tare_weight: protocol_info.tare_weight.unwrap_or(0.0),
            rated_weight: 0.0,
            length: 0.0,
            width: 0.0,
            height: 0.0,
            fuel_type: String::new(),
            manufacturer: String::new(),
            manufacture_date: String::new(),
            registration_date: String::new(),
            insurance_expire_date: String::new(),
            annual_inspection_date: String::new(),
            remark: String::new(),
            status: protocol_info.status.unwrap_or(0),
            create_time: String::new(),
            update_time: String::new(),
            create_by: String::new(),
            update_by: String::new(),
        }
    }

    /// 将处理器用户信息转换为协议用户信息
    pub fn user_info_from_handler(handler_info: &HandlerUserInfo) -> UserInfo {
        UserInfo {
            user_id: handler_info.user_id.clone(),
            user_name: handler_info.user_name.clone(),
            real_name: if handler_info.real_name.is_empty() {
                None
            } else {
                Some(handler_info.real_name.clone())
            },
            user_type: handler_info.user_type,
            group_id: if handler_info.group_id.is_empty() {
                None
            } else {
                Some(handler_info.group_id.clone())
            },
            company: if handler_info.company.is_empty() {
                None
            } else {
                Some(handler_info.company.clone())
            },
            mobile: if handler_info.mobile.is_empty() {
                None
            } else {
                Some(handler_info.mobile.clone())
            },
            email: if handler_info.email.is_empty() {
                None
            } else {
                Some(handler_info.email.clone())
            },
            status: handler_info.status,
            permissions: if handler_info.permission.is_empty() {
                None
            } else {
                Some(
                    handler_info
                        .permission
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                )
            },
        }
    }

    /// 将协议用户信息转换为处理器用户信息
    pub fn user_info_to_handler(protocol_info: &UserInfo) -> HandlerUserInfo {
        HandlerUserInfo {
            user_id: protocol_info.user_id.clone(),
            user_name: protocol_info.user_name.clone(),
            password: String::new(),
            real_name: protocol_info.real_name.clone().unwrap_or_default(),
            user_type: protocol_info.user_type,
            group_id: protocol_info.group_id.clone().unwrap_or_default(),
            company: protocol_info.company.clone().unwrap_or_default(),
            department: String::new(),
            tel: String::new(),
            mobile: protocol_info.mobile.clone().unwrap_or_default(),
            email: protocol_info.email.clone().unwrap_or_default(),
            address: String::new(),
            permission: protocol_info
                .permissions
                .as_ref()
                .map(|p| p.join(","))
                .unwrap_or_default(),
            veh_group_list: String::new(),
            status: protocol_info.status,
            expiration_time: String::new(),
            title: String::new(),
            id_card: String::new(),
            id_card_expire_date: String::new(),
            education: String::new(),
            birth_date: String::new(),
            gender: 0,
            avatar: String::new(),
            signature: String::new(),
            last_login_time: String::new(),
            last_login_ip: String::new(),
            login_count: 0,
            remark: String::new(),
            create_time: String::new(),
            update_time: String::new(),
            create_by: String::new(),
            update_by: String::new(),
        }
    }

    /// 将处理器车组信息转换为协议车组信息
    pub fn vehicle_group_info_from_handler(
        handler_info: &HandlerVehicleGroupInfo,
    ) -> VehicleGroupInfo {
        VehicleGroupInfo {
            group_id: handler_info.group_id.clone(),
            parent_id: if handler_info.parent_id.is_empty() {
                None
            } else {
                Some(handler_info.parent_id.clone())
            },
            group_name: handler_info.group_name.clone(),
            contact_people: if handler_info.contact_people.is_empty() {
                None
            } else {
                Some(handler_info.contact_people.clone())
            },
            contact_tel: if handler_info.contact_tel.is_empty() {
                None
            } else {
                Some(handler_info.contact_tel.clone())
            },
            children: None,
        }
    }

    /// 将协议车组信息转换为处理器车组信息
    pub fn vehicle_group_info_to_handler(
        protocol_info: &VehicleGroupInfo,
    ) -> HandlerVehicleGroupInfo {
        HandlerVehicleGroupInfo {
            group_id: protocol_info.group_id.clone(),
            parent_id: protocol_info.parent_id.clone().unwrap_or_default(),
            group_name: protocol_info.group_name.clone(),
            contact_people: protocol_info.contact_people.clone().unwrap_or_default(),
            contact_tel: protocol_info.contact_tel.clone().unwrap_or_default(),
        }
    }

    /// 将处理器用户组信息转换为协议用户组信息
    pub fn user_group_info_from_handler(handler_info: &HandlerUserGroupInfo) -> UserGroupInfo {
        UserGroupInfo {
            group_id: handler_info.group_id.clone(),
            group_name: handler_info.group_name.clone(),
            user_type: handler_info.user_type,
            permission: if handler_info.permission.is_empty() {
                None
            } else {
                Some(handler_info.permission.clone())
            },
            user_count: None,
        }
    }

    /// 将协议用户组信息转换为处理器用户组信息
    pub fn user_group_info_to_handler(protocol_info: &UserGroupInfo) -> HandlerUserGroupInfo {
        HandlerUserGroupInfo {
            group_id: protocol_info.group_id.clone(),
            group_name: protocol_info.group_name.clone(),
            user_type: protocol_info.user_type,
            permission: protocol_info.permission.clone().unwrap_or_default(),
        }
    }

    /// 创建车辆数据查询消息
    pub fn create_query_vehicle_message(
        vehicle_id: Option<String>,
        plate_no: Option<String>,
        group_id: Option<String>,
        page: Option<i32>,
        page_size: Option<i32>,
        session_id: Option<String>,
    ) -> UnifiedMessage {
        let mut message = UnifiedMessage::new(
            MessageType::QueryVehicle,
            MessageBody::QueryVehicle(QueryVehicleRequest {
                vehicle_id,
                plate_no,
                group_id,
                page,
                page_size,
            }),
        );

        if let Some(sid) = session_id {
            message = message.with_session_id(sid);
        }

        message
    }

    /// 创建车辆数据响应消息
    pub fn create_vehicle_data_message(
        vehicles: Vec<HandlerVehicleInfo>,
        total: Option<i32>,
        page: Option<i32>,
        page_size: Option<i32>,
        session_id: Option<String>,
    ) -> UnifiedMessage {
        let protocol_vehicles: Vec<VehicleInfo> = vehicles
            .iter()
            .map(Self::vehicle_info_from_handler)
            .collect();

        let mut message = UnifiedMessage::new(
            MessageType::VehicleData,
            MessageBody::VehicleData(VehicleData {
                vehicles: protocol_vehicles,
                total,
                page,
                page_size,
            }),
        );

        if let Some(sid) = session_id {
            message = message.with_session_id(sid);
        }

        message
    }

    /// 创建用户数据查询消息
    pub fn create_query_user_message(
        user_id: Option<String>,
        user_name: Option<String>,
        group_id: Option<String>,
        user_type: Option<i32>,
        page: Option<i32>,
        page_size: Option<i32>,
        session_id: Option<String>,
    ) -> UnifiedMessage {
        let mut message = UnifiedMessage::new(
            MessageType::QueryUser,
            MessageBody::QueryUser(QueryUserRequest {
                user_id,
                user_name,
                group_id,
                user_type,
                page,
                page_size,
            }),
        );

        if let Some(sid) = session_id {
            message = message.with_session_id(sid);
        }

        message
    }

    /// 创建用户数据响应消息
    pub fn create_user_data_message(
        users: Vec<HandlerUserInfo>,
        total: Option<i32>,
        page: Option<i32>,
        page_size: Option<i32>,
        session_id: Option<String>,
    ) -> UnifiedMessage {
        let protocol_users: Vec<UserInfo> =
            users.iter().map(Self::user_info_from_handler).collect();

        let mut message = UnifiedMessage::new(
            MessageType::UserData,
            MessageBody::UserData(UserData {
                users: protocol_users,
                total,
                page,
                page_size,
            }),
        );

        if let Some(sid) = session_id {
            message = message.with_session_id(sid);
        }

        message
    }

    /// 创建数据上报消息
    pub fn create_data_report_message(
        report_type: String,
        device_id: String,
        data: serde_json::Value,
        session_id: Option<String>,
        protocol_type: Option<String>,
    ) -> UnifiedMessage {
        let mut message = UnifiedMessage::new(
            MessageType::DataReport,
            MessageBody::DataReport(DataReport {
                report_type,
                device_id: device_id.clone(),
                data,
                timestamp: chrono::Utc::now(),
            }),
        )
        .with_device_id(device_id);

        if let Some(sid) = session_id {
            message = message.with_session_id(sid);
        }

        if let Some(pt) = protocol_type {
            message = message.with_protocol_type(pt);
        }

        message
    }

    /// 创建错误消息
    pub fn create_error_message(
        error_code: i32,
        error_message: String,
        details: Option<String>,
        session_id: Option<String>,
    ) -> UnifiedMessage {
        let mut message = UnifiedMessage::new(
            MessageType::Error,
            MessageBody::Error(ErrorMessage {
                error_code,
                error_message,
                details,
            }),
        );

        if let Some(sid) = session_id {
            message = message.with_session_id(sid);
        }

        message
    }

    /// 创建登录响应消息
    pub fn create_login_response(
        success: bool,
        user_id: Option<String>,
        real_name: Option<String>,
    ) -> UnifiedMessage {
        UnifiedMessage::new(
            MessageType::LoginResponse,
            MessageBody::LoginResponse(LoginResponse {
                success,
                user_id,
                user_name: real_name,
                user_type: None,
                session_id: None,
                permissions: None,
                vehicle_groups: None,
                error_code: None,
                error_message: None,
            }),
        )
    }
}

impl Default for MessageTransformer {
    fn default() -> Self {
        Self::new()
    }
}
