//! / 设备领域实体

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 设备实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// 设备ID
    pub device_id: String,
    /// 设备名称
    pub device_name: String,
    /// 设备类型
    pub device_type: String,
    /// 设备型号
    pub device_model: String,
    /// 制造商
    pub manufacturer: String,
    /// 序列号
    pub serial_number: String,
    /// 通信类型
    pub communication_type: String,
    /// SIM卡号
    pub sim_card_no: Option<String>,
    /// IP地址
    pub ip_address: Option<String>,
    /// 端口
    pub port: Option<i32>,
    /// MAC地址
    pub mac_address: Option<String>,
    /// 安装日期
    pub install_date: Option<NaiveDateTime>,
    /// 安装地址
    pub install_address: Option<String>,
    /// 安装技师
    pub install_technician: Option<String>,
    /// 状态
    pub status: i16,
    /// 备注
    pub remark: Option<String>,
    /// 创建用户ID
    pub create_user_id: i32,
    /// 创建时间
    pub create_time: NaiveDateTime,
    /// 更新时间
    pub update_time: Option<NaiveDateTime>,
    /// 更新用户ID
    pub update_user_id: Option<i32>,
}

/// 设备创建实体
#[derive(Debug, Clone, Serialize, Deserialize, validator::Validate)]
pub struct DeviceCreate {
    /// 设备ID
    #[validate(length(min = 1, max = 50))]
    pub device_id: String,
    /// 设备名称
    #[validate(length(min = 1, max = 100))]
    pub device_name: String,
    /// 设备类型
    #[validate(length(min = 1, max = 50))]
    pub device_type: String,
    /// 设备型号
    #[validate(length(min = 1, max = 50))]
    pub device_model: String,
    /// 制造商
    #[validate(length(min = 1, max = 100))]
    pub manufacturer: String,
    /// 序列号
    #[validate(length(min = 1, max = 50))]
    pub serial_number: String,
    /// 通信类型
    #[validate(length(min = 1, max = 50))]
    pub communication_type: String,
    /// SIM卡号
    pub sim_card_no: Option<String>,
    /// IP地址
    pub ip_address: Option<String>,
    /// 端口
    pub port: Option<i32>,
    /// MAC地址
    pub mac_address: Option<String>,
    /// 安装日期
    pub install_date: Option<NaiveDateTime>,
    /// 安装地址
    pub install_address: Option<String>,
    /// 安装技师
    pub install_technician: Option<String>,
    /// 状态
    pub status: i16,
    /// 备注
    pub remark: Option<String>,
    /// 创建用户ID
    pub create_user_id: i32,
}

/// 设备更新实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceUpdate {
    /// 设备名称
    pub device_name: Option<String>,
    /// 设备类型
    pub device_type: Option<String>,
    /// 设备型号
    pub device_model: Option<String>,
    /// 制造商
    pub manufacturer: Option<String>,
    /// 序列号
    pub serial_number: Option<String>,
    /// 通信类型
    pub communication_type: Option<String>,
    /// SIM卡号
    pub sim_card_no: Option<String>,
    /// IP地址
    pub ip_address: Option<String>,
    /// 端口
    pub port: Option<i32>,
    /// MAC地址
    pub mac_address: Option<String>,
    /// 安装日期
    pub install_date: Option<NaiveDateTime>,
    /// 安装地址
    pub install_address: Option<String>,
    /// 安装技师
    pub install_technician: Option<String>,
    /// 状态
    pub status: Option<i16>,
    /// 备注
    pub remark: Option<String>,
    /// 更新用户ID
    pub update_user_id: Option<i32>,
}

/// 设备查询条件实体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceQuery {
    /// 页码
    pub page: Option<i32>,
    /// 每页大小
    pub page_size: Option<i32>,
    /// 设备ID
    pub device_id: Option<String>,
    /// 设备名称
    pub device_name: Option<String>,
    /// 设备类型
    pub device_type: Option<String>,
    /// 制造商
    pub manufacturer: Option<String>,
    /// 状态
    pub status: Option<i16>,
}
