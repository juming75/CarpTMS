//! Qwen3.5 模型模块
//!
//! 在 Candle 框架上实现的 Qwen3.5 模型，核心是 **GatedDeltaNet** 架构。
//!
//! ## 模块结构
//!
//! | 文件 | 内容 |
//! |------|------|
//! | `config.rs` | Qwen3_5Config，含 GatedDeltaNet 特有参数 |
//! | `gated_delta.rs` | 核心：多头门控循环层 |
//! | `layers.rs` | DecoderLayer + SwiGLU MLP |
//! | `model.rs` | Qwen3_5Model + Qwen3_5ForCausalLM |
//! | `pipeline.rs` | 模型加载 + Tokenizer + 推理管线 |
//!
//! ## 贡献计划
//!
//! 实现稳定后，计划将此模块 PR 到 [huggingface/candle](https://github.com/huggingface/candle) 主仓库：
//! 1. 创建 `candle-transformers/src/models/qwen3_5/` 目录
//! 2. 将本模块代码适配为 candle 标准格式
//! 3. 提交 PR，添加 Qwen3.5 支持
//!
//! 这将使 CarpTMS 团队跻身 HuggingFace Candle 核心贡献者行列。

pub mod config;
pub mod gated_delta;
pub mod gguf_loader;
pub mod layers;
pub mod model;
pub mod pipeline;
pub mod prompts;
pub mod tasks;

pub use config::Qwen3_5Config;
pub use gguf_loader::Qwen3_5GGufLoader;
pub use pipeline::{InferenceParams, PipelineError, Qwen3_5Pipeline};
pub use tasks::*;
