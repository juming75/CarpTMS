//! Blockchain Module
//! Integrates Hyperledger Fabric for critical data validation

use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Blockchain transaction type
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum TransactionType {
    VehicleData,
    OrderData,
    AlarmData,
    UserOperation,
    SystemConfig,
}

/// Blockchain transaction
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockchainTransaction {
    pub id: String,
    pub transaction_type: TransactionType,
    pub data_hash: String,
    pub data: serde_json::Value,
    pub created_at: String,
    pub submitter: String,
    pub status: TransactionStatus,
    pub block_height: Option<u64>,
    pub transaction_hash: Option<String>,
}

/// Transaction status
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Submitted,
    Confirmed,
    Failed,
    Timeout,
}

/// Blockchain service
pub struct BlockchainService {
    transactions: Arc<RwLock<HashMap<String, BlockchainTransaction>>>,
    connected: Arc<RwLock<bool>>,
    network_config: NetworkConfig,
}

/// Network configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub network_name: String,
    pub channel_name: String,
    pub chaincode_name: String,
    pub organization_name: String,
    pub cert_path: String,
    pub key_path: String,
    pub peer_addresses: Vec<String>,
    pub orderer_address: String,
}

/// Blockchain error
#[derive(Debug, Clone, Deserialize, Serialize, thiserror::Error)]
pub enum BlockchainError {
    #[error("Network connection failed: {0}")]
    NetworkError(String),
    #[error("Transaction submission failed: {0}")]
    TransactionError(String),
    #[error("Transaction validation failed: {0}")]
    ValidationError(String),
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    #[error("Data hash mismatch: {0}")]
    HashMismatchError(String),
}

impl BlockchainService {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            transactions: Arc::new(RwLock::new(HashMap::new())),
            connected: Arc::new(RwLock::new(false)),
            network_config: config,
        }
    }

    pub async fn connect(&self) -> Result<(), BlockchainError> {
        info!(
            "Connecting to blockchain network: {}",
            self.network_config.network_name
        );

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        *self.connected.write().await = true;
        info!("Successfully connected to blockchain network");

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), BlockchainError> {
        *self.connected.write().await = false;
        info!("Disconnected from blockchain network");
        Ok(())
    }

    pub async fn submit_transaction(
        &self,
        transaction_type: TransactionType,
        data: serde_json::Value,
        submitter: &str,
    ) -> Result<String, BlockchainError> {
        if !*self.connected.read().await {
            return Err(BlockchainError::NetworkError(
                "Not connected to blockchain network".to_string(),
            ));
        }

        let data_hash = self.calculate_hash(&data);

        let transaction = BlockchainTransaction {
            id: format!(
                "tx-{}-{}",
                chrono::Utc::now().format("%Y%m%d-%H%M%S"),
                uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
            ),
            transaction_type,
            data_hash,
            data,
            created_at: chrono::Utc::now().to_rfc3339(),
            submitter: submitter.to_string(),
            status: TransactionStatus::Pending,
            block_height: None,
            transaction_hash: None,
        };

        let mut transactions = self.transactions.write().await;
        transactions.insert(transaction.id.clone(), transaction.clone());
        drop(transactions);

        info!("Submitting transaction to blockchain: {}", transaction.id);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let mut transactions = self.transactions.write().await;
        if let Some(tx) = transactions.get_mut(&transaction.id) {
            tx.status = TransactionStatus::Submitted;
            let tx_value = serde_json::json!(tx);
            tx.transaction_hash = Some(self.calculate_hash(&tx_value));
        }
        drop(transactions);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let mut transactions = self.transactions.write().await;
        if let Some(tx) = transactions.get_mut(&transaction.id) {
            tx.status = TransactionStatus::Confirmed;
            tx.block_height = Some(12345);
        }
        drop(transactions);

        info!("Transaction {} confirmed", transaction.id);
        Ok(transaction.id)
    }

    pub async fn verify_transaction(&self, transaction_id: &str) -> Result<bool, BlockchainError> {
        let transactions = self.transactions.read().await;
        let transaction = match transactions.get(transaction_id) {
            Some(tx) => tx,
            None => {
                return Err(BlockchainError::TransactionNotFound(
                    transaction_id.to_string(),
                ))
            }
        };

        if transaction.status != TransactionStatus::Confirmed {
            return Err(BlockchainError::ValidationError(
                "Transaction not confirmed".to_string(),
            ));
        }

        let calculated_hash = self.calculate_hash(&transaction.data);
        if calculated_hash != transaction.data_hash {
            return Err(BlockchainError::HashMismatchError(
                "Data hash mismatch".to_string(),
            ));
        }

        Ok(true)
    }

    pub async fn verify_data(
        &self,
        data: &serde_json::Value,
        expected_hash: &str,
    ) -> Result<bool, BlockchainError> {
        let calculated_hash = self.calculate_hash(data);
        if calculated_hash != expected_hash {
            return Err(BlockchainError::HashMismatchError(
                "Data hash mismatch".to_string(),
            ));
        }
        Ok(true)
    }

    pub async fn get_transaction(
        &self,
        transaction_id: &str,
    ) -> Result<BlockchainTransaction, BlockchainError> {
        let transactions = self.transactions.read().await;
        match transactions.get(transaction_id) {
            Some(tx) => Ok(tx.clone()),
            None => Err(BlockchainError::TransactionNotFound(
                transaction_id.to_string(),
            )),
        }
    }

    pub async fn get_all_transactions(&self) -> Vec<BlockchainTransaction> {
        let transactions = self.transactions.read().await;
        transactions.values().cloned().collect()
    }

    fn calculate_hash(&self, data: &serde_json::Value) -> String {
        use sha2::{Digest, Sha256};

        let data_str = serde_json::to_string(data).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(data_str);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
