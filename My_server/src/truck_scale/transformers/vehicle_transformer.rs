//! / 车辆数据转换器
use crate::truck_scale::handlers::vehicle_handler::VehicleInfo;
use crate::truck_scale::protocol::encoding;
use anyhow::Result;

/// 字段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    String = 0x01,
    Int32 = 0x02,
    Int64 = 0x03,
    Float64 = 0x04,
    DateTime = 0x05,
    Boolean = 0x06,
    Binary = 0x07,
}

impl FieldType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0x01 => FieldType::String,
            0x02 => FieldType::Int32,
            0x03 => FieldType::Int64,
            0x04 => FieldType::Float64,
            0x05 => FieldType::DateTime,
            0x06 => FieldType::Boolean,
            0x07 => FieldType::Binary,
            _ => FieldType::String,
        }
    }
}

/// 车辆数据转换器
pub struct VehicleTransformer;

impl VehicleTransformer {
    /// 创建新的车辆数据转换器
    pub fn new() -> Self {
        Self
    }

    /// 将原始数据转换为车辆信息
    pub fn to_vehicle_info(&self, data: &[(u8, Vec<u8>)]) -> Result<VehicleInfo> {
        let mut vehicle = VehicleInfo {
            vehicle_id: String::new(),
            plate_no: String::new(),
            terminal_no: String::new(),
            sim_no: String::new(),
            engine_no: String::new(),
            frame_no: String::new(),
            owner_name: String::new(),
            owner_tel: String::new(),
            owner_address: String::new(),
            vehicle_type: String::new(),
            vehicle_color: String::new(),
            vehicle_brand: String::new(),
            vehicle_model: String::new(),
            group_id: String::new(),
            driver_name: String::new(),
            driver_tel: String::new(),
            driver_license: String::new(),
            max_weight: 0.0,
            tare_weight: 0.0,
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
            status: 0,
            create_time: String::new(),
            update_time: String::new(),
            create_by: String::new(),
            update_by: String::new(),
        };

        // 字段索引映射(根据 TF_CarManager 协议的字段顺序)
        // 这是一个示例映射,实际需要根据协议文档调整
        for (field_index, field_data) in data.iter().enumerate() {
            let _field_type = FieldType::from_u8(field_data.0);

            match field_index {
                0 => vehicle.vehicle_id = self.parse_string(field_data),
                1 => vehicle.plate_no = self.parse_string(field_data),
                2 => vehicle.terminal_no = self.parse_string(field_data),
                3 => vehicle.sim_no = self.parse_string(field_data),
                4 => vehicle.engine_no = self.parse_string(field_data),
                5 => vehicle.frame_no = self.parse_string(field_data),
                6 => vehicle.owner_name = self.parse_string(field_data),
                7 => vehicle.owner_tel = self.parse_string(field_data),
                8 => vehicle.owner_address = self.parse_string(field_data),
                9 => vehicle.vehicle_type = self.parse_string(field_data),
                10 => vehicle.vehicle_color = self.parse_string(field_data),
                11 => vehicle.vehicle_brand = self.parse_string(field_data),
                12 => vehicle.vehicle_model = self.parse_string(field_data),
                13 => vehicle.group_id = self.parse_string(field_data),
                14 => vehicle.driver_name = self.parse_string(field_data),
                15 => vehicle.driver_tel = self.parse_string(field_data),
                16 => vehicle.driver_license = self.parse_string(field_data),
                17 => vehicle.max_weight = self.parse_float64(field_data),
                18 => vehicle.tare_weight = self.parse_float64(field_data),
                19 => vehicle.rated_weight = self.parse_float64(field_data),
                20 => vehicle.length = self.parse_float64(field_data),
                21 => vehicle.width = self.parse_float64(field_data),
                22 => vehicle.height = self.parse_float64(field_data),
                23 => vehicle.fuel_type = self.parse_string(field_data),
                24 => vehicle.manufacturer = self.parse_string(field_data),
                25 => vehicle.manufacture_date = self.parse_string(field_data),
                26 => vehicle.registration_date = self.parse_string(field_data),
                27 => vehicle.insurance_expire_date = self.parse_string(field_data),
                28 => vehicle.annual_inspection_date = self.parse_string(field_data),
                29 => vehicle.remark = self.parse_string(field_data),
                30 => vehicle.status = self.parse_int32(field_data),
                31 => vehicle.create_time = self.parse_string(field_data),
                32 => vehicle.update_time = self.parse_string(field_data),
                33 => vehicle.create_by = self.parse_string(field_data),
                34 => vehicle.update_by = self.parse_string(field_data),
                _ => {}
            }
        }

        Ok(vehicle)
    }

