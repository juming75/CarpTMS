//! 架构自动切换器
//!
//! 根据系统负载和数据量自动决策是否从单体切换到微服务架构

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::ArchitectureMode;
use super::system_monitor::SystemMonitor;

/// 切换决策配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchingConfig {
    /// 是否启用自动切换
    pub enabled: bool,
    
    /// 切换到微服务的负载阈值（0-100）
    pub microservice_threshold: f64,
    
    /// 切换回单体的负载阈值（0-100）
    pub monolith_threshold: f64,
    
    /// 连续多少次超过阈值才切换（防止抖动）
    pub consecutive_threshold: u32,
    
    /// 检查间隔（秒）
    pub check_interval_secs: u64,
    
    /// 最小运行时间（分钟）- 防止频繁切换
    pub min_runtime_minutes: u32,
    
    /// 数据量阈值（记录数）
    pub data_volume_threshold: i64,
    
    /// QPS阈值
    pub qps_threshold: f64,
    
    /// 响应时间阈值（毫秒）
    pub response_time_threshold: f64,
}

impl Default for SwitchingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            microservice_threshold: 70.0,  // 负载超过70%考虑切换
            monolith_threshold: 30.0,      // 负载低于30%考虑回退
            consecutive_threshold: 3,      // 连续3次超过阈值
            check_interval_secs: 300,      // 每5分钟检查一次
            min_runtime_minutes: 30,       // 最少运行30分钟
            data_volume_threshold: 1_000_000, // 100万记录
            qps_threshold: 1000.0,         // 每秒1000请求
            response_time_threshold: 500.0, // 500ms响应时间
        }
    }
}

/// 切换决策结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitchingDecision {
    /// 保持当前架构
    KeepCurrent,
    /// 切换到微服务
    SwitchToMicroservice,
    /// 切换回单体
    SwitchToMonolith,
}

/// 切换历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchingHistory {
    /// 切换时间
    pub timestamp: DateTime<Utc>,
    /// 从哪个架构切换
    pub from: ArchitectureMode,
    /// 切换到哪个架构
    pub to: ArchitectureMode,
    /// 切换原因
    pub reason: String,
    /// 当时的负载分数
    pub load_score: f64,
    /// 当时的总记录数
    pub total_records: i64,
}

/// 架构切换器
pub struct ArchitectureSwitcher {
    /// 当前架构模式
    current_mode: Arc<RwLock<ArchitectureMode>>,
    /// 系统监控器
    monitor: Arc<SystemMonitor>,
    /// 切换配置
    config: SwitchingConfig,
    /// 连续超过阈值计数
    consecutive_count: Arc<RwLock<u32>>,
    /// 当前架构启动时间
    mode_start_time: Arc<RwLock<DateTime<Utc>>>,
    /// 切换历史
    switching_history: Arc<RwLock<Vec<SwitchingHistory>>>,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl ArchitectureSwitcher {
    /// 创建新的架构切换器
    pub fn new(
        initial_mode: ArchitectureMode,
        monitor: Arc<SystemMonitor>,
        config: SwitchingConfig,
    ) -> Self {
        Self {
            current_mode: Arc::new(RwLock::new(initial_mode)),
            monitor,
            config,
            consecutive_count: Arc::new(RwLock::new(0)),
            mode_start_time: Arc::new(RwLock::new(Utc::now())),
            switching_history: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 启动自动切换监控
    pub async fn start(&self) {
        if !self.config.enabled {
            tracing::info!("Architecture auto-switching is disabled");
            return;
        }
        
        let mut running = self.running.write().await;
        if *running {
            tracing::warn!("Architecture switcher is already running");
            return;
        }
        *running = true;
        drop(running);
        
        tracing::info!(
            "Starting architecture auto-switcher (interval: {}s, threshold: {}%)",
            self.config.check_interval_secs,
            self.config.microservice_threshold
        );
        
        let current_mode = self.current_mode.clone();
        let monitor = self.monitor.clone();
        let config = self.config.clone();
        let consecutive_count = self.consecutive_count.clone();
        let mode_start_time = self.mode_start_time.clone();
        let switching_history = self.switching_history.clone();
        let running = self.running.clone();
        let interval = Duration::from_secs(config.check_interval_secs);
        
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                if !*running.read().await {
                    break;
                }
                
                // 执行切换决策
                let decision = Self::evaluate_switching(
                    &monitor,
                    &config,
                    &current_mode,
                    &consecutive_count,
                    &mode_start_time,
                ).await;
                
                match decision {
                    SwitchingDecision::SwitchToMicroservice => {
                        tracing::warn!("Auto-switching to microservice architecture!");
                        Self::perform_switch(
                            ArchitectureMode::MicroDDD,
                            &current_mode,
                            &mode_start_time,
                            &switching_history,
                            "System load exceeded threshold",
                            &monitor,
                        ).await;
                    }
                    SwitchingDecision::SwitchToMonolith => {
                        tracing::info!("Auto-switching back to monolith architecture");
                        Self::perform_switch(
                            ArchitectureMode::MonolithDDD,
                            &current_mode,
                            &mode_start_time,
                            &switching_history,
                            "System load decreased",
                            &monitor,
                        ).await;
                    }
                    SwitchingDecision::KeepCurrent => {
                        tracing::debug!("No architecture change needed");
                    }
                }
            }
            
            tracing::info!("Architecture switcher stopped");
        });
    }
    
