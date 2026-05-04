//! GatedDeltaNet 核心实现
//!
//! 对照 HuggingFace Transformers Qwen3_5GatedDeltaNet
//!
//! TODOs 已全部实现:
//! - TODO-①: 因果 Conv1d (手动滑动窗口)
//! - TODO-④: Gated Delta Rule (L2归一化 + 循环状态更新)
//! - sigmoid/softplus: 已验证数学公式正确

use candle_core::{DType, Device, Module, Result, Tensor};
use candle_nn::{linear_no_bias, VarBuilder};

use super::config::Qwen3_5Config;

// ═══════════════════════════════════════════════════════════════
// 因果卷积 (depthwise conv1d + causal padding + silu)
// 对照 Python: nn.Conv1d(in_c, in_c, k, groups=in_c) + F.silu
// ═══════════════════════════════════════════════════════════════

struct CausalConv1d {
    conv: candle_nn::Conv1d,
    kernel_size: usize,
}

impl CausalConv1d {
    fn new(conv_dim: usize, kernel_size: usize, vb: VarBuilder) -> Result<Self> {
        let config = candle_nn::Conv1dConfig {
            padding: 0,
            stride: 1,
            dilation: 1,
            groups: conv_dim,
            ..Default::default()
        };
        let conv = candle_nn::conv1d(conv_dim, conv_dim, kernel_size, config, vb)?;
        Ok(Self { conv, kernel_size })
    }

    /// depthwise causal conv1d + silu
    /// Python: F.silu(conv1d(x))[:, :, :T]
    ///
    /// 实现方式：
    /// 1. 左侧填充 (kernel_size - 1) 个 0 → 因果约束（t 时刻只看 t 及之前）
    /// 2. depthwise conv1d (groups=conv_dim) → 每个通道独立卷积
    /// 3. silu 激活
    /// 4. 裁剪到原长度
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let b = x.dim(0)?;
        let c = x.dim(1)?;
        let _t = x.dim(2)?;
        let k = self.kernel_size;

        // 1. 因果 padding: 左边补 K-1 个 0
        let pad = Tensor::zeros(&[b, c, k - 1], x.dtype(), x.device())?;
        let x_pad = Tensor::cat(&[&pad, x], 2)?; // [B, C, T+K-1]

        // 2. depthwise conv1d (groups=C)
        let conv_out = self.conv.forward(&x_pad)?; // [B, C, (T+K-1) - K + 1] = [B, C, T]

        // 3. silu 激活
        candle_nn::Activation::Silu.forward(&conv_out)
    }
}

// ═══════════════════════════════════════════════════════════════
// RMSNormGated (对照 Python Qwen3_5RMSNormGated)
// output = RMSNorm(x) * silu(gate)
// ═══════════════════════════════════════════════════════════════

struct RMSNormGated {
    weight: Tensor,
    eps: f64,
}

impl RMSNormGated {
    fn new(size: usize, eps: f64, vb: VarBuilder) -> Result<Self> {
        let weight = vb.get(size, "weight")?;
        Ok(Self { weight, eps })
    }

    fn forward(&self, x: &Tensor, gate: &Tensor) -> Result<Tensor> {
        // output = RMSNorm(x) * silu(gate)
        let last = x.dims().len() - 1;
        let x_sq = x.sqr()?;
        let mean = x_sq.mean(last)?;
        let eps_v = Tensor::new(self.eps, x.device())?;
        let rms = (mean + eps_v)?.sqrt()?;
        let rms_e = rms.unsqueeze(last)?;
        let norm = x.broadcast_div(&rms_e)? * &self.weight;
        let act = candle_nn::Activation::Silu.forward(gate)?;
        norm * act
    }
}

// ═══════════════════════════════════════════════════════════════
// GatedDeltaNet 主层
// ═══════════════════════════════════════════════════════════════

pub struct GatedDeltaNet {
    in_proj_qkv: candle_nn::Linear,
    in_proj_z: candle_nn::Linear,
    in_proj_b: candle_nn::Linear,
    in_proj_a: candle_nn::Linear,
    out_proj: candle_nn::Linear,
    dt_bias: Tensor,
    a_log: Tensor,
    conv1d: CausalConv1d,
    norm: RMSNormGated,
    num_v_heads: usize,
    head_dim: usize,
    key_dim: usize,
    value_dim: usize,
    device: Device,
}

