//! 车辆 DTO

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::entities::vehicle::Vehicle;

/// 车辆 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleDto {
    /// 车辆ID
    pub vehicle_id: i32,
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
    /// 覇架号
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
    /// 覇辆宽度
    pub vehicle_width: f64,
    /// 覇辆高度
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
    /// 创建时间
    pub create_time: NaiveDateTime,
    /// 更新时间
    pub update_time: Option<NaiveDateTime>,
    /// 创建用户ID
    pub create_user_id: i32,
    /// 更新用户ID
    pub update_user_id: Option<i32>,
}

/// 从领域实体转换为 DTO
impl From<Vehicle> for VehicleDto {
    fn from(vehicle: Vehicle) -> Self {
        Self {
            vehicle_id: vehicle.vehicle_id,
            vehicle_name: vehicle.vehicle_name,
            license_plate: vehicle.license_plate,
            vehicle_type: vehicle.vehicle_type,
            vehicle_color: vehicle.vehicle_color,
            vehicle_brand: vehicle.vehicle_brand,
            vehicle_model: vehicle.vehicle_model,
            engine_no: vehicle.engine_no,
            frame_no: vehicle.frame_no,
            register_date: vehicle.register_date,
            inspection_date: vehicle.inspection_date,
            insurance_date: vehicle.insurance_date,
            seating_capacity: vehicle.seating_capacity,
            load_capacity: vehicle.load_capacity,
            vehicle_length: vehicle.vehicle_length,
            vehicle_width: vehicle.vehicle_width,
            vehicle_height: vehicle.vehicle_height,
            device_id: vehicle.device_id,
            terminal_type: vehicle.terminal_type,
            communication_type: vehicle.communication_type,
            sim_card_no: vehicle.sim_card_no,
            install_date: vehicle.install_date,
            install_address: vehicle.install_address,
            install_technician: vehicle.install_technician,
            own_no: vehicle.own_no,
            own_name: vehicle.own_name,
            own_phone: vehicle.own_phone,
            own_id_card: vehicle.own_id_card,
            own_address: vehicle.own_address,
            own_email: vehicle.own_email,
            group_id: vehicle.group_id,
            operation_status: vehicle.operation_status,
            operation_route: vehicle.operation_route,
            operation_area: vehicle.operation_area,
            operation_company: vehicle.operation_company,
            driver_name: vehicle.driver_name,
            driver_phone: vehicle.driver_phone,
            driver_license_no: vehicle.driver_license_no,
            purchase_price: vehicle.purchase_price,
            annual_fee: vehicle.annual_fee,
            insurance_fee: vehicle.insurance_fee,
            remark: vehicle.remark,
            status: vehicle.status,
            create_time: vehicle.create_time,
            update_time: vehicle.update_time,
            create_user_id: vehicle.create_user_id,
            update_user_id: vehicle.update_user_id,
        }
    }
}

/// 车辆简要 DTO（用于列表显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleSummaryDto {
    /// 车辆ID
    pub vehicle_id: i32,
    /// 车辆名称
    pub vehicle_name: String,
    /// 车牌号
    pub license_plate: String,
    /// 车辆类型
    pub vehicle_type: String,
    /// 车辆品牌
    pub vehicle_brand: String,
    /// 状态
    pub status: i32,
    /// 运营状态
    pub operation_status: i32,
}

impl From<Vehicle> for VehicleSummaryDto {
    fn from(vehicle: Vehicle) -> Self {
        Self {
            vehicle_id: vehicle.vehicle_id,
            vehicle_name: vehicle.vehicle_name,
            license_plate: vehicle.license_plate,
            vehicle_type: vehicle.vehicle_type,
            vehicle_brand: vehicle.vehicle_brand,
            status: vehicle.status,
            operation_status: vehicle.operation_status,
        }
    }
}
