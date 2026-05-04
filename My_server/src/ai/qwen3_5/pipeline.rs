//! Qwen3.5 模型加载与推理管线
//!
//! 提供从 HuggingFace safetensors 加载 Qwen3.5 模型的端到端流程。

use std::path::Path;

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;

use super::config::Qwen3_5Config;
use super::model::Qwen3_5ForCausalLM;

/// Qwen3.5 推理管线错误
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("模型未加载")]
    NotLoaded,
    #[error("模型加载失败: {0}")]
    LoadError(String),
    #[error("推理失败: {0}")]
    InferenceError(String),
    #[error("配置错误: {0}")]
    ConfigError(String),
    #[error("标记化失败: {0}")]
    TokenizationError(String),
}

/// 推理参数
#[derive(Debug, Clone)]
pub struct InferenceParams {
    pub max_tokens: usize,
    pub temperature: f64,
    pub top_p: f64,
    pub seed: Option<u64>,
    pub eos_token_id: u32,
}

impl Default for InferenceParams {
    fn default() -> Self {
        Self {
            max_tokens: 1024,
            temperature: 0.7,
            top_p: 0.9,
            seed: None,
            eos_token_id: 151645,
        }
    }
}

/// Qwen3.5 推理管线
pub struct Qwen3_5Pipeline {
    pub model: Option<Qwen3_5ForCausalLM>,
    config: Option<Qwen3_5Config>,
    tokenizer: Option<tokenizers::Tokenizer>,
    device: Device,
    pub params: InferenceParams,
}

impl Qwen3_5Pipeline {
    /// 创建新管线
    pub fn new() -> Self {
        let device = Self::select_device();
        Self {
            model: None,
            config: None,
            tokenizer: None,
            device,
            params: InferenceParams::default(),
        }
    }

    fn select_device() -> Device {
        if let Ok(d) = Device::cuda_if_available(0) {
            if d.is_cuda() {
                log::info!("Qwen3.5 使用 CUDA");
                return d;
            }
        }
        #[cfg(target_os = "macos")]
        if let Ok(d) = Device::metal_if_available(0) {
            if d.is_metal() {
                log::info!("Qwen3.5 使用 Metal");
                return d;
            }
        }
        Device::Cpu
    }

    /// 从 HuggingFace Hub 下载并加载
    #[cfg(feature = "ai")]
    pub fn load_from_hub(&mut self, model_id: &str) -> Result<(), PipelineError> {
        use hf_hub::api::sync::Api;
        let api = Api::new().map_err(|e| PipelineError::LoadError(e.to_string()))?;
        let repo = api.model(model_id.to_string());

        let config: Qwen3_5Config = serde_json::from_slice(
            &std::fs::read(
                repo.get("config.json")
                    .map_err(|e| PipelineError::LoadError(e.to_string()))?,
            )
            .map_err(|e| PipelineError::LoadError(e.to_string()))?,
        )
        .map_err(|e| PipelineError::LoadError(e.to_string()))?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(
                &[repo
                    .get("model.safetensors")
                    .map_err(|e| PipelineError::LoadError(e.to_string()))?],
                DType::F32,
                &self.device,
            )
        }
        .map_err(|e| PipelineError::LoadError(e.to_string()))?;

        let model = Qwen3_5ForCausalLM::new(&config, vb)
            .map_err(|e| PipelineError::LoadError(e.to_string()))?;

        let tokenizer = tokenizers::Tokenizer::from_file(
            repo.get("tokenizer.json")
                .map_err(|e| PipelineError::LoadError(e.to_string()))?,
        )
        .map_err(|e| PipelineError::LoadError(e.to_string()))?;

        self.model = Some(model);
        self.config = Some(config);
        self.tokenizer = Some(tokenizer);
        log::info!("Qwen3.5 模型加载完成");
        Ok(())
    }

    /// 从本地目录加载
    pub fn load_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), PipelineError> {
        let d = dir.as_ref();
        let cfg_p = d.join("config.json");
        let wgt_p = d.join("model.safetensors");
        let tok_p = d.join("tokenizer.json");
        if !cfg_p.exists() || !wgt_p.exists() {
            return Err(PipelineError::LoadError(format!(
                "目录不完整: {}",
                d.display()
            )));
        }
        let config: Qwen3_5Config = serde_json::from_slice(
            &std::fs::read(&cfg_p).map_err(|e| PipelineError::LoadError(e.to_string()))?,
        )
        .map_err(|e| PipelineError::LoadError(e.to_string()))?;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[wgt_p], DType::F32, &self.device) }
            .map_err(|e| PipelineError::LoadError(e.to_string()))?;
        let model = Qwen3_5ForCausalLM::new(&config, vb)
            .map_err(|e| PipelineError::LoadError(e.to_string()))?;
        let tokenizer = if tok_p.exists() {
            Some(
                tokenizers::Tokenizer::from_file(&tok_p)
                    .map_err(|e| PipelineError::LoadError(e.to_string()))?,
            )
        } else {
            None
        };
        self.model = Some(model);
        self.config = Some(config);
        self.tokenizer = tokenizer;
        Ok(())
    }

    /// 推理
    pub fn infer(&mut self, prompt: &str) -> Result<String, PipelineError> {
        let m = self.model.as_ref().ok_or(PipelineError::NotLoaded)?;
        let t = self.tokenizer.as_ref().ok_or(PipelineError::NotLoaded)?;
        let full = format!(
            "<|im_start|>system\n你是 CarpTMS AI 助手。<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n", prompt
        );
        let enc = t
            .encode(full, true)
            .map_err(|e| PipelineError::TokenizationError(e.to_string()))?;
        let ids = Tensor::new(enc.get_ids().to_vec(), &self.device)
            .map_err(|e| PipelineError::InferenceError(e.to_string()))?
            .unsqueeze(0)
            .map_err(|e| PipelineError::InferenceError(e.to_string()))?;
        let proc = LogitsProcessor::new(
            self.params.seed.unwrap_or_else(rand::random),
            Some(self.params.temperature),
            Some(self.params.top_p),
        );
        let (tokens, _) = m
            .generate(&ids, proc, self.params.max_tokens, self.params.eos_token_id)
            .map_err(|e| PipelineError::InferenceError(e.to_string()))?;
        let out = t
            .decode(&tokens, true)
            .map_err(|e| PipelineError::TokenizationError(e.to_string()))?;
        Ok(out.trim().to_string())
    }
}

impl Qwen3_5Pipeline {
    /// 使用指定设备创建（给 GGUF 加载器用）
    pub fn new_with_device(device: Device) -> Self {
        Self {
            model: None,
            config: None,
            tokenizer: None,
            device,
            params: InferenceParams::default(),
        }
    }

    /// 从已创建的模型和配置构建管线（给 GGUF 加载器用）
    pub fn new_with_model(model: Qwen3_5ForCausalLM, config: Option<Qwen3_5Config>) -> Self {
        let device = Self::select_device();
        Self {
            model: Some(model),
            config,
            tokenizer: None,
            device,
            params: InferenceParams::default(),
        }
    }

    /// 设置 tokenizer（GGUF 加载完成后设置）
    pub fn set_tokenizer(&mut self, tokenizer: tokenizers::Tokenizer) {
        self.tokenizer = Some(tokenizer);
    }

    /// 获取设备引用
    pub fn device(&self) -> &Device {
        &self.device
    }
}

impl Default for Qwen3_5Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
