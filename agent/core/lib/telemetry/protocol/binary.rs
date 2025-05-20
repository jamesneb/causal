// agent/core/lib/telemetry/protocol/binary.rs

use anyhow::{Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{Read, Write};
use uuid::Uuid;

// Protocol constants
pub const PROTOCOL_MAGIC: &[u8; 4] = b"PRBM";
pub const PROTOCOL_VERSION: u32 = 1;
pub const PROTOCOL_MIN_COMPRESSION_SIZE: usize = 1024; // Only compress if larger than 1KB

// Value types for binary protocol
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ValueType {
    Null = 0,
    Bool = 1,
    Int8 = 2,
    Int16 = 3,
    Int32 = 4,
    Int64 = 5,
    Float32 = 6,
    Float64 = 7,
    String8 = 8,
    String16 = 9,
    List = 10,
    Map = 11,
    Binary8 = 12,
    Binary16 = 13,
    Timestamp = 14,
    UUID = 15,
}

impl From<u8> for ValueType {
    fn from(value: u8) -> Self {
        match value {
            1 => ValueType::Bool,
            2 => ValueType::Int8,
            3 => ValueType::Int16,
            4 => ValueType::Int32,
            5 => ValueType::Int64,
            6 => ValueType::Float32,
            7 => ValueType::Float64,
            8 => ValueType::String8,
            9 => ValueType::String16,
            10 => ValueType::List,
            11 => ValueType::Map,
            12 => ValueType::Binary8,
            13 => ValueType::Binary16,
            14 => ValueType::Timestamp,
            15 => ValueType::UUID,
            _ => ValueType::Null,
        }
    }
}

// Define metric entry type
#[derive(Clone)]
pub struct MetricEntry {
    pub request_id: String,
    pub metrics: Value,
    pub timestamp: DateTime<Utc>,
}

impl heapless::pool::Init for MetricEntry {
    fn init() -> Self {
        Self {
            request_id: String::with_capacity(36), // UUID size
            metrics: json!({}),
            timestamp: Utc::now(),
        }
    }
}

// Encode a JSON value in binary format
pub fn encode_value(value: &Value, buffer: &mut Vec<u8>) {
    match value {
        Value::Null => {
            buffer.push(ValueType::Null as u8);
        }
        Value::Bool(b) => {
            buffer.push(ValueType::Bool as u8);
            buffer.push(if *b { 1 } else { 0 });
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= i8::MIN as i64 && i <= i8::MAX as i64 {
                    buffer.push(ValueType::Int8 as u8);
                    buffer.push(i as u8);
                } else if i >= i16::MIN as i64 && i <= i16::MAX as i64 {
                    buffer.push(ValueType::Int16 as u8);
                    buffer.extend_from_slice(&(i as i16).to_le_bytes());
                } else if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    buffer.push(ValueType::Int32 as u8);
                    buffer.extend_from_slice(&(i as i32).to_le_bytes());
                } else {
                    buffer.push(ValueType::Int64 as u8);
                    buffer.extend_from_slice(&i.to_le_bytes());
                }
            } else if let Some(f) = n.as_f64() {
                if (f as f32) as f64 == f {
                    buffer.push(ValueType::Float32 as u8);
                    buffer.extend_from_slice(&(f as f32).to_le_bytes());
                } else {
                    buffer.push(ValueType::Float64 as u8);
                    buffer.extend_from_slice(&f.to_le_bytes());
                }
            }
        }
        Value::String(s) => {
            let bytes = s.as_bytes();
            if bytes.len() <= 255 {
                buffer.push(ValueType::String8 as u8);
                buffer.push(bytes.len() as u8);
                buffer.extend_from_slice(bytes);
            } else {
                buffer.push(ValueType::String16 as u8);
                buffer.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
                buffer.extend_from_slice(bytes);
            }
        }
        Value::Array(arr) => {
            if arr.len() <= 255 {
                buffer.push(ValueType::List as u8);
                buffer.push(arr.len() as u8);
                for item in arr {
                    encode_value(item, buffer);
                }
            } else {
                // For long arrays, truncate to 255 items
                buffer.push(ValueType::List as u8);
                buffer.push(255);
                for item in arr.iter().take(255) {
                    encode_value(item, buffer);
                }
            }
        }
        Value::Object(obj) => {
            // For objects, encode as a list of key-value pairs
            if obj.len() <= 255 {
                buffer.push(ValueType::Map as u8);
                buffer.push(obj.len() as u8);
                for (key, value) in obj {
                    // Encode key as a string
                    let key_bytes = key.as_bytes();
                    if key_bytes.len() <= 255 {
                        buffer.push(ValueType::String8 as u8);
                        buffer.push(key_bytes.len() as u8);
                        buffer.extend_from_slice(key_bytes);
                    } else {
                        buffer.push(ValueType::String16 as u8);
                        buffer.extend_from_slice(&(key_bytes.len() as u16).to_le_bytes());
                        buffer.extend_from_slice(key_bytes);
                    }
                    
                    // Encode value
                    encode_value(value, buffer);
                }
            } else {
                // For large objects, truncate to 255 entries
                buffer.push(ValueType::Map as u8);
                buffer.push(255);
                for (key, value) in obj.iter().take(255) {
                    // Encode key as a string
                    let key_bytes = key.as_bytes();
                    if key_bytes.len() <= 255 {
                        buffer.push(ValueType::String8 as u8);
                        buffer.push(key_bytes.len() as u8);
                        buffer.extend_from_slice(key_bytes);
                    } else {
                        buffer.push(ValueType::String16 as u8);
                        buffer.extend_from_slice(&(key_bytes.len() as u16).to_le_bytes());
                        buffer.extend_from_slice(key_bytes);
                    }
                    
                    // Encode value
                    encode_value(value, buffer);
                }
            }
        }
    }
}

