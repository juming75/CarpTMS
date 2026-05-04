//! Candle 本地推理后端
//!
//! 使用 HuggingFace Candle（纯 Rust ML 框架）加载 Qwen2 模型
//!
//! ## 核心优势
//! - **纯 Rust**：无需 C++ 编译链, 无 libclang, 无 bindgen
//! - **跨平台**：ARM64/x86_64/龙芯/申威, 国产信创开箱即用
//! - **HuggingFace 亲儿子**：HuggingFace 官方维护, 社区活跃
//!
//! ## 模型加载方式
//!
//! ### 1. SafeTensors 模式（推荐，全平台）
//! 通过 hf-hub 自动下载或本地加载 safetensors 模型。
//! ```bash
//! export CARTPMS_MODEL_DIR=/data/models/Qwen2.5-1.5B-Instruct
//! ```
//!
//! ### 2. GGUF 模式（实验性，需 Candle >= 0.10）
//! 通过 candle_core::quantized 加载 GGUF。
//! ```bash
//! export CARTPMS_MODEL_PATH=/data/models/qwen2.5-1.5b-instruct-q8_0.gguf
//! ```

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::resource::{AiModuleConfig, CapabilityLevel, ResourceDetector};

// ==================== Candle 条件编译导入 ====================

#[cfg(feature = "ai")]
use candle_core::{DType, Device, Tensor};
#[cfg(feature = "ai")]
use candle_nn::VarBuilder;
#[cfg(feature = "ai")]
use candle_transformers::generation::LogitsProcessor;
#[cfg(feature = "ai")]
use candle_transformers::models::qwen2::{Config as QwenConfig, Model as QwenModel};
#[cfg(feature = "ai")]
use hf_hub::api::sync::Api;
#[cfg(feature = "ai")]
use tokenizers::Tokenizer;

// ==================== 错误类型 ====================

/// Candle 后端错误
#[derive(Debug, thiserror::Error)]
pub enum CandleError {
    #[error("模型加载失败: {0}")]
    ModelLoadError(String),
    #[error("推理失败: {0}")]
    InferenceError(String),
    #[error("模型未加载")]
    ModelNotLoaded,
    #[error("配置错误: {0}")]
    ConfigError(String),
    #[error("标记化失败: {0}")]
    TokenizationError(String),
    #[error("反标记化失败: {0}")]
    DetokenizationError(String),
    #[error("资源不足: {0}")]
    InsufficientResources(String),
}

// ==================== 后端状态 ====================

/// Candle 后端状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendStatus {
    /// 未初始化
    Uninitialized,
    /// 资源不足，已静默禁用
    Disabled,
    /// 已初始化但未加载模型
    Ready,
    /// 模型已加载
    Loaded,
    /// 加载失败
    Failed,
}

impl std::fmt::Display for BackendStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendStatus::Uninitialized => write!(f, "未初始化"),
            BackendStatus::Disabled => write!(f, "已禁用（资源不足）"),
            BackendStatus::Ready => write!(f, "就绪"),
            BackendStatus::Loaded => write!(f, "已加载"),
            BackendStatus::Failed => write!(f, "加载失败"),
        }
    }
}

// ==================== 类型定义 ====================

/// 推理参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParams {
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_p: f64,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
    pub seed: Option<u64>,
}

impl Default for InferenceParams {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            seed: None,
        }
    }
}

/// 推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub content: String,
    pub tokens_used: usize,
    pub inference_time_ms: u64,
    pub tokens_per_second: f64,
    pub finish_reason: String,
}

// ==================== Candle 后端 ====================

/// Candle 本地推理后端
///
/// 所有 [`#[cfg(feature = "ai")]`] 方法在 feature 关闭时均返回错误/空操作。
pub struct CandleBackend {
    /// 当前使用的设备（CPU/CUDA/Metal）
    #[cfg(feature = "ai")]
    device: Device,
    /// Qwen2 模型（Mutex 确保 forward(&mut self) 可用）
    #[cfg(feature = "ai")]
    model: Mutex<Option<QwenModel>>,
    /// HuggingFace tokenizer
    #[cfg(feature = "ai")]
    tokenizer: Mutex<Option<Tokenizer>>,
    /// 后端状态
    status: BackendStatus,
    /// 启动时间
    #[allow(dead_code)]
    start_time: Instant,
}

