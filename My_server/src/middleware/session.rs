use actix_session::{Session, SessionExt};
use actix_web::{Error, HttpRequest};

// 会话管理中间件
pub async fn session_middleware(req: HttpRequest) -> Result<(), Error> {
    let session = req.get_session();
    // 检查会话是否存在
    if session.get::<String>("user_id").ok().is_none() {
        return Err(actix_web::error::ErrorUnauthorized("未认证"));
    }
    Ok(())
}

// 会话管理工具函数
pub fn get_user_id(session: &Session) -> Option<String> {
    session.get::<String>("user_id").unwrap_or(None)
}

pub fn set_user_id(session: &Session, user_id: &str) {
    session.insert("user_id", user_id).ok();
}

pub fn clear_session(session: &Session) {
    session.clear();
}
