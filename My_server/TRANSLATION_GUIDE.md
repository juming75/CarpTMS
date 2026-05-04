# `modeling_qwen3_5.py` → `gated_delta.rs` 逐行对照翻译指南

对照源：`transformers/src/transformers/models/qwen3_5/modeling_qwen3_5.py`

---

## 总体偏差评估

| # | Python 关键特性 | Rust 当前实现 | 严重程度 |
|---|----------------|---------------|----------|
| ① | `causal_conv1d` 因果卷积 | **缺失** | 🔴 关键 |
| ② | `in_proj_qkv` 单投影 → (key_dim×2+value_dim) | 三个独立投影 q/k/v | 🔴 结构错 |
| ③ | `dt_bias + A_log + softplus(a)` 计算 g | `sigmoid(gate)` | 🔴 公式错 |
| ④ | `recurrent/chunk_gated_delta_rule` 核 | 手写 `s = g*s + (1-g)*k` | 🔴 简过 |
| ⑤ | `RMSNormGated(x, gate) = RMSNorm(x) * silu(gate)` | 标准 LayerNorm | 🟡 不同 |
| ⑥ | `layer_types`: 每层可配 linear/full_attention | 所有层相同 | 🟡 遗漏 |
| ⑦ | `Qwen3_5Attention`（标准 Self-Attention + RoPE） | 未实现 | 🟡 遗漏 |

---

## ① 因果卷积 (Conv1d) 替换

```python
# Python:
conv_dim = key_dim * 2 + value_dim
self.conv1d = nn.Conv1d(
    in_channels=conv_dim, out_channels=conv_dim,
    bias=False, kernel_size=conv_kernel,
    groups=conv_dim, padding=conv_kernel - 1,
)

# forward:
mixed_qkv = x.transpose(1, 2)          # [B, T, D] → [B, D, T]
mixed_qkv = F.silu(self.conv1d(         # [B, D, T] 因果conv
    mixed_qkv
)[:, :, :mixed_qkv.shape[-1]])          # 去掉多余 padding
mixed_qkv = x.transpose(1, 2)          # [B, D, T] → [B, T, D]
```

```rust
// Rust: candle 没有原生 depthwise Conv1d
// 方案 A: 手动实现 unfold + matmul
// 方案 B: 用 candle_nn::conv1d + mask 模拟因果
// 方案 C: 简化: 跳过 conv1d，直接用 silu(linear(x))
```

---

## ② 投影结构

```python
# Python:
self.in_proj_qkv = Linear(hidden → key_dim*2 + value_dim)
self.in_proj_z   = Linear(hidden → value_dim)
self.in_proj_b   = Linear(hidden → num_v_heads)
self.in_proj_a   = Linear(hidden → num_v_heads)

# forward:
mixed_qkv = in_proj_qkv(x)           # [B, T, K*2+V]
query, key, value = split(mixed_qkv, [K, K, V], dim=-1)
```

```rust
// Rust 当前: q_proj, k_proj, v_proj (三个独立) → ❌ 错误
// 正确: in_proj_qkv (一个投影) + chunk split
let mixed = self.in_proj_qkv.forward(x)?;                 // [B, T, K*2+V]
let (q, k, v) = /* candle 没有原生chunk, 用 narrow 实现 */;
```

---

## ③ 门控公式

```python
# Python (真实):
beta = b.sigmoid()                     # [B, T, num_v_h]  遗忘门
g = -A_log.exp() * softplus(a + dt_bias) # [B, T, num_v_h] 输入门
# A ∈ Uniform(0,16), A_log = ln(A), dt_bias ∈ ℝ^num_v_h
```

```rust
// Rust 当前 (错误):
// let gate = sigmoid(gate_proj(x));  // 这只是 beta！缺了 g 的计算

// 正确:
let beta = sigmoid(&b)?;  // 遗忘门
let exp_a = (-self.A_log)?.exp()?;
let s = softplus(&(a.broadcast_add(&self.dt_bias)?))?;
let g = s.broadcast_mul(&exp_a)?;
```

---

## ④ Gated Delta Rule（核心中的核心）

