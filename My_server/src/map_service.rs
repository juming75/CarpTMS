//! 本地地图文件服务
//! 提供 .gst/.MAP 地图文件的 HTTP 访问

use actix_files::NamedFile;
use actix_web::{web, HttpResponse};
use std::path::PathBuf;

const MAP_DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/map");

pub async fn list_map_files() -> HttpResponse {
    let dir = PathBuf::from(MAP_DATA_DIR);
    if !dir.exists() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "code": 404, "message": "地图数据目录不存在"
        }));
    }
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_size = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
            files.push(serde_json::json!({
                "name": path.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                "path": format!("/api/map/{}", path.file_name().and_then(|s| s.to_str()).unwrap_or("")),
                "size": file_size,
            }));
        }
    }
    HttpResponse::Ok().json(serde_json::json!({ "code": 200, "data": { "count": files.len(), "files": files } }))
}

pub async fn get_map_metadata() -> HttpResponse {
    let gst_path = PathBuf::from(MAP_DATA_DIR).join("2014.gst");
    if !gst_path.exists() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "code": 404, "message": "地图文件 2014.gst 不存在"
        }));
    }
    let metadata = match std::fs::metadata(&gst_path) {
        Ok(m) => m,
        Err(e) => return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "code": 500, "message": format!("读取失败: {}", e) })),
    };
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200, "data": {
            "name": "2014年版中国地图", "file": "2014.gst", "size": metadata.len(),
            "projection": "WGS84", "center": [104.195, 35.861], "zoomLevel": 5,
            "layers": [
                { "id": 1, "file": "省界.MAP", "label": "省级边界", "visible": true, "minZoom": 2, "maxZoom": 8 },
                { "id": 2, "file": "市界.MAP", "label": "市级边界", "visible": true, "minZoom": 6, "maxZoom": 10 },
                { "id": 3, "file": "县界.MAP", "label": "县级边界", "visible": true, "minZoom": 8, "maxZoom": 12 },
                { "id": 4, "file": "高速.MAP", "label": "高速公路", "visible": true, "minZoom": 4, "maxZoom": 14 },
                { "id": 5, "file": "国道.MAP", "label": "国道", "visible": true, "minZoom": 5, "maxZoom": 14 },
                { "id": 6, "file": "省道.MAP", "label": "省道", "visible": true, "minZoom": 6, "maxZoom": 14 },
                { "id": 7, "file": "县道.MAP", "label": "县道", "visible": true, "minZoom": 7, "maxZoom": 14 },
                { "id": 8, "file": "水系.MAP", "label": "水系", "visible": true, "minZoom": 3, "maxZoom": 12 },
                { "id": 9, "file": "绿地.MAP", "label": "绿地", "visible": true, "minZoom": 8, "maxZoom": 16 },
                { "id": 10, "file": "建成区界.MAP", "label": "建成区边界", "visible": true, "minZoom": 10, "maxZoom": 16 },
            ],
        }
    }))
}

pub async fn download_map_file(path: web::Path<String>) -> actix_web::Result<NamedFile> {
    let filename = path.into_inner();
    let file_path = PathBuf::from(MAP_DATA_DIR).join(&filename);
    let canonical = std::fs::canonicalize(&file_path)
        .map_err(|_| actix_web::error::ErrorNotFound("文件不存在"))?;
    let map_dir = std::fs::canonicalize(MAP_DATA_DIR).unwrap_or_else(|_| PathBuf::from(MAP_DATA_DIR));
    if !canonical.starts_with(&map_dir) {
        return Err(actix_web::error::ErrorForbidden("访问被拒绝"));
    }
    Ok(NamedFile::open(file_path)?)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/map")
            .route("", web::get().to(list_map_files))
            .route("/metadata", web::get().to(get_map_metadata))
            .route("/{filename}", web::get().to(download_map_file)),
    );
}
