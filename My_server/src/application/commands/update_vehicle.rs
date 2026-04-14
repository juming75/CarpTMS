//! 更新车辆命令

use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::{Command, CommandHandler, CommandResponse};
use crate::domain::entities::vehicle::VehicleUpdate;
use crate::domain::repositories::{SqlxVehicleRepository, VehicleRepository};
use crate::errors::{AppError, AppResult};

/// 更新车辆命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVehicleCommand {
    /// 车辆ID
    pub vehicle_id: i32,
    /// 车辆名称
    pub vehicle_name: Option<String>,
    /// 车牌号
    pub license_plate: Option<String>,
    /// 车辆类型
    pub vehicle_type: Option<String>,
    /// 车辆颜色
    pub vehicle_color: Option<String>,
    /// 车辆品牌
    pub vehicle_brand: Option<String>,
    /// 车辆型号
    pub vehicle_model: Option<String>,
    /// 发动机号
    pub engine_no: Option<String>,
    /// 车架号
    pub frame_no: Option<String>,
    /// 注册日期
    pub register_date: Option<NaiveDateTime>,
    /// 年检日期
    pub inspection_date: Option<NaiveDateTime>,
    /// 保险日期
    pub insurance_date: Option<NaiveDateTime>,
    /// 座位数
    pub seating_capacity: Option<i32>,
    /// 载重
    pub load_capacity: Option<f64>,
    /// 车辆长度
    pub vehicle_length: Option<f64>,
    /// 车辆宽度
    pub vehicle_width: Option<f64>,
    /// 车辆高度
    pub vehicle_height: Option<f64>,
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
    pub group_id: Option<i32>,
    /// 运营状态
    pub operation_status: Option<i16>,
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
    pub status: Option<i16>,
    /// 更新用户ID
    pub update_user_id: Option<i32>,
}

impl Command for UpdateVehicleCommand {
    fn command_type() -> &'static str {
        "update_vehicle"
    }
}

impl UpdateVehicleCommand {
    /// 转换为领域实体
    pub fn to_vehicle_update(&self) -> VehicleUpdate {
        VehicleUpdate {
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
            update_user_id: self.update_user_id,
        }
    }
}

/// 更新车辆命令处理器
pub struct UpdateVehicleCommandHandler {
    pool: sqlx::PgPool,
    repository: SqlxVehicleRepository,
}

impl UpdateVehicleCommandHandler {
    /// 创建处理器实例
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            repository: SqlxVehicleRepository::new(),
        }
    }
}

#[async_trait]
impl CommandHandler<UpdateVehicleCommand> for UpdateVehicleCommandHandler {
    async fn handle(&self, command: UpdateVehicleCommand) -> AppResult<CommandResponse> {
        // 验证命令
        if command.vehicle_id <= 0 {
            return Err(AppError::validation_error("车辆ID无效", None));
        }

        // 验证日期逻辑
        if let (Some(register_date), Some(inspection_date)) =
            (command.register_date, command.inspection_date)
        {
            if inspection_date < register_date {
                return Err(AppError::validation_error("年检日期不能早于注册日期", None));
            }
        }

        if let (Some(register_date), Some(insurance_date)) =
            (command.register_date, command.insurance_date)
        {
            if insurance_date < register_date {
                return Err(AppError::validation_error("保险日期不能早于注册日期", None));
            }
        }

        // 转换为领域实体
        let vehicle_update = command.to_vehicle_update();

        // 调用仓储更新车辆
        let result = self
            .repository
            .update(&self.pool, command.vehicle_id, vehicle_update)
            .await?;

        match result {
            Some(vehicle) => Ok(CommandResponse::success_with_message(
                vehicle.vehicle_id,
                "车辆更新成功",
            )),
            None => Err(AppError::not_found_error("车辆不存在".to_string())),
        }
    }
}
