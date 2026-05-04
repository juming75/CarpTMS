//! / 视频路由模块

use actix_web::{web, web::ServiceConfig};

/// 配置视频路由
pub fn configure_video_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/api/video")
            // 视频流路由
            .route("/streams", web::get().to(list_streams))
            .route("/streams/{stream_id}", web::get().to(get_stream_info))
            .route("/streams/{stream_id}/play", web::get().to(play_stream)),
    );
}

/// 列出所有视频流
async fn list_streams() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "streams": [],
        "total": 0
    }))
}

/// 获取视频流信息
async fn get_stream_info(path: web::Path<String>) -> impl actix_web::Responder {
    let stream_id = path.into_inner();
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "stream_id": stream_id,
        "status": "active"
    }))
}

/// 播放视频流
async fn play_stream(path: web::Path<String>) -> impl actix_web::Responder {
    let stream_id = path.into_inner();
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "stream_id": stream_id,
        "play_url": format!("/hls/{}.m3u8", stream_id)
    }))
}
