//! AI 资源检测模块
//!
//! 自动检测系统硬件资源，为 AI 模块提供自适应能力：
//! - GPU VRAM 检测（NVIDIA CUDA / AMD ROCm）
//! - 系统内存检测
//! - Candle 模型加载能力评估（纯 Rust，适配所有平台包括 ARM/信创）
//!
//! 当资源不足时，AI 模块会静默降级或禁用

use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::System;

/// 硬件资源能力等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityLevel {
    /// 极低配：无 GPU，内存 < 8GB
    Minimal,
    /// 低配：GPU VRAM < 4GB 或 内存 < 16GB
    Low,
    /// 中配：GPU VRAM 4-8GB 或 内存 >= 16GB
    Medium,
    /// 高配：GPU VRAM 8-16GB
    High,
    /// 旗舰：GPU VRAM > 16GB
    Flagship,
}

impl CapabilityLevel {
    /// 获取支持的模型规模
    pub fn recommended_model_size(&self) -> &'static str {
        match self {
            CapabilityLevel::Minimal => "无 AI 模型支持（建议使用规则引擎）",
            CapabilityLevel::Low => "Qwen2.5-0.5B / Qwen2.5-1.5B (CPU推理)",
            CapabilityLevel::Medium => "Qwen2.5-3B (CPU) 或 Qwen2.5-7B (GPU量化)",
            CapabilityLevel::High => "Qwen2.5-7B (GPU) / Qwen2.5-14B (GPU量化)",
            CapabilityLevel::Flagship => "Qwen2.5-14B (GPU) / Qwen2.5-32B (GPU量化)",
        }
    }

    /// 获取最小推荐 VRAM (GB)
    pub fn min_vram_gb(&self) -> usize {
        match self {
            CapabilityLevel::Minimal => 0,
            CapabilityLevel::Low => 0,
            CapabilityLevel::Medium => 4,
            CapabilityLevel::High => 8,
            CapabilityLevel::Flagship => 16,
        }
    }
}

impl std::fmt::Display for CapabilityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityLevel::Minimal => write!(f, "极低配"),
            CapabilityLevel::Low => write!(f, "低配"),
            CapabilityLevel::Medium => write!(f, "中配"),
            CapabilityLevel::High => write!(f, "高配"),
            CapabilityLevel::Flagship => write!(f, "旗舰"),
        }
    }
}

/// GPU 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU 名称
    pub name: String,
    /// VRAM 大小 (MB)
    pub vram_mb: usize,
    /// VRAM 大小 (GB)
    pub vram_gb: f64,
    /// 支持 CUDA
    pub cuda_capable: bool,
    /// 支持 ROCm
    pub rocm_capable: bool,
}

impl GpuInfo {
    /// 获取推荐的量化位数
    pub fn recommended_quantization(&self) -> &'static str {
        if self.vram_gb >= 16.0 {
            "Q4_K_M"
        } else if self.vram_gb >= 8.0 {
            "Q5_K_M"
        } else if self.vram_gb >= 4.0 {
            "Q8_0"
        } else {
            "F16"
        }
    }
}

/// 系统资源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    /// 总系统内存 (MB)
    pub total_memory_mb: usize,
    /// 可用内存 (MB)
    pub available_memory_mb: usize,
    /// 系统内存 (GB)
    pub total_memory_gb: f64,
    /// 可用内存 (GB)
    pub available_memory_gb: f64,
    /// CPU 核心数
    pub cpu_cores: usize,
    /// GPU 信息列表
    pub gpus: Vec<GpuInfo>,
    /// 能力等级
    pub capability_level: CapabilityLevel,
    /// CUDA 是否可用
    pub cuda_available: bool,
    /// 是否支持本地 AI
    pub local_ai_supported: bool,
}

impl Default for SystemResources {
    fn default() -> Self {
        Self {
            total_memory_mb: 0,
            available_memory_mb: 0,
            total_memory_gb: 0.0,
            available_memory_gb: 0.0,
            cpu_cores: 0,
            gpus: Vec::new(),
            capability_level: CapabilityLevel::Minimal,
            cuda_available: false,
            local_ai_supported: false,
        }
    }
}

/// 资源检测器
pub struct ResourceDetector;

impl ResourceDetector {
    /// 检测完整系统资源
    pub fn detect() -> SystemResources {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_memory_mb = (sys.total_memory() / (1024 * 1024)) as usize;
        let available_memory_mb = (sys.available_memory() / (1024 * 1024)) as usize;
        let cpu_cores = sys.cpus().len();

        // 检测 GPU
        let gpus = Self::detect_gpus();
        let cuda_available = !gpus.is_empty() && gpus.iter().any(|g| g.cuda_capable);

        // 计算能力等级
        let capability_level = Self::calculate_capability(&gpus, total_memory_mb, cuda_available);

        // 判断是否支持本地 AI
        let local_ai_supported = capability_level != CapabilityLevel::Minimal;

        SystemResources {
            total_memory_mb,
            available_memory_mb,
            total_memory_gb: total_memory_mb as f64 / 1024.0,
            available_memory_gb: available_memory_mb as f64 / 1024.0,
            cpu_cores,
            gpus,
            capability_level,
            cuda_available,
            local_ai_supported,
        }
    }

    /// 检测 GPU 信息
    fn detect_gpus() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();

