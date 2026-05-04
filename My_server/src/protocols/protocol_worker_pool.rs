//! /! 协议解析线程池

use std::sync::{Arc, AtomicBool, RwLock};
use std::collections::HashMap;
use tokio::sync::mpsc;
use crate::protocols::jt808::models::Jt808Message;
use crate::errors::AppError;

/// 协议消息
#[derive(Debug, Clone)]
pub struct ProtocolMessage {
    pub vehicle_id: String,
    pub data: Vec<u8>,
    pub protocol_type: ProtocolType,
}

/// 协议类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    Jt808,
    Jt1078,
    Mqtt,
    Http,
}

/// 工作线程
struct Worker {
    receiver: mpsc::Receiver<ProtocolMessage>,
    shutdown: Arc<AtomicBool>,
    parser_cache: Arc<RwLock<HashMap<u16, Box<dyn MessageParser + Send + Sync>>>,
}

impl Worker {
    fn new(
        receiver: mpsc::Receiver<ProtocolMessage>,
        shutdown: Arc<AtomicBool>,
    ) -> Self {
        Self {
            receiver,
            shutdown,
            parser_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn run(mut self) {
        while !self.shutdown.load(std::sync::atomic::Ordering::SeqCst) {
            match self.receiver.recv().await {
                Some(msg) => {
                    if let Err(e) = self.process_message(msg).await {
                        tracing::error!("Worker processing error: {}", e);
                    }
                }
                None => break,
            }
        }
    }

    async fn process_message(&self, msg: ProtocolMessage) -> Result<(), AppError> {
        match msg.protocol_type {
            ProtocolType::Jt808 => self.process_jt808(msg).await,
            ProtocolType::Jt1078 => self.process_jt1078(msg).await,
            ProtocolType::Mqtt => self.process_mqtt(msg).await,
            ProtocolType::Http => self.process_http(msg).await,
        }
    }

    async fn process_jt808(&self, msg: ProtocolMessage) -> Result<(), AppError> {
        let parser = self.get_jt808_parser(&msg).await?;
        let message = parser.parse(&msg.data)?;
        
        crate::protocols::jt808::handler::handle_message(&msg.vehicle_id, message).await?;
        Ok(())
    }

    async fn get_jt808_parser(&self, msg: &ProtocolMessage) -> Result<Box<dyn MessageParser + Send + Sync>, AppError> {
        let msg_id = extract_msg_id(&msg.data)?;
        
        if let Some(parser) = self.parser_cache.read().unwrap().get(&msg_id) {
            return Ok(parser.clone());
        }
        
        let parser = crate::protocols::jt808::parser::create_parser(msg_id)?;
        self.parser_cache.write().unwrap().insert(msg_id, parser.clone());
        Ok(parser)
    }

    async fn process_jt1078(&self, _msg: ProtocolMessage) -> Result<(), AppError> {
        Ok(())
    }

    async fn process_mqtt(&self, _msg: ProtocolMessage) -> Result<(), AppError> {
        Ok(())
    }

    async fn process_http(&self, _msg: ProtocolMessage) -> Result<(), AppError> {
        Ok(())
    }
}

/// 消息解析器 trait
pub trait MessageParser {
    fn parse(&self, data: &[u8]) -> Result<Jt808Message, AppError>;
}

fn extract_msg_id(data: &[u8]) -> Result<u16, AppError> {
    if data.len() < 5 {
        return Err(AppError::validation_error("Invalid JT808 message"));
    }
    Ok(u16::from_be_bytes([data[3], data[4]]))
}

/// 协议工作线程池
pub struct ProtocolWorkerPool {
    workers: Vec<tokio::task::JoinHandle<()>>,
    sender: mpsc::Sender<ProtocolMessage>,
    shutdown: Arc<AtomicBool>,
}

impl ProtocolWorkerPool {
    /// 创建新的工作线程池
    pub fn new(num_workers: usize) -> Self {
        let (sender, receiver) = mpsc::channel(1000);
        let shutdown = Arc::new(AtomicBool::new(false));
        
        let mut workers = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            let worker = Worker::new(receiver.clone(), shutdown.clone());
            workers.push(tokio::spawn(worker.run()));
        }
        
        Self { workers, sender, shutdown }
    }

    /// 提交消息
    pub async fn submit(&self, message: ProtocolMessage) -> Result<(), AppError> {
        self.sender.send(message).await.map_err(|e| AppError::internal_error(&e.to_string(), None))
    }

    /// 关闭线程池
    pub async fn shutdown(&self) {
        self.shutdown.store(true, std::sync::atomic::Ordering::SeqCst);
        
        for worker in &self.workers {
            let _ = worker.await;
        }
    }
}

/// 全局协议线程池
static PROTOCOL_WORKER_POOL: once_cell::sync::Lazy<ProtocolWorkerPool> = once_cell::sync::Lazy::new(|| {
    let num_workers = std::env::var("PROTOCOL_WORKERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(num_cpus::get());
    
    ProtocolWorkerPool::new(num_workers)
});

pub fn get_protocol_worker_pool() -> &'static ProtocolWorkerPool {
    &PROTOCOL_WORKER_POOL
}