impl CandleBackend {
    /// 创建新的 Candle 后端
    ///
    /// 自动检测系统资源，资源不足时静默设置为 Disabled。
    pub fn new() -> Self {
        let resources = ResourceDetector::detect();
        let config = AiModuleConfig::from(resources);

        let status = if config.enabled {
            log::info!("Candle 后端初始化（能力等级: {}）", config.capability_level);
            log::info!(
                "推荐模型: {} (量化: {})",
                config.recommended_model,
                config.recommended_quantization
            );
            BackendStatus::Ready
        } else {
            log::info!(
                "Candle 后端已静默禁用（系统资源: {}）",
                config.capability_level
            );
            BackendStatus::Disabled
        };

        #[cfg(feature = "ai")]
        {
            let device = Self::select_device();
            Self {
                device,
                model: Mutex::new(None),
                tokenizer: Mutex::new(None),
                status,
                start_time: Instant::now(),
            }
        }
        #[cfg(not(feature = "ai"))]
        {
            Self {
                status,
                start_time: Instant::now(),
            }
        }
    }

    /// 选择最优计算设备
    #[cfg(feature = "ai")]
    fn select_device() -> Device {
        if let Ok(device) = Device::cuda_if_available(0) {
            if device.is_cuda() {
                log::info!("Candle 使用 CUDA GPU 加速");
                return device;
            }
        }
        #[cfg(target_os = "macos")]
        if let Ok(device) = Device::metal_if_available(0) {
            if device.is_metal() {
                log::info!("Candle 使用 Metal GPU 加速");
                return device;
            }
        }
        log::info!("Candle 使用 CPU 推理");
        Device::Cpu
    }

    // ==================== 公开接口 ====================

    /// 获取后端状态
    pub fn status(&self) -> BackendStatus {
        self.status
    }

    /// 后端是否可用（Ready 或 Loaded）
    pub fn is_available(&self) -> bool {
        matches!(self.status, BackendStatus::Ready | BackendStatus::Loaded)
    }

    /// 模型是否已加载
    pub fn is_model_loaded(&self) -> bool {
        self.status == BackendStatus::Loaded
    }

    // ==================== 模型加载 ====================

