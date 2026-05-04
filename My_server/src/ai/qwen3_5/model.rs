//! Qwen3.5 完整模型

use candle_core::{Device, Module, Result, Tensor};
use candle_nn::{linear_no_bias, Embedding, Linear, VarBuilder};

use super::config::Qwen3_5Config;
use super::gated_delta::GatedDeltaCache;
use super::layers::Qwen3_5DecoderLayer;

pub struct Qwen3_5Model {
    embed_tokens: Embedding,
    layers: Vec<Qwen3_5DecoderLayer>,
    norm: candle_nn::LayerNorm,
    device: Device,
}

impl Qwen3_5Model {
    pub fn new(cfg: &Qwen3_5Config, vb: VarBuilder) -> Result<Self> {
        let vb_m = vb.pp("model");
        let embed = candle_nn::embedding(cfg.vocab_size, cfg.hidden_size, vb_m.pp("embed_tokens"))?;
        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        for i in 0..cfg.num_hidden_layers {
            layers.push(Qwen3_5DecoderLayer::new(cfg, i, vb_m.pp("layers").pp(i))?);
        }
        let norm = candle_nn::layer_norm(cfg.hidden_size, cfg.rms_norm_eps, vb_m.pp("norm"))?;
        Ok(Self {
            embed_tokens: embed,
            layers,
            norm,
            device: vb.device().clone(),
        })
    }

    pub fn forward(&self, input_ids: &Tensor) -> Result<Tensor> {
        self.forward_with_cache(input_ids, None)
    }

    /// 前向传播，支持可选的 KV-cache
    pub fn forward_with_cache(
        &self,
        input_ids: &Tensor,
        mut cache: Option<&mut GatedDeltaCache>,
    ) -> Result<Tensor> {
        let mut h = self.embed_tokens.forward(input_ids)?;
        for (i, layer) in self.layers.iter().enumerate() {
            h = layer.forward(&h, &mut cache, i)?;
        }
        self.norm.forward(&h)
    }
}

pub struct Qwen3_5ForCausalLM {
    model: Qwen3_5Model,
    lm_head: Linear,
}

impl Qwen3_5ForCausalLM {
    pub fn new(cfg: &Qwen3_5Config, vb: VarBuilder) -> Result<Self> {
        let model = Qwen3_5Model::new(cfg, vb.pp("model"))?;
        let lm_head = linear_no_bias(cfg.hidden_size, cfg.vocab_size, vb.pp("lm_head"))?;
        Ok(Self { model, lm_head })
    }

    pub fn forward(&self, input_ids: &Tensor) -> Result<Tensor> {
        let hidden = self.model.forward(input_ids)?;
        let last = hidden.narrow(1, hidden.dim(1)? - 1, 1)?;
        self.lm_head.forward(&last)
    }

    /// KV-cache 加速的生成方法。
    ///
    /// - Prefill 阶段：用完整 prompt 初始化缓存
    /// - Decode 阶段：每步只处理最后一个 token，缓存 CausalConv1d 状态和记忆矩阵 M
    pub fn generate(
        &self,
        input_ids: &Tensor,
        mut processor: candle_transformers::generation::LogitsProcessor,
        max_tokens: usize,
        eos_id: u32,
    ) -> Result<(Vec<u32>, Vec<Tensor>)> {
        let mut gen = Vec::new();
        let mut logits_list = Vec::new();
        let seq_len = input_ids.dim(1)?;

        // 初始化 KV-cache
        let num_layers = self.model.layers.len();
        let device = self.model.device.clone();
        let mut cache = GatedDeltaCache::new(num_layers);

        // 1. Prefill: 用完整 prompt 计算，初始化 M 矩阵
        let logits = self.model.forward_with_cache(input_ids, Some(&mut cache))?;
        let last = logits.narrow(1, seq_len - 1, 1)?;
        let lm = self.lm_head.forward(&last)?.squeeze(1)?;
        logits_list.push(lm.clone());
        let next = processor.sample(&lm)?;
        if next != eos_id {
            gen.push(next);
        }

        // 2. Decode: 每步只处理新 token（利用缓存），O(T) 而非 O(L²)
        for _ in 1..max_tokens {
            let next_t = Tensor::new(&[next], &device)?.unsqueeze(0)?; // [1, 1]
            let logits = self.model.forward_with_cache(&next_t, Some(&mut cache))?;
            let lm = self.lm_head.forward(&logits)?.squeeze(1)?;
            logits_list.push(lm.clone());
            let next = processor.sample(&lm)?;
            if next == eos_id {
                break;
            }
            gen.push(next);
        }

        Ok((gen, logits_list))
    }
}
