//! 自动化测试支持模块
//!
//! 提供自动化测试方案支持：
//! - 单元测试框架
//! - 集成测试模拟
//! - 性能测试工具
//! - 异常测试场景

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// 测试名称
    pub test_name: String,
    /// 测试是否通过
    pub passed: bool,
    /// 测试耗时（毫秒）
    pub duration_ms: u64,
    /// 错误信息（如果有）
    pub error_message: Option<String>,
    /// 测试时间
    pub test_time: String,
}

/// 测试套件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// 套件名称
    pub suite_name: String,
    /// 测试结果列表
    pub results: Vec<TestResult>,
    /// 总耗时（毫秒）
    pub total_duration_ms: u64,
}

/// 性能测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    /// 测试名称
    pub test_name: String,
    /// 平均延迟（毫秒）
    pub avg_latency_ms: f64,
    /// 最大延迟（毫秒）
    pub max_latency_ms: f64,
    /// 最小延迟（毫秒）
    pub min_latency_ms: f64,
    /// P99延迟（毫秒）
    pub p99_latency_ms: f64,
    /// 吞吐量（请求/秒）
    pub throughput_rps: f64,
    /// 总请求数
    pub total_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
}

/// 测试管理器
pub struct TestManager {
    /// 测试结果历史
    results: Arc<RwLock<HashMap<String, TestSuite>>>,
    /// 性能测试结果
    perf_results: Arc<RwLock<Vec<PerformanceTestResult>>>,
    /// 测试配置
    config: Arc<RwLock<TestConfig>>,
}

/// 测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// 并发用户数（性能测试）
    pub concurrent_users: u32,
    /// 测试持续时间（秒）
    pub test_duration_seconds: u64,
    /// 请求间隔（毫秒）
    pub request_interval_ms: u64,
    /// 是否启用详细日志
    pub verbose_logging: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            concurrent_users: 100,
            test_duration_seconds: 60,
            request_interval_ms: 10,
            verbose_logging: false,
        }
    }
}

