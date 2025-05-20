// agent/core/lib/telemetry/protocol/compact.rs

//! Compact structures and bitpacking for telemetry data
//!
//! This module implements space-efficient data structures
//! for telemetry data, dramatically reducing memory footprint
//! and network payload size.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// =======================
// Compact field values
// =======================

/// Memory-efficient field value representation using bitpacking
#[derive(Debug, Clone)]
pub enum CompactValue {
    /// Boolean value (1 bit internally)
    Boolean(bool),

    /// Small integer in range 0-15 (4 bits internally)
    SmallInt(u8),

    /// Medium integer in range 0-255 (8 bits internally)
    MediumInt(u8),

    /// Integer 0-1023 (10 bits)
    LargeInt(u16),

    /// Integer 0-65535 (16 bits)
    Integer16(u16),

    /// Integer 0-4294967295 (32 bits)
    Integer32(u32),

    /// Integer 0-2^64-1 (64 bits)
    Integer64(u64),

    /// String (length-prefixed)
    String(String),

    /// Duration (48 bits total - 32 for seconds, 16 for millis)
    Duration(Duration),

    /// Timestamp (48 bits)
    Timestamp(DateTime<Utc>),

    /// Hash map of compact values
    Map(HashMap<u16, CompactValue>),

    /// Fixed-size array of compact values
    Array(Vec<CompactValue>),
}

/// Type codes for bitpacked values
#[repr(u8)]
enum CompactValueType {
    Boolean = 0,
    SmallInt = 1,
    MediumInt = 2,
    LargeInt = 3,
    Integer16 = 4,
    Integer32 = 5,
    Integer64 = 6,
    String = 7,
    Duration = 8,
    Timestamp = 9,
    Map = 10,
    Array = 11,
}

