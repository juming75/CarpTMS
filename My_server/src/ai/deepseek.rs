//! /! DeepSeek 模型集成模块
//! 统一管理与 DeepSeek 模型的通信

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;

/// DeepSeek API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepSeekConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout: u64,
    pub model: String,
}

impl Default for DeepSeekConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8000".to_string(),
            api_key: "your-api-key".to_string(),
            timeout: 30,
            model: "deepseek-coder:6.7b".to_string(),
        }
    }
}

/// DeepSeek API 客户端
#[derive(Debug, Clone)]
pub struct DeepSeekClient {
    client: Client,
    config: DeepSeekConfig,
}

impl DeepSeekClient {
    /// 创建新的 DeepSeek 客户端
    pub fn new(config: DeepSeekConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// 生成文本
    pub async fn generate(
        &self,
        prompt: &str,
        max_tokens: usize,
    ) -> Result<String, Box<dyn Error>> {
        let request = GenerateRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            max_tokens,
            temperature: 0.7,
            top_p: 0.95,
        };

        let response = self
            .client
            .post(format!("{}/v1/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await?
            .json::<GenerateResponse>()
            .await?;

        Ok(response.choices[0].text.clone())
    }

    /// 聊天完成
    pub async fn chat(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
    ) -> Result<String, Box<dyn Error>> {
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            max_tokens,
            temperature: 0.7,
            top_p: 0.95,
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;

        Ok(response.choices[0].message.content.clone())
    }
}

/// 生成请求
#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    max_tokens: usize,
    temperature: f64,
    top_p: f64,
}

/// 生成响应
#[derive(Debug, Deserialize)]
struct GenerateResponse {
    choices: Vec<GenerateChoice>,
}

/// 生成选项
#[derive(Debug, Deserialize)]
struct GenerateChoice {
    text: String,
}

/// 聊天消息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// 聊天请求
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: usize,
    temperature: f64,
    top_p: f64,
}

/// 聊天响应
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

/// 聊天选项
#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}
