//! / JT808 指令下发
// 支持向车载终端发送各种控制指令

use log::{debug, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// JT808 指令ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jt808CommandId {
    /// 终端通用应答
    TerminalAck = 0x8001,
    /// 终端注册应答
    RegisterAck = 0x8100,
    /// 终端鉴权
    Auth = 0x8102,
    /// 设置终端参数
    SetParams = 0x8103,
    /// 查询终端参数
    QueryParams = 0x8104,
    /// 终端控制
    TerminalControl = 0x8105,
    /// 位置信息查询
    QueryLocation = 0x8201,
    /// 文本信息下发
    SendText = 0x8300,
    /// 事件设置
    SetEvents = 0x8400,
    /// 下发车辆识别号
    SetVehicleId = 0x8800,
    /// 电话回拨
    PhoneDial = 0x8801,
    /// 设置电话本
    SetPhonebook = 0x8802,
    /// 立即拍照
    TakePhoto = 0x8803,
    /// 录音开始
    AudioStart = 0x8804,
    /// 录音结束
    AudioStop = 0x8805,
}

/// JT808 指令状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    /// 待发送
    Pending,
    /// 已发送
    Sent,
    /// 已确认
    Acknowledged,
    /// 超时
    Timeout,
    /// 失败
    Failed,
}

/// JT808 指令记录
#[derive(Debug, Clone)]
pub struct Jt808Command {
    /// 命令ID
    pub cmd_id: Jt808CommandId,
    /// 命令名称
    pub cmd_name: String,
    /// 参数
    pub params: HashMap<String, serde_json::Value>,
    /// 状态
    pub status: CommandStatus,
    /// 发送时间
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 确认时间
    pub ack_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 重试次数
    pub retry_count: u32,
    /// 响应数据
    pub response_data: Option<Vec<u8>>,
}

/// 指令结果
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Command generation failed: {0}")]
    GenerationFailed(String),

    #[error("Command send failed: {0}")]
    SendFailed(String),

    #[error("Command timeout")]
    Timeout,

    #[error("Command rejected by device: {0}")]
    Rejected(String),
}

/// 发送指令消息
#[derive(Clone)]
pub struct SendCommand {
    pub device_id: String,
    pub cmd_id: Jt808CommandId,
    pub params: HashMap<String, serde_json::Value>,
}

/// 指令回调
pub type CommandCallback = Box<dyn Fn(Result<Vec<u8>, CommandError>) + Send + Sync>;

/// JT808 指令队列
pub struct Jt808CommandQueue {
    /// 指令队列:device_id -> Vec<Command>
    queues: Arc<RwLock<HashMap<String, Vec<Jt808Command>>>>,
    /// 超时时间(秒)
    timeout_seconds: u64,
    /// 最大重试次数
    max_retries: u32,
}

impl Jt808CommandQueue {
    /// 创建新的指令队列
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }

    /// 配置队列
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout_seconds = timeout;
        self
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// 添加指令到队列
    pub async fn add_command(&self, device_id: &str, command: Jt808Command) {
        let mut queues = self.queues.write().await;
        queues
            .entry(device_id.to_string())
            .or_insert_with(Vec::new)
            .push(command);
        info!("Command queued for device {}", device_id);
    }

    /// 获取设备的待发送指令
    pub async fn get_pending_commands(&self, device_id: &str) -> Vec<Jt808Command> {
        let queues = self.queues.read().await;
        queues.get(device_id).cloned().unwrap_or_default()
    }

    /// 更新指令状态
    pub async fn update_command_status(
        &self,
        device_id: &str,
        cmd_id: Jt808CommandId,
        status: CommandStatus,
        response_data: Option<Vec<u8>>,
    ) {
        let mut queues = self.queues.write().await;
        if let Some(cmds) = queues.get_mut(device_id) {
            for cmd in cmds.iter_mut() {
                if cmd.cmd_id == cmd_id {
                    cmd.status = status;
                    if status == CommandStatus::Sent {
                        cmd.sent_at = Some(chrono::Utc::now());
                    } else if status == CommandStatus::Acknowledged {
                        cmd.ack_at = Some(chrono::Utc::now());
                    }
                    cmd.response_data = response_data;
                    debug!(
                        "Command 0x{:04X} status updated to {:?}",
                        cmd_id as u16, status
                    );
                    break;
                }
            }
        }
    }

    /// 清理已完成的指令
    pub async fn cleanup_completed_commands(&self, device_id: &str) {
        let mut queues = self.queues.write().await;
        if let Some(cmds) = queues.get_mut(device_id) {
            cmds.retain(|cmd| matches!(cmd.status, CommandStatus::Pending | CommandStatus::Sent));
            debug!("Cleaned up completed commands for {}", device_id);
        }
    }
}

