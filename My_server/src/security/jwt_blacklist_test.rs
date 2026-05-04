//!
//! JWT 黑名单测试 (fail-close 行为验证)
//! 确保已撤销的令牌被正确拒绝

#[cfg(test)]
mod jwt_blacklist_tests {
    use crate::security::jwt::{JwtManager, JwtConfig};
    use std::time::{Duration, SystemTime};

    #[tokio::test]
    async fn test_blacklisted_token_rejected() {
        let config = JwtConfig {
            secret: "test_secret_key_at_least_64_characters_for_production_use_123456".to_string(),
            refresh_secret: "test_refresh_secret_at_least_64_characters_for_production_use".to_string(),
            expiration_hours: 24,
        };
        
        let jwt_manager = JwtManager::new(config);
        
        // 1. 创建正常令牌
        let user_id = 1i32;
        let claims = jwt_manager.create_claims(user_id, "user".to_string());
        let token = jwt_manager.create_token(&claims).unwrap();
        
        // 2. 验证令牌有效
        let validation = jwt_manager.validate_token(&token).await;
        assert!(validation.is_ok(), "新创建的令牌应该有效");
        
        // 3. 将令牌加入黑名单
        jwt_manager.add_to_blacklist(&token).await.unwrap();
        
        // 4. 验证黑名单中的令牌被拒绝 (fail-close)
        let validation = jwt_manager.validate_token(&token).await;
        assert!(validation.is_err(), "黑名单中的令牌必须被拒绝 (fail-close)");
        
        // 5. 清理
        jwt_manager.remove_from_blacklist(&token).await.unwrap();
    }

    #[tokio::test]
    async fn test_token_after_logout() {
        let config = JwtConfig {
            secret: "test_secret_key_at_least_64_characters_for_production_use_123456".to_string(),
            refresh_secret: "test_refresh_secret_at_least_64_characters_for_production_use".to_string(),
            expiration_hours: 24,
        };
        
        let jwt_manager = JwtManager::new(config);
        
        let user_id = 1i32;
        let claims = jwt_manager.create_claims(user_id, "user".to_string());
        let token = jwt_manager.create_token(&claims).unwrap();
        
        // 用户登出 -> 加入黑名单
        jwt_manager.logout(&token).await.unwrap();
        
        // 验证登出后令牌无效
        let result = jwt_manager.validate_token(&token).await;
        assert!(result.is_err(), "登出后令牌必须无效");
    }

    #[tokio::test]
    async fn test_refresh_token_after_rotation() {
        let config = JwtConfig {
            secret: "test_secret_key_at_least_64_characters_for_production_use_123456".to_string(),
            refresh_secret: "test_refresh_secret_at_least_64_characters_for_production_use".to_string(),
            expiration_hours: 24,
        };
        
        let jwt_manager = JwtManager::new(config);
        
        let user_id = 1i32;
        
        // 创建刷新令牌
        let refresh_token = jwt_manager.create_refresh_token(user_id).unwrap();
        
        // 验证刷新令牌有效
        let validation = jwt_manager.validate_refresh_token(&refresh_token).await;
        assert!(validation.is_ok());
        
        // 使用刷新令牌获取新令牌
        let new_tokens = jwt_manager.refresh_token(&refresh_token).await.unwrap();
        
        // 旧刷新令牌应该被加入黑名单
        let old_validation = jwt_manager.validate_refresh_token(&refresh_token).await;
        assert!(old_validation.is_err(), "刷新令牌轮换后旧令牌必须失效");
        
        // 新令牌应该有效
        let new_validation = jwt_manager.validate_token(&new_tokens.access_token).await;
        assert!(new_validation.is_ok(), "新令牌应该有效");
    }

    #[tokio::test]
    async fn test_concurrent_token_validation() {
        let config = JwtConfig {
            secret: "test_secret_key_at_least_64_characters_for_production_use_123456".to_string(),
            refresh_secret: "test_refresh_secret_at_least_64_characters_for_production_use".to_string(),
            expiration_hours: 24,
        };
        
        let jwt_manager = JwtManager::new(config);
        let user_id = 1i32;
        let claims = jwt_manager.create_claims(user_id, "user".to_string());
        let token = jwt_manager.create_token(&claims).unwrap();
        
        // 并发验证同一令牌
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let jwt_manager = jwt_manager.clone();
                let token = token.clone();
                tokio::spawn(async move {
                    jwt_manager.validate_token(&token).await
                })
            })
            .collect();
        
        let results = futures::future::join_all(handles).await;
        
        // 所有结果应该一致 (都有效)
        for result in results {
            assert!(result.unwrap().is_ok());
        }
        
        // 加入黑名单后
        jwt_manager.add_to_blacklist(&token).await.unwrap();
        
        // 再次并发验证
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let jwt_manager = jwt_manager.clone();
                let token = token.clone();
                tokio::spawn(async move {
                    jwt_manager.validate_token(&token).await
                })
            })
            .collect();
        
        let results = futures::future::join_all(handles).await;
        
        // 所有结果应该一致 (都无效 - fail-close)
        for result in results {
            assert!(result.unwrap().is_err(), "fail-close: 黑名单令牌必须全部拒绝");
        }
    }

    #[tokio::test]
    async fn test_blacklist_persistence() {
        let config = JwtConfig {
            secret: "test_secret_key_at_least_64_characters_for_production_use_123456".to_string(),
            refresh_secret: "test_refresh_secret_at_least_64_characters_for_production_use".to_string(),
            expiration_hours: 24,
        };
        
        let jwt_manager = JwtManager::new(config);
        let user_id = 1i32;
        let claims = jwt_manager.create_claims(user_id, "user".to_string());
        let token = jwt_manager.create_token(&claims).unwrap();
        
        // 添加到黑名单
        jwt_manager.add_to_blacklist(&token).await.unwrap();
        
        // 验证黑名单包含此令牌
        assert!(jwt_manager.is_blacklisted(&token).await.unwrap());
        
        // 移除黑名单
        jwt_manager.remove_from_blacklist(&token).await.unwrap();
        
        // 验证黑名单不再包含此令牌
        assert!(!jwt_manager.is_blacklisted(&token).await.unwrap());
        
        // 令牌重新有效
        let validation = jwt_manager.validate_token(&token).await;
        assert!(validation.is_ok());
    }
}