impl CompactValue {
    /// Serialize to a byte vector with bitpacking
    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.write_to(&mut buffer)?;
        Ok(buffer)
    }

    /// Deserialize from bytes
    pub fn deserialize(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = std::io::Cursor::new(bytes);
        Self::read_from(&mut cursor)
    }

    /// Write to a buffer
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            CompactValue::Boolean(val) => {
                writer.write_u8(CompactValueType::Boolean as u8)?;
                writer.write_u8(if *val { 1 } else { 0 })?;
            }
            CompactValue::SmallInt(val) => {
                writer.write_u8(CompactValueType::SmallInt as u8)?;
                writer.write_u8(*val)?;
            }
            CompactValue::MediumInt(val) => {
                writer.write_u8(CompactValueType::MediumInt as u8)?;
                writer.write_u8(*val)?;
            }
            CompactValue::LargeInt(val) => {
                writer.write_u8(CompactValueType::LargeInt as u8)?;
                writer.write_u16::<LittleEndian>(*val)?;
            }
            CompactValue::Integer16(val) => {
                writer.write_u8(CompactValueType::Integer16 as u8)?;
                writer.write_u16::<LittleEndian>(*val)?;
            }
            CompactValue::Integer32(val) => {
                writer.write_u8(CompactValueType::Integer32 as u8)?;
                writer.write_u32::<LittleEndian>(*val)?;
            }
            CompactValue::Integer64(val) => {
                writer.write_u8(CompactValueType::Integer64 as u8)?;
                writer.write_u64::<LittleEndian>(*val)?;
            }
            CompactValue::String(val) => {
                writer.write_u8(CompactValueType::String as u8)?;
                let bytes = val.as_bytes();
                if bytes.len() <= 255 {
                    writer.write_u8(1)?; // 1-byte length
                    writer.write_u8(bytes.len() as u8)?;
                } else if bytes.len() <= 65535 {
                    writer.write_u8(2)?; // 2-byte length
                    writer.write_u16::<LittleEndian>(bytes.len() as u16)?;
                } else {
                    writer.write_u8(4)?; // 4-byte length
                    writer.write_u32::<LittleEndian>(bytes.len() as u32)?;
                }
                writer.write_all(bytes)?;
            }
            CompactValue::Duration(val) => {
                writer.write_u8(CompactValueType::Duration as u8)?;
                let secs = val.as_secs();
                let millis = val.subsec_millis();
                writer.write_u32::<LittleEndian>(secs as u32)?;
                writer.write_u16::<LittleEndian>(millis)?;
            }
            CompactValue::Timestamp(val) => {
                writer.write_u8(CompactValueType::Timestamp as u8)?;
                let timestamp = val.timestamp();
                let millis = val.timestamp_subsec_millis();
                writer.write_u32::<LittleEndian>(timestamp as u32)?;
                writer.write_u16::<LittleEndian>(millis)?;
            }
            CompactValue::Map(val) => {
                writer.write_u8(CompactValueType::Map as u8)?;
                writer.write_u16::<LittleEndian>(val.len() as u16)?;
                for (key, value) in val {
                    writer.write_u16::<LittleEndian>(*key)?;
                    value.write_to(writer)?;
                }
            }
            CompactValue::Array(val) => {
                writer.write_u8(CompactValueType::Array as u8)?;
                writer.write_u16::<LittleEndian>(val.len() as u16)?;
                for value in val {
                    value.write_to(writer)?;
                }
            }
        }

        Ok(())
    }

    /// Read from a buffer
    fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let type_byte = reader.read_u8()?;

        match type_byte {
            b if b == CompactValueType::Boolean as u8 => {
                let val = reader.read_u8()? != 0;
                Ok(CompactValue::Boolean(val))
            }
            b if b == CompactValueType::SmallInt as u8 => {
                let val = reader.read_u8()?;
                Ok(CompactValue::SmallInt(val))
            }
            b if b == CompactValueType::MediumInt as u8 => {
                let val = reader.read_u8()?;
                Ok(CompactValue::MediumInt(val))
            }
            b if b == CompactValueType::LargeInt as u8 => {
                let val = reader.read_u16::<LittleEndian>()?;
                Ok(CompactValue::LargeInt(val))
            }
            b if b == CompactValueType::Integer16 as u8 => {
                let val = reader.read_u16::<LittleEndian>()?;
                Ok(CompactValue::Integer16(val))
            }
            b if b == CompactValueType::Integer32 as u8 => {
                let val = reader.read_u32::<LittleEndian>()?;
                Ok(CompactValue::Integer32(val))
            }
            b if b == CompactValueType::Integer64 as u8 => {
                let val = reader.read_u64::<LittleEndian>()?;
                Ok(CompactValue::Integer64(val))
            }
            b if b == CompactValueType::String as u8 => {
                let length_type = reader.read_u8()?;
                let length = match length_type {
                    1 => reader.read_u8()? as usize,
                    2 => reader.read_u16::<LittleEndian>()? as usize,
                    4 => reader.read_u32::<LittleEndian>()? as usize,
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid string length type",
                        ))
                    }
                };

                let mut bytes = vec![0u8; length];
                reader.read_exact(&mut bytes)?;

                let string = String::from_utf8(bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

                Ok(CompactValue::String(string))
            }
            b if b == CompactValueType::Duration as u8 => {
                let secs = reader.read_u32::<LittleEndian>()? as u64;
                let millis = reader.read_u16::<LittleEndian>()? as u32;
                let duration = Duration::from_secs(secs) + Duration::from_millis(millis as u64);
                Ok(CompactValue::Duration(duration))
            }
            b if b == CompactValueType::Timestamp as u8 => {
                let timestamp = reader.read_u32::<LittleEndian>()? as i64;
                let millis = reader.read_u16::<LittleEndian>()? as u32;

                let datetime = DateTime::<Utc>::from_timestamp(timestamp, millis * 1_000_000)
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData, "Invalid timestamp")
                    })?;

                Ok(CompactValue::Timestamp(datetime))
            }
            b if b == CompactValueType::Map as u8 => {
                let count = reader.read_u16::<LittleEndian>()? as usize;
                let mut map = HashMap::with_capacity(count);

                for _ in 0..count {
                    let key = reader.read_u16::<LittleEndian>()?;
                    let value = Self::read_from(reader)?;
                    map.insert(key, value);
                }

                Ok(CompactValue::Map(map))
            }
            b if b == CompactValueType::Array as u8 => {
                let count = reader.read_u16::<LittleEndian>()? as usize;
                let mut array = Vec::with_capacity(count);

                for _ in 0..count {
                    let value = Self::read_from(reader)?;
                    array.push(value);
                }

                Ok(CompactValue::Array(array))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown value type",
            )),
        }
    }
}

