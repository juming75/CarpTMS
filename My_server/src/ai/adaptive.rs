//! 自适应 AI 管理器
//!
//! 根据系统资源自动选择最佳 AI 后端：
//! - Candle（纯 Rust，默认）: 适用于 ai feature，适配所有平台
//! - Ollama（HTTP）: 作为备选方案
//! - 规则引擎：资源不足时静默降级
//!
//! 工作流程：
//! 1. 启动时检测系统资源
//! 2. 根据能力等级自动配置后端
//! 3. 运行时监控资源，自动降级/恢复
//! 4. 对外提供统一的 AI 接口

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[cfg(feature = "ai")]
use super::candle_backend::CandleBackend;
use super::resource::{AiModuleConfig, CapabilityLevel, ResourceDetector, SystemResources};

/// AI 后端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiBackendType {
    /// Candle 纯 Rust 推理
    Candle,
    /// Ollama HTTP API
    Ollama,
    /// 规则引擎（无 AI）
    RuleEngine,
    /// 已禁用
    Disabled,
}

impl std::fmt::Display for AiBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiBackendType::Candle => write!(f, "candle"),
            AiBackendType::Ollama => write!(f, "ollama"),
            AiBackendType::RuleEngine => write!(f, "rule-engine"),
            AiBackendType::Disabled => write!(f, "disabled"),
        }
    }
}

/// AI 服务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiServiceStatus {
    /// 当前后端类型
    pub backend: AiBackendType,
    /// 后端是否可用
    pub available: bool,
    /// 模型是否已加载
    pub model_loaded: bool,
    /// AI 模块是否启用
    pub enabled: bool,
    /// 能力等级
    pub capability_level: CapabilityLevel,
    /// 推荐模型
    pub recommended_model: String,
    /// 系统资源配置
    pub resources: SystemResources,
    /// AI 模块配置
    pub config: AiModuleConfig,
    /// 最后检测时间
    pub last_check: String,
    /// 运行时间 (秒)
    pub uptime_seconds: u64,
}

/// 统一推理响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedInferenceResponse {
    /// 生成的文本
    pub content: String,
    /// 消耗的 token 数
    pub tokens_used: usize,
    /// 推理耗时 (ms)
    pub inference_time_ms: u64,
    /// 完成原因
    pub finish_reason: String,
    /// 使用的后端
    pub backend: AiBackendType,
}

/// 自适应 AI 管理器
#[allow(dead_code)]
pub struct AdaptiveAiManager {
    /// 当前激活的后端类型
    active_backend: RwLock<AiBackendType>,
    /// Candle 后端（ai feature 启用时使用）
    #[cfg(feature = "ai")]
    candle_backend: Option<CandleBackend>,
    /// 系统资源配置
    resources: SystemResources,
    /// AI 模块配置
    config: AiModuleConfig,
    /// 启动时间
    start_time: Instant,
    /// 资源检测间隔
    resource_check_interval: Duration,
    /// 最后资源检测时间
    last_resource_check: RwLock<Instant>,
}

impl AdaptiveAiManager {
    /// 创建新的自适应 AI 管理器
    pub fn new() -> Self {
        let resources = ResourceDetector::detect();
        let config = AiModuleConfig::from(resources.clone());

        let backend = Self::select_backend(&resources, &config);

        log::info!("=== AI 模块初始化 ===");
        log::info!(
            "系统能力等级: {} ({})",
            config.capability_level,
            ResourceDetector::recommended_backend()
        );
        log::info!(
            "推荐模型: {} (量化: {})",
            config.recommended_model,
            config.recommended_quantization
        );
        log::info!("选定的后端: {}", backend);
        log::info!(
            "AI 模块状态: {}",
            if config.enabled {
                "启用"
            } else {
                "禁用（资源不足）"
            }
        );

        Self {
            active_backend: RwLock::new(backend),
            #[cfg(feature = "ai")]
            candle_backend: None,
            resources,
            config,
            start_time: Instant::now(),
            resource_check_interval: Duration::from_secs(300), // 5 分钟检测一次
            last_resource_check: RwLock::new(Instant::now()),
        }
    }

