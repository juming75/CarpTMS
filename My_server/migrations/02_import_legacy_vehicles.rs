use sqlx::{PgPool, Connection, Error, sqlite::SqlitePool};
use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, Local};
use std::env;

// 原有C/S系统车辆模型
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct LegacyVehicle {
    // 基本信息
    #[serde(rename = "VehicleID")]
    vehicle_id: i32,
    #[serde(rename = "VehicleName")]
    vehicle_name: String,
    #[serde(rename = "LicensePlate")]
    license_plate: String,
    #[serde(rename = "VehicleType")]
    vehicle_type: String,
    #[serde(rename = "VehicleColor")]
    vehicle_color: String,
    #[serde(rename = "VehicleBrand")]
    vehicle_brand: String,
    #[serde(rename = "VehicleModel")]
    vehicle_model: String,
    #[serde(rename = "EngineNo")]
    engine_no: String,
    #[serde(rename = "FrameNo")]
    frame_no: String,
    #[serde(rename = "RegisterDate")]
    register_date: NaiveDateTime,
    #[serde(rename = "AnnualInspectionDate")]
    annual_inspection_date: NaiveDateTime,
    #[serde(rename = "InsuranceDate")]
    insurance_date: NaiveDateTime,
    #[serde(rename = "SeatingCapacity")]
    seating_capacity: i32,
    #[serde(rename = "LoadCapacity")]
    load_capacity: f64,
    #[serde(rename = "VehicleLength")]
    vehicle_length: f64,
    #[serde(rename = "VehicleWidth")]
    vehicle_width: f64,
    #[serde(rename = "VehicleHeight")]
    vehicle_height: f64,
    
    // 终端信息
    #[serde(rename = "DeviceID")]
    device_id: Option<String>,
    #[serde(rename = "TerminalType")]
    terminal_type: Option<String>,
    #[serde(rename = "CommunicationType")]
    communication_type: Option<String>,
    #[serde(rename = "SIMCardNo")]
    sim_card_no: Option<String>,
    #[serde(rename = "InstallDate")]
    install_date: Option<NaiveDateTime>,
    #[serde(rename = "InstallAddress")]
    install_address: Option<String>,
    #[serde(rename = "InstallTechnician")]
    install_technician: Option<String>,
    
    // 车主信息
    #[serde(rename = "OwnerNo")]
    owner_no: Option<String>,
    #[serde(rename = "OwnerName")]
    owner_name: Option<String>,
    #[serde(rename = "OwnerPhone")]
    owner_phone: Option<String>,
    #[serde(rename = "OwnerIDCard")]
    owner_id_card: Option<String>,
    #[serde(rename = "OwnerAddress")]
    owner_address: Option<String>,
    #[serde(rename = "OwnerEmail")]
    owner_email: Option<String>,
    
    // 运营信息
    #[serde(rename = "GroupID")]
    group_id: i32,
    #[serde(rename = "OperationStatus")]
    operation_status: i16,
    #[serde(rename = "OperationRoute")]
    operation_route: Option<String>,
    #[serde(rename = "OperationArea")]
    operation_area: Option<String>,
    #[serde(rename = "OperationCompany")]
    operation_company: Option<String>,
    #[serde(rename = "DriverName")]
    driver_name: Option<String>,
    #[serde(rename = "DriverPhone")]
    driver_phone: Option<String>,
    #[serde(rename = "DriverLicenseNo")]
    driver_license_no: Option<String>,
    
    // 财务信息
    #[serde(rename = "PurchasePrice")]
    purchase_price: Option<f64>,
    #[serde(rename = "AnnualFee")]
    annual_fee: Option<f64>,
    #[serde(rename = "InsuranceFee")]
    insurance_fee: Option<f64>,
    
    // 其他信息
    #[serde(rename = "Remark")]
    remark: Option<String>,
    #[serde(rename = "Status")]
    status: i16,
    #[serde(rename = "CreateTime")]
    create_time: NaiveDateTime,
    #[serde(rename = "UpdateTime")]
    update_time: Option<NaiveDateTime>,
    #[serde(rename = "CreateUserID")]
    create_user_id: i32,
    #[serde(rename = "UpdateUserID")]
    update_user_id: Option<i32>,
}

// 连接到原有数据库
async fn connect_legacy_db() -> Result<SqlitePool, Error> {
    // 从环境变量获取原有数据库连接信息
    let legacy_db_url = env::var("LEGACY_DB_URL").unwrap_or_else(|_| "sqlite:legacy.db".to_string());
    let pool = SqlitePool::connect(&legacy_db_url).await?;
    Ok(pool)
}

// 连接到新数据库
async fn connect_new_db() -> Result<PgPool, Error> {
    // 从环境变量获取新数据库连接信息
    let new_db_url = env::var("NEW_DB_URL").unwrap_or_else(|_| "postgres://postgres:123@localhost:5432/tms_db?sslmode=disable".to_string());
    let pool = PgPool::connect(&new_db_url).await?;
    Ok(pool)
}

