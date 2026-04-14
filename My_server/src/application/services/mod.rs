//! Services - 应用服务模块
//!
//! 应用服务负责协调领域对象和基础设施服务，实现业务用例。

pub mod alert_service;
pub mod audit_log_service;
pub mod calibration_service;
pub mod department_service;
pub mod device_service;
pub mod driver_service;
pub mod finance_service;
pub mod location_service;
pub mod organization_service;
pub mod organization_settings_service;
pub mod order_service;
pub mod role_service;
pub mod settings_service;
pub mod statistics_service;
pub mod system_monitor_service;
pub mod user_service;
pub mod vehicle_group_service;
pub mod vehicle_service;
pub mod weighing_data_service;

pub use alert_service::*;
pub use audit_log_service::*;
pub use calibration_service::*;
pub use department_service::*;
pub use device_service::*;
pub use driver_service::*;
pub use finance_service::*;
pub use location_service::*;
pub use organization_service::*;
pub use organization_settings_service::*;
pub use order_service::*;
pub use role_service::*;
pub use settings_service::*;
pub use statistics_service::*;
pub use system_monitor_service::*;
pub use user_service::*;
pub use vehicle_group_service::*;
pub use vehicle_service::*;
pub use weighing_data_service::*;
