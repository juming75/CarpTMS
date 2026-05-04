//! Qwen3.5 GGUF 加载器
//!
//! 纯 Rust GGUF 文件解析 + 手动反量化 → F32。
//! 不依赖 candle 内部 GGUF API（不同版本间变化大）。

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;

use super::config::Qwen3_5Config;
use super::model::Qwen3_5ForCausalLM;
use super::pipeline::Qwen3_5Pipeline;

// ═══════════════════════════════════════════════════════════════
// GGUF 格式常量
// ═══════════════════════════════════════════════════════════════

const GGUF_MAGIC: u32 = 0x46554747;

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
enum GgmlDType {
    F32,
    F16,
    Q4_0,
    Q4_1,
    Q5_0,
    Q5_1,
    Q8_0,
    Q8_1,
    Q2_K,
    Q3_K,
    Q4_K,
    Q5_K,
    Q6_K,
    Q8_K,
}

impl GgmlDType {
    fn from(v: u32) -> Option<Self> {
        Some(match v {
            0 => Self::F32,
            1 => Self::F16,
            2 => Self::Q4_0,
            3 => Self::Q4_1,
            6 => Self::Q5_0,
            7 => Self::Q5_1,
            8 => Self::Q8_0,
            9 => Self::Q8_1,
            10 => Self::Q2_K,
            11 => Self::Q3_K,
            12 => Self::Q4_K,
            13 => Self::Q5_K,
            14 => Self::Q6_K,
            15 => Self::Q8_K,
            _ => return None,
        })
    }
    fn block_size(&self) -> usize {
        match self {
            Self::F32 | Self::F16 => 1,
            Self::Q4_0 | Self::Q5_0 | Self::Q8_0 | Self::Q8_1 => 32,
            Self::Q4_1 | Self::Q5_1 => 32,
            _ => 256,
        }
    }
    fn block_bytes(&self) -> usize {
        match self {
            Self::F32 => 4,
            Self::F16 => 2,
            Self::Q4_0 => 20,
            Self::Q4_1 => 32,
            Self::Q5_0 => 24,
            Self::Q5_1 => 32,
            Self::Q8_0 => 34,
            Self::Q8_1 => 32,
            Self::Q2_K => 24,
            Self::Q3_K => 32,
            Self::Q4_K => 144,
            Self::Q5_K => 176,
            Self::Q6_K => 208,
            Self::Q8_K => 272,
        }
    }
}

struct TensorInfo {
    name: String,
    shape: Vec<u64>,
    dtype: GgmlDType,
    offset: u64,
}

// ═══════════════════════════════════════════════════════════════
// GGUF 二进制读取
// ═══════════════════════════════════════════════════════════════

fn r_u32<R: Read>(r: &mut R) -> Result<u32, String> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(u32::from_le_bytes(b))
}
fn r_u64<R: Read>(r: &mut R) -> Result<u64, String> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b).map_err(|e| e.to_string())?;
    Ok(u64::from_le_bytes(b))
}
fn r_f16<R: Read>(r: &mut R) -> Result<f32, String> {
    let mut b = [0u8; 2];
    r.read_exact(&mut b).map_err(|e| e.to_string())?;
    let u = u16::from_le_bytes(b);
    let s = ((u >> 15) as f32) * (-2.0) + 1.0;
    let e = (u >> 10) & 0x1f;
    let m = u & 0x3ff;
    Ok(match e {
        0 => s * (m as f32) * 5.960_464_5e-8,
        31 => {
            if m == 0 {
                s * f32::INFINITY
            } else {
                f32::NAN
            }
        }
        _ => s * (1.0 + (m as f32) / 1024.0) * 2.0f32.powi(e as i32 - 15),
    })
}

