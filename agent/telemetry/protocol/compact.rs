// agent/telemetry/protocol/compact.rs

//! Compact binary protocol implementation for telemetry
//!
//! This module provides an optimized binary serialization format
//! for telemetry data with minimal overhead, designed for Lambda.

use std::io::{self, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use agent_core::startup::hardware_crc32;

/// Binary protocol value types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactValueType {
    Null = 0,
    Boolean = 1,
    Integer = 2,
    Integer64 = 3,
    Float = 4,
    String = 5,
    Timestamp = 6,
    Binary = 7,
    Array = 8,
    Map = 9,
}

/// Compact binary protocol value
#[derive(Debug, Clone)]
pub enum CompactValue {
    Null,
    Boolean(bool),
    Integer(i32),
    Integer64(i64),
    Float(f64),
    String(String),
    Timestamp(SystemTime),
    Binary(Vec<u8>),
    Array(Vec<CompactValue>),
    Map(Vec<(String, CompactValue)>),
}

impl CompactValue {
    /// Write value to the given writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            CompactValue::Null => {
                writer.write_all(&[CompactValueType::Null as u8])?;
            }
            CompactValue::Boolean(value) => {
                writer.write_all(&[CompactValueType::Boolean as u8])?;
                writer.write_all(&[if *value { 1 } else { 0 }])?;
            }
            CompactValue::Integer(value) => {
                writer.write_all(&[CompactValueType::Integer as u8])?;
                writer.write_all(&value.to_le_bytes())?;
            }
            CompactValue::Integer64(value) => {
                writer.write_all(&[CompactValueType::Integer64 as u8])?;
                
                // Use assembly-optimized fast_memcpy for enhanced performance on x86_64
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    use agent_core::startup::fast_memcpy;
                    let mut buffer = [0u8; 8];
                    fast_memcpy(buffer.as_mut_ptr(), value.to_le_bytes().as_ptr(), 8);
                    writer.write_all(&buffer)?;
                }
                
                #[cfg(not(target_arch = "x86_64"))]
                {
                    writer.write_all(&value.to_le_bytes())?;
                }
            }
            CompactValue::Float(value) => {
                writer.write_all(&[CompactValueType::Float as u8])?;
                writer.write_all(&value.to_le_bytes())?;
            }
            CompactValue::String(value) => {
                writer.write_all(&[CompactValueType::String as u8])?;
                let len = value.len() as u32;
                writer.write_all(&len.to_le_bytes())?;
                writer.write_all(value.as_bytes())?;
            }
            CompactValue::Timestamp(value) => {
                writer.write_all(&[CompactValueType::Timestamp as u8])?;
                let duration = value
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0));
                let millis = duration.as_millis() as u64;
                writer.write_all(&millis.to_le_bytes())?;
            }
            CompactValue::Binary(value) => {
                writer.write_all(&[CompactValueType::Binary as u8])?;
                let len = value.len() as u32;
                writer.write_all(&len.to_le_bytes())?;
                writer.write_all(value)?;
            }
            CompactValue::Array(values) => {
                writer.write_all(&[CompactValueType::Array as u8])?;
                let len = values.len() as u32;
                writer.write_all(&len.to_le_bytes())?;
                for value in values {
                    value.write_to(writer)?;
                }
            }
            CompactValue::Map(entries) => {
                writer.write_all(&[CompactValueType::Map as u8])?;
                let len = entries.len() as u32;
                writer.write_all(&len.to_le_bytes())?;
                for (key, value) in entries {
                    let key_len = key.len() as u16;
                    writer.write_all(&key_len.to_le_bytes())?;
                    writer.write_all(key.as_bytes())?;
                    value.write_to(writer)?;
                }
            }
        }
        Ok(())
    }
}