// =======================
// Compact metrics record
// =======================

/// Ultra-compact metrics record structure
#[derive(Debug, Clone)]
pub struct CompactMetricsRecord {
    /// Request ID as hash
    pub request_id_hash: u64,

    /// Timestamp
    pub timestamp: u64,

    /// Milliseconds since epoch, truncated
    pub bitpacked_data: Vec<u64>,

    /// Field mapping (field ID -> data offset)
    pub field_mapping: HashMap<u16, (u16, u8)>,
}

impl CompactMetricsRecord {
    /// Create a new empty record
    pub fn new(request_id: &str, timestamp: DateTime<Utc>) -> Self {
        // Hash the request ID to save space
        let mut hasher = fnv::FnvHasher::default();
        std::hash::Hasher::write(&mut hasher, request_id.as_bytes());
        let request_id_hash = hasher.finish();

        Self {
            request_id_hash,
            timestamp: timestamp.timestamp_millis() as u64,
            bitpacked_data: Vec::new(),
            field_mapping: HashMap::new(),
        }
    }

    /// Add a boolean field (consumes just 1 bit plus overhead)
    pub fn add_bool_field(&mut self, field_id: u16, value: bool) {
        let (block_idx, bit_offset) = self.allocate_bits(1);

        if value {
            // Set the bit
            let block = &mut self.bitpacked_data[block_idx as usize];
            *block |= 1u64 << bit_offset;
        }

        // Record the field mapping
        self.field_mapping.insert(field_id, (block_idx, bit_offset));
    }

    /// Add a small integer field (4 bits, range 0-15)
    pub fn add_small_int_field(&mut self, field_id: u16, value: u8) {
        let value = value & 0x0F; // Ensure it fits in 4 bits
        let (block_idx, bit_offset) = self.allocate_bits(4);

        // Set the bits
        let block = &mut self.bitpacked_data[block_idx as usize];
        *block |= (value as u64) << bit_offset;

        // Record the field mapping
        self.field_mapping.insert(field_id, (block_idx, bit_offset));
    }

    /// Add a medium integer field (8 bits, range 0-255)
    pub fn add_medium_int_field(&mut self, field_id: u16, value: u8) {
        let (block_idx, bit_offset) = self.allocate_bits(8);

        // Set the bits
        let block = &mut self.bitpacked_data[block_idx as usize];
        *block |= (value as u64) << bit_offset;

        // Record the field mapping
        self.field_mapping.insert(field_id, (block_idx, bit_offset));
    }

    /// Add a large integer field (16 bits, range 0-65535)
    pub fn add_large_int_field(&mut self, field_id: u16, value: u16) {
        let (block_idx, bit_offset) = self.allocate_bits(16);

        // Set the bits
        let block = &mut self.bitpacked_data[block_idx as usize];
        *block |= (value as u64) << bit_offset;

        // Record the field mapping
        self.field_mapping.insert(field_id, (block_idx, bit_offset));
    }

    /// Get a boolean field
    pub fn get_bool_field(&self, field_id: u16) -> Option<bool> {
        let &(block_idx, bit_offset) = self.field_mapping.get(&field_id)?;

        let block = self.bitpacked_data[block_idx as usize];
        let bit = (block >> bit_offset) & 1;

        Some(bit != 0)
    }

    /// Get a small integer field
    pub fn get_small_int_field(&self, field_id: u16) -> Option<u8> {
        let &(block_idx, bit_offset) = self.field_mapping.get(&field_id)?;

        let block = self.bitpacked_data[block_idx as usize];
        let value = ((block >> bit_offset) & 0x0F) as u8;

        Some(value)
    }

    /// Get a medium integer field
    pub fn get_medium_int_field(&self, field_id: u16) -> Option<u8> {
        let &(block_idx, bit_offset) = self.field_mapping.get(&field_id)?;

        let block = self.bitpacked_data[block_idx as usize];
        let value = ((block >> bit_offset) & 0xFF) as u8;

        Some(value)
    }

