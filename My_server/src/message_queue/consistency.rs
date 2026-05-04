use crate::message_queue::{MessageQueue, EventPublisher};
use serde::{Deserialize, DeserializeOwned, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono;

/// 分布式事务协调器
pub struct DistributedTransactionCoordinator {
    message_queue: Arc<dyn MessageQueue>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl DistributedTransactionCoordinator {
    /// 创建分布式事务协调器
    pub fn new(
        message_queue: Arc<dyn MessageQueue>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            message_queue,
            event_publisher,
        }
    }
    
    /// 开始分布式事务
    pub async fn begin_transaction(&self) -> anyhow::Result<TransactionContext> {
        let transaction_id = Uuid::new_v4().to_string();
        
        Ok(TransactionContext {
            transaction_id,
            steps: vec!(),
            status: TransactionStatus::Pending,
        })
    }
    
    /// 提交分布式事务
    pub async fn commit_transaction(&self, ctx: &mut TransactionContext) -> anyhow::Result<()> {
        // 1. 执行所有事务步骤
        for step in &mut ctx.steps {
            match step.status {
                StepStatus::Pending => {
                    // 执行步骤
                    step.status = StepStatus::Completed;
                }
                _ => continue,
            }
        }
        
        // 2. 发布事务提交事件
        let event = TransactionEvent {
            transaction_id: ctx.transaction_id.clone(),
            event_type: TransactionEventType::Commit,
            timestamp: chrono::Utc::now(),
        };
        
        self.event_publisher.publish_event(&event).await?;
        
        // 3. 更新事务状态
        ctx.status = TransactionStatus::Committed;
        
        Ok(())
    }
    
    /// 回滚分布式事务
    pub async fn rollback_transaction(&self, ctx: &mut TransactionContext) -> anyhow::Result<()> {
        // 1. 回滚所有已执行的步骤
        for step in ctx.steps.iter_mut().rev() {
            if step.status == StepStatus::Completed {
                // 回滚步骤
                step.status = StepStatus::RolledBack;
            }
        }
        
        // 2. 发布事务回滚事件
        let event = TransactionEvent {
            transaction_id: ctx.transaction_id.clone(),
            event_type: TransactionEventType::Rollback,
            timestamp: chrono::Utc::now(),
        };
        
        self.event_publisher.publish_event(&event).await?;
        
        // 3. 更新事务状态
        ctx.status = TransactionStatus::RolledBack;
        
        Ok(())
    }
}

/// 事务上下文
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionContext {
    pub transaction_id: String,
    pub steps: Vec<TransactionStep>,
    pub status: TransactionStatus,
}

/// 事务步骤
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionStep {
    pub id: String,
    pub service: String,
    pub operation: String,
    pub data: serde_json::Value,
    pub status: StepStatus,
}

/// 事务状态
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Committed,
    RolledBack,
    Failed,
}

/// 步骤状态
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Completed,
    RolledBack,
    Failed,
}

/// 事务事件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionEvent {
    pub transaction_id: String,
    pub event_type: TransactionEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 事务事件类型
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TransactionEventType {
    Begin,
    Commit,
    Rollback,
    Failed,
}

impl crate::domain::ddd::DomainEvent for TransactionEvent {
    fn event_type(&self) -> String {
        "transaction_event".to_string()
    }
    
    fn event_id(&self) -> String {
        self.transaction_id.clone()
    }
    
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }
}

/// 最终一致性管理器
pub struct EventuallyConsistentManager {
    message_queue: Arc<dyn MessageQueue>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl EventuallyConsistentManager {
    /// 创建最终一致性管理器
    pub fn new(
        message_queue: Arc<dyn MessageQueue>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            message_queue,
            event_publisher,
        }
    }
    
    /// 发布一致性事件
    pub async fn publish_consistency_event<T: Serialize>(
        &self, 
        event_type: &str, 
        data: &T,
    ) -> anyhow::Result<()> {
        self.message_queue.publish(event_type, data).await?;
        Ok(())
    }
    
    /// 处理一致性事件
    pub async fn process_consistency_event<T: DeserializeOwned>(
        &self, 
        event_type: &str,
        handler: impl Fn(T) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        let mut stream = self.message_queue.subscribe(event_type).await?;
        
        while let Ok(Some(message)) = stream.receive().await {
            if let Ok(event) = serde_json::from_slice::<T>(&message) {
                if let Err(e) = handler(event) {
                    eprintln!("Error processing consistency event: {:?}", e);
                }
            }
        }
        
        Ok(())
    }
}