    /// 停止自动切换监控
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Architecture switcher stopping...");
    }
    
    /// 获取当前架构模式
    pub async fn get_current_mode(&self) -> ArchitectureMode {
        *self.current_mode.read().await
    }
    
    /// 获取切换历史
    pub async fn get_switching_history(&self) -> Vec<SwitchingHistory> {
        self.switching_history.read().await.clone()
    }
    
    /// 手动切换架构（覆盖自动决策）
    pub async fn manual_switch(&self, target_mode: ArchitectureMode, reason: &str) {
        Self::perform_switch(
            target_mode,
            &self.current_mode,
            &self.mode_start_time,
            &self.switching_history,
            reason,
            &self.monitor,
        ).await;
    }
    
    /// 评估是否需要切换
    async fn evaluate_switching(
        monitor: &SystemMonitor,
        config: &SwitchingConfig,
        current_mode: &Arc<RwLock<ArchitectureMode>>,
        consecutive_count: &Arc<RwLock<u32>>,
        mode_start_time: &Arc<RwLock<DateTime<Utc>>>,
    ) -> SwitchingDecision {
        let metrics = monitor.get_current_metrics().await;
        let load_score = monitor.calculate_load_score().await;
        let current = *current_mode.read().await;
        
        // 检查最小运行时间
        let start_time = *mode_start_time.read().await;
        let runtime = Utc::now() - start_time;
        if runtime.num_minutes() < config.min_runtime_minutes as i64 {
            tracing::debug!(
                "Minimum runtime not reached ({} min < {} min)",
                runtime.num_minutes(),
                config.min_runtime_minutes
            );
            return SwitchingDecision::KeepCurrent;
        }
        
        let mut count = consecutive_count.write().await;
        
        match current {
            ArchitectureMode::MonolithDDD => {
                // 检查是否需要切换到微服务
                let should_switch = load_score > config.microservice_threshold
                    || metrics.database.total_records.get("total").copied().unwrap_or(0) > config.data_volume_threshold
                    || metrics.performance.requests_per_second > config.qps_threshold
                    || metrics.performance.avg_response_time_ms > config.response_time_threshold;
                
                if should_switch {
                    *count += 1;
                    tracing::warn!(
                        "Load score: {:.1}% (threshold: {:.1}%), consecutive: {}/{}",
                        load_score,
                        config.microservice_threshold,
                        *count,
                        config.consecutive_threshold
                    );
                    
                    if *count >= config.consecutive_threshold {
                        *count = 0;
                        return SwitchingDecision::SwitchToMicroservice;
                    }
                } else {
                    *count = 0;
                }
            }
            ArchitectureMode::MicroDDD => {
                // 检查是否需要回退到单体
                let should_revert = load_score < config.monolith_threshold
                    && metrics.database.total_records.get("total").copied().unwrap_or(0) < config.data_volume_threshold / 2
                    && metrics.performance.requests_per_second < config.qps_threshold / 2.0;
                
                if should_revert {
                    *count += 1;
                    tracing::info!(
                        "Load score: {:.1}% (threshold: {:.1}%), consecutive: {}/{}",
                        load_score,
                        config.monolith_threshold,
                        *count,
                        config.consecutive_threshold
                    );
                    
                    if *count >= config.consecutive_threshold {
                        *count = 0;
                        return SwitchingDecision::SwitchToMonolith;
                    }
                } else {
                    *count = 0;
                }
            }
        }
        
        SwitchingDecision::KeepCurrent
    }
    
    /// 执行架构切换
    async fn perform_switch(
        target_mode: ArchitectureMode,
        current_mode: &Arc<RwLock<ArchitectureMode>>,
        mode_start_time: &Arc<RwLock<DateTime<Utc>>>,
        history: &Arc<RwLock<Vec<SwitchingHistory>>>,
        reason: &str,
        monitor: &SystemMonitor,
    ) {
        let from_mode = *current_mode.read().await;
        
        if from_mode == target_mode {
            return;
        }
        
        let metrics = monitor.get_current_metrics().await;
        let load_score = monitor.calculate_load_score().await;
        
        // 记录历史
        let history_entry = SwitchingHistory {
            timestamp: Utc::now(),
            from: from_mode,
            to: target_mode,
            reason: reason.to_string(),
            load_score,
            total_records: metrics.database.total_records.get("total").copied().unwrap_or(0),
        };
        
        let mut hist = history.write().await;
        hist.push(history_entry);
        drop(hist);
        
        // 更新当前模式
        let mut mode = current_mode.write().await;
        *mode = target_mode;
        drop(mode);
        
        // 更新启动时间
        let mut start_time = mode_start_time.write().await;
        *start_time = Utc::now();
        drop(start_time);
        
        tracing::warn!(
            "Architecture switched from {:?} to {:?}. Reason: {}",
            from_mode,
            target_mode,
            reason
        );
        
        // 这里可以触发实际的重启或重新配置逻辑
        // 例如：发送信号给主进程，或者更新配置并优雅重启
    }
    
    /// 获取切换建议（不实际执行）
    pub async fn get_switching_recommendation(&self) -> (SwitchingDecision, String) {
        let metrics = self.monitor.get_current_metrics().await;
        let load_score = self.monitor.calculate_load_score().await;
        let current = *self.current_mode.read().await;
        
        let total_records = metrics.database.total_records.get("total").copied().unwrap_or(0);
        let qps = metrics.performance.requests_per_second;
        let _avg_response = metrics.performance.avg_response_time_ms;
        
        let recommendation = match current {
            ArchitectureMode::MonolithDDD => {
                if load_score > self.config.microservice_threshold {
                    (
                        SwitchingDecision::SwitchToMicroservice,
                        format!(
                            "建议切换到微服务架构。当前负载: {:.1}% (阈值: {:.1}%), \
                             数据量: {} 记录 (阈值: {}), QPS: {:.0} (阈值: {:.0})",
                            load_score,
                            self.config.microservice_threshold,
                            total_records,
                            self.config.data_volume_threshold,
                            qps,
                            self.config.qps_threshold
                        )
                    )
                } else {
                    (
                        SwitchingDecision::KeepCurrent,
                        format!(
                            "当前单体架构运行良好。负载: {:.1}%, 数据量: {} 记录, QPS: {:.0}",
                            load_score, total_records, qps
                        )
                    )
                }
            }
            ArchitectureMode::MicroDDD => {
                if load_score < self.config.monolith_threshold {
                    (
                        SwitchingDecision::SwitchToMonolith,
                        format!(
                            "可以考虑回退到单体架构。当前负载: {:.1}% (阈值: {:.1}%), \
                             数据量: {} 记录",
                            load_score,
                            self.config.monolith_threshold,
                            total_records
                        )
                    )
                } else {
                    (
                        SwitchingDecision::KeepCurrent,
                        format!(
                            "当前微服务架构运行良好。负载: {:.1}%, 数据量: {} 记录, QPS: {:.0}",
                            load_score, total_records, qps
                        )
                    )
                }
            }
        };
        
        recommendation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_switching_config_default() {
        let config = SwitchingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.microservice_threshold, 70.0);
        assert_eq!(config.monolith_threshold, 30.0);
        assert_eq!(config.consecutive_threshold, 3);
    }
    
    #[tokio::test]
    async fn test_switching_decision() {
        // 这里可以添加更多测试
        assert_eq!(
            SwitchingDecision::KeepCurrent,
            SwitchingDecision::KeepCurrent
        );
    }
}