    /// 将车辆信息转换为原始数据
    pub fn from_vehicle_info(&self, vehicle: &VehicleInfo) -> Result<Vec<(u8, Vec<u8>)>> {
        let fields = vec![
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.vehicle_id),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.plate_no),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.terminal_no),
            ),
            (FieldType::String as u8, self.encode_string(&vehicle.sim_no)),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.engine_no),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.frame_no),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.owner_name),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.owner_tel),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.owner_address),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.vehicle_type),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.vehicle_color),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.vehicle_brand),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.vehicle_model),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.group_id),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.driver_name),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.driver_tel),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.driver_license),
            ),
            (
                FieldType::Float64 as u8,
                self.encode_float64(vehicle.max_weight),
            ),
            (
                FieldType::Float64 as u8,
                self.encode_float64(vehicle.tare_weight),
            ),
            (
                FieldType::Float64 as u8,
                self.encode_float64(vehicle.rated_weight),
            ),
            (
                FieldType::Float64 as u8,
                self.encode_float64(vehicle.length),
            ),
            (FieldType::Float64 as u8, self.encode_float64(vehicle.width)),
            (
                FieldType::Float64 as u8,
                self.encode_float64(vehicle.height),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.fuel_type),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.manufacturer),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.manufacture_date),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.registration_date),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.insurance_expire_date),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.annual_inspection_date),
            ),
            (FieldType::String as u8, self.encode_string(&vehicle.remark)),
            (FieldType::Int32 as u8, self.encode_int32(vehicle.status)),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.create_time),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.update_time),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.create_by),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&vehicle.update_by),
            ),
        ];

        Ok(fields)
    }

    // ==================== 辅助方法 ====================

    /// 解析字符串(自动处理 GB2312 编码)
    fn parse_string(&self, data: &(u8, Vec<u8>)) -> String {
        if data.1.is_empty() {
            return String::new();
        }
        encoding::auto_convert_to_utf8(&data.1)
    }

    /// 编码字符串(转换为 GB2312)
    fn encode_string(&self, s: &str) -> Vec<u8> {
        encoding::utf8_to_gb2312(s)
    }

    /// 解析 float64
    fn parse_float64(&self, data: &(u8, Vec<u8>)) -> f64 {
        if data.1.len() >= 8 {
            let bytes: [u8; 8] = data.1[..8].try_into().unwrap_or([0u8; 8]);
            f64::from_le_bytes(bytes)
        } else {
            0.0
        }
    }

    /// 编码 float64
    fn encode_float64(&self, value: f64) -> Vec<u8> {
        value.to_le_bytes().to_vec()
    }

    /// 解析 int32
    fn parse_int32(&self, data: &(u8, Vec<u8>)) -> i32 {
        if data.1.len() >= 4 {
            let bytes: [u8; 4] = data.1[..4].try_into().unwrap_or([0u8; 4]);
            i32::from_le_bytes(bytes)
        } else {
            0
        }
    }

    /// 编码 int32
    fn encode_int32(&self, value: i32) -> Vec<u8> {
        value.to_le_bytes().to_vec()
    }
}

impl Default for VehicleTransformer {
    fn default() -> Self {
        Self::new()
    }
}
