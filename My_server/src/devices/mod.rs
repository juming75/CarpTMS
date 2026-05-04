//! 设备管理模块
//!
//! 管理车载终端、无人机、对讲机等设备

pub mod drones;
#[cfg(test)]
mod drones_test;

pub mod radios;
#[cfg(test)]
mod radios_test;

pub use drones::{DroneCommand, DroneInfo, DroneService, DroneTelemetry};
pub use radios::{RadioCommand, RadioInfo, RadioService, RadioTelemetry};
