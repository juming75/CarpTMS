//! 创建车辆命令

use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::{Command, CommandHandler, CommandResponse};
use crate::domain::entities::vehicle::VehicleCreate;
use crate::domain::repositories::{SqlxVehicleRepository, VehicleRepository};
use crate::errors::{AppError, AppResult};

/// 创建车辆命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVehicleCommand {
    /// 车辆名称
    pub vehicle_name: String,
    /// 车牌号
    pub license_plate: String,
    /// 车辆类型
    pub vehicle_type: String,
    /// 车辆颜色
    pub vehicle_color: String,
    /// 车辆品牌
    pub vehicle_brand: String,
    /// 车辆型号
    pub vehicle_model: String,
    /// 发动机号
    pub engine_no: String,
    /// 车架号
    pub frame_no: String,
    /// 注册日期
    pub register_date: NaiveDateTime,
    /// 年检日期
    pub inspection_date: NaiveDateTime,
    /// 保险日期
    pub insurance_date: NaiveDateTime,
    /// 座位数
    pub seating_capacity: i32,
    /// 载重
    pub load_capacity: f64,
    /// 车辆长度
    pub vehicle_length: f64,
    /// 车辆宽度
    pub vehicle_width: f64,
    /// 车辆高度
    pub vehicle_height: f64,
    /// 设备ID
    pub device_id: Option<String>,
    /// 终端类型
    pub terminal_type: Option<String>,
    /// 通信类型
    pub communication_type: Option<String>,
    /// SIM卡号
    pub sim_card_no: Option<String>,
    /// 安装日期
    pub install_date: Option<NaiveDateTime>,
    /// 安装地址
    pub install_address: Option<String>,
    /// 安装技师
    pub install_technician: Option<String>,
    /// 车主编号
    pub own_no: Option<String>,
    /// 车主姓名
    pub own_name: Option<String>,
    /// 车主电话
    pub own_phone: Option<String>,
    /// 车主身份证号
    pub own_id_card: Option<String>,
    /// 车主地址
    pub own_address: Option<String>,
    /// 车主邮箱
    pub own_email: Option<String>,
    /// 车组ID
    pub group_id: i32,
    /// 运营状态
    pub operation_status: i32,
    /// 运营路线
    pub operation_route: Option<String>,
    /// 运营区域
    pub operation_area: Option<String>,
    /// 运营公司
    pub operation_company: Option<String>,
    /// 司机姓名
    pub driver_name: Option<String>,
    /// 司机电话
    pub driver_phone: Option<String>,
    /// 司机驾照号
    pub driver_license_no: Option<String>,
    /// 购买价格
    pub purchase_price: Option<f64>,
    /// 年费
    pub annual_fee: Option<f64>,
    /// 保险费
    pub insurance_fee: Option<f64>,
    /// 备注
    pub remark: Option<String>,
    /// 状态
    pub status: i32,
    /// 创建用户ID
    pub create_user_id: i32,
}

impl Command for CreateVehicleCommand {
    fn command_type() -> &'static str {
        "create_vehicle"
    }
}