        // 尝试通过 nvidia-smi 检测 NVIDIA GPU
        if let Ok(output) = Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 2 {
                        let name = parts[0].to_string();
                        let vram_mb: usize = parts[1].parse().unwrap_or(0);
                        gpus.push(GpuInfo {
                            name,
                            vram_mb,
                            vram_gb: vram_mb as f64 / 1024.0,
                            cuda_capable: true,
                            rocm_capable: false,
                        });
                    }
                }
            }
        }

        // 尝试通过 rocm-smi 检测 AMD GPU
        if gpus.is_empty() {
            if let Ok(output) = Command::new("rocm-smi")
                .args(["--showid", "--showmeminfo", "vram"])
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // 解析 ROCm 输出（格式因版本而异）
                    if stdout.contains("GPU") {
                        gpus.push(GpuInfo {
                            name: "AMD GPU".to_string(),
                            vram_mb: 8192, // 默认值，实际应解析
                            vram_gb: 8.0,
                            cuda_capable: false,
                            rocm_capable: true,
                        });
                    }
                }
            }
        }

        gpus
    }

    /// 计算能力等级
    fn calculate_capability(
        gpus: &[GpuInfo],
        total_memory_mb: usize,
        _cuda_available: bool,
    ) -> CapabilityLevel {
        let total_memory_gb = total_memory_mb as f64 / 1024.0;

        // 有 GPU 的情况
        if !gpus.is_empty() {
            let max_vram_gb = gpus.iter().map(|g| g.vram_gb).fold(0.0, f64::max);

            if max_vram_gb >= 16.0 {
                return CapabilityLevel::Flagship;
            } else if max_vram_gb >= 8.0 {
                return CapabilityLevel::High;
            } else if max_vram_gb >= 4.0 {
                return CapabilityLevel::Medium;
            } else {
                return CapabilityLevel::Low;
            }
        }

        // 无 GPU 的情况
        if total_memory_gb >= 16.0 {
            CapabilityLevel::Medium
        } else if total_memory_gb >= 8.0 {
            CapabilityLevel::Low
        } else {
            CapabilityLevel::Minimal
        }
    }

    /// 检查模型是否可以在当前资源配置下运行
    pub fn can_load_model(model_vram_gb: f64, model_memory_gb: f64) -> bool {
        let resources = Self::detect();

        // 检查 VRAM
        let available_vram = resources
            .gpus
            .iter()
            .map(|g| g.vram_gb)
            .fold(0.0, |a, b| a + b);

        // 检查系统内存（预留 4GB 给系统）
        let available_sys_mem = resources.available_memory_gb - 4.0;

        available_vram >= model_vram_gb || available_sys_mem >= model_memory_gb
    }

    /// 获取推荐的推理后端
    pub fn recommended_backend() -> &'static str {
        let resources = Self::detect();

        if resources.cuda_available {
            "candle-cuda"
        } else if resources.capability_level == CapabilityLevel::Medium {
            "candle-cpu"
        } else if resources.capability_level == CapabilityLevel::Low {
            "candle-cpu-small"
        } else {
            "disabled"
        }
    }
}

/// AI 模块配置（基于检测结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModuleConfig {
    /// 是否启用本地 AI
    pub enabled: bool,
    /// 推荐的后端类型
    pub backend: String,
    /// 能力等级
    pub capability_level: CapabilityLevel,
    /// 推荐模型大小
    pub recommended_model: String,
    /// 推荐的量化
    pub recommended_quantization: String,
    /// 最大上下文长度
    pub max_context_length: usize,
    /// 推理线程数
    pub n_threads: usize,
}

impl Default for AiModuleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: "disabled".to_string(),
            capability_level: CapabilityLevel::Minimal,
            recommended_model: "none".to_string(),
            recommended_quantization: "none".to_string(),
            max_context_length: 0,
            n_threads: 4,
        }
    }
}

impl From<SystemResources> for AiModuleConfig {
    fn from(resources: SystemResources) -> Self {
        if !resources.local_ai_supported {
            return AiModuleConfig {
                enabled: false,
                backend: "disabled".to_string(),
                capability_level: resources.capability_level,
                ..Default::default()
            };
        }

        let (model, quant, ctx_len, threads) = match resources.capability_level {
            CapabilityLevel::Minimal => ("none", "none", 0, 4),
            CapabilityLevel::Low => ("qwen2.5-1.5b", "Q8_0", 2048, 4),
            CapabilityLevel::Medium => {
                if resources.cuda_available {
                    ("qwen2.5-7b", "Q4_K_M", 4096, 4)
                } else {
                    ("qwen2.5-3b", "Q8_0", 2048, 4)
                }
            }
            CapabilityLevel::High => {
                if resources.cuda_available {
                    ("qwen2.5-14b", "Q4_K_M", 4096, 4)
                } else {
                    ("qwen2.5-7b", "Q5_K_M", 2048, 4)
                }
            }
            CapabilityLevel::Flagship => ("qwen2.5-32b", "Q4_K_M", 8192, 8),
        };

        let backend = if resources.cuda_available {
            "candle-cuda"
        } else {
            "candle-cpu"
        };

        AiModuleConfig {
            enabled: true,
            backend: backend.to_string(),
            capability_level: resources.capability_level,
            recommended_model: model.to_string(),
            recommended_quantization: quant.to_string(),
            max_context_length: ctx_len,
            n_threads: threads,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_level_display() {
        assert_eq!(format!("{}", CapabilityLevel::Medium), "中配");
    }

    #[test]
    fn test_resource_detection() {
        let resources = ResourceDetector::detect();
        tracing::info!(resources = ?resources, "System Resources");
        assert!(resources.cpu_cores > 0);
    }

    #[test]
    fn test_ai_config_from_resources() {
        let resources = ResourceDetector::detect();
        let config = AiModuleConfig::from(resources);
        tracing::info!(config = ?config, "AI Config");
    }
}