/// Compact metrics record format for efficient serialization
pub struct CompactMetricsRecord {
    /// Record ID
    pub id: String,
    /// Source of the metrics (e.g., "lambda", "extension")
    pub source: String,
    /// Timestamp of the record
    pub timestamp: SystemTime,
    /// Record type
    pub record_type: String,
    /// Metrics data
    pub metrics: Vec<(String, CompactValue)>,
}

impl CompactMetricsRecord {
    /// Create a new metrics record
    pub fn new(
        id: String,
        source: String,
        record_type: String,
        metrics: Vec<(String, CompactValue)>,
    ) -> Self {
        Self {
            id,
            source,
            timestamp: SystemTime::now(),
            record_type,
            metrics,
        }
    }
    
    /// Compute checksum using hardware acceleration when available
    pub fn compute_checksum(&self, data: &[u8]) -> u32 {
        // Use hardware-accelerated CRC32 for better performance
        hardware_crc32(data)
    }
    
    /// Serialize record to binary format
    pub fn serialize(&self) -> Vec<u8> {
        // Estimate buffer size for better performance
        let estimated_size = 
            4 + // Magic bytes
            8 + // Timestamp
            4 + // ID length + content
            self.id.len() +
            4 + // Source length + content
            self.source.len() +
            4 + // Type length + content
            self.record_type.len() +
            4 + // Metrics count
            self.metrics.len() * 20 + // Rough estimate per metric
            4; // Checksum
            
        let mut buffer = Vec::with_capacity(estimated_size);
        
        // Write magic bytes "CMTR"
        buffer.extend_from_slice(b"CMTR");
        
        // Write timestamp
        let timestamp = self.timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        buffer.extend_from_slice(&timestamp.to_le_bytes());
        
        // Write ID
        let id_len = self.id.len() as u32;
        buffer.extend_from_slice(&id_len.to_le_bytes());
        buffer.extend_from_slice(self.id.as_bytes());
        
        // Write source
        let source_len = self.source.len() as u32;
        buffer.extend_from_slice(&source_len.to_le_bytes());
        buffer.extend_from_slice(self.source.as_bytes());
        
        // Write record type
        let type_len = self.record_type.len() as u32;
        buffer.extend_from_slice(&type_len.to_le_bytes());
        buffer.extend_from_slice(self.record_type.as_bytes());
        
        // Write metrics
        let metrics_count = self.metrics.len() as u32;
        buffer.extend_from_slice(&metrics_count.to_le_bytes());
        
        for (key, value) in &self.metrics {
            let key_len = key.len() as u16;
            buffer.extend_from_slice(&key_len.to_le_bytes());
            buffer.extend_from_slice(key.as_bytes());
            
            // This part would be enhanced with a real implementation
            let mut value_buffer = Vec::new();
            if let Err(e) = value.write_to(&mut value_buffer) {
                // In production code, handle this error properly
                tracing::warn!("Failed to serialize metric value: {}", e);
                continue;
            }
            
            buffer.extend_from_slice(&value_buffer);
        }
        
        // Compute and add checksum - use hardware acceleration when available
        let checksum = self.compute_checksum(&buffer);
        buffer.extend_from_slice(&checksum.to_le_bytes());
        
        buffer
    }
}

/// Convert JSON value to CompactValue
pub fn json_to_compact_value(value: &serde_json::Value) -> CompactValue {
    match value {
        serde_json::Value::Null => CompactValue::Null,
        serde_json::Value::Bool(b) => CompactValue::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    CompactValue::Integer(i as i32)
                } else {
                    CompactValue::Integer64(i)
                }
            } else if let Some(f) = n.as_f64() {
                CompactValue::Float(f)
            } else {
                // Fallback
                CompactValue::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => {
            // Check if it looks like a timestamp
            if s.len() >= 20 && (s.contains('T') || s.contains('-')) {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(dt.timestamp() as u64);
                    return CompactValue::Timestamp(system_time);
                }
            }
            CompactValue::String(s.clone())
        }
        serde_json::Value::Array(arr) => {
            let values = arr.iter().map(json_to_compact_value).collect();
            CompactValue::Array(values)
        }
        serde_json::Value::Object(obj) => {
            let entries = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_compact_value(v)))
                .collect();
            CompactValue::Map(entries)
        }
    }
}

