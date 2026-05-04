//! / 领域用例层,定义核心业务逻辑

// 车辆相关用例
pub mod vehicle;

// 用户相关用例
pub mod user;

// 角色相关用例
// pub mod role;

// 设备相关用例
pub mod device;

// 称重数据相关用例（已拆分为目录）
pub mod weighing_data;

// 订单相关用例
pub mod order;

// 物流轨迹相关用例
// pub mod logistics_track;

// 车组相关用例
pub mod vehicle_group;

// 司机相关用例
pub mod driver;

// 财务相关用例
pub mod finance;

// 部门相关用例
pub mod department;

// 组织相关用例
pub mod organization;

// 统计相关用例
pub mod statistic;
pub mod statistics;

// 认证相关用例
pub mod auth;

// 数据同步相关用例
pub mod sync;

// 应用服务接口
pub mod application_service;

// OpenAPI 平台相关用例
pub mod openapi_platform;