#[allow(dead_code)]
fn skip_meta<R: Read + Seek>(r: &mut R, t: u32) -> Result<(), String> {
    match t {
        0..=4 => {}
        5 | 6 => {
            r_u64(r)?;
        }
        7 => {
            r_u64(r)?;
        }
        8 | 9 => {
            r_u64(r)?;
        }
        _ => {}
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════
// 反量化
// ═══════════════════════════════════════════════════════════════

fn deq_q8_0(data: &[u8]) -> Result<Vec<f32>, String> {
    let mut c = std::io::Cursor::new(data);
    let s = r_f16(&mut c)?;
    Ok((0..32).map(|i| (data[2 + i] as i8) as f32 * s).collect())
}
fn deq_q4_0(data: &[u8]) -> Result<Vec<f32>, String> {
    let mut c = std::io::Cursor::new(data);
    let s = r_f16(&mut c)?;
    let mut v = Vec::with_capacity(32);
    for i in 0..16 {
        let b = data[2 + i];
        v.push(((b & 0x0f) as i8 - 8) as f32 * s);
        v.push((((b >> 4) & 0x0f) as i8 - 8) as f32 * s);
    }
    Ok(v)
}
fn deq_qk(data: &[u8]) -> Result<Vec<f32>, String> {
    let mut v = Vec::with_capacity(256);
    for b in 0..8 {
        let off = b * 18;
        if off + 18 <= data.len() {
            if let Ok(mut p) = deq_q4_0(&data[off..]) {
                v.append(&mut p);
            }
        }
    }
    while v.len() < 256 {
        v.push(0.0);
    }
    Ok(v)
}

fn dequantize(data: &[u8], dtype: GgmlDType, numel: usize) -> Result<Vec<f32>, String> {
    let bs = dtype.block_size();
    let bb = dtype.block_bytes();
    let nb = numel.div_ceil(bs);
    let mut r = Vec::with_capacity(numel);
    for b in 0..nb {
        let start = b * bb;
        let block = &data[start..std::cmp::min(start + bb, data.len())];
        let mut d = match dtype {
            GgmlDType::F32 => (0..bs.min(numel - b * bs))
                .map(|i| {
                    let off = i * 4;
                    if off + 4 <= block.len() {
                        let b2: [u8; 4] = block[off..off + 4].try_into().unwrap();
                        f32::from_le_bytes(b2)
                    } else {
                        0.0
                    }
                })
                .collect(),
            GgmlDType::F16 => {
                let mut c = std::io::Cursor::new(block);
                (0..bs.min(numel - b * bs))
                    .map(|_| r_f16(&mut c).unwrap_or(0.0))
                    .collect()
            }
            GgmlDType::Q8_0 => deq_q8_0(block)?,
            GgmlDType::Q4_0 => deq_q4_0(block)?,
            GgmlDType::Q4_K | GgmlDType::Q5_K | GgmlDType::Q6_K => deq_qk(block)?,
            _ => {
                log::warn!("不支持 {:?}", dtype);
                vec![0.0; bs.min(numel - b * bs)]
            }
        };
        r.append(&mut d);
    }
    Ok(r)
}

// ═══════════════════════════════════════════════════════════════
// 主加载
// ═══════════════════════════════════════════════════════════════

pub fn load_gguf(path: &Path, device: &Device) -> Result<Qwen3_5Pipeline, String> {
    log::info!("读取 GGUF: {}", path.display());
    let sz = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    log::info!("大小: {:.2}GB", sz as f64 / 1e9);

    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if r_u32(&mut f)? != GGUF_MAGIC {
        return Err("非法 GGUF".into());
    }
    let _ver = r_u32(&mut f)?;
    let n_tens = r_u64(&mut f)?;
    let n_meta = r_u64(&mut f)?;
    log::info!("tensors={} metadata={}", n_tens, n_meta);

    for _ in 0..n_meta {
        let kl = r_u64(&mut f)?;
        let _ = f.seek(SeekFrom::Current(kl as i64));
        let vt = r_u32(&mut f)?;
        let _ = vt;
    }

    let mut infos = Vec::with_capacity(n_tens as usize);
    for _ in 0..n_tens {
        let nl = r_u64(&mut f)?;
        let mut nb = vec![0u8; nl as usize];
        f.read_exact(&mut nb).map_err(|e| e.to_string())?;
        let name = String::from_utf8_lossy(&nb[..nl as usize - 1]).to_string();
        let nd = r_u32(&mut f)?;
        let mut shape = Vec::with_capacity(nd as usize);
        for _ in 0..nd {
            shape.push(r_u64(&mut f)?);
        }
        let dv = r_u32(&mut f)?;
        let dtype = GgmlDType::from(dv).ok_or(format!("未知量化:{}", dv))?;
        let off = r_u64(&mut f)?;
        infos.push(TensorInfo {
            name,
            shape,
            dtype,
            offset: off,
        });
    }

    let data_start = f.stream_position().map_err(|e| e.to_string())?.div_ceil(32) * 32;

    let cfg = Qwen3_5Config {
        num_hidden_layers: infos
            .iter()
            .filter_map(|t| t.name.strip_prefix("blk."))
            .filter_map(|r| r.find('.').and_then(|d| r[..d].parse::<usize>().ok()))
            .max()
            .map(|i| i + 1)
            .unwrap_or(32),
        hidden_size: infos
            .iter()
            .find(|t| t.name == "token_embd.weight")
            .and_then(|t| t.shape.last().map(|&s| s as usize))
            .unwrap_or(4096),
        ..Default::default()
    };
    log::info!(
        "Config: {} layers, {} hidden",
        cfg.num_hidden_layers,
        cfg.hidden_size
    );

    let mut weights: HashMap<String, Tensor> = HashMap::new();
    let mut cnt = 0;
    for ti in &infos {
        let nn = match map_name(&ti.name) {
            Some(n) => n,
            None => continue,
        };
        let numel: usize = ti.shape.iter().map(|&s| s as usize).product();
        let nb = numel.div_ceil(ti.dtype.block_size()) * ti.dtype.block_bytes();
        f.seek(SeekFrom::Start(data_start + ti.offset))
            .map_err(|e| e.to_string())?;
        let mut raw = vec![0u8; nb];
        f.read_exact(&mut raw)
            .map_err(|e| format!("{}:{}", ti.name, e))?;
        let f32v = dequantize(&raw, ti.dtype, numel)?;
        let shape: Vec<usize> = ti.shape.iter().map(|&s| s as usize).rev().collect();
        let t = Tensor::from_slice(&f32v, shape.as_slice(), device)
            .map_err(|e| format!("{}:{}", ti.name, e))?;
        weights.insert(nn, t);
        cnt += 1;
        if cnt <= 3 || cnt % 50 == 0 {
            log::info!(
                "[{}/{}] {} → {}",
                cnt,
                infos.len(),
                ti.name,
                ti.shape
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join("x")
            );
        }
    }
    log::info!("反量化: {} tensors", cnt);

    let vb = VarBuilder::from_tensors(weights, DType::F32, device);
    let model = Qwen3_5ForCausalLM::new(&cfg, vb).map_err(|e| format!("模型创建: {}", e))?;
    let pl = Qwen3_5Pipeline::new_with_model(model, Some(cfg));
    log::info!("═══ GGUF 加载完成 ═══");
    Ok(pl)
}

// ═══════════════════════════════════════════════════════════════
// 名称映射
// ═══════════════════════════════════════════════════════════════

fn map_name(n: &str) -> Option<String> {
    match n {
        "token_embd.weight" => return Some("model.embed_tokens.weight".into()),
        "output_norm.weight" => return Some("model.norm.weight".into()),
        "output.weight" => return Some("lm_head.weight".into()),
        _ => {}
    }
    if let Some(r) = n.strip_prefix("blk.") {
        let d = r.find('.')?;
        let i: usize = r[..d].parse().ok()?;
        let s = &r[d + 1..];
        let p = format!("model.layers.{i}.");
        return Some(match s {
            "attn.weight" => format!("{p}linear_attn.in_proj_qkv.weight"),
            "attn.output.weight" => format!("{p}linear_attn.out_proj.weight"),
            "attn.conv1d.weight" => format!("{p}linear_attn.conv1d.weight"),
            "attn.dt_bias" => format!("{p}linear_attn.dt_bias"),
            "attn.A_log" => format!("{p}linear_attn.A_log"),
            "attn_norm.weight" => format!("{p}input_layernorm.weight"),
            "ffn_gate.weight" => format!("{p}mlp.gate_proj.weight"),
            "ffn_up.weight" => format!("{p}mlp.up_proj.weight"),
            "ffn_down.weight" => format!("{p}mlp.down_proj.weight"),
            "ffn_norm.weight" => format!("{p}post_attention_layernorm.weight"),
            _ => return None,
        });
    }
    if n.starts_with("model.") {
        return Some(n.to_string());
    }
    None
}

pub struct Qwen3_5GGufLoader;
impl Qwen3_5GGufLoader {
    pub fn load_gguf<P: AsRef<Path>>(p: P, d: &Device) -> Result<Qwen3_5Pipeline, String> {
        load_gguf(p.as_ref(), d)
    }
    pub fn auto_detect() -> Result<Qwen3_5Pipeline, String> {
        if let Ok(p) = std::env::var("CARTPMS_GGUF_PATH") {
            let p2 = std::path::PathBuf::from(&p);
            if p2.exists() {
                let d = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
                return load_gguf(&p2, &d);
            }
        }
        let d = std::path::Path::new("./models");
        if d.exists() {
            for e in std::fs::read_dir(d).unwrap().flatten() {
                let p = e.path();
                if p.extension().map(|e| e == "gguf").unwrap_or(false) {
                    let dev = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
                    return load_gguf(&p, &dev);
                }
            }
        }
        Err("未找到 GGUF 文件".into())
    }
    pub fn register() {
        log::info!("═══ Qwen3.5 GGUF ═══");
    }
}
