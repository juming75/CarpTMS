//! 地图服务相关路由
//!
//! 提供本地地图文件的元数据解析和访问功能

use actix_web::{web, HttpResponse};
use std::path::PathBuf;

/// 获取本地地图 GeoSet 元数据
///
/// # 功能说明
/// 解析 2014.gst 文件并返回图层信息、中心坐标、投影等元数据
#[utoipa::path(
    get,
    path = "/api/map/metadata",
    responses(
        (status = 200, description = "获取地图元数据成功", body = MapMetadataResponse),
        (status = 404, description = "地图文件不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "Map"
)]
pub async fn get_map_metadata() -> HttpResponse {
    // 构建地图文件路径（相对于项目根目录）
    let mut gst_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    gst_path.push("../../map/2014.gst");

    // 规范化路径
    if let Ok(canonical) = gst_path.canonicalize() {
        gst_path = canonical;
    }

    // 检查文件是否存在
    if !gst_path.exists() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "code": 404,
            "message": "地图文件不存在",
            "data": null
        }));
    }

    // 尝试读取和解析 GST 文件
    match parse_geoset_file(&gst_path) {
        Ok(metadata) => {
            log::info!("[Map] 成功加载 GeoSet 文件: {}", metadata.name);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 200,
                "message": "获取地图元数据成功",
                "data": metadata
            }))
        }
        Err(e) => {
            log::error!("[Map] 解析 GeoSet 文件失败: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "code": 500,
                "message": format!("解析地图文件失败: {}", e),
                "data": null
            }))
        }
    }
}

/// GeoSet 元数据响应结构
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MapMetadataResponse {
    name: String,
    projection: String,
    center: String,
    zoom_level: f64,
    layers: Vec<LayerInfo>,
}

/// 图层信息结构
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LayerInfo {
    id: u32,
    file: String,
    visible: bool,
    min_zoom: f64,
    max_zoom: f64,
    label: Option<String>,
}

/// 解析 MapInfo GeoSet (.gst) 文件
fn parse_geoset_file(path: &PathBuf) -> Result<MapMetadataResponse, String> {
    use std::fs::read_to_string;

    let content = read_to_string(path)
        .map_err(|e| format!("无法读取文件: {}", e))?;

    // 解析基本属性
    let name = extract_gst_value(&content, "\\GEOSET\\NAME")
        .unwrap_or_else(|| "2014年版中国地图".to_string());

    let projection = extract_gst_value(&content, "\\GEOSET\\PROJECTION")
        .unwrap_or_default();

    // 如果投影为空，默认使用 WGS84
    let projection = if projection.is_empty() {
        "WGS84".to_string()
    } else {
        projection
    };

    let center = extract_gst_value(&content, "\\GEOSET\\CENTER")
        .unwrap_or_else(|| "104.195,35.861".to_string());

    let zoom_level_str = extract_gst_value(&content, "\\GEOSET\\ZOOMLEVEL")
        .unwrap_or_else(|| "5182.24".to_string());

    let zoom_level: f64 = zoom_level_str.parse().unwrap_or(5182.24);

    // 将 MapInfo 缩放级别转换为 OpenLayers 缩放级别（近似转换）
    // MapInfo 使用地面单位/像素，OpenLayers 使用标准缩放级别
    let ol_zoom = mapinfo_zoom_to_ol_zoom(zoom_level);

    // 解析所有图层
    let layers = parse_layers_from_gst(&content);

    Ok(MapMetadataResponse {
        name,
        projection,
        center,
        zoom_level: ol_zoom,
        layers,
    })
}

/// 从 GST 内容中提取指定键的值
fn extract_gst_value(content: &str, key: &str) -> Option<String> {
    let pattern = format!("{} = \"", key);
    if let Some(start) = content.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = content[value_start..].find('"') {
            return Some(content[value_start..value_start + end].to_string());
        }
    }
    None
}

