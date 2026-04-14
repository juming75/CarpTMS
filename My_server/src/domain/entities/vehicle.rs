//! / 车辆领域实体

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 车辆实体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, FromRow)]
pub struct Vehicle {
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
    /// 创建时间
    pub create_time: NaiveDateTime,
    /// 更新时间
    pub update_time: Option<NaiveDateTime>,
    /// 创建用户ID
    pub create_user_id: i32,
    /// 更新用户ID
    pub update_user_id: Option<i32>,
}

/// 车辆创建实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleCreate {
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

/// 车辆更新实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VehicleUpdate {
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

/// 车辆查询条件实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VehicleQuery {
    /// 页码
    pub page: Option<i32>,
    /// 每页大小
    pub page_size: Option<i32>,
    /// 车辆名称
    pub vehicle_name: Option<String>,
    /// 车牌号
    pub license_plate: Option<String>,
    /// 车辆类型
    pub vehicle_type: Option<String>,
    /// 状态
    pub status: Option<i16>,
}