/// Convert JSON to a compact metrics record
pub fn json_to_compact_metrics(
    id: String,
    source: String,
    record_type: String,
    json: &serde_json::Value,
) -> Option<CompactMetricsRecord> {
    match json {
        serde_json::Value::Object(obj) => {
            let metrics = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_compact_value(v)))
                .collect();
                
            Some(CompactMetricsRecord::new(id, source, record_type, metrics))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_compact_value_serialization() {
        // Test various types
        let values = vec![
            CompactValue::Null,
            CompactValue::Boolean(true),
            CompactValue::Boolean(false),
            CompactValue::Integer(42),
            CompactValue::Integer(-42),
            CompactValue::Integer64(123456789012345),
            CompactValue::Float(3.14159),
            CompactValue::String("Hello, world!".to_string()),
            CompactValue::Timestamp(SystemTime::now()),
            CompactValue::Binary(vec![1, 2, 3, 4, 5]),
            CompactValue::Array(vec![
                CompactValue::Integer(1),
                CompactValue::Integer(2),
                CompactValue::Integer(3),
            ]),
            CompactValue::Map(vec![
                ("key1".to_string(), CompactValue::String("value1".to_string())),
                ("key2".to_string(), CompactValue::Integer(42)),
            ]),
        ];
        
        // Test each value
        for value in &values {
            let mut buffer = Vec::new();
            let result = value.write_to(&mut buffer);
            assert!(result.is_ok(), "Failed to serialize {:?}", value);
            
            // We can't easily deserialize and check in this simple test,
            // but we can verify the buffer is not empty
            assert!(!buffer.is_empty());
        }
    }
    
    #[test]
    fn test_compact_metrics_record() {
        let metrics = vec![
            ("cpu_usage".to_string(), CompactValue::Float(0.75)),
            ("memory_mb".to_string(), CompactValue::Integer(1024)),
            ("duration_ms".to_string(), CompactValue::Integer64(123456)),
        ];
        
        let record = CompactMetricsRecord::new(
            "test-1234".to_string(),
            "lambda".to_string(),
            "performance".to_string(),
            metrics,
        );
        
        let binary = record.serialize();
        
        // Validate the binary format
        assert!(!binary.is_empty());
        assert_eq!(&binary[0..4], b"CMTR"); // Magic bytes
        
        // Extract and validate the checksum
        let data_len = binary.len() - 4; // Last 4 bytes are checksum
        let computed_checksum = record.compute_checksum(&binary[0..data_len]);
        
        let mut checksum_bytes = [0u8; 4];
        checksum_bytes.copy_from_slice(&binary[data_len..]);
        let stored_checksum = u32::from_le_bytes(checksum_bytes);
        
        assert_eq!(computed_checksum, stored_checksum, "Checksum validation failed");
    }
    
    #[test]
    fn test_json_conversion() {
        let json = serde_json::json!({
            "string_value": "test",
            "int_value": 42,
            "float_value": 3.14,
            "bool_value": true,
            "null_value": null,
            "array_value": [1, 2, 3],
            "object_value": {
                "nested": "value"
            },
            "timestamp": "2023-01-01T12:00:00Z"
        });
        
        let compact_record = json_to_compact_metrics(
            "test-conversion".to_string(),
            "unit-test".to_string(),
            "json-conversion".to_string(),
            &json,
        );
        
        assert!(compact_record.is_some());
        let record = compact_record.unwrap();
        
        // Verify some fields
        assert_eq!(record.id, "test-conversion");
        assert_eq!(record.source, "unit-test");
        assert_eq!(record.record_type, "json-conversion");
        
        // Serialize to binary
        let binary = record.serialize();
        assert!(!binary.is_empty());
    }
}