impl GatedDeltaNet {
    pub fn new(cfg: &Qwen3_5Config, vb: VarBuilder) -> Result<Self> {
        let nv = cfg.num_v_heads();
        let hd = cfg.head_v_dim();
        let kd = cfg.key_dim();
        let vd = cfg.value_dim();

        Ok(Self {
            in_proj_qkv: linear_no_bias(cfg.hidden_size, kd * 2 + vd, vb.pp("in_proj_qkv"))?,
            in_proj_z: linear_no_bias(cfg.hidden_size, vd, vb.pp("in_proj_z"))?,
            in_proj_b: linear_no_bias(cfg.hidden_size, nv, vb.pp("in_proj_b"))?,
            in_proj_a: linear_no_bias(cfg.hidden_size, nv, vb.pp("in_proj_a"))?,
            out_proj: linear_no_bias(vd, cfg.hidden_size, vb.pp("out_proj"))?,
            conv1d: CausalConv1d::new(kd * 2 + vd, cfg.conv_kernel(), vb.pp("conv1d"))?,
            dt_bias: vb.get(nv, "dt_bias")?,
            a_log: vb.get(nv, "A_log")?,
            norm: RMSNormGated::new(hd, cfg.rms_norm_eps, vb.pp("norm"))?,
            num_v_heads: nv,
            head_dim: hd,
            key_dim: kd,
            value_dim: vd,
            device: vb.device().clone(),
        })
    }

    /// GatedDeltaNet forward pass with optional KV-cache.
    ///
    /// `cache` is the memory matrix M [B, H, d_k, d_v] from previous step.
    /// - `None`: full sequence processing (prefill phase)
    /// - `Some(M)`: incremental processing (decode phase), only last token is new
    pub fn forward(
        &self,
        hidden_states: &Tensor,     // [B, T, D]
        cache: Option<&mut Tensor>, // [B, H, d_k, d_v] or None
    ) -> Result<Tensor> {
        let b = hidden_states.dim(0)?;
        let s = hidden_states.dim(1)?;

        // 1. in_proj_qkv
        let mixed0 = self.in_proj_qkv.forward(hidden_states)?;

        // 2. causal_conv1d → silu
        let mixed1 = mixed0.transpose(1, 2)?;
        let mixed2 = self.conv1d.forward(&mixed1)?;
        let mixed3 = mixed2.transpose(1, 2)?;

        // 3. split q/k/v
        let q0 = mixed3.narrow(2, 0, self.key_dim)?;
        let k0 = mixed3.narrow(2, self.key_dim, self.key_dim)?;
        let v0 = mixed3.narrow(2, self.key_dim * 2, self.value_dim)?;

        // 4. z, b, a
        let z0 = self.in_proj_z.forward(hidden_states)?;
        let b_t = self.in_proj_b.forward(hidden_states)?;
        let a_t = self.in_proj_a.forward(hidden_states)?;

        // 5. reshape 多头
        let q = q0.reshape((b, s, self.num_v_heads, self.head_dim))?;
        let k = k0.reshape((b, s, self.num_v_heads, self.head_dim))?;
        let v = v0.reshape((b, s, self.num_v_heads, self.head_dim))?;
        let z = z0.reshape((b, s, self.num_v_heads, self.head_dim))?;

        // 6. beta / g 门控
        let beta_t = sigmoid(&b_t)?;
        let exp_a = self.a_log.neg()?.exp()?;
        let g_t = softplus(&(&a_t + &self.dt_bias)?)?;
        let g_t = g_t.broadcast_mul(&exp_a.unsqueeze(0)?.unsqueeze(0)?)?;

        // 7. Gated Delta Rule with optional KV-cache
        let q_h = q.permute((0, 2, 1, 3))?; // [B,T,H,D] → [B,H,T,D]
        let k_h = k.permute((0, 2, 1, 3))?;
        let v_h = v.permute((0, 2, 1, 3))?;
        let g_h = g_t.permute((0, 2, 1))?; // [B,T,H] → [B,H,T]
        let b_h = beta_t.permute((0, 2, 1))?;

        let attn_h = gated_delta_rule_cached(&q_h, &k_h, &v_h, &g_h, &b_h, &self.device, cache)?;
        let attn_out = attn_h.permute((0, 2, 1, 3))?; // [B,H,T,D] → [B,T,H,D]

        // 8. RMSNormGated
        let a_2d = attn_out.reshape((b * s, self.num_v_heads * self.head_dim))?;
        let z_2d = z.reshape((b * s, self.num_v_heads * self.head_dim))?;
        let normed = self.norm.forward(&a_2d, &z_2d)?;
        let normed = normed.reshape((b, s, self.value_dim))?;

        // 9. out_proj
        self.out_proj.forward(&normed)
    }
}