impl TestManager {
    /// 创建新的测试管理器
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(HashMap::new())),
            perf_results: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(RwLock::new(TestConfig::default())),
        }
    }

    /// 使用自定义配置创建测试管理器
    pub fn with_config(config: TestConfig) -> Self {
        Self {
            results: Arc::new(RwLock::new(HashMap::new())),
            perf_results: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// 运行单元测试
    pub async fn run_unit_tests(&self, test_name: &str, test_fn: impl FnOnce() -> Result<(), String>) -> TestResult {
        info!("Running unit test: {}", test_name);
        
        let start = Instant::now();
        let result = test_fn();
        let duration_ms = start.elapsed().as_millis() as u64;
        
        let test_result = TestResult {
            test_name: test_name.to_string(),
            passed: result.is_ok(),
            duration_ms,
            error_message: result.err(),
            test_time: chrono::Utc::now().to_rfc3339(),
        };
        
        self.save_test_result("unit_tests", &test_result).await;
        test_result
    }

    /// 运行集成测试
    pub async fn run_integration_test(&self, test_name: &str, test_fn: impl FnOnce() -> Result<(), String>) -> TestResult {
        info!("Running integration test: {}", test_name);
        
        let start = Instant::now();
        let result = test_fn();
        let duration_ms = start.elapsed().as_millis() as u64;
        
        let test_result = TestResult {
            test_name: test_name.to_string(),
            passed: result.is_ok(),
            duration_ms,
            error_message: result.err(),
            test_time: chrono::Utc::now().to_rfc3339(),
        };
        
        self.save_test_result("integration_tests", &test_result).await;
        test_result
    }

    /// 运行性能测试
    pub async fn run_performance_test(
        &self,
        test_name: &str,
        request_fn: impl Fn() -> Result<(), String> + Send + Sync + 'static,
    ) -> PerformanceTestResult {
        info!("Running performance test: {}", test_name);
        
        let config = self.config.read().await;
        let concurrent_users = config.concurrent_users;
        let test_duration = Duration::from_secs(config.test_duration_seconds);
        
        let mut latencies = Vec::new();
        let mut failed_requests = 0;
        let mut total_requests = 0;
        
        let start = Instant::now();
        let mut handles = Vec::new();
        
        // 创建并发用户
        for _ in 0..concurrent_users {
            let request_fn_clone = &request_fn;
            let handle = tokio::spawn(async move {
                let mut user_latencies = Vec::new();
                let mut user_failed = 0;
                let mut user_total = 0;
                
                loop {
                    let req_start = Instant::now();
                    let result = request_fn_clone();
                    let req_duration = req_start.elapsed().as_micros() as f64 / 1000.0; // 转换为毫秒
                    
                    user_latencies.push(req_duration);
                    user_total += 1;
                    
                    if result.is_err() {
                        user_failed += 1;
                    }
                    
                    // 检查是否超过测试时间
                    if req_start.elapsed() > test_duration {
                        break;
                    }
                }
                
                (user_latencies, user_failed, user_total)
            });
            
            handles.push(handle);
        }
        
        // 等待所有用户完成
        for handle in handles {
            if let Ok((latencies, failed, total)) = handle.await {
                latencies.extend(latencies);
                failed_requests += failed;
                total_requests += total;
            }
        }
        
        let total_duration = start.elapsed().as_secs_f64();
        
        // 计算统计数据
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let avg_latency = if !latencies.is_empty() {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        } else {
            0.0
        };
        
        let max_latency = latencies.iter().cloned().fold(0.0_f64, f64::max);
        let min_latency = latencies.iter().cloned().fold(f64::MAX, f64::min);
        
        let p99_index = (latencies.len() as f64 * 0.99) as usize;
        let p99_latency = if p99_index < latencies.len() {
            latencies[p99_index]
        } else {
            max_latency
        };
        
        let throughput = total_requests as f64 / total_duration;
        
        let perf_result = PerformanceTestResult {
            test_name: test_name.to_string(),
            avg_latency_ms: avg_latency,
            max_latency_ms: max_latency,
            min_latency_ms: if min_latency == f64::MAX { 0.0 } else { min_latency },
            p99_latency_ms: p99_latency,
            throughput_rps: throughput,
            total_requests,
            failed_requests,
        };
        
        self.save_performance_result(&perf_result).await;
        perf_result
    }

    /// 运行异常测试
    pub async fn run_exception_test(&self, test_name: &str, test_fn: impl FnOnce() -> Result<(), String>) -> TestResult {
        info!("Running exception test: {}", test_name);
        
        let start = Instant::now();
        let result = test_fn();
        let duration_ms = start.elapsed().as_millis() as u64;
        
        let test_result = TestResult {
            test_name: test_name.to_string(),
            passed: result.is_ok(),
            duration_ms,
            error_message: result.err(),
            test_time: chrono::Utc::now().to_rfc3339(),
        };
        
        self.save_test_result("exception_tests", &test_result).await;
        test_result
    }

    /// 保存测试结果
    async fn save_test_result(&self, suite_name: &str, result: &TestResult) {
        let mut results = self.results.write().await;
        let suite = results
            .entry(suite_name.to_string())
            .or_insert_with(|| TestSuite {
                suite_name: suite_name.to_string(),
                results: Vec::new(),
                total_duration_ms: 0,
            });
        
        suite.results.push(result.clone());
        suite.total_duration_ms += result.duration_ms;
        
        debug!("Saved test result for suite: {}", suite_name);
    }

    /// 保存性能测试结果
    async fn save_performance_result(&self, result: &PerformanceTestResult) {
        let mut perf_results = self.perf_results.write().await;
        perf_results.push(result.clone());
        
        debug!("Saved performance test result: {}", result.test_name);
    }

    /// 获取测试套件结果
    pub async fn get_suite_results(&self, suite_name: &str) -> Option<TestSuite> {
        let results = self.results.read().await;
        results.get(suite_name).cloned()
    }

    /// 获取所有性能测试结果
    pub async fn get_performance_results(&self) -> Vec<PerformanceTestResult> {
        let perf_results = self.perf_results.read().await;
        perf_results.clone()
    }

    /// 生成测试报告
    pub async fn generate_test_report(&self) -> String {
        let results = self.results.read().await;
        let perf_results = self.perf_results.read().await;
        
        let mut report = String::from("# CarpTMS 自动化测试报告\n\n");
        
        report.push_str(&format!("生成时间: {}\n\n", chrono::Utc::now().to_rfc3339()));
        
        // 单元测试报告
        if let Some(suite) = results.get("unit_tests") {
            report.push_str("## 单元测试结果\n");
            let passed = suite.results.iter().filter(|r| r.passed).count();
            let total = suite.results.len();
            report.push_str(&format!(
                "通过率: {}/{} ({:.2}%)\n\n",
                passed,
                total,
                if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 }
            ));
        }
        
        // 集成测试报告
        if let Some(suite) = results.get("integration_tests") {
            report.push_str("## 集成测试结果\n");
            let passed = suite.results.iter().filter(|r| r.passed).count();
            let total = suite.results.len();
            report.push_str(&format!(
                "通过率: {}/{} ({:.2}%)\n\n",
                passed,
                total,
                if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 }
            ));
        }
        
        // 性能测试报告
        if !perf_results.is_empty() {
            report.push_str("## 性能测试结果\n");
            for result in perf_results.iter() {
                report.push_str(&format!(
                    "- {}: 平均延迟 {:.2}ms, P99延迟 {:.2}ms, 吞吐量 {:.2} req/s\n",
                    result.test_name,
                    result.avg_latency_ms,
                    result.p99_latency_ms,
                    result.throughput_rps
                ));
            }
            report.push('\n');
        }
        
        report
    }
}

/// 创建测试管理器实例
pub fn create_test_manager() -> TestManager {
    TestManager::new()
}

/// 创建带自定义配置的测试管理器
pub fn create_test_manager_with_config(config: TestConfig) -> TestManager {
    TestManager::with_config(config)
}