impl Default for Jt808CommandQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// JT808 指令生成器
pub struct Jt808CommandBuilder;

impl Jt808CommandBuilder {
    /// 生成终端通用应答
    pub fn build_terminal_ack(flow_no: u16, msg_id: u16, result: u8) -> Vec<u8> {
        let mut body = vec![0u8; 5];
        body[0..2].copy_from_slice(&flow_no.to_be_bytes());
        body[2..4].copy_from_slice(&msg_id.to_be_bytes());
        body[4] = result;

        body
    }

    /// 生成终端注册应答
    pub fn build_register_ack(flow_no: u16, result: u8, auth_code: &str) -> Vec<u8> {
        let mut body = vec![0u8; 5];
        body[0..2].copy_from_slice(&flow_no.to_be_bytes());
        body[2] = result;
        // 添加鉴权码
        body.extend_from_slice(auth_code.as_bytes());

        body
    }

    /// 生成终端鉴权
    pub fn build_auth(flow_no: u16, auth_code: &str) -> Vec<u8> {
        let mut body = vec![0u8; 2];
        body[0..2].copy_from_slice(&flow_no.to_be_bytes());
        body.extend_from_slice(auth_code.as_bytes());

        body
    }

    /// 生成设置终端参数
    pub fn build_set_params(flow_no: u16, params: &HashMap<String, serde_json::Value>) -> Vec<u8> {
        let mut body = vec![0u8; 2];
        body[0..2].copy_from_slice(&flow_no.to_be_bytes());

        // 添加参数总数
        let param_count = params.len() as u8;
        body.push(param_count);

        // 添加参数项
        for (key, value) in params.iter() {
            if let Some(id) = Self::get_param_id(key) {
                body.push(id);
                // 简化:假设所有参数长度为4字节
                body.push(4);
                if let Some(num) = value.as_i64() {
                    body.extend_from_slice(&(num as u32).to_be_bytes());
                } else if let Some(s) = value.as_str() {
                    let s_bytes = s.as_bytes();
                    body.extend_from_slice(s_bytes);
                    // 调整长度
                    let len_pos = body.len() - s_bytes.len() - 1;
                    body[len_pos] = s.len() as u8;
                }
            }
        }

        body
    }

    /// 获取参数ID
    fn get_param_id(key: &str) -> Option<u8> {
        match key {
            "heartbeat_interval" => Some(0x0001),
            "tcp_timeout" => Some(0x0002),
            "retry_times" => Some(0x0003),
            "apn" => Some(0x0010),
            "server_ip" => Some(0x0011),
            "server_port" => Some(0x0012),
            _ => None,
        }
    }

    /// 生成终端控制
    pub fn build_terminal_control(flow_no: u16, cmd_word: u32) -> Vec<u8> {
        let mut body = vec![0u8; 6];
        body[0..2].copy_from_slice(&flow_no.to_be_bytes());
        body[2..6].copy_from_slice(&cmd_word.to_be_bytes());

        body
    }

    /// 生成文本信息下发
    pub fn build_send_text(flow_no: u16, text: &str, flags: u8) -> Vec<u8> {
        let mut body = vec![0u8; 2];
        body[0..2].copy_from_slice(&flow_no.to_be_bytes());
        body.push(flags);
        body.push(text.len() as u8);
        body.extend_from_slice(text.as_bytes());

        body
    }