    /// 根据资源选择最佳后端
    #[allow(unreachable_code)]
    fn select_backend(_resources: &SystemResources, config: &AiModuleConfig) -> AiBackendType {
        if !config.enabled {
            return AiBackendType::Disabled;
        }

        // ai feature 启用时优先使用 Candle
        #[cfg(feature = "ai")]
        {
            log::info!("Candle 后端可用");
            return AiBackendType::Candle;
        }

        // 无 ai 时检查 Ollama (这段代码在 ai 下不会执行)
        if let Ok(ollama) = std::env::var("OLLAMA_BASE_URL") {
            if !ollama.is_empty() {
                return AiBackendType::Ollama;
            }
        }

        AiBackendType::RuleEngine
    }

    /// 初始化后端
    pub async fn initialize(&mut self) -> Result<(), String> {
        let backend = *self.active_backend.read().await;

        match backend {
            AiBackendType::Candle => {
                #[cfg(feature = "ai")]
                {
                    let candle = CandleBackend::new();
                    if candle.is_available() {
                        self.candle_backend = Some(candle);
                        log::info!("Candle 后端初始化成功（资源检测通过）");

                        // 尝试自动加载模型（静默失败）
                        #[cfg(feature = "ai")]
                        {
                            match super::candle_backend::get_recommended_model_path() {
                                Ok((ref model_path, ref _kind)) => {
                                    if let Some(ref mut c) = self.candle_backend {
                                        if let Err(e) = c.load_gguf(model_path) {
                                            log::warn!("自动加载模型失败（可稍后手动加载）: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::warn!("未找到推荐模型: {}\n提示：设置 CARTPMS_MODEL_PATH 环境变量", e);
                                }
                            }
                        }
                        return Ok(());
                    }
                }
                log::warn!("Candle 不可用，降级到 Ollama");
                *self.active_backend.write().await = AiBackendType::Ollama;
            }
            AiBackendType::Ollama => {
                // 检查 Ollama 可用性
                if std::env::var("OLLAMA_BASE_URL").is_ok() {
                    log::info!("Ollama 后端已配置（如需使用请确保 Ollama 正在运行）");
                    return Ok(());
                }
                log::warn!("Ollama 未配置，降级到规则引擎");
                *self.active_backend.write().await = AiBackendType::RuleEngine;
            }
            AiBackendType::RuleEngine => {
                log::info!("使用规则引擎（无 AI 推理）");
            }
            AiBackendType::Disabled => {
                log::info!("AI 模块已禁用（系统资源不足）");
            }
        }

        Ok(())
    }

    /// 获取服务状态
    pub async fn get_status(&self) -> AiServiceStatus {
        let backends = *self.active_backend.read().await;
        #[cfg(feature = "ai")]
        let model_loaded = self
            .candle_backend
            .as_ref()
            .map(|b| b.is_model_loaded())
            .unwrap_or(false);
        #[cfg(not(feature = "ai"))]
        let model_loaded = false;

        AiServiceStatus {
            backend: backends,
            available: self.is_available().await,
            model_loaded,
            enabled: self.config.enabled,
            capability_level: self.config.capability_level,
            recommended_model: self.config.recommended_model.clone(),
            resources: self.resources.clone(),
            config: self.config.clone(),
            last_check: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    /// 检查 AI 服务是否可用
    pub async fn is_available(&self) -> bool {
        let backend = *self.active_backend.read().await;
        match backend {
            AiBackendType::Candle => self.config.enabled,
            AiBackendType::Ollama => std::env::var("OLLAMA_BASE_URL").is_ok(),
            AiBackendType::RuleEngine => true,
            AiBackendType::Disabled => false,
        }
    }

    /// 执行统一推理
    pub async fn inference(&self, prompt: &str) -> Result<UnifiedInferenceResponse, String> {
        let backend = *self.active_backend.read().await;

        match backend {
            #[cfg(feature = "ai")]
            AiBackendType::Candle => {
                if let Some(ref candle) = self.candle_backend {
                    let result = candle.inference(prompt).await.map_err(|e| e.to_string())?;
                    return Ok(UnifiedInferenceResponse {
                        content: result.content,
                        tokens_used: result.tokens_used,
                        inference_time_ms: result.inference_time_ms,
                        finish_reason: result.finish_reason,
                        backend,
                    });
                }
                Err("Candle 模型未加载".to_string())
            }
            AiBackendType::Ollama => {
                // 使用 reqwest 直接调用 Ollama API
                let base_url = std::env::var("OLLAMA_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".to_string());
                let model =
                    std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5:7b".to_string());

                let client = reqwest::Client::new();
                let start = Instant::now();

                let resp = client
                    .post(format!("{}/api/generate", base_url))
                    .json(&serde_json::json!({
                        "model": model,
                        "prompt": format!(
                            "<|im_start|>system\n你是专业车联网 AI 助手。<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
                            prompt
                        ),
                        "stream": false,
                    }))
                    .send()
                    .await
                    .map_err(|e| format!("Ollama 请求失败: {}", e))?;

                let body: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("解析 Ollama 响应失败: {}", e))?;

                let content = body["response"].as_str().unwrap_or("").trim().to_string();
                let tokens = body["eval_count"].as_u64().unwrap_or(0) as usize;

                Ok(UnifiedInferenceResponse {
                    content,
                    tokens_used: tokens,
                    inference_time_ms: start.elapsed().as_millis() as u64,
                    finish_reason: "stop".to_string(),
                    backend,
                })
            }
            AiBackendType::RuleEngine => Ok(UnifiedInferenceResponse {
                content: format!(
                    "【规则引擎响应】当前 AI 模块因资源限制已降级为规则引擎模式。\n\
                        系统能力等级: {}\n\
                        建议: {}\n\n\
                        如需启用完整 AI 功能:\n\
                        1. 使用 `cargo build --features ai` 编译以启用 Candle 本地推理\n\
                        2. 或配置 Ollama 服务（设置 OLLAMA_BASE_URL 环境变量）",
                    self.config.capability_level, self.config.recommended_model
                ),
                tokens_used: 0,
                inference_time_ms: 0,
                finish_reason: "rule-engine".to_string(),
                backend,
            }),
            AiBackendType::Disabled => Err("AI 模块已禁用（系统资源不足，已静默降级）".to_string()),
        }
    }

    /// 检查并更新资源状态
    #[allow(dead_code)]
    async fn check_and_update_resources(&mut self) {
        let last_check = *self.last_resource_check.read().await;
        if last_check.elapsed() < self.resource_check_interval {
            return;
        }

        let new_resources = ResourceDetector::detect();
        let new_config = AiModuleConfig::from(new_resources.clone());

        if new_config.capability_level != self.config.capability_level {
            log::info!(
                "资源检测: 能力等级从 {} 变化为 {}",
                self.config.capability_level,
                new_config.capability_level
            );

            if !self.config.enabled && new_config.enabled {
                log::info!("检测到可用资源，尝试启用 AI 模块...");
            }
        }

        *self.last_resource_check.write().await = Instant::now();
        self.resources = new_resources;
        self.config = new_config;
    }

    /// 加载模型
    #[cfg(feature = "ai")]
    pub async fn load_model(&mut self, model_path: Option<String>) -> Result<(), String> {
        let backend = *self.active_backend.read().await;

        match backend {
            AiBackendType::Candle => {
                let path = model_path
                    .unwrap_or_else(|| std::env::var("CARTPMS_MODEL_DIR").unwrap_or_default());

                if path.is_empty() {
                    return Err("请设置 CARTPMS_MODEL_DIR 或 CARTPMS_GGUF_PATH".to_string());
                }

                let bp = std::path::PathBuf::from(&path);
                if let Some(ref mut candle) = self.candle_backend {
                    candle.load_from_dir(&bp).map_err(|e| e.to_string())?;
                } else {
                    let mut nc = super::candle_backend::CandleBackend::new();
                    nc.load_from_dir(&bp).map_err(|e| e.to_string())?;
                    self.candle_backend = Some(nc);
                }
                Ok(())
            }
            _ => Err("当前后端不支持手动加载".to_string()),
        }
    }

    /// 卸载模型
    pub async fn unload_model(&mut self) {
        #[cfg(feature = "ai")]
        {
            if let Some(ref mut candle) = self.candle_backend {
                candle.unload_model().await;
            }
        }
    }

    /// 获取当前后端类型
    pub async fn current_backend(&self) -> AiBackendType {
        *self.active_backend.read().await
    }
}

impl Default for AdaptiveAiManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adaptive_manager() {
        let manager = AdaptiveAiManager::new();
        let status = manager.get_status().await;

        tracing::info!(backend = %status.backend, enabled = status.enabled, capability = %status.capability_level, model = %status.recommended_model, available = status.resources.local_ai_supported, "AI Service Status");
    }

    #[tokio::test]
    async fn test_inference_fallback() {
        let manager = AdaptiveAiManager::new();
        let result = manager.inference("测试提示").await;
        tracing::info!(result = ?result, "Inference result");
    }
}
