//! / 配置中心模块测试

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_manager() {
        // 创建配置管理器
        let storage = Arc::new(MemoryConfigStorage::new(None));
        let watcher_manager = Arc::new(ConfigWatcherManager::new(Duration::from_secs(10)));
        let config_manager = ConfigManager::new(storage, watcher_manager, Duration::from_secs(300));

        // 启动配置管理器
        let mut config_manager = config_manager;
        config_manager.start();

        // 创建配置
        let params = ConfigCreateParams {
            namespace: "test",
            key: "test-key",
            value: ConfigValue::from_string("test-value"),
            description: Some("Test configuration".to_string()),
            tags: vec!["test", "development"],
            config_type: ConfigType::Custom,
            changed_by: Some("test-user".to_string()),
        };

        let created_config = config_manager.create_config(params).await.unwrap();
        assert_eq!(created_config.key.namespace, "test");
        assert_eq!(created_config.key.key, "test-key");
        assert_eq!(created_config.value.as_string().unwrap(), "test-value");

        // 获取配置
        let retrieved_config = config_manager.get_config("test", "test-key").await.unwrap();
        assert!(retrieved_config.is_some());
        let retrieved_config = retrieved_config.unwrap();
        assert_eq!(retrieved_config.key.namespace, "test");
        assert_eq!(retrieved_config.key.key, "test-key");
        assert_eq!(retrieved_config.value.as_string().unwrap(), "test-value");

        // 更新配置
        let update_params = ConfigUpdateParams {
            value: Some(ConfigValue::from_string("updated-value".to_string())),
            description: Some("Updated test configuration".to_string()),
            tags: Some(vec!["test", "updated"]),
            changed_by: Some("test-user".to_string()),
        };

        let updated_config = config_manager.update_config("test", "test-key", update_params).await.unwrap();
        assert_eq!(updated_config.value.as_string().unwrap(), "updated-value");
        assert_eq!(updated_config.description.unwrap(), "Updated test configuration".to_string());

        // 删除配置
        let deleted = config_manager.delete_config("test", "test-key").await.unwrap();
        assert!(deleted);

        // 验证配置已删除
        let retrieved_config = config_manager.get_config("test", "test-key").await.unwrap();
        assert!(retrieved_config.is_none());

        // 停止配置管理器
        config_manager.stop().await;
    }

    #[tokio::test]
    async fn test_config_storage() {
        // 测试内存存储
        let storage = MemoryConfigStorage::new(None);
        let storage = Arc::new(storage);

        // 创建配置条目
        let config_key = ConfigKey::new("test", "test-key");
        let config_entry = ConfigEntry {
            key: config_key,
            value: ConfigValue::from_string("test-value"),
            created_at: Instant::now(),
            updated_at: Instant::now(),
            description: Some("Test configuration".to_string()),
            tags: vec!["test"],
            version: "1.0.0".to_string(),
            status: ConfigStatus::Active,
            config_type: ConfigType::Custom,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };

        // 存储配置
        storage.store(&config_entry).unwrap();

        // 获取配置
        let retrieved_entry = storage.get(&config_entry.key).unwrap();
        assert!(retrieved_entry.is_some());
        let retrieved_entry = retrieved_entry.unwrap();
        assert_eq!(retrieved_entry.key.full_key(), config_entry.key.full_key());
        assert_eq!(retrieved_entry.value.as_string().unwrap(), "test-value");

        // 检查配置是否存在
        let exists = storage.exists(&config_entry.key).unwrap();
        assert!(exists);

        // 删除配置
        let deleted = storage.delete(&config_entry.key).unwrap();
        assert!(deleted);

        // 验证配置已删除
        let retrieved_entry = storage.get(&config_entry.key).unwrap();
        assert!(retrieved_entry.is_none());

        // 检查配置是否不存在
        let exists = storage.exists(&config_entry.key).unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_config_value() {
        // 测试字符串值
        let string_value = ConfigValue::from_string("test");
        assert_eq!(string_value.as_string().unwrap(), "test");

        // 测试整数值
        let integer_value = ConfigValue::from_integer(42);
        assert_eq!(integer_value.as_integer().unwrap(), 42);

        // 测试布尔值
        let boolean_value = ConfigValue::from_boolean(true);
        assert_eq!(boolean_value.as_boolean().unwrap(), true);

        // 测试浮点数
        let float_value = ConfigValue::from_float(3.14);
        assert_eq!(float_value.as_float().unwrap(), 3.14);

        // 测试JSON值
        let json_value = serde_json::json!({
            "name": "test",
            "value": 42
        });
        let config_json_value = ConfigValue::from_json(json_value.clone());
        assert_eq!(config_json_value.as_json().unwrap(), &json_value);
    }
}






