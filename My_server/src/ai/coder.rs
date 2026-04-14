//! /! DeepSeek-Coder 集成模块
//! 提供代码生成、优化建议等开发辅助功能

use super::deepseek::{ChatMessage, DeepSeekClient};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// 代码生成请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeGenerateRequest {
    pub prompt: String,
    pub language: String,
    pub task: String,
    pub context: Option<String>,
}

/// 代码生成响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeGenerateResponse {
    pub code: String,
    pub explanation: String,
    pub suggestions: Vec<String>,
}

/// 数据库查询优化请求
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryOptimizeRequest {
    pub query: String,
    pub table_schema: Option<String>,
    pub performance_issues: Option<String>,
}

/// 数据库查询优化响应
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryOptimizeResponse {
    pub optimized_query: String,
    pub explanations: Vec<String>,
    pub execution_plan: Option<String>,
}

/// API 文档生成请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDocRequest {
    pub endpoint: String,
    pub method: String,
    pub request_schema: Option<String>,
    pub response_schema: Option<String>,
    pub description: Option<String>,
}

/// API 文档生成响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDocResponse {
    pub openapi: String,
    pub markdown: String,
    pub examples: Vec<String>,
}

/// 单元测试生成请求
#[derive(Debug, Serialize, Deserialize)]
pub struct TestGenerateRequest {
    pub code: String,
    pub language: String,
    pub test_framework: String,
    pub edge_cases: Option<Vec<String>>,
}

/// 单元测试生成响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TestGenerateResponse {
    pub tests: String,
    pub test_cases: Vec<String>,
    pub coverage_hints: Vec<String>,
}

/// DeepSeek-Coder 服务
#[derive(Debug, Clone)]
pub struct DeepSeekCoderService {
    client: DeepSeekClient,
}

impl DeepSeekCoderService {
    /// 创建新的 DeepSeek-Coder 服务
    pub fn new(client: DeepSeekClient) -> Self {
        Self { client }
    }

    /// 生成代码
    pub async fn generate_code(
        &self,
        request: &CodeGenerateRequest,
    ) -> Result<CodeGenerateResponse, Box<dyn Error>> {
        let prompt = format!(
            "你是一个专业的 Rust 后端开发工程师。请根据以下需求生成 {} 代码:\n\n任务:{}\n\n要求:\n1. 符合 {} 最佳实践\n2. 代码清晰、可维护\n3. 包含适当的注释\n4. 处理错误情况\n5. 提供使用示例\n\n上下文:{}\n\n生成代码:",
            request.language,
            request.task,
            request.language,
            request.context.as_deref().unwrap_or("无")
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.client.chat(&messages, 2000).await?;

        // 解析响应
        let (code, explanation, suggestions) = self.parse_code_response(&response);

        Ok(CodeGenerateResponse {
            code,
            explanation,
            suggestions,
        })
    }

    /// 优化数据库查询
    pub async fn optimize_query(
        &self,
        request: &QueryOptimizeRequest,
    ) -> Result<QueryOptimizeResponse, Box<dyn Error>> {
        let prompt = format!(
            "你是一个数据库优化专家。请优化以下 SQL 查询:\n\n原始查询:\n{}\n\n表结构:{}\n\n性能问题:{}\n\n要求:\n1. 提供优化后的查询\n2. 解释优化原因\n3. 分析执行计划\n4. 提供索引建议\n\n优化方案:",
            request.query,
            request.table_schema.as_deref().unwrap_or("未知"),
            request.performance_issues.as_deref().unwrap_or("无")
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.client.chat(&messages, 2000).await?;

        // 解析响应
        let (optimized_query, explanations, execution_plan) = self.parse_query_response(&response);

        Ok(QueryOptimizeResponse {
            optimized_query,
            explanations,
            execution_plan,
        })
    }

    /// 生成 API 文档
    pub async fn generate_api_doc(
        &self,
        request: &ApiDocRequest,
    ) -> Result<ApiDocResponse, Box<dyn Error>> {
        let prompt = format!(
            "你是一个 API 文档专家。请为以下 API 端点生成文档:\n\n端点:{}\n方法:{}\n\n请求结构:{}\n\n响应结构:{}\n\n描述:{}\n\n要求:\n1. 生成 OpenAPI 规范\n2. 生成 Markdown 文档\n3. 提供请求示例\n4. 提供响应示例\n\n文档:",
            request.endpoint,
            request.method,
            request.request_schema.as_deref().unwrap_or("无"),
            request.response_schema.as_deref().unwrap_or("无"),
            request.description.as_deref().unwrap_or("无")
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.client.chat(&messages, 2000).await?;

        // 解析响应
        let (openapi, markdown, examples) = self.parse_api_doc_response(&response);

        Ok(ApiDocResponse {
            openapi,
            markdown,
            examples,
        })
    }

    /// 生成单元测试
    pub async fn generate_tests(
        &self,
        request: &TestGenerateRequest,
    ) -> Result<TestGenerateResponse, Box<dyn Error>> {
        let prompt = format!(
            "你是一个测试专家。请为以下代码生成单元测试:\n\n代码:\n{}\n\n语言:{}\n测试框架:{}\n\n边界情况:{}\n\n要求:\n1. 覆盖主要功能\n2. 测试边界情况\n3. 提供测试数据\n4. 包含断言\n5. 遵循测试最佳实践\n\n测试代码:",
            request.code,
            request.language,
            request.test_framework,
            request.edge_cases.as_ref().map(|cases| cases.join("\n")).unwrap_or_else(|| "无".to_string())
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let response = self.client.chat(&messages, 2000).await?;

        // 解析响应
        let (tests, test_cases, coverage_hints) = self.parse_test_response(&response);

        Ok(TestGenerateResponse {
            tests,
            test_cases,
            coverage_hints,
        })
    }

    /// 解析代码响应
    fn parse_code_response(&self, response: &str) -> (String, String, Vec<String>) {
        // 简单解析,实际应用中可能需要更复杂的解析
        (response.to_string(), "代码生成完成".to_string(), vec![])
    }

    /// 解析查询响应
    fn parse_query_response(&self, response: &str) -> (String, Vec<String>, Option<String>) {
        (response.to_string(), vec!["查询优化完成".to_string()], None)
    }

    /// 解析 API 文档响应
    fn parse_api_doc_response(&self, response: &str) -> (String, String, Vec<String>) {
        (response.to_string(), response.to_string(), vec![])
    }

    /// 解析测试响应
    fn parse_test_response(&self, response: &str) -> (String, Vec<String>, Vec<String>) {
        (
            response.to_string(),
            vec!["测试生成完成".to_string()],
            vec![],
        )
    }
}
