//! /! 依赖注入容器使用示例

use super::{Container, ContainerBuilder, Module};
use std::sync::Arc;

/// 示例服务
pub trait ExampleService: Send + Sync {
    fn get_message(&self) -> String;
}

/// 示例服务实现
pub struct ExampleServiceImpl {
    message: String,
}

impl ExampleServiceImpl {
    pub fn new(message: String) -> Self {
        Self {
            message,
        }
    }
}

impl ExampleService for ExampleServiceImpl {
    fn get_message(&self) -> String {
        self.message.clone()
    }
}

/// 示例模块
pub struct ExampleModule {
    message: String,
}

impl ExampleModule {
    pub fn new(message: String) -> Self {
        Self {
            message,
        }
    }
}

impl Module for ExampleModule {
    fn register(&self, container: &Container) {
        // 注册示例服务
        let message = self.message.clone();
        container.register_singleton(move |_| {
            Arc::new(ExampleServiceImpl::new(message.clone()))
        });
    }
}

/// 初始化示例依赖
pub fn init_example_dependencies() {
    // 创建容器构建器
    let builder = ContainerBuilder::new();
    
    // 注册示例模块
    let container = builder
        .register_module(ExampleModule::new("Hello from Example Service!".to_string()))
        .build();
    
    // 初始化全局容器
    super::init_global_container(container);
    
    println!("Example dependencies initialized");
}

/// 使用示例依赖
pub fn use_example_dependencies() {
    // 解析示例服务
    if let Some(service) = super::resolve::<ExampleServiceImpl>() {
        println!("Example service message: {}", service.get_message());
    } else {
        println!("Failed to resolve example service");
    }
}

/// 完整示例
pub async fn run_example() {
    println!("=== Dependency Injection Example ===");
    
    // 初始化示例依赖
    init_example_dependencies();
    
    // 使用示例依赖
    use_example_dependencies();
    
    println!("=== Example Complete ===");
}