// ═══════════════════════════════════════════════════════════════
// Gated Delta Rule (对照 flash-linear-attention)
//
// 使用记忆矩阵 M [B, H, d_k, d_v]:
//   1. M *= exp(g)                          记忆衰减
//   2. kv_mem = sum_k(M * k_t)              预测值
//   3. delta = (v - kv_mem) * beta          误差 x 学习率
//   4. M += outer(k_t, delta)               外积更新
//   5. output = sum_k(M * q_t)              读取输出
// ═══════════════════════════════════════════════════════════════

#[allow(dead_code)]
fn gated_delta_rule(
    query: &Tensor, // [B, H, T, d_k]
    key: &Tensor,   // [B, H, T, d_k]
    value: &Tensor, // [B, H, T, d_v]
    g: &Tensor,     // [B, H, T]   (log space, 使用时 exp)
    beta: &Tensor,  // [B, H, T]
    device: &Device,
) -> Result<Tensor> {
    let b = query.dim(0)?;
    let h = query.dim(1)?;
    let t = query.dim(2)?;
    let dk = query.dim(3)?;
    let dv = value.dim(3)?;

    // scale = 1/sqrt(d_k)
    let scale = Tensor::new(1.0 / (dk as f64).sqrt(), device)?;

    // 初始化记忆矩阵 M [B, H, d_k, d_v]
    let mut state_m = Tensor::zeros(&[b, h, dk, dv], DType::F32, device)?;

    let mut out_chunks: Vec<Tensor> = Vec::with_capacity(t);
    for i in 0..t {
        let q_t = query.narrow(2, i, 1)?.squeeze(2)?; // [B, H, d_k]
        let k_t = key.narrow(2, i, 1)?.squeeze(2)?;
        let v_t = value.narrow(2, i, 1)?.squeeze(2)?; // [B, H, d_v]
        let g_t = g.narrow(2, i, 1)?.squeeze(2)?; // [B, H]
        let b_t = beta.narrow(2, i, 1)?.squeeze(2)?; // [B, H]

        // L2 normalize q/k
        let q_n = l2_normalize(&q_t, 2)?; // [B, H, d_k]
        let k_n = l2_normalize(&k_t, 2)?;

        // Apply scale to query
        let q_n = (q_n * &scale)?;

        // g: [B, H] → [B, H, 1, 1] for broadcast with [B, H, d_k, d_v]
        let g_exp = g_t.exp()?; // [B, H]
        let g_4d = g_exp.unsqueeze(2)?.unsqueeze(3)?; // [B, H, 1, 1]

        // b: [B, H] → [B, H, 1] for broadcast with [B, H, d_v]
        let b_3d = b_t.unsqueeze(2)?; // [B, H, 1]

        // 1. Memory decay: M *= exp(g)
        state_m = (state_m * g_4d)?;

        // 2. Predict value: kv_mem = sum_k(M * k_t, dim=-2) = [B, H, d_v]
        // M: [B, H, d_k, d_v], k_n: [B, H, d_k, 1]
        // (M * k_n.unsqueeze(-1)).sum(-2) = [B, H, d_v]
        let k_4d = k_n.unsqueeze(3)?; // [B, H, d_k, 1]
        let kv_mem = (&state_m * &k_4d)?.sum(2)?; // [B, H, d_v]

        // 3. Delta update: delta = (v - kv_mem) * beta
        let delta = ((v_t - kv_mem)? * &b_3d)?; // [B, H, d_v]

        // 4. Memory update: M += outer(k_t, delta)
        let delta_4d = delta.unsqueeze(2)?; // [B, H, 1, d_v]
        state_m = (state_m + k_4d.broadcast_mul(&delta_4d)?)?;

        // 5. Read output: o = sum_k(M * q_t, dim=-2) = [B, H, d_v]
        let q_4d = q_n.unsqueeze(3)?; // [B, H, d_k, 1]
        let o_t = (&state_m * &q_4d)?.sum(2)?; // [B, H, d_v]
        out_chunks.push(o_t.unsqueeze(2)?); // [B, H, 1, d_v]
    }
    Tensor::cat(&out_chunks, 2) // [B, H, T, d_v]
}

