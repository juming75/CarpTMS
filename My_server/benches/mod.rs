//! /! Benchmark Module
//!
//! Provides performance benchmarks for key system components

pub mod api_benchmark;
pub mod config_center_benchmark;
pub mod health_benchmark;
pub mod secret_manager_benchmark;

pub use api_benchmark::api_benchmarks;
pub use config_center_benchmark::benchmarks as config_center_benchmarks;
pub use health_benchmark::benchmarks as health_benchmarks;
pub use secret_manager_benchmark::benchmarks as secret_manager_benchmarks;
