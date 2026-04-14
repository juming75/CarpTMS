// 开发环境支持mock token
if token.starts_with("mock-token-") {
    info!("Using mock token for path: {}", path);
    
    // 创建模拟的claims
    let claims = crate::utils::jwt::Claims {
        sub: "1".to_string(),
        exp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as usize + 3600,
        iat: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as usize,
        iss: "tms_server".to_string(),
        role: "admin".to_string(),
        group_id: 1,
        token_type: "access".to_string()
    };
    
    // 直接允许访问，跳过细粒度权限检查
    req.extensions_mut().insert(claims);
    let res = service.call(req).await?;
    Ok(res)
}