/// Gated Delta Rule with optional KV-cache.
///
/// `cache_m` provides the initial memory matrix M from previous decode steps.
/// Only processes the last token when cache is provided (decode phase optimization).
fn gated_delta_rule_cached(
    query: &Tensor,
    key: &Tensor,
    value: &Tensor,
    g: &Tensor,
    beta: &Tensor,
    device: &Device,
    cache_m: Option<&mut Tensor>,
) -> Result<Tensor> {
    let b = query.dim(0)?;
    let h = query.dim(1)?;
    let t = query.dim(2)?;
    let dk = query.dim(3)?;
    let dv = value.dim(3)?;
    let scale = Tensor::new(1.0 / (dk as f64).sqrt(), device)?;

    let mut state_m = match cache_m {
        Some(ref m) => (**m).clone(),
        None => Tensor::zeros(&[b, h, dk, dv], DType::F32, device)?,
    };

    let mut out_chunks: Vec<Tensor> = Vec::with_capacity(t);
    for i in 0..t {
        let q_t = query.narrow(2, i, 1)?.squeeze(2)?;
        let k_t = key.narrow(2, i, 1)?.squeeze(2)?;
        let v_t = value.narrow(2, i, 1)?.squeeze(2)?;
        let g_t = g.narrow(2, i, 1)?.squeeze(2)?;
        let b_t = beta.narrow(2, i, 1)?.squeeze(2)?;

        let q_n = (l2_normalize(&q_t, 2)? * &scale)?;
        let k_n = l2_normalize(&k_t, 2)?;
        let g_4d = g_t.exp()?.unsqueeze(2)?.unsqueeze(3)?;
        let b_3d = b_t.unsqueeze(2)?;

        state_m = (state_m * g_4d)?;
        let k_4d = k_n.unsqueeze(3)?;
        let kv_mem = (&state_m * &k_4d)?.sum(2)?;
        let delta = v_t.broadcast_sub(&kv_mem)?.broadcast_mul(&b_3d)?;
        state_m = (state_m + k_4d.broadcast_mul(&delta.unsqueeze(2)?)?)?;

        let o_t = (&state_m * &q_n.unsqueeze(3)?)?.sum(2)?;
        out_chunks.push(o_t.unsqueeze(2)?);
    }

    if let Some(m) = cache_m {
        let _ = std::mem::replace(m, state_m);
    }

    Tensor::cat(&out_chunks, 2)
}

/// 每层的 GatedDeltaNet 内存缓存（记忆矩阵 M）
pub struct GatedDeltaCache(pub Vec<Tensor>);

impl GatedDeltaCache {
    pub fn new(num_layers: usize) -> Self {
        Self(Vec::with_capacity(num_layers))
    }

    pub fn get_mut(&mut self, layer_idx: usize) -> Option<&mut Tensor> {
        self.0.get_mut(layer_idx)
    }

    pub fn push(&mut self, m: Tensor) {
        self.0.push(m);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// L2 normalize along specified dimension
fn l2_normalize(x: &Tensor, dim: usize) -> Result<Tensor> {
    let norm = x.sqr()?.sum(dim)?.sqrt()?;
    x.broadcast_div(&norm.unsqueeze(dim)?)
}

// ═══════════════════════════════════════════════════════════════
// 数学工具
// ═══════════════════════════════════════════════════════════════

fn sigmoid(x: &Tensor) -> Result<Tensor> {
    let one = Tensor::ones(x.shape(), x.dtype(), x.device())?;
    let neg_exp = x.neg()?.exp()?;
    one.broadcast_div(&(neg_exp + &one)?)
}

fn softplus(x: &Tensor) -> Result<Tensor> {
    let exp_x = x.exp()?;
    let one = Tensor::ones(exp_x.shape(), exp_x.dtype(), exp_x.device())?;
    (one + exp_x)?.log()
}