// 从原有数据库获取车辆数据
async fn get_legacy_vehicles(pool: &SqlitePool) -> Result<Vec<LegacyVehicle>, Error> {
    let vehicles = sqlx::query_as!(LegacyVehicle, "SELECT * FROM vehicles")
        .fetch_all(pool)
        .await?;
    Ok(vehicles)
}

// 将车辆数据迁移到新数据库
async fn migrate_vehicle_data(legacy_vehicles: Vec<LegacyVehicle>, new_pool: &PgPool) -> Result<usize, Error> {
    let mut migrated_count = 0;
    
    for legacy_vehicle in legacy_vehicles {
        // 转换为新数据库的车辆数据格式
        let result = sqlx::query!(r#"INSERT INTO vehicles (
            -- 基本信息
            vehicle_name, license_plate, vehicle_type, vehicle_color, 
            vehicle_brand, vehicle_model, engine_no, frame_no, 
            register_date, 年检_date, insurance_date, seating_capacity, 
            load_capacity, vehicle_length, vehicle_width, vehicle_height, 
            
            -- 终端信息
            device_id, terminal_type, communication_type, sim_card_no, 
            install_date, install_address, install_technician, 
            
            -- 车主信息
            own_no, own_name, own_phone, own_id_card, 
            own_address, own_email, 
            
            -- 运营信息
            group_id, operation_status, operation_route, operation_area, 
            operation_company, driver_name, driver_phone, driver_license_no, 
            
            -- 财务信息
            purchase_price, annual_fee, insurance_fee, 
            
            -- 其他信息
            remark, status, create_time, update_time, create_user_id, update_user_id
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, 
            $17, $18, $19, $20, $21, $22, $23, 
            $24, $25, $26, $27, $28, $29, 
            $30, $31, $32, $33, $34, $35, $36, $37, 
            $38, $39, $40, 
            $41, $42, $43, $44, $45, $46
        ) ON CONFLICT (license_plate) DO NOTHING"#, 
            
            // 基本信息
            legacy_vehicle.vehicle_name,
            legacy_vehicle.license_plate,
            legacy_vehicle.vehicle_type,
            legacy_vehicle.vehicle_color,
            legacy_vehicle.vehicle_brand,
            legacy_vehicle.vehicle_model,
            legacy_vehicle.engine_no,
            legacy_vehicle.frame_no,
            legacy_vehicle.register_date,
            legacy_vehicle.annual_inspection_date, // 映射到年检_date
            legacy_vehicle.insurance_date,
            legacy_vehicle.seating_capacity,
            legacy_vehicle.load_capacity,
            legacy_vehicle.vehicle_length,
            legacy_vehicle.vehicle_width,
            legacy_vehicle.vehicle_height,
            
            // 终端信息
            legacy_vehicle.device_id,
            legacy_vehicle.terminal_type,
            legacy_vehicle.communication_type,
            legacy_vehicle.sim_card_no,
            legacy_vehicle.install_date,
            legacy_vehicle.install_address,
            legacy_vehicle.install_technician,
            
            // 车主信息
            legacy_vehicle.owner_no,
            legacy_vehicle.owner_name,
            legacy_vehicle.owner_phone,
            legacy_vehicle.owner_id_card,
            legacy_vehicle.owner_address,
            legacy_vehicle.owner_email,
            
            // 运营信息
            legacy_vehicle.group_id,
            legacy_vehicle.operation_status,
            legacy_vehicle.operation_route,
            legacy_vehicle.operation_area,
            legacy_vehicle.operation_company,
            legacy_vehicle.driver_name,
            legacy_vehicle.driver_phone,
            legacy_vehicle.driver_license_no,
            
            // 财务信息
            legacy_vehicle.purchase_price,
            legacy_vehicle.annual_fee,
            legacy_vehicle.insurance_fee,
            
            // 其他信息
            legacy_vehicle.remark,
            legacy_vehicle.status,
            legacy_vehicle.create_time,
            legacy_vehicle.update_time,
            legacy_vehicle.create_user_id,
            legacy_vehicle.update_user_id
        )
        .execute(new_pool)
        .await;
        
        if let Ok(result) = result {
            if result.rows_affected() > 0 {
                migrated_count += 1;
            }
        }
    }
    
    Ok(migrated_count)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("开始从原有C/S系统迁移车辆数据...");
    
    // 连接原有数据库
    println!("连接原有数据库...");
    let legacy_pool = connect_legacy_db().await?;
    println!("成功连接到原有数据库");
    
    // 连接新数据库
    println!("连接新数据库...");
    let new_pool = connect_new_db().await?;
    println!("成功连接到新数据库");
    
    // 获取原有车辆数据
    println!("获取原有车辆数据...");
    let legacy_vehicles = get_legacy_vehicles(&legacy_pool).await?;
    println!("共获取到 {} 辆车辆数据", legacy_vehicles.len());
    
    // 迁移车辆数据
    println!("开始迁移车辆数据...");
    let migrated_count = migrate_vehicle_data(legacy_vehicles, &new_pool).await?;
    println!("成功迁移 {} 辆车辆数据", migrated_count);
    
    println!("车辆数据迁移完成！");
    
    Ok(())
}