```python
# Python (来自 fla 库):
core_attn_out, last_state = fused_recurrent_gated_delta_rule(
    query, key, value,
    g=g,                   # 输入门 (softplus+exp)
    beta=beta,             # 遗忘门 (sigmoid)
    initial_state=state,
    output_final_state=True,
    use_qk_l2norm_in_kernel=True,  # L2 norm on q, k
)
```

这个 kernel 的实际数学 (pseudocode):
```
for t in 0..seq_len:
    q_n = q[t] / ||q[t]||      # L2 normalize
    k_n = k[t] / ||k[t]||  
    state = beta[t] * state + (1 - beta[t]) * k_n * g[t]  
    o[t] = state * v[t]
    # 实际还有 q 参与运算，这里简化了
```

**注意**：`fused_recurrent_gated_delta_rule` 是 CUDA kernel（来自 `flash-linear-attention` 库）。
**在纯 Rust CPU 环境无法直接使用**，需要在 candle 中用**手动循环**实现纯 torch 版。

---

## ⑤ RMSNormGated

```python
# Python:
class Qwen3_5RMSNormGated:
    def forward(self, x, gate):
        # weight ∈ ℝ^{D}
        # x ∈ ℝ^{*×D}, gate ∈ ℝ^{*×D}
        rms = x.pow(2).mean(-1, keepdim=True).add(eps).sqrt()
        x_norm = x / rms * weight
        return x_norm * silu(gate)        # 关键：乘以门控激活
```

```rust
// Rust 当前: LayerNorm(x)  ← 完全不同
// 正确:
fn rms_norm_gated(x: &Tensor, gate: &Tensor, weight: &Tensor, eps: f64) -> Tensor {
    let rms = (x.powf(2.0)?.mean(-1)? + eps)?.sqrt()?;
    let x_norm = x.broadcast_div(&rms)? * weight;
    let activated = Activation::Silu.forward(gate)?;
    x_norm * activated
}
```

---

## ⑥ DecoderLayer 结构（完整版）

```python
class Qwen3_5DecoderLayer:
    def __init__(self, config, layer_idx):
        layer_type = config.layer_types[layer_idx]
        if layer_type == "linear_attention":
            self.linear_attn = Qwen3_5GatedDeltaNet(config, layer_idx)
        elif layer_type == "full_attention":
            self.self_attn = Qwen3_5Attention(config, layer_idx)  # 标准RoPE attn
        self.mlp = Qwen3_5MLP(...)
        self.input_layernorm = RMSNorm(config.hidden_size)
        self.post_attention_layernorm = RMSNorm(config.hidden_size)

    def forward(self, x, ...):
        # Pre-Norm
        residual = x
        x = self.input_layernorm(x)
        if self.layer_type == "linear_attention":
            x = self.linear_attn(x, cache, mask)
        else:
            x, _ = self.self_attn(x, rope, mask, kv_cache)
        x = residual + x
        # MLP
        residual = x
        x = self.post_attention_layernorm(x)
        x = self.mlp(x)
        return residual + x
```

---

## 修正优先级建议

```
P0 (必须立刻改):  
  ② 投影结构 (in_proj_qkv)  
  ③ 门控公式 (dt_bias + A_log)
  ④ Gated Delta Rule (核心循环) 

P1 (必须改, 可稍后):  
  ⑤ RMSNormGated
  ⑥ layer_types 支持

P2 (复杂依赖):  
  ① causal_conv1d (可先用 silu(linear(x)) 跳过)

P3 (辅助):  
  ⑦ 标准 Attention 实现 (只在特定层使用)
```

---

## 你接下来需要做的事

1. **先确认 `config.json` 中 Qwen3.5-9B 的所有参数**
   - `linear_num_value_heads`, `linear_num_key_heads`
   - `linear_key_head_dim`, `linear_value_head_dim`
   - `linear_conv_kernel_dim`, `layer_types`

2. **获取 `flash-linear-attention` 库中 `recurrent_gated_delta_rule` 的纯 PyTorch 实现**
   ```bash
   git clone https://github.com/fla-org/flash-linear-attention
   # 找到 fla/ops/gated_delta_rule/recurrent_fuse.py
   ```

3. **逐行移植到 Rust**
   - 先从纯 torch 版开始（不要碰 CUDA kernel）
   - 用 candle 的 Tensor API 逐行翻译

4. **编译 + 单元测试**
   ```bash
   cargo test --features ai-local -p carptms_server -- qwen3_5
   ```
