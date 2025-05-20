// agent/common/utils/error_correction.rs

use anyhow::{Context, Result};
use crc32fast::Hasher;
use std::io::{self, Read, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

// Protocol constant
pub const PROTOCOL_ERROR_CORRECTION_ENABLED: bool = true;

// Function to compute CRC32 for error correction
pub fn compute_crc32(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

// Validate data with CRC32
pub fn validate_crc32(data: &[u8], expected_crc: u32) -> bool {
    let computed_crc = compute_crc32(data);
    computed_crc == expected_crc
}

// Add CRC32 checksum to a buffer
pub fn add_crc32_checksum(data: &[u8]) -> Vec<u8> {
    let crc = compute_crc32(data);
    
    let mut result = Vec::with_capacity(data.len() + 4);
    result.extend_from_slice(data);
    result.extend_from_slice(&crc.to_le_bytes());
    
    result
}

// Validate and extract data from a buffer with CRC32 checksum
pub fn validate_and_extract_data(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 4 {
        return Err(anyhow::anyhow!("Data too short to contain CRC32 checksum"));
    }
    
    let (content, crc_bytes) = data.split_at(data.len() - 4);
    let expected_crc = u32::from_le_bytes([crc_bytes[0], crc_bytes[1], crc_bytes[2], crc_bytes[3]]);
    
    if validate_crc32(content, expected_crc) {
        Ok(content.to_vec())
    } else {
        Err(anyhow::anyhow!("CRC32 validation failed"))
    }
}

// Error correction code structure
#[derive(Debug, Clone)]
pub struct ErrorCorrectionCode {
    crc32: u32,
    data_length: u32,
    checksum_offset: u32,
}

impl ErrorCorrectionCode {
    // Create a new ECC for data
    pub fn new(data: &[u8]) -> Self {
        Self {
            crc32: compute_crc32(data),
            data_length: data.len() as u32,
            checksum_offset: 0, // Default, can be set if needed
        }
    }
    
    // Serialize the ECC
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(12);
        buffer.write_u32::<LittleEndian>(self.crc32).unwrap();
        buffer.write_u32::<LittleEndian>(self.data_length).unwrap();
        buffer.write_u32::<LittleEndian>(self.checksum_offset).unwrap();
        buffer
    }
    
    // Deserialize from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            return Err(anyhow::anyhow!("Data too short for ECC"));
        }
        
        let mut reader = io::Cursor::new(data);
        let crc32 = reader.read_u32::<LittleEndian>()?;
        let data_length = reader.read_u32::<LittleEndian>()?;
        let checksum_offset = reader.read_u32::<LittleEndian>()?;
        
        Ok(Self {
            crc32,
            data_length,
            checksum_offset,
        })
    }
    
    // Validate data against this ECC
    pub fn validate(&self, data: &[u8]) -> bool {
        if data.len() as u32 != self.data_length {
            return false;
        }
        
        compute_crc32(data) == self.crc32
    }
}

// Apply error correction to potentially corrupted data
// This is a simple implementation that just validates, but could be extended
// to perform actual correction for specific types of errors
pub fn apply_error_correction(data: &[u8], ecc: &ErrorCorrectionCode) -> Result<Vec<u8>> {
    if ecc.validate(data) {
        Ok(data.to_vec())
    } else {
        // In a more sophisticated implementation, we might try to correct errors here
        Err(anyhow::anyhow!("Data corruption detected, unable to correct"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_crc32_computation() {
        let data = b"test data for CRC32";
        let crc = compute_crc32(data);
        
        // CRC32 should be consistent
        assert_eq!(crc, compute_crc32(data));
        
        // Different data should have different CRC
        assert_ne!(crc, compute_crc32(b"different test data"));
    }
    
    #[test]
    fn test_crc32_validation() {
        let data = b"test data for validation";
        let crc = compute_crc32(data);
        
        assert!(validate_crc32(data, crc));
        assert!(!validate_crc32(data, crc + 1));
    }
    
    #[test]
    fn test_add_and_validate_checksum() {
        let original = b"test data with checksum";
        let with_checksum = add_crc32_checksum(original);
        
        // Result should be 4 bytes longer
        assert_eq!(with_checksum.len(), original.len() + 4);
        
        // Should validate correctly
        let extracted = validate_and_extract_data(&with_checksum).unwrap();
        assert_eq!(extracted, original);
        
        // Corrupting the data should fail validation
        let mut corrupted = with_checksum.clone();
        corrupted[0] = corrupted[0].wrapping_add(1);
        assert!(validate_and_extract_data(&corrupted).is_err());
    }
    
    #[test]
    fn test_error_correction_code() {
        let data = b"test data for error correction code";
        let ecc = ErrorCorrectionCode::new(data);
        
        // Should validate original data
        assert!(ecc.validate(data));
        
        // Serialization/deserialization should work
        let serialized = ecc.serialize();
        let deserialized = ErrorCorrectionCode::deserialize(&serialized).unwrap();
        
        assert_eq!(deserialized.crc32, ecc.crc32);
        assert_eq!(deserialized.data_length, ecc.data_length);
        
        // Modified data should fail validation
        let mut modified = data.to_vec();
        modified[0] = modified[0].wrapping_add(1);
        assert!(!ecc.validate(&modified));
    }
}
