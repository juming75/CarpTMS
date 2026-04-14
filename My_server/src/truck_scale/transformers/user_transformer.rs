//! / 用户数据转换器
use crate::truck_scale::handlers::user_handler::UserInfo;
use crate::truck_scale::protocol::encoding;
use crate::truck_scale::transformers::vehicle_transformer::FieldType;
use anyhow::Result;

/// 用户数据转换器
pub struct UserTransformer;

impl UserTransformer {
    /// 创建新的用户数据转换器
    pub fn new() -> Self {
        Self
    }

    /// 将原始数据转换为用户信息
    pub fn to_user_info(&self, data: &[(u8, Vec<u8>)]) -> Result<UserInfo> {
        let mut user = UserInfo {
            user_id: String::new(),
            user_name: String::new(),
            password: String::new(),
            real_name: String::new(),
            user_type: 3,
            group_id: String::new(),
            company: String::new(),
            department: String::new(),
            tel: String::new(),
            mobile: String::new(),
            email: String::new(),
            address: String::new(),
            permission: String::new(),
            veh_group_list: String::new(),
            status: 0,
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
        };

        // 字段索引映射(根据 TF_CarManager 协议的字段顺序)
        for (field_index, field_data) in data.iter().enumerate() {
            match field_index {
                0 => user.user_id = self.parse_string(field_data),
                1 => user.user_name = self.parse_string(field_data),
                2 => user.password = self.parse_string(field_data), // 原始密码,需要哈希
                3 => user.real_name = self.parse_string(field_data),
                4 => user.user_type = self.parse_int32(field_data),
                5 => user.group_id = self.parse_string(field_data),
                6 => user.company = self.parse_string(field_data),
                7 => user.department = self.parse_string(field_data),
                8 => user.tel = self.parse_string(field_data),
                9 => user.mobile = self.parse_string(field_data),
                10 => user.email = self.parse_string(field_data),
                11 => user.address = self.parse_string(field_data),
                12 => user.permission = self.parse_string(field_data),
                13 => user.veh_group_list = self.parse_string(field_data),
                14 => user.status = self.parse_int32(field_data),
                15 => user.expiration_time = self.parse_string(field_data),
                16 => user.title = self.parse_string(field_data),
                17 => user.id_card = self.parse_string(field_data),
                18 => user.id_card_expire_date = self.parse_string(field_data),
                19 => user.education = self.parse_string(field_data),
                20 => user.birth_date = self.parse_string(field_data),
                21 => user.gender = self.parse_int32(field_data),
                22 => user.avatar = self.parse_string(field_data),
                23 => user.signature = self.parse_string(field_data),
                24 => user.last_login_time = self.parse_string(field_data),
                25 => user.last_login_ip = self.parse_string(field_data),
                26 => user.login_count = self.parse_int32(field_data),
                27 => user.remark = self.parse_string(field_data),
                28 => user.create_time = self.parse_string(field_data),
                29 => user.update_time = self.parse_string(field_data),
                30 => user.create_by = self.parse_string(field_data),
                31 => user.update_by = self.parse_string(field_data),
                _ => {}
            }
        }

        Ok(user)
    }

    /// 将用户信息转换为原始数据
    pub fn from_user_info(&self, user: &UserInfo) -> Result<Vec<(u8, Vec<u8>)>> {
        let fields = vec![
            (FieldType::String as u8, self.encode_string(&user.user_id)),
            (FieldType::String as u8, self.encode_string(&user.user_name)),
            (FieldType::String as u8, self.encode_string(&user.password)), // 已哈希的密码
            (FieldType::String as u8, self.encode_string(&user.real_name)),
            (FieldType::Int32 as u8, self.encode_int32(user.user_type)),
            (FieldType::String as u8, self.encode_string(&user.group_id)),
            (FieldType::String as u8, self.encode_string(&user.company)),
            (
                FieldType::String as u8,
                self.encode_string(&user.department),
            ),
            (FieldType::String as u8, self.encode_string(&user.tel)),
            (FieldType::String as u8, self.encode_string(&user.mobile)),
            (FieldType::String as u8, self.encode_string(&user.email)),
            (FieldType::String as u8, self.encode_string(&user.address)),
            (
                FieldType::String as u8,
                self.encode_string(&user.permission),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&user.veh_group_list),
            ),
            (FieldType::Int32 as u8, self.encode_int32(user.status)),
            (
                FieldType::String as u8,
                self.encode_string(&user.expiration_time),
            ),
            (FieldType::String as u8, self.encode_string(&user.title)),
            (FieldType::String as u8, self.encode_string(&user.id_card)),
            (
                FieldType::String as u8,
                self.encode_string(&user.id_card_expire_date),
            ),
            (FieldType::String as u8, self.encode_string(&user.education)),
            (
                FieldType::String as u8,
                self.encode_string(&user.birth_date),
            ),
            (FieldType::Int32 as u8, self.encode_int32(user.gender)),
            (FieldType::String as u8, self.encode_string(&user.avatar)),
            (FieldType::String as u8, self.encode_string(&user.signature)),
            (
                FieldType::String as u8,
                self.encode_string(&user.last_login_time),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&user.last_login_ip),
            ),
            (FieldType::Int32 as u8, self.encode_int32(user.login_count)),
            (FieldType::String as u8, self.encode_string(&user.remark)),
            (
                FieldType::String as u8,
                self.encode_string(&user.create_time),
            ),
            (
                FieldType::String as u8,
                self.encode_string(&user.update_time),
            ),
            (FieldType::String as u8, self.encode_string(&user.create_by)),
            (FieldType::String as u8, self.encode_string(&user.update_by)),
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

impl Default for UserTransformer {
    fn default() -> Self {
        Self::new()
    }
}