    /// 从 HuggingFace Hub 自动下载并加载 safetensors 模型
    #[cfg(feature = "ai")]
    pub fn load_from_hub(&mut self, model_id: &str) -> Result<(), CandleError> {
        log::info!("正在加载模型 (hf): {} ...", model_id);
        let load_start = Instant::now();

        let api = Api::new().map_err(|e| CandleError::ModelLoadError(e.to_string()))?;
        let repo = api.model(model_id.to_string());

        // 下载并解析 config.json
        let config_path = repo
            .get("config.json")
            .map_err(|e| CandleError::ModelLoadError(format!("config.json 下载失败: {}", e)))?;
        let qwen_config: QwenConfig = serde_json::from_slice(
            &std::fs::read(&config_path).map_err(|e| CandleError::ModelLoadError(e.to_string()))?,
        )
        .map_err(|e| CandleError::ModelLoadError(format!("config.json 解析失败: {}", e)))?;

        // 下载 model.safetensors
        let weights_path = repo
            .get("model.safetensors")
            .map_err(|e| CandleError::ModelLoadError(format!("weights 下载失败: {}", e)))?;
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &self.device)
        }
        .map_err(|e| CandleError::ModelLoadError(e.to_string()))?;

        // 创建 Qwen2 模型
        let model = QwenModel::new(&qwen_config, vb)
            .map_err(|e| CandleError::ModelLoadError(e.to_string()))?;

        // 下载 tokenizer
        let tokenizer_path = repo
            .get("tokenizer.json")
            .map_err(|e| CandleError::ModelLoadError(format!("tokenizer 下载失败: {}", e)))?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| CandleError::ModelLoadError(e.to_string()))?;

        *self
            .model
            .lock()
            .map_err(|_| CandleError::InferenceError("模型锁被污染".to_string()))? = Some(model);
        *self
            .tokenizer
            .lock()
            .map_err(|_| CandleError::InferenceError("分词器锁被污染".to_string()))? =
            Some(tokenizer);
        self.status = BackendStatus::Loaded;

        log::info!(
            "模型加载完成，耗时: {:.2}s",
            load_start.elapsed().as_secs_f32()
        );
        Ok(())
    }

    /// 从本地目录加载模型（safetensors）
    #[cfg(feature = "ai")]
    pub fn load_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), CandleError> {
        let dir = dir.as_ref();
        let _load_start = Instant::now();

        let config_path = dir.join("config.json");
        let weights_path = dir.join("model.safetensors");
        let tokenizer_path = dir.join("tokenizer.json");

        if !config_path.exists() || !weights_path.exists() {
            return Err(CandleError::ModelLoadError(format!(
                "模型目录不完整，缺少 config.json 或 model.safetensors: {}",
                dir.display()
            )));
        }

        log::info!("正在加载本地模型: {} ...", dir.display());

        let qwen_config: QwenConfig = serde_json::from_slice(
            &std::fs::read(&config_path).map_err(|e| CandleError::ModelLoadError(e.to_string()))?,
        )
        .map_err(|e| CandleError::ModelLoadError(e.to_string()))?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &self.device)
        }
        .map_err(|e| CandleError::ModelLoadError(e.to_string()))?;

        let model = QwenModel::new(&qwen_config, vb)
            .map_err(|e| CandleError::ModelLoadError(e.to_string()))?;

        let tokenizer = if tokenizer_path.exists() {
            Some(
                Tokenizer::from_file(&tokenizer_path)
                    .map_err(|e| CandleError::ModelLoadError(e.to_string()))?,
            )
        } else {
            log::warn!("tokenizer.json 不存在，将使用简单解码");
            None
        };

        *self
            .model
            .lock()
            .map_err(|_| CandleError::InferenceError("模型锁被污染".to_string()))? = Some(model);
        *self
            .tokenizer
            .lock()
            .map_err(|_| CandleError::InferenceError("分词器锁被污染".to_string()))? = tokenizer;
        self.status = BackendStatus::Loaded;
        Ok(())
    }

    /// 从 GGUF 文件加载（实验性，需要 Candle >= 0.10 的 quantized 支持）
    #[cfg(feature = "ai")]
    pub fn load_gguf<P: AsRef<Path>>(&mut self, _path: P) -> Result<(), CandleError> {
        // GGUF 加载目前是一个占位符号。
        // Qwen2 在 candle-transformers 中的 GGUF/quantized 支持取决于版本。
        // 如果当前 Candle 版本支持，后续可在此实现。
        Err(CandleError::ConfigError(
            "GGUF 加载为实验性功能。请使用 SafeTensors 格式代替。\n\
             推荐: 设置 CARTPMS_MODEL_DIR 指向包含 model.safetensors 的目录\n\
             例: export CARTPMS_MODEL_DIR=/data/models/Qwen2.5-1.5B-Instruct"
                .to_string(),
        ))
    }

    // ==================== 推理 ====================

    /// 执行推理
    #[cfg(feature = "ai")]
    pub async fn inference(&self, prompt: &str) -> Result<InferenceResult, CandleError> {
        let mut model = self
            .model
            .lock()
            .map_err(|_| CandleError::InferenceError("模型锁被污染".to_string()))?;
        let tokenizer_lock = self
            .tokenizer
            .lock()
            .map_err(|_| CandleError::InferenceError("分词器锁被污染".to_string()))?;
        let model = model.as_mut().ok_or(CandleError::ModelNotLoaded)?;
        let tokenizer = tokenizer_lock.as_ref().ok_or(CandleError::ModelNotLoaded)?;

        let start = Instant::now();

        // Qwen 聊天格式
        let full_prompt = format!(
            "<|im_start|>system\n你是 CarpTMS 车联网运输管理系统的 AI 助手，\
             请基于给定数据生成专业分析结果。<|im_end|>\n\
             <|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            prompt
        );

        // Tokenize
        let encoding = tokenizer
            .encode(full_prompt, true)
            .map_err(|e| CandleError::TokenizationError(e.to_string()))?;
        let input_ids = Tensor::from_slice(encoding.get_ids(), (1, encoding.len()), &self.device)
            .map_err(|e| CandleError::InferenceError(e.to_string()))?;

        let mut all_tokens = encoding.get_ids().to_vec();
        let params = InferenceParams::default();
        let mut logits_processor = LogitsProcessor::new(
            params.seed.unwrap_or_else(rand::random),
            Some(params.temperature),
            Some(params.top_p),
        );

        let inference_start = Instant::now();
        for _ in 0..params.max_tokens {
            let seqlen = all_tokens.len() - input_ids.dims()[1];
            let logits = model
                .forward(&input_ids, seqlen, None)
                .map_err(|e| CandleError::InferenceError(e.to_string()))?;

            let next = logits_processor
                .sample(&logits)
                .map_err(|e| CandleError::InferenceError(e.to_string()))?;

            if next == 151643 || next == 151645 {
                break;
            } // <|im_end|> or <|endoftext|>
            all_tokens.push(next);
        }

        let inference_time = inference_start.elapsed();
        let tokens_generated = all_tokens.len() - encoding.len();
        let output = tokenizer
            .decode(&all_tokens[encoding.len()..], true)
            .map_err(|e| CandleError::DetokenizationError(e.to_string()))?;

        let total_time = start.elapsed();
        let tps = if inference_time.as_secs_f64() > 0.0 {
            tokens_generated as f64 / inference_time.as_secs_f64()
        } else {
            0.0
        };

        Ok(InferenceResult {
            content: output.trim().to_string(),
            tokens_used: tokens_generated,
            inference_time_ms: total_time.as_millis() as u64,
            tokens_per_second: tps,
            finish_reason: "stop".to_string(),
        })
    }

    /// 卸载模型，释放内存
    #[cfg(feature = "ai")]
    pub async fn unload_model(&mut self) {
        if let Ok(mut m) = self.model.lock() {
            *m = None;
        }
        if let Ok(mut t) = self.tokenizer.lock() {
            *t = None;
        }
        self.status = BackendStatus::Ready;
        log::info!("Candle 模型已卸载");
    }
}

