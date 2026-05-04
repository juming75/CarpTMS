//! AI 服务模块
//! 纯本地推理，数据不出网

pub mod adaptive;
pub mod candle_backend;
pub mod qwen3_5;
pub mod request_router;
pub mod resource;
pub mod routes;

pub use crate::ml;
pub use adaptive::{AdaptiveAiManager, AiBackendType, AiServiceStatus, UnifiedInferenceResponse};
pub use candle_backend::{
    BackendStatus, CandleBackend, CandleError, InferenceParams, InferenceResult,
};
pub use qwen3_5::{PipelineError, Qwen3_5Config, Qwen3_5Pipeline};
pub use resource::{AiModuleConfig, CapabilityLevel, ResourceDetector, SystemResources};
