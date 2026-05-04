//! /! 车联网通讯模块
//! 统一处理所有车联网终端通讯,包括JT808、Truck Scale、GPRS等协议

// pub mod crud; // TODO: crud module removed, will be re-added later
pub mod protocol;
pub mod router;
pub mod server;
pub mod session;

// pub use crud::{VehicleCommCrud, VehicleCommRecord};
pub use protocol::{ProtocolParser, ProtocolType};
pub use router::MessageRouter;
pub use server::VehicleCommServer;
pub use session::SessionManager;