    /// 生成位置信息查询
    pub fn build_query_location(flow_no: u16) -> Vec<u8> {
        let mut body = vec![0u8; 2];
        body[0..2].copy_from_slice(&flow_no.to_be_bytes());

        body
    }

    /// 立即拍照
    pub fn build_take_photo(flow_no: u16, channel: u8, interval: u16, count: u8) -> Vec<u8> {
        let mut body = vec![0u8; 6];
        body[0..2].copy_from_slice(&flow_no.to_be_bytes());
        body[2] = channel;
        body[3..5].copy_from_slice(&interval.to_be_bytes());
        body[5] = count;

        body
    }
}

/// 编码 JT808 协议帧
pub fn encode_jt808_frame(
    phone: &str,
    flow_no: u16,
    msg_id: u16,
    body: &[u8],
) -> Result<Vec<u8>, CommandError> {
    debug!(
        "Encoding JT808 frame: msg_id=0x{:04X}, body_len={}",
        msg_id,
        body.len()
    );

    let body_len = body.len() as u16;

    // 构建消息属性:子包标识(0) + 加密(0) + 分包(0) + 体长度(10bit)
    let msg_attr = body_len & 0x03FF;

    // 构建完整帧
    let mut frame = vec![0x7E]; // 起始标识

    // 消息ID
    frame.extend_from_slice(&msg_id.to_be_bytes());

    // 消息属性
    frame.extend_from_slice(&msg_attr.to_be_bytes());

    // 手机号(BCD编码)
    let phone_bytes = encode_bcd_phone(phone);
    if phone_bytes.len() != 6 {
        return Err(CommandError::GenerationFailed(
            "Invalid phone number".to_string(),
        ));
    }
    frame.extend_from_slice(&phone_bytes);

    // 流水号
    frame.extend_from_slice(&flow_no.to_be_bytes());

    // 消息体
    frame.extend_from_slice(body);

    // 计算校验码
    let checksum = calculate_checksum(&frame[1..]);
    frame.push(checksum);

    // 结束标识
    frame.push(0x7E);

    // 转义处理
    let escaped = escape_data(&frame);

    Ok(escaped)
}

/// BCD编码手机号
fn encode_bcd_phone(phone: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let _digits: Vec<u8> = phone
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|d| d as u8)
        .collect();

    // 填充到12位
    let padded = format!("{:0>12}", phone.replace("+86", "").replace("86", ""));

    for i in (0..12).step_by(2) {
        let high = padded
            .chars()
            .nth(i)
            .and_then(|c| c.to_digit(10))
            .unwrap_or(0) as u8;
        let low = padded
            .chars()
            .nth(i + 1)
            .and_then(|c| c.to_digit(10))
            .unwrap_or(0) as u8;
        result.push(high << 4 | low);
    }

    result
}

/// 计算校验码
fn calculate_checksum(data: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for &byte in data {
        sum = sum.wrapping_add(byte);
    }
    sum
}

/// 转义数据
fn escape_data(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();

    for &byte in data {
        match byte {
            0x7E => {
                result.push(0x7D);
                result.push(0x02);
            }
            0x7D => {
                result.push(0x7D);
                result.push(0x01);
            }
            _ => {
                result.push(byte);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_frame() {
        let frame = encode_jt808_frame("12345678901", 1, 0x8001, &[0x00, 0x01, 0x00, 0x00, 0x00]);
        assert!(frame.is_ok());
        assert_eq!(frame.unwrap()[0], 0x7E);
    }

    #[test]
    fn test_command_queue() {
        let queue = Jt808CommandQueue::new();
        let command = Jt808Command {
            cmd_id: Jt808CommandId::TerminalControl,
            cmd_name: "TerminalControl".to_string(),
            params: HashMap::new(),
            status: CommandStatus::Pending,
            sent_at: None,
            ack_at: None,
            retry_count: 0,
            response_data: None,
        };

        // 添加指令
        tokio::runtime::Handle::current().block_on(async {
            queue.add_command("123456", command).await;

            // 获取指令
            let commands = queue.get_pending_commands("123456").await;
            assert_eq!(commands.len(), 1);
        });
    }
}
