//! Qwen3.5 解码器层
//!
//! 支持两种层类型:
//! - linear_attention: GatedDeltaNet (默认)
//! - full_attention: 标准 Self-Attention + RoPE

use candle_core::{Module, Result, Tensor};
use candle_nn::{linear_no_bias, Activation, Linear, VarBuilder};

use super::config::Qwen3_5Config;
use super::gated_delta::{GatedDeltaCache, GatedDeltaNet};

/// Qwen3.5 解码器层
pub struct Qwen3_5DecoderLayer {
    /// GatedDeltaNet (layer_type = "linear_attention")
    linear_attn: Option<GatedDeltaNet>,
    /// 标准 Self-Attention (layer_type = "full_attention") TODO-⑦
    // self_attn: Option<Qwen3_5Attention>,
    /// MLP (SwiGLU)
    mlp: Qwen3_5MLP,
    /// 归一化
    input_norm: candle_nn::LayerNorm,
    post_attn_norm: candle_nn::LayerNorm,
    /// 层类型
    is_linear: bool,
}

impl Qwen3_5DecoderLayer {
    pub fn new(cfg: &Qwen3_5Config, layer_idx: usize, vb: VarBuilder) -> Result<Self> {
        let is_linear = cfg.is_linear_layer(layer_idx);

        let linear_attn = if is_linear {
            Some(GatedDeltaNet::new(cfg, vb.pp("linear_attn"))?)
        } else {
            None // TODO-⑦: Qwen3_5Attention::new(cfg, vb.pp("self_attn"))
        };

        let mlp = Qwen3_5MLP::new(cfg, vb.pp("mlp"))?;
        let input_norm =
            candle_nn::layer_norm(cfg.hidden_size, cfg.rms_norm_eps, vb.pp("input_layernorm"))?;
        let post_attn_norm = candle_nn::layer_norm(
            cfg.hidden_size,
            cfg.rms_norm_eps,
            vb.pp("post_attention_layernorm"),
        )?;

        Ok(Self {
            linear_attn,
            mlp,
            input_norm,
            post_attn_norm,
            is_linear,
        })
    }

    /// 前向: Pre-Norm → GDN/SelfAttn → Residual → MLP → Residual
    ///
    /// `cache` 是每层的 GatedDeltaNet 记忆矩阵缓存（用于解阶段优化）。
    pub fn forward(
        &self,
        hidden_states: &Tensor,
        cache: &mut Option<&mut GatedDeltaCache>,
        layer_idx: usize,
    ) -> Result<Tensor> {
        // 注意力子层
        let hidden = {
            let normed = self.input_norm.forward(hidden_states)?;
            let attn_out = if self.is_linear {
                let layer_cache = cache.as_mut().and_then(|c| c.get_mut(layer_idx));
                self.linear_attn
                    .as_ref()
                    .unwrap()
                    .forward(&normed, layer_cache)?
            } else {
                normed.clone() // TODO-⑦ 标准 Attention
            };
            hidden_states.add(&attn_out)?
        };
        // MLP 子层
        let normed = self.post_attn_norm.forward(&hidden)?;
        let mlp_out = self.mlp.forward(&normed)?;
        hidden.add(&mlp_out)
    }
}

/// SwiGLU MLP: (x@W_gate ⊙ silu(x@W_up)) @ W_down
pub struct Qwen3_5MLP {
    gate_proj: Linear,
    up_proj: Linear,
    down_proj: Linear,
}

impl Qwen3_5MLP {
    pub fn new(cfg: &Qwen3_5Config, vb: VarBuilder) -> Result<Self> {
        Ok(Self {
            gate_proj: linear_no_bias(cfg.hidden_size, cfg.intermediate_size, vb.pp("gate_proj"))?,
            up_proj: linear_no_bias(cfg.hidden_size, cfg.intermediate_size, vb.pp("up_proj"))?,
            down_proj: linear_no_bias(cfg.intermediate_size, cfg.hidden_size, vb.pp("down_proj"))?,
        })
    }

    pub fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let gate = Activation::Silu.forward(&self.gate_proj.forward(xs)?)?;
        let up = self.up_proj.forward(xs)?;
        let gated = gate.mul(&up)?;
        self.down_proj.forward(&gated)
    }
}
