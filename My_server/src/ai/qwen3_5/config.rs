//! Qwen3.5 模型配置
//!
//! 参考 HuggingFace Qwen3.5-9B config.json
//!
//! GatedDeltaNet 特有字段:
//! - layer_types: 每层类型 ["linear_attention"|"full_attention"]
//! - linear_num_value_heads: Value 头数
//! - linear_num_key_heads: Key 头数
//! - linear_key_head_dim: Key 头维度
//! - linear_value_head_dim: Value 头维度
//! - linear_conv_kernel_dim: Conv1d 核大小

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Qwen3_5Config {
    // 标准 Transformer
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub max_position_embeddings: usize,
    pub rope_theta: f64,
    pub rms_norm_eps: f64,
    pub tie_word_embeddings: bool,
    #[serde(default = "default_hidden_act")]
    pub hidden_act: String,

    // GatedDeltaNet 特有
    #[serde(default)]
    pub layer_types: Vec<String>,
    #[serde(default)]
    pub linear_num_value_heads: Option<usize>,
    #[serde(default)]
    pub linear_num_key_heads: Option<usize>,
    #[serde(default)]
    pub linear_key_head_dim: Option<usize>,
    #[serde(default)]
    pub linear_value_head_dim: Option<usize>,
    #[serde(default = "default_conv_kernel")]
    pub linear_conv_kernel_dim: usize,
}

fn default_hidden_act() -> String {
    "silu".to_string()
}
fn default_conv_kernel() -> usize {
    4
}

impl Qwen3_5Config {
    pub fn is_linear_layer(&self, idx: usize) -> bool {
        self.layer_types
            .get(idx)
            .map(|t| t == "linear_attention")
            .unwrap_or(true)
    }
    pub fn num_v_heads(&self) -> usize {
        self.linear_num_value_heads
            .unwrap_or(self.num_attention_heads)
    }
    pub fn num_k_heads(&self) -> usize {
        self.linear_num_key_heads
            .unwrap_or(self.num_key_value_heads)
    }
    pub fn head_k_dim(&self) -> usize {
        self.linear_key_head_dim
            .unwrap_or(self.hidden_size / self.num_attention_heads)
    }
    pub fn head_v_dim(&self) -> usize {
        self.linear_value_head_dim
            .unwrap_or(self.hidden_size / self.num_attention_heads)
    }
    pub fn key_dim(&self) -> usize {
        self.head_k_dim() * self.num_k_heads()
    }
    pub fn value_dim(&self) -> usize {
        self.head_v_dim() * self.num_v_heads()
    }
    pub fn conv_kernel(&self) -> usize {
        self.linear_conv_kernel_dim
    }
}

impl Default for Qwen3_5Config {
    fn default() -> Self {
        Self {
            vocab_size: 152064,
            hidden_size: 4096,
            intermediate_size: 11008,
            num_hidden_layers: 32,
            num_attention_heads: 32,
            num_key_value_heads: 8,
            max_position_embeddings: 131072,
            rope_theta: 1000000.0,
            rms_norm_eps: 1e-6,
            tie_word_embeddings: false,
            hidden_act: "silu".to_string(),
            layer_types: vec![],
            linear_num_value_heads: None,
            linear_num_key_heads: None,
            linear_key_head_dim: None,
            linear_value_head_dim: None,
            linear_conv_kernel_dim: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_layer_types() {
        let json = r#"{"vocab_size":152064,"hidden_size":4096,"intermediate_size":11008,"num_hidden_layers":32,"num_attention_heads":32,"num_key_value_heads":8,"max_position_embeddings":131072,"rope_theta":1000000.0,"rms_norm_eps":1e-6,"layer_types":["linear_attention","full_attention"],"linear_num_value_heads":32,"linear_num_key_heads":8,"linear_key_head_dim":128,"linear_value_head_dim":128,"linear_conv_kernel_dim":4}"#;
        let c: Qwen3_5Config = serde_json::from_str(json).unwrap();
        assert!(c.is_linear_layer(0));
        assert!(!c.is_linear_layer(1));
        assert_eq!(c.num_v_heads(), 32);
    }
}