impl Default for CandleBackend {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== 桩实现（ai 未启用时） ====================

#[cfg(not(feature = "ai"))]
impl CandleBackend {
    pub fn load_from_hub(&mut self, _: &str) -> Result<(), CandleError> {
        Err(CandleError::ConfigError("ai feature 未启用".into()))
    }
    pub fn load_from_dir<P: AsRef<Path>>(&mut self, _dir: P) -> Result<(), CandleError> {
        Err(CandleError::ConfigError("ai feature 未启用".into()))
    }
    pub fn load_gguf<P: AsRef<Path>>(&mut self, _path: P) -> Result<(), CandleError> {
        Err(CandleError::ConfigError("ai feature 未启用".into()))
    }
    pub async fn inference(&self, _: &str) -> Result<InferenceResult, CandleError> {
        Err(CandleError::ModelNotLoaded)
    }
    pub async fn unload_model(&mut self) {}
}

// ==================== 辅助函数 ====================

/// 获取 home 目录
fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// 获取推荐的模型目录/路径
pub fn get_recommended_model_path() -> Result<(PathBuf, String), CandleError> {
    // 1. 环境变量 CARTPMS_MODEL_DIR 优先（推荐）
    if let Ok(dir) = std::env::var("CARTPMS_MODEL_DIR") {
        let p = PathBuf::from(&dir);
        if p.join("config.json").exists() && p.join("model.safetensors").exists() {
            return Ok((p, "dir".to_string()));
        }
    }

    // 2. 环境变量 CARTPMS_MODEL_PATH（GGUF 文件）
    if let Ok(path) = std::env::var("CARTPMS_MODEL_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok((p.parent().unwrap_or(&p).to_path_buf(), "gguf".to_string()));
        }
    }

    // 3. 根据能力等级推荐
    let resources = ResourceDetector::detect();
    let config = AiModuleConfig::from(resources);

    let hf_id = match config.capability_level {
        CapabilityLevel::Minimal | CapabilityLevel::Low => "Qwen/Qwen2.5-0.5B-Instruct",
        CapabilityLevel::Medium => "Qwen/Qwen2.5-1.5B-Instruct",
        CapabilityLevel::High => "Qwen/Qwen2.5-7B-Instruct",
        CapabilityLevel::Flagship => "Qwen/Qwen2.5-14B-Instruct",
    };

    // 检查本地缓存
    let cache_dir = home_dir().join(".cache").join("huggingface").join("hub");
    for e in std::fs::read_dir(&cache_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
    {
        let name = e.file_name().to_string_lossy().to_string();
        if name.contains("qwen") && name.contains("snapshots") {
            return Ok((e.path(), "dir".to_string()));
        }
    }

    Err(CandleError::ConfigError(format!(
        "未找到本地缓存模型。\n\
         推荐模型: {}\n\
         设置环境变量:\n  \
         export CARTPMS_MODEL_DIR=/path/to/{}  [本地 safetensors]\n  \
         export HF_ENDPOINT=https://hf-mirror.com  [国内镜像]\n\
         或启用 hf-hub 自动下载: load_from_hub(\"{}\")",
        hf_id, hf_id, hf_id
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = CandleBackend::new();
        log::info!(status = %backend.status(), "Candle 后端状态");
        assert!(matches!(
            backend.status(),
            BackendStatus::Ready | BackendStatus::Disabled
        ));
    }
}
