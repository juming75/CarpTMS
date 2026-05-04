//! 四维智能路由网关
//!
//! 1.5B（执行官）= 单点 + 简单指令
//! 9B（指挥官）= 区域 + 多车博弈 + 异常策略
//!
//! 四维分类：
//!   空间: 单点 vs 区域/群体
//!   复杂度: 检索 vs 逻辑推理
//!   时间: 即时 vs 长期规划
//!   场景: 上海2026 特殊规则（高峰/POI）
//!
//! 模式切换：
//!   Normal: 完整分类 + 投机执行
//!   VRAM_Constrained: 纯正则引擎（显存不足时）
//!
//! 编译模式：
//!   CPU 模式（默认）: cargo build
//!   GPU 模式: cargo build --features ai-gpu
//!   Full 模式: cargo build --features ai-full

#[cfg(feature = "ai")]
use regex::Regex;
#[cfg(feature = "ai")]
use serde::Serialize;
#[cfg(feature = "ai")]
use std::sync::Arc;
#[cfg(feature = "ai")]
use tokio::sync::RwLock;

#[cfg(feature = "ai")]
use super::qwen3_5::Qwen3_5Pipeline;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetModel {
    Direct,
    Light,
    Heavy,
    Escalate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteMode {
    Normal,
    VRAMConstrained,
}

#[derive(Debug, Serialize)]
pub struct RoutedResponse {
    pub content: String,
    pub model_used: &'static str,
    pub tokens_used: usize,
    pub is_truncated: bool,
}

#[cfg(feature = "ai")]
pub struct RouteEngine {
    light: Option<Arc<RwLock<Qwen3_5Pipeline>>>,
    heavy: Option<Arc<RwLock<Qwen3_5Pipeline>>>,
    mode: RouteMode,
    id_re: Regex,
    shanghai_poi_re: Regex,
    verb_get_re: Regex,
    verb_opt_re: Regex,
    region_re: Regex,
    group_re: Regex,
    emergency_re: Regex,
}

#[cfg(feature = "ai")]
impl RouteEngine {
    pub fn new(
        light: Option<Arc<RwLock<Qwen3_5Pipeline>>>,
        heavy: Option<Arc<RwLock<Qwen3_5Pipeline>>>,
        vram_limited: bool,
    ) -> Self {
        Self {
            light,
            heavy,
            mode: if vram_limited {
                RouteMode::VRAMConstrained
            } else {
                RouteMode::Normal
            },
            id_re: Regex::new(r"[A-Z]{1,2}\d{5,6}|car_\d+|driver_\d+|[沪京粤苏浙][A-Z]\d{5}")
                .expect("ID正则编译失败"),
            shanghai_poi_re: Regex::new(r"虹桥|陆家嘴|外滩|张江|金桥|浦东|徐汇|静安|内环|中环")
                .expect("上海POI正则编译失败"),
            verb_get_re: Regex::new(r"^(查|看|读|报|获取|读取|查询|get|status|check)")
                .expect("查询动词正则编译失败"),
            verb_opt_re: Regex::new(r"(分配|平衡|解决|规划|优化|调度|派单|怎么(走|办|分配))")
                .expect("优化动词正则编译失败"),
            region_re: Regex::new(r"(方圆|公里|区域|全区|商圈|范围)").expect("区域正则编译失败"),
            group_re: Regex::new(r"(所有|全部|全体|整个|每(一|个))").expect("群体正则编译失败"),
            emergency_re: Regex::new(r"(紧急|故障|死锁|暴雨|事故|警报|立刻|马上)")
                .expect("紧急正则编译失败"),
        }
    }

    fn route_vram_constrained(&self, text: &str) -> TargetModel {
        let t = text;
        if self.emergency_re.is_match(t) {
            log::warn!("紧急熔断（VRAM受限，无法处理高危）: {}", t);
            return TargetModel::Direct;
        }
        if self.group_re.is_match(t)
            || self.region_re.is_match(t)
            || self.shanghai_poi_re.is_match(t)
            || self.verb_opt_re.is_match(t)
        {
            log::info!("群体/复杂拦截（VRAM受限）");
            return TargetModel::Direct;
        }
        if self.id_re.is_match(t) {
            return TargetModel::Light;
        }
        if self.verb_get_re.is_match(t) {
            return TargetModel::Light;
        }
        TargetModel::Direct
    }

    fn classify(&self, text: &str) -> TargetModel {
        let t = text;
        // R01: 单体属性查询
        if self.id_re.is_match(t) && self.verb_get_re.is_match(t) {
            return TargetModel::Light;
        }
        // R04: 异常与应急
        if self.emergency_re.is_match(t) {
            return TargetModel::Heavy;
        }
        // R03: 约束优化 / R02: 区域 + POI / 群体
        if self.verb_opt_re.is_match(t) {
            return TargetModel::Heavy;
        }
        if self.group_re.is_match(t) {
            return TargetModel::Heavy;
        }
        if self.region_re.is_match(t) || self.shanghai_poi_re.is_match(t) {
            return TargetModel::Heavy;
        }
        // 默认给 1.5B
        TargetModel::Light
    }

    fn build_light_prompt(&self, raw: &str) -> String {
        let vid = self
            .id_re
            .find(raw)
            .map(|m| m.as_str())
            .unwrap_or("unknown");
        let intent = if raw.contains('电') {
            "电量"
        } else if raw.contains("位置") || raw.contains("在哪") {
            "位置/GPS"
        } else if raw.contains("状态") || raw.contains("故障") {
            "状态/故障码"
        } else {
            raw
        };
        format!(
            "你是一个车辆助手。用户查询车辆[{}]的[{}]。只回答数据，不要废话。",
            vid, intent
        )
    }

    async fn exec_light(&self, raw: &str) -> Result<RoutedResponse, String> {
        let prompt = self.build_light_prompt(raw);
        if let Some(ref l) = self.light {
            let mut p = l.write().await;
            p.set_max_tokens(128);
            match p.infer(&prompt) {
                Ok(r) if r.len() > 500 || r.contains("...") => {
                    log::info!("1.5B截断,转9B");
                    self.exec_heavy(raw).await
                }
                Ok(r) => Ok(RoutedResponse {
                    content: r,
                    model_used: "Qwen2.5-1.5B",
                    tokens_used: 128,
                    is_truncated: false,
                }),
                Err(e) => {
                    log::warn!("1.5B失败: {}", e);
                    self.exec_heavy(raw).await
                }
            }
        } else {
            self.exec_heavy(raw).await
        }
    }

    async fn exec_heavy(&self, raw: &str) -> Result<RoutedResponse, String> {
        if let Some(ref h) = self.heavy {
            let mut p = h.write().await;
            p.set_max_tokens(2048);
            match p.infer(raw) {
                Ok(r) => {
                    let len = r.len();
                    return Ok(RoutedResponse {
                        content: r,
                        model_used: "Qwen3.5-9B",
                        tokens_used: len,
                        is_truncated: false,
                    });
                }
                Err(e) => log::warn!("9B失败: {}", e),
            }
        }
        if let Some(ref l) = self.light {
            let mut p = l.write().await;
            p.set_max_tokens(1024);
            log::info!("9B不可用,1.5B兜底");
            match p.infer(&self.build_light_prompt(raw)) {
                Ok(r) => {
                    let len = r.len();
                    return Ok(RoutedResponse {
                        content: r,
                        model_used: "Qwen2.5-1.5B",
                        tokens_used: len,
                        is_truncated: false,
                    });
                }
                Err(e) => log::warn!("1.5B也失败: {}", e),
            }
        }
        Ok(RoutedResponse {
            content: "【规则引擎】AI全部不可用，系统正常运行。".into(),
            model_used: "rule-engine",
            tokens_used: 0,
            is_truncated: false,
        })
    }

    pub async fn route(&self, text: &str) -> Result<RoutedResponse, String> {
        let target = match self.mode {
            RouteMode::VRAMConstrained => self.route_vram_constrained(text),
            RouteMode::Normal => self.classify(text),
        };
        match target {
            TargetModel::Direct => Ok(RoutedResponse {
                content: format!(
                    "【规则引擎】请求已记录处理（{}模式）",
                    if self.mode == RouteMode::VRAMConstrained {
                        "VRAM受限"
                    } else {
                        "正常"
                    }
                ),
                model_used: "rule-engine",
                tokens_used: 0,
                is_truncated: false,
            }),
            TargetModel::Light => self.exec_light(text).await,
            TargetModel::Heavy | TargetModel::Escalate => self.exec_heavy(text).await,
        }
    }
}

#[cfg(feature = "ai")]
impl Qwen3_5Pipeline {
    pub fn set_max_tokens(&mut self, max: usize) {
        self.params.max_tokens = max;
    }
}

#[cfg(feature = "ai")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vram() {
        let e = RouteEngine::new(None, None, true);
        assert_eq!(e.route_vram_constrained("紧急！死锁"), TargetModel::Direct);
        assert_eq!(e.route_vram_constrained("所有车状态"), TargetModel::Direct);
        assert_eq!(
            e.route_vram_constrained("查沪A12345电量"),
            TargetModel::Light
        );
    }

    #[test]
    fn test_normal() {
        let e = RouteEngine::new(None, None, false);
        assert_eq!(e.classify("查沪A12345电量"), TargetModel::Light);
        assert_eq!(e.classify("浦东调度"), TargetModel::Heavy);
        assert_eq!(e.classify("优化派单"), TargetModel::Heavy);
        assert_eq!(e.classify("死锁"), TargetModel::Heavy);
    }
}