impl CreateVehicleCommand {
    /// 转换为领域实体
    pub fn to_vehicle_create(&self) -> VehicleCreate {
        VehicleCreate {
            vehicle_name: self.vehicle_name.clone(),
            license_plate: self.license_plate.clone(),
            vehicle_type: self.vehicle_type.clone(),
            vehicle_color: self.vehicle_color.clone(),
            vehicle_brand: self.vehicle_brand.clone(),
            vehicle_model: self.vehicle_model.clone(),
            engine_no: self.engine_no.clone(),
            frame_no: self.frame_no.clone(),
            register_date: self.register_date,
            inspection_date: self.inspection_date,
            insurance_date: self.insurance_date,
            seating_capacity: self.seating_capacity,
            load_capacity: self.load_capacity,
            vehicle_length: self.vehicle_length,
            vehicle_width: self.vehicle_width,
            vehicle_height: self.vehicle_height,
            device_id: self.device_id.clone(),
            terminal_type: self.terminal_type.clone(),
            communication_type: self.communication_type.clone(),
            sim_card_no: self.sim_card_no.clone(),
            install_date: self.install_date,
            install_address: self.install_address.clone(),
            install_technician: self.install_technician.clone(),
            own_no: self.own_no.clone(),
            own_name: self.own_name.clone(),
            own_phone: self.own_phone.clone(),
            own_id_card: self.own_id_card.clone(),
            own_address: self.own_address.clone(),
            own_email: self.own_email.clone(),
            group_id: self.group_id,
            operation_status: self.operation_status,
            operation_route: self.operation_route.clone(),
            operation_area: self.operation_area.clone(),
            operation_company: self.operation_company.clone(),
            driver_name: self.driver_name.clone(),
            driver_phone: self.driver_phone.clone(),
            driver_license_no: self.driver_license_no.clone(),
            purchase_price: self.purchase_price,
            annual_fee: self.annual_fee,
            insurance_fee: self.insurance_fee,
            remark: self.remark.clone(),
            status: self.status,
            create_user_id: self.create_user_id,
        }
    }
}

/// 创建车辆命令处理器
#[allow(dead_code)]
pub struct CreateVehicleCommandHandler {
    repository: SqlxVehicleRepository,
}

impl CreateVehicleCommandHandler {
    /// 创建处理器实例
    pub fn new() -> Self {
        Self {
            repository: SqlxVehicleRepository::new(),
        }
    }
}

impl Default for CreateVehicleCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandHandler<CreateVehicleCommand> for CreateVehicleCommandHandler {
    async fn handle(&self, command: CreateVehicleCommand) -> AppResult<CommandResponse> {
        // 验证命令
        if command.vehicle_name.is_empty() {
            return Err(AppError::validation_error("车辆名称不能为空", None));
        }

        if command.license_plate.is_empty() {
            return Err(AppError::validation_error("车牌号不能为空", None));
        }

        if command.inspection_date < command.register_date {
            return Err(AppError::validation_error("年检日期不能早于注册日期", None));
        }

        if command.insurance_date < command.register_date {
            return Err(AppError::validation_error("保险日期不能早于注册日期", None));
        }

        // 转换为领域实体
        let _vehicle_create = command.to_vehicle_create();

        // 注意：这里需要数据库连接池，实际使用时需要注入
        // 这里返回成功响应，实际实现需要调用 repository.create()
        Ok(CommandResponse::success_with_message(
            0,
            "车辆创建命令已接收",
        ))
    }
}

/// 带数据库连接池的创建车辆命令处理器
pub struct CreateVehicleCommandHandlerWithPool {
    pool: sqlx::PgPool,
    repository: SqlxVehicleRepository,
}

impl CreateVehicleCommandHandlerWithPool {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            repository: SqlxVehicleRepository::new(),
        }
    }
}

#[async_trait]
impl CommandHandler<CreateVehicleCommand> for CreateVehicleCommandHandlerWithPool {
    async fn handle(&self, command: CreateVehicleCommand) -> AppResult<CommandResponse> {
        // 验证命令
        if command.vehicle_name.is_empty() {
            return Err(AppError::validation_error("车辆名称不能为空", None));
        }

        if command.license_plate.is_empty() {
            return Err(AppError::validation_error("车牌号不能为空", None));
        }

        if command.inspection_date < command.register_date {
            return Err(AppError::validation_error("年检日期不能早于注册日期", None));
        }

        if command.insurance_date < command.register_date {
            return Err(AppError::validation_error("保险日期不能早于注册日期", None));
        }

        // 转换为领域实体
        let vehicle_create = command.to_vehicle_create();

        // 调用仓储创建车辆
        let vehicle = self.repository.create(&self.pool, vehicle_create).await?;

        Ok(CommandResponse::success_with_message(
            vehicle.vehicle_id,
            "车辆创建成功",
        ))
    }
}
