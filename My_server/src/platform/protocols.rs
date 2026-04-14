//! Protocol Management Module
//!
//! Provides unified protocol parsing and management for various vehicle communication protocols:
//! - JT808 (Chinese vehicle communication standard)
//! - JT1078 (Video protocol)
//! - GB/T 32960 (Electric vehicle monitoring)
//! - BSJ (Beijing standard)
//! - Custom protocols

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Protocol error types
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Unsupported protocol: {0}")]
    Unsupported(String),
    
    #[error("Protocol version mismatch: {0}")]
    VersionMismatch(String),
    
    #[error("Checksum error")]
    Checksum,
    
    #[error("Incomplete message")]
    Incomplete,
}

/// Protocol types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProtocolType {
    Jt808,
    Jt1078,
    Gb32960,
    Bsj,
    Db44,
    Custom(String),
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolType::Jt808 => write!(f, "JT808"),
            ProtocolType::Jt1078 => write!(f, "JT1078"),
            ProtocolType::Gb32960 => write!(f, "GB32960"),
            ProtocolType::Bsj => write!(f, "BSJ"),
            ProtocolType::Db44 => write!(f, "DB44"),
            ProtocolType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Message direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageDirection {
    Upstream,   // Device to server
    Downstream, // Server to device
}

/// Parsed message header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub protocol_type: ProtocolType,
    pub message_id: u16,
    pub message_body_length: u16,
    pub encryption_type: u8,
    pub device_id: String,
    pub message_sn: u16,
    pub version: Option<String>,
}

/// Parsed message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMessage {
    pub header: MessageHeader,
    pub body: Bytes,
    pub checksum: u8,
    pub direction: MessageDirection,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub raw_data: Bytes,
}

/// Protocol parser trait
#[async_trait]
pub trait ProtocolParser: Send + Sync {
    /// Parse message from bytes
    async fn parse(&self, data: &[u8]) -> Result<ParsedMessage, ProtocolError>;
    
    /// Serialize message to bytes
    async fn serialize(&self, message: &ParsedMessage) -> Result<Vec<u8>, ProtocolError>;
    
    /// Validate message
    async fn validate(&self, message: &ParsedMessage) -> Result<bool, ProtocolError>;
    
    /// Get supported protocol type
    fn protocol_type(&self) -> ProtocolType;
    
    /// Get protocol version
    fn version(&self) -> &str;
}

/// JT808 protocol parser
pub struct Jt808Parser {
    version: String,
}

impl Jt808Parser {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
        }
    }
}

#[async_trait]
impl ProtocolParser for Jt808Parser {
    async fn parse(&self, data: &[u8]) -> Result<ParsedMessage, ProtocolError> {
        if data.len() < 15 {
            return Err(ProtocolError::Incomplete);
        }
        
        // Basic JT808 parsing logic
        let mut index = 0;
        
        // Start flag (0x7E)
        if data[index] != 0x7E {
            return Err(ProtocolError::Parse("Invalid start flag".to_string()));
        }
        index += 1;
        
        // Message ID (2 bytes)
        let message_id = u16::from_be_bytes([data[index], data[index + 1]]);
        index += 2;
        
        // Message body length (2 bytes)
        let message_body_length = u16::from_be_bytes([data[index], data[index + 1]]);
        index += 2;
        
        // Encryption type (1 byte)
        let encryption_type = data[index];
        index += 1;
        
        // Device ID (6 bytes BCD)
        let device_id_bytes = &data[index..index + 6];
        let device_id = device_id_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<String>();
        index += 6;
        
        // Message SN (2 bytes)
        let message_sn = u16::from_be_bytes([data[index], data[index + 1]]);
        index += 2;
        
        // Message body
        let body_start = index;
        let body_end = index + message_body_length as usize;
        
        if body_end > data.len() - 2 {
            return Err(ProtocolError::Incomplete);
        }
        
        let body = Bytes::copy_from_slice(&data[body_start..body_end]);
        index = body_end;
        
        // Checksum (1 byte)
        let checksum = data[index];
        index += 1;
        
        // End flag (0x7E)
        if data[index] != 0x7E {
            return Err(ProtocolError::Parse("Invalid end flag".to_string()));
        }
        
        // Verify checksum
        let calculated_checksum = calculate_checksum(&data[1..body_end]);
        if calculated_checksum != checksum {
            return Err(ProtocolError::Checksum);
        }
        
        let header = MessageHeader {
            protocol_type: ProtocolType::Jt808,
            message_id,
            message_body_length,
            encryption_type,
            device_id,
            message_sn,
            version: Some(self.version.clone()),
        };
        
        Ok(ParsedMessage {
            header,
            body,
            checksum,
            direction: MessageDirection::Upstream,
            timestamp: chrono::Utc::now(),
            raw_data: Bytes::copy_from_slice(data),
        })
    }
    
