//! / 健康检查端点测试

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use actix_web::http::StatusCode;

    #[actix_web::test]
    async fn test_health_check_endpoint() {
        let mut app = test::init_service(
            App::new()
                .route("/health", actix_web::web::get().to(crate::health_check))
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let mut app = test::init_service(
            App::new()
                .route("/metrics", actix_web::web::get().to(crate::metrics_endpoint))
        ).await;

        let req = test::TestRequest::get()
            .uri("/metrics")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_health_check_response_body() {
        let mut app = test::init_service(
            App::new()
                .route("/health", actix_web::web::get().to(crate::health_check))
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        let body = test::read_body(resp).await;
        
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("\"status\":\"healthy\""));
    }
}