    /// Get a large integer field
    pub fn get_large_int_field(&self, field_id: u16) -> Option<u16> {
        let &(block_idx, bit_offset) = self.field_mapping.get(&field_id)?;

        let block = self.bitpacked_data[block_idx as usize];
        let value = ((block >> bit_offset) & 0xFFFF) as u16;

        Some(value)
    }

    /// Allocate bits in the bitpacked data
    /// Returns (block_index, bit_offset)
    fn allocate_bits(&mut self, bit_count: u8) -> (u16, u8) {
        if self.bitpacked_data.is_empty() {
            // First allocation
            self.bitpacked_data.push(0);
            return (0, 0);
        }

        // Find space in the existing blocks
        let last_block_idx = self.bitpacked_data.len() - 1;
        let last_block = &mut self.bitpacked_data[last_block_idx];

        // Find the highest bit set in the last block
        let mut highest_bit = 0;
        for bit in (0..64).rev() {
            if *last_block & (1 << bit) != 0 {
                highest_bit = bit + 1;
                break;
            }
        }

        // Check if the new field fits in the current block
        if highest_bit + bit_count as u64 <= 64 {
            // It fits in the current block
            return (last_block_idx as u16, highest_bit as u8);
        }

        // Need a new block
        self.bitpacked_data.push(0);
        (self.bitpacked_data.len() as u16 - 1, 0)
    }

    /// Serialize to bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(
            8 + // request_id_hash
            8 + // timestamp
            4 + // bitpacked_data length
            self.bitpacked_data.len() * 8 + // bitpacked data
            4 + // field_mapping length
            self.field_mapping.len() * 4, // field mappings
        );

        // Write request ID hash
        buffer.extend_from_slice(&self.request_id_hash.to_le_bytes());

        // Write timestamp
        buffer.extend_from_slice(&self.timestamp.to_le_bytes());

        // Write bitpacked data
        buffer.extend_from_slice(&(self.bitpacked_data.len() as u32).to_le_bytes());
        for block in &self.bitpacked_data {
            buffer.extend_from_slice(&block.to_le_bytes());
        }

        // Write field mapping
        buffer.extend_from_slice(&(self.field_mapping.len() as u32).to_le_bytes());
        for (&field_id, &(block_idx, bit_offset)) in &self.field_mapping {
            buffer.extend_from_slice(&field_id.to_le_bytes());
            buffer.extend_from_slice(&block_idx.to_le_bytes());
            buffer.push(bit_offset);
        }

        buffer
    }

    /// Deserialize from bytes
    pub fn deserialize(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 20 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Data too short"));
        }

        let mut cursor = std::io::Cursor::new(bytes);

        // Read request ID hash
        let request_id_hash = cursor.read_u64::<LittleEndian>()?;

        // Read timestamp
        let timestamp = cursor.read_u64::<LittleEndian>()?;

        // Read bitpacked data
        let data_len = cursor.read_u32::<LittleEndian>()? as usize;
        let mut bitpacked_data = Vec::with_capacity(data_len);
        for _ in 0..data_len {
            bitpacked_data.push(cursor.read_u64::<LittleEndian>()?);
        }

        // Read field mapping
        let mapping_len = cursor.read_u32::<LittleEndian>()? as usize;
        let mut field_mapping = HashMap::with_capacity(mapping_len);
        for _ in 0..mapping_len {
            let field_id = cursor.read_u16::<LittleEndian>()?;
            let block_idx = cursor.read_u16::<LittleEndian>()?;
            let bit_offset = cursor.read_u8()?;
            field_mapping.insert(field_id, (block_idx, bit_offset));
        }

        Ok(Self {
            request_id_hash,
            timestamp,
            bitpacked_data,
            field_mapping,
        })
    }

    /// Get estimated size in bytes
    pub fn size_bytes(&self) -> usize {
        8 + // request_id_hash
        8 + // timestamp
        4 + // bitpacked_data length
        self.bitpacked_data.len() * 8 + // bitpacked data
        4 + // field_mapping length
        self.field_mapping.len() * 5 // field mappings (2 + 2 + 1 bytes each)
    }
}
