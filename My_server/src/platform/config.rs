//! Unified Configuration Management
//!
//! Provides a unified interface for configuration management with support for:
//! - Environment variables
//! - Configuration files (JSON, YAML, TOML)
//! - Remote configuration (etcd, Consul)
//! - Hot reloading

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::interval;

/// Configuration error types
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Environment variable error: {0}")]
    Environment(String),
    
    #[error("Remote configuration error: {0}")]
    Remote(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Configuration key not found: {0}")]
    KeyNotFound(String),
}

/// Configuration provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigProviderType {
    Environment,
    File,
    Remote,
    Hybrid,
}

/// Configuration source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSource {
    pub provider: ConfigProviderType,
    pub path: Option<String>,
    pub url: Option<String>,
    pub format: ConfigFormat,
    pub reload_interval: Option<Duration>,
}

/// Configuration format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Env,
}

/// Configuration provider trait
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    /// Load configuration
    async fn load(&self) -> Result<HashMap<String, serde_json::Value>, ConfigError>;
    
    /// Watch for configuration changes
    async fn watch(&self) -> Result<(), ConfigError>;
    
    /// Get a specific configuration value
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, ConfigError>;
}

/// Environment configuration provider
pub struct EnvironmentConfigProvider {
    prefix: String,
}

impl EnvironmentConfigProvider {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }
}

#[async_trait]
impl ConfigProvider for EnvironmentConfigProvider {
    async fn load(&self) -> Result<HashMap<String, serde_json::Value>, ConfigError> {
        let mut config = HashMap::new();
        
        for (key, value) in std::env::vars() {
            if key.starts_with(&self.prefix) {
                let config_key = key[self.prefix.len()..].to_lowercase();
                config.insert(config_key, serde_json::Value::String(value));
            }
        }
        
        Ok(config)
    }
    
    async fn watch(&self) -> Result<(), ConfigError> {
        // Environment variables don't change at runtime in most cases
        Ok(())
    }
    
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, ConfigError> {
        let env_key = format!("{}{}", self.prefix, key.to_uppercase());
        match std::env::var(&env_key) {
            Ok(value) => Ok(Some(serde_json::Value::String(value))),
            Err(_) => Ok(None),
        }
    }
}

/// File configuration provider
pub struct FileConfigProvider {
    path: String,
    format: ConfigFormat,
}

impl FileConfigProvider {
    pub fn new(path: String, format: ConfigFormat) -> Self {
        Self { path, format }
    }
    
    fn parse_content(&self, content: &str) -> Result<HashMap<String, serde_json::Value>, ConfigError> {
        match self.format {
            ConfigFormat::Json => {
                serde_json::from_str(content)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))
            }
            ConfigFormat::Yaml => {
                serde_yaml::from_str(content)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))
            }
            ConfigFormat::Toml => {
                toml::from_str(content)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))
            }
            ConfigFormat::Env => {
                let mut config = HashMap::new();
                for line in content.lines() {
                    if let Some((key, value)) = line.split_once('=') {
                        config.insert(key.trim().to_string(), serde_json::Value::String(value.trim().to_string()));
                    }
                }
                Ok(config)
            }
        }
    }
}

#[async_trait]
impl ConfigProvider for FileConfigProvider {
    async fn load(&self) -> Result<HashMap<String, serde_json::Value>, ConfigError> {
        let content = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| ConfigError::FileNotFound(e.to_string()))?;
        
        self.parse_content(&content)
    }
    
    async fn watch(&self) -> Result<(), ConfigError> {
        // TODO: Implement file watching for hot reload
        Ok(())
    }
    
    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, ConfigError> {
        let config = self.load().await?;
        Ok(config.get(key).cloned())
    }
}

/// Configuration manager that provides unified access to configuration
pub struct ConfigManager {
    providers: Vec<Arc<dyn ConfigProvider>>,
    cache: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    reload_interval: Option<Duration>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub async fn new() -> Result<Self, ConfigError> {
        let mut providers: Vec<Arc<dyn ConfigProvider>> = vec![];
        
        // Add environment provider by default
        providers.push(Arc::new(EnvironmentConfigProvider::new("TMS_".to_string())));
        
        // Add file provider if config file exists
        let config_paths = vec![
            "config.json",
            "config.yaml",
            "config.yml",
            "config.toml",
            "config.env",
        ];
        
        for path in config_paths {
            if Path::new(path).exists() {
                let format = match path {
                    p if p.ends_with(".json") => ConfigFormat::Json,
                    p if p.ends_with(".yaml") || p.ends_with(".yml") => ConfigFormat::Yaml,
                    p if p.ends_with(".toml") => ConfigFormat::Toml,
                    p if p.ends_with(".env") => ConfigFormat::Env,
                    _ => continue,
                };
                
                providers.push(Arc::new(FileConfigProvider::new(path.to_string(), format)));
                break;
            }
        }
        
        let mut manager = Self {
            providers,
            cache: Arc::new(RwLock::new(HashMap::new())),
            reload_interval: None,
        };
        
        // Load initial configuration
        manager.reload().await?;
        
        // Start background reload task if interval is set
        if let Some(interval) = manager.reload_interval {
            let cache = manager.cache.clone();
            let providers = manager.providers.clone();
            
            tokio::spawn(async move {
                let mut ticker = interval(interval);
                loop {
                    ticker.tick().await;
                    if let Err(e) = reload_config(cache.clone(), providers.clone()).await {
                        tracing::error!("Failed to reload configuration: {}", e);
                    }
                }
            });
        }
        
        Ok(manager)
    }
    
    /// Get a configuration value by key
    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, ConfigError> {
        let cache = self.cache.read().await;
        
        if let Some(value) = cache.get(key) {
            serde_json::from_value(value.clone())
                .map_err(|e| ConfigError::ParseError(e.to_string()))
        } else {
            Err(ConfigError::KeyNotFound(key.to_string()))
        }
    }
    
    /// Get an optional configuration value
    pub async fn get_optional<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, ConfigError> {
        match self.get::<T>(key).await {
            Ok(value) => Ok(Some(value)),
            Err(ConfigError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    /// Get a configuration value with default
    pub async fn get_with_default<T: serde::de::DeserializeOwned + Default>(
        &self,
        key: &str,
    ) -> Result<T, ConfigError> {
        self.get_optional::<T>(key).await.map(|opt| opt.unwrap_or_default())
    }
    
    /// Reload configuration from all providers
    pub async fn reload(&mut self) -> Result<(), ConfigError> {
        let mut new_config = HashMap::new();
        
        // Load from all providers, later providers override earlier ones
        for provider in &self.providers {
            if let Ok(config) = provider.load().await {
                for (key, value) in config {
                    new_config.insert(key, value);
                }
            }
        }
        
        let mut cache = self.cache.write().await;
        *cache = new_config;
        
        Ok(())
    }
    
    /// Add a new configuration provider
    pub fn add_provider(&mut self, provider: Arc<dyn ConfigProvider>) {
        self.providers.push(provider);
    }
}

async fn reload_config(
    cache: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    providers: Vec<Arc<dyn ConfigProvider>>,
) -> Result<(), ConfigError> {
    let mut new_config = HashMap::new();
    
    for provider in providers {
        if let Ok(config) = provider.load().await {
            for (key, value) in config {
                new_config.insert(key, value);
            }
        }
    }
    
    let mut cache = cache.write().await;
    *cache = new_config;
    
    Ok(())
}