    async fn serialize(&self, message: &ParsedMessage) -> Result<Vec<u8>, ProtocolError> {
        let mut result = Vec::new();
        
        // Start flag
        result.push(0x7E);
        
        // Message ID
        result.extend_from_slice(&message.header.message_id.to_be_bytes());
        
        // Message body length
        result.extend_from_slice(&message.header.message_body_length.to_be_bytes());
        
        // Encryption type
        result.push(message.header.encryption_type);
        
        // Device ID (convert from hex string to BCD)
        if message.header.device_id.len() == 12 {
            for i in 0..6 {
                let byte_str = &message.header.device_id[i * 2..i * 2 + 2];
                if let Ok(byte) = u8::from_str_radix(byte_str, 16) {
                    result.push(byte);
                } else {
                    return Err(ProtocolError::Validation("Invalid device ID format".to_string()));
                }
            }
        } else {
            return Err(ProtocolError::Validation("Invalid device ID length".to_string()));
        }
        
        // Message SN
        result.extend_from_slice(&message.header.message_sn.to_be_bytes());
        
        // Message body
        result.extend_from_slice(&message.body);
        
        // Calculate checksum
        let checksum = calculate_checksum(&result[1..]);
        result.push(checksum);
        
        // End flag
        result.push(0x7E);
        
        Ok(result)
    }
    
    async fn validate(&self, message: &ParsedMessage) -> Result<bool, ProtocolError> {
        // Basic validation
        if message.header.protocol_type != ProtocolType::Jt808 {
            return Ok(false);
        }
        
        if message.body.len() != message.header.message_body_length as usize {
            return Ok(false);
        }
        
        // Verify checksum
        let calculated_checksum = calculate_checksum(&message.raw_data[1..message.raw_data.len() - 2]);
        Ok(calculated_checksum == message.checksum)
    }
    
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Jt808
    }
    
    fn version(&self) -> &str {
        &self.version
    }
}

/// Protocol manager that coordinates all protocol parsers
pub struct ProtocolManager {
    parsers: Arc<RwLock<HashMap<ProtocolType, Arc<dyn ProtocolParser>>>>,
    default_protocol: ProtocolType,
}

impl ProtocolManager {
    /// Create a new protocol manager
    pub async fn new(config: Arc<crate::platform::config::ConfigManager>) -> Result<Self, ProtocolError> {
        let mut parsers: HashMap<ProtocolType, Arc<dyn ProtocolParser>> = HashMap::new();
        
        // Register default parsers
        parsers.insert(ProtocolType::Jt808, Arc::new(Jt808Parser::new()));
        
        // TODO: Add more parsers based on configuration
        
        Ok(Self {
            parsers: Arc::new(RwLock::new(parsers)),
            default_protocol: ProtocolType::Jt808,
        })
    }
    
    /// Parse message using appropriate parser
    pub async fn parse_message(&self, data: &[u8]) -> Result<ParsedMessage, ProtocolError> {
        // Try to detect protocol type from message
        let protocol_type = self.detect_protocol(data).await?;
        
        let parsers = self.parsers.read().await;
        if let Some(parser) = parsers.get(&protocol_type) {
            parser.parse(data).await
        } else {
            Err(ProtocolError::Unsupported(protocol_type.to_string()))
        }
    }
    
    /// Serialize message using appropriate parser
    pub async fn serialize_message(&self, message: &ParsedMessage) -> Result<Vec<u8>, ProtocolError> {
        let parsers = self.parsers.read().await;
        if let Some(parser) = parsers.get(&message.header.protocol_type) {
            parser.serialize(message).await
        } else {
            Err(ProtocolError::Unsupported(message.header.protocol_type.to_string()))
        }
    }
    
    /// Add a new protocol parser
    pub async fn add_parser(&self, parser: Arc<dyn ProtocolParser>) -> Result<(), ProtocolError> {
        let mut parsers = self.parsers.write().await;
        parsers.insert(parser.protocol_type(), parser);
        Ok(())
    }
    
    /// Remove a protocol parser
    pub async fn remove_parser(&self, protocol_type: &ProtocolType) -> Result<(), ProtocolError> {
        let mut parsers = self.parsers.write().await;
        parsers.remove(protocol_type);
        Ok(())
    }
    
    /// Get list of supported protocols
    pub async fn get_supported_protocols(&self) -> Vec<ProtocolType> {
        let parsers = self.parsers.read().await;
        parsers.keys().cloned().collect()
    }
    
    /// Detect protocol type from message data
    async fn detect_protocol(&self, data: &[u8]) -> Result<ProtocolType, ProtocolError> {
        if data.is_empty() {
            return Err(ProtocolError::Incomplete);
        }
        
        // Simple protocol detection based on start bytes
        match data[0] {
            0x7E => Ok(ProtocolType::Jt808), // JT808 start flag
            _ => Ok(self.default_protocol.clone()), // Default to JT808
        }
    }
}

/// Calculate checksum for JT808 protocol
fn calculate_checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &byte| acc.wrapping_add(byte))
}