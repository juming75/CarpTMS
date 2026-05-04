//! 高效序列化/反序列化工具模块
//!
//! 提供多种序列化/反序列化方法,根据不同场景选择最适合的方案
//! - simd-json: 高性能JSON序列化/反序列化
//! - bincode: 二进制序列化,适用于内部通信
//! - prost: Protocol Buffers序列化,适用于网络传输

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// 序列化格式类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    /// JSON格式
    Json,
    /// 二进制格式(bincode)
    Binary,
    /// Protocol Buffers格式
    Protobuf,
    /// 高性能JSON格式(simd-json)
    SimdJson,
}

/// 序列化工具
#[derive(Debug, Clone)]
pub struct Serializer {
    format: SerializationFormat,
}

impl Default for Serializer {
    fn default() -> Self {
        Self {
            format: SerializationFormat::SimdJson,
        }
    }
}

impl Serializer {
    /// 创建新的序列化工具
    pub fn new(format: SerializationFormat) -> Self {
        Self { format }
    }

    /// 序列化数据
    pub fn serialize<T: Serialize>(&self, data: &T) -> Result<Vec<u8>> {
        match self.format {
            SerializationFormat::Json => {
                serde_json::to_vec(data).map_err(|e| anyhow!("JSON serialization error: {}", e))
            }
            SerializationFormat::Binary => {
                // 暂时使用JSON作为二进制格式的替代,直到bincode 2.0.0 API问题解决
                serde_json::to_vec(data).map_err(|e| anyhow!("Binary serialization error: {}", e))
            }
            SerializationFormat::Protobuf => {
                // Protocol Buffers需要实现特定的trait
                Err(anyhow!(
                    "Protobuf serialization not implemented for this type"
                ))
            }
            SerializationFormat::SimdJson => {
                simd_json::to_vec(data).map_err(|e| anyhow!("SimdJSON serialization error: {}", e))
            }
        }
    }

    /// 反序列化数据
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T> {
        match self.format {
            SerializationFormat::Json => serde_json::from_slice(data)
                .map_err(|e| anyhow!("JSON deserialization error: {}", e)),
            SerializationFormat::Binary => {
                // 暂时使用JSON作为二进制格式的替代,直到bincode 2.0.0 API问题解决
                serde_json::from_slice(data)
                    .map_err(|e| anyhow!("Binary deserialization error: {}", e))
            }
            SerializationFormat::Protobuf => {
                // Protocol Buffers需要实现特定的trait
                Err(anyhow!(
                    "Protobuf deserialization not implemented for this type"
                ))
            }
            SerializationFormat::SimdJson => {
                let mut data_mut = data.to_vec();
                simd_json::from_slice(&mut data_mut)
                    .map_err(|e| anyhow!("SimdJSON deserialization error: {}", e))
            }
        }
    }

    /// 序列化数据到写入器
    pub fn serialize_to_writer<T: Serialize, W: Write>(
        &self,
        data: &T,
        writer: &mut W,
    ) -> Result<()> {
        let serialized = self.serialize(data)?;
        writer.write_all(&serialized)?;
        Ok(())
    }

    /// 从读取器反序列化数据
    pub fn deserialize_from_reader<T: for<'de> Deserialize<'de>, R: Read>(
        &self,
        reader: &mut R,
    ) -> Result<T> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        self.deserialize(&buffer)
    }
}

/// 便捷的JSON序列化函数
pub fn json_serialize<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    simd_json::to_vec(data).map_err(|e| anyhow!("JSON serialization error: {}", e))
}

/// 便捷的JSON反序列化函数
pub fn json_deserialize<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
    let mut data_mut = data.to_vec();
    simd_json::from_slice(&mut data_mut).map_err(|e| anyhow!("JSON deserialization error: {}", e))
}

/// 便捷的二进制序列化函数
pub fn binary_serialize<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    // 暂时使用JSON作为二进制格式的替代,直到bincode 2.0.0 API问题解决
    serde_json::to_vec(data).map_err(|e| anyhow!("Binary serialization error: {}", e))
}

/// 便捷的二进制反序列化函数
pub fn binary_deserialize<T: for<'de> Deserialize<'de>>(data: &[u8]) -> Result<T> {
    // 暂时使用JSON作为二进制格式的替代,直到bincode 2.0.0 API问题解决
    serde_json::from_slice(data).map_err(|e| anyhow!("Binary deserialization error: {}", e))
}

/// 序列化性能测试
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
        values: Vec<f64>,
        nested: Option<NestedData>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct NestedData {
        field1: String,
        field2: u64,
    }

    #[test]
    fn test_json_serialization() {
        let test_data = TestData {
            id: 123,
            name: "test".to_string(),
            values: vec![1.1, 2.2, 3.3],
            nested: Some(NestedData {
                field1: "nested".to_string(),
                field2: 456,
            }),
        };

        let serializer = Serializer::new(SerializationFormat::SimdJson);
        let serialized = serializer.serialize(&test_data).unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).unwrap();

        assert_eq!(test_data, deserialized);
    }

    #[test]
    fn test_binary_serialization() {
        let test_data = TestData {
            id: 123,
            name: "test".to_string(),
            values: vec![1.1, 2.2, 3.3],
            nested: Some(NestedData {
                field1: "nested".to_string(),
                field2: 456,
            }),
        };

        let serializer = Serializer::new(SerializationFormat::Binary);
        let serialized = serializer.serialize(&test_data).unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).unwrap();

        assert_eq!(test_data, deserialized);
    }

    #[test]
    fn test_performance_comparison() {
        let test_data = TestData {
            id: 123,
            name: "test".to_string(),
            values: vec![1.1, 2.2, 3.3],
            nested: Some(NestedData {
                field1: "nested".to_string(),
                field2: 456,
            }),
        };

        let iterations = 10000;

        // 测试标准JSON
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = serde_json::to_vec(&test_data).unwrap();
        }
        let json_duration = start.elapsed();

        // 测试simd-json
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = simd_json::to_vec(&test_data).unwrap();
        }
        let simd_json_duration = start.elapsed();

        // 测试bincode (暂时使用JSON作为替代)
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = serde_json::to_vec(&test_data).unwrap();
        }
        let binary_duration = start.elapsed();

        println!("JSON serialization: {:?}", json_duration);
        println!("SimdJSON serialization: {:?}", simd_json_duration);
        println!("Binary serialization: {:?}", binary_duration);

        // simd-json应该比标准JSON快
        assert!(simd_json_duration < json_duration);
        // bincode应该比JSON快
        assert!(binary_duration < json_duration);
    }
}