// Zero-copy JSON parsing
pub fn parse_without_copy<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T> {
    serde_json::from_slice(data).context("Failed to parse JSON")
}

// Serialize metrics in custom binary format
pub fn serialize_metrics_binary(
    metrics: &[heapless::pool::singleton::PoolPtr<MetricEntry>],
    registry: &crate::telemetry::protocol::registry::FieldRegistry,
    compression_level: flate2::Compression,
    use_error_correction: bool,
) -> Vec<u8> {
    // Pre-allocate with estimated size (48 bytes per metric is a good start)
    let mut buffer = Vec::with_capacity(metrics.len() * 48 + 16);
    
    // Write header: 4-byte magic number + 4-byte version + 4-byte count + 4-byte flags
    buffer.extend_from_slice(PROTOCOL_MAGIC);
    buffer.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    buffer.extend_from_slice(&(metrics.len() as u32).to_le_bytes());
    
    // Flags: bit 0 = compression, bit 1 = error correction
    let mut flags: u32 = 0;
    // We'll set compression flag later if needed
    if use_error_correction {
        flags |= 0x02;
    }
    buffer.extend_from_slice(&flags.to_le_bytes());
    
    // Create metrics section
    let mut metrics_data = Vec::with_capacity(metrics.len() * 48);
    
    // Write each metric
    for metric in metrics {
        // UUID (16 bytes) - parse or hash the request ID
        let request_id_bytes = if let Ok(uuid) = Uuid::parse_str(&metric.request_id) {
            *uuid.as_bytes()
        } else {
            // Hash non-UUID request IDs
            let mut hasher = fnv::FnvHasher::default();
            std::hash::Hasher::write(&mut hasher, metric.request_id.as_bytes());
            let hash = hasher.finish();
            let mut bytes = [0u8; 16];
            bytes[0..8].copy_from_slice(&hash.to_le_bytes());
            bytes
        };
        metrics_data.extend_from_slice(&request_id_bytes);
        
        // Timestamp (8 bytes) - milliseconds since epoch
        let timestamp_ms = metric.timestamp.timestamp_millis();
        metrics_data.extend_from_slice(&timestamp_ms.to_le_bytes());
        
        // Get common fields first
        let obj = metric.metrics.as_object().unwrap();
        
        // Memory usage (4 bytes - as float)
        let memory_mb = obj.get("memory_usage_mb")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        metrics_data.extend_from_slice(&(memory_mb as f32).to_le_bytes());
        
        // CPU usage (1 byte - percentage scaled 0-255)
        let cpu_pct = (obj.get("cpu_usage_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) * 2.55) as u8; // Scale 0-100% to 0-255
        metrics_data.push(cpu_pct);
        
        // Duration (4 bytes - as u32 milliseconds)
        let duration_ms = obj.get("duration_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        metrics_data.extend_from_slice(&duration_ms.to_le_bytes());
        
        // Additional fields
        let extra_fields: Vec<_> = obj.iter()
            .filter(|(k, _)| !matches!(k.as_str(), 
                "memory_usage_mb" | "cpu_usage_percent" | "duration_ms"))
            .collect();
        
        // Field count (1 byte)
        metrics_data.push(extra_fields.len() as u8);
        
        // Write each additional field
        for (key, value) in extra_fields {
            // Field ID (1 byte)
            let field_id = registry.get_id_for_field(key);
            metrics_data.push(field_id);
            
            // Field value - type and content
            encode_value(value, &mut metrics_data);
        }
    }
    
    // Check if compression would be beneficial
    let use_compression = metrics_data.len() > PROTOCOL_MIN_COMPRESSION_SIZE;
    if use_compression {
        // Set compression flag
        flags |= 0x01;
        buffer[12..16].copy_from_slice(&flags.to_le_bytes());
        
        // Compress data
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), compression_level);
        encoder.write_all(&metrics_data).unwrap();
        let compressed_data = encoder.finish().unwrap();
        
        // Add compressed data size
        buffer.extend_from_slice(&(metrics_data.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&(compressed_data.len() as u32).to_le_bytes());
        
        // Add compressed data
        buffer.extend_from_slice(&compressed_data);
    } else {
        // Add data directly
        buffer.extend_from_slice(&metrics_data);
    }
    
    // Add error correction code if enabled
    if use_error_correction {
        let crc = crate::common::utils::error_correction::compute_crc32(&buffer);
        buffer.extend_from_slice(&crc.to_le_bytes());
    }
    
    buffer
}
