//! / 车辆管理处理器
use crate::truck_scale::db::TruckScaleDb;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// 车辆信息(43个字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleInfo {
    // 基本信息
    pub vehicle_id: String,  // 车辆ID
    pub plate_no: String,    // 车牌号
    pub terminal_no: String, // 终端号
    pub sim_no: String,      // SIM卡号
    pub engine_no: String,   // 发动机号
    pub frame_no: String,    // 车架号

    // 车主信息
    pub owner_name: String,    // 车主姓名
    pub owner_tel: String,     // 车主电话
    pub owner_address: String, // 车主地址

    // 车辆信息
    pub vehicle_type: String,  // 车辆类型
    pub vehicle_color: String, // 车辆颜色
    pub vehicle_brand: String, // 车辆品牌
    pub vehicle_model: String, // 车辆型号

    // 运营信息
    pub group_id: String,       // 所属车组
    pub driver_name: String,    // 驾驶员姓名
    pub driver_tel: String,     // 驾驶员电话
    pub driver_license: String, // 驾驶证号

    // 称重信息
    pub max_weight: f64,   // 最大载重
    pub tare_weight: f64,  // 自重
    pub rated_weight: f64, // 额定载重

    // 其他字段(共43个)
    pub length: f64,                    // 车长
    pub width: f64,                     // 车宽
    pub height: f64,                    // 车高
    pub fuel_type: String,              // 燃油类型
    pub manufacturer: String,           // 制造商
    pub manufacture_date: String,       // 制造日期
    pub registration_date: String,      // 注册日期
    pub insurance_expire_date: String,  // 保险到期日期
    pub annual_inspection_date: String, // 年检日期
    pub remark: String,                 // 备注

    // 系统字段
    pub status: i32,         // 状态:0=正常,1=删除
    pub create_time: String, // 创建时间
    pub update_time: String, // 更新时间
    pub create_by: String,   // 创建人
    pub update_by: String,   // 更新人
}

/// 车组信息(5个字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleGroup {
    pub group_id: String,       // 车组ID
    pub parent_id: String,      // 父车组ID
    pub group_name: String,     // 车组名称
    pub contact_people: String, // 联系人
    pub contact_tel: String,    // 联系电话
}

/// 车辆管理处理器
pub struct VehicleHandler {
    db: TruckScaleDb,
}

impl VehicleHandler {
    /// 创建新的车辆管理处理器
    pub fn new(pool: PgPool) -> Self {
        Self {
            db: TruckScaleDb::new(pool.into()),
        }
    }

    /// 从数据库操作器创建
    pub fn from_db(db: TruckScaleDb) -> Self {
        Self { db }
    }

    /// 查询车辆信息
    pub async fn query_vehicle(&self, vehicle_id: &str) -> Result<Option<VehicleInfo>> {
        let vehicle_data = self.db.query_vehicle(vehicle_id).await?;
        Ok(vehicle_data.and_then(|data| serde_json::from_value(data).ok()))
    }

    /// 添加车辆
    pub async fn add_vehicle(&self, vehicle: VehicleInfo) -> Result<String> {
        let vehicle_data = serde_json::to_value(&vehicle)?;
        self.db.add_vehicle(vehicle_data).await
    }

    /// 更新车辆
    pub async fn update_vehicle(&self, vehicle: VehicleInfo) -> Result<()> {
        let vehicle_data = serde_json::to_value(&vehicle)?;
        self.db.update_vehicle(vehicle_data).await
    }

    /// 删除车辆
    pub async fn delete_vehicle(&self, vehicle_id: &str, delete_by: &str) -> Result<()> {
        self.db.delete_vehicle(vehicle_id, delete_by).await
    }

    /// 查询车组
    pub async fn query_vehicle_group(&self, group_id: &str) -> Result<Option<VehicleGroup>> {
        let group_data = self.db.query_vehicle_group(group_id).await?;
        group_data
            .map(|g| serde_json::from_value(g).map_err(anyhow::Error::from))
            .transpose()
    }

    /// 添加车组
    pub async fn add_vehicle_group(&self, group: VehicleGroup) -> Result<String> {
        let group_data = serde_json::to_value(&group)?;
        self.db.add_vehicle_group(group_data).await
    }

    /// 更新车组
    pub async fn update_vehicle_group(&self, group: VehicleGroup) -> Result<()> {
        let group_data = serde_json::to_value(&group)?;
        self.db.update_vehicle_group(group_data).await
    }

    /// 删除车组
    pub async fn delete_vehicle_group(&self, group_id: &str) -> Result<()> {
        // 假设当前用户为admin,实际应该从上下文获取
        self.db.delete_vehicle_group(group_id, "admin").await
    }

    /// 查询车辆列表
    pub async fn query_vehicle_list(&self, page: i32, page_size: i32) -> Result<Vec<VehicleInfo>> {
        // 计算偏移量
        let offset = (page - 1) * page_size;

        // 从数据库查询车辆列表
        let vehicles_data = self.db.query_vehicle_list(offset, page_size).await?;

        // 转换为 VehicleInfo 类型
        let vehicles = vehicles_data
            .into_iter()
            .filter_map(|data| serde_json::from_value(data).ok())
            .collect();

        Ok(vehicles)
    }
}

// impl Default for VehicleHandler {
//     fn default() -> Self {
//         Self::new()
//     }
// }
