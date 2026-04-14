//! / JT808协议模块
// 基于JT/T 808-2019协议规范
// 专门用于解析车载终端上传的GPS+传感器融合数据

pub mod command;
pub mod models;
pub mod parser;
pub mod session;
pub mod storage;

pub use command::{
    encode_jt808_frame, CommandError, CommandStatus, Jt808Command, Jt808CommandBuilder,
    Jt808CommandId, Jt808CommandQueue, SendCommand,
};
pub use models::*;
pub use parser::JT808Parser;
pub use session::{
    AuthStatus, DeviceConnect, DeviceDisconnect, DeviceHeartbeat, GetAllSessions,
    Jt808DeviceSession, Jt808SessionManager, QuerySession, SessionError, UpdateSession,
};
pub use storage::{
    Jt808AlarmRecord, Jt808CommandRecord, Jt808DeviceSessionRecord, Jt808LocationRecord,
    Jt808Repository,
};