/// 从 GST 内容中解析所有图层信息
fn parse_layers_from_gst(content: &str) -> Vec<LayerInfo> {
    let mut layers = Vec::new();
    let mut layer_id = 1u32;

    // 查找所有 TABLE 条目
    let mut search_pos = 0;
    loop {
        let table_pattern = format!("\\TABLE\\{}\\FILE", layer_id);
        if let Some(pos) = content[search_pos..].find(&table_pattern) {
            let abs_pos = search_pos + pos;

            // 提取文件名
            let file_pattern = format!("\\TABLE\\{}\\FILE\" = \"", layer_id);
            if let Some(file_start) = content[abs_pos..].find(&file_pattern) {
                let value_start = abs_pos + file_start + file_pattern.len();
                if let Some(file_end) = content[value_start..].find('"') {
                    let file_name = content[value_start..value_start + file_end].to_string();

                    // 提取可见性
                    let visible_pattern = format!("\\TABLE\\{}\\ISVISIBLE\" = \"", layer_id);
                    let visible = if let Some(vis_start) = content[abs_pos..abs_pos+500].find(&visible_pattern) {
                        let vis_value_start = abs_pos + vis_start + visible_pattern.len();
                        if let Some(vis_end) = content[vis_value_start..vis_value_start+10].find('"') {
                            &content[vis_value_start..vis_value_start + vis_end] == "TRUE"
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    // 提取最小缩放
                    let min_zoom_pattern = format!("\\TABLE\\{}\\ZOOM\\MIN\" = \"", layer_id);
                    let min_zoom = if let Some(mz_start) = content[abs_pos..abs_pos+1000].find(&min_zoom_pattern) {
                        let mz_value_start = abs_pos + mz_start + min_zoom_pattern.len();
                        if let Some(mz_end) = content[mz_value_start..mz_value_start+20].find('"') {
                            content[mz_value_start..mz_value_start + mz_end]
                                .parse::<f64>()
                                .unwrap_or(0.0)
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };

                    // 提取最大缩放
                    let max_zoom_pattern = format!("\\TABLE\\{}\\ZOOM\\MAX\" = ", layer_id);
                    let max_zoom = if let Some(mz_start) = content[abs_pos..abs_pos+1200].find(&max_zoom_pattern) {
                        let mz_value_start = abs_pos + mz_start + max_zoom_pattern.len();
                        // 处理可能的引号或直接数值
                        let remaining = &content[mz_value_start..mz_value_start+30];
                        if let Some(mz_end) = remaining.find(|c: char| !c.is_numeric() && c != '.' && c != '-') {
                            remaining[..mz_end].parse::<f64>().unwrap_or(62137.12)
                        } else {
                            62137.12
                        }
                    } else {
                        62137.12
                    };

                    layers.push(LayerInfo {
                        id: layer_id,
                        file: file_name,
                        visible,
                        min_zoom,
                        max_zoom,
                        label: None,
                    });
                }
            }

            layer_id += 1;
            search_pos = abs_pos + table_pattern.len();
        } else {
            break;
        }
    }

    layers
}

/// 将 MapInfo 缩放级别转换为 OpenLayers 缩放级别
///
/// MapInfo 使用"每像素地面单位数"，值越大表示缩放得越远
/// OpenLayers 使用标准缩放级别（0-18左右）
fn mapinfo_zoom_to_ol_zoom(mapinfo_zoom: f64) -> f64 {
    // 近似转换公式（基于经验值）
    // MapInfo 5182.24 ≈ OL zoom 5 (全国视图)
    // MapInfo 310.686 ≈ OL zoom 8 (省级视图)
    // MapInfo 0 ≈ OL zoom 18 (街道级视图)

    if mapinfo_zoom <= 0.0 {
        18.0 // 最大缩放
    } else if mapinfo_zoom >= 10000.0 {
        3.0 // 最小缩放
    } else {
        // 对数转换，使用 clamp 确保范围在 2-18 之间
        let ol_zoom = 18.0 - (mapinfo_zoom.log10() * 3.5);
        ol_zoom.clamp(2.0, 18.0).round()
    }
}

/// 配置地图服务路由
pub fn configure_map_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/api/map/metadata", web::get().to(get_map_metadata));
}
